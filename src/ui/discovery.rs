//! The personal discovery workspace, using the existing native player actions.

use super::widgets::{self, TrackRow};
use crate::app::App;
use crate::discovery::{Rating, Value};
use crate::model::{Action, RowContext};
use crate::theme;
use egui::{CornerRadius, Frame, Margin, RichText, Stroke};
use std::sync::Arc;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    theme::text(ui, "Spotiurge", theme::bold(36.0), palette.text);
    theme::text(
        ui,
        "A little familiar. Something unexpected.",
        theme::regular(16.0),
        palette.secondary,
    );
    ui.add_space(18.0);

    // A quiet translucent material over the existing native background. No
    // animated blur pass or idle repaint loop is needed for this workspace.
    Frame::new()
        .fill(palette.surface.gamma_multiply(0.88))
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            theme::text(ui, "Your next discoveries", theme::bold(24.0), palette.text);
            ui.add_space(6.0);
            theme::text(
                ui,
                "Tell me what you feel like hearing. Your feedback shapes the next mix.",
                theme::regular(14.0),
                palette.secondary,
            );
            ui.add_space(12.0);
            let mut draft = app.discovery.draft.clone();
            let input = ui.add_enabled(
                app.discovery.ready,
                egui::TextEdit::multiline(&mut draft)
                    .desired_width(f32::INFINITY)
                    .desired_rows(2)
                    .char_limit(4000)
                    .hint_text("Warm jazz, spacious electronics, a few surprises…")
                    .font(theme::regular(16.0)),
            );
            #[cfg(feature = "demo")]
            if app.offline && app.discovery.status == "DEMO_FOCUS" {
                input.request_focus();
            }
            if input.changed() {
                app.actions.push(Action::DiscoveryDraft(draft));
            }
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.add_enabled_ui(app.discovery.ready && !app.offline, |ui| {
                    if theme::pill_button(ui, &palette, "Save taste", false).clicked() {
                        app.actions.push(Action::DiscoverySaveTaste);
                    }
                });
                let enabled = app.discovery.ready
                    && !app.discovery.busy
                    && !app.discovery.draft.trim().is_empty()
                    && !app.offline;
                ui.add_enabled_ui(enabled, |ui| {
                    if theme::pill_button(
                        ui,
                        &palette,
                        if app.discovery.busy {
                            "Finding music…"
                        } else {
                            "Find music for me"
                        },
                        true,
                    )
                    .clicked()
                    {
                        app.actions.push(Action::DiscoveryRecommend);
                    }
                });
                ui.add_enabled_ui(
                    app.discovery.ready && !app.discovery.syncing && !app.offline,
                    |ui| {
                        if theme::pill_button(
                            ui,
                            &palette,
                            if app.discovery.syncing {
                                "Syncing…"
                            } else {
                                "Sync my devices"
                            },
                            false,
                        )
                        .clicked()
                        {
                            app.actions.push(Action::DiscoverySync);
                        }
                    },
                );
                if app.discovery.busy || app.discovery.syncing {
                    ui.spinner();
                }
            });
            ui.add_space(8.0);
            let status = if app.offline
                && (app.discovery.status.is_empty() || app.discovery.status == "DEMO_FOCUS")
            {
                "Demo discoveries. Feedback, recommendations and cloud requests are disabled."
            } else if app.discovery.status.is_empty() {
                "Opening your personal discovery workspace…"
            } else {
                &app.discovery.status
            };
            ui.label(
                RichText::new(status)
                    .font(theme::regular(13.0))
                    .color(palette.secondary),
            );
        });
    ui.add_space(20.0);

    let picks = app.discovery.picks.clone();
    if !picks.is_empty() {
        ui.horizontal_wrapped(|ui| {
            theme::text(ui, "Picked for you", theme::bold(20.0), palette.text);
            ui.add_enabled_ui(
                app.discovery.ready && !app.offline && picks.iter().any(|p| p.track.is_some()),
                |ui| {
                    if theme::pill_button(ui, &palette, "Save this mix", false).clicked() {
                        app.actions.push(Action::DiscoverySaveMix);
                    }
                },
            );
        });
        let uris: Arc<[String]> = picks
            .iter()
            .filter_map(|p| p.track.as_ref().map(|t| t.uri.clone()))
            .collect::<Vec<_>>()
            .into();
        let context = RowContext::Uris(uris);
        let mut playable_index = 0;
        for pick in picks {
            if let Some(track) = pick.track {
                let item = crate::api::models::PlayableItem::Track(track.clone());
                widgets::track_row(
                    ui,
                    app,
                    TrackRow {
                        index: playable_index,
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
                );
                playable_index += 1;
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(&pick.suggestion.reason)
                            .font(theme::regular(13.0))
                            .color(palette.secondary),
                    );
                    for (label, rating) in [
                        ("More like this", Rating::Love),
                        ("Less like this", Rating::Less),
                    ] {
                        let selected =
                            app.discovery.replica.document.rating(&track.uri) == Some(rating);
                        if ui
                            .add_enabled(
                                app.discovery.ready && !app.offline,
                                egui::Button::selectable(selected, label),
                            )
                            .on_disabled_hover_text("Feedback is unavailable in the demo.")
                            .clicked()
                        {
                            app.actions.push(Action::DiscoveryRate {
                                uri: track.uri.clone(),
                                title: track.name.clone(),
                                artist: track.artist_names(),
                                rating,
                            });
                        }
                    }
                });
            } else {
                theme::text(
                    ui,
                    format!("{} · {}", pick.suggestion.title, pick.suggestion.artist),
                    theme::semibold(15.0),
                    palette.text,
                );
                ui.label(
                    RichText::new("Spotify match not verified. This suggestion cannot play.")
                        .color(palette.secondary),
                );
            }
            ui.add_space(12.0);
        }
    }

    let mixes = app
        .discovery
        .replica
        .document
        .records
        .iter()
        .filter_map(|(key, record)| match &record.value {
            Some(Value::Mix { title, uris }) => Some((key.clone(), title.clone(), uris.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !mixes.is_empty() {
        theme::section_title(ui, &palette, "Your saved mixes");
        for (key, title, uris) in mixes {
            ui.push_id(key, |ui| {
                ui.horizontal_wrapped(|ui| {
                    theme::text(ui, &title, theme::semibold(15.0), palette.text);
                    if !uris.is_empty()
                        && theme::pill_button(ui, &palette, "Play mix", false).clicked()
                    {
                        app.actions.push(Action::PlayUris { uris, index: 0 });
                    }
                });
            });
        }
        ui.add_space(16.0);
    }
    ui.separator();
    let mut histories = app
        .discovery
        .replica
        .document
        .records
        .values()
        .filter_map(|r| match &r.value {
            Some(Value::History {
                prompt,
                suggestions,
            }) => Some((r.stamp.clone(), prompt.clone(), suggestions.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    histories.sort_by(|a, b| b.0.cmp(&a.0));
    if !histories.is_empty() {
        ui.collapsing("Your AI history", |ui| {
            for (stamp, prompt, suggestions) in histories.iter().take(10) {
                ui.push_id((&stamp.device, stamp.counter), |ui| {
                    theme::text(ui, prompt, theme::semibold(14.0), palette.text);
                    for suggestion in suggestions {
                        ui.label(format!("{} · {}", suggestion.title, suggestion.artist));
                    }
                    ui.add_space(10.0);
                });
            }
        });
    }
    ui.add_space(18.0);
}
