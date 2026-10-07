//! Personal discovery state. No Spotify grants or audio enter this document.
//!
//! Per-record logical clocks make offline edits merge independently of wall
//! clocks. The cloud stores only the document; writer identity stays local.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const MAX_BYTES: usize = 1_048_576;
pub const MAX_PROMPT_BYTES: usize = 4000;
/// Live AI history entries per document, matching the newest ten the UI shows.
/// Refreshes reuse the oldest history key once this many keys exist.
pub const HISTORY_LIMIT: usize = 10;
/// New saves reuse a deleted slot or the oldest slot once 100 exist.
pub const MIX_LIMIT: usize = 100;
/// Keep recent ratings and cleared ratings within the shared storage budget.
pub const FEEDBACK_LIMIT: usize = 500;
const FEEDBACK_FLOOR: &str = "feedback:retention";

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

/// Local wall-clock checkpoints, never synchronized or sent to AI. Runtime
/// scheduling remains monotonic; bounded restoration handles clock changes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RecommendationThrottle {
    pub last_attempt_at: Option<u64>,
    pub retry_at: Option<u64>,
    pub changed_at: Option<u64>,
    pub failures: u32,
    pub suspended: bool,
}

impl AutoRecommendations {
    pub fn restore(saved: &RecommendationThrottle, now: std::time::Instant, wall_now: u64) -> Self {
        Self {
            failures: saved.failures.min(10),
            suspended: saved.suspended,
            last_attempt: saved.last_attempt_at.and_then(|last| {
                now.checked_sub(std::time::Duration::from_secs(
                    wall_now.saturating_sub(last).min(600),
                ))
            }),
            retry_at: saved.retry_at.map(|retry| {
                now + std::time::Duration::from_secs(retry.saturating_sub(wall_now).min(21_600))
            }),
            changed_at: saved.changed_at.map(|changed| {
                now + std::time::Duration::from_secs(changed.saturating_sub(wall_now).min(600))
            }),
        }
    }

    pub fn checkpoint(&self, now: std::time::Instant, wall_now: u64) -> RecommendationThrottle {
        let deadline = |time: std::time::Instant| {
            let remaining = time.saturating_duration_since(now);
            wall_now.saturating_add(remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0))
        };
        RecommendationThrottle {
            last_attempt_at: self
                .last_attempt
                .map(|last| wall_now.saturating_sub(now.saturating_duration_since(last).as_secs())),
            retry_at: self.retry_at.map(deadline),
            changed_at: self.changed_at.map(deadline),
            failures: self.failures,
            suspended: self.suspended,
        }
    }

    pub fn retry_after(&self, now: std::time::Instant) -> Option<std::time::Duration> {
        self.retry_at
            .filter(|retry| *retry > now)
            .map(|retry| retry - now)
    }

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
        let freshness = std::time::Duration::from_secs(
            refreshed.saturating_add(43_200).saturating_sub(wall_now),
        );
        Some(self.last_attempt.map_or(freshness, |last| {
            freshness
                .max((last + std::time::Duration::from_secs(600)).saturating_duration_since(now))
        }))
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
        self.validate_records()?;
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

    fn validate_records(&self) -> Result<(), String> {
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
        Ok(())
    }

    /// A single tombstone is the forgetting boundary for all feedback keys.
    /// Retaining it makes compaction converge even with an old offline replica.
    /// Equal-clock cohorts are forgotten together, never split by merge order.
    fn compact_feedback(&mut self) {
        let mut floor = self.records.get(FEEDBACK_FLOOR).map(|r| r.stamp.clone());
        let mut stamps = self
            .records
            .iter()
            .filter(|(key, _)| key.starts_with("feedback:") && key.as_str() != FEEDBACK_FLOOR)
            .map(|(_, record)| record.stamp.clone())
            .collect::<Vec<_>>();
        stamps.sort_unstable_by(|a, b| b.cmp(a));
        if let Some(pruned) = stamps.get(FEEDBACK_LIMIT) {
            floor = Some(floor.map_or_else(|| pruned.clone(), |old| old.max(pruned.clone())));
        }
        if let Some(floor) = floor {
            self.records.retain(|key, record| {
                !key.starts_with("feedback:") || key == FEEDBACK_FLOOR || record.stamp > floor
            });
            self.records.insert(
                FEEDBACK_FLOOR.into(),
                Record {
                    stamp: floor,
                    value: None,
                },
            );
        }
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
        merged.compact_feedback();
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

    /// Compare effective recommendation inputs, including removed feedback.
    /// Compare the same newest 100 ordered ratings the model receives.
    pub fn same_inputs(&self, other: &Self) -> bool {
        self.taste() == other.taste() && self.feedback() == other.feedback()
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
    /// Unsynced local ratings, persisted separately from the cloud document.
    /// Only exact current stamps survive; acknowledged or forgotten edits leave.
    #[serde(default)]
    pub pending_feedback: BTreeMap<String, Stamp>,
    /// Restart-safe per-installation AI limits, outside the cloud document.
    #[serde(default)]
    pub recommendation_throttle: RecommendationThrottle,
}

impl Default for Replica {
    fn default() -> Self {
        Self {
            device: format!("{:032x}", rand::random::<u128>()),
            document: Document::default(),
            cached_picks: Vec::new(),
            pending_feedback: BTreeMap::new(),
            recommendation_throttle: RecommendationThrottle::default(),
        }
    }
}

impl Replica {
    /// Reuse slots rather than accumulating a permanent record on every save.
    /// Existing legacy mixes remain readable and can be removed individually.
    pub fn save_mix(&mut self, title: String, uris: Vec<String>) -> Result<(), String> {
        let mixes = self
            .document
            .records
            .iter()
            .filter(|(key, _)| key.starts_with("mix:"))
            .collect::<Vec<_>>();
        let target = mixes
            .iter()
            .filter(|(_, record)| record.value.is_none())
            .min_by_key(|(key, record)| (&record.stamp, *key))
            .or_else(|| {
                if mixes.len() >= MIX_LIMIT {
                    mixes
                        .iter()
                        .min_by_key(|(key, record)| (&record.stamp, *key))
                } else {
                    None
                }
            })
            .map(|(key, _)| (*key).clone())
            .unwrap_or_else(|| format!("mix:{:032x}", rand::random::<u128>()));
        self.edit(target, Some(Value::Mix { title, uris }))
    }

    /// Import a cloud clock before uploading. A new offline rating may be
    /// below its retention floor; distinguish unsent intent from old replicas.
    pub fn merge_for_sync(&mut self, remote: &Document) -> Result<(), String> {
        let pending = self
            .pending_feedback
            .iter()
            .filter_map(|(key, stamp)| {
                self.document
                    .records
                    .get(key)
                    .filter(|record| &record.stamp == stamp)
                    .map(|record| (key.clone(), record.value.clone()))
            })
            .collect::<Vec<_>>();
        let mut merged = self.clone();
        merged.document.merge(remote)?;
        let lost = pending
            .into_iter()
            .filter(|(key, _)| !merged.document.records.contains_key(key))
            .collect::<Vec<_>>();
        if !lost.is_empty() {
            merged.edit_many(lost)?;
        }
        merged.keep_current_pending();
        *self = merged;
        Ok(())
    }

    fn keep_current_pending(&mut self) {
        self.pending_feedback.retain(|key, stamp| {
            self.document
                .records
                .get(key)
                .is_some_and(|record| &record.stamp == stamp)
        });
    }

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
                    .filter(|(key, record)| {
                        key.as_str() != FEEDBACK_FLOOR
                            && snapshot.records.get(*key) != Some(*record)
                    })
                    .map(|(key, record)| (key.clone(), record.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut merged = self.clone();
        merged.document.merge(remote)?;
        // Incorporate the remote clock first, then express the newer local
        // intent above it in one write. Tombstones are edits as well.
        let edits = changed
            .into_iter()
            .filter(|(key, local)| merged.document.records.get(key) != Some(local))
            .map(|(key, local)| (key, local.value))
            .collect::<Vec<_>>();
        if !edits.is_empty() {
            merged.edit_many(edits)?;
        }
        if let Some(snapshot) = snapshot {
            merged.pending_feedback.retain(|key, stamp| {
                snapshot
                    .records
                    .get(key)
                    .is_none_or(|sent| &sent.stamp != stamp)
            });
        }
        merged.keep_current_pending();
        *self = merged;
        Ok(())
    }

    pub fn edit(&mut self, key: String, value: Option<Value>) -> Result<(), String> {
        self.edit_many(vec![(key, value)])
    }

    /// Store an AI result in a bounded set of history keys. Below
    /// [`HISTORY_LIMIT`] keys it adds one; after that it overwrites the oldest
    /// history key, live or tombstone, so keys stop accumulating. Live entries
    /// beyond the limit, such as legacy history, become tombstones in the same
    /// atomic write. Every other record and clock is left untouched.
    pub fn record_history(
        &mut self,
        prompt: String,
        suggestions: Vec<Suggestion>,
    ) -> Result<(), String> {
        let mut history = self
            .document
            .records
            .iter()
            .filter(|(key, _)| key.starts_with("history:"))
            .map(|(key, record)| (&record.stamp, key.as_str(), record.value.is_some()))
            .collect::<Vec<_>>();
        // Newest first; the oldest key is last.
        history.sort_unstable_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
        let target = if history.len() < HISTORY_LIMIT {
            format!("history:{:032x}", rand::random::<u128>())
        } else {
            history
                .pop()
                .map_or_else(String::new, |(_, key, _)| key.to_owned())
        };
        let mut edits = history
            .iter()
            .filter(|(_, _, live)| *live)
            .skip(HISTORY_LIMIT - 1)
            .map(|(_, key, _)| ((*key).to_owned(), None))
            .collect::<Vec<_>>();
        edits.push((
            target,
            Some(Value::History {
                prompt,
                suggestions,
            }),
        ));
        self.edit_many(edits)
    }

    /// Apply edits atomically under one logical clock above every record.
    fn edit_many(&mut self, edits: Vec<(String, Option<Value>)>) -> Result<(), String> {
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
        for (key, value) in edits {
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
        }
        candidate.validate_records()?;
        candidate.compact_feedback();
        candidate.validate()?;
        for (key, record) in &candidate.records {
            if key.starts_with("feedback:")
                && key != FEEDBACK_FLOOR
                && self.document.records.get(key) != Some(record)
                && record.stamp.device == self.device
                && record.stamp.counter == counter
            {
                self.pending_feedback
                    .insert(key.clone(), record.stamp.clone());
            }
        }
        self.document = candidate;
        self.keep_current_pending();
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
        if replica.pending_feedback.len() > FEEDBACK_LIMIT
            || replica.pending_feedback.iter().any(|(key, stamp)| {
                key == FEEDBACK_FLOOR
                    || !key.starts_with("feedback:")
                    || replica
                        .document
                        .records
                        .get(key)
                        .is_none_or(|record| &record.stamp != stamp)
            })
        {
            return Err("Invalid pending discovery feedback.".into());
        }
        replica.document.compact_feedback();
        replica.keep_current_pending();
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
pub(crate) mod tests {
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
    fn manual_cooldown_is_active_until_the_retry_deadline() {
        let now = std::time::Instant::now();
        let mut scheduler = AutoRecommendations::default();
        scheduler.failed(RecommendationErrorKind::RateLimited, now);
        assert_eq!(
            scheduler.retry_after(now),
            Some(std::time::Duration::from_secs(600))
        );
        assert!(
            scheduler
                .retry_after(now + std::time::Duration::from_secs(599))
                .is_some()
        );
        assert!(
            scheduler
                .retry_after(now + std::time::Duration::from_secs(600))
                .is_none()
        );
    }

    #[test]
    fn recommendation_throttles_survive_restart_and_clock_changes() {
        use std::time::{Duration, Instant};
        let now = Instant::now();
        let mut scheduler = AutoRecommendations::default();
        scheduler.attempted(now);
        scheduler.rearm();
        let mut replica = Replica {
            recommendation_throttle: scheduler.checkpoint(now, 100_000),
            ..Default::default()
        };
        // Exercise the persisted local format, not only an in-memory copy.
        let path =
            Path::new("target").join(format!("discovery-throttle-{}.json", rand::random::<u64>()));
        replica.save(&path).unwrap();
        let loaded = Replica::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let mut restored =
            AutoRecommendations::restore(&loaded.recommendation_throttle, now, 100_010);
        restored.changed(now, false);
        assert_eq!(
            restored.due(now, 100_010, &Preferences::default(), true, true, false),
            Some(Duration::from_secs(590))
        );
        let restored = AutoRecommendations::restore(&loaded.recommendation_throttle, now, 100_010);
        assert_eq!(
            restored.due(now, 100_010, &Preferences::default(), true, true, true),
            Some(Duration::from_secs(590))
        );

        scheduler.failed(RecommendationErrorKind::RateLimited, now);
        replica.recommendation_throttle = scheduler.checkpoint(now, 100_000);
        let stored: Replica =
            serde_json::from_slice(&serde_json::to_vec(&replica).unwrap()).unwrap();
        assert!(
            !serde_json::to_string(&stored.document)
                .unwrap()
                .contains("retry_at")
        );
        let mut restored =
            AutoRecommendations::restore(&stored.recommendation_throttle, now, 100_010);
        assert_eq!(restored.retry_after(now), Some(Duration::from_secs(590)));
        restored.attempted(now + Duration::from_secs(590));
        restored.failed(
            RecommendationErrorKind::RateLimited,
            now + Duration::from_secs(590),
        );
        assert_eq!(
            restored.retry_after(now + Duration::from_secs(590)),
            Some(Duration::from_secs(1200))
        );
        assert!(
            AutoRecommendations::restore(&stored.recommendation_throttle, now, 100_600)
                .retry_after(now)
                .is_none()
        );
        let skewed = RecommendationThrottle {
            last_attempt_at: Some(u64::MAX),
            retry_at: Some(u64::MAX),
            failures: u32::MAX,
            suspended: true,
            changed_at: None,
        };
        let restored = AutoRecommendations::restore(&skewed, now, 100_000);
        assert_eq!(restored.retry_after(now), Some(Duration::from_secs(21_600)));
        assert_eq!(restored.failures, 10);
        assert!(restored.suspended);
        let legacy: Replica = serde_json::from_str(&format!(
            r#"{{"device":"{}","document":{{"version":1,"records":{{}}}}}}"#,
            "a".repeat(32)
        ))
        .unwrap();
        assert_eq!(
            legacy.recommendation_throttle,
            RecommendationThrottle::default()
        );
    }

    #[test]
    fn repeated_mix_saves_reuse_slots_and_deleted_mixes_do_not_return() {
        let mut replica = Replica::default();
        let uri = "spotify:track:0123456789ABCDEFGHIJKL".to_owned();
        for index in 0..2500 {
            replica
                .save_mix(format!("Mix {index}"), vec![uri.clone()])
                .unwrap();
        }
        assert_eq!(replica.document.records.len(), MIX_LIMIT);
        assert!(
            replica
                .document
                .records
                .values()
                .any(|r| matches!(&r.value, Some(Value::Mix { title, .. }) if title == "Mix 2499"))
        );
        let key = replica.document.records.keys().next().unwrap().clone();
        let stale = replica.document.clone();
        replica.edit(key.clone(), None).unwrap();
        replica.document.merge(&stale).unwrap();
        assert!(replica.document.records[&key].value.is_none());
        replica
            .save_mix("Replacement".into(), vec![uri.clone()])
            .unwrap();
        assert_eq!(replica.document.records.len(), MIX_LIMIT);
        assert!(
            matches!(&replica.document.records[&key].value, Some(Value::Mix { title, .. }) if title == "Replacement")
        );
        replica
            .edit(
                format!("feedback:{uri}"),
                Some(Value::Feedback {
                    uri,
                    title: "Song".into(),
                    artist: "Artist".into(),
                    rating: Rating::Love,
                }),
            )
            .unwrap();
        replica
            .record_history(
                "Taste".into(),
                vec![Suggestion {
                    title: "Song".into(),
                    artist: "Artist".into(),
                    reason: "Fits".into(),
                }],
            )
            .unwrap();
        replica.document.validate().unwrap();
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

    fn feedback_edit(index: usize, clear: bool) -> (String, Option<Value>) {
        let uri = format!("spotify:track:{index:022}");
        (
            format!("feedback:{uri}"),
            (!clear).then_some(Value::Feedback {
                uri,
                title: "Song".into(),
                artist: "Artist".into(),
                rating: Rating::Love,
            }),
        )
    }

    #[test]
    fn unsent_offline_ratings_survive_a_new_remote_floor_and_restart() {
        let mut local = device('a');
        let (key, value) = feedback_edit(0, false);
        local.edit(key.clone(), value.clone()).unwrap();
        let bytes = serde_json::to_vec(&local).unwrap();
        let mut restarted: Replica = serde_json::from_slice(&bytes).unwrap();
        restarted.device = "c".repeat(32);
        let mut remote = Document::default();
        remote.records.insert(
            FEEDBACK_FLOOR.into(),
            Record {
                stamp: Stamp {
                    counter: 100,
                    device: "b".repeat(32),
                },
                value: None,
            },
        );
        restarted.merge_for_sync(&remote).unwrap();
        assert_eq!(restarted.document.records[&key].value, value);
        assert_eq!(restarted.document.records[&key].stamp.counter, 101);
        let uploaded = restarted.document.clone();
        let snapshot = local.document.clone();
        local.merge_synced(&uploaded, Some(&snapshot)).unwrap();
        assert_eq!(local.document, uploaded);
        assert!(local.pending_feedback.is_empty());
        // Once acknowledged, this is an old replica, not fresh user intent.
        remote
            .records
            .get_mut(FEEDBACK_FLOOR)
            .unwrap()
            .stamp
            .counter = 200;
        local.merge_for_sync(&remote).unwrap();
        assert!(!local.document.records.contains_key(&key));
        assert!(local.pending_feedback.is_empty());
    }

    #[test]
    fn pending_rating_clears_and_later_edits_remain_atomic_and_bounded() {
        let mut local = device('a');
        let (key, value) = feedback_edit(0, false);
        local.edit(key.clone(), value).unwrap();
        let snapshot = local.document.clone();
        local.edit(key.clone(), None).unwrap();
        let mut remote = Document::default();
        remote.records.insert(
            FEEDBACK_FLOOR.into(),
            Record {
                stamp: Stamp {
                    counter: 100,
                    device: "b".repeat(32),
                },
                value: None,
            },
        );
        local.merge_synced(&remote, Some(&snapshot)).unwrap();
        assert_eq!(local.document.records[&key].stamp.counter, 101);
        assert!(local.document.records[&key].value.is_none());
        assert_eq!(local.pending_feedback.len(), 1);
        let sent = local.document.clone();
        local.merge_synced(&sent, Some(&sent)).unwrap();
        assert!(local.pending_feedback.is_empty());
        for index in 1..=FEEDBACK_LIMIT + 100 {
            let (key, value) = feedback_edit(index, false);
            local.edit(key, value).unwrap();
        }
        assert_eq!(local.pending_feedback.len(), FEEDBACK_LIMIT);
    }

    #[test]
    fn effective_inputs_detect_feedback_removed_by_an_advanced_floor() {
        let mut replica = device('a');
        let (key, value) = feedback_edit(0, false);
        replica.edit(key, value).unwrap();
        let before = replica.document.clone();
        let mut after = before.clone();
        after.records.insert(
            FEEDBACK_FLOOR.into(),
            Record {
                stamp: Stamp {
                    counter: 10,
                    device: "b".repeat(32),
                },
                value: None,
            },
        );
        after.compact_feedback();
        assert!(!before.same_inputs(&after));
        let mut newer_clock = before.clone();
        for record in newer_clock.records.values_mut() {
            record.stamp.counter += 1;
        }
        assert!(before.same_inputs(&newer_clock));
    }

    #[test]
    fn feedback_and_clears_stay_bounded_without_crowding_out_mixes() {
        let mut replica = device('a');
        for index in 0..2100 {
            let (key, value) = feedback_edit(index, index % 2 == 0);
            replica.edit(key, value).unwrap();
        }
        assert_eq!(replica.document.records.len(), FEEDBACK_LIMIT + 1);
        assert_eq!(replica.document.records[FEEDBACK_FLOOR].stamp.counter, 1600);
        assert!(replica.document.records[FEEDBACK_FLOOR].value.is_none());
        replica
            .edit(
                "mix:new".into(),
                Some(Value::Mix {
                    title: "Still works".into(),
                    uris: vec![],
                }),
            )
            .unwrap();
        replica
            .record_history(
                "Still works".into(),
                vec![Suggestion {
                    title: "Song".into(),
                    artist: "Artist".into(),
                    reason: "Good match".into(),
                }],
            )
            .unwrap();
        assert_eq!(replica.document.records.len(), FEEDBACK_LIMIT + 3);
    }

    #[test]
    fn feedback_compaction_converges_and_stale_ratings_cannot_return() {
        let mut a = device('a');
        let (old_key, old_value) = feedback_edit(0, false);
        a.edit(old_key.clone(), old_value.clone()).unwrap();
        let stale = a.document.clone();
        for index in 1..=FEEDBACK_LIMIT {
            let (key, value) = feedback_edit(index, index % 2 == 0);
            a.edit(key, value).unwrap();
        }
        assert!(!a.document.records.contains_key(&old_key));
        let compacted = a.document.clone();
        a.document.merge(&stale).unwrap();
        assert_eq!(a.document, compacted);
        let mut old_replica = stale;
        old_replica.merge(&compacted).unwrap();
        assert_eq!(old_replica, compacted);
        a.edit(old_key.clone(), old_value).unwrap();
        assert!(
            a.document.records[&old_key].value.is_some(),
            "a new rating is above the cutoff"
        );
        assert_eq!(a.document.records.len(), FEEDBACK_LIMIT + 1);
    }

    #[test]
    fn feedback_union_compacts_before_capacity_validation_and_handles_clock_ties() {
        let mut a = device('a');
        let mut b = device('b');
        // Two valid legacy snapshots together exceed the 2,000-record ceiling.
        for index in 0..1200 {
            let stamp = |device: &str| Stamp {
                counter: index as u64 + 1,
                device: device.into(),
            };
            let (key, value) = feedback_edit(index, false);
            a.document.records.insert(
                key,
                Record {
                    stamp: stamp(&a.device),
                    value,
                },
            );
            let (key, value) = feedback_edit(index + 1200, true);
            b.document.records.insert(
                key,
                Record {
                    stamp: stamp(&b.device),
                    value,
                },
            );
        }
        let mut left = a.document.clone();
        left.merge(&b.document).unwrap();
        b.document.merge(&a.document).unwrap();
        assert_eq!(left, b.document);
        assert_eq!(left.records.len(), FEEDBACK_LIMIT + 1);
        // A merge acknowledgment can give several edits the same clock.
        let mut tied = device('c');
        tied.edit_many(
            (0..=FEEDBACK_LIMIT)
                .map(|i| feedback_edit(i, false))
                .collect(),
        )
        .unwrap();
        assert_eq!(
            tied.document.records.len(),
            1,
            "forget a tied cohort atomically"
        );
        tied.document.merge(&left).unwrap();
        left.merge(&tied.document).unwrap();
        assert_eq!(left, tied.document);
    }

    #[test]
    fn sync_ack_preserves_new_feedback_without_restamping_the_cutoff() {
        let mut local = device('a');
        for index in 0..=FEEDBACK_LIMIT {
            let (key, value) = feedback_edit(index, false);
            local.edit(key, value).unwrap();
        }
        let snapshot = local.document.clone();
        let (key, value) = feedback_edit(FEEDBACK_LIMIT + 1, false);
        local.edit(key.clone(), value).unwrap();
        let mut remote = device('b');
        remote.document.merge(&snapshot).unwrap();
        for _ in 0..10 {
            remote.edit("taste".into(), taste("remote clock")).unwrap();
        }
        let (_, conflicting) = feedback_edit(FEEDBACK_LIMIT + 1, false);
        remote.edit(key.clone(), conflicting).unwrap();
        local
            .merge_synced(&remote.document, Some(&snapshot))
            .unwrap();
        assert!(
            local.document.records[&key].stamp.counter
                > remote.document.records["taste"].stamp.counter
        );
        assert_eq!(local.document.records[FEEDBACK_FLOOR].stamp.counter, 2);
        remote.document.merge(&local.document).unwrap();
        assert_eq!(local.document, remote.document);
    }

    #[test]
    fn legacy_feedback_is_compacted_after_validation_on_load() {
        let mut legacy = device('a');
        for index in 0..1999 {
            let (key, value) = feedback_edit(index, index % 2 == 0);
            legacy.document.records.insert(
                key,
                Record {
                    stamp: Stamp {
                        counter: index as u64 + 1,
                        device: legacy.device.clone(),
                    },
                    value,
                },
            );
        }
        legacy.document.records.insert(
            "taste".into(),
            Record {
                stamp: Stamp {
                    counter: 2000,
                    device: legacy.device.clone(),
                },
                value: taste("Keep my taste"),
            },
        );
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!(
                "feedback-retention-{:032x}",
                rand::random::<u128>()
            ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("legacy.json");
        legacy.save(&path).unwrap();
        let loaded = Replica::load(&path).unwrap();
        assert_eq!(loaded.document.records.len(), FEEDBACK_LIMIT + 2);
        assert_eq!(loaded.document.taste(), "Keep my taste");
        assert_eq!(
            legacy.document.records.len(),
            2000,
            "loading does not alter the original file"
        );
        loaded.save(&path).unwrap();
        assert_eq!(Replica::load(&path).unwrap().document, loaded.document);
        std::fs::remove_dir_all(directory).unwrap();
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

    /// The largest valid history entry, about 16 KiB of JSON.
    fn large_suggestions(label: &str) -> Vec<Suggestion> {
        (0..12)
            .map(|n| Suggestion {
                title: format!("{label} {n} {}", "t".repeat(280)),
                artist: "a".repeat(300),
                reason: "r".repeat(600),
            })
            .collect()
    }

    fn history_keys(document: &Document) -> (usize, usize) {
        let history = document
            .records
            .iter()
            .filter(|(key, _)| key.starts_with("history:"));
        let live = history.clone().filter(|(_, r)| r.value.is_some()).count();
        (history.count(), live)
    }

    fn encoded_len(document: &Document) -> usize {
        serde_json::to_vec(document).unwrap().len()
    }

    #[test]
    fn repeated_refreshes_reuse_a_bounded_set_of_history_keys() {
        let mut replica = device('a');
        replica.edit("taste".into(), taste("kept taste")).unwrap();
        let mut capped = 0;
        for refresh in 0..60 {
            replica
                .record_history(
                    "p".repeat(MAX_PROMPT_BYTES),
                    large_suggestions(&format!("{refresh:02}")),
                )
                .unwrap();
            let (keys, live) = history_keys(&replica.document);
            assert_eq!(keys, (refresh + 1).min(HISTORY_LIMIT));
            assert_eq!(live, keys);
            if refresh + 1 == HISTORY_LIMIT {
                capped = encoded_len(&replica.document);
            }
        }
        // Only the clock digits may grow once the keys are reused.
        assert!(encoded_len(&replica.document) <= capped + 100);
        let recent = replica.document.recent_history();
        assert_eq!(recent.len(), HISTORY_LIMIT);
        let Some(Value::History { suggestions, .. }) = &recent[0].1.value else {
            panic!("newest history is live");
        };
        assert!(suggestions[0].title.starts_with("59 "));
        assert_eq!(replica.document.taste(), "kept taste");
        assert_eq!(replica.document.records["taste"].stamp.counter, 1);
    }

    /// Legacy history from before the limit, with other records interleaved.
    fn legacy(replica: &mut Replica, entries: usize) {
        let uri = "spotify:track:0123456789ABCDEFGHIJKL";
        replica.edit("taste".into(), taste("legacy taste")).unwrap();
        replica
            .edit(
                format!("feedback:{uri}"),
                Some(Value::Feedback {
                    uri: uri.into(),
                    title: "Song".into(),
                    artist: "Artist".into(),
                    rating: Rating::Love,
                }),
            )
            .unwrap();
        replica.edit("mix:gone".into(), None).unwrap();
        for n in 0..entries {
            let value = Some(Value::History {
                prompt: format!("legacy {n}"),
                suggestions: large_suggestions(&format!("legacy {n}")),
            });
            replica.edit(format!("history:{n:032x}"), value).unwrap();
        }
        replica
            .edit(
                "mix:kept".into(),
                Some(Value::Mix {
                    title: "Kept".into(),
                    uris: vec![uri.into()],
                }),
            )
            .unwrap();
    }

    #[test]
    fn near_limit_legacy_history_is_trimmed_without_touching_other_records() {
        let mut replica = device('a');
        const LEGACY: usize = 70;
        legacy(&mut replica, LEGACY);
        let before = replica.document.clone();
        assert!(
            encoded_len(&before) > MAX_BYTES - 20_000,
            "fixture is near the limit"
        );
        // Even the largest new entry fits once the excess is tombstoned.
        replica
            .record_history("new".into(), large_suggestions("new"))
            .unwrap();
        let after = &replica.document;
        assert_eq!(history_keys(after), (LEGACY, HISTORY_LIMIT), "no new key");
        assert!(encoded_len(after) < encoded_len(&before) / 4);
        let counter = before
            .records
            .values()
            .map(|r| r.stamp.counter)
            .max()
            .unwrap()
            + 1;
        for (key, old) in &before.records {
            let new = &after.records[key];
            if !key.starts_with("history:") {
                assert_eq!(new, old, "{key} keeps its value and clock");
            } else if new != old {
                assert_eq!(new.stamp.counter, counter);
            }
        }
        // The oldest key holds the new entry; the newest nine stay; the rest are tombstones.
        assert!(matches!(
            &after.records[&format!("history:{:032x}", 0)].value,
            Some(Value::History { prompt, .. }) if prompt == "new"
        ));
        for n in 1..LEGACY {
            let record = &after.records[&format!("history:{n:032x}")];
            assert_eq!(
                record.value.is_some(),
                n > LEGACY - HISTORY_LIMIT,
                "legacy {n}"
            );
        }
    }

    #[test]
    fn history_failures_leave_the_document_unchanged() {
        let mut replica = device('a');
        legacy(&mut replica, 20);
        replica
            .document
            .records
            .get_mut("mix:kept")
            .unwrap()
            .stamp
            .counter = u64::MAX - 1;
        let before = replica.document.clone();
        assert!(
            replica
                .record_history("x".into(), large_suggestions("x"))
                .is_err()
        );
        assert_eq!(replica.document, before, "clock exhaustion");
        replica
            .document
            .records
            .get_mut("mix:kept")
            .unwrap()
            .stamp
            .counter = u64::MAX - 2;
        let before = replica.document.clone();
        assert!(replica.record_history("x".into(), vec![]).is_err());
        assert_eq!(replica.document, before, "invalid entry");

        let mut full = device('b');
        full.edit("taste".into(), taste("full")).unwrap();
        fill_with_mixes(&mut full.document, 2000);
        let before = full.document.clone();
        assert_eq!(
            full.record_history("x".into(), large_suggestions("x")),
            Err("Discovery storage is full. Export and remove old mixes or history.".into())
        );
        assert_eq!(full.document, before, "storage full");
    }

    /// Fill a valid document with mixes until less than `room` + 200 bytes remain.
    pub(crate) fn fill_with_mixes(document: &mut Document, room: usize) {
        let mut n = 0;
        for size in [100, 1] {
            let uris = (0..size)
                .map(|n| format!("spotify:track:{n:022}"))
                .collect::<Vec<_>>();
            loop {
                let key = format!("mix:fill{n}");
                n += 1;
                document.records.insert(
                    key.clone(),
                    Record {
                        stamp: Stamp {
                            counter: 1,
                            device: "f".repeat(32),
                        },
                        value: Some(Value::Mix {
                            title: "Fill".into(),
                            uris: uris.clone(),
                        }),
                    },
                );
                if encoded_len(document) > MAX_BYTES - room {
                    document.records.remove(&key);
                    break;
                }
            }
        }
        document.validate().unwrap();
        assert!(encoded_len(document) > MAX_BYTES - room - 200);
    }

    #[test]
    fn offline_merges_never_resurrect_trimmed_history() {
        let mut a = device('a');
        legacy(&mut a, 15);
        let base = a.document.clone();
        let mut stale = device('b');
        stale.document = base.clone();
        let mut c = device('c');
        c.document = base.clone();
        // A refresh while a sync of `base` is in flight.
        a.record_history("new".into(), large_suggestions("new"))
            .unwrap();
        stale.edit("taste".into(), taste("offline taste")).unwrap();
        let trimmed = |document: &Document| {
            (1..6).all(|n| {
                document.records[&format!("history:{n:032x}")]
                    .value
                    .is_none()
            })
        };
        assert!(trimmed(&a.document));

        a.merge_synced(&stale.document, Some(&base)).unwrap();
        assert!(trimmed(&a.document));
        assert_eq!(a.document.taste(), "offline taste");
        stale.document.merge(&a.document).unwrap();
        assert_eq!(stale.document, a.document);
        assert_eq!(history_keys(&a.document), (15, HISTORY_LIMIT));

        // Concurrent refreshes reuse the same oldest key and still converge.
        c.record_history("offline".into(), large_suggestions("c"))
            .unwrap();
        let a_document = a.document.clone();
        a.document.merge(&c.document).unwrap();
        c.document.merge(&a_document).unwrap();
        assert_eq!(a.document, c.document);
        assert!(trimmed(&a.document));
        assert_eq!(history_keys(&a.document), (15, HISTORY_LIMIT));
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
