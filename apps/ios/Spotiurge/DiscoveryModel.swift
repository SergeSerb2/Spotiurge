// Home's radio desk: the same workflow and state rules as the desktop's
// Discovery (src/app.rs actions and backend events). Edits are optimistic
// and local first; a stale AI or sync answer never undoes a newer edit.

import Foundation
import Observation
import SpotiurgeCore

/// Whether the private-cloud token can be used. A locked or failing Keychain
/// is not "unpaired": the token may be there.
enum Pairing: Equatable {
    case paired, unpaired, unreadable
}

@Observable
final class DiscoveryModel {
    static let keychainLocked = "The Keychain is locked, so the pairing token cannot be read. Unlock this iPhone and return to Spotiurge."

    private(set) var replica = Replica()
    private(set) var picks: [Pick] = []
    private(set) var ready = false
    private(set) var busy = false
    private(set) var syncing = false
    private(set) var status = ""
    private(set) var lastError: RecommendationErrorKind?
    private(set) var catalogue = CatalogueOutcome.complete
    private(set) var preferences = Preferences()
    private(set) var pairing = Pairing.unpaired
    private(set) var lastSync: Date?
    var draft = ""
    var editingTaste = false
    /// Synthetic data for demos and captures. Never saved, synced or sent.
    private(set) var demo = false
    /// The real model while demo data shows: nothing new is sent, and any
    /// answer that started before is dropped.
    private(set) var suspended = false

    @ObservationIgnored private var request = 0
    @ObservationIgnored private var inFlight: Int?
    @ObservationIgnored private var automatic = AutoRecommendations()
    @ObservationIgnored private var autoTask: Task<Void, Never>?
    @ObservationIgnored private var syncSoon: Task<Void, Never>?
    @ObservationIgnored private var syncSnapshot: Document?
    @ObservationIgnored private var retryAfter: Date?
    @ObservationIgnored private var pendingWrite: Replica?
    @ObservationIgnored private var writing = false
    @ObservationIgnored var account: SpotifyAccount?
    /// Network work in flight, cancelled on suspension. `epoch` changes with
    /// each suspension so a late answer is recognised and dropped.
    @ObservationIgnored private var work: [UUID: Task<Void, Never>] = [:]
    @ObservationIgnored private var epoch = 0

    private let file: URL
    private let defaults: UserDefaults
    private let secrets: SecretStore
    private let session: URLSession

    init(file: URL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0].appending(path: "spotiurge-discovery.json"),
         defaults: UserDefaults = .standard, secrets: SecretStore = .keychain, session: URLSession = .shared) {
        self.file = file
        self.defaults = defaults
        self.secrets = secrets
        self.session = session
    }

    var endpoint: String {
        defaults.string(forKey: "cloudEndpoint") ?? PrivateCloud.defaultEndpoint
    }

    var paired: Bool { pairing == .paired }
    var exploration: Exploration { preferences.exploration }
    var playable: [Pick] { picks.filter { $0.track != nil } }
    var unmatched: [Pick] { picks.filter { $0.track == nil } }
    var dirty: Bool { lastSyncedDocument != replica.document }
    @ObservationIgnored private var lastSyncedDocument: Document?

    // MARK: - Lifecycle

    func load() {
        if let data = defaults.data(forKey: "discoveryPreferences"),
           let stored = try? JSONDecoder().decode(Preferences.self, from: data) {
            preferences = stored
        }
        refreshPairing()
        let file = file
        Task {
            let result = await Task.detached { Result { () throws(DiscoveryError) in try Replica.load(from: file) } }.value
            guard !demo else { return }
            switch result {
            case .success(let replica):
                self.replica = replica
                picks = replica.cachedPicks
                draft = replica.document.taste
                ready = true
                status = ""
                sync()
            case .failure(let error):
                status = error.message
            }
            schedule()
        }
    }

    func foreground() {
        guard ready, !demo, !suspended else { return }
        sync()
        schedule()
    }

    /// Demo data is showing: cancel automatic picks, the sync debounce and
    /// every request in flight, and start nothing new until `resume`. A
    /// request the server already received cannot be unsent; its answer is
    /// dropped. Local data stays as it is.
    func suspend() {
        guard !suspended else { return }
        suspended = true
        epoch += 1
        autoTask?.cancel()
        autoTask = nil
        syncSoon?.cancel()
        syncSoon = nil
        for task in work.values { task.cancel() }
        work = [:]
        request += 1
        inFlight = nil
        busy = false
        syncing = false
        syncSnapshot = nil
    }

    func resume() {
        guard suspended else { return }
        suspended = false
        foreground()
    }

    /// Runs network work that suspension cancels. `finish` runs only if no
    /// suspension happened since the work started.
    private func start<T>(_ operation: @escaping @MainActor () async -> T, finish: @escaping @MainActor (T) -> Void) {
        let id = UUID(), epoch = epoch
        work[id] = Task {
            let result = await operation()
            work[id] = nil
            guard epoch == self.epoch, !Task.isCancelled else { return }
            finish(result)
        }
    }

    // MARK: - Pairing

    func refreshPairing() {
        guard let origin = try? PrivateCloud.origin(endpoint) else {
            pairing = .unpaired
            return
        }
        switch Result(catching: { () throws(KeychainError) in try secrets.read(Keychain.cloud(origin.absoluteString)) }) {
        case .success(let token): pairing = token == nil ? .unpaired : .paired
        case .failure: pairing = .unreadable
        }
    }

    /// Stores a pairing token for the configured origin. The field is the
    /// only input; the token is never displayed, logged or put in a URL.
    func pair(endpoint: String, token: String) -> String? {
        let token = token.trimmingCharacters(in: .whitespacesAndNewlines)
        let origin: URL
        do { origin = try PrivateCloud.origin(endpoint) } catch { return error.message }
        guard (32...256).contains(token.utf8.count), token.allSatisfy(\.isASCII) else {
            return "Invalid pairing token. Paste the private cloud's SPOTIURGE_CLOUD_TOKEN."
        }
        do { try secrets.write(Data(token.utf8), Keychain.cloud(origin.absoluteString)) } catch {
            return "The Keychain did not keep the pairing token. Your previous pairing is unchanged."
        }
        defaults.set(origin.absoluteString, forKey: "cloudEndpoint")
        refreshPairing()
        if lastError == .pairing {
            automatic.rearm()
            lastError = nil
        }
        status = "Paired with your private cloud."
        sync()
        schedule()
        return nil
    }

    func unpair() {
        if let origin = try? PrivateCloud.origin(endpoint) {
            do { try secrets.delete(Keychain.cloud(origin.absoluteString)) } catch {
                status = "The Keychain did not remove the pairing token. Try again after unlocking."
                return
            }
        }
        refreshPairing()
        status = "This iPhone is no longer paired. Local picks, mixes and feedback stay here."
    }

    /// Every private-cloud request reads its token here, so none starts
    /// while the model is suspended.
    private func cloud() throws(DiscoveryError) -> CloudClient {
        let secrets = secrets
        return try CloudClient(endpoint: endpoint, session: session) { [weak self] origin in
            guard await self?.suspended == false else { throw CancellationError() }
            guard let data = try secrets.read(Keychain.cloud(origin)), let token = String(data: data, encoding: .utf8) else {
                throw DiscoveryError("Not paired")
            }
            return token
        }
    }

    /// The reason a pairing cannot be used right now, or nil when paired.
    private func pairingProblem(_ action: String) -> (kind: RecommendationErrorKind, message: String)? {
        refreshPairing()
        switch pairing {
        case .paired: return nil
        case .unpaired: return (.pairing, "Pair this iPhone with your private cloud in Settings to \(action).")
        case .unreadable: return (.unavailable, Self.keychainLocked)
        }
    }

    // MARK: - Actions (src/app.rs Action::Discovery*)

    func saveTaste() {
        guard ready, !demo else { return }
        edit("taste", .taste(text: limitPrompt(draft))) {
            editingTaste = false
            invalidate()
            automatic.changed(now: .now, exploration: true)
            status = "Taste saved. It syncs to your devices when you are online."
        }
    }

    func cancelTaste() {
        draft = replica.document.taste
        editingTaste = false
    }

    func setExploration(_ value: Exploration) {
        guard preferences.exploration != value else { return }
        preferences.exploration = value
        savePreferences()
        guard !demo else { return }
        invalidate()
        automatic.changed(now: .now, exploration: true)
        schedule()
    }

    func setAutomatic(_ enabled: Bool) {
        preferences.automatic = enabled
        savePreferences()
        schedule()
    }

    func rate(_ track: Track, _ rating: Rating?) {
        guard ready, !demo, isSpotifyTrack(track.uri) else { return }
        let value: Value? = rating.map { .feedback(uri: track.uri, title: track.name, artist: track.artistLine, rating: $0) }
        edit("feedback:\(track.uri)", value) {
            invalidate()
            automatic.changed(now: .now, exploration: false)
            status = "Feedback saved. It will shape your next discoveries."
        }
    }

    func rating(_ track: Track) -> Rating? {
        replica.document.rating(track.uri)
    }

    func saveMix() {
        guard ready, !demo else { return }
        let uris = playable.compactMap(\.track?.uri)
        guard !uris.isEmpty else { return }
        edit("mix:\(newDeviceID())", .mix(title: "My discovery mix", uris: Array(uris.prefix(100)))) {
            status = "Mix saved. It syncs to your devices when you are online."
        }
    }

    func deleteMix(_ key: String) {
        guard ready, !demo else { return }
        edit(key, nil) { status = "Mix removed." }
    }

    func recommend(automatic isAutomatic: Bool = false) {
        guard ready, !demo, !suspended, !busy, inFlight == nil else { return }
        if !isAutomatic && draft != replica.document.taste {
            do { try replica.edit("taste", .taste(text: limitPrompt(draft))) } catch {
                status = error.message
                return
            }
            persist()
        }
        guard replica.document.hasInputs else {
            editingTaste = true
            return
        }
        if let problem = pairingProblem("get picks") {
            lastError = problem.kind
            status = problem.message
            return
        }
        if !isAutomatic { automatic.rearm() }
        automatic.attempted(now: .now)
        lastError = nil
        request += 1
        busy = true
        inFlight = request
        let id = request, document = replica.document, exploration = preferences.exploration
        let web = account?.signedIn == true ? account?.web : nil
        let client: CloudClient
        do { client = try cloud() } catch {
            finishRecommendation(id: id, prompt: document.taste, .failure(RecommendationFailure(.pairing, error.message)))
            return
        }
        start { () async -> Result<(picks: [Pick], outcome: CatalogueOutcome), RecommendationFailure> in
            let result: Result<(picks: [Pick], outcome: CatalogueOutcome), RecommendationFailure>
            do throws(RecommendationFailure) {
                let suggestions = try await client.recommend(document, exploration: exploration)
                result = .success(await self.resolve(suggestions.map { Pick(suggestion: $0) }, web: web))
            } catch {
                result = .failure(error)
            }
            return result
        } finish: { result in
            self.finishRecommendation(id: id, prompt: document.taste, result)
        }
    }

    func retryMatches() {
        guard ready, !demo, !suspended, !busy, inFlight == nil, retryAfter.map({ $0 < .now }) ?? true, let web = account?.web, account?.signedIn == true else { return }
        request += 1
        busy = true
        inFlight = request
        let id = request, picks = picks
        start { await self.resolve(picks, web: web) } finish: { self.finishMatches(id: id, $0) }
    }

    private func resolve(_ picks: [Pick], web: SpotifyWeb?) async -> (picks: [Pick], outcome: CatalogueOutcome) {
        guard let web else { return (picks, .signInNeeded) }
        return await resolveDiscovery(picks, budget: .seconds(20)) { suggestion throws(CatalogueError) in
            try await web.catalogueSearch(suggestion)
        }
    }

    // MARK: - Results (src/app.rs Event::Discovery*)

    private func finishRecommendation(id: Int, prompt: String, _ result: Result<(picks: [Pick], outcome: CatalogueOutcome), RecommendationFailure>) {
        if inFlight == id { inFlight = nil }
        defer { schedule() }
        guard id == request else { return }
        busy = false
        switch result {
        case .success(let resolved):
            accept(resolved)
            automatic.rearm()
            lastError = nil
            preferences.refreshedAt = UInt64(Date().timeIntervalSince1970)
            savePreferences()
            do {
                try replica.recordHistory(prompt: prompt, suggestions: resolved.picks.map(\.suggestion))
                status = ""
            } catch {
                status = error.message
            }
            persist()
            requestSyncSoon()
        case .failure(let failure):
            automatic.failed(failure.kind, now: .now)
            lastError = failure.kind
            status = failure.message
        }
    }

    private func finishMatches(id: Int, _ resolved: (picks: [Pick], outcome: CatalogueOutcome)) {
        if inFlight == id { inFlight = nil }
        defer { schedule() }
        guard id == request else { return }
        busy = false
        accept(resolved)
        persist()
    }

    private func accept(_ resolved: (picks: [Pick], outcome: CatalogueOutcome)) {
        picks = resolved.picks
        catalogue = resolved.outcome
        retryAfter = [.rateLimited, .quotaExhausted].contains(resolved.outcome) ? Date(timeIntervalSinceNow: 30) : nil
    }

    // MARK: - Sync

    func sync(manual: Bool = false) {
        guard ready, !demo, !suspended, !syncing else { return }
        if let problem = pairingProblem("sync") {
            if manual || problem.kind == .unavailable { status = problem.message }
            return
        }
        let client: CloudClient
        do throws(DiscoveryError) { client = try cloud() } catch {
            status = error.message
            return
        }
        syncing = true
        syncSnapshot = replica.document
        if manual { status = "Syncing preferences, feedback, mixes and AI history…" }
        let sent = replica
        start { () async -> Result<Document, DiscoveryError> in
            do throws(DiscoveryError) { return .success(try await client.sync(sent)) } catch { return .failure(error) }
        } finish: { result in
            self.finishSync(result)
        }
    }

    private func finishSync(_ result: Result<Document, DiscoveryError>) {
        syncing = false
        let snapshot = syncSnapshot
        syncSnapshot = nil
        switch result {
        case .success(let remote):
            if status == Self.keychainLocked {
                status = ""
                if lastError == .unavailable { lastError = nil }
            }
            let before = replica.document
            let draftUnchanged = draft == replica.document.taste
            do {
                try replica.mergeSynced(remote, snapshot: snapshot)
            } catch {
                status = error.message
                return
            }
            let inputsChanged = replica.document.records.contains { key, record in
                (key == "taste" || key.hasPrefix("feedback:")) && before.records[key]?.value != record.value
            }
            if lastError == .pairing {
                automatic.rearm()
                lastError = nil
            }
            if inputsChanged {
                invalidate()
                automatic.changed(now: .now, exploration: true)
            }
            if draftUnchanged { draft = replica.document.taste }
            lastSyncedDocument = remote
            lastSync = .now
            if dirty { requestSyncSoon() }
            if status.hasPrefix("Syncing") || status.contains("syncs to your devices") { status = "Synced with your private cloud." }
            persist()
            schedule()
        case .failure(let error):
            status = error.message
            if error.message.contains("Pair this iPhone again") { lastError = .pairing }
        }
    }

    /// Edits sync a few seconds after they settle, so a burst becomes one write.
    private func requestSyncSoon() {
        syncSoon?.cancel()
        syncSoon = Task {
            try? await Task.sleep(for: .seconds(4))
            guard !Task.isCancelled else { return }
            sync()
        }
    }

    // MARK: - Helpers

    private func edit(_ key: String, _ value: Value?, then: () -> Void) {
        do {
            try replica.edit(key, value)
        } catch {
            status = error.message
            return
        }
        then()
        persist()
        requestSyncSoon()
        schedule()
    }

    /// A newer input makes an in-flight answer stale; it still occupies the
    /// single server slot until it returns, so no new request starts before.
    private func invalidate() {
        request += 1
        busy = false
    }

    private func schedule() {
        autoTask?.cancel()
        guard let due = automatic.due(now: .now, wallNow: UInt64(Date().timeIntervalSince1970), preferences: preferences,
                                      enabled: ready && !demo && !suspended && paired && inFlight == nil,
                                      hasInputs: replica.document.hasInputs, empty: picks.isEmpty)
        else { return }
        autoTask = Task {
            try? await Task.sleep(for: due)
            guard !Task.isCancelled else { return }
            recommend(automatic: true)
        }
    }

    private func savePreferences() {
        guard !demo, let data = try? JSONEncoder().encode(preferences) else { return }
        defaults.set(data, forKey: "discoveryPreferences")
    }

    /// One writer, latest snapshot wins, through an atomic replace off the main actor.
    private func persist() {
        guard !demo else { return }
        var snapshot = replica
        snapshot.cachedPicks = Array(picks.prefix(12))
        pendingWrite = snapshot
        guard !writing else { return }
        writing = true
        let file = file
        Task {
            while let next = pendingWrite {
                pendingWrite = nil
                let result = await Task.detached { Result { () throws(DiscoveryError) in try next.save(to: file) } }.value
                if case .failure(let error) = result { status = error.message }
            }
            writing = false
        }
    }

    // MARK: - Demo

    func showDemo(_ scenario: DemoScenario) {
        demo = true
        ready = true
        autoTask?.cancel()
        let data = scenario.discovery
        replica = data.replica
        picks = data.picks
        draft = replica.document.taste
        status = data.status
        lastError = data.error
        catalogue = data.outcome
        pairing = .paired
        preferences.exploration = .balanced
        lastSync = scenario == .home ? Date(timeIntervalSinceNow: -240) : nil
    }
}
