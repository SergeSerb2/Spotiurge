//! Personal discovery state. No Spotify grants or audio enter this document.
//!
//! Per-record logical clocks make offline edits merge independently of wall
//! clocks. The cloud stores only the document; installation identity stays local.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const MAX_BYTES: usize = 1_048_576;

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
                Some(Value::Taste { text }) => key == "taste" && text.len() <= 4000,
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
                        && prompt.len() <= 4000
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
        let replica: Self = serde_json::from_slice(&bytes)
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
    pub request: u64,
    pub status: String,
    pub dirty: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(id: char) -> Replica {
        Replica {
            device: id.to_string().repeat(32),
            ..Default::default()
        }
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
}
