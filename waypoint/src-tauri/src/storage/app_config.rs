use crate::error::WaypointError;
use crate::storage::paths::app_config_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    pub match_by: MatchBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MatchBy {
    Process,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub context_aliases: HashMap<String, String>,
    #[serde(default)]
    pub contexts: HashMap<String, ContextConfig>,
}

fn default_hotkey() -> String {
    "Ctrl+Shift+Space".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            hotkey: default_hotkey(),
            context_aliases: HashMap::new(),
            contexts: HashMap::new(),
        }
    }
}

pub fn load() -> Result<AppConfig, WaypointError> {
    let path = app_config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save(config: &AppConfig) -> Result<(), WaypointError> {
    let path = app_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_temp_home(f: impl FnOnce(&TempDir)) {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        f(&dir);
    }

    #[test]
    fn load_returns_default_when_no_file() {
        with_temp_home(|_| {
            let cfg = load().unwrap();
            assert_eq!(cfg.hotkey, "Ctrl+Shift+Space");
            assert!(cfg.context_aliases.is_empty());
        });
    }

    #[test]
    fn save_and_reload_roundtrip() {
        with_temp_home(|_| {
            let mut cfg = AppConfig::default();
            cfg.hotkey = "Ctrl+Alt+N".to_string();
            cfg.context_aliases.insert("steam_win".to_string(), "steam".to_string());
            save(&cfg).unwrap();
            let loaded = load().unwrap();
            assert_eq!(loaded.hotkey, "Ctrl+Alt+N");
            assert_eq!(loaded.context_aliases.get("steam_win").unwrap(), "steam");
        });
    }
}
