// Synthetic demo data for development and deterministic captures, with the
// same titles and reasons as the desktop demo (src/demo.rs). Demo mode is
// labelled on every screen, keeps nothing, syncs nothing, plays no audio and
// sends nothing to Spotify, the private cloud or the AI.
//
// Start it with the launch argument `-SpotiurgeDemo home|error|onboarding`,
// or from Settings.

import Foundation
import SpotiurgeCore
import SwiftUI

enum DemoScenario: String, CaseIterable {
    case home, error, onboarding

    static var launch: DemoScenario? {
        UserDefaults.standard.string(forKey: "SpotiurgeDemo").flatMap(DemoScenario.init(rawValue:))
    }

    struct Discovery {
        var replica: Replica
        var picks: [Pick]
        var status: String
        var error: RecommendationErrorKind?
        var outcome: CatalogueOutcome
    }

    var discovery: Discovery {
        var replica = Replica(device: String(repeating: "d", count: 32))
        switch self {
        case .home:
            try? replica.edit("taste", .taste(text: "Warm, spacious electronics and soulful grooves, with a few surprises."))
            try? replica.edit("mix:demo", .mix(title: "Demo discovery mix", uris: Demo.picks(8, 0, 0).compactMap(\.track?.uri)))
            try? replica.recordHistory(prompt: replica.document.taste, suggestions: Demo.picks(12, 0, 0).map(\.suggestion))
            return Discovery(replica: replica, picks: Demo.picks(10, 2, 0), status: "", error: nil, outcome: .complete)
        case .error:
            try? replica.edit("taste", .taste(text: "Warm, spacious electronics and soulful grooves."))
            return Discovery(replica: replica, picks: Demo.picks(5, 0, 7),
                             status: "The AI is rate limited. Your picks are kept; try again later.",
                             error: .rateLimited, outcome: .rateLimited)
        case .onboarding:
            return Discovery(replica: replica, picks: [], status: "", error: nil, outcome: .complete)
        }
    }
}

enum Demo {
    static let artists = ["Bonobo", "Khruangbin", "Nils Frahm", "Little Simz", "Floating Points", "Jon Hopkins", "Sault", "Four Tet"]
    static let titles = ["Rosewood", "Otomo", "Shadows", "Tides", "Elysian", "Closer", "Counterpart", "Sapien", "From You", "Day by Day",
                         "Age of Phase", "Polyghost", "Time Moves Slow", "August 10", "So Rare", "Fugue", "Encores", "Sunlight",
                         "My Friend the Forest", "Kaleidoscope"]
    static let reasons = [
        "Demo reason: a patient groove with room to breathe.",
        "Demo reason: warm textures close to your saved taste.",
        "Demo reason: one step further out, still spacious.",
        "Demo reason: a late-night pulse with a soft edge.",
    ]

    static func track(_ index: Int) -> Track {
        Track(uri: "spotify:track:" + String(format: "%022d", index), name: titles[index % titles.count],
              artists: [artists[index % artists.count]], durationMs: (180 + index * 37 % 240) * 1000, album: "Demo album")
    }

    static func picks(_ ready: Int, _ missing: Int, _ unchecked: Int) -> [Pick] {
        (0..<(ready + missing + unchecked)).map { index in
            let track = track(3 + index)
            return Pick(suggestion: Suggestion(title: track.name, artist: track.artistLine, reason: reasons[index % reasons.count]),
                        track: index < ready ? track : nil, checked: index < ready + missing)
        }
    }

    static var saved: [Track] { (0..<14).map(track) }

    static let playlists = ["Late night focus", "Sunday morning", "Running 2026", "Berlin nights", "Slow mornings"].enumerated().map {
        Playlist(uri: "spotify:playlist:demo\($0.offset)", name: $0.element, owner: "Demo", trackCount: 20 + $0.offset * 7)
    }

    static let albums = ["Fragments", "Mordechai", "All Melody", "Sometimes I Might Be Introvert", "Promises", "Immunity"].enumerated().map {
        Album(uri: "spotify:album:demo\($0.offset)", name: $0.element, artists: [artists[$0.offset]])
    }

    /// The first pick, paused at 1:23, so the live row shows its lamp.
    static var nowPlaying: NowPlaying {
        let first = track(3)
        return NowPlaying(uri: first.uri, title: first.name, artist: first.artistLine, durationMs: first.durationMs,
                          light: tile(first.uri))
    }

    /// Two deterministic colours per demo item: its generated cover.
    static func tile(_ seed: String) -> [UInt32] {
        var hash: UInt32 = 2_166_136_261
        for byte in seed.utf8 { hash = (hash ^ UInt32(byte)) &* 16_777_619 }
        let hue = (Double(hash) * 0.618_033_988_7).truncatingRemainder(dividingBy: 1)
        return [hue, (hue + 0.09).truncatingRemainder(dividingBy: 1)].map { h in
            let color = UIColor(hue: h, saturation: 0.55, brightness: 0.78, alpha: 1)
            var r: CGFloat = 0, g: CGFloat = 0, b: CGFloat = 0
            color.getRed(&r, green: &g, blue: &b, alpha: nil)
            return UInt32(r * 255) << 16 | UInt32(g * 255) << 8 | UInt32(b * 255)
        }
    }
}

/// The banner every demo screen carries.
struct DemoBanner: View {
    var body: some View {
        Label("Demo data. Synthetic, not your library; nothing plays, syncs or reaches Spotify or the AI.", systemImage: "theatermasks")
            .font(Typeface.caption)
            .foregroundStyle(Palette.text)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Palette.surface, in: .rect(cornerRadius: Radius.popover))
            .overlay(RoundedRectangle(cornerRadius: Radius.popover).strokeBorder(Palette.outline))
            .accessibilityElement(children: .combine)
    }
}
