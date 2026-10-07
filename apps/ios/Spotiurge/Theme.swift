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
    static let room = Color(dark: 0x0B0D12, light: 0xE9EBF1)
    static let panel = Color(dark: 0x14171F, light: 0xFCFCFE)
    static let surface = Color(dark: 0x1C202A, light: 0xEEF0F5)
    static let surfaceActive = Color(dark: 0x2E3444, light: 0xD6DAE4)
    static let outline = Color(dark: 0x2A2F3C, light: 0xD5D9E3)
    static let text = Color(dark: 0xF3F2EF, light: 0x15171C)
    static let secondary = Color(dark: 0xA7ADBB, light: 0x4D5463)
    static let dim = Color(dark: 0x707787, light: 0x868D9C)
    /// The VU-amber lamp: live state only (playing, primary play, current tab).
    static let lamp = Color(dark: 0xFFB547, light: 0xA95C06)
    static let onLamp = Color(dark: 0x1F1303, light: 0xFFFFFF)
    /// The lamp for small text (playing titles): derived to reach 4.5:1.
    static let lampText = Color(dark: 0xFFB547, light: 0x653706)
    /// The opaque base of every sheet, menu and dialog that floats over content.
    static let overlay = Color(dark: 0x1C202B, light: 0xFFFFFF)
    static let danger = Color(dark: 0xFF6F73, light: 0xC83344)
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

// MARK: - The room

/// DESIGN.md's room: the window colour lit by two soft fields, the playing
/// cover's key light from the upper left and a counter-light a third of the
/// way round the hue wheel from the lower right. With no cover the key light
/// is the lamp amber (an amber and teal room). Kept faint so text over it
/// holds contrast; crosses over in 600 ms, immediately with Reduce Motion.
struct Room: View {
    var light: [Color]
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.colorScheme) private var scheme

    var body: some View {
        let strength = scheme == .dark ? 0.24 : 0.2
        let key = light.first ?? Palette.lamp
        ZStack {
            Palette.room
            RadialGradient(colors: [key.opacity(strength), .clear], center: .topLeading, startRadius: 0, endRadius: 540)
            RadialGradient(colors: [key.rotatedHue(1.0 / 3).opacity(strength * 0.7), .clear], center: .bottomTrailing, startRadius: 0, endRadius: 480)
        }
        .animation(Motion.curve(Motion.ambient, reduced: reduceMotion), value: light)
        .ignoresSafeArea()
    }
}

extension Color {
    func rotatedHue(_ turn: Double) -> Color {
        var h: CGFloat = 0, s: CGFloat = 0, b: CGFloat = 0, a: CGFloat = 0
        UIColor(self).getHue(&h, saturation: &s, brightness: &b, alpha: &a)
        return Color(hue: (h + turn).truncatingRemainder(dividingBy: 1), saturation: s, brightness: b, opacity: a)
    }
}

struct RoomLightKey: EnvironmentKey { static let defaultValue: [Color] = [] }

extension EnvironmentValues {
    var roomLight: [Color] {
        get { self[RoomLightKey.self] }
        set { self[RoomLightKey.self] = newValue }
    }
}

private struct RoomBackground: View {
    @Environment(\.roomLight) private var light
    var body: some View { Room(light: light) }
}

extension View {
    /// Puts a screen in the cover-lit room.
    func roomBackground() -> some View {
        scrollContentBackground(.hidden).background { RoomBackground() }
    }
}

// MARK: - Glass

extension View {
    /// System Liquid Glass for the control layer, in the desktop pane shape.
    func paneGlass(radius: CGFloat = Radius.pane, interactive: Bool = false) -> some View {
        glassEffect(interactive ? .regular.interactive() : .regular, in: .rect(cornerRadius: radius))
    }
}

/// The primary play control: the single amber lamp on a screen.
struct LampButtonStyle: ButtonStyle {
    /// DESIGN.md: 44 on the desk, 36 in the console.
    var size: CGFloat = 44
    @Environment(\.isEnabled) private var enabled

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: size * 0.4, weight: .bold))
            .foregroundStyle(Palette.onLamp)
            .frame(width: size, height: size)
            .background(Palette.lamp, in: .circle)
            // Nothing to play: the lamp is unlit, not merely faded.
            .saturation(enabled ? 1 : 0)
            .opacity(enabled ? 1 : 0.35)
            .scaleEffect(configuration.isPressed ? 0.94 : 1)
            .animation(.easeOut(duration: Motion.feedback), value: configuration.isPressed)
    }
}

// MARK: - The mark

/// The amber surge S on a smoked glass tile (assets/brand/spotiurge-mark.svg),
/// drawn from the same geometry: two 17-unit arcs, skewed -6.84 degrees.
struct Mark: View {
    var size: CGFloat = 28

    var body: some View {
        Canvas { context, canvas in
            let scale = canvas.width / 128
            context.scaleBy(x: scale, y: scale)
            let tile = Path(roundedRect: CGRect(x: 4, y: 4, width: 120, height: 120), cornerRadius: 27)
            context.fill(tile, with: .linearGradient(Gradient(colors: [Color(hex: 0x262C4A), Color(hex: 0x0A0C16)]), startPoint: CGPoint(x: 0, y: 4), endPoint: CGPoint(x: 0, y: 124)))
            context.fill(tile, with: .radialGradient(Gradient(colors: [Color(hex: 0xFF9228).opacity(0.22), .clear]), center: CGPoint(x: 64, y: 70), startRadius: 0, endRadius: 58))
            var s = Path()
            s.move(to: CGPoint(x: 78.72, y: 38.5))
            s.addArc(center: CGPoint(x: 64, y: 47), radius: 17, startAngle: .degrees(-30), endAngle: .degrees(90), clockwise: true)
            s.addArc(center: CGPoint(x: 64, y: 81), radius: 17, startAngle: .degrees(-90), endAngle: .degrees(150), clockwise: false)
            let skew = CGAffineTransform(a: 1, b: 0, c: -tan(6.84 * .pi / 180), d: 1, tx: 64 * tan(6.84 * .pi / 180), ty: 0)
            let surge = s.applying(skew)
            let stroke = StrokeStyle(lineWidth: 15, lineCap: .round)
            context.stroke(surge, with: .linearGradient(Gradient(colors: [Color(hex: 0xFFD680), Color(hex: 0xFF9228)]), startPoint: CGPoint(x: 0, y: 22), endPoint: CGPoint(x: 0, y: 106)), style: stroke)
            context.stroke(surge.applying(CGAffineTransform(translationX: -1.6, y: -2.1)), with: .color(Color(hex: 0xFFF4D6).opacity(0.3)), style: StrokeStyle(lineWidth: 4, lineCap: .round))
            context.stroke(Path(roundedRect: CGRect(x: 4.6, y: 4.6, width: 118.8, height: 118.8), cornerRadius: 26.4), with: .linearGradient(Gradient(stops: [.init(color: Color(hex: 0xEBF0FF).opacity(0.55), location: 0), .init(color: .clear, location: 0.5), .init(color: .black.opacity(0.5), location: 1)]), startPoint: CGPoint(x: 0, y: 4), endPoint: CGPoint(x: 0, y: 124)), lineWidth: 1.2)
        }
        .frame(width: size, height: size)
        .accessibilityHidden(true)
    }
}
