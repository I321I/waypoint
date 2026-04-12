pub mod detector;
pub mod normalizer;

use crate::storage::app_config::{AppConfig, MatchBy};
use detector::FocusedWindowInfo;
use normalizer::{normalize_process_name, resolve_alias};

/// Derive the canonical context_id from a FocusedWindowInfo,
/// applying normalization, per-context matchBy config, and aliases.
pub fn derive_context_id(info: &FocusedWindowInfo, config: &AppConfig) -> String {
    let raw = if let Some(ctx_cfg) = config.contexts.get(
        &normalize_process_name(&info.process_name)
    ) {
        if ctx_cfg.match_by == MatchBy::Title {
            info.window_title.to_lowercase()
        } else {
            info.process_name.clone()
        }
    } else {
        info.process_name.clone()
    };

    let normalized = normalize_process_name(&raw);
    resolve_alias(&normalized, &config.context_aliases).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::app_config::{AppConfig, ContextConfig, MatchBy};

    fn make_info(process: &str, title: &str) -> FocusedWindowInfo {
        FocusedWindowInfo {
            process_name: process.to_string(),
            window_title: title.to_string(),
        }
    }

    #[test]
    fn derives_context_from_process_by_default() {
        let config = AppConfig::default();
        let info = make_info("Steam.exe", "Steam");
        assert_eq!(derive_context_id(&info, &config), "steam");
    }

    #[test]
    fn uses_window_title_when_configured() {
        let mut config = AppConfig::default();
        config.contexts.insert("steam".to_string(), ContextConfig { match_by: MatchBy::Title });
        let info = make_info("steam", "Counter-Strike 2");
        assert_eq!(derive_context_id(&info, &config), "counter-strike 2");
    }

    #[test]
    fn applies_alias_after_normalization() {
        let mut config = AppConfig::default();
        config.context_aliases.insert("mygame_win".to_string(), "mygame".to_string());
        let info = make_info("mygame_win.exe", "My Game");
        assert_eq!(derive_context_id(&info, &config), "mygame");
    }
}
