import AuthenticationServices
import CryptoKit
import Foundation
import Network
import Security

/// Spotify's streaming-scope sign-in, the same grant the desktop's local
/// playback asks for (src/auth.rs): PKCE against Spotify's own client ID
/// with its registered loopback redirect. The system authentication sheet
/// keeps the app in the foreground, so the loopback listener receives the
/// redirect. No browser engine is embedded.
@MainActor
final class SignIn: NSObject, ASWebAuthenticationPresentationContextProviding {
    static let clientID = "65b708073fc0480ea92a077233ca87bd"
    static let redirect = "http://127.0.0.1:8898/login"

    private var listener: NWListener?
    private var session: ASWebAuthenticationSession?
    private var finish: ((Result<String, Error>) -> Void)?
    private var exchangeTask: Task<Void, Never>?
    /// Set once the redirect arrived, so closing the sheet is not a failure.
    private var redirected = false
    private let anchor: ASPresentationAnchor

    init(anchor: ASPresentationAnchor) {
        self.anchor = anchor
    }

    nonisolated func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        MainActor.assumeIsolated { anchor }
    }

    /// Calls `completion` with an access token. The token is never logged.
    func start(completion: @escaping (Result<String, Error>) -> Void) {
        cancel()
        finish = completion
        let verifier: String
        do { verifier = try SecureVerifier.make() }
        catch { done(.failure(error)); return }
        let challenge = Data(SHA256.hash(data: Data(verifier.utf8))).base64URL
        let state = UUID().uuidString
        redirected = false

        do {
            let parameters = NWParameters.tcp
            parameters.requiredLocalEndpoint = .hostPort(host: "127.0.0.1", port: 8898)
            parameters.allowLocalEndpointReuse = true
            let listener = try NWListener(using: parameters)
            listener.newConnectionHandler = { [weak self] connection in
                Task { @MainActor in self?.accept(connection, state: state, verifier: verifier) }
            }
            listener.start(queue: .main)
            self.listener = listener
        } catch {
            done(.failure(error))
            return
        }

        var components = URLComponents(string: "https://accounts.spotify.com/authorize")!
        components.queryItems = [
            .init(name: "client_id", value: Self.clientID),
            .init(name: "response_type", value: "code"),
            .init(name: "redirect_uri", value: Self.redirect),
            .init(name: "code_challenge_method", value: "S256"),
            .init(name: "code_challenge", value: challenge),
            .init(name: "state", value: state),
            .init(name: "scope", value: "streaming"),
            .init(name: "show_dialog", value: "true"),
        ]
        // The callback scheme is never reached: the redirect is loopback HTTP,
        // answered by the listener above, which then closes this sheet.
        let session = ASWebAuthenticationSession(url: components.url!, callbackURLScheme: "spotiurge-probe") {
            [weak self] _, error in
            Task { @MainActor in
                guard let self, let error, self.finish != nil, !self.redirected else { return }
                self.done(.failure(error))
            }
        }
        session.presentationContextProvider = self
        self.session = session
        if !session.start() { done(.failure(ProbeError("could not open Spotify sign-in"))) }
    }

    private func accept(_ connection: NWConnection, state: String, verifier: String) {
        connection.start(queue: .main)
        read(connection, request: LoopbackRequest(), state: state, verifier: verifier)
    }

    private func read(_ connection: NWConnection, request: LoopbackRequest, state: String, verifier: String) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 16_384) { [weak self] data, _, complete, error in
            Task { @MainActor in
                guard let self, self.finish != nil, !self.redirected else { connection.cancel(); return }
                var request = request
                do {
                    let line = try request.append(data ?? Data())
                    guard let line else {
                        if complete || error != nil { connection.cancel() }
                        else { self.read(connection, request: request, state: state, verifier: verifier) }
                        return
                    }
                    let redirect = LoopbackRequest.redirect(line, state: state)
                    let body = "Spotiurge Probe: you can return to the app."
                    let status = redirect == .stray ? "404 Not Found" : "200 OK"
                    let reply = "HTTP/1.1 \(status)\r\ncontent-type: text/plain\r\ncontent-length: \(body.utf8.count)\r\nconnection: close\r\n\r\n\(body)"
                    connection.send(content: Data(reply.utf8), completion: .contentProcessed { _ in connection.cancel() })
                    switch redirect {
                    case .stray: return
                    case .refused: self.done(.failure(ProbeError("sign-in was refused")))
                    case .code(let code):
                        self.redirected = true
                        self.session?.cancel()
                        self.exchange(code: code, verifier: verifier)
                    }
                } catch {
                    connection.cancel()
                }
            }
        }
    }

    private func exchange(code: String, verifier: String) {
        var request = URLRequest(url: URL(string: "https://accounts.spotify.com/api/token")!)
        request.httpMethod = "POST"
        request.setValue("application/x-www-form-urlencoded", forHTTPHeaderField: "Content-Type")
        var form = URLComponents()
        form.queryItems = [
            .init(name: "grant_type", value: "authorization_code"),
            .init(name: "code", value: code),
            .init(name: "redirect_uri", value: Self.redirect),
            .init(name: "client_id", value: Self.clientID),
            .init(name: "code_verifier", value: verifier),
        ]
        request.httpBody = Data((form.percentEncodedQuery ?? "").utf8)
        exchangeTask = Task {
            do {
                let (data, response) = try await URLSession.shared.data(for: request)
                try Task.checkCancellation()
                let status = (response as? HTTPURLResponse)?.statusCode ?? 0
                guard status == 200,
                      let json = try JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let token = json["access_token"] as? String
                else { throw ProbeError("token exchange failed with HTTP \(status)") }
                done(.success(token))
            } catch {
                if !Task.isCancelled { done(.failure(error)) }
            }
        }
    }

    private func done(_ result: Result<String, Error>) {
        guard let finish else { return }
        cancel()
        finish(result)
    }

    /// Forget/replacement discards the completion before canceling work, so
    /// queued browser or token-exchange results cannot reconnect afterwards.
    func cancel() {
        finish = nil
        listener?.cancel()
        listener = nil
        session?.cancel()
        session = nil
        exchangeTask?.cancel()
        exchangeTask = nil
    }
}

struct ProbeError: LocalizedError {
    let errorDescription: String?
    init(_ message: String) { errorDescription = message }
}
