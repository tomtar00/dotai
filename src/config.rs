use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::util;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub providers: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            providers: vec![
                "claude".to_string(),
                "opencode".to_string(),
                "cursor".to_string(),
            ],
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
