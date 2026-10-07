// The floating console (tab-bar accessory) and the Now Playing sheet.
// Play/pause is the amber lamp while playing; the transport answers at
// once and engine events confirm it.

import SpotiurgeCore
import SwiftUI

struct MiniPlayer: View {
    @Environment(Player.self) private var player
    @Environment(\.tabViewBottomAccessoryPlacement) private var placement
    var open: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            Button(action: open) {
                HStack(spacing: 10) {
                    Artwork(url: player.nowPlaying?.imageURL, seed: player.nowPlaying?.uri ?? "", size: 32, radius: 6)
                    VStack(alignment: .leading, spacing: 0) {
                        Text(player.nowPlaying?.title ?? "Nothing playing")
                            .font(Typeface.inter(14, .semibold)).foregroundStyle(Palette.text).lineLimit(1)
                        Text(player.nowPlaying?.artist ?? (player.unavailableReason == nil ? "Pick something to play" : "Playback unavailable here"))
                            .font(Typeface.inter(12)).foregroundStyle(Palette.secondary).lineLimit(1)
                    }
                    Spacer(minLength: 0)
                }
                .contentShape(.rect)
            }
            .buttonStyle(.plain)
            .accessibilityLabel(player.nowPlaying.map { "Now playing, \($0.title) by \($0.artist)" } ?? "Nothing playing")
            .accessibilityHint("Opens Now Playing")
            // The console's primary play key: a 36-point lamp disc in a
            // 44-point target.
            Button { player.toggle() } label: {
                Image(systemName: player.playing ? "pause.fill" : "play.fill")
            }
            .buttonStyle(LampButtonStyle(size: 36))
            .frame(width: 44, height: 44)
            .disabled(player.nowPlaying == nil)
            .accessibilityLabel(player.playing ? "Pause" : "Play")
            .sensoryFeedback(.impact(weight: .light), trigger: player.playing)
            if placement != .inline {
                Button { player.next() } label: {
                    Image(systemName: "forward.fill").font(.system(size: 15, weight: .semibold)).foregroundStyle(Palette.text)
                        .frame(width: 44, height: 44).contentShape(.rect)
                }
                .buttonStyle(.plain)
                .disabled(player.nowPlaying == nil)
                .accessibilityLabel("Next")
            }
        }
        .padding(.leading, 12)
        .padding(.trailing, 4)
    }
}

struct NowPlayingSheet: View {
    @Environment(Player.self) private var player
    @Environment(DiscoveryModel.self) private var discovery
    @Environment(\.demoMode) private var demo
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        let now = player.nowPlaying
        ScrollView {
            VStack(spacing: 20) {
                Capsule().fill(Palette.dim.opacity(0.5)).frame(width: 36, height: 5).padding(.top, 8).accessibilityHidden(true)
                if demo { DemoBanner() }
                Artwork(url: now?.imageURL, seed: now?.uri ?? "", size: 300, radius: Radius.console)
                    .frame(maxWidth: .infinity)
                    .shadow(color: .black.opacity(0.35), radius: 24, y: 10)
                    .scaledToFit()
                HStack(alignment: .top) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(now?.title ?? "Nothing playing").font(Typeface.title).foregroundStyle(Palette.text).lineLimit(2)
                        Text(now?.artist ?? "").font(Typeface.body).foregroundStyle(Palette.secondary).lineLimit(1)
                    }
                    Spacer()
                    if let now, isSpotifyTrack(now.uri) {
                        let track = Track(uri: now.uri, name: now.title, artists: [now.artist], durationMs: now.durationMs)
                        FeedbackButtons(rating: discovery.rating(track), enabled: !demo) { discovery.rate(track, $0) }
                    }
                }
                scrubber
                transport
                if let notice = player.notice ?? (demo ? nil : player.unavailableReason) {
                    Notice(text: notice, symbol: "speaker.slash")
                }
                Spacer(minLength: 0)
                Label(demo ? "Demo: no audio" : "This iPhone", systemImage: "iphone")
                    .font(Typeface.caption).foregroundStyle(Palette.secondary)
                    .padding(.bottom, 8)
            }
            .padding(.horizontal, 24)
        }
        .scrollBounceBehavior(.basedOnSize)
        .background(Room(light: (now?.light ?? []).map { Color(hex: $0) }))
        .presentationBackground(Palette.room)
        .presentationDragIndicator(.hidden)
    }

    private var scrubber: some View {
        TimelineView(.periodic(from: .now, by: 1)) { context in
            let total = player.nowPlaying?.durationMs ?? 0
            let at = player.positionMs(at: context.date)
            VStack(spacing: 6) {
                // Read-only: the development engine has no seek yet.
                ProgressView(value: Double(at), total: Double(max(total, 1)))
                    .tint(player.playing ? Palette.lamp : Palette.secondary)
                HStack {
                    Text(duration(at))
                    Spacer()
                    Text(total > 0 ? duration(total) : "")
                }
                .font(Typeface.caption).monospacedDigit().foregroundStyle(Palette.secondary)
            }
            .accessibilityElement(children: .ignore)
            .accessibilityLabel("Position")
            .accessibilityValue(total > 0 ? "\(duration(at)) of \(duration(total))" : duration(at))
        }
    }

    private var transport: some View {
        HStack(spacing: 36) {
            Button { player.previous() } label: { Image(systemName: "backward.fill").frame(width: 56, height: 56) }
                .accessibilityLabel("Previous")
            Button { player.toggle() } label: { Image(systemName: player.playing ? "pause.fill" : "play.fill") }
                .buttonStyle(LampButtonStyle(size: 72))
                .accessibilityLabel(player.playing ? "Pause" : "Play")
                .sensoryFeedback(.impact(weight: .light), trigger: player.playing)
            Button { player.next() } label: { Image(systemName: "forward.fill").frame(width: 56, height: 56) }
                .accessibilityLabel("Next")
        }
        .font(.system(size: 26, weight: .semibold))
        .foregroundStyle(Palette.text)
        .buttonStyle(.plain)
        .disabled(player.nowPlaying == nil)
    }
}
