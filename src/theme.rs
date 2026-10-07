//! Shared palette, typography, icons, and base widgets.
//!
//! Inter provides real font weights, and Lucide provides a consistent icon set.
//! All colors use [`Palette`] so light, dark, and album-art-tinted themes stay
//! consistent.

use crate::i18n::{Locale, gettext};
use egui::{Color32, CornerRadius, Response, Sense, Stroke, Vec2};
use std::borrow::Cow;

/// A palette file from the themes directory.
pub type CustomTheme = fastframe_theme::CustomTheme<Palette>;
/// The palette files, and the Omarchy palette where the desktop has one.
pub type Catalog = fastframe_theme::Catalog<Palette>;

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Palette {
    pub dark: bool,
    pub window: Color32,
    pub panel: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub outline: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub dim: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub overlay: Color32,
    pub shadow: Color32,
}

impl Palette {
    /// Inactive controls have their own ink; dim text still carries information.
    pub fn disabled_text(&self) -> Color32 {
        if self.dark {
            Color32::from_rgb(0x84, 0x98, 0x8b)
        } else {
            Color32::from_rgb(0x59, 0x69, 0x5e)
        }
    }

    /// T3 Pretty's night forest palette, with moss for live controls.
    pub fn dark() -> Self {
        Self {
            dark: true,
            window: Color32::from_rgb(0x0e, 0x11, 0x10),
            panel: Color32::from_rgb(0x14, 0x1a, 0x17),
            surface: Color32::from_rgb(0x1a, 0x22, 0x1e),
            surface_hover: Color32::from_rgb(0x20, 0x2a, 0x25),
            surface_active: Color32::from_rgb(0x2c, 0x3a, 0x32),
            outline: Color32::from_rgb(0x2e, 0x3b, 0x34),
            text: Color32::from_rgb(0xf3, 0xf6, 0xf3),
            secondary: Color32::from_rgb(0xc5, 0xcf, 0xc8),
            dim: Color32::from_rgb(0xb8, 0xc5, 0xbb),
            accent: Color32::from_rgb(0x98, 0xd2, 0xac),
            accent_hover: Color32::from_rgb(0xb7, 0xe6, 0xc8),
            on_accent: Color32::from_rgb(0x07, 0x14, 0x0c),
            danger: Color32::from_rgb(0xff, 0xb0, 0xb8),
            warning: Color32::from_rgb(0xff, 0xb0, 0x20),
            overlay: Color32::from_rgb(0x20, 0x2a, 0x25),
            shadow: Color32::from_black_alpha(115),
        }
    }

    /// T3 Pretty's mist palette, with a deep forest accent for contrast.
    pub fn light() -> Self {
        Self {
            dark: false,
            window: Color32::from_rgb(0xf4, 0xf6, 0xf4),
            panel: Color32::from_rgb(0xff, 0xff, 0xff),
            surface: Color32::from_rgb(0xea, 0xf0, 0xeb),
            surface_hover: Color32::from_rgb(0xe3, 0xe9, 0xe4),
            surface_active: Color32::from_rgb(0xc9, 0xd1, 0xca),
            outline: Color32::from_rgb(0xd8, 0xde, 0xd9),
            text: Color32::from_rgb(0x16, 0x1a, 0x17),
            secondary: Color32::from_rgb(0x4b, 0x52, 0x4c),
            dim: Color32::from_rgb(0x47, 0x50, 0x4a),
            accent: Color32::from_rgb(0x27, 0x63, 0x3f),
            accent_hover: Color32::from_rgb(0x22, 0x57, 0x38),
            on_accent: Color32::WHITE,
            danger: Color32::from_rgb(0x95, 0x15, 0x24),
            warning: Color32::from_rgb(0x7b, 0x36, 0x05),
            overlay: Color32::from_rgb(0xff, 0xff, 0xff),
            shadow: Color32::from_rgba_unmultiplied(16, 24, 18, 36),
        }
    }

    /// The lamp's colour for small text: playing titles, the Connect pill,
    /// the active device. Fills keep the accent. Where the accent reads at
    /// 4.5:1 it is used as it is; otherwise it deepens (on dark glass,
    /// brightens) in steps that keep its hue until it does. It is measured
    /// against a conservative shaded `surface_active`. Custom palettes keep
    /// this guard beyond the built-in scenery contrast tests.
    pub fn accent_text(&self) -> Color32 {
        let toward = if self.dark {
            Color32::WHITE
        } else {
            Color32::BLACK
        };
        let shade = if self.dark { 0.1 } else { 0.2 };
        let ground = self.surface_active.lerp_to_gamma(toward, shade);
        let mut colour = self.accent;
        for _ in 0..40 {
            if contrast(colour, ground) >= 4.5 {
                break;
            }
            colour = colour.lerp_to_gamma(toward, 0.05);
        }
        colour
    }

    /// A colour derived from album art, softened so it can sit behind text.
    pub fn tint_from_art(&self, rgb: [u8; 3]) -> Color32 {
        let [r, g, b] = rgb.map(|c| c as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let lightness = (max + min) / 2.0;
        let target = if self.dark { 0.30 } else { 0.72 };
        let (r, g, b) = if lightness < 0.01 {
            (target, target, target)
        } else {
            let scale = target / lightness;
            (
                (r * scale).min(1.0),
                (g * scale).min(1.0),
                (b * scale).min(1.0),
            )
        };
        Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

/// The WCAG contrast ratio between two opaque colours, from 1 to 21.
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let luminance = |colour: Color32| {
        let linear = egui::Rgba::from(colour);
        0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b()
    };
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

impl fastframe_theme::Palette for Palette {
    fn base(base: fastframe_theme::Base) -> Self {
        match base {
            fastframe_theme::Base::Dark => Self::dark(),
            fastframe_theme::Base::Light => Self::light(),
        }
    }

    fn set(&mut self, name: &str, color: Color32) -> bool {
        match name {
            "window" => self.window = color,
            "panel" => self.panel = color,
            "surface" => self.surface = color,
            "surface_hover" => self.surface_hover = color,
            "surface_active" => self.surface_active = color,
            "outline" => self.outline = color,
            "text" => self.text = color,
            "secondary" => self.secondary = color,
            "dim" => self.dim = color,
            "accent" => self.accent = color,
            "accent_hover" => self.accent_hover = color,
            "on_accent" => self.on_accent = color,
            "danger" => self.danger = color,
            "warning" => self.warning = color,
            "overlay" => self.overlay = color,
            "shadow" => self.shadow = color,
            _ => return false,
        }
        true
    }
}

/// Adds the desktop's palettes to a normal launch: the eight shared palettes,
/// installed into the themes folder as files on the first launch, and
/// Omarchy's on Linux, with the packaged template and hook installed for the
/// user.
pub fn enable_desktop_themes(catalog: &mut Catalog) {
    catalog.enable_desktop_themes(fastframe_theme::DesktopThemes {
        slug: "spotifast",
        omarchy_template: include_str!("../contrib/omarchy/spotifast.json.tpl"),
        // The template has not changed since it first shipped.
        omarchy_previous_templates: &[],
        presets: true,
    });
}

/// The status line under the Theme setting, empty when all is well.
pub fn catalog_detail(
    catalog: &Catalog,
    locale: Locale,
    selected: Option<&str>,
) -> Cow<'static, str> {
    use fastframe_theme::{Problem, Status};
    let Some(status) = catalog.status(selected) else {
        return Cow::Borrowed("");
    };
    match status {
        Status::Loading => gettext(locale, "Loading local themes…"),
        Status::SelectedUnavailable => gettext(
            locale,
            "The selected theme is unavailable. Keeping the last usable appearance. See the log for details.",
        ),
        Status::Problem(Problem::Unreadable) => gettext(
            locale,
            "The themes folder could not be read. See the log for details.",
        ),
        Status::Problem(Problem::TooManyEntries) => gettext(
            locale,
            "The themes folder has more than 512 entries. Keep fewer files there to list the custom palettes.",
        ),
        Status::Problem(Problem::TooManyThemes) => gettext(
            locale,
            "Only 128 custom palettes can be listed. Keep fewer JSON files in the themes folder to see the rest.",
        ),
        Status::Problem(Problem::OmarchyUnreadable) => gettext(
            locale,
            "The Omarchy palette could not be loaded. Keeping the last usable appearance. See the log for details.",
        ),
        Status::Problem(_) => gettext(
            locale,
            "Custom themes could not be loaded. Run spotifast reload-themes to try again.",
        ),
    }
}

pub const RADIUS: u8 = 8;
pub const RADIUS_SMALL: u8 = 4;
pub const ROW_HEIGHT: f32 = 56.0;
pub const COMPACT_ROW_HEIGHT: f32 = 48.0;
/// The compact track list: one line, no cover.
pub const THIN_ROW_HEIGHT: f32 = 36.0;
pub const PLAYER_BAR_HEIGHT: f32 = 88.0;
/// The narrowest either right-hand panel goes. The queue and the lyrics
/// take the same edge and swap places there, so a width that suits one
/// has to suit the other, or the window would jump on the swap.
pub const SIDE_PANEL_MIN_WIDTH: f32 = 280.0;
pub const TOP_BAR_HEIGHT: f32 = 56.0;

/// macOS hides the titlebar and draws the window content all the way to the
/// top edge, so whatever sits at the top of the window has to leave room for
/// the traffic lights. Zero everywhere else, and in fullscreen, where the
/// buttons are gone.
pub fn titlebar_inset(ctx: &egui::Context) -> f32 {
    if cfg!(target_os = "macos") && !ctx.input(|input| input.viewport().fullscreen.unwrap_or(false))
    {
        28.0
    } else {
        0.0
    }
}

pub fn regular(size: f32) -> egui::FontId {
    fastframe_fonts::Weight::Regular.font_id(size)
}

pub fn medium(size: f32) -> egui::FontId {
    fastframe_fonts::Weight::Medium.font_id(size)
}

pub fn semibold(size: f32) -> egui::FontId {
    fastframe_fonts::Weight::SemiBold.font_id(size)
}

pub fn bold(size: f32) -> egui::FontId {
    fastframe_fonts::Weight::Bold.font_id(size)
}

/// How the desktop renders text, read once per process.
///
/// Tests use the platform's default instead of asking the desktop, so they
/// neither wait on D-Bus nor depend on the machine's settings.
pub fn text_rendering() -> fastframe_text::TextRendering {
    static RENDERING: std::sync::OnceLock<fastframe_text::TextRendering> =
        std::sync::OnceLock::new();
    *RENDERING.get_or_init(|| {
        if cfg!(test) {
            fastframe_text::TextRendering::platform_default()
        } else {
            fastframe_text::detect()
        }
    })
}

/// Install fonts, icons, and the base style once.
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    egui_extras::install_image_loaders(ctx);
    fastframe_icons::install::<Icon>(ctx);
    // Colour emoji over every text egui draws (see `crate::emoji`).
    ctx.add_plugin(fastframe_emoji::EmojiPlugin::default());
}

/// Applies the palette to egui's own widgets so dialogs, menus, and text
/// fields agree with the custom views.
pub fn apply(ctx: &egui::Context, palette: &Palette) {
    let mut style = (*ctx.global_style()).clone();
    apply_to_style(&mut style, palette, crate::ui::motion::reduced(ctx));
    ctx.set_global_style(style);
}

/// Applies a palette to this view and children without changing global style.
pub fn apply_local(ui: &mut egui::Ui, palette: &Palette) {
    let reduced = crate::ui::motion::reduced(ui.ctx());
    apply_to_style(ui.style_mut(), palette, reduced);
}

fn apply_to_style(style: &mut egui::Style, palette: &Palette, reduced: bool) {
    let visuals = &mut style.visuals;
    *visuals = if palette.dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.dark_mode = palette.dark;
    // Glyph coverage, hinting and sub-pixel positions as the desktop draws
    // them: linear coverage in both themes on Linux, where egui's dark curve
    // (2c - c²) made text heavier than GTK's.
    text_rendering().apply_to_visuals(visuals);
    visuals.panel_fill = palette.panel;
    let popover = crate::ui::material::glass(palette, crate::ui::material::Kind::Popover);
    visuals.window_fill = popover.fill;
    visuals.extreme_bg_color = palette.surface;
    visuals.faint_bg_color = palette.surface;
    visuals.code_bg_color = palette.surface;
    visuals.override_text_color = Some(palette.text);
    visuals.weak_text_color = Some(palette.secondary);
    visuals.hyperlink_color = palette.text;
    visuals.selection.bg_fill = palette.accent.gamma_multiply(0.35);
    visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    visuals.window_stroke = Stroke::new(1.0, crate::ui::material::rim_colour(palette));
    let popover_radius = crate::ui::material::POPOVER_RADIUS as u8;
    visuals.window_corner_radius = CornerRadius::same(popover_radius);
    visuals.menu_corner_radius = CornerRadius::same(popover_radius);
    visuals.window_shadow = popover.shadow.unwrap_or_default();
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: palette.shadow,
    };
    let corner = CornerRadius::same(RADIUS_SMALL + 2);
    for widget in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = corner;
        widget.bg_stroke = Stroke::NONE;
        widget.fg_stroke = Stroke::new(1.0, palette.text);
        widget.expansion = 0.0;
    }
    visuals.widgets.noninteractive.corner_radius = corner;
    visuals.widgets.noninteractive.bg_fill = palette.panel;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, palette.outline);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.bg_fill = palette.surface;
    visuals.widgets.inactive.weak_bg_fill = palette.surface;
    visuals.widgets.hovered.bg_fill = palette.surface_hover;
    visuals.widgets.hovered.weak_bg_fill = palette.surface_hover;
    visuals.widgets.active.bg_fill = palette.surface_active;
    visuals.widgets.active.weak_bg_fill = palette.surface_active;
    visuals.widgets.open.bg_fill = palette.surface_hover;
    visuals.widgets.open.weak_bg_fill = palette.surface_hover;
    visuals.text_cursor.stroke = Stroke::new(2.0, palette.accent);
    visuals.striped = false;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Circle;

    use egui::FontFamily::{Monospace, Proportional};
    use egui::{FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Small, FontId::new(11.5, Proportional)),
        (TextStyle::Body, FontId::new(14.0, Proportional)),
        (TextStyle::Button, FontId::new(14.0, Proportional)),
        (TextStyle::Heading, FontId::new(22.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Monospace)),
    ]
    .into();
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.spacing.button_padding = Vec2::new(12.0, 6.0);
    style.spacing.interact_size = Vec2::new(40.0, 28.0);
    style.spacing.menu_margin = egui::Margin::same(6);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.scroll = egui::style::ScrollStyle {
        bar_width: 8.0,
        floating_width: 6.0,
        floating_allocated_width: 0.0,
        handle_min_length: 28.0,
        bar_inner_margin: 3.0,
        bar_outer_margin: 2.0,
        dormant_background_opacity: 0.0,
        dormant_handle_opacity: 0.0,
        active_background_opacity: 0.0,
        active_handle_opacity: 0.55,
        interact_handle_opacity: 0.85,
        foreground_color: true,
        ..egui::style::ScrollStyle::floating()
    };
    style.interaction.selectable_labels = false;
    style.interaction.tooltip_delay = 0.4;
    style.url_in_tooltip = false;
    crate::ui::motion::apply_to_style(style, reduced);
}

/// Inter at its four weights with the monochrome emoji face right behind it
/// (so every emoji wears the same style, ahead of egui's own pair), then the
/// installed faces for the scripts Inter lacks, drawn the way the desktop
/// renders text.
fn install_fonts(ctx: &egui::Context) {
    let emoji = egui::FontData::from_static(include_bytes!("../assets/fonts/NotoEmoji.ttf"));
    let mut fonts = fastframe_fonts::FontSetup::default()
        .companion("noto_emoji", std::sync::Arc::new(emoji))
        .definitions();
    text_rendering().apply_to(&mut fonts);
    ctx.set_fonts(fonts);
}

fastframe_icons::icons! {
    /// Every icon the interface draws. The shared Lucide icons come from
    /// fastframe-icons; the rest are Spotifast's own files.
    pub enum Icon {
        prefix: "spotifast-icon-",
        directory: "../assets/icons/",
        ArrowLeft => lucide "arrow-left",
        ArrowRight => "arrow-right",
        AudioLines => "audio-lines",
        BadgeCheck => "badge-check",
        Bookmark => "bookmark",
        BookmarkFilled => "bookmark-filled",
        Car => "car",
        Cast => "cast",
        Check => lucide "check",
        ChevronDown => lucide "chevron-down",
        ChevronLeft => lucide "chevron-left",
        ChevronRight => lucide "chevron-right",
        ChevronUp => lucide "chevron-up",
        CircleAlert => lucide "circle-alert",
        CircleCheck => lucide "circle-check",
        CirclePlay => "circle-play",
        CirclePlus => "circle-plus",
        CircleX => lucide "circle-x",
        Clock => lucide "clock",
        Compass => "compass",
        Copy => lucide "copy",
        Disc => "disc-3",
        Ellipsis => lucide "ellipsis",
        Expand => "expand",
        ExternalLink => lucide "external-link",
        Gamepad => "gamepad-2",
        Globe => "globe",
        GripVertical => "grip-vertical",
        Headphones => "headphones",
        Heart => "heart",
        HeartFilled => "heart-filled",
        House => "house",
        Info => lucide "info",
        Laptop => "laptop",
        Library => "library",
        LayoutGrid => "layout-grid",
        LayoutList => "layout-list",
        ListEnd => "list-end",
        ListMusic => "list-music",
        ListPlus => "list-plus",
        ListVideo => "list-video",
        Loader => "loader-circle",
        Lock => lucide "lock",
        LogOut => lucide "log-out",
        Mic => lucide "mic",
        Minus => lucide "minus",
        Monitor => lucide "monitor",
        Moon => lucide "moon",
        Music => "music",
        Pause => lucide "pause",
        PauseFilled => "pause-filled",
        PanelLeft => lucide "panel-left",
        Pin => lucide "pin",
        PinOff => lucide "pin-off",
        Pencil => lucide "pencil",
        Play => lucide "play",
        PlayFilled => "play-filled",
        Plus => lucide "plus",
        Radio => "radio",
        Refresh => lucide "refresh-cw",
        Repeat => "repeat",
        Repeat1 => "repeat-1",
        Search => lucide "search",
        Settings => lucide "settings",
        Shrink => "shrink",
        Shuffle => "shuffle",
        SkipBack => "skip-back",
        SkipBackFilled => "skip-back-filled",
        SkipForward => "skip-forward",
        SkipForwardFilled => "skip-forward-filled",
        Smartphone => lucide "smartphone",
        Sparkles => "sparkles",
        Speaker => "speaker",
        Square => "square",
        SquarePen => lucide "square-pen",
        Sun => lucide "sun",
        Tablet => "tablet",
        ThumbsDown => "thumbs-down",
        ThumbsUp => "thumbs-up",
        Trash => lucide "trash-2",
        TrendingUp => "trending-up",
        Tv => "tv",
        User => lucide "user",
        Users => lucide "users",
        Volume => "volume",
        Volume1 => "volume-1",
        Volume2 => lucide "volume-2",
        VolumeX => lucide "volume-x",
        Watch => "watch",
        X => lucide "x",
        Zap => "zap",
    }
}

/// A static icon.
pub fn icon(ui: &mut egui::Ui, icon: Icon, size: f32, color: Color32) -> Response {
    ui.add(icon.image(color, size))
}

/// Paints an icon centred in `rect` without allocating space.
pub fn paint_icon(ui: &egui::Ui, icon: Icon, rect: egui::Rect, size: f32, color: Color32) {
    let icon_rect = egui::Rect::from_center_size(
        rect.center() + play_glyph_offset(icon, size),
        Vec2::splat(size),
    );
    icon.image(color, size).paint_at(ui, icon_rect);
}

/// Make keyboard focus visible without changing the control's layout.
pub fn focus_ring(ui: &egui::Ui, response: &Response) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.expand(2.0),
            4.0,
            ui.visuals().selection.stroke,
            egui::StrokeKind::Outside,
        );
    }
    if response.gained_focus() {
        response.scroll_to_me(None);
    }
}

/// A frameless icon control whose colour lifts on hover.
pub fn icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    size: f32,
    color: Color32,
    hover: Color32,
    tooltip: &str,
) -> Response {
    let edge = size + 12.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(edge), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tooltip)
    });
    if ui.is_rect_visible(rect) {
        let lift = crate::ui::motion::toggle(
            ui.ctx(),
            response.id.with("lift"),
            response.hovered() || response.has_focus(),
            crate::ui::motion::FEEDBACK,
        );
        let tint = color.lerp_to_gamma(hover, lift);
        // Pressing sinks the icon at once; letting go eases it back.
        let scale = if response.is_pointer_button_down_on() {
            crate::ui::motion::snap(ui.ctx(), response.id.with("press"), 0.9);
            0.9
        } else {
            crate::ui::motion::value(
                ui.ctx(),
                response.id.with("press"),
                1.0,
                crate::ui::motion::FEEDBACK,
            )
        };
        paint_icon(ui, icon, rect, size * scale, tint);
    }
    focus_ring(ui, &response);
    if tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(tooltip)
    }
}

/// Horizontal offset that optically centers play triangles.
///
/// Lucide includes a 1/24-width shift; a measured 3% shift centers the icon at
/// Spotifast's sizes. Use this everywhere instead of per-call adjustments.
pub fn play_glyph_offset(icon: Icon, icon_size: f32) -> Vec2 {
    if matches!(icon, Icon::PlayFilled | Icon::Play) {
        Vec2::new(icon_size * (0.03 - 1.0 / 24.0), 0.0)
    } else {
        Vec2::ZERO
    }
}

/// The flat ridge glyph, tinted by the current palette like T3 Pretty's mark.
pub fn logo(ui: &egui::Ui, center: egui::Pos2, diameter: f32, palette: &Palette) {
    let ppp = ui.ctx().pixels_per_point();
    // A little raster oversampling keeps the bare glyph crisp on Retina.
    let pixels = (diameter * ppp).round() as usize + 2;
    let id = egui::Id::new(("spotiurge-logo", pixels));
    let texture = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(id))
        .unwrap_or_else(|| {
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [pixels, pixels],
                &crate::util::glyph_rgba(pixels),
            );
            let texture =
                ui.ctx()
                    .load_texture("spotiurge-logo", image, egui::TextureOptions::LINEAR);
            ui.ctx()
                .data_mut(|data| data.insert_temp(id, texture.clone()));
            texture
        });
    let side = pixels as f32 / ppp;
    let rect = egui::Rect::from_center_size(center, Vec2::splat(side));
    ui.painter().image(
        texture.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        palette.accent,
    );
}

pub fn circle_button(
    ui: &mut egui::Ui,
    icon: Icon,
    diameter: f32,
    fill: Color32,
    fill_hover: Color32,
    icon_color: Color32,
    tooltip: &str,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tooltip)
    });
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered() || response.has_focus();
        let lift = crate::ui::motion::toggle(
            ui.ctx(),
            response.id.with("lift"),
            hovered,
            crate::ui::motion::FEEDBACK,
        );
        // Pressed, the disc sinks at once; released, it eases back up.
        let press = if response.is_pointer_button_down_on() {
            crate::ui::motion::snap(ui.ctx(), response.id.with("press"), 0.94);
            0.94
        } else {
            crate::ui::motion::value(
                ui.ctx(),
                response.id.with("press"),
                1.0,
                crate::ui::motion::FEEDBACK,
            )
        };
        let radius = diameter / 2.0 * (1.0 + 0.05 * lift) * press;
        let fill = fill.lerp_to_gamma(fill_hover, lift);
        ui.painter().circle_filled(rect.center(), radius, fill);
        let icon_size = diameter * 0.46 * press;
        let offset = play_glyph_offset(icon, icon_size);
        let icon_rect =
            egui::Rect::from_center_size(rect.center() + offset, Vec2::splat(icon_size));
        icon.image(icon_color, icon_size).paint_at(ui, icon_rect);
    }
    focus_ring(ui, &response);
    if tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(tooltip)
    }
}

/// A disc the size of a [`circle_button`] whose icon is replaced by a
/// spinner: the pressed play button itself shows that Spotify is reacting.
pub fn circle_spinner(
    ui: &mut egui::Ui,
    diameter: f32,
    fill: Color32,
    spin: Color32,
    tooltip: &str,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::hover());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::ProgressIndicator,
            ui.is_enabled(),
            tooltip,
        )
    });
    if ui.is_rect_visible(rect) {
        ui.painter()
            .circle_filled(rect.center(), diameter / 2.0, fill);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ));
        spinner(&mut child, diameter * 0.55, spin);
    }
    if tooltip.is_empty() {
        response
    } else {
        response.on_hover_text(tooltip)
    }
}

/// A pill-shaped text button: filled for the primary action, outlined otherwise.
pub fn pill_button(ui: &mut egui::Ui, palette: &Palette, label: &str, primary: bool) -> Response {
    let font = semibold(13.0);
    let enabled = ui.is_enabled();
    let color = if !enabled {
        palette.disabled_text()
    } else if primary {
        palette.on_accent
    } else {
        palette.text
    };
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, color);
    let padding = Vec2::new(18.0, 8.0);
    let size = galley.size() + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    let mut painter = ui.painter().clone();
    if !enabled {
        painter.set_opacity(1.0);
    }
    if ui.is_rect_visible(rect) {
        let lift = crate::ui::motion::toggle(
            ui.ctx(),
            response.id.with("lift"),
            enabled && response.hovered(),
            crate::ui::motion::FEEDBACK,
        );
        let radius = rect.height() / 2.0;
        if primary {
            let fill = if enabled {
                palette.accent.lerp_to_gamma(palette.accent_hover, lift)
            } else if palette.dark {
                Color32::WHITE.gamma_multiply(0.08)
            } else {
                palette.surface_active
            };
            painter.rect_filled(rect, radius, fill);
        } else {
            // A clear glass key: a faint fill that brightens under the
            // pointer, inside a rim that lights with it.
            painter.rect_filled(
                rect,
                radius,
                crate::ui::material::hover_fill(palette).gamma_multiply(0.6 + 0.9 * lift),
            );
            let stroke_color = palette.dim.lerp_to_gamma(palette.text, lift);
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, stroke_color.gamma_multiply(0.7 + 0.3 * lift)),
                egui::StrokeKind::Inside,
            );
        }
        let pos = rect.center() - galley.size() / 2.0;
        painter.galley(pos, galley, color);
    }
    focus_ring(ui, &response);
    response
}

/// A muted button with an icon and label, for row and header actions.
pub fn soft_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: Option<Icon>,
    label: &str,
    active: bool,
) -> Response {
    soft_button_inner(ui, palette, icon, label, active, false).0
}

/// A [`soft_button`] whose icon turns into a cross on hover, allowing the
/// entry to be dismissed. The returned flag reports clicks on that cross.
pub fn soft_button_dismiss(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: Icon,
    label: &str,
) -> (Response, bool) {
    soft_button_inner(ui, palette, Some(icon), label, false, true)
}

/// The width `soft_button` gives a button without an icon, for laying out
/// a row of them before drawing it.
pub fn soft_button_width(ui: &egui::Ui, label: &str) -> f32 {
    let galley = crate::bidi::layout_line(ui.painter(), label, medium(13.0), Color32::WHITE);
    galley.size().x + 24.0
}

fn soft_button_inner(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: Option<Icon>,
    label: &str,
    active: bool,
    dismissible: bool,
) -> (Response, bool) {
    let font = medium(13.0);
    let color = if active { palette.window } else { palette.text };
    let galley = crate::bidi::layout_line(ui.painter(), label, font, color);
    let icon_size = 15.0;
    let icon_width = if icon.is_some() { icon_size + 6.0 } else { 0.0 };
    let padding = Vec2::new(12.0, 7.0);
    let size = Vec2::new(galley.size().x + icon_width, galley.size().y) + padding * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), label)
    });
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + padding.x + icon_size / 2.0, rect.center().y),
        Vec2::splat(icon_size),
    );
    // Claimed after the button, so the cross sits on top and keeps its own click.
    let dismiss = (dismissible && icon.is_some()).then(|| {
        let dismiss = ui.interact(
            icon_rect.expand(3.0),
            response.id.with("dismiss"),
            Sense::click(),
        );
        dismiss.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled(),
                format!("Remove {label}"),
            )
        });
        dismiss
    });
    let dismissed = dismiss.as_ref().is_some_and(Response::clicked);
    let over_dismiss = dismiss
        .as_ref()
        .is_some_and(|dismiss| dismiss.hovered() || dismiss.has_focus());
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered() || over_dismiss;
        let lift = crate::ui::motion::toggle(
            ui.ctx(),
            response.id.with("lift"),
            hovered,
            crate::ui::motion::FEEDBACK,
        );
        let fill = if active {
            palette.text
        } else {
            palette.surface.lerp_to_gamma(palette.surface_hover, lift)
        };
        ui.painter().rect_filled(rect, rect.height() / 2.0, fill);
        let mut x = rect.left() + padding.x;
        if let Some(icon) = icon {
            let icon = if dismiss.is_some() && hovered {
                Icon::X
            } else {
                icon
            };
            icon.image(color, icon_size).paint_at(ui, icon_rect);
            x += icon_width;
        }
        let pos = egui::pos2(x, rect.center().y - galley.size().y / 2.0);
        ui.painter().galley(pos, galley, color);
    }
    focus_ring(ui, &response);
    if let Some(dismiss) = dismiss {
        focus_ring(ui, &dismiss);
    }
    (response, dismissed)
}

/// A row of mutually exclusive choices drawn as soft buttons, whose
/// selected fill glides from the old choice to the new one. Returns the
/// choice clicked this frame. Wraps onto more lines when `wrap` is set.
pub fn choice_chips<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    palette: &Palette,
    id: egui::Id,
    choices: &[(T, &str)],
    selected: T,
    wrap: bool,
) -> Option<T> {
    let font = medium(13.0);
    let padding = Vec2::new(12.0, 7.0);
    let mut chips = Vec::with_capacity(choices.len());
    let mut add = |ui: &mut egui::Ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(6.0);
        for &(value, label) in choices {
            let galley = crate::bidi::layout_line(ui.painter(), label, font.clone(), palette.text);
            let (rect, response) =
                ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    value == selected,
                    label,
                )
            });
            chips.push((value, label, rect, response));
        }
    };
    if wrap {
        ui.horizontal_wrapped(&mut add);
    } else {
        ui.horizontal(&mut add);
    }
    // The selected fill's rect glides; the chips it passes over turn their
    // text to match while it covers them. A value none of the choices
    // holds (a custom curve, say) leaves every chip unselected.
    let pill = chips
        .iter()
        .position(|chip| chip.0 == selected)
        .map(|selected_index| {
            let target = chips[selected_index].2;
            let key = id.with("selected");
            // The same choice somewhere else means the layout moved it (a
            // wrap, a resize): the fill follows at once instead of gliding.
            let relaid = ui
                .ctx()
                .data(|data| data.get_temp::<(usize, egui::Rect)>(key))
                .is_some_and(|(index, rect)| index == selected_index && rect != target);
            ui.ctx()
                .data_mut(|data| data.insert_temp(key, (selected_index, target)));
            let axes = [
                ("left", target.left()),
                ("top", target.top()),
                ("right", target.right()),
                ("bottom", target.bottom()),
            ];
            let [left, top, right, bottom] = axes.map(|(axis, value)| {
                if relaid {
                    crate::ui::motion::snap(ui.ctx(), id.with(axis), value);
                }
                crate::ui::motion::value(ui.ctx(), id.with(axis), value, crate::ui::motion::STATE)
            });
            egui::Rect::from_min_max(egui::pos2(left, top), egui::pos2(right, bottom))
        });
    let ctx = ui.ctx().clone();
    let painter = ui.painter();
    let mut clicked = None;
    for (value, _, rect, response) in &chips {
        let lift = crate::ui::motion::toggle(
            &ctx,
            response.id.with("lift"),
            response.hovered(),
            crate::ui::motion::FEEDBACK,
        );
        painter.rect_filled(
            *rect,
            rect.height() / 2.0,
            if palette.dark {
                Color32::WHITE.gamma_multiply(0.08 + 0.04 * lift)
            } else {
                palette.surface.lerp_to_gamma(palette.surface_hover, lift)
            },
        );
        if response.clicked() && *value != selected {
            clicked = Some(*value);
        }
    }
    if let Some(pill) = pill {
        painter.rect_filled(pill, pill.height() / 2.0, palette.text);
    }
    for (_, label, rect, response) in &chips {
        let covered = pill.map_or(0.0, |pill| {
            let overlap = rect.intersect(pill);
            if overlap.is_positive() {
                (overlap.width() / rect.width()).clamp(0.0, 1.0)
            } else {
                0.0
            }
        });
        let color = palette.text.lerp_to_gamma(palette.window, covered);
        let galley = crate::bidi::layout_line(painter, *label, font.clone(), color);
        painter.galley(rect.center() - galley.size() / 2.0, galley, color);
        focus_ring(ui, response);
    }
    clicked
}

/// An animated busy indicator paced independently of the graphics driver.
pub fn spinner(ui: &mut egui::Ui, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
        let radius = size / 2.0 - 2.0;
        let start = ui.input(|input| input.time) * std::f64::consts::TAU * 1.2;
        let sweep = 250_f64.to_radians();
        let points = (0..20)
            .map(|index| {
                let angle = start + sweep * f64::from(index) / 19.0;
                let (sin, cos) = angle.sin_cos();
                rect.center() + radius * egui::vec2(cos as f32, sin as f32)
            })
            .collect();
        ui.painter()
            .add(egui::Shape::line(points, Stroke::new(2.0, color)));
    }
    response
}

/// Truncated single-line text in a given font and colour.
pub fn text(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    font: egui::FontId,
    color: Color32,
) -> Response {
    let text = text.into();
    if crate::bidi::is_rtl(&text) {
        // Laid out here so a cut lands at the reading end, on the left.
        let galley = crate::bidi::layout(
            ui.painter(),
            &text,
            font,
            color,
            ui.available_width(),
            1,
            Some(crate::bidi::ELLIPSIS),
        );
        return ui.add(egui::Label::new(galley).selectable(false));
    }
    ui.add(
        egui::Label::new(egui::RichText::new(text).font(font).color(color))
            .truncate()
            .selectable(false),
    )
}

/// Single-line text that acts like a link: underlines on hover, clickable.
pub fn link(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    font: egui::FontId,
    color: Color32,
) -> Response {
    let text = text.into();
    let response = if crate::bidi::is_rtl(&text) {
        let galley = crate::bidi::layout(
            ui.painter(),
            &text,
            font,
            color,
            ui.available_width(),
            1,
            Some(crate::bidi::ELLIPSIS),
        );
        ui.add(
            egui::Label::new(galley)
                .selectable(false)
                .sense(Sense::click()),
        )
    } else {
        ui.add(
            egui::Label::new(egui::RichText::new(text.clone()).font(font).color(color))
                .truncate()
                .selectable(false)
                .sense(Sense::click()),
        )
    };
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Link, ui.is_enabled(), &text));
    focus_ring(ui, &response);
    if response.hovered() {
        let rect = response.rect;
        ui.painter()
            .hline(rect.x_range(), rect.bottom() - 1.0, Stroke::new(1.0, color));
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub fn section_title(ui: &mut egui::Ui, palette: &Palette, label: &str) -> Response {
    text(ui, label, bold(17.0), palette.text)
}

pub fn subtle(ui: &mut egui::Ui, palette: &Palette, label: &str) -> Response {
    text(ui, label, regular(13.0), palette.secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The header uploads the shared alpha mask; the palette supplies its colour.
    #[test]
    fn the_logo_uses_the_shared_ridge_mask() {
        // #given the logo drawn 40 points wide at twice the pixel density
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(2.0);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            logo(ui, egui::pos2(40.0, 40.0), 40.0, &Palette::dark());
        });

        // #then the frame uploads the icon rasterised at that size
        let uploaded = output
            .textures_delta
            .set
            .values()
            .flat_map(|deltas| deltas.iter())
            .find_map(|delta| match &delta.image {
                egui::ImageData::Color(image) if image.size == [82, 82] => Some(image.clone()),
                _ => None,
            })
            .expect("the logo's texture");
        let icon = crate::util::glyph_rgba(82);
        for (x, y) in [(41, 10), (46, 41), (41, 4)] {
            let expected = &icon[(y * 82 + x) * 4..(y * 82 + x) * 4 + 4];
            let got = uploaded.pixels[y * 82 + x].to_srgba_unmultiplied();
            assert_eq!(&got[..], expected, "pixel {x},{y}");
        }
        output.textures_delta.clear();
    }

    /// Custom pale accents still deepen to readable small text.
    #[test]
    fn custom_accent_text_keeps_its_contrast_guard() {
        for mut palette in [Palette::dark(), Palette::light()] {
            palette.accent = Color32::from_rgb(170, 190, 160);
            let toward = if palette.dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            let ground = palette
                .surface_active
                .lerp_to_gamma(toward, if palette.dark { 0.1 } else { 0.2 });
            assert!(contrast(palette.accent_text(), ground) >= 4.5);
        }
    }

    /// Palette files name the sixteen colours every app shares, and only
    /// those: a typo is an invalid file, not an ignored colour.
    #[test]
    fn palette_files_set_every_base_colour() {
        use fastframe_theme::Palette as _;
        for name in fastframe_theme::BASE_COLORS {
            let mut palette = Palette::dark();
            assert!(palette.set(name, Color32::from_rgb(1, 2, 3)), "{name}");
            assert_ne!(palette, Palette::dark(), "{name}");
        }
        assert!(!Palette::dark().set("typo", Color32::RED));
        let light: Palette =
            fastframe_theme::parse_palette(r##"{"base":"light","colors":{"accent":"#8c3fa5"}}"##)
                .unwrap();
        assert!(!light.dark);
        assert_eq!(light.accent, Color32::from_rgb(140, 63, 165));
        assert_eq!(light.window, Palette::light().window);
    }

    /// Compared line by line: a Windows checkout may turn the files' line
    /// endings into CRLF, which no Linux package ships.
    #[test]
    fn the_shipped_omarchy_files_are_the_shared_ones() {
        let lines = |text: &str| text.replace("\r\n", "\n");
        assert_eq!(
            lines(include_str!("../contrib/omarchy/spotifast-theme")),
            lines(&fastframe_theme::omarchy::hook_script("spotifast"))
        );
        assert_eq!(
            lines(include_str!("../contrib/omarchy/spotifast.json.tpl")),
            lines(fastframe_theme::omarchy::BASE_TEMPLATE)
        );
    }

    #[test]
    fn every_catalog_problem_has_a_sentence() {
        let catalog = Catalog::default();
        assert_eq!(catalog_detail(&catalog, Locale::English, None), "");
        assert!(
            catalog_detail(&catalog, Locale::English, Some("gone.json")).contains("last usable")
        );
    }

    /// A palette replaces egui's visuals, which must not take back the
    /// desktop's text rendering: on Linux, linear coverage in both themes.
    #[test]
    fn palettes_keep_the_desktops_text_rendering() {
        for palette in [Palette::dark(), Palette::light()] {
            let ctx = egui::Context::default();
            apply(&ctx, &palette);
            let options = ctx.global_style().visuals.text_options;
            assert_eq!(
                options.color_transfer_function,
                text_rendering().color_transfer_function(palette.dark),
                "dark: {}",
                palette.dark
            );
            if cfg!(target_os = "linux") {
                assert_eq!(
                    options.color_transfer_function,
                    egui::epaint::FontColorTransferFunction::Off
                );
            }
            assert!(options.subpixel_binning);
        }
    }

    /// Native desktop apps keep the arrow over buttons and switch to the
    /// hand only over links (#508).
    #[test]
    fn only_links_show_the_hand_cursor() {
        let ctx = egui::Context::default();
        install(&ctx);
        let palette = Palette::dark();
        let draw = |ctx: &egui::Context, pointer: Option<egui::Pos2>| {
            let mut rects = (egui::Rect::NOTHING, egui::Rect::NOTHING);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 200.0),
                )),
                events: pointer
                    .map(|at| vec![egui::Event::PointerMoved(at)])
                    .unwrap_or_default(),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                rects.0 = pill_button(ui, &palette, "Play", true).rect;
                rects.1 = link(ui, "Bonobo", regular(14.0), palette.text).rect;
            });
            output.textures_delta.clear();
            (rects, output.platform_output.cursor_icon)
        };
        draw(&ctx, None);
        let ((button, link_rect), _) = draw(&ctx, None);
        let (_, over_button) = draw(&ctx, Some(button.center()));
        assert_eq!(over_button, egui::CursorIcon::Default);
        let (_, over_link) = draw(&ctx, Some(link_rect.center()));
        assert_eq!(over_link, egui::CursorIcon::PointingHand);
    }

    /// A click on another choice reports it at once, while the selected
    /// fill still sets off from the old choice, so the change reads as a
    /// glide rather than a jump.
    #[test]
    fn choice_chips_report_the_click_and_glide_the_fill() {
        let ctx = egui::Context::default();
        install(&ctx);
        let palette = Palette::dark();
        let id = egui::Id::new("chips");
        let frame = |time: f64, selected: u8, events: Vec<egui::Event>| {
            let mut picked = None;
            let mut fill = None;
            let mut output = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| {
                    picked = choice_chips(
                        ui,
                        &palette,
                        id,
                        &[(0, "Familiar"), (1, "Bold")],
                        selected,
                        false,
                    );
                },
            );
            output.textures_delta.clear();
            for shape in &output.shapes {
                if let egui::Shape::Rect(rect) = &shape.shape
                    && rect.fill == palette.text
                {
                    fill = Some(rect.rect);
                }
            }
            (picked, fill)
        };
        let (_, first) = frame(0.0, 0, vec![]);
        let first = first.expect("the selected fill");
        let second_chip = first
            .translate(egui::vec2(first.width() + 20.0, 0.0))
            .center();
        frame(0.1, 0, vec![egui::Event::PointerMoved(second_chip)]);
        let click = |pressed| egui::Event::PointerButton {
            pos: second_chip,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let (picked, _) = frame(0.2, 0, vec![click(true), click(false)]);
        assert_eq!(picked, Some(1));
        let (_, start) = frame(1.0, 1, vec![]);
        assert_eq!(
            start.unwrap().left(),
            first.left(),
            "sets off from the old chip"
        );
        let (_, middle) = frame(1.1, 1, vec![]);
        assert!(middle.unwrap().left() > first.left());
        let (_, end) = frame(2.0, 1, vec![]);
        assert!(end.unwrap().left() > first.right(), "lands on the new chip");
    }

    #[test]
    fn a_local_palette_keeps_spotifasts_widget_style_local() {
        let ctx = egui::Context::default();
        apply(&ctx, &Palette::light());
        let dark = Palette::dark();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            apply_local(ui, &dark);
            let style = ui.style();
            assert!(style.visuals.dark_mode);
            assert_eq!(
                style.visuals.widgets.inactive.corner_radius,
                CornerRadius::same(RADIUS_SMALL + 2)
            );
            assert_eq!(
                style.visuals.selection.stroke,
                Stroke::new(1.0, dark.accent)
            );
            assert_eq!(style.spacing.button_padding, Vec2::new(12.0, 6.0));
        });
        output.textures_delta.clear();
        assert!(!ctx.global_style().visuals.dark_mode);
    }

    #[test]
    fn fonts_install_and_layout_emojis() {
        let ctx = egui::Context::default();
        install(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let galley = ui.painter().layout_no_wrap(
                "Rosewood 🔥 Otomo 🎵 ❤️ 🚀".to_string(),
                regular(14.0),
                Color32::WHITE,
            );
            assert!(galley.rows[0].glyphs.len() >= 5);
        });
        output.textures_delta.clear();
    }

    /// The monochrome emoji face comes right after Inter at every weight
    /// and in the monospace family, ahead of egui's own emoji pair.
    #[test]
    fn the_emoji_face_follows_inter_everywhere() {
        let ctx = egui::Context::default();
        install(&ctx);
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        let fonts = ctx.fonts(|fonts| fonts.definitions().clone());
        for weight in fastframe_fonts::Weight::ALL {
            let family = &fonts.families[&weight.family()];
            assert_eq!(family[..2], [weight.name(), "noto_emoji"], "{weight:?}");
        }
        assert_eq!(
            fonts.families[&egui::FontFamily::Monospace][1],
            "noto_emoji"
        );
    }

    #[test]
    fn fonts_install_and_layout_mixed_cjk() {
        let ctx = egui::Context::default();
        install(&ctx);
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let galley = ui.painter().layout_no_wrap(
                "Track 87: 恋におちて -Fall in love- (Live)".to_string(),
                regular(14.0),
                Color32::WHITE,
            );
            assert!(!galley.rows.is_empty());
            assert!(galley.rows[0].glyphs.len() >= 10);
        });
        output.textures_delta.clear();
    }
}
