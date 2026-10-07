// Regression tests for the app models: the Spotify grant across offline
// launches and sign-out, the single real session across demo round trips,
// and pairing with a locked Keychain. Every secret is an in-memory dummy and
// every request goes to `StubNetwork`; nothing reaches Spotify, the private
// cloud or the AI. One separate native-store test uses a disposable dummy
// item under a UUID account; it never reads a real sign-in or pairing item.

import Foundation
import Security
import SpotiurgeCore
import Synchronization
import Testing
@testable import Spotiurge

// MARK: - Fakes

/// An in-memory Keychain that can be made unreadable or refuse deletes.
final class FakeSecrets: Sendable {
    private let items: Mutex<[String: Data]>
    let unreadable = Mutex(false)
    let undeletable = Mutex(false)

    init(_ items: [String: Data] = [:]) {
        self.items = Mutex(items)
    }

    func value(_ account: String) -> Data? { items.withLock { $0[account] } }

    var store: SecretStore {
        SecretStore(
            read: { (account: String) throws(KeychainError) -> Data? in
                if self.unreadable.withLock({ $0 }) { throw KeychainError(status: errSecInteractionNotAllowed) }
                return self.items.withLock { $0[account] }
            },
            write: { (data: Data, account: String) throws(KeychainError) in
                self.items.withLock { $0[account] = data }
            },
            delete: { (account: String) throws(KeychainError) in
                if self.undeletable.withLock({ $0 }) { throw KeychainError(status: errSecInteractionNotAllowed) }
                self.items.withLock { $0[account] = nil }
            })
    }
}

/// A scripted network keyed by "METHOD /path". Chosen requests are held
/// until released; every request and cancellation is recorded.
final class StubNetwork: URLProtocol, @unchecked Sendable {
    struct Reply: Sendable {
        var status = 200
        var body = Data()
    }

    private struct State {
        var replies: [String: Reply] = [:]
        var hold: Set<String> = []
        var held: [(key: String, request: StubNetwork)] = []
        var seen: [String] = []
        var stopped: [String] = []
    }

    private static let state = Mutex(State())

    static func reset(_ replies: [String: Reply] = [:], hold: Set<String> = []) {
        state.withLock { $0 = State(replies: replies, hold: hold) }
    }

    static func reply(_ key: String, _ reply: Reply) { state.withLock { $0.replies[key] = reply } }
    static var seen: [String] { state.withLock { $0.seen } }
    static var stopped: [String] { state.withLock { $0.stopped } }

    /// Answers held requests for `key`, and stops holding new ones.
    static func release(_ key: String, _ reply: Reply) {
        let held = state.withLock { state in
            state.hold.remove(key)
            defer { state.held.removeAll { $0.key == key } }
            return state.held.filter { $0.key == key }.map(\.request)
        }
        for request in held { request.answer(reply) }
    }

    static var session: URLSession {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubNetwork.self]
        return URLSession(configuration: configuration)
    }

    private var key: String { "\(request.httpMethod ?? "GET") \(request.url?.path ?? "")" }
    private var thread: Thread?
    private var pending: Reply?
    private var cancelled = false

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        thread = Thread.current
        let key = key
        let reply = Self.state.withLock { state -> Reply? in
            state.seen.append(key)
            if state.hold.contains(key) {
                state.held.append((key, self))
                return nil
            }
            return state.replies[key] ?? Reply(status: 503)
        }
        if let reply { answer(reply) }
    }

    override func stopLoading() {
        cancelled = true
        let key = key
        Self.state.withLock { $0.stopped.append(key) }
    }

    /// URL loading expects its client on the thread that started loading.
    private func answer(_ reply: Reply) {
        pending = reply
        guard let thread else { return }
        perform(#selector(deliver), on: thread, with: nil, waitUntilDone: false, modes: [RunLoop.Mode.common.rawValue])
    }

    @objc private func deliver() {
        // A cancelled request never gets its late answer.
        guard !cancelled, let reply = pending, let url = request.url else { return }
        client?.urlProtocol(self, didReceive: HTTPURLResponse(url: url, statusCode: reply.status, httpVersion: nil, headerFields: nil)!,
                            cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: reply.body)
        client?.urlProtocolDidFinishLoading(self)
    }
}

/// Holds a token exchange open until the test lets it finish.
@MainActor
final class Gate {
    private var continuation: CheckedContinuation<Tokens, Error>?
    var waiting: Bool { continuation != nil }

    func wait() async throws -> Tokens {
        try await withCheckedThrowingContinuation { continuation = $0 }
    }

    func open(_ tokens: Tokens) {
        continuation?.resume(returning: tokens)
        continuation = nil
    }
}

@MainActor
func eventually(_ condition: () -> Bool) async -> Bool {
    for _ in 0..<300 {
        if condition() { return true }
        try? await Task.sleep(for: .milliseconds(10))
    }
    return condition()
}

/// Lets late tasks run before checking that they changed nothing.
func settle() async {
    try? await Task.sleep(for: .milliseconds(200))
}

func json(_ text: String) -> StubNetwork.Reply { .init(status: 200, body: Data(text.utf8)) }

// MARK: - Tests

@MainActor
@Suite(.serialized)
struct ModelLifecycle {
    let grant = Data("dummy-refresh-grant".utf8)
    let me = json(#"{"id":"serge","display_name":"Serge"}"#)
    let emptyTracks = json(#"{"items":[],"total":0}"#)
    let emptyPlaylists = json(#"{"items":[]}"#)
    let endpoint = "https://cloud.invalid"
    var cloudKey: String { Keychain.cloud(endpoint) }
    let cloudToken = Data(String(repeating: "t", count: 40).utf8)

    @Test func `native Keychain stores replaces and removes a disposable dummy item`() throws {
        let account = "spotiurge-unit-test:\(UUID().uuidString)"
        defer { try? Keychain.delete(account) }
        #expect(try Keychain.read(account) == nil)
        let first = Data("dummy-first-grant".utf8)
        let replacement = Data("dummy-replacement-grant".utf8)
        try Keychain.write(first, account)
        #expect(try Keychain.read(account) == first)
        try Keychain.write(replacement, account)
        #expect(try Keychain.read(account) == replacement)
        try Keychain.delete(account)
        #expect(try Keychain.read(account) == nil)
    }

    func makeAccount(_ secrets: FakeSecrets, exchange: @escaping ([String: String]) async throws -> Tokens) -> SpotifyAccount {
        SpotifyAccount(grant: secrets.store, authorize: { throw SignInError.cancelled }, exchange: exchange, session: StubNetwork.session)
    }

    func makeDiscovery(_ secrets: FakeSecrets) -> DiscoveryModel {
        let defaults = UserDefaults(suiteName: "spotiurge-tests-\(UUID().uuidString)")!
        defaults.set(endpoint, forKey: "cloudEndpoint")
        let file = FileManager.default.temporaryDirectory.appending(path: "discovery-\(UUID().uuidString).json")
        let model = DiscoveryModel(file: file, defaults: defaults, secrets: secrets.store, session: StubNetwork.session)
        // Automatic picks would race the requests these tests script.
        model.setAutomatic(false)
        return model
    }

    // MARK: Spotify grant (F1, F4, F7)

    @Test func `an offline launch or a failed refresh keeps the Spotify grant`() async {
        let failures: [any Error] = [SignInError.unreachable, SignInError.failed, URLError(.notConnectedToInternet), CancellationError()]
        for failure in failures {
            StubNetwork.reset()
            let secrets = FakeSecrets([Keychain.webRefresh: grant])
            let account = makeAccount(secrets) { _ in throw failure }
            account.restore()
            #expect(await eventually { account.state == .signedIn(name: "Spotify") })
            #expect(secrets.value(Keychain.webRefresh) == grant)
            #expect(StubNetwork.seen.isEmpty)
        }
    }

    @Test func `only a rejected grant signs out`() async {
        StubNetwork.reset()
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        let account = makeAccount(secrets) { _ in throw SignInError("Rejected.", rejected: true) }
        account.restore()
        #expect(await eventually { account.state == .failed(SpotifyAccount.ended) })
        #expect(secrets.value(Keychain.webRefresh) == nil)
    }

    @Test func `a locked Keychain at launch is not a sign-out and is retried`() async {
        StubNetwork.reset(["GET /v1/me": me, "GET /v1/me/tracks": emptyTracks, "GET /v1/me/playlists": emptyPlaylists])
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        secrets.unreadable.withLock { $0 = true }
        var exchanges = 0
        let account = makeAccount(secrets) { _ in
            exchanges += 1
            return Tokens(access_token: "dummy-access", expires_in: 3600, refresh_token: nil)
        }
        account.restore()
        #expect(account.state == .failed(SpotifyAccount.locked))
        account.foreground()
        #expect(account.state == .failed(SpotifyAccount.locked))
        #expect(exchanges == 0)
        #expect(secrets.value(Keychain.webRefresh) == grant)

        secrets.unreadable.withLock { $0 = false }
        account.foreground()
        #expect(await eventually { account.state == .signedIn(name: "Serge") && !account.loadingLibrary })
        #expect(exchanges == 1)
    }

    @Test func `a sign-out during a refresh never stores the rotated grant`() async {
        StubNetwork.reset(["GET /v1/me": me])
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        let gate = Gate()
        let account = makeAccount(secrets) { _ in try await gate.wait() }
        account.restore()
        #expect(await eventually { gate.waiting })
        account.signOut()
        #expect(account.state == .signedOut)
        gate.open(Tokens(access_token: "dummy-access", expires_in: 3600, refresh_token: "dummy-rotated"))
        await settle()
        #expect(secrets.value(Keychain.webRefresh) == nil)
        #expect(account.state == .signedOut)
        #expect(StubNetwork.seen.isEmpty)
    }

    @Test func `a library answer after sign-out is dropped`() async {
        StubNetwork.reset(["GET /v1/me": me], hold: ["GET /v1/me/tracks", "GET /v1/me/playlists"])
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        let account = makeAccount(secrets) { _ in Tokens(access_token: "dummy-access", expires_in: 3600, refresh_token: nil) }
        account.restore()
        #expect(await eventually { account.state == .signedIn(name: "Serge") && StubNetwork.seen.count == 3 })
        account.signOut()
        StubNetwork.release("GET /v1/me/tracks", json(#"{"items":[{"track":{"uri":"spotify:track:aaaaaaaaaaaaaaaaaaaaaa","name":"Late","artists":[{"name":"A"}],"duration_ms":1000}}],"total":1}"#))
        StubNetwork.release("GET /v1/me/playlists", json(#"{"items":[{"uri":"spotify:playlist:late","name":"Late","owner":{"display_name":"S"},"tracks":{"total":1}}]}"#))
        #expect(await eventually { !account.loadingLibrary })
        await settle()
        #expect(account.savedTracks.isEmpty && account.playlists.isEmpty && account.savedTotal == 0)
        #expect(account.state == .signedOut)
    }

    @Test func `a failed Keychain delete is reported, not shown as signed out`() async {
        StubNetwork.reset(["GET /v1/me": me, "GET /v1/me/tracks": emptyTracks, "GET /v1/me/playlists": emptyPlaylists])
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        secrets.undeletable.withLock { $0 = true }
        let account = makeAccount(secrets) { _ in Tokens(access_token: "dummy-access", expires_in: 3600, refresh_token: nil) }
        account.restore()
        #expect(await eventually { account.state == .signedIn(name: "Serge") })
        account.signOut()
        #expect(account.state == .failed(SpotifyAccount.notRemoved))
        #expect(secrets.value(Keychain.webRefresh) == grant)
    }

    // MARK: One real session across demo (F2, F3)

    @Test func `a demo round trip keeps one real session and suspends it`() {
        let model = makeDiscovery(FakeSecrets())
        let account = makeAccount(FakeSecrets()) { _ in throw SignInError.unreachable }
        var made = 0
        let sessions = Sessions(launch: .home) {
            made += 1
            return Session(discovery: model, account: account, player: Player(secrets: FakeSecrets().store))
        }
        #expect(made == 0 && sessions.current.discovery.demo)

        sessions.select(nil)
        #expect(made == 1 && sessions.current.discovery === model && !model.suspended)
        for _ in 0..<2 {
            sessions.select(.home)
            #expect(sessions.current.discovery !== model && sessions.current.discovery.demo)
            #expect(sessions.current.player.demo && sessions.current.player !== Player.shared)
            #expect(model.suspended && account.suspended)
            sessions.select(nil)
            #expect(sessions.current.discovery === model && !model.suspended && !account.suspended)
        }
        #expect(made == 1)
    }

    @Test func `a demo player never touches the real playback credential`() {
        let secrets = FakeSecrets([Keychain.playback: Data("dummy-playback".utf8)])
        let player = Player(secrets: secrets.store)
        player.showDemo(Demo.nowPlaying, positionMs: 0, playing: false)
        let engine = player.engine
        player.forget()
        player.takeOver()
        player.signIn()
        player.start()
        #expect(secrets.value(Keychain.playback) != nil)
        #expect(player.engine == engine)
    }

    @Test func `leaving demo retries a Spotify restore refused by the locked Keychain`() async {
        StubNetwork.reset(["GET /v1/me": me, "GET /v1/me/tracks": emptyTracks, "GET /v1/me/playlists": emptyPlaylists])
        let secrets = FakeSecrets([Keychain.webRefresh: grant])
        secrets.unreadable.withLock { $0 = true }
        let account = makeAccount(secrets) { _ in Tokens(access_token: "dummy-access", expires_in: 3600, refresh_token: nil) }
        let sessions = Sessions(launch: nil) {
            Session(discovery: makeDiscovery(FakeSecrets()), account: account, player: Player(secrets: FakeSecrets().store))
        }
        account.restore()
        #expect(account.state == .failed(SpotifyAccount.locked))
        sessions.select(.home)
        secrets.unreadable.withLock { $0 = false }
        sessions.select(nil)
        #expect(await eventually { account.state == .signedIn(name: "Serge") && !account.loadingLibrary })
        #expect(secrets.value(Keychain.webRefresh) == grant)
    }

    @Test func `suspending cancels the AI request, drops its answer and sends nothing new`() async {
        StubNetwork.reset(hold: ["POST /v1/recommendations"])
        let model = makeDiscovery(FakeSecrets([cloudKey: cloudToken]))
        model.load()
        #expect(await eventually { model.ready && StubNetwork.seen == ["GET /v1/state"] && !model.syncing })
        model.draft = "Warm, spacious electronics."
        model.recommend()
        #expect(model.busy)
        #expect(await eventually { StubNetwork.seen.contains("POST /v1/recommendations") })

        model.suspend()
        let status = model.status
        #expect(!model.busy)
        #expect(await eventually { StubNetwork.stopped.contains("POST /v1/recommendations") })
        let answer = json(#"{"suggestions":[{"title":"T","artist":"A","reason":"R"}],"model":"gpt-6-luna"}"#)
        StubNetwork.reply("POST /v1/recommendations", answer)
        StubNetwork.release("POST /v1/recommendations", answer)
        let sent = StubNetwork.seen
        model.recommend()
        model.sync(manual: true)
        model.foreground()
        await settle()
        #expect(StubNetwork.seen == sent)
        #expect(model.picks.isEmpty && model.lastError == nil && model.status == status && !model.busy)

        // Back from demo: the same model syncs and recommends again.
        model.resume()
        #expect(await eventually { StubNetwork.seen.count == sent.count + 1 && !model.syncing })
        #expect(StubNetwork.seen.last == "GET /v1/state")
        model.recommend()
        #expect(await eventually { !model.busy })
        #expect(model.picks.map(\.suggestion.title) == ["T"])
        model.suspend()
    }

    @Test func `manual refresh cannot bypass a rate limited recommendation`() async {
        StubNetwork.reset(["POST /v1/recommendations": .init(status: 429)])
        let model = makeDiscovery(FakeSecrets([cloudKey: cloudToken]))
        model.load()
        #expect(await eventually { model.ready && !model.syncing })
        model.draft = "Warm, spacious electronics."
        model.recommend()
        #expect(await eventually { !model.busy && model.lastError == .rateLimited })
        let sent = StubNetwork.seen.filter { $0 == "POST /v1/recommendations" }.count
        for _ in 0..<3 { model.recommend() }
        await settle()
        #expect(sent == 1)
        #expect(StubNetwork.seen.filter { $0 == "POST /v1/recommendations" }.count == sent)
        model.suspend()
    }

    // MARK: Pairing (F7)

    @Test func `a locked Keychain is not an unpaired phone`() async {
        StubNetwork.reset()
        let secrets = FakeSecrets([cloudKey: cloudToken])
        secrets.unreadable.withLock { $0 = true }
        let model = makeDiscovery(secrets)
        model.load()
        #expect(await eventually { model.ready })
        #expect(model.pairing == .unreadable && !model.paired)
        model.draft = "Warm, spacious electronics."
        model.recommend()
        #expect(model.lastError == .unavailable && model.status == DiscoveryModel.keychainLocked)
        #expect(StubNetwork.seen.isEmpty)
        #expect(secrets.value(cloudKey) == cloudToken)

        StubNetwork.reply("GET /v1/state", json(#"{"revision":0,"document":{"version":1,"records":{}}}"#))
        StubNetwork.reply("PUT /v1/state", json(#"{"revision":1}"#))
        secrets.unreadable.withLock { $0 = false }
        model.foreground()
        #expect(model.pairing == .paired)
        #expect(await eventually { !model.syncing && model.lastSync != nil })
        #expect(model.status != DiscoveryModel.keychainLocked && model.lastError == nil)
        model.suspend()
    }

    @Test func `an unpaired phone asks to pair`() async {
        StubNetwork.reset()
        let model = makeDiscovery(FakeSecrets())
        model.load()
        #expect(await eventually { model.ready })
        #expect(model.pairing == .unpaired)
        model.draft = "Warm, spacious electronics."
        model.recommend()
        #expect(model.lastError == .pairing && model.status.hasPrefix("Pair this iPhone"))
        #expect(StubNetwork.seen.isEmpty)
        model.suspend()
    }
}
