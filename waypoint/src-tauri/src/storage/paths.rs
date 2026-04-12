use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot find home dir")
        .join("waypoint")
}

pub fn global_dir() -> PathBuf {
    data_dir().join("global")
}

pub fn context_dir(context_id: &str) -> PathBuf {
    data_dir().join("contexts").join(context_id)
}

pub fn note_dir(context_id: Option<&str>, note_id: &str) -> PathBuf {
    match context_id {
        Some(ctx) => context_dir(ctx).join(note_id),
        None => global_dir().join(note_id),
    }
}

pub fn app_config_path() -> PathBuf {
    data_dir().join("app.json")
}

pub fn session_path(context_id: &str) -> PathBuf {
    context_dir(context_id).join("session.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_note_path_has_no_contexts_segment() {
        let p = note_dir(None, "abc123");
        assert!(p.to_str().unwrap().contains("global"));
        assert!(!p.to_str().unwrap().contains("contexts"));
    }

    #[test]
    fn context_note_path_contains_context_id() {
        let p = note_dir(Some("steam"), "abc123");
        let s = p.to_str().unwrap();
        assert!(s.contains("contexts"));
        assert!(s.contains("steam"));
        assert!(s.contains("abc123"));
    }

    #[test]
    fn session_path_is_inside_context_dir() {
        let p = session_path("steam");
        let s = p.to_str().unwrap();
        assert!(s.contains("contexts"));
        assert!(s.contains("steam"));
        assert!(s.ends_with("session.json"));
    }
}
