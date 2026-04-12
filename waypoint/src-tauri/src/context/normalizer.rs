use std::collections::HashMap;

/// Normalize a raw process name to a canonical context id.
/// Rules: strip .exe suffix (case-insensitive), lowercase all.
pub fn normalize_process_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// Resolve a normalized context id through the alias map.
pub fn resolve_alias<'a>(context_id: &'a str, aliases: &'a HashMap<String, String>) -> &'a str {
    aliases
        .get(context_id)
        .map(|s| s.as_str())
        .unwrap_or(context_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_exe_suffix() {
        assert_eq!(normalize_process_name("steam.exe"), "steam");
        assert_eq!(normalize_process_name("Steam.EXE"), "steam");
    }

    #[test]
    fn lowercases_without_exe() {
        assert_eq!(normalize_process_name("Steam"), "steam");
        assert_eq!(normalize_process_name("Firefox"), "firefox");
    }

    #[test]
    fn already_normalized_unchanged() {
        assert_eq!(normalize_process_name("steam"), "steam");
    }

    #[test]
    fn resolve_alias_returns_canonical() {
        let mut aliases = HashMap::new();
        aliases.insert("mygame_win".to_string(), "mygame".to_string());
        assert_eq!(resolve_alias("mygame_win", &aliases), "mygame");
    }

    #[test]
    fn resolve_alias_passthrough_when_no_match() {
        let aliases = HashMap::new();
        assert_eq!(resolve_alias("steam", &aliases), "steam");
    }
}
