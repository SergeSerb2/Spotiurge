//! Spotiurge's glass: panes of smoked glass floating over a dim room lit by
//! the playing cover.
//!
//! This is rendered frosted glass, not the system's backdrop blur. The room
//! behind the panes is a few large, soft colour fields, already as smooth as
//! blurred light, so a translucent pane over them reads as frosted without a
//! blur pass. Each pane adds a sheen along its top, a rim lit above and
//! shaded below, drawn one physical pixel wide, and a soft offset shadow.
//! Everything is a handful of meshes per frame and nothing here animates on
//! its own.

use egui::epaint::{ColorMode, PathShape, PathStroke, Shadow};
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, pos2, vec2};
use std::sync::Arc;

use crate::theme::Palette;

/// The gap between floating panes and between a pane and the window edge.
pub const GAP: f32 = 8.0;
/// Corner radius of the shell's panes: sidebar, page, side panels.
pub const PANE_RADIUS: f32 = 14.0;
/// Corner radius of the floating player console.
pub const CONSOLE_RADIUS: f32 = 18.0;
/// Corner radius of menus, popovers, toasts and dialogs.
pub const POPOVER_RADIUS: f32 = 12.0;

/// How a piece of glass sits in the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A shell pane: sidebar, page, queue, lyrics. Lets the room through.
    Pane,
    /// The player console: a little denser, it carries the controls.
    Console,
    /// Menus, popovers and dialogs over content: opaque, so the content
    /// beneath never competes with their text. Sheen, rim and shadow keep
    /// them glass.
    Popover,
    /// A quiet inset well inside a pane: fields, the taste prompt, wells
    /// holding a group of controls.
    Well,
}

/// The colours one piece of glass is drawn with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    pub fill: Color32,
    pub sheen: Color32,
    pub rim_top: Color32,
    pub rim_bottom: Color32,
    pub shadow: Option<Shadow>,
}

/// The glass for `kind` in this palette. Every colour derives from the
/// palette's roles, so custom themes keep working.
pub fn glass(palette: &Palette, kind: Kind) -> Glass {
    let dark = palette.dark;
    let (opacity, sheen, rim_top, rim_bottom) = match (kind, dark) {
        (Kind::Pane, true) => (0.72, 0.045, 0.11, 0.42),
        (Kind::Pane, false) => (0.58, 0.55, 0.95, 0.10),
        (Kind::Console, true) => (0.80, 0.06, 0.14, 0.45),
        (Kind::Console, false) => (0.72, 0.60, 1.0, 0.12),
        (Kind::Popover, true) => (1.0, 0.05, 0.13, 0.5),
        (Kind::Popover, false) => (1.0, 0.5, 1.0, 0.14),
        (Kind::Well, true) => (0.55, 0.0, 0.0, 0.0),
        (Kind::Well, false) => (0.75, 0.0, 0.0, 0.0),
    };
    let base = match kind {
        Kind::Popover => palette.overlay,
        Kind::Well => palette.surface,
        Kind::Pane | Kind::Console => palette.panel,
    };
    let shadow = match kind {
        Kind::Pane => Some(Shadow {
            offset: [0, 8],
            blur: 28,
            spread: 0,
            color: palette.shadow.gamma_multiply(if dark { 0.55 } else { 0.5 }),
        }),
        Kind::Console => Some(Shadow {
            offset: [0, 10],
            blur: 32,
            spread: 0,
            color: palette.shadow.gamma_multiply(if dark { 0.8 } else { 0.7 }),
        }),
        Kind::Popover => Some(Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color: palette.shadow,
        }),
        Kind::Well => None,
    };
    Glass {
        fill: base.gamma_multiply(opacity),
        sheen: Color32::WHITE.gamma_multiply(sheen),
        rim_top: Color32::WHITE.gamma_multiply(rim_top),
        rim_bottom: Color32::BLACK.gamma_multiply(rim_bottom),
        shadow,
    }
}

/// Paints a piece of glass filling `rect`. Paint it before what sits on it.
pub fn paint(painter: &Painter, rect: Rect, radius: f32, palette: &Palette, kind: Kind) {
    paint_glass(painter, rect, radius, &glass(palette, kind), palette);
}

/// Paints `glass` filling `rect`.
pub fn paint_glass(painter: &Painter, rect: Rect, radius: f32, glass: &Glass, palette: &Palette) {
    if !painter.is_visible() || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let corner = corner(radius);
    if let Some(shadow) = glass.shadow {
        painter.add(shadow.as_shape(rect, corner));
    }
    painter.rect_filled(rect, corner, glass.fill);
    if glass.sheen.a() > 0 {
        // The sheen fades out over the top of the pane, never more than
        // ninety points down, like light catching the upper edge.
        let depth = (rect.height() * 0.45).min(90.0);
        vertical_wash(
            painter,
            rect.shrink(0.5),
            radius,
            &[(0.0, glass.sheen), (depth, Color32::TRANSPARENT)],
        );
    }
    if glass.rim_top.a() > 0 || glass.rim_bottom.a() > 0 {
        rim(
            painter,
            rect,
            radius,
            glass.rim_top,
            glass.rim_bottom,
            palette,
        );
    }
}

/// A one-physical-pixel rim around `rect`, `top` along the upper edge
/// turning to `bottom` along the lower one, over the palette's outline.
pub fn rim(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    top: Color32,
    bottom: Color32,
    palette: &Palette,
) {
    let pixel = 1.0 / painter.pixels_per_point();
    let inset = rect.shrink(pixel / 2.0);
    let mut points = Vec::new();
    egui::epaint::tessellator::path::rounded_rectangle(
        &mut points,
        inset,
        egui::epaint::CornerRadiusF32::same(radius - pixel / 2.0),
    );
    let outline = palette
        .outline
        .gamma_multiply(if palette.dark { 0.6 } else { 0.8 });
    let blend = move |bounds: Rect, at: Pos2| {
        let t = ((at.y - bounds.top()) / bounds.height().max(1.0)).clamp(0.0, 1.0);
        // Lit along the top third, neutral down the sides, shaded below.
        let lit = (1.0 - t / 0.35).clamp(0.0, 1.0);
        let shade = ((t - 0.65) / 0.35).clamp(0.0, 1.0);
        over(
            over(outline, top.gamma_multiply(lit)),
            bottom.gamma_multiply(shade),
        )
    };
    painter.add(PathShape {
        points,
        closed: true,
        fill: Color32::TRANSPARENT,
        stroke: PathStroke {
            width: pixel.max(0.5),
            color: ColorMode::UV(Arc::new(blend)),
            kind: egui::StrokeKind::Middle,
        },
    });
}

/// `top` composited over `bottom`, both premultiplied.
fn over(bottom: Color32, top: Color32) -> Color32 {
    let alpha = f32::from(top.a()) / 255.0;
    let mix = |b: u8, t: u8| {
        (f32::from(t) + f32::from(b) * (1.0 - alpha))
            .round()
            .min(255.0) as u8
    };
    Color32::from_rgba_premultiplied(
        mix(bottom.r(), top.r()),
        mix(bottom.g(), top.g()),
        mix(bottom.b(), top.b()),
        mix(bottom.a(), top.a()),
    )
}

fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.clamp(0.0, 255.0) as u8)
}

/// Fills a rounded rectangle with a vertical gradient through `stops`:
/// (distance below the top in points, colour) pairs in order. Below the last
/// stop the last colour holds. Built as horizontal strips that follow the
/// corners, so any curve of stops keeps the rounded outline.
pub fn vertical_wash(painter: &Painter, rect: Rect, radius: f32, stops: &[(f32, Color32)]) {
    let Some(&(_, last)) = stops.last() else {
        return;
    };
    let radius = radius
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    let colour_at = |y: f32| -> Color32 {
        let offset = y - rect.top();
        let mut previous = stops[0];
        if offset <= previous.0 {
            return previous.1;
        }
        for &stop in &stops[1..] {
            if offset <= stop.0 {
                let span = (stop.0 - previous.0).max(f32::EPSILON);
                return lerp_colour(previous.1, stop.1, (offset - previous.0) / span);
            }
            previous = stop;
        }
        last
    };
    // Rows: the corners' curve, every stop, and the bottom of the wash.
    let end = (rect.top() + stops.iter().map(|stop| stop.0).fold(0.0, f32::max)).min(rect.bottom());
    let fill_rest = last.a() > 0;
    let bottom = if fill_rest { rect.bottom() } else { end };
    let mut ys: Vec<f32> = Vec::with_capacity(40);
    const CURVE_STEPS: usize = 8;
    for step in 0..=CURVE_STEPS {
        let y = rect.top() + radius * step as f32 / CURVE_STEPS as f32;
        ys.push(y);
        ys.push(rect.bottom() - radius * step as f32 / CURVE_STEPS as f32);
    }
    for &(offset, _) in stops {
        ys.push(rect.top() + offset);
    }
    // Long washes need rows along the way to follow their curve.
    let mut y = rect.top();
    while y < end {
        ys.push(y);
        y += 24.0;
    }
    ys.push(bottom);
    ys.retain(|y| (rect.top()..=bottom).contains(y));
    ys.sort_by(f32::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.25);
    if ys.len() < 2 {
        return;
    }
    let inset_at = |y: f32| -> f32 {
        let from_top = y - rect.top();
        let from_bottom = rect.bottom() - y;
        let into = if from_top < radius {
            radius - from_top
        } else if from_bottom < radius {
            radius - from_bottom
        } else {
            return 0.0;
        };
        radius - (radius * radius - into * into).max(0.0).sqrt()
    };
    let mut mesh = egui::Mesh::default();
    for &y in &ys {
        let inset = inset_at(y);
        let colour = colour_at(y);
        mesh.colored_vertex(pos2(rect.left() + inset, y), colour);
        mesh.colored_vertex(pos2(rect.right() - inset, y), colour);
    }
    for row in 0..ys.len() as u32 - 1 {
        let i = row * 2;
        mesh.add_triangle(i, i + 1, i + 3);
        mesh.add_triangle(i, i + 3, i + 2);
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn lerp_colour(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// A soft disc of light: `colour` at the centre falling smoothly to nothing
/// at `radius`. The falloff is eased across rings, so it has no visible edge.
pub fn light(painter: &Painter, center: Pos2, radius: f32, colour: Color32) {
    if colour.a() == 0 || radius <= 0.0 {
        return;
    }
    const SEGMENTS: u32 = 48;
    // Ring positions and how much light each keeps: a smooth bell.
    const RINGS: [(f32, f32); 5] = [
        (0.0, 1.0),
        (0.3, 0.78),
        (0.55, 0.45),
        (0.8, 0.14),
        (1.0, 0.0),
    ];
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, colour);
    for &(distance, strength) in &RINGS[1..] {
        let ring_colour = colour.gamma_multiply(strength);
        for segment in 0..SEGMENTS {
            let angle = std::f32::consts::TAU * segment as f32 / SEGMENTS as f32;
            let (sin, cos) = angle.sin_cos();
            mesh.colored_vertex(center + vec2(cos, sin) * radius * distance, ring_colour);
        }
    }
    for segment in 0..SEGMENTS {
        let next = (segment + 1) % SEGMENTS;
        mesh.add_triangle(0, 1 + segment, 1 + next);
        for ring in 0..RINGS.len() as u32 - 2 {
            let inner = 1 + ring * SEGMENTS;
            let outer = inner + SEGMENTS;
            mesh.add_triangle(inner + segment, outer + segment, outer + next);
            mesh.add_triangle(inner + segment, outer + next, inner + next);
        }
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// The room behind the panes: the window colour lit by the cover's light
/// from the upper left and a cool counter-light from the lower right.
pub fn paint_room(painter: &Painter, rect: Rect, palette: &Palette, light_colour: Color32) {
    painter.rect_filled(rect, 0.0, palette.window);
    let reach = rect.size().length();
    let (key, fill) = if palette.dark {
        (0.42, 0.20)
    } else {
        (0.40, 0.22)
    };
    light(
        painter,
        pos2(
            rect.left() + rect.width() * 0.18,
            rect.top() + rect.height() * 0.05,
        ),
        reach * 0.62,
        light_colour.gamma_multiply(key),
    );
    light(
        painter,
        pos2(
            rect.left() + rect.width() * 0.92,
            rect.bottom() - rect.height() * 0.02,
        ),
        reach * 0.5,
        counter_light(light_colour, palette).gamma_multiply(fill),
    );
}

/// The colour opposite the key light: its hue turned a third of the way
/// round, so the room never goes flat with one colour.
pub fn counter_light(colour: Color32, palette: &Palette) -> Color32 {
    let hsva = egui::ecolor::Hsva::from(colour);
    let turned = egui::ecolor::Hsva::new(
        (hsva.h + 0.33).fract(),
        hsva.s.min(0.55),
        if palette.dark {
            hsva.v.min(0.7)
        } else {
            hsva.v.max(0.8)
        },
        1.0,
    );
    Color32::from(turned)
}

/// The frame of a panel holding one pane of glass: clear, leaving `gaps`
/// outside the pane and `inside` between the pane's edge and its contents.
pub fn panel_frame(gaps: egui::Margin, inside: egui::Margin) -> egui::Frame {
    egui::Frame::new().inner_margin(gaps + inside)
}

/// Paints the pane of a panel framed by [`panel_frame`], from within its
/// contents, before anything else is drawn there.
pub fn paint_panel_pane(ui: &egui::Ui, inside: egui::Margin, palette: &Palette) -> Rect {
    let inner = ui.max_rect();
    let pane = Rect::from_min_max(
        inner.min - vec2(f32::from(inside.left), f32::from(inside.top)),
        inner.max + vec2(f32::from(inside.right), f32::from(inside.bottom)),
    );
    paint(ui.painter(), pane, PANE_RADIUS, palette, Kind::Pane);
    pane
}

/// The frame menus, popovers and toasts use: glass that keeps content
/// beneath it from competing with its text.
pub fn popover_frame(palette: &Palette) -> egui::Frame {
    let glass = glass(palette, Kind::Popover);
    egui::Frame::new()
        .fill(glass.fill)
        .stroke(Stroke::new(1.0, rim_colour(palette)))
        .corner_radius(corner(POPOVER_RADIUS))
        .shadow(glass.shadow.unwrap_or_default())
}

/// The single-colour rim egui frames take where a gradient one cannot go.
pub fn rim_colour(palette: &Palette) -> Color32 {
    if palette.dark {
        over(palette.outline, Color32::WHITE.gamma_multiply(0.06))
    } else {
        palette.outline
    }
}

/// The quiet fill under the pointer: light through dark glass, shade on
/// light glass.
pub fn hover_fill(palette: &Palette) -> Color32 {
    if palette.dark {
        Color32::WHITE.gamma_multiply(0.06)
    } else {
        Color32::BLACK.gamma_multiply(0.045)
    }
}

/// The fill of a key: a tile or shelf entry resting on a pane, a little
/// brighter than the glass around it, brightening further by `lift` (0 to 1)
/// under the pointer.
pub fn key_fill(palette: &Palette, lift: f32) -> Color32 {
    if palette.dark {
        Color32::WHITE.gamma_multiply(0.05 + 0.05 * lift)
    } else {
        Color32::BLACK.gamma_multiply(0.035 + 0.03 * lift)
    }
}

/// The fill of the selected row, a step above the hover fill.
pub fn selected_fill(palette: &Palette) -> Color32 {
    if palette.dark {
        Color32::WHITE.gamma_multiply(0.095)
    } else {
        Color32::BLACK.gamma_multiply(0.07)
    }
}

/// The selection lamp's shapes: the selected fill and a short bar of signal
/// colour at the leading edge, `strength` from 0 (off) to 1.
pub fn lamp_shapes(rect: Rect, radius: f32, palette: &Palette, strength: f32) -> Vec<egui::Shape> {
    if strength <= 0.0 {
        return Vec::new();
    }
    let bar = Rect::from_center_size(
        pos2(rect.left() + 1.5, rect.center().y),
        vec2(3.0, (rect.height() * 0.46).max(10.0)),
    );
    vec![
        egui::Shape::rect_filled(
            rect,
            corner(radius),
            selected_fill(palette).gamma_multiply(strength),
        ),
        egui::Shape::rect_filled(
            bar,
            CornerRadius::same(2),
            palette.accent.gamma_multiply(strength),
        ),
    ]
}

/// Paints the selection lamp over `rect`.
pub fn lamp(painter: &Painter, rect: Rect, radius: f32, palette: &Palette, strength: f32) {
    painter.extend(lamp_shapes(rect, radius, palette, strength));
}

/// The highlight behind a row or menu item: the hover fill easing in and
/// out under the pointer, and the lamp on the selected one.
pub fn row_highlight(
    ui: &egui::Ui,
    id: egui::Id,
    rect: Rect,
    radius: f32,
    palette: &Palette,
    hovered: bool,
    selected: bool,
) {
    let hover = super::motion::toggle(
        ui.ctx(),
        id.with("hover"),
        hovered && !selected,
        super::motion::FEEDBACK,
    );
    if hover > 0.0 {
        ui.painter().rect_filled(
            rect,
            corner(radius),
            hover_fill(palette).gamma_multiply(hover),
        );
    }
    if selected {
        lamp(ui.painter(), rect, radius, palette, 1.0);
    }
}

/// A lamp that glides between the items of a group, as a fader moves
/// between channels. Reserve its place with [`Glide::begin`] before drawing
/// the items, so it sits behind them, then [`Glide::end`] with the selected
/// item's key and rect, or `None` to let the lamp fade where it is. The lamp
/// glides only when the selection changes; when the layout moves the
/// selected item, the lamp moves with it.
pub struct Glide {
    slot: egui::layers::ShapeIdx,
    id: egui::Id,
}

impl Glide {
    pub fn begin(ui: &egui::Ui, id: egui::Id) -> Self {
        Self {
            slot: ui.painter().add(egui::Shape::Noop),
            id,
        }
    }

    pub fn end(self, ui: &egui::Ui, selected: Option<(u64, Rect)>, radius: f32, palette: &Palette) {
        let ctx = ui.ctx();
        let memory = self.id.with("selected");
        let last = ctx.data(|data| data.get_temp::<(u64, Rect)>(memory));
        let Some((key, target)) = selected.or(last) else {
            return;
        };
        let axes = [
            ("left", target.left()),
            ("top", target.top()),
            ("right", target.right()),
            ("bottom", target.bottom()),
        ];
        if let Some(selected) = selected {
            // The same item somewhere else: the layout moved it.
            if last.is_some_and(|(last_key, last_rect)| last_key == key && last_rect != target) {
                for (axis, value) in axes {
                    super::motion::snap(ctx, self.id.with(axis), value);
                }
            }
            ctx.data_mut(|data| data.insert_temp(memory, selected));
        }
        let duration = super::motion::STATE;
        let [left, top, right, bottom] = axes
            .map(|(axis, value)| super::motion::value(ctx, self.id.with(axis), value, duration));
        let rect = Rect::from_min_max(pos2(left, top), pos2(right, bottom));
        let strength = super::motion::toggle(ctx, self.id.with("on"), selected.is_some(), duration);
        ui.painter().set(
            self.slot,
            egui::Shape::Vec(lamp_shapes(rect, radius, palette, strength)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shapes(f: impl FnOnce(&Painter)) -> Vec<egui::epaint::ClippedShape> {
        let ctx = egui::Context::default();
        let mut f = Some(f);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(f) = f.take() {
                f(ui.painter());
            }
        });
        output.textures_delta.clear();
        output.shapes
    }

    /// Popovers stay readable over anything: opaque in both themes, and
    /// still glass, with sheen, rim and shadow, while shell panes let the
    /// room's light through.
    #[test]
    fn popovers_are_opaque_glass_over_translucent_panes() {
        for palette in [Palette::dark(), Palette::light()] {
            let pane = glass(&palette, Kind::Pane).fill.a();
            let popover = glass(&palette, Kind::Popover);
            assert_eq!(popover.fill.a(), 255, "popover alpha");
            assert_eq!(popover_frame(&palette).fill.a(), 255, "menu alpha");
            assert!(popover.sheen.a() > 0 && popover.rim_top.a() > 0);
            assert!(popover.shadow.is_some());
            assert!(pane < 255 && pane > 120, "pane alpha {pane}");
        }
    }

    /// A pane is a few shapes: its shadow, fill, sheen and rim, with no
    /// texture uploads or per-frame blur.
    #[test]
    fn a_pane_is_a_few_cheap_shapes() {
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(300.0, 400.0));
        let painted =
            shapes(|painter| paint(painter, rect, PANE_RADIUS, &Palette::dark(), Kind::Pane));
        assert_eq!(painted.len(), 4);
        let empty = shapes(|painter| {
            paint(
                painter,
                Rect::NOTHING,
                PANE_RADIUS,
                &Palette::dark(),
                Kind::Pane,
            );
        });
        assert!(empty.is_empty());
    }

    /// The wash keeps inside the rounded outline: its widest row in a
    /// corner is narrower than the rectangle.
    #[test]
    fn the_wash_follows_the_corners() {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 300.0));
        let painted = shapes(|painter| {
            vertical_wash(
                painter,
                rect,
                20.0,
                &[(0.0, Color32::WHITE), (120.0, Color32::TRANSPARENT)],
            );
        });
        let egui::Shape::Mesh(mesh) = &painted[0].shape else {
            panic!("a mesh");
        };
        let top_row: Vec<_> = mesh.vertices.iter().filter(|v| v.pos.y == 0.0).collect();
        assert!(top_row.iter().all(|v| v.pos.x >= 19.0 && v.pos.x <= 181.0));
        assert!(
            mesh.vertices.iter().all(|v| v.pos.y <= 120.0 + 0.01),
            "a transparent tail is not drawn"
        );
        let middle = mesh
            .vertices
            .iter()
            .find(|v| v.pos.y > 30.0 && v.pos.y < 100.0)
            .unwrap();
        assert_eq!(middle.pos.x.min(200.0 - middle.pos.x), 0.0);
    }

    /// The lamp glides when the selection changes, and moves with the
    /// layout, without gliding, when the selected item itself moves.
    #[test]
    fn the_lamp_glides_between_items_but_follows_the_layout() {
        let ctx = egui::Context::default();
        let palette = Palette::dark();
        let first = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 40.0));
        let second = first.translate(vec2(0.0, 40.0));
        let id = egui::Id::new("lamp");
        let lamp_top = |time: f64, selected: (u64, Rect)| {
            let mut top = None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    let glide = Glide::begin(ui, id);
                    glide.end(ui, Some(selected), 8.0, &palette);
                    // Read back where the lamp's top edge was drawn.
                    top = Some(super::super::motion::value(
                        ui.ctx(),
                        id.with("top"),
                        selected.1.top(),
                        super::super::motion::STATE,
                    ));
                },
            );
            output.textures_delta.clear();
            top.unwrap()
        };
        assert_eq!(lamp_top(0.0, (0, first)), 0.0);
        // A new selection: the lamp sets off from the old row.
        assert_eq!(lamp_top(1.0, (1, second)), 0.0);
        let middle = lamp_top(1.1, (1, second));
        assert!(0.0 < middle && middle < 40.0, "{middle}");
        assert_eq!(lamp_top(2.0, (1, second)), 40.0);
        // The same selection, moved by the layout: the lamp is there at once.
        assert_eq!(lamp_top(2.1, (1, second.translate(vec2(0.0, 25.0)))), 65.0);
    }

    /// The room's counter-light differs from the key light, so a cover
    /// with one colour still lights the room in two.
    #[test]
    fn the_counter_light_turns_the_hue() {
        let key = Color32::from_rgb(220, 60, 40);
        let counter = counter_light(key, &Palette::dark());
        assert_ne!(
            egui::ecolor::Hsva::from(key).h,
            egui::ecolor::Hsva::from(counter).h
        );
    }
}
