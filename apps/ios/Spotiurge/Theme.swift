// The desktop's control-room language on a phone. Tokens are copied from
// src/theme.rs (palette), src/ui/material.rs (glass, radii) and
// src/ui/motion.rs (durations); keep them in step with those files.

import SwiftUI

extension Color {
    init(hex: UInt32) {
        self.init(.sRGB, red: Double(hex >> 16 & 0xFF) / 255, green: Double(hex >> 8 & 0xFF) / 255, blue: Double(hex & 0xFF) / 255)
    }

    init(dark: UInt32, light: UInt32) {
        self.init(UIColor { traits in
            let hex = traits.userInterfaceStyle == .dark ? dark : light
            return UIColor(red: CGFloat(hex >> 16 & 0xFF) / 255, green: CGFloat(hex >> 8 & 0xFF) / 255, blue: CGFloat(hex & 0xFF) / 255, alpha: 1)
        })
    }
}

/// `Palette::dark()` and `Palette::light()` from src/theme.rs, with the
/// roles DESIGN.md names (lamp text, overlay).
enum Palette {
    static let room = Color(dark: 0x0E1110, light: 0xF4F6F4)
    static let panel = Color(dark: 0x141A17, light: 0xFFFFFF)
    static let surface = Color(dark: 0x1A221E, light: 0xEAF0EB)
    static let surfaceActive = Color(dark: 0x2C3A32, light: 0xC9D1CA)
    static let outline = Color(dark: 0x2E3B34, light: 0xD8DED9)
    static let text = Color(dark: 0xF3F6F3, light: 0x161A17)
    static let secondary = Color(dark: 0xC5CFC8, light: 0x4B524C)
    static let dim = Color(dark: 0xB8C5BB, light: 0x47504A)
    static let disabled = Color(dark: 0x84988B, light: 0x59695E)
    /// Moss is reserved for live state and the primary playback control.
    static let lamp = Color(dark: 0x98D2AC, light: 0x27633F)
    static let onLamp = Color(dark: 0x07140C, light: 0xFFFFFF)
    static let lampText = Color(dark: 0x98D2AC, light: 0x27633F)
    static let overlay = Color(dark: 0x202A25, light: 0xFFFFFF)
    static let danger = Color(dark: 0xFFB0B8, light: 0x951524)
    static let warning = Color(dark: 0xFFB020, light: 0x7B3605)
}

/// Inter, ranked by weight (src/theme.rs). The variable face is staged into
/// the bundle by build.sh from the desktop's pinned fastframe-fonts; without
/// it the system face stands in at the same sizes and weights.
enum Typeface {
    static let family = "Inter Variable"

    static func inter(_ size: CGFloat, _ weight: Font.Weight = .regular, relativeTo style: Font.TextStyle = .body) -> Font {
        .custom(family, size: size, relativeTo: style).weight(weight)
    }

    static let display = inter(26, .bold, relativeTo: .title)
    static let title = inter(20, .semibold, relativeTo: .title3)
    static let section = inter(17, .semibold, relativeTo: .headline)
    static let rowTitle = inter(15, .semibold, relativeTo: .body)
    static let body = inter(15, .regular, relativeTo: .body)
    static let detail = inter(13, .regular, relativeTo: .subheadline)
    static let caption = inter(12, .medium, relativeTo: .caption)
}

/// src/ui/motion.rs: finite exponential ease-out; Reduce Motion is immediate.
enum Motion {
    static let feedback = 0.12
    static let state = 0.22
    static let page = 0.22
    static let ambient = 0.6
    static let pageRise: CGFloat = 6

    static func curve(_ duration: Double, reduced: Bool) -> Animation? {
        reduced ? nil : .timingCurve(0.16, 1, 0.3, 1, duration: duration)
    }
}

/// src/ui/material.rs radii, on a 4-point grid.
enum Radius {
    static let pane: CGFloat = 14
    static let console: CGFloat = 18
    static let popover: CGFloat = 12
    static let gap: CGFloat = 8
}

// MARK: - Static mountain scenery

/// The desktop/T3 Pretty scene stack: one bundled image and a contrast wash.
/// It never depends on playback and performs no image network requests.
struct Room: View {
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Palette.room
                Image("Scenery").resizable().scaledToFill()
                    .frame(width: geometry.size.width, height: geometry.size.height)
                    .clipped()
                    .opacity(scheme == .dark ? 0.613402 : 0.53125)
                (scheme == .dark ? Color.black : Color.white)
                    .opacity(scheme == .dark ? 0.612 : 0.68)
            }
        }
        .ignoresSafeArea()
        .accessibilityHidden(true)
    }
}

extension View {
    func roomBackground() -> some View {
        scrollContentBackground(.hidden).background { Room() }
    }
}

// MARK: - Glass

private struct ContentPlate: ViewModifier {
    var radius: CGFloat
    @Environment(\.colorScheme) private var scheme
    func body(content: Content) -> some View {
        content.background((scheme == .dark ? Palette.surfaceActive : Palette.panel)
            .opacity(scheme == .dark ? 0.85 : 0.68), in: .rect(cornerRadius: radius))
    }
}

extension View {
    /// Quiet content plates; native Liquid Glass stays in the control layer.
    func paneGlass(radius: CGFloat = Radius.pane, interactive: Bool = false) -> some View {
        modifier(ContentPlate(radius: radius))
    }
}

/// The primary play control: the moss play control on a screen.
struct LampButtonStyle: ButtonStyle {
    /// DESIGN.md: 44 on the desk, 36 in the console.
    var size: CGFloat = 44
    @Environment(\.isEnabled) private var enabled

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: size * 0.4, weight: .bold))
            .foregroundStyle(enabled ? Palette.onLamp : Palette.disabled)
            .frame(width: size, height: size)
            .background(enabled ? Palette.lamp : Palette.surfaceActive, in: .circle)
            .scaleEffect(configuration.isPressed ? 0.94 : 1)
            .animation(.easeOut(duration: Motion.feedback), value: configuration.isPressed)
    }
}

// MARK: - The mark

/// The same flat ridge geometry as assets/brand/spotiurge-glyph.svg.
struct Mark: View {
    var size: CGFloat = 28
    var body: some View {
        Canvas { context, canvas in
            let scale = canvas.width / 84
            context.scaleBy(x: scale, y: scale)
            context.translateBy(x: -22, y: -22)
            var ridge = Path()
            for vertices in [
                [CGPoint(x: 24, y: 98), CGPoint(x: 24, y: 72), CGPoint(x: 46, y: 47), CGPoint(x: 46, y: 98)],
                [CGPoint(x: 53, y: 98), CGPoint(x: 53, y: 39.1), CGPoint(x: 61, y: 30), CGPoint(x: 75, y: 42.4), CGPoint(x: 75, y: 98)],
                [CGPoint(x: 82, y: 98), CGPoint(x: 82, y: 48.6), CGPoint(x: 104, y: 68), CGPoint(x: 104, y: 98)]
            ] {
                ridge.move(to: vertices[0])
                for point in vertices.dropFirst() { ridge.addLine(to: point) }
                ridge.closeSubpath()
            }
            context.fill(ridge, with: .color(Palette.lamp))
        }
        .frame(width: size, height: size)
        .accessibilityHidden(true)
    }
}
