//! Per-provider model list cache stored as JSON at
//! ~/.config/crabai/model_cache.json. Entries expire after a
//! configurable TTL (default 24 hours).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::CrabError;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    /// Unix timestamp (seconds) when this entry was written.
    timestamp: u64,
    models: Vec<String>,
}

/// Top-level cache structure. Serialized flat so the JSON keys are
/// provider names directly (e.g. {"openai": {...}, "groq": {...}}).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModelCache {
    #[serde(flatten)]
    entries: HashMap<String, CacheEntry>,
}

impl ModelCache {
    fn cache_path(config_dir: &Path) -> PathBuf {
        config_dir.join("model_cache.json")
    }

    /// Load from disk. Returns an empty cache on any read or parse failure.
    pub fn load(config_dir: &Path) -> Self {
        let path = Self::cache_path(config_dir);
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, config_dir: &Path) -> Result<(), CrabError> {
        std::fs::create_dir_all(config_dir)?;
        let path = Self::cache_path(config_dir);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Return cached models if the entry exists and has not exceeded ttl_hours.
    pub fn get(&self, provider: &str, ttl_hours: u64) -> Option<&[String]> {
        let entry = self.entries.get(provider)?;
        let now = now_unix();
        let age_hours = (now.saturating_sub(entry.timestamp)) / 3600;
        if age_hours < ttl_hours {
            Some(&entry.models)
        } else {
            None
        }
    }

    pub fn set(&mut self, provider: &str, models: Vec<String>) {
        self.entries.insert(
            provider.to_string(),
            CacheEntry {
                timestamp: now_unix(),
                models,
            },
        );
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
