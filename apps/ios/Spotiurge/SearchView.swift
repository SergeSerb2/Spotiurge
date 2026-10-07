// Search: the system search tab over Spotify's catalogue. A newer query
// cancels the older request, so stale results never replace newer ones.

import SpotiurgeCore
import SwiftUI

struct SearchView: View {
    @Binding var query: String
    @Environment(SpotifyAccount.self) private var account
    @Environment(Player.self) private var player
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(\.demoMode) private var demo
    @State private var results = SearchResults()
    @State private var error: String?
    @State private var searching = false

    var body: some View {
        NavigationStack {
            List {
                if demo { DemoBanner().roomRow(block: true) }
                if !demo && !account.signedIn {
                    SignInPrompt().roomRow(block: true).listRowSeparator(.hidden)
                } else if query.trimmingCharacters(in: .whitespaces).isEmpty {
                    Notice(text: "Search songs, albums and playlists on Spotify.", symbol: "magnifyingglass").roomRow(block: true)
                } else {
                    if let error { Notice(text: error, symbol: "exclamationmark.circle").roomRow(block: true) }
                    if !searching && error == nil && results == SearchResults() {
                        Notice(text: "No results for \u{201C}\(query)\u{201D}.").roomRow(block: true)
                    }
                    if !results.tracks.isEmpty {
                        Section {
                            ForEach(results.tracks) { track in
                                HStack(spacing: 4) {
                                    Button { player.play(track) } label: {
                                        TrackRow(track: track, reason: nil, live: player.nowPlaying?.uri == track.uri)
                                    }
                                    .buttonStyle(.plain)
                                    FeedbackButtons(rating: discovery.rating(track), enabled: !demo) { discovery.rate(track, $0) }
                                }
                                .roomRow()
                            }
                        } header: { SectionTitle(text: "Songs") }
                    }
                    if !results.albums.isEmpty {
                        Section {
                            ForEach(results.albums) { album in
                                contextRow(uri: album.uri, title: album.name, subtitle: "Album · " + album.artists.joined(separator: ", "), image: album.imageURL)
                            }
                        } header: { SectionTitle(text: "Albums") }
                    }
                    if !results.playlists.isEmpty {
                        Section {
                            ForEach(results.playlists) { playlist in
                                contextRow(uri: playlist.uri, title: playlist.name, subtitle: "Playlist · " + playlist.owner, image: playlist.imageURL)
                            }
                        } header: { SectionTitle(text: "Playlists") }
                    }
                }
                if let notice = player.notice { Notice(text: notice, symbol: "speaker.slash").roomRow(block: true) }
            }
            .listStyle(.plain)
            .roomBackground()
            .navigationTitle("Search")
            .searchable(text: $query, placement: .navigationBarDrawer(displayMode: .always), prompt: "Songs, albums, playlists")
            .task(id: query) { await search() }
        }
    }

    private func contextRow(uri: String, title: String, subtitle: String, image: URL?) -> some View {
        Button {
            player.playContext(uri: uri, title: title, subtitle: subtitle, imageURL: image)
        } label: {
            HStack(spacing: 12) {
                Artwork(url: image, seed: uri, size: 48)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(Typeface.rowTitle).foregroundStyle(Palette.text).lineLimit(2)
                    Text(subtitle).font(Typeface.detail).foregroundStyle(Palette.secondary).lineLimit(1)
                }
                Spacer(minLength: 0)
            }
            .frame(minHeight: 56)
            .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .roomRow()
    }

    private func search() async {
        let text = query.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else {
            results = SearchResults()
            error = nil
            return
        }
        if demo {
            let needle = text.lowercased()
            results = SearchResults(
                tracks: Demo.saved.filter { "\($0.name) \($0.artistLine)".lowercased().contains(needle) },
                albums: Demo.albums.filter { "\($0.name) \($0.artists.joined())".lowercased().contains(needle) },
                playlists: Demo.playlists.filter { $0.name.lowercased().contains(needle) })
            return
        }
        guard account.signedIn else { return }
        // Debounce typing; a newer query cancels this task.
        try? await Task.sleep(for: .milliseconds(300))
        guard !Task.isCancelled else { return }
        searching = true
        defer { searching = false }
        do throws(WebError) {
            let found = try await account.web.search(text)
            guard !Task.isCancelled else { return }
            results = found
            error = nil
        } catch {
            guard !Task.isCancelled else { return }
            self.error = error.message
        }
    }
}
