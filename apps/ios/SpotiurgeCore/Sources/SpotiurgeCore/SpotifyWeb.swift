// The few Spotify Web API reads the iPhone interface needs: profile,
// saved tracks, playlists, and catalogue search (also used for discovery
// matching). The access token comes from the caller and never appears in
// a URL or log.

import Foundation

public struct Album: Codable, Hashable, Sendable, Identifiable {
    public var uri: String
    public var name: String
    public var artists: [String]
    public var imageURL: URL?
    public var id: String { uri }

    public init(uri: String, name: String, artists: [String], imageURL: URL? = nil) {
        self.uri = uri
        self.name = name
        self.artists = artists
        self.imageURL = imageURL
    }
}

public struct Playlist: Codable, Hashable, Sendable, Identifiable {
    public var uri: String
    public var name: String
    public var owner: String
    public var trackCount: Int?
    public var imageURL: URL?
    public var id: String { uri }

    public init(uri: String, name: String, owner: String, trackCount: Int? = nil, imageURL: URL? = nil) {
        self.uri = uri
        self.name = name
        self.owner = owner
        self.trackCount = trackCount
        self.imageURL = imageURL
    }
}

public struct SearchResults: Sendable, Equatable {
    public var tracks: [Track] = []
    public var albums: [Album] = []
    public var playlists: [Playlist] = []
    public init(tracks: [Track] = [], albums: [Album] = [], playlists: [Playlist] = []) {
        self.tracks = tracks
        self.albums = albums
        self.playlists = playlists
    }
}

public enum WebError: Error, Equatable, Sendable {
    case signInNeeded
    case rateLimited(retryAfter: Int)
    case unavailable

    public var message: String {
        switch self {
        case .signInNeeded: "Sign in to Spotify in Settings to see your library and search."
        case .rateLimited: "Spotify asked Spotiurge to slow down. Try again in a moment."
        case .unavailable: "Spotify is unavailable right now."
        }
    }

    public var catalogue: CatalogueError {
        switch self {
        case .signInNeeded: .signInNeeded
        case .rateLimited: .rateLimited
        case .unavailable: .unavailable
        }
    }
}

public struct SpotifyWeb: Sendable {
    public typealias TokenProvider = @Sendable () async throws -> String
    let token: TokenProvider
    let session: URLSession
    static let base = URL(string: "https://api.spotify.com/v1/")!

    public init(session: URLSession = .shared, token: @escaping TokenProvider) {
        self.session = session
        self.token = token
    }

    func get<T: Decodable>(_ path: String, _ query: [URLQueryItem] = [], as: T.Type) async throws(WebError) -> T {
        // The provider decides whether a sign-in is needed. Anything else it
        // throws (offline, a locked Keychain, a server error) keeps the grant.
        let secret: String
        do { secret = try await token() } catch let error as WebError { throw error } catch { throw .unavailable }
        var components = URLComponents(url: Self.base.appending(path: path), resolvingAgainstBaseURL: false)!
        components.queryItems = query.isEmpty ? nil : query
        var request = URLRequest(url: components.url!, timeoutInterval: 15)
        request.setValue("Bearer \(secret)", forHTTPHeaderField: "Authorization")
        let data: Data, response: URLResponse
        do { (data, response) = try await session.data(for: request) } catch { throw .unavailable }
        let http = response as? HTTPURLResponse
        switch http?.statusCode ?? 0 {
        case 200...299:
            guard let value = try? JSONDecoder().decode(T.self, from: data) else { throw .unavailable }
            return value
        case 401: throw .signInNeeded
        case 429: throw .rateLimited(retryAfter: Int(http?.value(forHTTPHeaderField: "Retry-After") ?? "") ?? 30)
        default: throw .unavailable
        }
    }

    public func displayName() async throws(WebError) -> String {
        struct Me: Decodable { let display_name: String?; let id: String }
        let me = try await get("me", as: Me.self)
        return me.display_name ?? me.id
    }

    public func savedTracks(offset: Int = 0) async throws(WebError) -> (tracks: [Track], total: Int) {
        struct Page: Decodable { let items: [Item]; let total: Int }
        struct Item: Decodable { let track: WireTrack? }
        let page = try await get("me/tracks", [.init(name: "limit", value: "50"), .init(name: "offset", value: String(offset)), .init(name: "market", value: "from_token")], as: Page.self)
        return (page.items.compactMap { $0.track?.track }, page.total)
    }

    public func playlists() async throws(WebError) -> [Playlist] {
        struct Page: Decodable { let items: [WirePlaylist?] }
        return try await get("me/playlists", [.init(name: "limit", value: "50")], as: Page.self).items.compactMap { $0?.playlist }
    }

    public func search(_ text: String, types: String = "track,album,playlist", limit: Int = 10) async throws(WebError) -> SearchResults {
        struct Page<Item: Decodable>: Decodable { let items: [Item?] }
        struct Answer: Decodable {
            let tracks: Page<WireTrack>?
            let albums: Page<WireAlbum>?
            let playlists: Page<WirePlaylist>?
        }
        let answer = try await get("search", [
            .init(name: "q", value: text), .init(name: "type", value: types),
            .init(name: "limit", value: String(limit)), .init(name: "market", value: "from_token"),
        ], as: Answer.self)
        return SearchResults(
            tracks: answer.tracks?.items.compactMap { $0?.track } ?? [],
            albums: answer.albums?.items.compactMap { $0?.album } ?? [],
            playlists: answer.playlists?.items.compactMap { $0?.playlist } ?? [])
    }

    /// Track candidates for one discovery suggestion.
    public func catalogueSearch(_ suggestion: Suggestion) async throws(CatalogueError) -> [Track] {
        do { return try await search(discoverySearchTerm(suggestion), types: "track").tracks } catch { throw error.catalogue }
    }
}

struct WireImage: Decodable { let url: URL }
struct WireArtist: Decodable { let name: String }

struct WireAlbumRef: Decodable {
    let name: String?
    let images: [WireImage]?
}

struct WireTrack: Decodable {
    let uri: String?
    let name: String?
    let artists: [WireArtist]?
    let duration_ms: Int?
    let album: WireAlbumRef?
    let is_playable: Bool?

    var track: Track? {
        guard let uri, let name else { return nil }
        return Track(uri: uri, name: name, artists: artists?.map(\.name) ?? [], durationMs: duration_ms ?? 0,
                     album: album?.name, imageURL: album?.images?.first?.url, isPlayable: is_playable)
    }
}

struct WireAlbum: Decodable {
    let uri: String?
    let name: String?
    let artists: [WireArtist]?
    let images: [WireImage]?
    var album: Album? {
        guard let uri, let name else { return nil }
        return Album(uri: uri, name: name, artists: artists?.map(\.name) ?? [], imageURL: images?.first?.url)
    }
}

struct WirePlaylist: Decodable {
    struct Owner: Decodable { let display_name: String? }
    struct Count: Decodable { let total: Int? }
    let uri: String?
    let name: String?
    let owner: Owner?
    let tracks: Count?
    let images: [WireImage]?
    var playlist: Playlist? {
        guard let uri, let name else { return nil }
        return Playlist(uri: uri, name: name, owner: owner?.display_name ?? "", trackCount: tracks?.total, imageURL: images?.first?.url)
    }
}
