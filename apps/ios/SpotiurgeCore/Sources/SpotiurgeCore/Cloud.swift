// Bounded, authenticated private-cloud requests, the iPhone counterpart of
// src/discovery_cloud.rs. The token comes from a caller-supplied closure
// (the Keychain on iOS); it is never logged, stored here, or put in a URL.

import Foundation

public enum PrivateCloud {
    /// The deployed service from services/private-cloud (src/discovery_cloud.rs).
    public static let defaultEndpoint = "https://private-cloud-production.up.railway.app"
    public static let model = "gpt-6-luna"

    /// An HTTPS origin without credentials, path, query or fragment.
    public static func origin(_ endpoint: String) throws(DiscoveryError) -> URL {
        guard let components = URLComponents(string: endpoint.trimmingCharacters(in: .whitespaces)),
              components.scheme?.lowercased() == "https",
              let host = components.host, !host.isEmpty,
              components.user == nil, components.password == nil,
              components.query == nil, components.fragment == nil,
              components.path.isEmpty || components.path == "/"
        else { throw DiscoveryError("The private cloud must use an HTTPS origin without credentials or a path.") }
        var origin = URLComponents()
        origin.scheme = "https"
        origin.host = host.lowercased()
        origin.port = components.port == 443 ? nil : components.port
        guard let url = origin.url else { throw DiscoveryError("Configure the Spotiurge private cloud endpoint.") }
        return url
    }
}

public struct RecommendationFailure: Error, Equatable, Sendable {
    public let kind: RecommendationErrorKind
    public let message: String

    public init(_ kind: RecommendationErrorKind, _ message: String) {
        self.kind = kind
        self.message = message
    }

    /// The server reports why it refused in a bounded `code`; only known
    /// codes change the outcome, and no response text reaches logs or UI.
    public static func refused(status: Int, code: String?) -> RecommendationFailure {
        switch (status, code) {
        case (401, _), (403, _):
            .init(.pairing, "This iPhone is not paired with your private cloud. Pair it again in Settings.")
        case (429, "busy"):
            .init(.busy, "Your picks are already being prepared. Try again in a moment.")
        case (429, _):
            .init(.rateLimited, "The AI is rate limited. Your picks are kept; try again later.")
        case (400, _):
            .init(.unavailable, "Add a taste or rate a few tracks, then try again.")
        default:
            .init(.unavailable, "Recommendations are unavailable right now. Your picks are kept.")
        }
    }
}

public struct CloudClient: Sendable {
    public typealias TokenProvider = @Sendable (_ origin: String) async throws -> String

    let origin: URL
    let token: TokenProvider
    let session: URLSession

    public init(endpoint: String, session: URLSession = .shared, token: @escaping TokenProvider) throws(DiscoveryError) {
        origin = try PrivateCloud.origin(endpoint)
        self.session = session
        self.token = token
    }

    public var originString: String { origin.absoluteString }

    private func request(_ path: String, method: String, timeout: TimeInterval) async throws(DiscoveryError) -> URLRequest {
        let secret: String
        do {
            secret = try await token(originString)
        } catch {
            throw DiscoveryError("Pair this iPhone with your private cloud in Settings.")
        }
        var request = URLRequest(url: origin.appending(path: path), timeoutInterval: timeout)
        request.httpMethod = method
        request.setValue("Bearer \(secret)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.cachePolicy = .reloadIgnoringLocalCacheData
        return request
    }

    /// Never follows a redirect, so the bearer token cannot leave the origin.
    private func send(_ request: URLRequest, body: Data? = nil) async throws -> (Data, Int) {
        var request = request
        request.httpBody = body
        let (bytes, response) = try await session.bytes(for: request, delegate: NoRedirects.shared)
        var data = Data()
        for try await byte in bytes {
            data.append(byte)
            if data.count > DiscoveryLimits.maxBytes + 200 { throw DiscoveryError("Private cloud response is too large.") }
        }
        return (data, (response as? HTTPURLResponse)?.statusCode ?? 0)
    }

    private static func check(_ status: Int) throws(DiscoveryError) {
        switch status {
        case 200...299: return
        case 401, 403: throw DiscoveryError("Pair this iPhone again with your private cloud.")
        case 429: throw DiscoveryError("Requests are rate limited. Keep listening and try again later.")
        default: throw DiscoveryError("Private cloud is unavailable. Keep listening and try again later.")
        }
    }

    private struct Snapshot: Decodable {
        let revision: UInt64
        let document: Document
    }

    /// Fetch, merge, and write back under `If-Match`; a concurrent writer
    /// (409) makes it refetch and merge again, up to three times.
    public func sync(_ replica: Replica) async throws(DiscoveryError) -> Document {
        var replica = replica
        for _ in 0..<3 {
            let get = try await request("v1/state", method: "GET", timeout: 15)
            let (data, status): (Data, Int)
            do { (data, status) = try await send(get) } catch let error as DiscoveryError { throw error } catch {
                throw DiscoveryError("Sync is offline. Your local edits are kept.")
            }
            try Self.check(status)
            guard let remote = try? JSONDecoder().decode(Snapshot.self, from: data) else {
                throw DiscoveryError("Invalid private cloud response. Local state is preserved.")
            }
            try replica.mergeForSync(remote.document)
            var put = try await request("v1/state", method: "PUT", timeout: 15)
            put.setValue(String(remote.revision), forHTTPHeaderField: "If-Match")
            let body: Data
            do { body = try replica.document.encoded() } catch { throw DiscoveryError("Cannot encode discovery state.") }
            let written: Int
            do { (_, written) = try await send(put, body: body) } catch {
                throw DiscoveryError("Sync was interrupted. Your local edits are kept.")
            }
            if written == 409 { continue }
            try Self.check(written)
            return replica.document
        }
        throw DiscoveryError("Sync met concurrent edits. Retry to merge the latest state.")
    }

    private struct RecommendationRequest: Encodable {
        let taste: String
        let feedback: [Document.FeedbackEntry]
        let exploration: Exploration
    }

    /// Only the taste text and rated titles/artists leave the phone: no
    /// URIs, account identity, credentials or audio. GPT-6 Luna only.
    public func recommend(_ document: Document, exploration: Exploration) async throws(RecommendationFailure) -> [Suggestion] {
        do { try document.validate() } catch { throw RecommendationFailure(.unavailable, error.message) }
        let post: URLRequest
        do { post = try await request("v1/recommendations", method: "POST", timeout: 85) } catch {
            throw RecommendationFailure(.pairing, error.message)
        }
        let body = try? JSONEncoder().encode(RecommendationRequest(taste: document.taste, feedback: document.feedback, exploration: exploration))
        let (data, status): (Data, Int)
        do { (data, status) = try await send(post, body: body) } catch {
            throw RecommendationFailure(.unavailable, "Recommendations are unavailable. Your previous discoveries are kept.")
        }
        guard (200...299).contains(status) else {
            struct Refusal: Decodable { let code: String? }
            throw .refused(status: status, code: (try? JSONDecoder().decode(Refusal.self, from: data))?.code)
        }
        struct Answer: Decodable { let suggestions: [Suggestion] }
        guard let answer = try? JSONDecoder().decode(Answer.self, from: data), validSuggestions(answer.suggestions) else {
            throw RecommendationFailure(.invalidResponse, "The AI sent an answer Spotiurge could not use. Your picks are kept.")
        }
        return answer.suggestions
    }
}

final class NoRedirects: NSObject, URLSessionTaskDelegate, Sendable {
    static let shared = NoRedirects()

    func urlSession(_ session: URLSession, task: URLSessionTask, willPerformHTTPRedirection response: HTTPURLResponse, newRequest request: URLRequest) async -> URLRequest? {
        nil
    }
}
