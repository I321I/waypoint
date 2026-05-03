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
    let mut sess: Session = serde_json::from_str(&content)?;
    sess.open_context_notes.retain(|id| crate::storage::notes::note_exists(Some(context_id), id));
    sess.open_global_notes.retain(|id| crate::storage::notes::note_exists(None, id));
    Ok(sess)
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
        use crate::storage::notes;
        let n1 = notes::create_note(Some("steam"), "note-1").unwrap();
        let n2 = notes::create_note(Some("steam"), "note-2").unwrap();
        let g1 = notes::create_note(None, "global-1").unwrap();
        let session = Session {
            open_context_notes: vec![n1.id.clone(), n2.id.clone()],
            open_global_notes: vec![g1.id.clone()],
        };
        save_session("steam", &session).unwrap();
        let loaded = load_session("steam").unwrap();
        assert_eq!(loaded.open_context_notes, vec![n1.id, n2.id]);
        assert_eq!(loaded.open_global_notes, vec![g1.id]);
    }

    #[test]
    fn load_session_filters_missing_notes() {
        let (_dir, _guard) = setup();
        use crate::storage::notes;
        let note = notes::create_note(Some("ctx"), "alive").unwrap();
        let alive_id = note.id.clone();
        let s = Session {
            open_context_notes: vec![alive_id.clone(), "ghost".into()],
            open_global_notes: vec!["ghost-global".into()],
        };
        save_session("ctx", &s).unwrap();

        let loaded = load_session("ctx").unwrap();
        assert_eq!(loaded.open_context_notes, vec![alive_id]);
        assert!(loaded.open_global_notes.is_empty());
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
