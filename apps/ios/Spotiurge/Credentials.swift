// Protected storage and Spotify's PKCE sign-in for the developmental app.
// The flow follows the desktop's src/auth.rs. The playback probe proved the
// loopback redirect on a device, but its code was a research scaffold; this
// module adds the safeguards a product needs:
//
// - randomness fails closed;
// - Keychain errors are reported, never mistaken for "no credential", and an
//   existing credential is only replaced by a successful update;
// - the loopback listener lives for a bounded time, answers a bounded number
//   of requests, and accepts only `GET /login` with the exact state;
// - no URL, grant, response body or system error text is logged or shown.

import AuthenticationServices
import CryptoKit
import Foundation
import Network
import Security
import UIKit

struct KeychainError: Error, Equatable {
    let status: OSStatus
}

nonisolated enum Keychain {
    static let service = "com.sergeserbinenko.spotiurge"

    /// The reusable librespot credential for playback on this iPhone.
    static let playback = "librespot-reusable"
    /// The Spotify Web API refresh grant (library, search, catalogue matching).
    static let webRefresh = "spotify-web-refresh"
    /// Private-cloud device tokens are bound to their HTTPS origin.
    static func cloud(_ origin: String) -> String { "cloud:\(origin)" }

    private static func query(_ account: String) -> [String: Any] {
        [kSecClass as String: kSecClassGenericPassword, kSecAttrService as String: service, kSecAttrAccount as String: account]
    }

    /// `nil` only when no item exists. A locked or failing store throws, so
    /// callers never treat an unreadable credential as a missing one.
    static func read(_ account: String) throws(KeychainError) -> Data? {
        var query = query(account)
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne
        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        switch status {
        case errSecSuccess: return item as? Data
        case errSecItemNotFound: return nil
        default: throw KeychainError(status: status)
        }
    }

    /// Whether an item exists, for display only: an unreadable store reads
    /// as absent here, so never use this to decide sign-in or pairing.
    static func has(_ account: String) -> Bool {
        (try? read(account)) != nil
    }

    /// Updates in place, or adds when absent, then reads back. The previous
    /// value is never deleted first, so a failed write leaves it working.
    static func write(_ data: Data, _ account: String) throws(KeychainError) {
        let attributes: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
        var status = SecItemUpdate(query(account) as CFDictionary, attributes as CFDictionary)
        if status == errSecItemNotFound {
            status = SecItemAdd(query(account).merging(attributes) { $1 } as CFDictionary, nil)
        }
        guard status == errSecSuccess else { throw KeychainError(status: status) }
        guard try read(account) == data else { throw KeychainError(status: errSecInternalError) }
    }

    static func delete(_ account: String) throws(KeychainError) {
        let status = SecItemDelete(query(account) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else { throw KeychainError(status: status) }
    }
}

/// The Keychain operations the models use, injectable so tests never touch
/// real secrets.
nonisolated struct SecretStore: Sendable {
    var read: @Sendable (_ account: String) throws(KeychainError) -> Data?
    var write: @Sendable (_ data: Data, _ account: String) throws(KeychainError) -> Void
    var delete: @Sendable (_ account: String) throws(KeychainError) -> Void

    static let keychain = SecretStore(read: Keychain.read, write: Keychain.write, delete: Keychain.delete)
}

struct SignInError: LocalizedError {
    let errorDescription: String?
    /// Spotify rejected the grant itself; only a new sign-in can replace it.
    var rejected = false
    init(_ message: String, rejected: Bool = false) {
        errorDescription = message
        self.rejected = rejected
    }

    static let cancelled = SignInError("Sign-in was cancelled.")
    static let failed = SignInError("Spotify sign-in did not complete. Try again.")
    static let unreachable = SignInError("Spotify is unreachable. Check the connection and try again.")
}

/// One OAuth identity, as in src/auth.rs `Grant`.
struct Grant {
    let clientID: String
    let port: UInt16
    let scopes: [String]
    var redirect: String { "http://127.0.0.1:\(port)/login" }

    /// Spotify's own client and the streaming scope librespot plays with.
    static let playback = Grant(clientID: "65b708073fc0480ea92a077233ca87bd", port: 8898, scopes: ["streaming"])
    /// The shared Web API app the desktop uses, asked only for reads.
    static let web = Grant(clientID: "d420a117a32841c2b3474932e49fb54b", port: 8989, scopes: [
        "user-read-private", "user-library-read", "playlist-read-private", "playlist-read-collaborative",
    ])
}

struct Tokens: Decodable {
    let access_token: String
    let expires_in: Int?
    let refresh_token: String?
}

/// PKCE against a loopback redirect. The system authentication sheet keeps
/// the app in the foreground, so a Network.framework listener on 127.0.0.1
/// receives the redirect. No browser engine is embedded.
final class PKCESignIn: NSObject, ASWebAuthenticationPresentationContextProviding {
    /// Matches the desktop's LOGIN_TIMEOUT.
    static let lifetime: Duration = .seconds(600)
    /// Favicon and stray requests are answered, up to this many.
    static let maxRequests = 8

    private let grant: Grant
    private var listener: NWListener?
    private var session: ASWebAuthenticationSession?
    private var continuation: CheckedContinuation<Tokens, Error>?
    private var deadline: Task<Void, Never>?
    private var requests = 0
    private var redirected = false

    init(_ grant: Grant) {
        self.grant = grant
    }

    nonisolated func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        MainActor.assumeIsolated {
            let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
            return scenes.compactMap(\.keyWindow).first ?? UIWindow(windowScene: scenes[0])
        }
    }

    func run() async throws -> Tokens {
        try await withCheckedThrowingContinuation { continuation in
            self.continuation = continuation
            start()
        }
    }

    private func start() {
        guard let verifier = Self.random(48), let state = Self.random(24) else {
            finish(.failure(SignInError("This iPhone could not generate a secure sign-in. Try again.")))
            return
        }
        let challenge = Data(SHA256.hash(data: Data(verifier.utf8))).base64URL
        do {
            let parameters = NWParameters.tcp
            parameters.requiredLocalEndpoint = .hostPort(host: "127.0.0.1", port: NWEndpoint.Port(rawValue: grant.port)!)
            parameters.allowLocalEndpointReuse = true
            let listener = try NWListener(using: parameters)
            listener.newConnectionHandler = { [weak self] connection in
                Task { @MainActor in self?.accept(connection, state: state, verifier: verifier) }
            }
            listener.start(queue: .main)
            self.listener = listener
        } catch {
            finish(.failure(SignInError("Another sign-in is using the sign-in port. Try again in a moment.")))
            return
        }
        var components = URLComponents(string: "https://accounts.spotify.com/authorize")!
        components.queryItems = [
            .init(name: "client_id", value: grant.clientID),
            .init(name: "response_type", value: "code"),
            .init(name: "redirect_uri", value: grant.redirect),
            .init(name: "code_challenge_method", value: "S256"),
            .init(name: "code_challenge", value: challenge),
            .init(name: "state", value: state),
            .init(name: "scope", value: grant.scopes.joined(separator: " ")),
        ] + (grant.clientID == Grant.playback.clientID ? [.init(name: "show_dialog", value: "true")] : [])
        // The callback scheme is never reached: the loopback listener answers
        // the redirect and then closes this sheet.
        let session = ASWebAuthenticationSession(url: components.url!, callbackURLScheme: "spotiurge") { [weak self] _, error in
            Task { @MainActor in
                guard let self, error != nil, !self.redirected else { return }
                let cancelled = (error as? ASWebAuthenticationSessionError)?.code == .canceledLogin
                self.finish(.failure(cancelled ? SignInError.cancelled : SignInError.failed))
            }
        }
        session.presentationContextProvider = self
        session.prefersEphemeralWebBrowserSession = false
        self.session = session
        deadline = Task { [weak self] in
            try? await Task.sleep(for: Self.lifetime)
            guard !Task.isCancelled else { return }
            self?.session?.cancel()
            self?.finish(.failure(SignInError("Sign-in timed out. Try again.")))
        }
        if !session.start() { finish(.failure(SignInError.failed)) }
    }

    private func accept(_ connection: NWConnection, state: String, verifier: String) {
        requests += 1
        guard continuation != nil, !redirected, requests <= Self.maxRequests else {
            connection.cancel()
            return
        }
        connection.start(queue: .main)
        connection.receive(minimumIncompleteLength: 1, maximumLength: 16_384) { [weak self] data, _, _, _ in
            // Only the request line matters: "GET /login?code=…&state=… HTTP/1.1".
            let line = data.flatMap { String(data: $0, encoding: .utf8) }?.split(separator: "\r\n").first.map(String.init) ?? ""
            let parts = line.split(separator: " ", omittingEmptySubsequences: false)
            let target = parts.count == 3 && parts[0] == "GET" ? URLComponents(string: "http://127.0.0.1\(parts[1])") : nil
            let isCallback = target?.path == "/login"
            let items = target?.queryItems ?? []
            let code = items.first { $0.name == "code" }?.value
            let validState = items.filter { $0.name == "state" }.map(\.value) == [state]
            let body = isCallback ? "Spotiurge: you can return to the app." : "Not found"
            let reply = "HTTP/1.1 \(isCallback ? "200 OK" : "404 Not Found")\r\ncontent-type: text/plain\r\ncontent-length: \(body.utf8.count)\r\ncache-control: no-store\r\nconnection: close\r\n\r\n\(body)"
            connection.send(content: Data(reply.utf8), completion: .contentProcessed { _ in connection.cancel() })
            Task { @MainActor in
                // Favicon and other paths do not end the flow.
                guard let self, isCallback, self.continuation != nil, !self.redirected else { return }
                guard let code, !code.isEmpty, validState else {
                    self.session?.cancel()
                    self.finish(.failure(SignInError("Spotify refused the sign-in, or it did not match this request.")))
                    return
                }
                self.redirected = true
                self.session?.cancel()
                do {
                    self.finish(.success(try await Self.token([
                        "grant_type": "authorization_code", "code": code, "redirect_uri": self.grant.redirect,
                        "client_id": self.grant.clientID, "code_verifier": verifier,
                    ])))
                } catch {
                    self.finish(.failure(error))
                }
            }
        }
    }

    /// Exchanges a code or refresh grant. Only the status decides the
    /// message; the response body is never logged or shown.
    static func token(_ form: [String: String]) async throws -> Tokens {
        var request = URLRequest(url: URL(string: "https://accounts.spotify.com/api/token")!, timeoutInterval: 20)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        var body = URLComponents()
        body.queryItems = form.map { URLQueryItem(name: $0.key, value: $0.value) }
        request.httpBody = Data((body.percentEncodedQuery ?? "").replacingOccurrences(of: "+", with: "%2B").utf8)
        let data: Data, response: URLResponse
        do { (data, response) = try await URLSession.shared.data(for: request) } catch { throw SignInError.unreachable }
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        guard status == 200, let tokens = try? JSONDecoder().decode(Tokens.self, from: data) else {
            if status == 400 || status == 401 { throw SignInError("Spotify no longer accepts this sign-in. Sign in again.", rejected: true) }
            throw status >= 500 ? SignInError.unreachable : SignInError.failed
        }
        return tokens
    }

    private func finish(_ result: Result<Tokens, Error>) {
        deadline?.cancel()
        deadline = nil
        listener?.cancel()
        listener = nil
        session = nil
        let continuation = continuation
        self.continuation = nil
        continuation?.resume(with: result)
    }

    /// Base64url of `count` random bytes, or nil if the system RNG fails.
    private static func random(_ count: Int) -> String? {
        var bytes = [UInt8](repeating: 0, count: count)
        guard SecRandomCopyBytes(kSecRandomDefault, count, &bytes) == errSecSuccess else { return nil }
        return Data(bytes).base64URL
    }
}

extension Data {
    var base64URL: String {
        base64EncodedString().replacingOccurrences(of: "+", with: "-").replacingOccurrences(of: "/", with: "_").replacingOccurrences(of: "=", with: "")
    }
}
