// Ports of the src/discovery.rs merge and scheduling tests, plus wire-format
// checks against the desktop/server JSON shape.

import Foundation
import Testing
@testable import SpotiurgeCore

func device(_ id: Character) -> Replica {
    Replica(device: String(repeating: id, count: 32))
}

func taste(_ text: String) -> Value { .taste(text: text) }
let uri = "spotify:track:0123456789ABCDEFGHIJKL"

@Test func `offline edits converge without wall clocks`() throws {
    var a = device("a"), b = device("b")
    try a.edit("taste", taste("warm strings"))
    try b.edit("taste", taste("broken rhythms"))
    try a.edit("mix:morning", .mix(title: "Morning", uris: []))
    let original = a.document
    try a.document.merge(b.document)
    try b.document.merge(original)
    #expect(a.document == b.document)
    #expect(a.document.taste == "broken rhythms")
    #expect(a.document.records["mix:morning"] != nil)
}

@Test func `clearing feedback converges and retains a tombstone`() throws {
    var a = device("a"), b = device("b")
    try a.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
    try b.document.merge(a.document)
    try b.edit("feedback:\(uri)", nil)
    try a.document.merge(b.document)
    #expect(a.document.rating(uri) == nil)
    #expect(a.document.records["feedback:\(uri)"]?.value == nil)
    #expect(a.document == b.document)
}

@Test func `sync acknowledgment preserves later local taste above the remote clock`() throws {
    var local = device("a")
    try local.edit("taste", taste("before sync"))
    let snapshot = local.document
    var remote = device("b")
    for _ in 0..<10 { try remote.edit("taste", taste("remote taste")) }
    try local.edit("taste", taste("just saved here"))
    try local.mergeSynced(remote.document, snapshot: snapshot)
    #expect(local.document.taste == "just saved here")
    #expect(local.document.records["taste"]!.stamp.counter > 10)
    try remote.document.merge(local.document)
    #expect(remote.document == local.document)
}

@Test func `sync preserves in-flight tombstones and rejects invalid replies whole`() throws {
    var local = device("a")
    try local.edit("mix:removed", .mix(title: "old mix", uris: []))
    let snapshot = local.document
    try local.edit("mix:removed", nil)
    var remote = device("b")
    for _ in 0..<10 { try remote.edit("mix:removed", .mix(title: "remote mix", uris: [])) }
    try remote.edit("taste", taste("remote preference"))
    try local.mergeSynced(remote.document, snapshot: snapshot)
    #expect(local.document.taste == "remote preference")
    #expect(local.document.records["mix:removed"]?.value == nil)
    #expect(local.document.records["mix:removed"]!.stamp.counter > 11)
    try remote.document.merge(local.document)
    #expect(remote.document == local.document)

    let saved = local.document
    var invalid = remote.document
    invalid.records["taste"]!.stamp.counter = .max
    #expect(throws: DiscoveryError.self) { try local.mergeSynced(invalid, snapshot: snapshot) }
    #expect(local.document == saved)
}

@Test func `stale responses cannot undo deletions or edits`() throws {
    var a = device("a")
    try a.edit("mix:one", .mix(title: "One", uris: []))
    let stale = a.document
    try a.edit("mix:one", nil)
    try a.edit("taste", taste("new direction"))
    try a.document.merge(stale)
    #expect(a.document.records["mix:one"]?.value == nil)
    #expect(a.document.taste == "new direction")
}

@Test func `invalid or colliding remote never changes local state`() throws {
    var a = device("a")
    try a.edit("taste", taste("mine"))
    let before = a.document
    var bad = before
    bad.records["taste"]!.value = taste("collision")
    #expect(throws: DiscoveryError.self) { try a.document.merge(bad) }
    #expect(a.document == before)
    bad.version = 99
    #expect(throws: DiscoveryError.self) { try a.document.merge(bad) }
    #expect(a.document == before)
}

@Test func `model output cannot supply playback uris`() {
    #expect(!isSpotifyTrack("spotify:track:invented"))
    #expect(!isSpotifyTrack("https://example.com/music"))
    #expect(isSpotifyTrack(uri))
    #expect(!validSuggestions([]))
}

@Test(arguments: [
    String(repeating: "a", count: 4001), String(repeating: "音", count: 1334),
    String(repeating: "🎵", count: 1001), String(repeating: "a", count: 3999) + "🎵",
])
func `multibyte prompts fit taste and history storage`(input: String) throws {
    let text = limitPrompt(input)
    #expect(text.utf8.count <= DiscoveryLimits.maxPromptBytes)
    #expect(input.hasPrefix(text))
    var replica = device("a")
    try replica.edit("taste", taste(text))
    try replica.edit("history:test", .history(prompt: text, suggestions: [Suggestion(title: "Song", artist: "Artist", reason: "")]))
}

@Test func `repeated refreshes reuse a bounded set of history keys`() throws {
    var replica = device("a")
    try replica.edit("taste", taste("kept taste"))
    for refresh in 0..<30 {
        let suggestions = (0..<12).map { Suggestion(title: "\(refresh) \($0)", artist: "Artist", reason: "") }
        try replica.recordHistory(prompt: "p", suggestions: suggestions)
        let history = replica.document.records.filter { $0.key.hasPrefix("history:") }
        #expect(history.count == min(refresh + 1, DiscoveryLimits.historyLimit))
        #expect(history.values.allSatisfy { $0.value != nil })
    }
    let recent = replica.document.recentHistory
    #expect(recent.count == DiscoveryLimits.historyLimit)
    guard case .history(_, let suggestions) = recent[0].record.value else { Issue.record("newest is live"); return }
    #expect(suggestions[0].title.hasPrefix("29 "))
    #expect(replica.document.records["taste"]!.stamp.counter == 1)
}

@Test func `copied replicas get independent writers`() throws {
    let url = FileManager.default.temporaryDirectory.appending(path: "spotiurge-\(newDeviceID()).json")
    defer { try? FileManager.default.removeItem(at: url) }
    var original = device("a")
    try original.edit("taste", taste("original"))
    try original.save(to: url)
    var a = try Replica.load(from: url), b = try Replica.load(from: url)
    #expect(a.document == original.document)
    #expect(a.device != original.device && a.device != b.device)
    try a.edit("taste", taste("first copy"))
    try b.edit("taste", taste("second copy"))
    #expect(a.document.records["taste"]!.stamp.counter == 2)
    let oldA = a.document
    try a.document.merge(b.document)
    try b.document.merge(oldA)
    #expect(a.document == b.document)
    // A missing file is a fresh install, not an error.
    #expect(try Replica.load(from: url.deletingLastPathComponent().appending(path: "spotiurge-missing-\(newDeviceID()).json")).document == Document())
}

// MARK: - Wire format

@Test func `tombstones encode an explicit null and desktop documents decode`() throws {
    var replica = device("a")
    try replica.edit("mix:x", nil)
    let json = String(decoding: try replica.document.encoded(), as: UTF8.self)
    #expect(json.contains("\"value\":null"))
    let desktop = #"""
    {"version":1,"records":{"taste":{"stamp":{"counter":3,"device":"0123456789abcdef0123456789abcdef"},"value":{"kind":"taste","text":"Warm jazz/soul"}},
    "feedback:spotify:track:0123456789ABCDEFGHIJKL":{"stamp":{"counter":4,"device":"0123456789abcdef0123456789abcdef"},"value":{"kind":"feedback","uri":"spotify:track:0123456789ABCDEFGHIJKL","title":"Song","artist":"A, B","rating":"less"}},
    "history:1":{"stamp":{"counter":5,"device":"0123456789abcdef0123456789abcdef"},"value":{"kind":"history","prompt":"p","suggestions":[{"title":"T","artist":"A","reason":"R"}]}}}}
    """#
    let document = try JSONDecoder().decode(Document.self, from: Data(desktop.utf8))
    try document.validate()
    #expect(document.taste == "Warm jazz/soul")
    #expect(document.rating(uri) == .less)
    #expect(String(decoding: try document.encoded(), as: UTF8.self).contains("jazz/soul"))
}

@Test(arguments: [
    #"{"version":1,"records":{},"extra":1}"#,
    #"{"version":1,"records":{"taste":{"stamp":{"counter":1,"device":"0123456789abcdef0123456789abcdef"},"value":{"kind":"taste","text":"x","mood":"y"}}}}"#,
    #"{"version":1,"records":{"taste":{"stamp":{"counter":1,"device":"0123456789abcdef0123456789abcdef","wall":2},"value":null}}}"#,
    #"{"version":1,"records":{"taste":{"stamp":{"counter":1,"device":"0123456789abcdef0123456789abcdef"},"value":{"kind":"radio"}}}}"#,
])
func `unknown schemas fail closed`(json: String) {
    #expect(throws: (any Error).self) { try JSONDecoder().decode(Document.self, from: Data(json.utf8)) }
}

@Test func `feedback sent to the AI carries no URIs`() throws {
    var replica = device("a")
    try replica.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
    let json = String(decoding: try JSONEncoder().encode(replica.document.feedback), as: UTF8.self)
    #expect(!json.contains("spotify:"))
    #expect(json.contains("\"rating\":\"love\""))
}

// MARK: - Automatic picks

@Test func `automatic picks use freshness and never run without inputs`() {
    let now = ContinuousClock.now
    let scheduler = AutoRecommendations()
    var preferences = Preferences()
    preferences.refreshedAt = 100_000
    #expect(scheduler.due(now: now, wallNow: 100_001, preferences: preferences, enabled: true, hasInputs: true, empty: false) == .seconds(43_199))
    #expect(scheduler.due(now: now, wallNow: 143_200, preferences: preferences, enabled: true, hasInputs: true, empty: false) == .zero)
    #expect(scheduler.due(now: now, wallNow: 100_001, preferences: preferences, enabled: true, hasInputs: true, empty: true) == .zero)
    #expect(scheduler.due(now: now, wallNow: 143_200, preferences: preferences, enabled: false, hasInputs: true, empty: true) == nil)
    #expect(scheduler.due(now: now, wallNow: 143_200, preferences: preferences, enabled: true, hasInputs: false, empty: true) == nil)
    preferences.automatic = false
    #expect(scheduler.due(now: now, wallNow: 143_200, preferences: preferences, enabled: true, hasInputs: true, empty: true) == nil)
}

@Test func `automatic picks debounce feedback and respect backoff`() {
    let now = ContinuousClock.now
    let preferences = Preferences()
    var scheduler = AutoRecommendations()
    scheduler.attempted(now: now)
    for seconds in [10, 20, 30] { scheduler.changed(now: now + .seconds(seconds), exploration: false) }
    #expect(scheduler.due(now: now, wallNow: 100_000, preferences: preferences, enabled: true, hasInputs: true, empty: false) == .seconds(600))
    scheduler.changed(now: now, exploration: true)
    #expect(scheduler.due(now: now, wallNow: 100_000, preferences: preferences, enabled: true, hasInputs: true, empty: false) == .milliseconds(1500))
    for seconds in [600, 1200, 2400, 4800, 9600, 19_200, 21_600, 21_600] {
        scheduler.failed(.rateLimited, now: now)
        scheduler.changed(now: now, exploration: true)
        #expect(scheduler.due(now: now, wallNow: 100_000, preferences: preferences, enabled: true, hasInputs: true, empty: false) == .seconds(seconds))
        scheduler.attempted(now: now + .seconds(seconds))
    }
    scheduler.failed(.pairing, now: now)
    #expect(scheduler.due(now: now, wallNow: 100_000, preferences: preferences, enabled: true, hasInputs: true, empty: false) == nil)
    scheduler.rearm()
    #expect(scheduler.failures == 0 && !scheduler.suspended)
}

@Test func `partial preferences remain readable`() throws {
    #expect(try JSONDecoder().decode(Preferences.self, from: Data("{}".utf8)) == Preferences())
    let adventurous = try JSONDecoder().decode(Preferences.self, from: Data(#"{"exploration":"adventurous"}"#.utf8))
    #expect(adventurous.automatic && adventurous.exploration == .adventurous)
}

// MARK: - Server parity

/// Runs services/private-cloud/server.py's own `valid_document` on what this
/// package writes, so the iPhone and the server agree on the wire format.
@Test(.enabled(if: FileManager.default.isExecutableFile(atPath: "/usr/bin/python3")))
func `the private cloud accepts documents written here`() throws {
    var replica = device("a")
    try replica.edit("taste", taste("Warm jazz/soul, \u{97F3}\u{697D} and \"quotes\""))
    try replica.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "A, B", rating: .less))
    try replica.edit("mix:m", .mix(title: "Mix", uris: [uri]))
    try replica.edit("mix:gone", nil)
    try replica.recordHistory(prompt: "p", suggestions: [Suggestion(title: "T", artist: "A", reason: "R")])
    let service = URL(filePath: #filePath).deletingLastPathComponent().appending(path: "../../../../../services/private-cloud").standardized
    let python = Process()
    python.executableURL = URL(filePath: "/usr/bin/python3")
    python.arguments = ["-I", "-B", "-c", "import json,sys; sys.path.insert(0, sys.argv[1]); import server; sys.exit(0 if server.valid_document(json.load(sys.stdin)) else 1)", service.path]
    let input = Pipe()
    python.standardInput = input
    try python.run()
    input.fileHandleForWriting.write(try replica.document.encoded())
    try input.fileHandleForWriting.close()
    python.waitUntilExit()
    #expect(python.terminationStatus == 0)
}

func feedbackEdit(_ index: Int, clear: Bool = false) -> (String, Value?) {
    let uri = "spotify:track:" + String(format: "%022d", index)
    return ("feedback:\(uri)", clear ? nil : .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
}

@Test func `feedback clears stay bounded and stale records cannot return`() throws {
    var a = device("a")
    let (oldKey, oldValue) = feedbackEdit(0)
    try a.edit(oldKey, oldValue)
    let stale = a.document
    for index in 1..<2100 {
        let (key, value) = feedbackEdit(index, clear: index % 2 == 0)
        try a.edit(key, value)
    }
    #expect(a.document.records.count == DiscoveryLimits.feedbackLimit + 1)
    #expect(a.document.records[DiscoveryLimits.feedbackFloor]?.stamp.counter == 1600)
    let compacted = a.document
    try a.document.merge(stale)
    #expect(a.document == compacted)
    var old = stale
    try old.merge(compacted)
    #expect(old == compacted)
    try a.edit(oldKey, oldValue)
    #expect(a.document.records[oldKey]?.value != nil)
    try a.edit("mix:new", .mix(title: "Still works", uris: []))
    #expect(a.document.records.count == DiscoveryLimits.feedbackLimit + 2)
}

@Test func `legacy feedback union compacts before capacity validation`() throws {
    var a = device("a"), b = device("b")
    for index in 0..<1200 {
        let (key, value) = feedbackEdit(index)
        a.document.records[key] = Record(stamp: Stamp(counter: UInt64(index + 1), device: a.device), value: value)
        let (otherKey, otherValue) = feedbackEdit(index + 1200, clear: true)
        b.document.records[otherKey] = Record(stamp: Stamp(counter: UInt64(index + 1), device: b.device), value: otherValue)
    }
    var left = a.document
    try left.merge(b.document)
    try b.document.merge(a.document)
    #expect(left == b.document)
    #expect(left.records.count == DiscoveryLimits.feedbackLimit + 1)
    var tied = device("c")
    try tied.editMany((0...DiscoveryLimits.feedbackLimit).map { let (key, value) = feedbackEdit($0); return (key: key, value: value) })
    #expect(tied.document.records.count == 1)
}

@Test func `sync reexpresses newer feedback without restamping its cutoff`() throws {
    var local = device("a")
    for index in 0...DiscoveryLimits.feedbackLimit {
        let (key, value) = feedbackEdit(index)
        try local.edit(key, value)
    }
    let snapshot = local.document
    let (key, value) = feedbackEdit(DiscoveryLimits.feedbackLimit + 1)
    try local.edit(key, value)
    var remote = device("b")
    try remote.document.merge(snapshot)
    for _ in 0..<10 { try remote.edit("taste", taste("remote clock")) }
    try remote.edit(key, value)
    try local.mergeSynced(remote.document, snapshot: snapshot)
    #expect(local.document.records[key]!.stamp.counter > remote.document.records[key]!.stamp.counter)
    #expect(local.document.records[DiscoveryLimits.feedbackFloor]?.stamp.counter == 2)
    try remote.document.merge(local.document)
    #expect(remote.document == local.document)
}

@Test func `unsent offline ratings survive an advanced floor and an acknowledged rating stays forgotten`() throws {
    var local = device("a")
    let key = "feedback:\(uri)"
    let value = Value.feedback(uri: uri, title: "Song", artist: "Artist", rating: .love)
    try local.edit(key, value)
    let snapshot = local.document
    let bytes = try JSONEncoder().encode(local)
    var restarted = try JSONDecoder().decode(Replica.self, from: bytes)
    restarted.device = String(repeating: "c", count: 32)
    var remote = Document()
    remote.records[DiscoveryLimits.feedbackFloor] = Record(stamp: Stamp(counter: 100, device: String(repeating: "b", count: 32)), value: nil)
    try restarted.mergeForSync(remote)
    #expect(restarted.document.records[key]?.value == value)
    #expect(restarted.document.records[key]?.stamp.counter == 101)
    try local.mergeSynced(restarted.document, snapshot: snapshot)
    #expect(local.pendingFeedback.isEmpty)
    remote.records[DiscoveryLimits.feedbackFloor]!.stamp.counter = 200
    try local.mergeForSync(remote)
    #expect(local.document.records[key] == nil)
}

@Test func `effective inputs include forgotten feedback but exclude stamp changes`() throws {
    var local = device("a")
    try local.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
    let before = local.document
    var remote = Document()
    remote.records[DiscoveryLimits.feedbackFloor] = Record(stamp: Stamp(counter: 100, device: String(repeating: "b", count: 32)), value: nil)
    try local.document.merge(remote)
    #expect(!before.sameInputs(as: local.document))
    var stampOnly = before
    stampOnly.records["feedback:\(uri)"]!.stamp.counter += 1
    #expect(before.sameInputs(as: stampOnly))
}

@Test func `manual retry obeys the deadline`() {
    let now = ContinuousClock.now
    var scheduler = AutoRecommendations()
    scheduler.failed(.rateLimited, now: now)
    #expect(scheduler.retryAfter(now: now) == .seconds(600))
    #expect(scheduler.retryAfter(now: now + .seconds(599)) != nil)
    #expect(scheduler.retryAfter(now: now + .seconds(600)) == nil)
}

@Test func `local recommendation checkpoints retain restart limits`() throws {
    let now = ContinuousClock.now
    var scheduler = AutoRecommendations()
    scheduler.attempted(now: now)
    scheduler.rearm()
    var replica = Replica()
    replica.recommendationThrottle = scheduler.checkpoint(now: now, wallNow: 100_000)
    let stored = try JSONDecoder().decode(Replica.self, from: JSONEncoder().encode(replica))
    var restored = AutoRecommendations(saved: stored.recommendationThrottle, now: now, wallNow: 100_010)
    restored.changed(now: now, exploration: false)
    #expect(restored.due(now: now, wallNow: 100_010, preferences: Preferences(), enabled: true, hasInputs: true, empty: false) == .seconds(590))
    scheduler.failed(.rateLimited, now: now)
    let cooldown = scheduler.checkpoint(now: now, wallNow: 100_000)
    restored = AutoRecommendations(saved: cooldown, now: now, wallNow: 100_010)
    #expect(restored.retryAfter(now: now) == .seconds(590))
    restored.attempted(now: now + .seconds(590))
    restored.failed(.rateLimited, now: now + .seconds(590))
    #expect(restored.retryAfter(now: now + .seconds(590)) == .seconds(1200))
    #expect(AutoRecommendations(saved: cooldown, now: now, wallNow: 100_600).retryAfter(now: now) == nil)
    var skewed = cooldown
    skewed.retryAt = UInt64.max
    skewed.failures = UInt32.max
    skewed.suspended = true
    let bounded = AutoRecommendations(saved: skewed, now: now, wallNow: 100_000)
    #expect(bounded.retryAfter(now: now) == .seconds(21_600))
    #expect(bounded.failures == 10 && bounded.suspended)
    #expect(!String(decoding: try JSONEncoder().encode(replica.document), as: UTF8.self).contains("retry_at"))
}

@Test func `mix slots stay bounded and removal survives stale sync`() throws {
    var local = device("a")
    for index in 0..<2500 { try local.saveMix(title: "Mix \(index)", uris: [uri]) }
    #expect(local.document.records.count == DiscoveryLimits.mixLimit)
    let key = local.document.records.keys.sorted().first!
    let stale = local.document
    try local.edit(key, nil)
    try local.document.merge(stale)
    #expect(local.document.records[key]?.value == nil)
    try local.saveMix(title: "Replacement", uris: [uri])
    #expect(local.document.records.count == DiscoveryLimits.mixLimit)
    #expect(local.document.records[key]?.value == .mix(title: "Replacement", uris: [uri]))
}

@Test func `loopback request lines accept every TCP split and reject malformed callbacks`() throws {
    let bytes = Data("GET /login?code=dummy&state=test-state HTTP/1.1\r\n".utf8)
    for split in 0...bytes.count {
        var request = LoopbackRequest()
        let first = try request.append(bytes.prefix(split))
        if split < bytes.count { #expect(first == nil) }
        let complete = try first ?? request.append(bytes.suffix(bytes.count - split))
        #expect(complete.map { LoopbackRequest.redirect($0, state: "test-state") } == .code("dummy"))
    }
    #expect(LoopbackRequest.redirect("GET /favicon.ico HTTP/1.1", state: "x") == .stray)
    #expect(LoopbackRequest.redirect("GET /login-extra?code=x&state=x HTTP/1.1", state: "x") == .stray)
    #expect(LoopbackRequest.redirect("GET /login?code=x&state=wrong HTTP/1.1", state: "x") == .refused)
    #expect(LoopbackRequest.redirect("GET /login?code=x&state=x&state=x HTTP/1.1", state: "x") == .refused)
    var oversized = LoopbackRequest()
    #expect(throws: (any Error).self) { try oversized.append(Data(repeating: 65, count: 16_385)) }
}
