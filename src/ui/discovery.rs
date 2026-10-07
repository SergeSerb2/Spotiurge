//! The personal discovery workspace, using the existing native player actions.
//!
//! One restrained well opens on playable picks: a compact header with play
//! and refresh, an exploration strip, one honest catalogue summary and 56-point
//! rows. Taste editing, unmatched suggestions and AI history stay out of the way
//! until asked for.

use super::widgets::{self, TrackRow};
use super::{material, motion};
use crate::app::App;
use crate::discovery::{
    CatalogueOutcome, Document, Exploration, MAX_PROMPT_BYTES, Rating, RecommendationErrorKind,
    Record, Value, limit_prompt,
};
use crate::i18n::{Locale, gettext, ngettext};
use crate::model::{Action, RowContext};
use crate::theme::{self, Icon, Palette};
use egui::{Align, Color32, CornerRadius, Frame, Layout, Margin, Rect, Sense, Vec2, pos2, vec2};
use std::sync::Arc;

/// Width of the main content above which reasons get their own column.
const WIDE: f32 = 920.0;
const FEEDBACK: f32 = 28.0;
/// The overflow menu's width range, in points.
const MENU_MIN_WIDTH: f32 = 240.0;
const MENU_MAX_WIDTH: f32 = 280.0;
/// The widest the taste prompt grows, so a line stays a comfortable read.
const PROMPT_MAX_WIDTH: f32 = 560.0;

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
    Frame::new()
        .fill(material::glass(&palette, material::Kind::Content).fill)
        .corner_radius(CornerRadius::same(material::PANE_RADIUS as u8))
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            pane(app, ui);
        });
    ui.add_space(24.0);
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
    ui.add_space(16.0);

    let onboarding = app.discovery.ready && !app.discovery.replica.document.has_inputs();
    if app.discovery.editing_taste || onboarding {
        taste_editor(app, ui, onboarding);
        ui.add_space(16.0);
    }

    summary(app, ui, &counts);
    ui.add_space(8.0);

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
                // The lamp lights only while there is something to play.
                // Unavailable, the key goes neutral at full opacity, so it
                // reads as dimmed without its icon fading into the glass.
                let (fill, hover, icon) = if can_play {
                    (palette.accent, palette.accent_hover, palette.on_accent)
                } else {
                    ui.set_opacity(1.0);
                    let key = material::selected_fill(&palette);
                    (key, key, palette.secondary)
                };
                if theme::circle_button(
                    ui,
                    Icon::PlayFilled,
                    44.0,
                    fill,
                    hover,
                    icon,
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
        if app.discovery.status.is_empty() {
            gettext(app.locale, "Opening your personal discovery workspace…").into_owned()
        } else {
            app.discovery.status.clone()
        }
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
    // Hung from the button's trailing edge, as wide as its items need.
    egui::Popup::menu(&more)
        .frame(widgets::menu_frame(&palette))
        .align(egui::RectAlign::BOTTOM_END)
        .show(|ui| {
            ui.set_min_width(MENU_MIN_WIDTH);
            ui.set_max_width(MENU_MAX_WIDTH);
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
            widgets::menu_separator(ui, &palette);
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
    let choices = [
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
    ];
    let options: Vec<(Exploration, &str)> = choices
        .iter()
        .map(|(choice, label, _)| (*choice, label.as_ref()))
        .collect();
    let id = ui.make_persistent_id("discovery-exploration");
    let strip = ui.scope(|ui| theme::choice_chips(ui, &palette, id, &options, current, false));
    // ponytail: choice_chips does not return its chips, so the hints sit on
    // hover-only overlays at its layout (soft-button widths, six points
    // apart). Hover-only widgets never take the click from the chip below.
    // A chips variant returning responses would replace this.
    let strip_rect = strip.response.rect;
    let mut x = strip_rect.left();
    for (index, (_, label, hint)) in choices.iter().enumerate() {
        let width = theme::soft_button_width(ui, label);
        let rect = Rect::from_min_size(pos2(x, strip_rect.top()), vec2(width, strip_rect.height()));
        ui.interact(rect, id.with(("hint", index)), Sense::hover())
            .on_hover_text(hint.as_ref());
        x += width + 6.0;
    }
    if let Some(choice) = strip.inner {
        app.actions.push(Action::DiscoveryExploration(choice));
    }
}

fn taste_editor(app: &mut App, ui: &mut egui::Ui, onboarding: bool) {
    let palette = app.palette;
    let locale = app.locale;
    let heading = if onboarding {
        gettext(locale, "Tell Spotiurge what you like")
    } else {
        gettext(locale, "Your taste")
    };
    theme::section_title(ui, &palette, &heading);
    ui.add_space(8.0);
    let mut draft = app.discovery.draft.clone();
    let ready = app.discovery.ready;
    let input = ui
        .scope(|ui| {
            ui.set_max_width(ui.available_width().min(PROMPT_MAX_WIDTH));
            widgets::field_well(ui, &palette, |ui| {
                ui.add_enabled(
                    ready,
                    egui::TextEdit::multiline(&mut PromptBuffer(&mut draft))
                        .frame(egui::Frame::NONE)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3)
                        .hint_text(gettext(
                            locale,
                            "Warm jazz, spacious electronics, a few surprises…",
                        ))
                        .font(theme::regular(15.0)),
                )
            })
        })
        .inner;
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
        ui.add_enabled_ui(can_save, |ui| {
            if theme::pill_button(ui, &palette, &gettext(locale, "Save taste"), true)
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
        // Model failures are warnings; local workspace failures below need recovery.
        let response = notice(ui, &palette, Icon::CircleAlert, palette.warning, &text);
        if !detail.is_empty() && detail != text {
            response.on_hover_text(detail);
        }
        ui.add_space(4.0);
    }

    if !app.discovery.ready {
        if app.discovery.status.is_empty() {
            notice(
                ui,
                &palette,
                Icon::Loader,
                palette.secondary,
                &gettext(locale, "Opening your personal discovery workspace…"),
            );
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                theme::icon(ui, Icon::CircleAlert, 14.0, palette.danger);
                // Keep the complete recovery advice visible in narrow windows.
                let galley = crate::bidi::layout(
                    ui.painter(),
                    &app.discovery.status,
                    theme::regular(13.0),
                    palette.danger,
                    ui.available_width(),
                    usize::MAX,
                    None,
                );
                ui.add(egui::Label::new(galley).selectable(false));
            });
        }
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
        ui.add_space(4.0);
        theme::text(
            ui,
            status.to_owned(),
            theme::regular(13.0),
            palette.secondary,
        );
    }
    if app.offline {
        ui.add_space(4.0);
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
        theme::text(
            ui,
            text,
            theme::regular(13.0),
            if icon_color == palette.warning || icon_color == palette.danger {
                icon_color
            } else {
                palette.secondary
            },
        )
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
        // The row's hover fill sits behind the song, its reason and its
        // feedback, so the line lifts as one. The song carries its own
        // stronger lift and, while it plays, moss text and a playing meter.
        let highlight = ui.painter().add(egui::Shape::Noop);
        let line = ui.horizontal(|ui| {
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
            let row_id = (row.id, row.has_focus());
            if !wide && !reason.is_empty() {
                row.on_hover_text(reason);
            }
            if wide {
                reason_cell(ui, &palette, reason, reason_width);
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
            row_id
        });
        let ((row_id, focused), rect) = (line.inner, line.response.rect);
        let lift = motion::toggle(
            ui.ctx(),
            row_id.with("line"),
            ui.rect_contains_pointer(rect) || focused,
            motion::FEEDBACK,
        );
        if lift > 0.0 {
            ui.painter().set(
                highlight,
                egui::Shape::rect_filled(
                    rect,
                    CornerRadius::same(8),
                    material::hover_fill(&palette).gamma_multiply(0.7 * lift),
                ),
            );
        }
        index += 1;
    }
}

/// A pick's reason in a cell of fixed width, so the feedback after it sits
/// in one column however long each reason is.
fn reason_cell(ui: &mut egui::Ui, palette: &Palette, reason: &str, width: f32) {
    ui.allocate_ui_with_layout(
        vec2(width, theme::ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_min_width(width);
            let galley = crate::bidi::layout(
                ui.painter(),
                reason,
                theme::regular(13.0),
                palette.secondary,
                width,
                1,
                Some(crate::bidi::ELLIPSIS),
            );
            ui.add(egui::Label::new(galley).selectable(false))
                .on_hover_text(reason);
        },
    );
}

/// A 28-point round toggle. Love fills with the accent and Less with the
/// neutral text colour, so neither reads as the Spotify heart beside it.
/// Hover and selection ease in rather than switching.
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
        let lift = motion::toggle(
            ui.ctx(),
            response.id.with("lift"),
            hovered,
            motion::FEEDBACK,
        );
        let on = motion::toggle(ui.ctx(), response.id.with("on"), selected, motion::STATE);
        let (on_fill, on_icon) = match rating {
            Rating::Love => (
                palette.accent.lerp_to_gamma(palette.accent_hover, lift),
                palette.on_accent,
            ),
            Rating::Less => (palette.text, palette.window),
        };
        let painter = ui.painter();
        let radius = FEEDBACK / 2.0;
        if lift > 0.0 {
            painter.circle_filled(
                rect.center(),
                radius,
                material::selected_fill(palette).gamma_multiply(lift),
            );
        }
        if on > 0.0 {
            painter.circle_filled(rect.center(), radius, on_fill.gamma_multiply(on));
        }
        let color = palette
            .secondary
            .lerp_to_gamma(palette.text, lift)
            .lerp_to_gamma(on_icon, on);
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
    // The stroke chevron every other disclosure uses.
    .icon(move |ui, openness, response| {
        let icon = if openness > 0.5 {
            Icon::ChevronDown
        } else {
            Icon::ChevronRight
        };
        theme::paint_icon(ui, icon, response.rect, 14.0, palette.secondary);
    })
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

/// Saved mixes shown at first, and how many more each "See more" reveals.
const MIXES_SHOWN: usize = 8;
const MIXES_PAGE: usize = 24;

fn saved_mixes(app: &mut App, ui: &mut egui::Ui) {
    let playable = app.discovery_playback_available();
    let document = &app.discovery.replica.document;
    if let Some(key) = mix_list(ui, &app.palette, app.locale, playable, document) {
        app.actions.push(Action::DiscoveryPlayMix(key));
    }
}

/// The newest saved mixes, a page at a time. Thousands can sync here, so
/// only the shown ones get widgets; "See more" reaches every one of them.
/// Returns the key of the mix to play.
fn mix_list(
    ui: &mut egui::Ui,
    palette: &Palette,
    locale: Locale,
    playable: bool,
    document: &Document,
) -> Option<String> {
    let mut mixes: Vec<(&String, &Record, &str, &[String])> = document
        .records
        .iter()
        .filter_map(|(key, record)| match &record.value {
            Some(Value::Mix { title, uris }) => {
                Some((key, record, title.as_str(), uris.as_slice()))
            }
            _ => None,
        })
        .collect();
    if mixes.is_empty() {
        return None;
    }
    let total = mixes.len();
    let id = ui.make_persistent_id("discovery-saved-mixes");
    let wanted = ui
        .data(|data| data.get_temp::<usize>(id))
        .unwrap_or(MIXES_SHOWN);
    let shown = wanted.min(total);
    // Newest first, by the same stamp order sync uses.
    mixes.sort_unstable_by(|a, b| (&b.1.stamp, b.0).cmp(&(&a.1.stamp, a.0)));
    mixes.truncate(shown);

    ui.add_space(16.0);
    theme::section_title(ui, palette, &gettext(locale, "Your saved mixes"));
    ui.add_space(4.0);
    let mut play = None;
    for (key, _, title, uris) in mixes {
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
                theme::text(ui, title, theme::medium(14.0), palette.text);
                theme::text(
                    ui,
                    locale.song_count(uris.len() as u32),
                    theme::regular(13.0),
                    palette.secondary,
                );
            });
        });
    }
    if total > MIXES_SHOWN {
        ui.add_space(4.0);
        let mut next = None;
        ui.horizontal(|ui| {
            if shown < total
                && theme::soft_button(ui, palette, None, &gettext(locale, "See more"), false)
                    .clicked()
            {
                next = Some((shown + MIXES_PAGE).min(total));
            }
            if shown > MIXES_SHOWN
                && theme::soft_button(ui, palette, None, &gettext(locale, "Show less"), false)
                    .clicked()
            {
                next = Some(MIXES_SHOWN);
            }
        });
        if let Some(next) = next {
            ui.data_mut(|data| data.insert_temp(id, next));
        }
    }
    play
}

fn history(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    ui.add_space(14.0);
    theme::section_title(ui, &palette, &gettext(locale, "Your AI history"));
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
            theme::text(ui, prompt.as_str(), theme::medium(14.0), palette.text);
            for suggestion in suggestions {
                theme::text(
                    ui,
                    format!("{} · {}", suggestion.title, suggestion.artist),
                    theme::regular(13.0),
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

    /// Each reason holds a cell of the same width, so the feedback after it
    /// lines up in one column whatever the reason's length (the finish
    /// review saw it drift by up to 21 points).
    #[test]
    fn feedback_lines_up_after_reasons_of_any_length() {
        let ctx = egui::Context::default();
        theme::install(&ctx);
        let palette = Palette::light();
        let mut feedback = Vec::new();
        for _ in 0..2 {
            feedback.clear();
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                for reason in [
                    "",
                    "Demo reason: one step further out, still spacious.",
                    "Demo reason: a late-night pulse with a soft edge.",
                    "Demo reason: warm textures close to your saved songs, and then some more.",
                ] {
                    ui.horizontal(|ui| {
                        reason_cell(ui, &palette, reason, 300.0);
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::splat(FEEDBACK), Sense::hover());
                        feedback.push(rect.left());
                    });
                }
            });
            output.textures_delta.clear();
        }
        let first = feedback[0];
        assert!(feedback.iter().all(|x| *x == first), "{feedback:?}");
        assert!(first >= 300.0, "the cell keeps its width: {first}");
    }

    /// Thousands of synced mixes cost a page of rows, newest first, and
    /// "See more" reveals the next page.
    #[test]
    fn saved_mixes_draw_one_page_at_a_time() {
        use crate::discovery::Stamp;
        use egui::accesskit::{Action as AccessibleAction, ActionRequest, Role, TreeId};

        let mut document = Document::default();
        for counter in 0..2_000u64 {
            document.records.insert(
                format!("mix:{counter:04}"),
                Record {
                    stamp: Stamp {
                        counter,
                        device: "a".into(),
                    },
                    value: Some(Value::Mix {
                        title: format!("Mix {counter}"),
                        uris: vec!["spotify:track:x".into()],
                    }),
                },
            );
        }
        let ctx = egui::Context::default();
        theme::install(&ctx);
        ctx.enable_accesskit();
        let palette = Palette::dark();
        let draw = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        vec2(900.0, 700.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    mix_list(ui, &palette, Locale::English, true, &document);
                },
            );
            output.textures_delta.clear();
            output.platform_output.accesskit_update.unwrap()
        };
        let labelled = |tree: &egui::accesskit::TreeUpdate, label: &str| {
            tree.nodes
                .iter()
                .filter(|(_, node)| node.label() == Some(label) || node.value() == Some(label))
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        };

        draw(Vec::new());
        let tree = draw(Vec::new());
        assert_eq!(labelled(&tree, "Play mix").len(), MIXES_SHOWN);
        assert_eq!(labelled(&tree, "Mix 1999").len(), 1, "the newest leads");
        assert!(labelled(&tree, "Mix 1991").is_empty());
        let more = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some("See more"))
            .expect("a See more button")
            .0;

        draw(vec![egui::Event::AccessKitActionRequest(ActionRequest {
            target_tree: TreeId::ROOT,
            target_node: more,
            action: AccessibleAction::Click,
            data: None,
        })]);
        let tree = draw(Vec::new());
        assert_eq!(labelled(&tree, "Play mix").len(), MIXES_SHOWN + MIXES_PAGE);
        assert_eq!(labelled(&tree, "Mix 1968").len(), 1);
        assert!(labelled(&tree, "Mix 1967").is_empty());
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Show less")),
            "the list can fold back"
        );
    }
}
