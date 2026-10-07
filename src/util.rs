//! Small helpers shared across the application.

use std::borrow::Cow;

use crate::i18n::{Locale, gettext, ngettext, pgettext};

/// `3:45` for track lengths, `1:02:03` past an hour.
pub fn format_duration_ms(ms: u32) -> String {
    let total = ms / 1000;
    let hours = total / 3600;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// `2 hr 13 min` for playlist totals, `45 min 12 sec` under an hour.
pub fn format_total_ms(locale: Locale, ms: u64) -> String {
    let total = ms / 1000;
    let hours = total / 3600;
    let minutes = (total / 60) % 60;
    let seconds = total % 60;
    if hours > 0 {
        // Translators: A length of time, abbreviated. Keep {hours} and {minutes}.
        gettext(locale, "{hours} hr {minutes} min")
            .replace("{hours}", &hours.to_string())
            .replace("{minutes}", &minutes.to_string())
    } else if minutes > 0 {
        // Translators: A length of time, abbreviated. Keep {minutes} and {seconds}.
        gettext(locale, "{minutes} min {seconds} sec")
            .replace("{minutes}", &minutes.to_string())
            .replace("{seconds}", &seconds.to_string())
    } else {
        // Translators: A length of time, abbreviated. Keep {seconds}.
        gettext(locale, "{seconds} sec").replace("{seconds}", &seconds.to_string())
    }
}

/// Episode lengths read as `1 hr 12 min` or `38 min`.
pub fn format_episode_ms(locale: Locale, ms: u32) -> String {
    let minutes = ms / 60_000;
    let hours = minutes / 60;
    if hours > 0 {
        gettext(locale, "{hours} hr {minutes} min")
            .replace("{hours}", &hours.to_string())
            .replace("{minutes}", &(minutes % 60).to_string())
    } else {
        // Translators: A length of time, abbreviated. Keep {minutes}.
        gettext(locale, "{minutes} min").replace("{minutes}", &minutes.max(1).to_string())
    }
}

pub fn format_count(count: u64) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(character);
    }
    out
}

/// `Jan 5, 2024` from an ISO-8601 timestamp or a bare date.
pub fn format_date(locale: Locale, iso: &str) -> String {
    let date = iso.get(..10).unwrap_or(iso);
    let mut parts = date.split('-');
    let (Some(year), Some(month)) = (parts.next(), parts.next()) else {
        return iso.to_string();
    };
    let day = parts.next();
    let month_name = match month {
        "01" => pgettext(locale, "month", "Jan"),
        "02" => pgettext(locale, "month", "Feb"),
        "03" => pgettext(locale, "month", "Mar"),
        "04" => pgettext(locale, "month", "Apr"),
        "05" => pgettext(locale, "month", "May"),
        "06" => pgettext(locale, "month", "Jun"),
        "07" => pgettext(locale, "month", "Jul"),
        "08" => pgettext(locale, "month", "Aug"),
        "09" => pgettext(locale, "month", "Sep"),
        "10" => pgettext(locale, "month", "Oct"),
        "11" => pgettext(locale, "month", "Nov"),
        "12" => pgettext(locale, "month", "Dec"),
        _ => return iso.to_string(),
    };
    let dated = match day.and_then(|day| day.trim_start_matches('0').parse::<u8>().ok()) {
        Some(day) => {
            // Translators: A date. {month} is an abbreviated month name; reorder as your language writes dates.
            gettext(locale, "{month} {day}, {year}").replace("{day}", &day.to_string())
        }
        // Translators: A month and year. {month} is an abbreviated month name.
        None => gettext(locale, "{month} {year}").into_owned(),
    };
    dated
        .replace("{month}", &month_name)
        .replace("{year}", year)
}

/// `5 minutes ago` for recent ISO-8601 timestamps, otherwise the usual date.
///
/// Dates are shown relatively for their first 30 days, matching the playlist
/// table's compact, time-aware presentation. `now` is an argument so callers
/// can render against one instant and the boundary behaviour stays testable.
pub fn format_relative_date(locale: Locale, iso: &str, now: jiff::Timestamp) -> String {
    let Ok(added) = iso.parse::<jiff::Timestamp>() else {
        return format_date(locale, iso);
    };
    let seconds = added.duration_until(now).as_secs_f64().floor() as i64;
    if !(0..30 * 24 * 60 * 60).contains(&seconds) {
        return format_date(locale, iso);
    }

    let (count, text) = if seconds < 60 {
        let count = seconds;
        let text = ngettext(
            locale,
            // Translators: How long ago a song was added. Keep {count}.
            "{count} second ago",
            "{count} seconds ago",
            count as u32,
        );
        (count, text)
    } else if seconds < 60 * 60 {
        let count = seconds / 60;
        let text = ngettext(
            locale,
            // Translators: How long ago a song was added. Keep {count}.
            "{count} minute ago",
            "{count} minutes ago",
            count as u32,
        );
        (count, text)
    } else if seconds < 24 * 60 * 60 {
        let count = seconds / (60 * 60);
        let text = ngettext(
            locale,
            // Translators: How long ago a song was added. Keep {count}.
            "{count} hour ago",
            "{count} hours ago",
            count as u32,
        );
        (count, text)
    } else if seconds < 7 * 24 * 60 * 60 {
        let count = seconds / (24 * 60 * 60);
        // Translators: How long ago a song was added. Keep {count}.
        let text = ngettext(locale, "{count} day ago", "{count} days ago", count as u32);
        (count, text)
    } else {
        let count = seconds / (7 * 24 * 60 * 60);
        let text = ngettext(
            locale,
            // Translators: How long ago a song was added. Keep {count}.
            "{count} week ago",
            "{count} weeks ago",
            count as u32,
        );
        (count, text)
    };
    text.replace("{count}", &count.to_string())
}

/// Tears the id out of `spotify:track:abc` and friends.
pub fn uri_id(uri: &str) -> Option<&str> {
    uri.rsplit(':').next().filter(|id| !id.is_empty())
}

pub fn uri_kind(uri: &str) -> Option<&str> {
    let mut parts = uri.split(':');
    parts.next()?;
    parts.next()
}

/// Spotify's radio station seeded by a song, playlist, album, or artist.
pub fn station_uri(seed: &str) -> Option<String> {
    let kind = uri_kind(seed)?;
    let id = uri_id(seed)?;
    (seed == format!("spotify:{kind}:{id}")
        && matches!(kind, "track" | "playlist" | "album" | "artist"))
    .then(|| format!("spotify:station:{kind}:{id}"))
}

/// The song, playlist, album, or artist a radio station is seeded by.
pub fn station_seed(station: &str) -> Option<String> {
    let seed = format!("spotify:{}", station.strip_prefix("spotify:station:")?);
    station_uri(&seed).is_some().then_some(seed)
}

pub fn open_spotify_url(uri: &str) -> Option<String> {
    let kind = uri_kind(uri)?;
    let id = uri_id(uri)?;
    Some(format!("https://open.spotify.com/{kind}/{id}"))
}

/// The menu-bar shape for macOS: the surge S alone, drawn heavier so it
/// holds at menu-bar size. macOS template images use only the alpha
/// channel and paint the shape themselves, black in a light menu bar and
/// white in a dark one.
pub fn tray_template_rgba(size: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; size * size * 4];
    // The S fills the square, with a pixel of margin.
    let unit = (size as f32 - 2.0) / 84.0;
    let origin = (size as f32 - 128.0 * unit) / 2.0;
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5 - origin) / unit;
            let v = (y as f32 + 0.5 - origin) / unit;
            let coverage =
                ((TEMPLATE_STROKE / 2.0 - surge_distance(u, v)) * unit + 0.5).clamp(0.0, 1.0);
            rgba[(y * size + x) * 4 + 3] = (coverage * 255.0).round() as u8;
        }
    }
    rgba
}

/// The mark rasterised to pixels: the window icon, the trays and the logo
/// drawn in the app (`theme::logo`) all use this one picture, which
/// `assets/brand/spotiurge-mark.svg` draws as a vector.
///
/// A tile of smoked glass, lit along its top edge, with a VU-amber surge
/// S glowing through it: an S for Spotiurge whose two bowls are one swell
/// of a wave.
pub fn app_icon_rgba(size: usize) -> Vec<u8> {
    let mut rgba = vec![0u8; size * size * 4];
    // The tile keeps a pixel of margin, so its edge is never clipped.
    let unit = (size as f32 - 2.0) / 120.0;
    let origin = size as f32 / 2.0 - 64.0 * unit;
    // A glow only where there are pixels enough to show one.
    let glow = (size as f32 / 128.0).clamp(0.0, 1.0);
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5 - origin) / unit;
            let v = (y as f32 + 0.5 - origin) / unit;
            let tile = rounded_square_distance(u, v);
            let coverage = (0.5 - tile * unit).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }
            // Smoked glass: deep ink below, a cooler blue above.
            let depth = (v - 4.0) / 120.0;
            let mut colour = mix(TILE_TOP, TILE_BOTTOM, depth);
            // The room's light behind the glass, warm around the S.
            let warmth =
                (1.0 - ((u - 64.0).powi(2) + (v - 70.0).powi(2)).sqrt() / 58.0).clamp(0.0, 1.0);
            colour = mix(colour, AMBER_DEEP, 0.22 * warmth * warmth);
            // The sheen across the upper part of the tile.
            let sheen = (1.0 - (v - 4.0) / 50.0).clamp(0.0, 1.0);
            colour = mix(colour, [255.0, 255.0, 255.0], 0.07 * sheen * sheen);
            // The rim: a lit edge above, a shaded one below.
            let rim = (1.0 - (tile + 1.2).abs() / 1.2).clamp(0.0, 1.0);
            if depth < 0.5 {
                colour = mix(
                    colour,
                    [235.0, 240.0, 255.0],
                    rim * 0.55 * (1.0 - depth * 2.0),
                );
            } else {
                colour = mix(colour, [0.0, 0.0, 0.0], rim * 0.5 * (depth - 0.5) * 2.0);
            }
            let surge = surge_distance(u, v);
            let halo = ((surge - STROKE / 2.0) / 9.0).clamp(0.0, 1.0);
            colour = mix(colour, AMBER_DEEP, glow * 0.35 * (1.0 - halo).powi(2));
            let ink = ((STROKE / 2.0 - surge) * unit + 0.5).clamp(0.0, 1.0);
            if ink > 0.0 {
                // The S is lit from above like the tile, and rounded like a
                // tube of amber glass: a highlight along the edges facing
                // the upper left, a deeper tone along the lower right.
                let mut amber = mix(AMBER_LIGHT, AMBER_DEEP, (v - 22.0) / 84.0);
                let edge = |du: f32, dv: f32| {
                    ((surge_distance(u + du, v + dv) - (STROKE / 2.0 - 3.0)) / 3.0).clamp(0.0, 1.0)
                };
                amber = mix(amber, [255.0, 244.0, 214.0], 0.7 * edge(-1.8, -2.4) * glow);
                amber = mix(amber, [205.0, 92.0, 14.0], 0.4 * edge(1.8, 2.4) * glow);
                colour = mix(colour, amber, ink);
            }
            let index = (y * size + x) * 4;
            rgba[index] = colour[0].round() as u8;
            rgba[index + 1] = colour[1].round() as u8;
            rgba[index + 2] = colour[2].round() as u8;
            rgba[index + 3] = (coverage * 255.0).round() as u8;
        }
    }
    rgba
}

const TILE_TOP: [f32; 3] = [38.0, 44.0, 74.0];
const TILE_BOTTOM: [f32; 3] = [10.0, 12.0, 22.0];
const AMBER_LIGHT: [f32; 3] = [255.0, 214.0, 128.0];
const AMBER_DEEP: [f32; 3] = [255.0, 146.0, 40.0];
/// The S's stroke on the tile, and the heavier one of the menu-bar shape.
const STROKE: f32 = 15.0;
const TEMPLATE_STROKE: f32 = 17.0;

/// Mixes two colours, `t` of the way from `a` to `b`.
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Signed distance from the tile's edge, negative inside: a square of 120
/// units centred on 64, with corners rounded by 27, the proportion of a
/// macOS icon.
fn rounded_square_distance(u: f32, v: f32) -> f32 {
    const HALF: f32 = 60.0;
    const RADIUS: f32 = 27.0;
    let qx = (u - 64.0).abs() - (HALF - RADIUS);
    let qy = (v - 64.0).abs() - (HALF - RADIUS);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    outside + qx.max(qy).min(0.0) - RADIUS
}

/// Distance from the centre line of the surge S, on the mark's 128-unit
/// square: two bowls of radius 17, one above the other, joined at the
/// centre, the upper turning back from the top right and the lower ending
/// at the bottom left, the whole leaning forward like a rising wave.
fn surge_distance(u: f32, v: f32) -> f32 {
    const RADIUS: f32 = 17.0;
    const LEAN: f32 = 0.12;
    // Undo the lean, so the bowls are plain circles.
    let u = u - (64.0 - v) * LEAN;
    let upper = arc_distance(
        u,
        v,
        (64.0, 47.0),
        RADIUS,
        330_f32.to_radians(),
        -240_f32.to_radians(),
    );
    let lower = arc_distance(
        u,
        v,
        (64.0, 81.0),
        RADIUS,
        270_f32.to_radians(),
        240_f32.to_radians(),
    );
    upper.min(lower)
}

/// Distance from an arc of a circle at `centre` with `radius`, starting at
/// angle `start` (radians, y down) and turning through `sweep`.
fn arc_distance(u: f32, v: f32, centre: (f32, f32), radius: f32, start: f32, sweep: f32) -> f32 {
    use std::f32::consts::TAU;
    let (dx, dy) = (u - centre.0, v - centre.1);
    let angle = dy.atan2(dx).rem_euclid(TAU);
    let along = ((angle - start) * sweep.signum()).rem_euclid(TAU);
    if along <= sweep.abs() {
        ((dx * dx + dy * dy).sqrt() - radius).abs()
    } else {
        let end = |a: f32| {
            let (sin, cos) = a.sin_cos();
            ((u - centre.0 - radius * cos).powi(2) + (v - centre.1 - radius * sin).powi(2)).sqrt()
        };
        end(start).min(end(start + sweep))
    }
}

pub fn greeting(locale: Locale) -> Cow<'static, str> {
    match local_hour() {
        5..=11 => gettext(locale, "Good morning"),
        12..=17 => gettext(locale, "Good afternoon"),
        _ => gettext(locale, "Good evening"),
    }
}

fn local_hour() -> u8 {
    jiff::Zoned::now().hour() as u8
}

/// Strips the HTML Spotify embeds in playlist descriptions.
pub fn strip_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for character in text.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(character),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&#x2F;", "/")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

/// Atomically replaces `path` with `temporary` on the current platform.
#[cfg(not(windows))]
pub(crate) fn replace_file(
    temporary: &std::path::Path,
    path: &std::path::Path,
) -> std::io::Result<()> {
    std::fs::rename(temporary, path)
}

/// Atomically replaces `path` with `temporary` on Windows.
#[cfg(windows)]
pub(crate) fn replace_file(
    temporary: &std::path::Path,
    path: &std::path::Path,
) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };

    let temporary: Vec<u16> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
    let path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let moved = unsafe {
        MoveFileExW(
            temporary.as_ptr(),
            path.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(rgba: &[u8], size: usize, x: usize, y: usize) -> [u8; 4] {
        let index = (y * size + x) * 4;
        [
            rgba[index],
            rgba[index + 1],
            rgba[index + 2],
            rgba[index + 3],
        ]
    }

    /// The icon is the glass tile at every size: transparent corners, a
    /// lit top edge, a dark glass body and the amber S at its heart.
    #[test]
    fn the_icon_is_the_glass_tile_at_every_size() {
        for size in [16, 32, 128, 512] {
            let icon = app_icon_rgba(size);
            assert_eq!(pixel(&icon, size, 0, 0)[3], 0, "clear corner at {size}");
            let body = pixel(&icon, size, size / 2, size * 9 / 10);
            assert_eq!(body[3], 255, "opaque body at {size}");
            assert!(body[2] > body[0], "smoked blue glass at {size}: {body:?}");
            // The S crosses the middle of the tile; amber is red over blue.
            let heart = pixel(&icon, size, size / 2, size / 2);
            assert!(
                heart[0] > 200 && heart[0] > heart[2] + 80,
                "amber S at the centre at {size}: {heart:?}"
            );
        }
        // The glass reads as glass: lighter at the top than the bottom.
        let large = app_icon_rgba(128);
        let top = pixel(&large, 128, 20, 12);
        let bottom = pixel(&large, 128, 20, 116);
        assert!(top[2] > bottom[2], "top {top:?} bottom {bottom:?}");
    }

    /// The menu-bar shape is the S alone, opaque on its stroke and clear
    /// beside it, so macOS can paint it in the bar's own colour.
    #[test]
    fn the_menu_bar_template_is_the_surge_s() {
        let template = tray_template_rgba(44);
        assert!(
            template
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p[..3] == [0, 0, 0])
        );
        assert_eq!(
            pixel(&template, 44, 22, 22)[3],
            255,
            "the S crosses the centre"
        );
        assert_eq!(pixel(&template, 44, 2, 22)[3], 0, "clear beside it");
        assert_eq!(pixel(&template, 44, 1, 1)[3], 0, "clear in the corner");
    }

    /// Writes the raster exports beside the vector mark. Run it after
    /// changing the mark:
    /// `cargo test --lib util::tests::export_brand_rasters -- --ignored`.
    #[test]
    #[ignore = "writes assets/brand/png"]
    fn export_brand_rasters() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/brand/png");
        std::fs::create_dir_all(&directory).unwrap();
        for size in [16, 32, 64, 128, 256, 512, 1024] {
            image::save_buffer(
                directory.join(format!("spotiurge-{size}.png")),
                &app_icon_rgba(size),
                size as u32,
                size as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
        for size in [22, 44] {
            image::save_buffer(
                directory.join(format!("spotiurge-glyph-template-{size}.png")),
                &tray_template_rgba(size),
                size as u32,
                size as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }

    /// The vector mark and the raster agree on the S's geometry.
    #[test]
    fn the_shipped_svg_draws_the_same_s() {
        let svg = include_str!("../assets/brand/spotiurge-mark.svg");
        for fragment in [
            "M78.72 38.5",
            "A17 17 0 1 0 64 64",
            "A17 17 0 1 1 49.28 89.5",
            "stroke-width=\"15\"",
            "skewX(-6.84)",
            "rx=\"27\"",
        ] {
            assert!(svg.contains(fragment), "{fragment}");
        }
    }

    #[test]
    fn radio_stations_map_to_their_seeds_and_back() {
        for kind in ["track", "playlist", "album", "artist"] {
            let seed = format!("spotify:{kind}:4uLU6hMCjMI75M1A2tKUQC");
            let station = station_uri(&seed).expect("a station");
            assert_eq!(
                station,
                format!("spotify:station:{kind}:4uLU6hMCjMI75M1A2tKUQC")
            );
            assert_eq!(station_seed(&station).as_deref(), Some(seed.as_str()));
        }
        for seed in [
            "spotify:show:abc",
            "spotify:episode:abc",
            "spotify:user:me:collection",
            "spotify:track:",
            "track:abc",
        ] {
            assert_eq!(station_uri(seed), None, "{seed}");
        }
        assert_eq!(station_seed("spotify:playlist:abc"), None);
    }

    #[test]
    fn durations() {
        assert_eq!(format_duration_ms(225_000), "3:45");
        assert_eq!(format_duration_ms(3_723_000), "1:02:03");
        assert_eq!(format_total_ms(Locale::English, 7_980_000), "2 hr 13 min");
        assert_eq!(format_total_ms(Locale::English, 2_712_000), "45 min 12 sec");
        assert_eq!(format_episode_ms(Locale::English, 4_320_000), "1 hr 12 min");
    }

    #[test]
    fn counts_and_dates() {
        assert_eq!(format_count(1_234_567), "1,234,567");
        assert_eq!(format_count(12), "12");
        assert_eq!(
            format_date(Locale::English, "2024-01-05T10:00:00Z"),
            "Jan 5, 2024"
        );
        assert_eq!(format_date(Locale::English, "2024-03"), "Mar 2024");
        assert_eq!(format_date(Locale::English, "2024"), "2024");
    }

    #[test]
    fn recent_dates_are_relative_for_the_first_month() {
        let now: jiff::Timestamp = "2026-08-31T12:00:00Z".parse().unwrap();
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-31T11:59:30Z", now),
            "30 seconds ago"
        );
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-31T11:59:00Z", now),
            "1 minute ago"
        );
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-31T11:00:00Z", now),
            "1 hour ago"
        );
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-30T12:00:00Z", now),
            "1 day ago"
        );
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-17T12:00:00Z", now),
            "2 weeks ago"
        );
        assert_eq!(
            format_relative_date(Locale::English, "2026-08-01T12:00:00Z", now),
            "Aug 1, 2026"
        );
    }

    #[test]
    fn dates_and_lengths_follow_the_interface_language() {
        let now: jiff::Timestamp = "2026-08-31T12:00:00Z".parse().unwrap();
        assert_eq!(format_date(Locale::Spanish, "2024-01-05"), "5 ene 2024");
        assert_eq!(format_date(Locale::Spanish, "2024-09"), "sept 2024");
        assert_eq!(format_total_ms(Locale::Spanish, 7_980_000), "2 h 13 min");
        assert_eq!(format_episode_ms(Locale::Spanish, 2_280_000), "38 min");
        for (added, expected) in [
            ("2026-08-31T11:59:59Z", "hace 1 segundo"),
            ("2026-08-31T11:58:00Z", "hace 2 minutos"),
            ("2026-08-30T12:00:00Z", "hace 1 día"),
            ("2026-08-17T12:00:00Z", "hace 2 semanas"),
        ] {
            assert_eq!(format_relative_date(Locale::Spanish, added, now), expected);
        }
        // Each language orders the date its own way.
        assert_eq!(format_date(Locale::Swedish, "2024-01-05"), "5 jan. 2024");
        assert_eq!(format_date(Locale::English, "2024-01-05"), "Jan 5, 2024");
        assert_eq!(
            format_relative_date(Locale::Swedish, "2026-08-30T12:00:00Z", now),
            "för 1 dag sedan"
        );
        assert_eq!(format_date(Locale::Turkish, "2024-01-05"), "5 Oca 2024");
        assert_eq!(format_date(Locale::Turkish, "2024-09"), "Eyl 2024");
        assert_eq!(format_total_ms(Locale::Turkish, 7_980_000), "2 sa 13 dk");
        assert_eq!(format_episode_ms(Locale::Turkish, 2_280_000), "38 dk");
        for (added, expected) in [
            ("2026-08-31T11:59:59Z", "1 saniye önce"),
            ("2026-08-31T11:58:00Z", "2 dakika önce"),
            ("2026-08-30T12:00:00Z", "1 gün önce"),
            ("2026-08-17T12:00:00Z", "2 hafta önce"),
        ] {
            assert_eq!(format_relative_date(Locale::Turkish, added, now), expected);
        }
    }

    #[test]
    fn relative_dates_fall_back_for_future_and_invalid_timestamps() {
        let now: jiff::Timestamp = "2026-08-31T12:00:00Z".parse().unwrap();
        assert_eq!(
            format_relative_date(Locale::English, "2026-09-01T12:00:00Z", now),
            "Sep 1, 2026"
        );
        assert_eq!(
            format_relative_date(Locale::English, "not-a-date", now),
            "not-a-date"
        );
    }

    #[test]
    fn uris() {
        assert_eq!(uri_id("spotify:track:abc"), Some("abc"));
        assert_eq!(uri_kind("spotify:playlist:x"), Some("playlist"));
        assert_eq!(
            open_spotify_url("spotify:album:z").as_deref(),
            Some("https://open.spotify.com/album/z")
        );
    }

    #[test]
    fn html_is_stripped() {
        assert_eq!(
            strip_html("Hi <a href=\"x\">there</a> &amp; you"),
            "Hi there & you"
        );
        assert_eq!(strip_html("ONE&#x2F;TWO&#x2F;THREE"), "ONE/TWO/THREE");
    }
}
