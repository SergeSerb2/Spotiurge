// Shared pieces: artwork, track rows, feedback, the exploration fader.

import SpotiurgeCore
import SwiftUI

func duration(_ ms: Int) -> String {
    let seconds = max(ms, 0) / 1000
    return "\(seconds / 60):" + String(format: "%02d", seconds % 60)
}

struct DemoModeKey: EnvironmentKey { static let defaultValue = false }

extension EnvironmentValues {
    var demoMode: Bool {
        get { self[DemoModeKey.self] }
        set { self[DemoModeKey.self] = newValue }
    }
}

// MARK: - Artwork

/// A cover. Demo items get a generated tile; real items without art get a
/// quiet glyph rather than an invented picture.
struct Artwork: View {
    var url: URL?
    var seed: String
    var size: CGFloat
    var radius: CGFloat = 6
    @Environment(\.demoMode) private var demo

    var body: some View {
        Group {
            if let url {
                AsyncImage(url: url, transaction: Transaction(animation: nil)) { phase in
                    if let image = phase.image { image.resizable().scaledToFill() } else { placeholder }
                }
            } else if demo {
                let colors = Demo.tile(seed).map { Color(hex: $0) }
                LinearGradient(colors: colors, startPoint: .topLeading, endPoint: .bottomTrailing)
                    .overlay(alignment: .bottomTrailing) {
                        Circle().fill(.white.opacity(0.18)).frame(width: size * 0.7).offset(x: size * 0.18, y: size * 0.2)
                    }
            } else {
                placeholder
            }
        }
        .frame(width: size, height: size)
        .clipShape(.rect(cornerRadius: radius))
        .accessibilityHidden(true)
    }

    private var placeholder: some View {
        Palette.surface.overlay(Image(systemName: "music.note").font(.system(size: size * 0.35)).foregroundStyle(Palette.dim))
    }
}

// MARK: - Exploration fader

/// Familiar / Balanced / Adventurous. Choices select with the desktop's
/// neutral inverted pill, which glides between channels (the fader glide);
/// moss stays reserved for live state.
struct ExplorationPicker: View {
    var selection: Exploration
    var onSelect: (Exploration) -> Void
    @Namespace private var pill
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        // At accessibility text sizes the channels stack rather than truncate.
        ViewThatFits(in: .horizontal) {
            HStack(spacing: 4) { channels }.padding(4).glassEffect(.regular, in: .capsule)
            VStack(spacing: 4) { channels }.padding(4).glassEffect(.regular, in: .rect(cornerRadius: 26))
        }
        .frame(minHeight: 44)
        .sensoryFeedback(.selection, trigger: selection)
        .accessibilityElement(children: .contain)
        .accessibilityLabel("Exploration")
    }

    private var channels: some View {
        ForEach(Exploration.allCases, id: \.self) { option in
                let selected = option == selection
                Button {
                    withAnimation(Motion.curve(Motion.state, reduced: reduceMotion)) { onSelect(option) }
                } label: {
                    Text(option.label)
                        .font(Typeface.inter(14, selected ? .semibold : .medium, relativeTo: .subheadline))
                        .foregroundStyle(selected ? Palette.room : Palette.text)
                        .lineLimit(1)
                        .fixedSize()
                        .padding(.horizontal, 12)
                        .frame(maxWidth: .infinity, minHeight: 36)
                        .background {
                            if selected {
                                Capsule().fill(Palette.text).matchedGeometryEffect(id: "pill", in: pill)
                            }
                        }
                        .contentShape(.capsule)
                }
                .buttonStyle(.plain)
                .accessibilityHint(option.hint)
                .accessibilityAddTraits(selected ? .isSelected : [])
        }
    }
}

extension Exploration {
    var label: String {
        switch self {
        case .familiar: "Familiar"
        case .balanced: "Balanced"
        case .adventurous: "Adventurous"
        }
    }

    var hint: String {
        switch self {
        case .familiar: "Stay close to what you already love"
        case .balanced: "Mix favourites with new finds"
        case .adventurous: "Reach further from your usual music"
        }
    }
}

// MARK: - Rows

/// A playable track row: cover, title, artist and time, an optional
/// reason, and the moss accent when it is the one playing.
struct TrackRow: View {
    var track: Track
    var reason: String?
    var live: Bool

    var body: some View {
        HStack(spacing: 12) {
            Artwork(url: track.imageURL, seed: track.uri, size: 48)
                .overlay(alignment: .bottomLeading) {
                    if live {
                        Image(systemName: "waveform").font(.system(size: 11, weight: .bold)).foregroundStyle(Palette.onLamp)
                            .padding(3).background(Palette.lamp, in: .rect(cornerRadius: 4)).padding(3)
                    }
                }
            VStack(alignment: .leading, spacing: 2) {
                Text(track.name)
                    .font(Typeface.rowTitle)
                    .foregroundStyle(live ? Palette.lampText : Palette.text)
                    .lineLimit(2)
                Text(track.durationMs > 0 ? "\(track.artistLine) · \(duration(track.durationMs))" : track.artistLine)
                    .font(Typeface.detail)
                    .monospacedDigit()
                    .foregroundStyle(Palette.secondary)
                    .lineLimit(1)
                if let reason, !reason.isEmpty {
                    Text(reason).font(Typeface.detail).foregroundStyle(Palette.dim).lineLimit(3)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(.vertical, 4)
        .frame(minHeight: 56)
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(live ? [.isSelected] : [])
        // The current track, playing or paused.
        .accessibilityValue(live ? "Current track" : "")
    }
}

/// More like this / less like this, 44-point targets. A selected Love takes
/// the lamp (DESIGN.md's recorded exception); Less stays neutral.
struct FeedbackButtons: View {
    var rating: Rating?
    var enabled: Bool
    var onRate: (Rating?) -> Void

    var body: some View {
        HStack(spacing: 0) {
            button(.love, symbol: "hand.thumbsup", label: "More like this")
            button(.less, symbol: "hand.thumbsdown", label: "Less like this")
        }
        .disabled(!enabled)
        .sensoryFeedback(.selection, trigger: rating)
    }

    private func button(_ value: Rating, symbol: String, label: String) -> some View {
        let on = rating == value
        return Button {
            onRate(on ? nil : value)
        } label: {
            Image(systemName: on ? symbol + ".fill" : symbol)
                .font(.system(size: 16, weight: .medium))
                .foregroundStyle(on ? (value == .love ? Palette.lampText : Palette.text) : Palette.dim)
                .frame(width: 44, height: 44)
                .contentShape(.rect)
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label)
        .accessibilityAddTraits(on ? .isSelected : [])
    }
}

struct SectionTitle: View {
    var text: String
    var body: some View {
        Text(text).font(Typeface.section).foregroundStyle(Palette.text).accessibilityAddTraits(.isHeader)
    }
}

/// A one-line notice. Cautions stay neutral (desktop rule); the icon tells
/// the kind.
struct Notice: View {
    var text: String
    var symbol = "info.circle"
    var body: some View {
        Label { Text(text) } icon: { Image(systemName: symbol) }
            .font(Typeface.detail)
            .foregroundStyle(Palette.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}

extension View {
    /// Content rows sit on the room, not on glass.
    func roomRow() -> some View {
        listRowBackground(Color.clear)
            .listRowSeparatorTint(Palette.outline)
            .listRowInsets(EdgeInsets(top: 4, leading: 16, bottom: 4, trailing: 8))
    }
}
