// Personal discovery state, wire-compatible with src/discovery.rs and the
// private cloud's validator. No Spotify grants or audio enter this document.
//
// Per-record logical clocks make offline edits merge independently of wall
// clocks. The cloud stores only the document; writer identity stays local.

import Foundation

public enum DiscoveryLimits {
    public static let maxBytes = 1_048_576
    public static let maxPromptBytes = 4000
    /// Live AI history entries per document, matching the newest ten shown.
    public static let historyLimit = 10
    public static let mixLimit = 100
    public static let maxRecords = 2000
    public static let feedbackLimit = 500
    static let feedbackFloor = "feedback:retention"
}

public struct DiscoveryError: Error, Equatable, LocalizedError, Sendable {
    public let message: String
    public init(_ message: String) { self.message = message }
    public var errorDescription: String? { message }
}

/// Keep editor input within the wire/storage limit without splitting a character.
public func limitPrompt(_ text: String) -> String {
    guard text.utf8.count > DiscoveryLimits.maxPromptBytes else { return text }
    var result = ""
    for character in text {
        if result.utf8.count + String(character).utf8.count > DiscoveryLimits.maxPromptBytes { break }
        result.append(character)
    }
    return result
}

public func isSpotifyTrack(_ uri: String) -> Bool {
    guard uri.hasPrefix("spotify:track:") else { return false }
    let id = uri.dropFirst("spotify:track:".count)
    return id.utf8.count == 22 && id.utf8.allSatisfy { ($0 >= 48 && $0 <= 57) || ($0 >= 65 && $0 <= 90) || ($0 >= 97 && $0 <= 122) }
}

func isDeviceID(_ id: String) -> Bool {
    id.utf8.count == 32 && id.utf8.allSatisfy { ($0 >= 48 && $0 <= 57) || ($0 >= 97 && $0 <= 102) }
}

public func newDeviceID() -> String {
    var generator = SystemRandomNumberGenerator()
    return String(format: "%016llx%016llx", generator.next() as UInt64, generator.next() as UInt64)
}

// MARK: - Records

public struct Stamp: Codable, Hashable, Comparable, Sendable {
    public var counter: UInt64
    public var device: String

    public init(counter: UInt64, device: String) {
        self.counter = counter
        self.device = device
    }

    public static func < (a: Stamp, b: Stamp) -> Bool {
        (a.counter, a.device) < (b.counter, b.device)
    }

    public init(from decoder: any Decoder) throws {
        try requireExactKeys(decoder, ["counter", "device"])
        let container = try decoder.container(keyedBy: CodingKeys.self)
        counter = try container.decode(UInt64.self, forKey: .counter)
        device = try container.decode(String.self, forKey: .device)
    }
}

public enum Rating: String, Codable, Sendable {
    case love, less
}

public enum Exploration: String, Codable, CaseIterable, Sendable {
    case familiar, balanced, adventurous
}

public struct Suggestion: Codable, Hashable, Sendable {
    public var title: String
    public var artist: String
    public var reason: String

    public init(title: String, artist: String, reason: String) {
        self.title = title
        self.artist = artist
        self.reason = reason
    }

    public init(from decoder: any Decoder) throws {
        try requireExactKeys(decoder, ["title", "artist", "reason"])
        let container = try decoder.container(keyedBy: CodingKeys.self)
        title = try container.decode(String.self, forKey: .title)
        artist = try container.decode(String.self, forKey: .artist)
        reason = try container.decode(String.self, forKey: .reason)
    }
}

public enum Value: Codable, Hashable, Sendable {
    case taste(text: String)
    case feedback(uri: String, title: String, artist: String, rating: Rating)
    case mix(title: String, uris: [String])
    case history(prompt: String, suggestions: [Suggestion])

    private enum Keys: String, CodingKey {
        case kind, text, uri, title, artist, rating, uris, prompt, suggestions
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: Keys.self)
        switch try container.decode(String.self, forKey: .kind) {
        case "taste":
            try requireExactKeys(decoder, ["kind", "text"])
            self = .taste(text: try container.decode(String.self, forKey: .text))
        case "feedback":
            try requireExactKeys(decoder, ["kind", "uri", "title", "artist", "rating"])
            self = .feedback(
                uri: try container.decode(String.self, forKey: .uri),
                title: try container.decode(String.self, forKey: .title),
                artist: try container.decode(String.self, forKey: .artist),
                rating: try container.decode(Rating.self, forKey: .rating))
        case "mix":
            try requireExactKeys(decoder, ["kind", "title", "uris"])
            self = .mix(
                title: try container.decode(String.self, forKey: .title),
                uris: try container.decode([String].self, forKey: .uris))
        case "history":
            try requireExactKeys(decoder, ["kind", "prompt", "suggestions"])
            self = .history(
                prompt: try container.decode(String.self, forKey: .prompt),
                suggestions: try container.decode([Suggestion].self, forKey: .suggestions))
        default:
            throw DecodingError.dataCorruptedError(forKey: .kind, in: container, debugDescription: "Unknown discovery value")
        }
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: Keys.self)
        switch self {
        case .taste(let text):
            try container.encode("taste", forKey: .kind)
            try container.encode(text, forKey: .text)
        case .feedback(let uri, let title, let artist, let rating):
            try container.encode("feedback", forKey: .kind)
            try container.encode(uri, forKey: .uri)
            try container.encode(title, forKey: .title)
            try container.encode(artist, forKey: .artist)
            try container.encode(rating, forKey: .rating)
        case .mix(let title, let uris):
            try container.encode("mix", forKey: .kind)
            try container.encode(title, forKey: .title)
            try container.encode(uris, forKey: .uris)
        case .history(let prompt, let suggestions):
            try container.encode("history", forKey: .kind)
            try container.encode(prompt, forKey: .prompt)
            try container.encode(suggestions, forKey: .suggestions)
        }
    }
}

public struct Record: Codable, Hashable, Sendable {
    public var stamp: Stamp
    /// `nil` is a durable tombstone, not an absent record.
    public var value: Value?

    enum CodingKeys: String, CodingKey { case stamp, value }

    public init(stamp: Stamp, value: Value?) {
        self.stamp = stamp
        self.value = value
    }

    public init(from decoder: any Decoder) throws {
        try requireExactKeys(decoder, ["stamp", "value"])
        let container = try decoder.container(keyedBy: CodingKeys.self)
        stamp = try container.decode(Stamp.self, forKey: .stamp)
        value = try container.decode(Value?.self, forKey: .value)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(stamp, forKey: .stamp)
        // The server requires an explicit null for a tombstone.
        try container.encode(value, forKey: .value)
    }
}

// MARK: - Document

public struct Document: Codable, Hashable, Sendable {
    public var version: UInt32 = 1
    public var records: [String: Record] = [:]

    public init() {}

    public init(from decoder: any Decoder) throws {
        try requireExactKeys(decoder, ["version", "records"])
        let container = try decoder.container(keyedBy: CodingKeys.self)
        version = try container.decode(UInt32.self, forKey: .version)
        records = try container.decode([String: Record].self, forKey: .records)
    }

    public func encoded() throws -> Data {
        try discoveryEncoder.encode(self)
    }

    public func validate() throws(DiscoveryError) {
        if version != 1 || records.count > DiscoveryLimits.maxRecords {
            throw DiscoveryError("Unsupported or oversized discovery state. Your local state is preserved.")
        }
        try validateRecords()
        guard let bytes = try? encoded() else { throw DiscoveryError("Cannot encode discovery state.") }
        if bytes.count > DiscoveryLimits.maxBytes {
            throw DiscoveryError("Discovery storage is full. Export and remove old mixes or history.")
        }
    }

    func validateRecords() throws(DiscoveryError) {
        for (key, record) in records {
            if key.utf8.count > 200 || !isDeviceID(record.stamp.device) || record.stamp.counter == 0 || record.stamp.counter == .max {
                throw DiscoveryError("Invalid discovery record.")
            }
            let valid: Bool = switch record.value {
            case .taste(let text):
                key == "taste" && text.utf8.count <= DiscoveryLimits.maxPromptBytes
            case .feedback(let uri, let title, let artist, _):
                key == "feedback:\(uri)" && isSpotifyTrack(uri) && title.utf8.count <= 300 && artist.utf8.count <= 300
            case .mix(let title, let uris):
                key.hasPrefix("mix:") && title.utf8.count <= 300 && uris.count <= 100 && uris.allSatisfy(isSpotifyTrack)
            case .history(let prompt, let suggestions):
                key.hasPrefix("history:") && prompt.utf8.count <= DiscoveryLimits.maxPromptBytes && validSuggestions(suggestions)
            case nil:
                key == "taste" || ["feedback:", "mix:", "history:"].contains { key.hasPrefix($0) }
            }
            if !valid { throw DiscoveryError("Invalid discovery record content.") }
        }
    }

    /// Same deterministic forgetting boundary and tied-cohort rule as Rust.
    mutating func compactFeedback() {
        var floor = records[DiscoveryLimits.feedbackFloor]?.stamp
        let stamps = records.filter { $0.key.hasPrefix("feedback:") && $0.key != DiscoveryLimits.feedbackFloor }
            .map { $0.value.stamp }.sorted(by: >)
        if stamps.count > DiscoveryLimits.feedbackLimit {
            let pruned = stamps[DiscoveryLimits.feedbackLimit]
            floor = floor.map { max($0, pruned) } ?? pruned
        }
        if let floor {
            records = records.filter { !$0.key.hasPrefix("feedback:") || $0.key == DiscoveryLimits.feedbackFloor || $0.value.stamp > floor }
            records[DiscoveryLimits.feedbackFloor] = Record(stamp: floor, value: nil)
        }
    }

    /// Validate the entire remote snapshot before touching local records.
    public mutating func merge(_ other: Document) throws(DiscoveryError) {
        try other.validate()
        var merged = self
        for (key, remote) in other.records {
            if let local = merged.records[key] {
                if local.stamp == remote.stamp && local.value != remote.value {
                    throw DiscoveryError("Discovery clock collision. Your local state is preserved.")
                }
                if local.stamp >= remote.stamp { continue }
            }
            merged.records[key] = remote
        }
        merged.compactFeedback()
        try merged.validate()
        self = merged
    }

    public var taste: String {
        if case .taste(let text) = records["taste"]?.value { return text }
        return ""
    }

    /// Stamp-only changes are immaterial; removed feedback is an input change.
    public func sameInputs(as other: Document) -> Bool {
        taste == other.taste && feedback == other.feedback
    }

    public var hasInputs: Bool {
        !taste.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            || records.values.contains { if case .feedback = $0.value { true } else { false } }
    }

    public func rating(_ uri: String) -> Rating? {
        if case .feedback(_, _, _, let rating) = records["feedback:\(uri)"]?.value { return rating }
        return nil
    }

    public struct FeedbackEntry: Codable, Hashable, Sendable {
        public var title: String
        public var artist: String
        public var rating: Rating
    }

    /// The newest hundred ratings, newest first: what the AI may see.
    public var feedback: [FeedbackEntry] {
        records.values
            .filter { if case .feedback = $0.value { true } else { false } }
            .sorted { $0.stamp > $1.stamp }
            .prefix(100)
            .compactMap {
                if case .feedback(_, let title, let artist, let rating) = $0.value {
                    FeedbackEntry(title: title, artist: artist, rating: rating)
                } else { nil }
            }
    }

    /// Live history entries, newest ten.
    public var recentHistory: [(key: String, record: Record)] {
        records
            .filter { if case .history = $0.value.value { true } else { false } }
            .sorted { ($0.value.stamp, $0.key) > ($1.value.stamp, $1.key) }
            .prefix(DiscoveryLimits.historyLimit)
            .map { (key: $0.key, record: $0.value) }
    }

    /// Saved mixes, newest first.
    public var mixes: [(key: String, title: String, uris: [String])] {
        records
            .compactMap { key, record -> (Stamp, String, String, [String])? in
                if case .mix(let title, let uris) = record.value { (record.stamp, key, title, uris) } else { nil }
            }
            .sorted { ($0.0, $0.1) > ($1.0, $1.1) }
            .map { (key: $0.1, title: $0.2, uris: $0.3) }
    }
}

public func validSuggestions(_ suggestions: [Suggestion]) -> Bool {
    !suggestions.isEmpty && suggestions.count <= 12 && suggestions.allSatisfy {
        !$0.title.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !$0.artist.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && $0.title.utf8.count <= 300 && $0.artist.utf8.count <= 300 && $0.reason.utf8.count <= 600
    }
}

// MARK: - Local replica

/// A catalogue track as Spotiurge shows and plays it. Local only.
public struct Track: Codable, Hashable, Sendable, Identifiable {
    public var uri: String
    public var name: String
    public var artists: [String]
    public var durationMs: Int
    public var album: String?
    public var imageURL: URL?
    public var isPlayable: Bool?

    public var id: String { uri }
    public var artistLine: String { artists.joined(separator: ", ") }

    public init(uri: String, name: String, artists: [String], durationMs: Int, album: String? = nil, imageURL: URL? = nil, isPlayable: Bool? = nil) {
        self.uri = uri
        self.name = name
        self.artists = artists
        self.durationMs = durationMs
        self.album = album
        self.imageURL = imageURL
        self.isPlayable = isPlayable
    }
}

public struct Pick: Codable, Hashable, Sendable {
    public var suggestion: Suggestion
    public var track: Track?
    public var checked: Bool

    public init(suggestion: Suggestion, track: Track? = nil, checked: Bool = false) {
        self.suggestion = suggestion
        self.track = track
        self.checked = checked
    }
}

public struct Replica: Codable, Sendable {
    public var device: String
    public var document: Document
    /// Local catalogue matches, excluded from the cloud document and AI input.
    public var cachedPicks: [Pick]
    /// Current unsynced rating stamps; local only, bounded by retained feedback.
    public var pendingFeedback: [String: Stamp] = [:]

    enum CodingKeys: String, CodingKey {
        case device, document
        case cachedPicks = "cached_picks"
        case pendingFeedback = "pending_feedback"
    }

    public init(device: String = newDeviceID(), document: Document = Document(), cachedPicks: [Pick] = []) {
        self.device = device
        self.document = document
        self.cachedPicks = cachedPicks
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        device = try container.decode(String.self, forKey: .device)
        document = try container.decode(Document.self, forKey: .document)
        cachedPicks = try container.decodeIfPresent([Pick].self, forKey: .cachedPicks) ?? []
        pendingFeedback = try container.decodeIfPresent([String: Stamp].self, forKey: .pendingFeedback) ?? [:]
    }

    /// Import the remote clock without confusing fresh offline intent with old replicas.
    public mutating func mergeForSync(_ remote: Document) throws(DiscoveryError) {
        let pending = pendingFeedback.compactMap { key, stamp -> (key: String, value: Value?)? in
            guard let record = document.records[key], record.stamp == stamp else { return nil }
            return (key, record.value)
        }
        var merged = self
        try merged.document.merge(remote)
        let lost = pending.filter { merged.document.records[$0.key] == nil }
        if !lost.isEmpty { try merged.editMany(lost) }
        merged.keepCurrentPending()
        self = merged
    }

    private mutating func keepCurrentPending() {
        pendingFeedback = pendingFeedback.filter { document.records[$0.key]?.stamp == $0.value }
    }

    /// Merge an acknowledged sync without undoing edits made after its snapshot.
    public mutating func mergeSynced(_ remote: Document, snapshot: Document?) throws(DiscoveryError) {
        let changed = snapshot.map { snapshot in
            document.records.filter { $0.key != DiscoveryLimits.feedbackFloor && snapshot.records[$0.key] != $0.value }
        } ?? [:]
        var merged = self
        try merged.document.merge(remote)
        // Incorporate the remote clock first, then express the newer local
        // intent above it in one write. Tombstones are edits as well.
        let edits = changed
            .filter { merged.document.records[$0.key] != $0.value }
            .map { (key: $0.key, value: $0.value.value) }
        if !edits.isEmpty { try merged.editMany(edits) }
        if let snapshot {
            merged.pendingFeedback = merged.pendingFeedback.filter { snapshot.records[$0.key]?.stamp != $0.value }
        }
        merged.keepCurrentPending()
        self = merged
    }

    public mutating func edit(_ key: String, _ value: Value?) throws(DiscoveryError) {
        try editMany([(key: key, value: value)])
    }

    /// Store an AI result in a bounded set of history keys. Below the limit it
    /// adds one; after that it overwrites the oldest history key, live or
    /// tombstone. Live entries beyond the limit become tombstones in the same
    /// write. Independent feedback retention also applies to every edit.
    /// Reuse removed slots, then the oldest slot once 100 already exist.
    public mutating func saveMix(title: String, uris: [String]) throws(DiscoveryError) {
        let mixes = document.records.filter { $0.key.hasPrefix("mix:") }
        let oldest: (Dictionary<String, Record>.Element, Dictionary<String, Record>.Element) -> Bool = {
            $0.value.stamp == $1.value.stamp ? $0.key < $1.key : $0.value.stamp < $1.value.stamp
        }
        let removed = mixes.filter { $0.value.value == nil }.min(by: oldest)
        let target = removed?.key ?? (mixes.count >= DiscoveryLimits.mixLimit ? mixes.min(by: oldest)?.key : nil)
        try edit(target ?? "mix:\(newDeviceID())", .mix(title: title, uris: uris))
    }

    public mutating func recordHistory(prompt: String, suggestions: [Suggestion]) throws(DiscoveryError) {
        var history = document.records
            .filter { $0.key.hasPrefix("history:") }
            .map { (stamp: $0.value.stamp, key: $0.key, live: $0.value.value != nil) }
            .sorted { ($0.stamp, $0.key) > ($1.stamp, $1.key) }
        let target = history.count < DiscoveryLimits.historyLimit
            ? "history:\(newDeviceID())"
            : history.removeLast().key
        var edits: [(key: String, value: Value?)] = history
            .filter(\.live)
            .dropFirst(DiscoveryLimits.historyLimit - 1)
            .map { (key: $0.key, value: nil) }
        edits.append((key: target, value: .history(prompt: prompt, suggestions: suggestions)))
        try editMany(edits)
    }

    /// Apply edits atomically under one logical clock above every record.
    mutating func editMany(_ edits: [(key: String, value: Value?)]) throws(DiscoveryError) {
        let highest = document.records.values.map(\.stamp.counter).max() ?? 0
        guard highest < .max - 1 else { throw DiscoveryError("Discovery clock is exhausted.") }
        var candidate = document
        for edit in edits {
            candidate.records[edit.key] = Record(stamp: Stamp(counter: highest + 1, device: device), value: edit.value)
        }
        try candidate.validateRecords()
        candidate.compactFeedback()
        try candidate.validate()
        for (key, record) in candidate.records where key.hasPrefix("feedback:") && key != DiscoveryLimits.feedbackFloor {
            if document.records[key] != record && record.stamp.device == device && record.stamp.counter == highest + 1 {
                pendingFeedback[key] = record.stamp
            }
        }
        document = candidate
        keepCurrentPending()
    }

    /// Reads a stored replica. Backups and restored copies must not keep a
    /// live writer's identity, so every load gets a fresh writer; old stamps
    /// stay intact and the next edit advances past the largest counter.
    public static func load(from url: URL) throws(DiscoveryError) -> Replica {
        let data: Data
        do {
            data = try Data(contentsOf: url)
        } catch {
            if (error as? CocoaError)?.code == .fileReadNoSuchFile { return Replica() }
            throw DiscoveryError("Cannot read discovery state. Restart Spotiurge.")
        }
        if data.count > DiscoveryLimits.maxBytes * 2 { throw DiscoveryError("Discovery state is too large.") }
        guard var replica = try? JSONDecoder().decode(Replica.self, from: data) else {
            throw DiscoveryError("Cannot read discovery state. The original file is preserved.")
        }
        try replica.document.validate()
        guard replica.pendingFeedback.count <= DiscoveryLimits.feedbackLimit,
              replica.pendingFeedback.allSatisfy({ key, stamp in
                  key.hasPrefix("feedback:") && key != DiscoveryLimits.feedbackFloor && replica.document.records[key]?.stamp == stamp
              }) else { throw DiscoveryError("Invalid pending discovery feedback.") }
        replica.document.compactFeedback()
        replica.keepCurrentPending()
        if replica.cachedPicks.count > 12 { throw DiscoveryError("Invalid cached discovery results.") }
        if !isDeviceID(replica.device) { throw DiscoveryError("Invalid discovery installation identity.") }
        replica.device = newDeviceID()
        return replica
    }

    /// Writes through a temporary file and an atomic rename.
    public func save(to url: URL) throws(DiscoveryError) {
        try document.validate()
        guard let data = try? discoveryEncoder.encode(self), data.count <= DiscoveryLimits.maxBytes * 2 else {
            throw DiscoveryError("Local discovery storage is full.")
        }
        do {
            try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
            #if os(iOS)
            try data.write(to: url, options: [.atomic, .completeFileProtectionUntilFirstUserAuthentication])
            #else
            try data.write(to: url, options: .atomic)
            #endif
        } catch {
            throw DiscoveryError("Cannot save discovery state. Your edits remain in memory.")
        }
    }
}

// MARK: - Preferences and automatic picks

public struct Preferences: Codable, Equatable, Sendable {
    public var automatic = true
    public var exploration = Exploration.balanced
    public var refreshedAt: UInt64?

    public init() {}

    enum CodingKeys: String, CodingKey {
        case automatic, exploration
        case refreshedAt = "refreshed_at"
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        automatic = (try? container.decode(Bool.self, forKey: .automatic)) ?? true
        exploration = (try? container.decode(Exploration.self, forKey: .exploration)) ?? .balanced
        refreshedAt = try? container.decode(UInt64.self, forKey: .refreshedAt)
    }
}

public enum RecommendationErrorKind: Sendable, Equatable {
    case pairing, busy, rateLimited, unavailable, invalidResponse
}

/// When to ask for fresh picks without a tap. Same schedule as the desktop:
/// at most every twelve hours, 1.5 s after a taste or exploration change,
/// 45 s (and at least ten minutes after the last attempt) after feedback,
/// exponential backoff on failure, and suspension until re-paired.
public struct AutoRecommendations: Sendable {
    public var failures: UInt32 = 0
    public var suspended = false
    var retryAt: ContinuousClock.Instant?
    var changedAt: ContinuousClock.Instant?
    var lastAttempt: ContinuousClock.Instant?

    public init() {}

    public func retryAfter(now: ContinuousClock.Instant) -> Duration? {
        guard let retryAt, retryAt > now else { return nil }
        return retryAt - now
    }

    public mutating func changed(now: ContinuousClock.Instant, exploration: Bool) {
        var due = now + .milliseconds(exploration ? 1500 : 45_000)
        if !exploration, let lastAttempt { due = max(due, lastAttempt + .seconds(600)) }
        changedAt = due
    }

    public func due(now: ContinuousClock.Instant, wallNow: UInt64, preferences: Preferences, enabled: Bool, hasInputs: Bool, empty: Bool) -> Duration? {
        guard enabled, hasInputs, preferences.automatic, !suspended else { return nil }
        if let retryAt { return max(.zero, retryAt - now) }
        if let changedAt { return max(.zero, changedAt - now) }
        if empty && lastAttempt == nil { return .zero }
        let refreshed = preferences.refreshedAt.flatMap { $0 <= wallNow &+ 43_200 ? $0 : nil } ?? 0
        let next = refreshed &+ 43_200
        return .seconds(next > wallNow ? next - wallNow : 0)
    }

    public mutating func attempted(now: ContinuousClock.Instant) {
        lastAttempt = now
        changedAt = nil
        retryAt = nil
    }

    public mutating func failed(_ kind: RecommendationErrorKind, now: ContinuousClock.Instant) {
        if kind == .pairing {
            suspended = true
            return
        }
        failures = min(failures + 1, 10)
        let seconds = min(UInt64(600) << (failures - 1), 21_600)
        retryAt = now + .seconds(seconds)
    }

    public mutating func rearm() {
        failures = 0
        suspended = false
        retryAt = nil
    }
}

// MARK: - Coding helpers

let discoveryEncoder: JSONEncoder = {
    let encoder = JSONEncoder()
    // Match serde_json's compact form so byte limits agree with the server.
    encoder.outputFormatting = [.withoutEscapingSlashes]
    return encoder
}()

struct AnyKey: CodingKey {
    var stringValue: String
    var intValue: Int? { nil }
    init(stringValue: String) { self.stringValue = stringValue }
    init?(intValue: Int) { nil }
}

/// Unknown schemas fail closed, like serde's `deny_unknown_fields`.
func requireExactKeys(_ decoder: any Decoder, _ keys: Set<String>) throws {
    let container = try decoder.container(keyedBy: AnyKey.self)
    let present = Set(container.allKeys.map(\.stringValue))
    guard present == keys else {
        throw DecodingError.dataCorrupted(.init(codingPath: decoder.codingPath, debugDescription: "Unexpected fields"))
    }
}
