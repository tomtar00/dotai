use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::util;

pub type Models = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub models: Models,
}

impl Default for Config {
    fn default() -> Self {
        let mut models = Models::new();
        for (provider, prefix) in [("claude", ""), ("cursor", ""), ("opencode", "anthropic/")] {
            let mut map = HashMap::new();
            map.insert("small".to_string(), format!("{}claude-haiku-4.5", prefix));
            map.insert("medium".to_string(), format!("{}claude-sonnet-5", prefix));
            map.insert("large".to_string(), format!("{}claude-opus-5", prefix));
            models.insert(provider.to_string(), map);
        }
        Self {
            providers: vec![
                "claude".to_string(),
                "opencode".to_string(),
                "cursor".to_string(),
            ],
            models,
        }
    }
}

pub fn path() -> std::path::PathBuf {
    util::config_dir().join("dotai.json")
}

pub fn load() -> Result<Option<Config>> {
    let path = path();
    if !path.exists() {
        return Ok(None);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config = serde_json::from_str(&content)
        .with_context(|| format!("Invalid config {}", path.display()))?;
    Ok(Some(config))
}

pub fn ensure_default() -> Result<bool> {
    let path = path();
    if path.exists() {
        return Ok(false);
    }
    util::write(&path, &serde_json::to_string_pretty(&Config::default())?)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}
