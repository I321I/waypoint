use crate::error::WaypointError;
use crate::storage::paths::session_path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub open_context_notes: Vec<String>,
    #[serde(default)]
    pub open_global_notes: Vec<String>,
}

pub fn load_session(context_id: &str) -> Result<Session, WaypointError> {
    let path = session_path(context_id);
    if !path.exists() {
        return Ok(Session::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save_session(context_id: &str, session: &Session) -> Result<(), WaypointError> {
    let path = session_path(context_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(session)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn clear_session(context_id: &str) -> Result<(), WaypointError> {
    let path = session_path(context_id);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_utils::HOME_LOCK;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = HOME_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        (dir, guard)
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let (_dir, _guard) = setup();
        let s = load_session("steam").unwrap();
        assert!(s.open_context_notes.is_empty());
        assert!(s.open_global_notes.is_empty());
    }

    #[test]
    fn save_and_reload_session() {
        let (_dir, _guard) = setup();
        let session = Session {
            open_context_notes: vec!["note-1".to_string(), "note-2".to_string()],
            open_global_notes: vec!["global-1".to_string()],
        };
        save_session("steam", &session).unwrap();
        let loaded = load_session("steam").unwrap();
        assert_eq!(loaded.open_context_notes, vec!["note-1", "note-2"]);
        assert_eq!(loaded.open_global_notes, vec!["global-1"]);
    }

    #[test]
    fn clear_session_removes_file() {
        let (_dir, _guard) = setup();
        let session = Session {
            open_context_notes: vec!["note-1".to_string()],
            open_global_notes: vec![],
        };
        save_session("steam", &session).unwrap();
        clear_session("steam").unwrap();
        let loaded = load_session("steam").unwrap();
        assert!(loaded.open_context_notes.is_empty());
    }
}
