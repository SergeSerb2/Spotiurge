//! The personal discovery workspace, using the existing native player actions.
//!
//! One restrained pane opens on playable picks: a compact header with play
//! and refresh, an exploration strip, one honest catalogue summary and 56-point
//! rows. Taste editing, unmatched suggestions and AI history stay out of the way
//! until asked for.

use super::widgets::{self, TrackRow};
use crate::app::App;
use crate::discovery::{
    CatalogueOutcome, Exploration, MAX_PROMPT_BYTES, Rating, RecommendationErrorKind, Value,
    limit_prompt,
};
use crate::i18n::{gettext, ngettext};
use crate::model::{Action, RowContext};
use crate::theme::{self, Icon, Palette};
use egui::{Align, Color32, CornerRadius, Frame, Layout, Margin, Sense, Stroke, Vec2, vec2};
use std::sync::Arc;

/// Width of the main content above which reasons get their own column.
const WIDE: f32 = 920.0;
const FEEDBACK: f32 = 28.0;

struct PromptBuffer<'a>(&'a mut String);

impl egui::TextBuffer for PromptBuffer<'_> {
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.0
    }
    fn insert_text(&mut self, text: &str, index: egui::text::CharIndex) -> usize {
        let mut end = text
            .len()
            .min(MAX_PROMPT_BYTES.saturating_sub(self.0.len()));
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        egui::TextBuffer::insert_text(self.0, &text[..end], index)
    }
    fn delete_char_range(&mut self, range: std::ops::Range<egui::text::CharIndex>) {
        egui::TextBuffer::delete_char_range(self.0, range);
    }
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<PromptBuffer<'static>>()
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    // Cover-derived light when the theme follows the art, the accent otherwise.
    let tint = app.now_playing_tint().unwrap_or(palette.accent);
    let fill = palette
        .surface
        .gamma_multiply(if palette.dark { 0.72 } else { 0.80 });
    // A static translucent material over the native background: no blur pass
    // or idle repaint loop. The glow is reserved inside the frame, so it sits
    // above the fill and below the content.
    let framed = Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(20, 18))
        .show(ui, |ui| {
            let glow = ui.painter().add(egui::Shape::Noop);
            ui.set_width(ui.available_width());
            pane(app, ui);
            glow
        });
    ui.painter().set(
        framed.inner,
        glow_mesh(framed.response.rect, tint, palette.dark),
    );
    ui.add_space(24.0);
}

/// A soft light rising from the upper left. Every edge vertex is transparent,
/// so the glow never spills past the pane's rounded corners.
fn glow_mesh(rect: egui::Rect, tint: Color32, dark: bool) -> egui::Shape {
    let mut mesh = egui::Mesh::default();
    let center = rect.lerp_inside(vec2(0.22, 0.18));
    mesh.colored_vertex(center, tint.gamma_multiply(if dark { 0.16 } else { 0.12 }));
    let ring = [
        rect.left_top(),
        rect.center_top(),
        rect.right_top(),
        rect.right_center(),
        rect.right_bottom(),
        rect.center_bottom(),
        rect.left_bottom(),
        rect.left_center(),
    ];
    for point in ring {
        mesh.colored_vertex(point, Color32::TRANSPARENT);
    }
    for index in 0..ring.len() as u32 {
        mesh.add_triangle(0, 1 + index, 1 + (index + 1) % ring.len() as u32);
    }
    egui::Shape::mesh(mesh)
}

#[derive(Default)]
struct Counts {
    ready: usize,
    missing: usize,
    unchecked: usize,
}

fn pane(app: &mut App, ui: &mut egui::Ui) {
    let mut counts = Counts::default();
    for pick in &app.discovery.picks {
        match (&pick.track, pick.checked) {
            (Some(_), _) => counts.ready += 1,
            (None, true) => counts.missing += 1,
            (None, false) => counts.unchecked += 1,
        }
    }
    let can_play = app.discovery_playback_available() && counts.ready > 0;

    header(app, ui, can_play);
    ui.add_space(12.0);
    exploration(app, ui);
    ui.add_space(14.0);

    let onboarding = app.discovery.ready && !app.discovery.replica.document.has_inputs();
    if app.discovery.editing_taste || onboarding {
        taste_editor(app, ui, onboarding);
        ui.add_space(14.0);
    }

    summary(app, ui, &counts);
    ui.add_space(10.0);

    if counts.ready > 0 {
        playable_rows(app, ui);
    }
    if counts.missing + counts.unchecked > 0 {
        ui.add_space(8.0);
        unmatched(app, ui, counts.missing + counts.unchecked);
    }
    saved_mixes(app, ui);
    if app.discovery.show_history {
        history(app, ui);
    }
}

fn header(app: &mut App, ui: &mut egui::Ui, can_play: bool) {
    let palette = app.palette;
    let locale = app.locale;
    let offline = app.offline;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            theme::text(
                ui,
                gettext(locale, "For you"),
                theme::bold(26.0),
                palette.text,
            );
            let age = if offline {
                gettext(locale, "Demo picks").into_owned()
            } else if let Some(when) = app
                .settings
                .discovery
                .refreshed_at
                .and_then(|seconds| jiff::Timestamp::from_second(seconds as i64).ok())
            {
                // Translators: {when} is a relative time such as "5 minutes ago" or a date.
                gettext(locale, "Updated {when}").replace(
                    "{when}",
                    &crate::util::format_relative_date(
                        locale,
                        &when.to_string(),
                        jiff::Timestamp::now(),
                    ),
                )
            } else if app.discovery.picks.is_empty() {
                String::new()
            } else {
                gettext(locale, "Saved picks").into_owned()
            };
            if !age.is_empty() {
                theme::text(ui, age, theme::regular(13.0), palette.secondary);
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            more_menu(app, ui);

            let refresh = gettext(locale, "Find new picks");
            if app.discovery.busy {
                theme::circle_spinner(
                    ui,
                    30.0,
                    Color32::TRANSPARENT,
                    palette.secondary,
                    &gettext(locale, "Finding music…"),
                );
            } else {
                let enabled = app.discovery.ready && !offline && app.is_connected();
                ui.add_enabled_ui(enabled, |ui| {
                    if theme::icon_button(
                        ui,
                        Icon::Refresh,
                        18.0,
                        palette.secondary,
                        palette.text,
                        &refresh,
                    )
                    .on_disabled_hover_text(demo_disabled(app))
                    .clicked()
                    {
                        app.actions.push(Action::DiscoveryRecommend);
                    }
                });
            }

            let unavailable = play_unavailable(app);
            ui.add_enabled_ui(can_play, |ui| {
                if theme::icon_button(
                    ui,
                    Icon::Shuffle,
                    18.0,
                    palette.secondary,
                    palette.text,
                    &gettext(locale, "Shuffle"),
                )
                .on_disabled_hover_text(unavailable.as_str())
                .clicked()
                {
                    app.actions.push(Action::DiscoveryPlayAll { shuffle: true });
                }
                ui.add_space(4.0);
                if theme::circle_button(
                    ui,
                    Icon::PlayFilled,
                    44.0,
                    palette.accent,
                    palette.accent_hover,
                    palette.on_accent,
                    &gettext(locale, "Play all"),
                )
                .on_disabled_hover_text(unavailable.as_str())
                .clicked()
                {
                    app.actions
                        .push(Action::DiscoveryPlayAll { shuffle: false });
                }
            });
        });
    });
}

fn play_unavailable(app: &App) -> String {
    if app.discovery_playback_available() {
        gettext(app.locale, "No playable picks yet.").into_owned()
    } else {
        gettext(
            app.locale,
            "Choose this computer to play discovery tracks and mixes.",
        )
        .into_owned()
    }
}

fn demo_disabled(app: &App) -> String {
    if app.offline {
        gettext(
            app.locale,
            "Demo discoveries. Feedback, recommendations and cloud requests are disabled.",
        )
        .into_owned()
    } else if !app.discovery.ready {
        gettext(app.locale, "Opening your personal discovery workspace…").into_owned()
    } else {
        gettext(app.locale, "Sign in to Spotify to find new picks.").into_owned()
    }
}

fn more_menu(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let more = theme::icon_button(
        ui,
        Icon::Ellipsis,
        18.0,
        palette.secondary,
        palette.text,
        &gettext(locale, "More discovery options"),
    );
    #[cfg(feature = "demo")]
    if app.offline && app.discovery.status == "DEMO_MENU" {
        egui::Popup::open_id(ui.ctx(), egui::Popup::default_response_id(&more));
    }
    egui::Popup::menu(&more)
        .frame(widgets::menu_frame(&palette))
        .show(|ui| {
            ui.set_min_width(220.0);
            let writable = app.discovery.ready && !app.offline;
            let has_ready = app.discovery.picks.iter().any(|pick| pick.track.is_some());
            if widgets::menu_item_enabled(
                ui,
                &palette,
                Some(Icon::ListPlus),
                &gettext(locale, "Save this mix"),
                writable && has_ready,
            ) {
                app.actions.push(Action::DiscoverySaveMix);
            }
            let sync = if app.discovery.syncing {
                gettext(locale, "Syncing…")
            } else {
                gettext(locale, "Sync my devices")
            };
            if widgets::menu_item_enabled(
                ui,
                &palette,
                Some(Icon::Refresh),
                &sync,
                writable && !app.discovery.syncing,
            ) {
                app.actions.push(Action::DiscoverySync);
            }
            ui.separator();
            if widgets::menu_item_enabled(
                ui,
                &palette,
                Some(Icon::SquarePen),
                &gettext(locale, "Tune taste"),
                app.discovery.ready,
            ) {
                app.actions.push(Action::DiscoveryEditTaste(true));
            }
            let history = if app.discovery.show_history {
                gettext(locale, "Hide AI history")
            } else {
                gettext(locale, "Show AI history")
            };
            if widgets::menu_item(ui, &palette, Some(Icon::Clock), &history) {
                app.actions.push(Action::DiscoveryShowHistory);
            }
            let automatic = app.settings.discovery.automatic;
            let label = if automatic {
                gettext(locale, "Turn off automatic picks")
            } else {
                gettext(locale, "Turn on automatic picks")
            };
            if widgets::menu_item(ui, &palette, Some(Icon::Sparkles), &label) {
                app.actions.push(Action::DiscoveryAutomatic(!automatic));
            }
        });
}

fn exploration(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let current = app.settings.discovery.exploration;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (choice, label, hint) in [
            (
                Exploration::Familiar,
                gettext(locale, "Familiar"),
                gettext(locale, "Stay close to what you already love"),
            ),
            (
                Exploration::Balanced,
                gettext(locale, "Balanced"),
                gettext(locale, "Mix favourites with new finds"),
            ),
            (
                Exploration::Adventurous,
                gettext(locale, "Adventurous"),
                gettext(locale, "Reach further from your usual music"),
            ),
        ] {
            let response = theme::soft_button(ui, &palette, None, &label, choice == current)
                .on_hover_text(hint.into_owned());
            if response.clicked() && choice != current {
                app.actions.push(Action::DiscoveryExploration(choice));
            }
        }
    });
}

fn taste_editor(app: &mut App, ui: &mut egui::Ui, onboarding: bool) {
    let palette = app.palette;
    let locale = app.locale;
    let heading = if onboarding {
        gettext(locale, "Tell Spotiurge what you like")
    } else {
        gettext(locale, "Your taste")
    };
    theme::text(ui, heading, theme::semibold(15.0), palette.text);
    ui.add_space(6.0);
    let mut draft = app.discovery.draft.clone();
    let input = ui.add_enabled(
        app.discovery.ready,
        egui::TextEdit::multiline(&mut PromptBuffer(&mut draft))
            .desired_width(f32::INFINITY)
            .desired_rows(3)
            .hint_text(gettext(
                locale,
                "Warm jazz, spacious electronics, a few surprises…",
            ))
            .font(theme::regular(15.0)),
    );
    #[cfg(feature = "demo")]
    if app.offline && app.discovery.status == "DEMO_FOCUS" {
        input.request_focus();
    }
    if input.changed() {
        limit_prompt(&mut draft);
        app.actions.push(Action::DiscoveryDraft(draft));
    }
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let can_save = app.discovery.ready
            && !app.offline
            && (!app.discovery.draft.trim().is_empty() || !onboarding);
        // Small text needs stronger contrast than the primary play icon.
        let mut text_palette = palette;
        if !palette.dark {
            text_palette.on_accent = egui::Color32::BLACK;
        }
        ui.add_enabled_ui(can_save, |ui| {
            if theme::pill_button(ui, &text_palette, &gettext(locale, "Save taste"), true)
                .on_disabled_hover_text(demo_disabled(app))
                .clicked()
            {
                app.actions.push(Action::DiscoverySaveTaste);
            }
        });
        if !onboarding
            && theme::pill_button(ui, &palette, &gettext(locale, "Cancel"), false).clicked()
        {
            app.actions.push(Action::DiscoveryCancelTaste);
        }
    });
}

/// One line about the catalogue, one about the last request. Never one per pick.
fn summary(app: &mut App, ui: &mut egui::Ui, counts: &Counts) {
    let palette = app.palette;
    let locale = app.locale;

    if let Some(kind) = app.discovery.last_error {
        let text = match kind {
            // Translators: {variable} is an environment variable name. Keep it as written.
            RecommendationErrorKind::Pairing => gettext(
                locale,
                "This computer is not paired with your private cloud. Quit Spotiurge, set {variable} in its launch environment and open it again.",
            )
            .replace("{variable}", "SPOTIURGE_CLOUD_TOKEN"),
            RecommendationErrorKind::Busy => gettext(
                locale,
                "Your picks are already being prepared. Try again in a moment.",
            )
            .into_owned(),
            RecommendationErrorKind::RateLimited => gettext(
                locale,
                "The AI is rate limited. Your picks are kept; try again later.",
            )
            .into_owned(),
            RecommendationErrorKind::Unavailable => gettext(
                locale,
                "Recommendations are unavailable right now. Your picks are kept.",
            )
            .into_owned(),
            RecommendationErrorKind::InvalidResponse => gettext(
                locale,
                "The AI sent an answer Spotiurge could not use. Your picks are kept.",
            )
            .into_owned(),
        };
        let detail = app.discovery.status.clone();
        let response = notice(ui, &palette, Icon::CircleAlert, palette.warning, &text);
        if !detail.is_empty() && detail != text {
            response.on_hover_text(detail);
        }
        ui.add_space(4.0);
    }

    if !app.discovery.ready {
        notice(
            ui,
            &palette,
            Icon::Loader,
            palette.secondary,
            &gettext(locale, "Opening your personal discovery workspace…"),
        );
        return;
    }

    if counts.ready + counts.missing + counts.unchecked == 0 {
        if !app.discovery.busy && app.discovery.replica.document.has_inputs() {
            notice(
                ui,
                &palette,
                Icon::Sparkles,
                palette.secondary,
                &gettext(
                    locale,
                    "No picks yet. Refresh to find music for your taste.",
                ),
            );
        }
    } else {
        let mut parts = vec![
            // Translators: {count} is a number of discovery tracks that can play.
            ngettext(
                locale,
                "{count} ready to play",
                "{count} ready to play",
                counts.ready as u32,
            )
            .replace("{count}", &counts.ready.to_string()),
        ];
        if counts.missing > 0 {
            parts.push(
                // Translators: {count} is a number of AI suggestions Spotify has no match for.
                ngettext(
                    locale,
                    "{count} not found on Spotify",
                    "{count} not found on Spotify",
                    counts.missing as u32,
                )
                .replace("{count}", &counts.missing.to_string()),
            );
        }
        if counts.unchecked > 0 {
            parts.push(
                // Translators: {count} is a number of AI suggestions not yet looked up on Spotify.
                ngettext(
                    locale,
                    "{count} not checked yet",
                    "{count} not checked yet",
                    counts.unchecked as u32,
                )
                .replace("{count}", &counts.unchecked.to_string()),
            );
        }
        let mut line = parts.join(" · ");
        if counts.unchecked > 0 {
            let reason = match app.discovery.catalogue {
                CatalogueOutcome::Complete => None,
                CatalogueOutcome::TimedOut => {
                    Some(gettext(locale, "Spotify took too long to answer."))
                }
                CatalogueOutcome::RateLimited => {
                    Some(gettext(locale, "Spotify asked Spotiurge to slow down."))
                }
                CatalogueOutcome::QuotaExhausted => Some(gettext(
                    locale,
                    "Spotiurge has used its Spotify search allowance for now.",
                )),
                CatalogueOutcome::Unavailable => {
                    Some(gettext(locale, "Spotify search is unavailable right now."))
                }
                CatalogueOutcome::SignInNeeded => {
                    Some(gettext(locale, "Sign in to Spotify to check the rest."))
                }
            };
            if let Some(reason) = reason {
                line.push_str(". ");
                line.push_str(&reason);
            }
        }
        // Wraps to two lines at most, so the retry below stays reachable.
        let galley = crate::bidi::layout(
            ui.painter(),
            &line,
            theme::regular(13.0),
            palette.secondary,
            ui.available_width(),
            2,
            Some(crate::bidi::ELLIPSIS),
        );
        ui.add(egui::Label::new(galley).selectable(false));
        if counts.unchecked > 0 {
            ui.add_space(6.0);
            retry(app, ui);
        }
    }

    if !app.discovery_playback_available() && counts.ready > 0 {
        notice(
            ui,
            &palette,
            Icon::Laptop,
            palette.secondary,
            &gettext(
                locale,
                "Choose this computer to play discovery tracks and mixes.",
            ),
        );
    }

    let status = app.discovery.status.as_str();
    if app.discovery.last_error.is_none()
        && !app.discovery.busy
        && !status.is_empty()
        && !status.starts_with("DEMO_")
    {
        ui.add_space(2.0);
        theme::text(
            ui,
            status.to_owned(),
            theme::regular(12.5),
            palette.secondary,
        );
    }
    if app.offline {
        ui.add_space(2.0);
        notice(
            ui,
            &palette,
            Icon::Info,
            palette.secondary,
            &gettext(
                locale,
                "Demo discoveries. Feedback, recommendations and cloud requests are disabled.",
            ),
        );
    }
}

/// An icon and one line of secondary text.
fn notice(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: Icon,
    icon_color: Color32,
    text: &str,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        theme::icon(ui, icon, 14.0, icon_color);
        theme::text(ui, text, theme::regular(13.0), palette.secondary)
    })
    .inner
}

/// Look up only the suggestions the catalogue has not answered for yet.
fn retry(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let now = std::time::Instant::now();
    let waiting = app.discovery.retry_after.filter(|until| *until > now);
    if let Some(until) = waiting {
        // Re-enable the control when Spotify's wait has passed.
        ui.ctx().request_repaint_after(until - now);
    }
    let enabled = app.discovery.ready && !app.discovery.busy && !app.offline && waiting.is_none();
    ui.add_enabled_ui(enabled, |ui| {
        let disabled = if waiting.is_some() {
            gettext(locale, "Spotify asked to wait. Retry in a moment.").into_owned()
        } else if app.discovery.busy {
            gettext(locale, "Finding music…").into_owned()
        } else {
            demo_disabled(app)
        };
        if theme::soft_button(
            ui,
            &palette,
            Some(Icon::Refresh),
            &gettext(locale, "Check again"),
            false,
        )
        .on_disabled_hover_text(disabled)
        .clicked()
        {
            app.actions.push(Action::DiscoveryRetryMatches);
        }
    });
}

fn playable_rows(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let playback = app.discovery_playback_available();
    let can_rate = app.discovery.ready && !app.offline;
    // Rows borrow the app mutably, so draw from a copy of this short list.
    let picks = app.discovery.picks.clone();
    let uris: Arc<[String]> = picks
        .iter()
        .filter_map(|pick| pick.track.as_ref().map(|track| track.uri.clone()))
        .collect::<Vec<_>>()
        .into();
    let context = RowContext::Discovery(uris);
    let width = ui.available_width();
    let wide = width > WIDE;
    let feedback = FEEDBACK * 2.0 + 4.0;
    let reason_width = if wide {
        (width * 0.30).clamp(200.0, 360.0)
    } else {
        0.0
    };
    let gap = 12.0;
    let row_width = width - feedback - gap - if wide { reason_width + gap } else { 0.0 };

    let (love_label, less_label) = (
        gettext(locale, "More like this"),
        gettext(locale, "Less like this"),
    );
    let mut index = 0;
    for pick in &picks {
        let Some(track) = &pick.track else {
            continue;
        };
        let item = crate::api::models::PlayableItem::Track(track.clone());
        let reason = pick.suggestion.reason.as_str();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let row = ui
                .allocate_ui_with_layout(
                    vec2(row_width, theme::ROW_HEIGHT),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.add_enabled_ui(playback, |ui| {
                            widgets::track_row_response(
                                ui,
                                app,
                                TrackRow {
                                    index,
                                    number: None,
                                    item: &item,
                                    context: &context,
                                    show_cover: true,
                                    show_album: false,
                                    added_at: None,
                                    added_by: None,
                                    show_added_by: false,
                                    compact: false,
                                    thin: false,
                                    shift: 0.0,
                                    picked: false,
                                    picked_songs: &[],
                                },
                            )
                            .0
                        })
                        .inner
                    },
                )
                .inner;
            if !wide && !reason.is_empty() {
                row.on_hover_text(reason);
            }
            if wide {
                ui.allocate_ui_with_layout(
                    vec2(reason_width, theme::ROW_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let galley = crate::bidi::layout(
                            ui.painter(),
                            reason,
                            theme::regular(12.5),
                            palette.secondary,
                            reason_width,
                            1,
                            Some(crate::bidi::ELLIPSIS),
                        );
                        ui.add(egui::Label::new(galley).selectable(false))
                            .on_hover_text(reason);
                    },
                );
            }
            ui.allocate_ui_with_layout(
                vec2(feedback, theme::ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let current = app.discovery.replica.document.rating(&track.uri);
                    ui.add_enabled_ui(can_rate, |ui| {
                        for (icon, label, rating) in [
                            (Icon::ThumbsUp, &love_label, Rating::Love),
                            (Icon::ThumbsDown, &less_label, Rating::Less),
                        ] {
                            let selected = current == Some(rating);
                            if feedback_button(ui, &palette, icon, label, rating, selected)
                                .on_disabled_hover_text(demo_disabled(app))
                                .clicked()
                            {
                                app.actions.push(Action::DiscoveryRate {
                                    uri: track.uri.clone(),
                                    title: track.name.clone(),
                                    artist: track.artist_names(),
                                    // Choosing the selected rating again clears it.
                                    rating: (!selected).then_some(rating),
                                });
                            }
                        }
                    });
                },
            );
        });
        index += 1;
    }
}

/// A 28-point round toggle. Love fills with the accent and Less with the
/// neutral text colour, so neither reads as the Spotify heart beside it.
fn feedback_button(
    ui: &mut egui::Ui,
    palette: &Palette,
    icon: Icon,
    label: &str,
    rating: Rating,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(FEEDBACK), Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    if ui.is_rect_visible(rect) {
        let hovered = response.hovered() || response.has_focus();
        let (fill, color) = match (selected, rating) {
            (true, Rating::Love) => (palette.accent, palette.on_accent),
            (true, Rating::Less) => (palette.text, palette.window),
            (false, _) if hovered => (palette.surface_hover, palette.text),
            (false, _) => (Color32::TRANSPARENT, palette.secondary),
        };
        ui.painter()
            .circle_filled(rect.center(), FEEDBACK / 2.0, fill);
        theme::paint_icon(ui, icon, rect, 15.0, color);
    }
    theme::focus_ring(ui, &response);
    response.on_hover_text(label)
}

fn unmatched(app: &mut App, ui: &mut egui::Ui, count: usize) {
    let palette = app.palette;
    let locale = app.locale;
    // Translators: {count} is a number of AI suggestions that cannot play.
    let title = gettext(locale, "Couldn't play ({count})").replace("{count}", &count.to_string());
    #[cfg(feature = "demo")]
    let demo_open = app.offline && app.discovery.status == "DEMO_UNMATCHED";
    #[cfg(not(feature = "demo"))]
    let demo_open = false;
    egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .font(theme::medium(13.0))
            .color(palette.secondary),
    )
    .id_salt("discovery-unmatched")
    .default_open(demo_open)
    .show(ui, |ui| {
        let not_checked = gettext(locale, "Not checked on Spotify yet");
        let not_found = gettext(locale, "Not found on Spotify");
        let search = gettext(locale, "Search Spotify");
        for (index, pick) in app.discovery.picks.iter().enumerate() {
            if pick.track.is_some() {
                continue;
            }
            let suggestion = &pick.suggestion;
            ui.push_id(("unmatched", index), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let (icon, hint) = if pick.checked {
                        (Icon::CircleX, &not_found)
                    } else {
                        (Icon::Clock, &not_checked)
                    };
                    theme::icon(ui, icon, 14.0, palette.dim).on_hover_text(hint.to_string());
                    let query = format!("{} {}", suggestion.title, suggestion.artist);
                    if theme::icon_button(
                        ui,
                        Icon::Search,
                        14.0,
                        palette.secondary,
                        palette.text,
                        &search,
                    )
                    .clicked()
                    {
                        app.actions.push(Action::Search(query));
                    }
                    theme::text(
                        ui,
                        format!("{} · {}", suggestion.title, suggestion.artist),
                        theme::regular(13.0),
                        palette.text,
                    );
                });
            });
        }
    });
}

fn saved_mixes(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    let playable = app.discovery_playback_available();
    let mut first = true;
    let mut play = None;
    for (key, record) in &app.discovery.replica.document.records {
        let Some(Value::Mix { title, uris }) = &record.value else {
            continue;
        };
        if first {
            ui.add_space(14.0);
            theme::text(
                ui,
                gettext(locale, "Your saved mixes"),
                theme::semibold(15.0),
                palette.text,
            );
            ui.add_space(4.0);
            first = false;
        }
        ui.push_id(key, |ui| {
            ui.horizontal(|ui| {
                ui.add_enabled_ui(playable && !uris.is_empty(), |ui| {
                    if theme::icon_button(
                        ui,
                        Icon::PlayFilled,
                        14.0,
                        palette.secondary,
                        palette.text,
                        &gettext(locale, "Play mix"),
                    )
                    .clicked()
                    {
                        play = Some(key.clone());
                    }
                });
                theme::text(ui, title.as_str(), theme::medium(13.5), palette.text);
                theme::text(
                    ui,
                    locale.song_count(uris.len() as u32),
                    theme::regular(12.5),
                    palette.secondary,
                );
            });
        });
    }
    if let Some(key) = play {
        app.actions.push(Action::DiscoveryPlayMix(key));
    }
}

fn history(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    ui.add_space(14.0);
    theme::text(
        ui,
        gettext(locale, "Your AI history"),
        theme::semibold(15.0),
        palette.text,
    );
    ui.add_space(4.0);
    // Only an open section sorts the newest ten, by reference.
    let entries = app.discovery.replica.document.recent_history();
    if entries.is_empty() {
        theme::subtle(ui, &palette, &gettext(locale, "No AI requests yet."));
        return;
    }
    for (key, record) in entries {
        let Some(Value::History {
            prompt,
            suggestions,
        }) = &record.value
        else {
            continue;
        };
        ui.push_id(key, |ui| {
            theme::text(ui, prompt.as_str(), theme::medium(13.5), palette.text);
            for suggestion in suggestions {
                theme::text(
                    ui,
                    format!("{} · {}", suggestion.title, suggestion.artist),
                    theme::regular(12.5),
                    palette.secondary,
                );
            }
            ui.add_space(8.0);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{TextBuffer, text::CharIndex};

    #[test]
    fn prompt_insertion_respects_bytes_without_erasing_existing_text() {
        let mut text = "音".repeat(1333);
        let mut buffer = PromptBuffer(&mut text);
        assert_eq!(buffer.insert_text("🎵", CharIndex(0)), 0);
        assert_eq!(buffer.insert_text("ab", CharIndex(0)), 1);
        assert_eq!(buffer.as_str().len(), MAX_PROMPT_BYTES);
        assert!(buffer.as_str().ends_with(&"音".repeat(1333)));
        buffer.delete_char_range(CharIndex(0)..CharIndex(2));
        assert_eq!(buffer.insert_text("🎵", CharIndex(0)), 1);
        assert!(buffer.as_str().starts_with("🎵音"));
        assert_eq!(buffer.as_str().len(), MAX_PROMPT_BYTES);
    }
}
