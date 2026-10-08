//! Flat chrome glass over a scenery photo.
//!
//! Like T3 Pretty, the scene sits beneath a black/white contrast wash. Glass
//! belongs to navigation and controls; the main page stays clear. The page's
//! Unsplash photo (see `crate::scenery`) loads through the artwork cache and
//! cross-fades in over the previous one; the bundled mountain lake, decoded
//! once off the UI thread, stands in until it arrives and when photos are off.

use crate::theme::Palette;
use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, Painter, Rect, Stroke, pos2, vec2};
use std::sync::Arc;

pub const GAP: f32 = 8.0;
pub const PANE_RADIUS: f32 = 14.0;
pub const CONSOLE_RADIUS: f32 = 18.0;
pub const POPOVER_RADIUS: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Pane,
    Content,
    Console,
    Popover,
    Well,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glass {
    pub fill: Color32,
    pub stroke: Stroke,
    pub shadow: Option<Shadow>,
}

pub fn glass(palette: &Palette, kind: Kind) -> Glass {
    let (base, opacity) = match kind {
        Kind::Pane => (palette.panel, 0.68),
        Kind::Content => (
            if palette.dark {
                palette.surface_active
            } else {
                palette.panel
            },
            if palette.dark { 0.85 } else { 0.68 },
        ),
        Kind::Console => (palette.panel, 0.80),
        Kind::Popover => (palette.overlay, 1.0),
        Kind::Well => (palette.surface, if palette.dark { 0.55 } else { 0.75 }),
    };
    Glass {
        fill: base.gamma_multiply(opacity),
        stroke: match kind {
            Kind::Pane | Kind::Content => Stroke::NONE,
            Kind::Well => Stroke::new(1.0, palette.outline),
            Kind::Console | Kind::Popover => Stroke::new(1.0, rim_colour(palette)),
        },
        shadow: (kind == Kind::Popover).then_some(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: palette.shadow,
        }),
    }
}

pub fn paint(painter: &Painter, rect: Rect, radius: f32, palette: &Palette, kind: Kind) {
    paint_glass(painter, rect, radius, &glass(palette, kind));
}

pub fn paint_glass(painter: &Painter, rect: Rect, radius: f32, glass: &Glass) {
    if !painter.is_visible() || !rect.is_positive() {
        return;
    }
    let corner = corner(radius);
    if let Some(shadow) = glass.shadow {
        painter.add(shadow.as_shape(rect, corner));
    }
    painter.rect(
        rect,
        corner,
        glass.fill,
        glass.stroke,
        egui::StrokeKind::Inside,
    );
}

fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius.clamp(0.0, 255.0) as u8)
}

const SCENERY: &[u8] = include_bytes!("../../assets/scenery/alpine-lake.jpg");

#[derive(Clone)]
enum Scene {
    Pending,
    Decoded(Arc<egui::ColorImage>),
    Ready(egui::TextureHandle),
    Failed,
}

fn decode_scene() -> Option<egui::ColorImage> {
    let image = image::load_from_memory(SCENERY).ok()?.to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ))
}

fn scene_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("spotiurge-static-scenery");
    match ctx.data(|data| data.get_temp::<Scene>(id)) {
        Some(Scene::Ready(texture)) => Some(texture),
        Some(Scene::Decoded(image)) => {
            let texture =
                ctx.load_texture("spotiurge-scenery", image, egui::TextureOptions::LINEAR);
            ctx.data_mut(|data| data.insert_temp(id, Scene::Ready(texture.clone())));
            Some(texture)
        }
        Some(Scene::Pending | Scene::Failed) => None,
        None => {
            ctx.data_mut(|data| data.insert_temp(id, Scene::Pending));
            let worker_ctx = ctx.clone();
            if std::thread::Builder::new()
                .name("spotiurge-scenery".into())
                .spawn(move || {
                    let result = decode_scene()
                        .map_or(Scene::Failed, |image| Scene::Decoded(Arc::new(image)));
                    worker_ctx.data_mut(|data| data.insert_temp(id, result));
                    worker_ctx.request_repaint();
                })
                .is_err()
            {
                ctx.data_mut(|data| data.insert_temp(id, Scene::Failed));
            }
            None
        }
    }
}

/// T3 Pretty's scenery stack, with the dense-text contrast boost enabled.
/// Image + wash cover 85% of the base, without an animated gradient or blur.
/// The dark wash is a little heavier than T3 Pretty's 0.612 so dim text
/// stays at 4.5:1 even over a pure white photo.
fn scenery_layers(palette: &Palette) -> (Color32, Color32) {
    let wash = if palette.dark { 0.63 } else { 0.68 };
    let image = (0.85 - wash) / (1.0 - wash);
    (
        Color32::WHITE.gamma_multiply(image),
        if palette.dark {
            Color32::BLACK
        } else {
            Color32::WHITE
        }
        .gamma_multiply(wash),
    )
}

/// The photo shown and the one it is fading in over.
#[derive(Clone, Default)]
struct Shown {
    current: Option<String>,
    previous: Option<String>,
}

/// A loaded photo's texture, kept within the artwork memory budget.
fn photo_texture(
    ctx: &egui::Context,
    art: &crate::images::ArtLoader,
    url: &str,
) -> Option<egui::load::SizedTexture> {
    art.touch(url);
    let Ok(egui::load::TexturePoll::Ready { texture }) =
        ctx.try_load_texture(url, egui::TextureOptions::LINEAR, egui::SizeHint::default())
    else {
        return None;
    };
    art.release_bytes(url);
    art.note_decoded(
        url,
        texture.size.x.round() as usize,
        texture.size.y.round() as usize,
    );
    Some(texture)
}

/// Paints the window's scenery: `photo` once it has loaded, faded in over
/// what was there, else the bundled scene.
pub fn paint_scenery(
    painter: &Painter,
    rect: Rect,
    palette: &Palette,
    photo: Option<&str>,
    art: &crate::images::ArtLoader,
) {
    let ctx = painter.ctx();
    let id = egui::Id::new("spotiurge-scenery-shown");
    let mut shown = ctx
        .data(|data| data.get_temp::<Shown>(id))
        .unwrap_or_default();
    let target = photo.and_then(|url| photo_texture(ctx, art, url).map(|texture| (url, texture)));
    match (&target, photo) {
        (Some((url, _)), _) if shown.current.as_deref() != Some(*url) => {
            shown.previous = shown.current.replace((*url).to_string());
        }
        // Photos turned off, or a new one is not here yet: keep what shows.
        (None, None) => shown = Shown::default(),
        _ => {}
    }
    let current = target
        .map(|(_, texture)| texture)
        .or_else(|| photo_texture(ctx, art, shown.current.as_deref()?));
    let fade = super::motion::entrance(
        ctx,
        id.with("fade"),
        shown.current.as_deref().map_or(0, stable_key),
        super::motion::AMBIENT,
    );
    let under = (fade < 1.0)
        .then(|| {
            shown
                .previous
                .as_deref()
                .and_then(|url| photo_texture(ctx, art, url))
                .map(|texture| (texture.id, texture.size))
                .or_else(|| scene_texture(ctx).map(|texture| (texture.id(), texture.size_vec2())))
        })
        .flatten();
    if fade >= 1.0 {
        shown.previous = None;
    }
    ctx.data_mut(|data| data.insert_temp(id, shown));

    painter.rect_filled(rect, 0.0, palette.window);
    let (image, wash) = scenery_layers(palette);
    let layers: Vec<((egui::TextureId, egui::Vec2), f32)> = match current {
        Some(texture) => under
            .map(|under| (under, 1.0))
            .into_iter()
            .chain([((texture.id, texture.size), fade)])
            .collect(),
        None => scene_texture(ctx)
            .map(|texture| ((texture.id(), texture.size_vec2()), 1.0))
            .into_iter()
            .collect(),
    };
    if layers.is_empty() {
        return;
    }
    // Opaque photos first, then the window colour over them, so a fade
    // between two photos leaves the same image strength as either alone.
    for ((texture, size), alpha) in layers {
        painter.image(
            texture,
            rect,
            cover_uv(rect.size(), size),
            Color32::WHITE.gamma_multiply(alpha),
        );
    }
    painter.rect_filled(
        rect,
        0.0,
        palette
            .window
            .gamma_multiply(1.0 - f32::from(image.a()) / 255.0),
    );
    painter.rect_filled(rect, 0.0, wash);
}

fn stable_key(url: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    hasher.finish()
}

/// Centred aspect fill, shared across window sizes without resizing the texture.
fn cover_uv(view: egui::Vec2, image: egui::Vec2) -> Rect {
    let scale = (view.x / image.x).max(view.y / image.y);
    let visible = view / (image * scale);
    Rect::from_center_size(pos2(0.5, 0.5), visible)
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

/// A quiet, single-colour hairline for controls and popovers.
pub fn rim_colour(palette: &Palette) -> Color32 {
    palette
        .text
        .gamma_multiply(if palette.dark { 0.07 } else { 0.08 })
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
        Color32::WHITE.gamma_multiply(0.095 + 0.04 * lift)
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

/// The selected row's fill, with `strength` from 0 (off) to 1.
pub fn lamp_shapes(rect: Rect, radius: f32, palette: &Palette, strength: f32) -> Vec<egui::Shape> {
    if strength <= 0.0 {
        return Vec::new();
    }
    vec![egui::Shape::rect_filled(
        rect,
        corner(radius),
        selected_fill(palette).gamma_multiply(strength),
    )]
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
    /// with a flat hairline and offset shadow, while chrome lets the scene through.
    #[test]
    fn popovers_are_opaque_glass_over_translucent_panes() {
        for palette in [Palette::dark(), Palette::light()] {
            let pane = glass(&palette, Kind::Pane).fill.a();
            let popover = glass(&palette, Kind::Popover);
            assert_eq!(popover.fill.a(), 255, "popover alpha");
            assert_eq!(popover_frame(&palette).fill.a(), 255, "menu alpha");
            assert!(popover.stroke.width > 0.0);
            assert!(glass(&palette, Kind::Pane).shadow.is_none());
            assert!(popover.shadow.is_some());
            assert!(pane < 255 && pane > 120, "pane alpha {pane}");
        }
    }

    /// A chrome pane is one flat shape, with no per-frame blur or shadow.
    #[test]
    fn a_pane_is_a_few_cheap_shapes() {
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(300.0, 400.0));
        let painted =
            shapes(|painter| paint(painter, rect, PANE_RADIUS, &Palette::dark(), Kind::Pane));
        assert_eq!(painted.len(), 1);
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

    /// The real bundled photo, plus the extremes any Unsplash photo can
    /// reach, is sampled after the same premultiplied gamma compositing used
    /// by egui. Text remains readable both directly on the wash and beneath
    /// translucent chrome, including hovered/selected rows.
    #[test]
    fn every_scene_pixel_preserves_text_contrast() {
        let scene = decode_scene().expect("bundled JPEG");
        assert_eq!(scene.size, [960, 540]);
        let extremes = [
            Color32::BLACK,
            Color32::WHITE,
            Color32::from_rgb(255, 0, 0),
            Color32::from_rgb(0, 255, 0),
            Color32::from_rgb(0, 0, 255),
            Color32::from_rgb(255, 255, 0),
            Color32::from_rgb(0, 255, 255),
            Color32::from_rgb(255, 0, 255),
        ];
        for palette in [Palette::dark(), Palette::light()] {
            let (image, wash) = scenery_layers(&palette);
            let roles = [
                palette.text,
                palette.secondary,
                palette.dim,
                palette.accent_text(),
                palette.danger,
                palette.warning,
            ];
            let mut minima = [f32::MAX; 6];
            let mut chip_minimum = f32::MAX;
            for pixel in scene.pixels.iter().chain(&extremes) {
                let ground = palette
                    .window
                    .blend(pixel.gamma_multiply(f32::from(image.a()) / 255.0))
                    .blend(wash);
                for ground in [
                    ground,
                    ground.blend(glass(&palette, Kind::Pane).fill),
                    ground.blend(glass(&palette, Kind::Content).fill),
                ] {
                    // Check option labels over the actual resting and hover
                    // fills on each scene/chrome/card ground.
                    for lift in [0.0, 1.0] {
                        let fill = if palette.dark {
                            Color32::WHITE.gamma_multiply(0.08 + 0.04 * lift)
                        } else {
                            palette.surface.lerp_to_gamma(palette.surface_hover, lift)
                        };
                        chip_minimum = chip_minimum
                            .min(crate::theme::contrast(palette.text, ground.blend(fill)));
                    }
                    for ground in [
                        ground,
                        ground.blend(hover_fill(&palette)),
                        ground.blend(selected_fill(&palette)),
                    ] {
                        for (index, role) in roles.iter().enumerate() {
                            minima[index] =
                                minima[index].min(crate::theme::contrast(*role, ground));
                        }
                    }
                }
            }
            eprintln!("dark={} minima={minima:?}", palette.dark);
            assert!(
                chip_minimum >= 4.5,
                "dark={} chip label: {chip_minimum:.2}:1",
                palette.dark
            );
            for (role, ratio) in roles.iter().zip(minima) {
                assert!(
                    ratio >= 4.5,
                    "dark={} role={role:?}: {ratio:.2}:1",
                    palette.dark
                );
            }
        }
    }

    #[test]
    fn scene_crop_fills_wide_and_tall_windows_without_distortion() {
        for view in [vec2(1440.0, 900.0), vec2(900.0, 760.0), vec2(360.0, 800.0)] {
            let uv = cover_uv(view, vec2(960.0, 540.0));
            assert!(uv.min.x >= 0.0 && uv.min.y >= 0.0 && uv.max.x <= 1.0 && uv.max.y <= 1.0);
            assert!((uv.width() * 960.0 / (uv.height() * 540.0) - view.x / view.y).abs() < 0.001);
        }
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
}
