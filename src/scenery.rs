//! Scenery photos behind the window, after T3 Pretty's World Scenery.
//!
//! Each photo set starts from the bundled seed pool
//! (`assets/scenery/photos.tsv`) and, with an Unsplash access key, grows
//! from Unsplash searches of the set's curated places
//! (`assets/scenery/catalog.tsv`) every two weeks. Pages without a cover
//! show the set's photo of the day; album, artist, playlist, show, and radio
//! pages show the photo whose colours sit closest to the cover's.
//!
//! The key comes from `SPOTIURGE_UNSPLASH_KEY` (at run or build time) or the
//! `unsplash-access-key` file in the config directory. It is sent only to
//! api.unsplash.com and never logged. Without one, the seed pool still works.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::http::Http;
use crate::images::{ArtColors, ArtLoader, art_colors};

/// The photos behind the window.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum PhotoSet {
    /// The one bundled photo. Sends nothing to Unsplash.
    AlpineLake,
    #[default]
    WorldScenery,
    NightCities,
    DeepForest,
    NightSky,
    GrandBuildings,
}

impl PhotoSet {
    pub const ALL: [PhotoSet; 6] = [
        PhotoSet::WorldScenery,
        PhotoSet::NightCities,
        PhotoSet::DeepForest,
        PhotoSet::NightSky,
        PhotoSet::GrandBuildings,
        PhotoSet::AlpineLake,
    ];

    fn id(self) -> &'static str {
        match self {
            PhotoSet::AlpineLake => "alpine-lake",
            PhotoSet::WorldScenery => "world-scenery",
            PhotoSet::NightCities => "night-cities",
            PhotoSet::DeepForest => "deep-forest",
            PhotoSet::NightSky => "night-sky",
            PhotoSet::GrandBuildings => "grand-buildings",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|set| set.id() == id)
    }

    /// The English label; the settings page translates it.
    pub fn label(self) -> &'static str {
        match self {
            PhotoSet::AlpineLake => "Alpine Lake",
            PhotoSet::WorldScenery => "World Scenery",
            PhotoSet::NightCities => "Night Cities",
            PhotoSet::DeepForest => "Deep Forest",
            PhotoSet::NightSky => "Night Sky",
            PhotoSet::GrandBuildings => "Grand Buildings",
        }
    }
}

/// One Unsplash photo, reduced to what the scenery needs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Photo {
    pub id: String,
    /// The image's path on images.unsplash.com, with Unsplash's `ixid`.
    pub path: String,
    /// The curated place the photo was found for.
    pub place: String,
    pub photographer: String,
    /// The photographer's Unsplash username.
    pub handle: String,
    pub mean: [u8; 3],
    pub accent: [u8; 3],
}

const UTM: &str = "utm_source=Spotiurge&utm_medium=referral";

impl Photo {
    /// The wallpaper: wide enough for a large window under the wash.
    pub fn url(&self) -> String {
        self.sized(2048, 80)
    }

    fn sized(&self, width: u32, quality: u32) -> String {
        let join = if self.path.contains('?') { '&' } else { '?' };
        format!(
            "https://images.unsplash.com{}{join}w={width}&q={quality}&fm=jpg&fit=max",
            self.path
        )
    }

    /// The photographer's page, for the credit Unsplash asks for.
    pub fn photographer_url(&self) -> String {
        if self.handle.is_empty() {
            format!("https://unsplash.com/?{UTM}")
        } else {
            format!("https://unsplash.com/@{}?{UTM}", self.handle)
        }
    }

    /// The download ping Unsplash asks for when a photo is put to use.
    fn download_location(&self) -> String {
        let ixid = self
            .path
            .split_once("ixid=")
            .map(|(_, ixid)| ixid.split('&').next().unwrap_or_default())
            .unwrap_or_default();
        let mut url = format!("https://api.unsplash.com/photos/{}/download", self.id);
        if !ixid.is_empty() {
            url.push_str("?ixid=");
            url.push_str(ixid);
        }
        url
    }
}

fn hex(text: &str) -> Option<[u8; 3]> {
    let text = text.strip_prefix('#')?;
    if text.len() != 6 {
        return None;
    }
    let channel = |at: usize| u8::from_str_radix(text.get(at..at + 2)?, 16).ok();
    Some([channel(0)?, channel(2)?, channel(4)?])
}

fn seed() -> &'static BTreeMap<PhotoSet, Vec<Photo>> {
    static SEED: OnceLock<BTreeMap<PhotoSet, Vec<Photo>>> = OnceLock::new();
    SEED.get_or_init(|| {
        let mut sets: BTreeMap<PhotoSet, Vec<Photo>> = BTreeMap::new();
        for line in include_str!("../assets/scenery/photos.tsv").lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            let [set, id, path, place, photographer, handle, mean, accent] = fields[..] else {
                continue;
            };
            let (Some(set), Some(mean), Some(accent)) =
                (PhotoSet::from_id(set), hex(mean), hex(accent))
            else {
                continue;
            };
            sets.entry(set).or_default().push(Photo {
                id: id.into(),
                path: path.into(),
                place: place.into(),
                photographer: photographer.into(),
                handle: handle.into(),
                mean,
                accent,
            });
        }
        sets
    })
}

/// The set's curated places and their Unsplash search queries.
fn catalog(set: PhotoSet) -> Vec<(&'static str, &'static str)> {
    include_str!("../assets/scenery/catalog.tsv")
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let (id, place, query) = (fields.next()?, fields.next()?, fields.next()?);
            (id == set.id()).then_some((place, query))
        })
        .collect()
}

/// Refresh a set's fetched photos when they are this old (T3 Pretty: 14 days).
const STALE: Duration = Duration::from_secs(14 * 24 * 60 * 60);
/// Places searched per refresh, safe under Unsplash's 50 requests an hour.
const PLACES_PER_REFRESH: usize = 24;
const PHOTOS_PER_PLACE: usize = 8;
/// Fetched photos kept per set, oldest let go first.
const MAX_FETCHED: usize = 768;
const MAX_REGISTERED: usize = 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

/// A chained iterator that knows its length, so the daily pick can index it.
struct Pool<I>(I, usize);

impl<'a, I: Iterator<Item = &'a Photo>> Iterator for Pool<I> {
    type Item = &'a Photo;
    fn next(&mut self) -> Option<&'a Photo> {
        let next = self.0.next();
        self.1 = self.1.saturating_sub(usize::from(next.is_some()));
        next
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<'a, I: Iterator<Item = &'a Photo>> ExactSizeIterator for Pool<I> {}

/// What survives a restart, in the cache directory.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Stored {
    fetched: BTreeMap<PhotoSet, Vec<Photo>>,
    /// Seconds since the epoch of each set's last refresh.
    fetched_at: BTreeMap<PhotoSet, u64>,
    /// The catalog index the next refresh continues from.
    cursor: BTreeMap<PhotoSet, usize>,
    /// Photos whose download ping Unsplash has received.
    registered: Vec<String>,
    /// Seconds since the epoch of the last search run, for any set.
    searched_at: u64,
}

struct Inner {
    http: Http,
    runtime: tokio::runtime::Handle,
    key: Option<String>,
    file: PathBuf,
    online: bool,
    /// The day the photo of the day follows; `None` is today.
    date: Option<jiff::civil::Date>,
    stored: Mutex<Stored>,
    /// Held while `scenery.json` is written and replaced.
    saving: Mutex<()>,
    refreshing: AtomicBool,
    generation: std::sync::atomic::AtomicU64,
    /// Sets refreshed, or found fresh, this run.
    attempted: Mutex<HashSet<PhotoSet>>,
    registering: Mutex<HashSet<String>>,
}

/// The photo pools, shared with the runtime that refreshes them.
#[derive(Clone)]
pub struct Scenery {
    inner: Arc<Inner>,
}

impl Scenery {
    /// `online` false (demo, tests) shows only the bundled photo and sends
    /// nothing.
    pub fn new(art: &ArtLoader, config_dir: PathBuf, cache_dir: PathBuf, online: bool) -> Self {
        let (http, runtime) = art.network();
        let file = cache_dir.join("scenery.json");
        let stored = if online {
            std::fs::read(&file)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_default()
        } else {
            Stored::default()
        };
        Self {
            inner: Arc::new(Inner {
                http,
                runtime,
                key: online.then(|| access_key(&config_dir)).flatten(),
                file,
                online,
                date: None,
                stored: Mutex::new(stored),
                saving: Mutex::new(()),
                refreshing: AtomicBool::new(false),
                generation: std::sync::atomic::AtomicU64::new(0),
                attempted: Mutex::new(HashSet::new()),
                registering: Mutex::new(HashSet::new()),
            }),
        }
    }

    /// Demo mode: the seed pool on a fixed day, with no Unsplash key and
    /// nothing saved, so captures repeat.
    pub fn demo(art: &ArtLoader) -> Self {
        let (http, runtime) = art.network();
        Self {
            inner: Arc::new(Inner {
                http,
                runtime,
                key: None,
                file: PathBuf::new(),
                online: true,
                date: Some(jiff::civil::date(2026, 10, 8)),
                stored: Mutex::new(Stored::default()),
                saving: Mutex::new(()),
                refreshing: AtomicBool::new(false),
                generation: std::sync::atomic::AtomicU64::new(0),
                attempted: Mutex::new(HashSet::new()),
                registering: Mutex::new(HashSet::new()),
            }),
        }
    }

    /// The day the photo of the day is for.
    pub fn today(&self) -> jiff::civil::Date {
        self.inner.date.unwrap_or_else(|| jiff::Zoned::now().date())
    }

    /// The set's photo of the day, the same all day.
    pub fn daily(&self, set: PhotoSet, date: jiff::civil::Date) -> Option<Photo> {
        self.with_pool(set, |pool| {
            let index = stable_hash(&format!("daily|{date}")) as usize % pool.len().max(1);
            pool.nth(index).cloned()
        })
    }

    /// The set's photo whose colours sit closest to a cover's.
    pub fn matching(&self, set: PhotoSet, cover: ArtColors) -> Option<Photo> {
        self.with_pool(set, |pool| best_match(pool, cover).cloned())
    }

    /// Changes whenever a refresh adds photos, so a cached pick can renew.
    pub fn generation(&self) -> u64 {
        self.inner.generation.load(Ordering::Acquire)
    }

    /// The seed pool, then this set's fetched photos.
    fn with_pool<R>(
        &self,
        set: PhotoSet,
        f: impl FnOnce(&mut dyn ExactSizeIterator<Item = &Photo>) -> Option<R>,
    ) -> Option<R> {
        if !self.inner.online || set == PhotoSet::AlpineLake {
            return None;
        }
        let stored = self.inner.stored.lock().unwrap_or_else(|p| p.into_inner());
        let seed = seed().get(&set).map(Vec::as_slice).unwrap_or_default();
        let fetched = stored
            .fetched
            .get(&set)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut pool = Pool(seed.iter().chain(fetched), seed.len() + fetched.len());
        f(&mut pool)
    }

    /// Searches Unsplash for more of the set's places once its photos are
    /// two weeks old. A fresh install starts that clock on the seed pool.
    pub fn refresh_if_stale(&self, set: PhotoSet, ctx: &egui::Context) {
        let inner = &self.inner;
        if set == PhotoSet::AlpineLake || inner.key.is_none() {
            return;
        }
        let mut attempted = inner.attempted.lock().unwrap_or_else(|p| p.into_inner());
        if attempted.contains(&set) {
            return;
        }
        let now = unix_now();
        let mut stored = inner.stored.lock().unwrap_or_else(|p| p.into_inner());
        match stored.fetched_at.get(&set) {
            Some(&at) if now.saturating_sub(at) < STALE.as_secs() => {
                attempted.insert(set);
                return;
            }
            None if seed().get(&set).is_some_and(|seed| seed.len() >= 50) => {
                attempted.insert(set);
                stored.fetched_at.insert(set, now);
                drop(stored);
                self.save();
                return;
            }
            _ => {}
        }
        // A stale set waits, checking again on later frames, while the proxy
        // is still being restored, while another set searches, and for an
        // hour after any search, so all sets together stay under Unsplash's
        // 50 requests an hour.
        if inner.http.client().is_err()
            || now.saturating_sub(stored.searched_at) < 60 * 60
            || inner.refreshing.swap(true, Ordering::AcqRel)
        {
            return;
        }
        attempted.insert(set);
        stored.searched_at = now;
        drop((stored, attempted));
        self.save();
        let this = self.clone();
        let ctx = ctx.clone();
        inner.runtime.spawn(async move {
            this.refresh(set).await;
            this.inner.refreshing.store(false, Ordering::Release);
            ctx.request_repaint();
        });
    }

    async fn refresh(&self, set: PhotoSet) {
        let (Some(key), Ok(client)) = (self.inner.key.clone(), self.inner.http.client()) else {
            return;
        };
        let places = catalog(set);
        if places.is_empty() {
            return;
        }
        let start = self
            .inner
            .stored
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .cursor
            .get(&set)
            .copied()
            .unwrap_or(0)
            % places.len();
        let mut searches = tokio::task::JoinSet::new();
        for offset in 0..PLACES_PER_REFRESH.min(places.len()) {
            let (place, query) = places[(start + offset) % places.len()];
            let (client, key) = (client.clone(), key.clone());
            searches.spawn(async move { search(&client, &key, place, query).await });
        }
        let mut fetched = Vec::new();
        let mut answered = false;
        while let Some(result) = searches.join_next().await {
            if let Ok(Ok(photos)) = result {
                answered = true;
                fetched.extend(photos);
            }
        }
        if !answered {
            log::warn!("scenery: Unsplash searches for {} failed", set.id());
            return;
        }
        // The seed pool already holds its own photos.
        let seeded: HashSet<&str> = seed()
            .get(&set)
            .into_iter()
            .flatten()
            .map(|photo| photo.id.as_str())
            .collect();
        fetched.retain(|photo| !seeded.contains(photo.id.as_str()));
        {
            let mut stored = self.inner.stored.lock().unwrap_or_else(|p| p.into_inner());
            let pool = stored.fetched.entry(set).or_default();
            pool.retain(|old| !fetched.iter().any(|new| new.id == old.id));
            pool.extend(fetched);
            let excess = pool.len().saturating_sub(MAX_FETCHED);
            pool.drain(..excess);
            stored.fetched_at.insert(set, unix_now());
            stored
                .cursor
                .insert(set, (start + PLACES_PER_REFRESH) % places.len());
        }
        self.inner.generation.fetch_add(1, Ordering::AcqRel);
        self.save();
    }

    /// Tells Unsplash a photo is shown, once per photo, as its guidelines ask.
    pub fn register(&self, photo: &Photo) {
        let inner = &self.inner;
        let Some(key) = inner.key.clone() else {
            return;
        };
        if inner
            .stored
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .registered
            .contains(&photo.id)
            || !inner
                .registering
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .insert(photo.id.clone())
        {
            return;
        }
        let (this, id, url) = (self.clone(), photo.id.clone(), photo.download_location());
        inner.runtime.spawn(async move {
            let pinged = match this.inner.http.client() {
                Ok(client) => client
                    .get(&url)
                    .header("Authorization", format!("Client-ID {key}"))
                    .header("Accept-Version", "v1")
                    .timeout(REQUEST_TIMEOUT)
                    .send()
                    .await
                    .is_ok_and(|response| response.status().is_success()),
                Err(_) => false,
            };
            if pinged {
                let mut stored = this.inner.stored.lock().unwrap_or_else(|p| p.into_inner());
                stored.registered.push(id.clone());
                let excess = stored.registered.len().saturating_sub(MAX_REGISTERED);
                stored.registered.drain(..excess);
                drop(stored);
                this.save();
            }
            this.inner
                .registering
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .remove(&id);
        });
    }

    fn save(&self) {
        if self.inner.file.as_os_str().is_empty() {
            return;
        }
        let _saving = self.inner.saving.lock().unwrap_or_else(|p| p.into_inner());
        let text = {
            let stored = self.inner.stored.lock().unwrap_or_else(|p| p.into_inner());
            match serde_json::to_vec(&*stored) {
                Ok(text) => text,
                Err(_) => return,
            }
        };
        let file = &self.inner.file;
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let temporary = file.with_extension("json.tmp");
        if let Err(error) = std::fs::write(&temporary, text)
            .and_then(|()| crate::util::replace_file(&temporary, file))
        {
            log::warn!("unable to save scenery to {}: {error}", file.display());
        }
    }
}

/// The access key, from the environment, the build, or the config file.
fn access_key(config_dir: &std::path::Path) -> Option<String> {
    let valid = |key: &str| {
        let key = key.trim();
        (!key.is_empty()
            && key.len() <= 256
            && key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'))
        .then(|| key.to_string())
    };
    std::env::var("SPOTIURGE_UNSPLASH_KEY")
        .ok()
        .and_then(|key| valid(&key))
        .or_else(|| option_env!("SPOTIURGE_UNSPLASH_KEY").and_then(valid))
        .or_else(|| {
            std::fs::read_to_string(config_dir.join("unsplash-access-key"))
                .ok()
                .and_then(|key| valid(&key))
        })
}

/// One page of Unsplash results for a place, measured for matching.
async fn search(
    client: &reqwest::Client,
    key: &str,
    place: &str,
    query: &str,
) -> Result<Vec<Photo>, String> {
    let response = client
        .get("https://api.unsplash.com/search/photos")
        .query(&[
            ("query", query),
            ("per_page", &PHOTOS_PER_PLACE.to_string()),
            ("orientation", "landscape"),
            ("content_filter", "high"),
        ])
        .header("Authorization", format!("Client-ID {key}"))
        .header("Accept-Version", "v1")
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|error| error.without_url().to_string())?;
    if !response.status().is_success() {
        return Err(format!("status {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| error.without_url().to_string())?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err("response too large".into());
    }
    let body: SearchBody = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    let mut photos = Vec::new();
    for result in body.results {
        let Some(photo) = result.into_photo(place) else {
            continue;
        };
        // Measure the photo as build-photos.py measures the seed pool.
        let thumbnail = client
            .get(photo.sized(96, 70))
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await
            .and_then(|response| response.error_for_status());
        let Ok(thumbnail) = thumbnail else { continue };
        let Ok(bytes) = thumbnail.bytes().await else {
            continue;
        };
        let colors = tokio::task::spawn_blocking(move || art_colors(&bytes))
            .await
            .ok()
            .flatten();
        if let Some(colors) = colors {
            photos.push(Photo {
                mean: colors.mean,
                accent: colors.accent,
                ..photo
            });
        }
    }
    Ok(photos)
}

#[derive(Deserialize)]
struct SearchBody {
    #[serde(default)]
    results: Vec<SearchPhoto>,
}

#[derive(Deserialize)]
struct SearchPhoto {
    id: String,
    urls: SearchUrls,
    user: SearchUser,
    #[serde(default)]
    premium: Option<bool>,
    #[serde(default)]
    plus: Option<bool>,
}

#[derive(Deserialize)]
struct SearchUrls {
    raw: String,
}

#[derive(Deserialize)]
struct SearchUser {
    name: String,
    #[serde(default)]
    username: String,
}

impl SearchPhoto {
    /// A free photo hosted where the scenery loads images from, unmeasured.
    fn into_photo(self, place: &str) -> Option<Photo> {
        if self.premium == Some(true) || self.plus == Some(true) {
            return None;
        }
        let path = self.urls.raw.strip_prefix("https://images.unsplash.com")?;
        let path = match path.split_once('?') {
            Some((path, query)) => match query.split('&').find(|part| part.starts_with("ixid=")) {
                Some(ixid) => format!("{path}?{ixid}"),
                None => path.to_string(),
            },
            None => path.to_string(),
        };
        let valid = |text: &str| !text.is_empty() && text.len() <= 256;
        if !path.starts_with('/') || path.contains(['#', ' ']) || !valid(&self.id) {
            return None;
        }
        let handle = self.user.username;
        let handle_ok = handle
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
        Some(Photo {
            id: self.id,
            path,
            place: place.to_string(),
            photographer: self
                .user
                .name
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
            handle: if handle_ok { handle } else { String::new() },
            mean: [0; 3],
            accent: [0; 3],
        })
        .filter(|photo| valid(&photo.photographer))
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

/// FNV-1a, so a day picks the same photo on every run and platform.
fn stable_hash(text: &str) -> u64 {
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Oklab, where straight-line distance tracks how different colours look.
fn oklab([r, g, b]: [u8; 3]) -> [f32; 3] {
    let linear = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (linear(r), linear(g), linear(b));
    let l = (0.412_221_5 * r + 0.536_332_5 * g + 0.051_445_99 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

fn distance(a: [u8; 3], b: [u8; 3]) -> f32 {
    let (a, b) = (oklab(a), oklab(b));
    a.iter().zip(b).map(|(a, b)| (a - b) * (a - b)).sum()
}

/// The photo nearest the cover in Oklab: its overall tone (mean) and its
/// most vivid colour (accent) both count, so a dark cover with a red accent
/// lands on a dark photo with red in it.
// ponytail: two colours per image; compare small palettes if matches feel coarse.
fn best_match<'a>(pool: impl Iterator<Item = &'a Photo>, cover: ArtColors) -> Option<&'a Photo> {
    pool.min_by(|a, b| {
        let score =
            |photo: &Photo| distance(photo.mean, cover.mean) + distance(photo.accent, cover.accent);
        score(a).total_cmp(&score(b))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn photo(id: &str, mean: [u8; 3], accent: [u8; 3]) -> Photo {
        Photo {
            id: id.into(),
            path: format!("/photo-{id}?ixid=abc"),
            place: "Somewhere".into(),
            photographer: "Someone".into(),
            handle: "someone".into(),
            mean,
            accent,
        }
    }

    #[test]
    fn the_bundled_seed_pool_fills_every_photo_set() {
        for set in PhotoSet::ALL {
            let count = seed().get(&set).map_or(0, Vec::len);
            if set == PhotoSet::AlpineLake {
                assert_eq!(count, 0);
            } else {
                assert!(count >= 100, "{set:?} has {count} photos");
                assert!(!catalog(set).is_empty(), "{set:?} has no places");
            }
        }
    }

    #[test]
    fn a_cover_matches_the_photo_closest_in_tone_and_colour() {
        let pool = [
            photo("night-red", [30, 20, 20], [200, 30, 40]),
            photo("night-blue", [20, 25, 40], [40, 70, 200]),
            photo("day-red", [220, 200, 190], [210, 50, 50]),
        ];
        let dark_red = ArtColors {
            mean: [25, 15, 15],
            accent: [190, 20, 30],
        };
        assert_eq!(best_match(pool.iter(), dark_red).unwrap().id, "night-red");
        let pale_red = ArtColors {
            mean: [230, 210, 200],
            accent: [220, 60, 60],
        };
        assert_eq!(best_match(pool.iter(), pale_red).unwrap().id, "day-red");
        let dark_blue = ArtColors {
            mean: [20, 20, 35],
            accent: [30, 60, 220],
        };
        assert_eq!(best_match(pool.iter(), dark_blue).unwrap().id, "night-blue");
    }

    #[test]
    fn photo_urls_keep_unsplash_tracking_and_credit() {
        let photo = photo("x", [0; 3], [0; 3]);
        assert_eq!(
            photo.url(),
            "https://images.unsplash.com/photo-x?ixid=abc&w=2048&q=80&fm=jpg&fit=max"
        );
        assert_eq!(
            photo.download_location(),
            "https://api.unsplash.com/photos/x/download?ixid=abc"
        );
        assert_eq!(
            photo.photographer_url(),
            "https://unsplash.com/@someone?utm_source=Spotiurge&utm_medium=referral"
        );
    }

    #[test]
    fn search_results_keep_only_free_unsplash_hosted_photos() {
        let body: SearchBody = serde_json::from_str(
            r#"{"results":[
                {"id":"a","urls":{"raw":"https://images.unsplash.com/photo-a?ixid=Q&ixlib=rb"},
                 "user":{"name":"Ann  Lee","username":"ann"}},
                {"id":"b","premium":true,"urls":{"raw":"https://images.unsplash.com/photo-b"},
                 "user":{"name":"Bo","username":"bo"}},
                {"id":"c","urls":{"raw":"https://evil.example/photo-c"},
                 "user":{"name":"Cy","username":"cy"}}
            ]}"#,
        )
        .unwrap();
        let photos: Vec<Photo> = body
            .results
            .into_iter()
            .filter_map(|result| result.into_photo("Kyoto, Japan"))
            .collect();
        assert_eq!(photos.len(), 1);
        assert_eq!(photos[0].path, "/photo-a?ixid=Q");
        assert_eq!(photos[0].photographer, "Ann Lee");
        assert_eq!(photos[0].place, "Kyoto, Japan");
    }

    #[test]
    fn the_photo_of_the_day_holds_all_day_and_offline_shows_none() {
        let dir = std::env::temp_dir().join(format!("spotiurge-scenery-{}", std::process::id()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let art = ArtLoader::new(Http::default(), runtime.handle().clone(), dir.join("art"));
        let date = jiff::civil::date(2026, 10, 8);
        let offline = Scenery::new(&art, dir.clone(), dir.clone(), false);
        assert!(offline.daily(PhotoSet::WorldScenery, date).is_none());
        let online = Scenery::new(&art, dir.clone(), dir.clone(), true);
        let first = online.daily(PhotoSet::WorldScenery, date).unwrap();
        assert_eq!(
            online.daily(PhotoSet::WorldScenery, date),
            Some(first.clone())
        );
        assert_ne!(
            online.daily(PhotoSet::WorldScenery, date.tomorrow().unwrap()),
            Some(first)
        );
        assert!(online.daily(PhotoSet::AlpineLake, date).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stale_set_waits_an_hour_after_any_search() {
        let dir =
            std::env::temp_dir().join(format!("spotiurge-scenery-gate-{}", std::process::id()));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let art = ArtLoader::new(Http::default(), runtime.handle().clone(), dir.join("art"));
        let base = Scenery::new(&art, dir.clone(), dir.clone(), false);
        let scenery = Scenery {
            inner: Arc::new(Inner {
                key: Some("test-key".into()),
                file: dir.join("scenery.json"),
                online: true,
                ..Arc::into_inner(base.inner).unwrap()
            }),
        };
        let now = unix_now();
        {
            let mut stored = scenery.inner.stored.lock().unwrap();
            let stale = now - STALE.as_secs() - 1;
            stored.fetched_at.insert(PhotoSet::NightSky, stale);
            stored.fetched_at.insert(PhotoSet::DeepForest, now);
            // Another set searched a minute ago.
            stored.searched_at = now - 60;
        }
        let ctx = egui::Context::default();
        scenery.refresh_if_stale(PhotoSet::NightSky, &ctx);
        assert!(!scenery.inner.refreshing.load(Ordering::Acquire));
        let attempted = scenery.inner.attempted.lock().unwrap().clone();
        assert!(
            !attempted.contains(&PhotoSet::NightSky),
            "it tries again later"
        );
        scenery.refresh_if_stale(PhotoSet::DeepForest, &ctx);
        assert!(
            scenery
                .inner
                .attempted
                .lock()
                .unwrap()
                .contains(&PhotoSet::DeepForest)
        );
        assert!(!scenery.inner.refreshing.load(Ordering::Acquire));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
