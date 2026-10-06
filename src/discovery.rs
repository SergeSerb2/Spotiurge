//! Personal discovery state. No Spotify grants or audio enter this document.
//!
//! Per-record logical clocks make offline edits merge independently of wall
//! clocks. The cloud stores only the document; writer identity stays local.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_PROMPT_BYTES: usize = 4000;

/// Keep editor input within the wire/storage limit without splitting UTF-8.
pub fn limit_prompt(text: &mut String) {
    if text.len() > MAX_PROMPT_BYTES {
        let mut end = MAX_PROMPT_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stamp {
    pub counter: u64,
    pub device: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub stamp: Stamp,
    /// `None` is a durable tombstone, not an absent record.
    pub value: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Taste {
        text: String,
    },
    Feedback {
        uri: String,
        title: String,
        artist: String,
        rating: Rating,
    },
    Mix {
        title: String,
        uris: Vec<String>,
    },
    History {
        prompt: String,
        suggestions: Vec<Suggestion>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    Love,
    Less,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploration {
    Familiar,
    #[default]
    Balanced,
    Adventurous,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub automatic: bool,
    pub exploration: Exploration,
    pub refreshed_at: Option<u64>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            automatic: true,
            exploration: Exploration::Balanced,
            refreshed_at: None,
        }
    }
}

#[derive(Default)]
pub struct AutoRecommendations {
    pub failures: u32,
    pub suspended: bool,
    retry_at: Option<std::time::Instant>,
    changed_at: Option<std::time::Instant>,
    last_attempt: Option<std::time::Instant>,
}

impl AutoRecommendations {
    pub fn changed(&mut self, now: std::time::Instant, exploration: bool) {
        let delay = std::time::Duration::from_millis(if exploration { 1500 } else { 45_000 });
        let mut due = now + delay;
        if !exploration && let Some(last) = self.last_attempt {
            due = due.max(last + std::time::Duration::from_secs(600));
        }
        self.changed_at = Some(due);
    }

    pub fn due(
        &self,
        now: std::time::Instant,
        wall_now: u64,
        preferences: &Preferences,
        enabled: bool,
        has_inputs: bool,
        empty: bool,
    ) -> Option<std::time::Duration> {
        if !enabled || !has_inputs || !preferences.automatic || self.suspended {
            return None;
        }
        if let Some(retry) = self.retry_at {
            return Some(retry.saturating_duration_since(now));
        }
        if let Some(changed) = self.changed_at {
            return Some(changed.saturating_duration_since(now));
        }
        if empty && self.last_attempt.is_none() {
            return Some(std::time::Duration::ZERO);
        }
        let refreshed = preferences
            .refreshed_at
            .filter(|time| *time <= wall_now.saturating_add(43_200))
            .unwrap_or(0);
        Some(std::time::Duration::from_secs(
            refreshed.saturating_add(43_200).saturating_sub(wall_now),
        ))
    }

    pub fn attempted(&mut self, now: std::time::Instant) {
        self.last_attempt = Some(now);
        self.changed_at = None;
        self.retry_at = None;
    }

    pub fn failed(&mut self, kind: RecommendationErrorKind, now: std::time::Instant) {
        if kind == RecommendationErrorKind::Pairing {
            self.suspended = true;
            return;
        }
        self.failures = self.failures.saturating_add(1).min(10);
        let seconds = (600_u64 << (self.failures - 1)).min(21_600);
        self.retry_at = Some(now + std::time::Duration::from_secs(seconds));
    }

    pub fn rearm(&mut self) {
        self.failures = 0;
        self.suspended = false;
        self.retry_at = None;
    }
}

pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |time| time.as_secs())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CatalogueOutcome {
    #[default]
    Complete,
    TimedOut,
    RateLimited,
    QuotaExhausted,
    Unavailable,
    SignInNeeded,
}

#[derive(Clone, Debug)]
pub struct ResolvedDiscovery {
    pub picks: Vec<Pick>,
    pub outcome: CatalogueOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecommendationErrorKind {
    Pairing,
    Busy,
    RateLimited,
    Unavailable,
    InvalidResponse,
}

#[derive(Clone, Debug)]
pub struct RecommendationError {
    pub kind: RecommendationErrorKind,
    pub message: String,
}

impl RecommendationError {
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self {
            kind: RecommendationErrorKind::Unavailable,
            message: message.into(),
        }
    }
}

impl From<String> for RecommendationError {
    fn from(message: String) -> Self {
        Self::unavailable(message)
    }
}

impl From<&str> for RecommendationError {
    fn from(message: &str) -> Self {
        Self::unavailable(message)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suggestion {
    pub title: String,
    pub artist: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u32,
    pub records: BTreeMap<String, Record>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            records: BTreeMap::new(),
        }
    }
}

impl Document {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.records.len() > 2000 {
            return Err(
                "Unsupported or oversized discovery state. Your local state is preserved.".into(),
            );
        }
        for (key, record) in &self.records {
            if key.len() > 200
                || record.stamp.device.len() != 32
                || !record
                    .stamp
                    .device
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                || record.stamp.counter == 0
                || record.stamp.counter == u64::MAX
            {
                return Err("Invalid discovery record.".into());
            }
            let valid = match &record.value {
                Some(Value::Taste { text }) => key == "taste" && text.len() <= MAX_PROMPT_BYTES,
                Some(Value::Feedback {
                    uri, title, artist, ..
                }) => {
                    key == &format!("feedback:{uri}")
                        && spotify_track(uri)
                        && title.len() <= 300
                        && artist.len() <= 300
                }
                Some(Value::Mix { title, uris }) => {
                    key.starts_with("mix:")
                        && title.len() <= 300
                        && uris.len() <= 100
                        && uris.iter().all(|u| spotify_track(u))
                }
                Some(Value::History {
                    prompt,
                    suggestions,
                }) => {
                    key.starts_with("history:")
                        && prompt.len() <= MAX_PROMPT_BYTES
                        && valid_suggestions(suggestions)
                }
                None => {
                    key == "taste"
                        || ["feedback:", "mix:", "history:"]
                            .iter()
                            .any(|p| key.starts_with(p))
                }
            };
            if !valid {
                return Err("Invalid discovery record content.".into());
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "Cannot encode discovery state.")?
            .len()
            > MAX_BYTES
        {
            return Err(
                "Discovery storage is full. Export and remove old mixes or history.".into(),
            );
        }
        Ok(())
    }

    /// Validate the entire remote snapshot before touching local records.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        other.validate()?;
        let mut merged = self.clone();
        for (key, remote) in &other.records {
            match merged.records.get(key) {
                Some(local) if local.stamp == remote.stamp && local.value != remote.value => {
                    return Err("Discovery clock collision. Your local state is preserved.".into());
                }
                Some(local) if local.stamp >= remote.stamp => {}
                _ => {
                    merged.records.insert(key.clone(), remote.clone());
                }
            }
        }
        merged.validate()?;
        *self = merged;
        Ok(())
    }

    pub fn taste(&self) -> &str {
        match self.records.get("taste").and_then(|r| r.value.as_ref()) {
            Some(Value::Taste { text }) => text,
            _ => "",
        }
    }

    pub fn has_inputs(&self) -> bool {
        !self.taste().trim().is_empty()
            || self
                .records
                .values()
                .any(|record| matches!(record.value, Some(Value::Feedback { .. })))
    }

    pub fn rating(&self, uri: &str) -> Option<Rating> {
        match self
            .records
            .get(&format!("feedback:{uri}"))
            .and_then(|r| r.value.as_ref())
        {
            Some(Value::Feedback { rating, .. }) => Some(*rating),
            _ => None,
        }
    }

    pub fn feedback(&self) -> Vec<&Value> {
        let mut records = self
            .records
            .values()
            .filter(|r| matches!(r.value, Some(Value::Feedback { .. })))
            .collect::<Vec<_>>();
        records.sort_by(|a, b| b.stamp.cmp(&a.stamp));
        records
            .into_iter()
            .take(100)
            .filter_map(|r| r.value.as_ref())
            .collect()
    }

    /// Only the newest ten references, without cloning stored prompts/results.
    pub fn recent_history(&self) -> Vec<(&str, &Record)> {
        let mut newest: Vec<(&str, &Record)> = Vec::with_capacity(11);
        for (key, record) in &self.records {
            if !matches!(record.value, Some(Value::History { .. })) {
                continue;
            }
            let position = newest.partition_point(|(held_key, held)| {
                (&held.stamp, *held_key) > (&record.stamp, key.as_str())
            });
            if position < 10 {
                newest.insert(position, (key, record));
                newest.truncate(10);
            }
        }
        newest
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replica {
    pub device: String,
    pub document: Document,
    /// Local catalogue matches, excluded from the cloud document and AI input.
    #[serde(default)]
    pub cached_picks: Vec<Pick>,
}

impl Default for Replica {
    fn default() -> Self {
        Self {
            device: format!("{:032x}", rand::random::<u128>()),
            document: Document::default(),
            cached_picks: Vec::new(),
        }
    }
}

impl Replica {
    /// Merge an acknowledged sync without undoing edits made after its snapshot.
    pub fn merge_synced(
        &mut self,
        remote: &Document,
        snapshot: Option<&Document>,
    ) -> Result<(), String> {
        let changed = snapshot
            .map(|snapshot| {
                self.document
                    .records
                    .iter()
                    .filter(|(key, record)| snapshot.records.get(*key) != Some(*record))
                    .map(|(key, record)| (key.clone(), record.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut merged = self.clone();
        merged.document.merge(remote)?;
        for (key, local) in changed {
            if merged.document.records.get(&key) != Some(&local) {
                // Incorporate the remote clock first, then express the newer
                // local intent above it. Tombstones are edits as well.
                merged.edit(key, local.value)?;
            }
        }
        self.document = merged.document;
        Ok(())
    }

    pub fn edit(&mut self, key: String, value: Option<Value>) -> Result<(), String> {
        let counter = self
            .document
            .records
            .values()
            .map(|r| r.stamp.counter)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Discovery clock is exhausted.")?;
        let mut candidate = self.document.clone();
        candidate.records.insert(
            key,
            Record {
                stamp: Stamp {
                    counter,
                    device: self.device.clone(),
                },
                value,
            },
        );
        candidate.validate()?;
        self.document = candidate;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(_) => {
                return Err(
                    "Cannot read discovery state. Fix its file permissions and restart.".into(),
                );
            }
        };
        if bytes.len() > MAX_BYTES * 2 {
            return Err("Discovery state is too large.".into());
        }
        let mut replica: Self = serde_json::from_slice(&bytes)
            .map_err(|_| "Cannot read discovery state. The original file is preserved.")?;
        replica.document.validate()?;
        if replica.cached_picks.len() > 12 {
            return Err("Invalid cached discovery results.".into());
        }
        if replica.device.len() != 32
            || !replica
                .device
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err("Invalid discovery installation identity.".into());
        }
        // Backups/profile copies must not keep a live writer's identity. Old
        // record stamps remain intact; the next edit uses a fresh writer and
        // advances past the largest persisted logical counter.
        replica.device = format!("{:032x}", rand::random::<u128>());
        Ok(replica)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        self.document.validate()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| "Cannot create discovery storage.")?;
        }
        let temporary = path.with_extension("json.pending");
        let bytes =
            serde_json::to_vec_pretty(self).map_err(|_| "Cannot encode discovery state.")?;
        if bytes.len() > MAX_BYTES * 2 {
            return Err("Local discovery storage is full.".into());
        }
        std::fs::write(&temporary, bytes)
            .and_then(|()| crate::util::replace_file(&temporary, path))
            .map_err(|_| "Cannot save discovery state. Your edits remain in memory.".into())
    }
}

pub fn spotify_track(uri: &str) -> bool {
    uri.strip_prefix("spotify:track:")
        .is_some_and(|id| id.len() == 22 && id.bytes().all(|c| c.is_ascii_alphanumeric()))
}

pub fn valid_suggestions(suggestions: &[Suggestion]) -> bool {
    !suggestions.is_empty()
        && suggestions.len() <= 12
        && suggestions.iter().all(|s| {
            !s.title.trim().is_empty()
                && !s.artist.trim().is_empty()
                && s.title.len() <= 300
                && s.artist.len() <= 300
                && s.reason.len() <= 600
        })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pick {
    pub suggestion: Suggestion,
    pub track: Option<crate::api::models::Track>,
    #[serde(default)]
    pub checked: bool,
}

/// UI state is separate from the synchronized document and secrets.
#[derive(Default)]
pub struct Discovery {
    pub replica: Replica,
    pub ready: bool,
    pub draft: String,
    pub picks: Vec<Pick>,
    pub busy: bool,
    pub syncing: bool,
    pub sync_snapshot: Option<Document>,
    pub request: u64,
    /// A superseded request still occupies the worker until its result arrives.
    pub in_flight_request: Option<u64>,
    pub status: String,
    pub dirty: bool,
    pub catalogue: CatalogueOutcome,
    pub retry_after: Option<std::time::Instant>,
    pub editing_taste: bool,
    pub show_history: bool,
    pub automatic: AutoRecommendations,
    pub last_error: Option<RecommendationErrorKind>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_recommendations_use_freshness_and_never_run_without_inputs() {
        use std::time::{Duration, Instant};
        let now = Instant::now();
        let scheduler = AutoRecommendations::default();
        let preferences = Preferences {
            refreshed_at: Some(100_000),
            ..Default::default()
        };
        assert_eq!(
            scheduler.due(now, 100_001, &preferences, true, true, false),
            Some(Duration::from_secs(43_199))
        );
        assert_eq!(
            scheduler.due(now, 143_200, &preferences, true, true, false),
            Some(Duration::ZERO)
        );
        assert_eq!(
            scheduler.due(now, 100_001, &preferences, true, true, true),
            Some(Duration::ZERO)
        );
        assert_eq!(
            scheduler.due(now, 143_200, &preferences, false, true, true),
            None
        );
        assert_eq!(
            scheduler.due(now, 143_200, &preferences, true, false, true),
            None
        );
        let disabled = Preferences {
            automatic: false,
            ..preferences
        };
        assert_eq!(
            scheduler.due(now, 143_200, &disabled, true, true, true),
            None
        );
    }

    #[test]
    fn automatic_recommendations_debounce_feedback_and_respect_backoff() {
        use std::time::{Duration, Instant};
        let now = Instant::now();
        let preferences = Preferences::default();
        let mut scheduler = AutoRecommendations::default();
        scheduler.attempted(now);
        scheduler.changed(now + Duration::from_secs(10), false);
        scheduler.changed(now + Duration::from_secs(20), false);
        scheduler.changed(now + Duration::from_secs(30), false);
        assert_eq!(
            scheduler.due(now, 100_000, &preferences, true, true, false),
            Some(Duration::from_secs(600))
        );
        scheduler.changed(now, true);
        assert_eq!(
            scheduler.due(now, 100_000, &preferences, true, true, false),
            Some(Duration::from_millis(1500))
        );
        for seconds in [600, 1200, 2400, 4800, 9600, 19_200, 21_600, 21_600] {
            scheduler.failed(RecommendationErrorKind::RateLimited, now);
            // New feedback cannot override an AI cooldown.
            scheduler.changed(now, true);
            assert_eq!(
                scheduler.due(now, 100_000, &preferences, true, true, false),
                Some(Duration::from_secs(seconds))
            );
            scheduler.attempted(now + Duration::from_secs(seconds));
        }
        scheduler.failed(RecommendationErrorKind::Pairing, now);
        assert_eq!(
            scheduler.due(now, 100_000, &preferences, true, true, false),
            None
        );
        scheduler.rearm();
        assert_eq!(scheduler.failures, 0);
        assert!(!scheduler.suspended);
    }

    #[test]
    fn old_pick_caches_and_partial_preferences_remain_readable() {
        let pick: Pick = serde_json::from_str(
            r#"{"suggestion":{"title":"Song","artist":"Artist","reason":""},"track":null}"#,
        )
        .unwrap();
        assert!(!pick.checked);
        let preferences: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(preferences, Preferences::default());
        let preferences: Preferences =
            serde_json::from_str(r#"{"exploration":"adventurous"}"#).unwrap();
        assert!(preferences.automatic);
        assert_eq!(preferences.exploration, Exploration::Adventurous);
    }

    #[test]
    fn clearing_feedback_converges_and_retains_a_tombstone() {
        let mut a = device('a');
        let mut b = device('b');
        let uri = "spotify:track:0123456789ABCDEFGHIJKL";
        a.edit(
            format!("feedback:{uri}"),
            Some(Value::Feedback {
                uri: uri.into(),
                title: "Song".into(),
                artist: "Artist".into(),
                rating: Rating::Love,
            }),
        )
        .unwrap();
        b.document.merge(&a.document).unwrap();
        b.edit(format!("feedback:{uri}"), None).unwrap();
        a.document.merge(&b.document).unwrap();
        assert_eq!(a.document.rating(uri), None);
        assert!(
            a.document.records[&format!("feedback:{uri}")]
                .value
                .is_none()
        );
        assert_eq!(a.document, b.document);
    }

    fn device(id: char) -> Replica {
        Replica {
            device: id.to_string().repeat(32),
            ..Default::default()
        }
    }

    #[test]
    fn sync_acknowledgment_preserves_later_local_taste_above_the_remote_clock() {
        let mut local = device('a');
        local.edit("taste".into(), taste("before sync")).unwrap();
        let snapshot = local.document.clone();
        let mut remote = device('b');
        for _ in 0..10 {
            remote.edit("taste".into(), taste("remote taste")).unwrap();
        }
        local
            .edit("taste".into(), taste("just saved here"))
            .unwrap();
        local
            .merge_synced(&remote.document, Some(&snapshot))
            .unwrap();
        assert_eq!(local.document.taste(), "just saved here");
        assert!(local.document.records["taste"].stamp.counter > 10);
        remote.document.merge(&local.document).unwrap();
        assert_eq!(remote.document, local.document);
    }

    #[test]
    fn sync_preserves_in_flight_tombstones_and_imports_unrelated_remote_edits() {
        let mut local = device('a');
        local
            .edit(
                "mix:removed".into(),
                Some(Value::Mix {
                    title: "old mix".into(),
                    uris: vec![],
                }),
            )
            .unwrap();
        let snapshot = local.document.clone();
        local.edit("mix:removed".into(), None).unwrap();
        let mut remote = device('b');
        for _ in 0..10 {
            remote
                .edit(
                    "mix:removed".into(),
                    Some(Value::Mix {
                        title: "remote mix".into(),
                        uris: vec![],
                    }),
                )
                .unwrap();
        }
        remote
            .edit("taste".into(), taste("remote preference"))
            .unwrap();
        local
            .merge_synced(&remote.document, Some(&snapshot))
            .unwrap();
        assert_eq!(local.document.taste(), "remote preference");
        assert!(local.document.records["mix:removed"].value.is_none());
        assert!(local.document.records["mix:removed"].stamp.counter > 11);
        remote.document.merge(&local.document).unwrap();
        assert_eq!(remote.document, local.document);

        let saved = local.document.clone();
        let mut invalid = remote.document;
        invalid.records.get_mut("taste").unwrap().stamp.counter = u64::MAX;
        assert!(local.merge_synced(&invalid, Some(&snapshot)).is_err());
        assert_eq!(local.document, saved, "invalid replies cannot partly apply");
    }
    fn taste(text: &str) -> Option<Value> {
        Some(Value::Taste { text: text.into() })
    }

    #[test]
    fn offline_edits_converge_without_wall_clocks() {
        let mut a = device('a');
        let mut b = device('b');
        a.edit("taste".into(), taste("warm strings")).unwrap();
        b.edit("taste".into(), taste("broken rhythms")).unwrap();
        a.edit(
            "mix:morning".into(),
            Some(Value::Mix {
                title: "Morning".into(),
                uris: vec![],
            }),
        )
        .unwrap();
        let original = a.document.clone();
        a.document.merge(&b.document).unwrap();
        b.document.merge(&original).unwrap();
        assert_eq!(a.document, b.document);
        assert_eq!(a.document.taste(), "broken rhythms");
        assert!(a.document.records.contains_key("mix:morning"));
    }

    #[test]
    fn deletion_and_edits_during_sync_survive_stale_responses() {
        let mut a = device('a');
        a.edit(
            "mix:one".into(),
            Some(Value::Mix {
                title: "One".into(),
                uris: vec![],
            }),
        )
        .unwrap();
        let stale = a.document.clone();
        a.edit("mix:one".into(), None).unwrap();
        a.edit("taste".into(), taste("new direction")).unwrap();
        a.document.merge(&stale).unwrap();
        assert!(a.document.records["mix:one"].value.is_none());
        assert_eq!(a.document.taste(), "new direction");
    }

    #[test]
    fn invalid_or_colliding_remote_never_changes_local_state() {
        let mut a = device('a');
        a.edit("taste".into(), taste("mine")).unwrap();
        let before = a.document.clone();
        let mut bad = before.clone();
        bad.records.get_mut("taste").unwrap().value = taste("collision");
        assert!(a.document.merge(&bad).is_err());
        assert_eq!(a.document, before);
        bad.version = 99;
        assert!(a.document.merge(&bad).is_err());
        assert_eq!(a.document, before);
    }

    #[test]
    fn model_output_cannot_supply_playback_uris() {
        assert!(!spotify_track("spotify:track:invented"));
        assert!(!spotify_track("https://example.com/music"));
        assert!(spotify_track("spotify:track:0123456789ABCDEFGHIJKL"));
        assert!(!valid_suggestions(&[]));
    }

    #[test]
    fn multibyte_prompts_fit_taste_and_history_storage() {
        for input in [
            "a".repeat(4001),
            "音".repeat(1334),
            "🎵".repeat(1001),
            format!("{}🎵", "a".repeat(3999)),
        ] {
            let mut text = input.clone();
            limit_prompt(&mut text);
            assert!(text.len() <= MAX_PROMPT_BYTES);
            assert!(input.starts_with(&text));
            let mut replica = device('a');
            replica.edit("taste".into(), taste(&text)).unwrap();
            replica
                .edit(
                    "history:test".into(),
                    Some(Value::History {
                        prompt: text,
                        suggestions: vec![Suggestion {
                            title: "Song".into(),
                            artist: "Artist".into(),
                            reason: String::new(),
                        }],
                    }),
                )
                .unwrap();
        }
    }

    #[test]
    fn copied_profiles_preserve_history_but_get_independent_writers() {
        let path = Path::new("target").join(format!(
            "discovery-copy-{:032x}.json",
            rand::random::<u128>()
        ));
        let mut original = device('a');
        original.edit("taste".into(), taste("original")).unwrap();
        original.save(&path).unwrap();
        let mut a = Replica::load(&path).unwrap();
        let mut b = Replica::load(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(a.document, original.document);
        assert_eq!(b.document, original.document);
        assert_ne!(a.device, original.device);
        assert_ne!(a.device, b.device);
        a.edit("taste".into(), taste("first copy")).unwrap();
        b.edit("taste".into(), taste("second copy")).unwrap();
        assert_eq!(a.document.records["taste"].stamp.counter, 2);
        assert_eq!(b.document.records["taste"].stamp.counter, 2);
        let old_a = a.document.clone();
        a.document.merge(&b.document).unwrap();
        b.document.merge(&old_a).unwrap();
        assert_eq!(a.document, b.document);
    }

    #[test]
    fn history_selects_only_ten_newest_live_records_by_reference() {
        let mut replica = device('a');
        for number in 0..25 {
            replica
                .edit(
                    format!("history:{number:02}"),
                    Some(Value::History {
                        prompt: format!("Prompt {number}"),
                        suggestions: vec![Suggestion {
                            title: "Song".into(),
                            artist: "Artist".into(),
                            reason: String::new(),
                        }],
                    }),
                )
                .unwrap();
        }
        replica.edit("history:24".into(), None).unwrap();
        replica
            .edit("taste".into(), taste("latest non-history record"))
            .unwrap();
        let history = replica.document.recent_history();
        assert_eq!(history.len(), 10);
        assert_eq!(history[0].0, "history:23");
        assert_eq!(history[9].0, "history:14");
        for (key, record) in history {
            assert!(std::ptr::eq(record, &replica.document.records[key]));
        }
    }
}
