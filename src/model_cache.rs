//! Per-provider model list cache stored as JSON at
//! ~/.config/crabai/model_cache.json. Entries expire after a
//! configurable TTL (default 24 hours).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::CrabError;

use crate::types::ModelInfo;

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    /// Unix timestamp (seconds) when this entry was written.
    timestamp: u64,
    models: Vec<ModelInfo>,
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
        if path.exists() {
            if let Ok(s) = std::fs::read_to_string(&path) {
                if let Ok(cache) = serde_json::from_str::<Self>(&s) {
                    return cache;
                }
            }
        }

        // Use bundled seed cache if no local cache exists or is invalid.
        Self::load_seed()
    }

    fn load_seed() -> Self {
        let seed_json = include_str!("seed_model_cache.json");
        serde_json::from_str(seed_json).unwrap_or_default()
    }

    pub fn save(&self, config_dir: &Path) -> Result<(), CrabError> {
        std::fs::create_dir_all(config_dir)?;
        let path = Self::cache_path(config_dir);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Return cached models if the entry exists and has not exceeded ttl_hours.
    pub fn get(&self, provider: &str, ttl_hours: u64) -> Option<Vec<ModelInfo>> {
        let entry = self.entries.get(provider)?;
        let now = now_unix();
        let age_hours = (now.saturating_sub(entry.timestamp)) / 3600;
        if age_hours < ttl_hours {
            Some(entry.models.clone())
        } else {
            None
        }
    }

    pub fn set(&mut self, provider: &str, models: Vec<ModelInfo>) {
        self.entries.insert(
            provider.to_string(),
            CacheEntry {
                timestamp: now_unix(),
                models,
            },
        );
    }

    /// Update a single model's information in the cache without replacing the whole provider entry.
    pub fn update_model(&mut self, provider: &str, info: ModelInfo) {
        if let Some(entry) = self.entries.get_mut(provider) {
            if let Some(existing) = entry.models.iter_mut().find(|m| m.id == info.id) {
                *existing = info;
            } else {
                entry.models.push(info);
            }
        } else {
            // If provider not in cache, create it
            self.set(provider, vec![info]);
        }
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
