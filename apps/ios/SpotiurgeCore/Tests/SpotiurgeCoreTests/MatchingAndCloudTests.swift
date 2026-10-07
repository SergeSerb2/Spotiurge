// Ports of the src/backend.rs matching tables and the src/discovery_cloud.rs
// endpoint and refusal tests, plus a stubbed sync conflict round.

import Foundation
import Testing
@testable import SpotiurgeCore

func suggestion(_ title: String, _ artist: String) -> Suggestion {
    Suggestion(title: title, artist: artist, reason: "A discovery trial.")
}

func track(_ name: String, _ artists: [String], _ id: Character = "a") -> Track {
    Track(uri: "spotify:track:" + String(repeating: id, count: 22), name: name, artists: artists, durationMs: 1000)
}

@Test(arguments: [
    ("adore u", "Fred again.. and Obongjayar", "adore u", ["Fred again..", "Obongjayar"]),
    ("Imagination", "Gorgon City feat. Katy Menditta", "Imagination", ["Gorgon City", "Katy Menditta"]),
    ("Where You Are", "John Summit & HAYLA", "Where You Are", ["John Summit", "HAYLA"]),
    ("Butterflies", "Skrillex, Starrah & Four Tet", "Butterflies", ["Skrillex", "Starrah", "Four Tet"]),
    ("Sun In Your Eyes", "Above & Beyond", "Sun In Your Eyes", ["Above & Beyond"]),
    ("The Boxer", "Simon & Garfunkel", "The Boxer", ["Simon & Garfunkel"]),
    ("Dog Days", "Florence and the Machine", "Dog Days", ["Florence and the Machine"]),
    ("Imagination", "Gorgon City", "Imagination (feat. Katy Menditta)", ["Gorgon City", "Katy Menditta"]),
    ("Imagination feat. Katy Menditta", "GORGON  CITY", "Imagination", ["Gorgon City", "Katy Menditta"]),
    ("Don\u{2019}t Stop", "Prospa", "Don't Stop", ["Prospa"]),
])
func `matches collaborations by their credited artists`(title: String, artist: String, name: String, artists: [String]) {
    #expect(discoveryMatches(suggestion(title, artist), track(name, artists)))
}

@Test(arguments: [
    ("Imagination", "Gorgon City feat. Katy Menditta", "Imagination", ["Gorgon City"]),
    ("Imagination", "Katy Menditta", "Imagination", ["Gorgon City", "Katy Menditta"]),
    ("Imagination", "Gorgon City feat. Katy Mendita", "Imagination", ["Gorgon City", "Katy Menditta"]),
    ("Imagination", "Gorgon City (Official)", "Imagination", ["Gorgon City"]),
    ("Imagination", "Gorgon", "Imagination", ["Gorgon City"]),
    ("adore u", "Fred again", "adore u", ["Fred again.."]),
    ("Imagination", "Gorgon City", "Imagination - Extended Mix", ["Gorgon City"]),
    ("Imagination - Radio Edit", "Gorgon City", "Imagination", ["Gorgon City"]),
    ("Imagination", "Gorgon City", "Imagination (Live)", ["Gorgon City"]),
    ("Imagination", "Gorgon City", "Imagination (Prospa Remix)", ["Gorgon City", "Prospa"]),
    ("Imagination", "Gorgon City", "Imagination (feat. Someone Else)", ["Gorgon City"]),
    ("Imagine", "Gorgon City", "Imagination", ["Gorgon City"]),
    ("Song", "Artist & Artist", "Song", ["Artist"]),
])
func `rejects credits and versions Spotify does not have`(title: String, artist: String, name: String, artists: [String]) {
    #expect(!discoveryMatches(suggestion(title, artist), track(name, artists)))
}

@Test func `unplayable or invented tracks never match`() {
    var unavailable = track("Imagination", ["Gorgon City"])
    unavailable.isPlayable = false
    #expect(!discoveryMatches(suggestion("Imagination", "Gorgon City"), unavailable))
    var invented = track("Imagination", ["Gorgon City"])
    invented.uri = "spotify:track:invented"
    #expect(!discoveryMatches(suggestion("Imagination", "Gorgon City"), invented))
}

@Test func `searches free text with only the primary artist`() {
    #expect(discoverySearchTerm(suggestion("Imagination", "Gorgon City feat. Katy Menditta")) == "Imagination gorgon city")
    #expect(discoverySearchTerm(suggestion("\"Butterflies\"", "Skrillex, Starrah & Four Tet")) == "Butterflies skrillex")
    #expect(discoverySearchTerm(suggestion("Sun", "Above & Beyond")) == "Sun above & beyond")
}

@Test func `retry keeps stored matches and timeouts keep suggestions`() async {
    var cached = Pick(suggestion: suggestion("Cached", "Artist"))
    cached.track = track("Cached", ["Artist"])
    let kept = await resolveDiscovery([cached], budget: .seconds(1)) { _ throws(CatalogueError) in
        Issue.record("a stored match must not be searched again")
        return []
    }
    #expect(kept.picks[0].track == cached.track && kept.picks[0].checked && kept.outcome == .complete)

    let slow = await resolveDiscovery([cached, Pick(suggestion: suggestion("Trial", "Artist"))], budget: .milliseconds(20)) { _ throws(CatalogueError) in
        try? await Task.sleep(for: .seconds(30))
        return []
    }
    #expect(slow.outcome == .timedOut)
    #expect(slow.picks.count == 2 && slow.picks[1].track == nil)

    let limited = await resolveDiscovery([Pick(suggestion: suggestion("A", "B")), Pick(suggestion: suggestion("C", "D"))], budget: .seconds(5)) { _ throws(CatalogueError) in
        throw .rateLimited
    }
    #expect(limited.outcome == .rateLimited && !limited.picks[0].checked)
}

@Test func `matches are unique across picks`() async {
    let picks = [Pick(suggestion: suggestion("Song", "Artist")), Pick(suggestion: suggestion("Song", "Artist"))]
    let result = await resolveDiscovery(picks, budget: .seconds(5)) { _ throws(CatalogueError) in [track("Song", ["Artist"])] }
    #expect(result.picks[0].track != nil && result.picks[1].track == nil && result.picks[1].checked)
}

// MARK: - Private cloud

@Test(arguments: ["http://example.com", "https://user:secret@example.com", "https://example.com/api", "https://example.com?token=secret", "https://example.com/#secret"])
func `authenticated endpoints reject plaintext credentials paths and queries`(bad: String) {
    #expect(throws: DiscoveryError.self) { try PrivateCloud.origin(bad) }
}

@Test func `origins are normalized`() throws {
    #expect(try PrivateCloud.origin("https://Example.com/").absoluteString == "https://example.com")
    #expect(try PrivateCloud.origin(PrivateCloud.defaultEndpoint).absoluteString == PrivateCloud.defaultEndpoint)
}

@Test func `server busy and model rate limits are distinct`() {
    #expect(RecommendationFailure.refused(status: 429, code: "busy").kind == .busy)
    for code in ["rate_limited", "unknown", nil] {
        #expect(RecommendationFailure.refused(status: 429, code: code).kind == .rateLimited)
    }
    #expect(RecommendationFailure.refused(status: 401, code: "busy").kind == .pairing)
    #expect(RecommendationFailure.refused(status: 503, code: "busy").kind == .unavailable)
}

/// A scripted private cloud: answers each request in order and records it.
final class StubCloud: URLProtocol, @unchecked Sendable {
    nonisolated(unsafe) static var replies: [(Int, Data)] = []
    nonisolated(unsafe) static var seen: [URLRequest] = []

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        var request = request
        if let stream = request.httpBodyStream {
            stream.open()
            var body = Data()
            var buffer = [UInt8](repeating: 0, count: 4096)
            while stream.hasBytesAvailable, case let count = stream.read(&buffer, maxLength: 4096), count > 0 { body.append(buffer, count: count) }
            request.httpBody = body
        }
        Self.seen.append(request)
        let (status, data) = Self.replies.removeFirst()
        client?.urlProtocol(self, didReceive: HTTPURLResponse(url: request.url!, statusCode: status, httpVersion: nil, headerFields: nil)!, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: data)
        client?.urlProtocolDidFinishLoading(self)
    }
    override func stopLoading() {}
}

@Suite(.serialized) struct CloudRounds {
    let secret = String(repeating: "s", count: 40)

    func client() throws -> CloudClient {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubCloud.self]
        let secret = secret
        return try CloudClient(endpoint: "https://cloud.example", session: URLSession(configuration: configuration)) { origin in
            #expect(origin == "https://cloud.example")
            return secret
        }
    }

    @Test func `sync refetches and merges after a concurrent write`() async throws {
        var remote = device("b")
        try remote.edit("taste", taste("remote"))
        let first = #"{"revision":4,"document":{"version":1,"records":{}}}"#
        let second = #"{"revision":5,"document":"# + String(decoding: try remote.document.encoded(), as: UTF8.self) + "}"
        StubCloud.seen = []
        StubCloud.replies = [(200, Data(first.utf8)), (409, Data()), (200, Data(second.utf8)), (200, Data(#"{"revision":6}"#.utf8))]
        var local = device("a")
        try local.edit("mix:m", .mix(title: "Mine", uris: [uri]))
        let merged = try await client().sync(local)
        #expect(merged.taste == "remote")
        #expect(merged.records["mix:m"] != nil)
        #expect(StubCloud.seen.map { $0.value(forHTTPHeaderField: "If-Match") } == [nil, "4", nil, "5"])
        #expect(StubCloud.seen.allSatisfy { !$0.url!.absoluteString.contains(secret) && $0.value(forHTTPHeaderField: "Authorization") == "Bearer \(secret)" })
    }

    @Test func `sync uploads a pending rating above the remote retention clock`() async throws {
        var local = device("a")
        try local.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
        var remote = Document()
        remote.records[DiscoveryLimits.feedbackFloor] = Record(stamp: Stamp(counter: 100, device: String(repeating: "b", count: 32)), value: nil)
        let snapshot = #"{"revision":4,"document":"# + String(decoding: try remote.encoded(), as: UTF8.self) + "}"
        StubCloud.seen = []
        StubCloud.replies = [(200, Data(snapshot.utf8)), (200, Data(#"{"revision":5}"#.utf8))]
        let uploaded = try await client().sync(local)
        #expect(uploaded.records["feedback:\(uri)"]?.stamp.counter == 101)
        #expect(uploaded.rating(uri) == .love)
        #expect(StubCloud.seen.last?.httpMethod == "PUT")
    }

    @Test func `recommendations send taste and ratings only and map refusals`() async throws {
        var local = device("a")
        try local.edit("taste", taste("Warm jazz"))
        try local.edit("feedback:\(uri)", .feedback(uri: uri, title: "Song", artist: "Artist", rating: .love))
        StubCloud.seen = []
        StubCloud.replies = [(429, Data(#"{"error":"x","code":"busy"}"#.utf8)), (200, Data(#"{"suggestions":[{"title":"T","artist":"A","reason":"R"}],"model":"gpt-6-luna"}"#.utf8))]
        await #expect(throws: RecommendationFailure.refused(status: 429, code: "busy")) { try await client().recommend(local.document, exploration: .adventurous) }
        let suggestions = try await client().recommend(local.document, exploration: .adventurous)
        #expect(suggestions == [Suggestion(title: "T", artist: "A", reason: "R")])
        let text = String(decoding: try #require(StubCloud.seen.last?.httpBody), as: UTF8.self)
        #expect(text.contains("\"exploration\":\"adventurous\"") && text.contains("Warm jazz") && !text.contains("spotify:"))
    }
}

/// Fails every request and counts it, so a test can prove nothing was sent.
final class NoNetwork: URLProtocol, @unchecked Sendable {
    nonisolated(unsafe) static var requests = 0

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }
    override func startLoading() {
        Self.requests += 1
        client?.urlProtocol(self, didFailWithError: URLError(.notConnectedToInternet))
    }
    override func stopLoading() {}
}

@Suite(.serialized) struct WebTokenFailures {
    func web(_ token: @escaping SpotifyWeb.TokenProvider) -> SpotifyWeb {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [NoNetwork.self]
        return SpotifyWeb(session: URLSession(configuration: configuration), token: token)
    }

    @Test func `an unavailable token provider keeps the sign-in`() async {
        NoNetwork.requests = 0
        await #expect(throws: WebError.unavailable) { try await web { throw WebError.unavailable }.displayName() }
        #expect(NoNetwork.requests == 0)
    }

    @Test func `an offline refresh is unavailable, not a sign-out`() async {
        NoNetwork.requests = 0
        await #expect(throws: WebError.unavailable) { try await web { throw URLError(.notConnectedToInternet) }.displayName() }
        await #expect(throws: WebError.unavailable) { try await web { throw CancellationError() }.savedTracks() }
        #expect(NoNetwork.requests == 0)
    }

    @Test func `only the provider asks for a new sign-in`() async {
        NoNetwork.requests = 0
        await #expect(throws: WebError.signInNeeded) { try await web { throw WebError.signInNeeded }.playlists() }
        await #expect(throws: CatalogueError.unavailable) { try await web { throw URLError(.timedOut) }.catalogueSearch(suggestion("T", "A")) }
        #expect(NoNetwork.requests == 0)
    }
}
