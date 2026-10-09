// Library: liked songs, playlists and Spotiurge's saved mixes, from the
// Spotify Web API grant (or demo data, clearly labelled).

import SpotiurgeCore
import SwiftUI

struct LibraryView: View {
    @Environment(SpotifyAccount.self) private var account
    @Environment(Player.self) private var player
    @Environment(\.demoMode) private var demo

    private var saved: [Track] { demo ? Demo.saved : account.savedTracks }
    private var playlists: [Playlist] { demo ? Demo.playlists : account.playlists }

    var body: some View {
        NavigationStack {
            List {
                if demo { DemoBanner().roomRow(block: true) }
                if !demo && !account.signedIn {
                    SignInPrompt().roomRow(block: true).listRowSeparator(.hidden)
                } else {
                    if let error = account.libraryError, !demo {
                        Notice(text: error, symbol: "exclamationmark.circle").roomRow(block: true)
                    }
                    NavigationLink {
                        TrackList(title: "Liked Songs", tracks: saved)
                    } label: {
                        HStack(spacing: 12) {
                            Image(systemName: "heart.fill").font(.system(size: 18, weight: .semibold)).foregroundStyle(Palette.text)
                                .frame(width: 48, height: 48).background(Palette.surfaceActive, in: .rect(cornerRadius: 6))
                            VStack(alignment: .leading, spacing: 2) {
                                Text("Liked Songs").font(Typeface.rowTitle).foregroundStyle(Palette.text)
                                Text(demo ? "\(saved.count) songs" : "\(account.savedTotal) songs").font(Typeface.detail).foregroundStyle(Palette.secondary)
                            }
                        }
                        .frame(minHeight: 56)
                    }
                    .roomRow()
                    Section {
                        ForEach(playlists) { playlist in
                            Button {
                                player.playContext(uri: playlist.uri, title: playlist.name, subtitle: playlist.owner, imageURL: playlist.imageURL)
                            } label: {
                                HStack(spacing: 12) {
                                    Artwork(url: playlist.imageURL, seed: playlist.uri, size: 48)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(playlist.name).font(Typeface.rowTitle).foregroundStyle(Palette.text).lineLimit(2)
                                        Text(["Playlist", playlist.owner, playlist.trackCount.map { "\($0) songs" }].compactMap { $0?.isEmpty == false ? $0 : nil }.joined(separator: " · "))
                                            .font(Typeface.detail).foregroundStyle(Palette.secondary).lineLimit(1)
                                    }
                                    Spacer(minLength: 0)
                                }
                                .frame(minHeight: 56)
                                .contentShape(.rect)
                            }
                            .buttonStyle(.plain)
                            .accessibilityHint("Plays this playlist on this iPhone")
                            .roomRow()
                        }
                    } header: {
                        SectionTitle(text: "Playlists").padding(.top, 8)
                    }
                }
                if let notice = player.notice { Notice(text: notice, symbol: "speaker.slash").roomRow(block: true) }
            }
            .listStyle(.plain)
            .roomBackground()
            .refreshable { await account.loadLibrary() }
            .navigationTitle("Library")
        }
    }
}

struct TrackList: View {
    var title: String
    var tracks: [Track]
    @Environment(Player.self) private var player
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(\.demoMode) private var demo

    var body: some View {
        List(tracks) { track in
            HStack(spacing: 4) {
                Button { player.play(track, from: tracks) } label: {
                    TrackRow(track: track, reason: nil, live: player.nowPlaying?.uri == track.uri)
                }
                .buttonStyle(.plain)
                FeedbackButtons(rating: discovery.rating(track), enabled: !demo) { discovery.rate(track, $0) }
            }
            .roomRow()
        }
        .listStyle(.plain)
        .roomBackground()
        .navigationTitle(title)
    }
}

/// Shown where Spotify data is needed and the Web API grant is missing.
struct SignInPrompt: View {
    @Environment(SpotifyAccount.self) private var account

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Sign in to Spotify").font(Typeface.title).foregroundStyle(Palette.text)
            Text("Spotiurge reads your liked songs and playlists, searches the catalogue and checks AI picks on Spotify. Spotify's system sign-in sheet opens; no browser is embedded.")
                .font(Typeface.body).foregroundStyle(Palette.secondary)
            if case .failed(let message) = account.state {
                Notice(text: message, symbol: "exclamationmark.circle")
            }
            Button(account.state == .signingIn ? "Waiting for Spotify…" : "Sign in to Spotify") { account.signIn() }
                .font(Typeface.inter(15, .semibold))
                .buttonStyle(ReadableButtonStyle(glass: true))
                .disabled(account.state == .signingIn)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .paneGlass()
        .padding(.vertical, 8)
    }
}
