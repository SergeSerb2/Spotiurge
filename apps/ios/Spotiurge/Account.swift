// The Spotify Web API grant: library, search and discovery matching.
// Separate from the playback credential, as on the desktop.

import Foundation
import Observation
import SpotiurgeCore

@Observable
final class SpotifyAccount {
    enum State: Equatable {
        case signedOut
        case signingIn
        case signedIn(name: String)
        case failed(String)
    }

    /// Fixed messages; no Keychain status or Spotify response text is shown.
    static let ended = "Spotify ended this sign-in. Sign in again."
    static let locked = "The Keychain is locked, so the Spotify sign-in cannot be read. Unlock this iPhone and return to Spotiurge."
    static let notRemoved = "The Keychain did not remove the Spotify sign-in, so it would return on the next launch. Unlock this iPhone and sign out again."

    private(set) var state: State = .signedOut
    private(set) var savedTracks: [Track] = []
    private(set) var savedTotal = 0
    private(set) var playlists: [Playlist] = []
    private(set) var libraryError: String?
    private(set) var loadingLibrary = false

    @ObservationIgnored private var accessToken: (value: String, expires: Date)?
    @ObservationIgnored private var refreshing: Task<String, Error>?
    /// Bumped by sign-out; an answer that started before it is dropped.
    @ObservationIgnored private var generation = 0
    /// Set while demo data shows: no new Spotify request starts.
    @ObservationIgnored private(set) var suspended = false

    @ObservationIgnored private let grant: SecretStore
    @ObservationIgnored private let authorize: () async throws -> Tokens
    @ObservationIgnored private let exchange: ([String: String]) async throws -> Tokens
    @ObservationIgnored private let session: URLSession

    init(grant: SecretStore = .keychain,
         authorize: @escaping () async throws -> Tokens = { try await PKCESignIn(.web).run() },
         exchange: @escaping ([String: String]) async throws -> Tokens = { try await PKCESignIn.token($0) },
         session: URLSession = .shared) {
        self.grant = grant
        self.authorize = authorize
        self.exchange = exchange
        self.session = session
    }

    var signedIn: Bool { if case .signedIn = state { true } else { false } }

    /// The client every Web API read goes through. The token never leaves
    /// this object except in an Authorization header.
    var web: SpotifyWeb {
        SpotifyWeb(session: session) { [self] in try await self.token() }
    }

    func restore() {
        switch Result(catching: { () throws(KeychainError) in try grant.read(Keychain.webRefresh) }) {
        case .success(nil): return
        case .success: Task { await afterSignIn() }
        // Not "signed out": the grant may be there. Retried on foreground.
        case .failure: state = .failed(Self.locked)
        }
    }

    /// Retries a restore the locked Keychain refused.
    func foreground() {
        if state == .failed(Self.locked) { restore() }
    }

    func signIn() {
        guard state != .signingIn, !suspended else { return }
        state = .signingIn
        let generation = generation
        Task {
            do {
                let tokens = try await authorize()
                guard generation == self.generation else { return }
                guard store(tokens) else {
                    state = .failed("The Keychain did not keep the Spotify sign-in. Try again after unlocking.")
                    return
                }
                await afterSignIn()
            } catch let error as SignInError {
                guard generation == self.generation else { return }
                state = .failed(error.errorDescription ?? SignInError.failed.errorDescription!)
            } catch {
                guard generation == self.generation else { return }
                state = .failed(SignInError.failed.errorDescription!)
            }
        }
    }

    func signOut() {
        generation += 1
        refreshing?.cancel()
        refreshing = nil
        accessToken = nil
        savedTracks = []
        savedTotal = 0
        playlists = []
        libraryError = nil
        do { try grant.delete(Keychain.webRefresh) } catch {
            state = .failed(Self.notRemoved)
            return
        }
        state = .signedOut
    }

    func suspend() { suspended = true }

    func resume() {
        suspended = false
        if signedIn && savedTracks.isEmpty && playlists.isEmpty { Task { await loadLibrary() } }
    }

    private func afterSignIn() async {
        let generation = generation
        do throws(WebError) {
            let name = try await web.displayName()
            guard generation == self.generation else { return }
            state = .signedIn(name: name)
            await loadLibrary()
        } catch .signInNeeded {
            guard generation == self.generation else { return }
            signOut()
            if state == .signedOut { state = .failed(Self.ended) }
        } catch {
            guard generation == self.generation else { return }
            // Offline: keep the grant, show what is known.
            state = .signedIn(name: "Spotify")
        }
    }

    func loadLibrary() async {
        guard signedIn, !loadingLibrary, !suspended else { return }
        loadingLibrary = true
        defer { loadingLibrary = false }
        let generation = generation
        do {
            async let saved = web.savedTracks()
            async let lists = web.playlists()
            let (page, found) = try await (saved, lists)
            guard generation == self.generation else { return }
            savedTracks = page.tracks
            savedTotal = page.total
            playlists = found
            libraryError = nil
        } catch {
            guard generation == self.generation else { return }
            libraryError = (error as? WebError ?? .unavailable).message
        }
    }

    /// Keeps the access token in memory and the refresh grant in the
    /// Keychain. False when the grant could not be stored.
    @discardableResult
    private func store(_ tokens: Tokens) -> Bool {
        accessToken = (tokens.access_token, Date().addingTimeInterval(TimeInterval((tokens.expires_in ?? 3600) - 90)))
        guard let refresh = tokens.refresh_token else { return true }
        do { try grant.write(Data(refresh.utf8), Keychain.webRefresh) } catch { return false }
        return true
    }

    /// A fresh access token, refreshing once for concurrent callers. Only a
    /// missing grant or one Spotify rejects asks for a new sign-in; offline,
    /// a server error or a locked Keychain keeps the grant.
    private func token() async throws -> String {
        guard !suspended else { throw WebError.unavailable }
        if let accessToken, accessToken.expires > Date() { return accessToken.value }
        if let refreshing { return try await refreshing.value }
        let stored: Data?
        do { stored = try grant.read(Keychain.webRefresh) } catch { throw WebError.unavailable }
        guard let refresh = stored.flatMap({ String(data: $0, encoding: .utf8) }) else { throw WebError.signInNeeded }
        let generation = generation
        let task = Task {
            let tokens = try await exchange(["grant_type": "refresh_token", "refresh_token": refresh, "client_id": Grant.web.clientID])
            // A sign-out while this was in flight must not store a rotated grant.
            guard generation == self.generation else { throw WebError.signInNeeded }
            store(tokens)
            return tokens.access_token
        }
        refreshing = task
        defer { if refreshing == task { refreshing = nil } }
        do {
            return try await task.value
        } catch let error as WebError {
            throw error
        } catch let error as SignInError where error.rejected {
            if generation == self.generation {
                signOut()
                if state == .signedOut { state = .failed(Self.ended) }
            }
            throw WebError.signInNeeded
        } catch {
            throw WebError.unavailable
        }
    }
}
