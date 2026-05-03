//! 全應用層級的「重啟恢復 session」。
//!
//! 跟 per-context 的 session.rs 不同：app_session 記錄「當下所有已開啟的筆記視窗」，
//! 讓使用者改快捷鍵後可以重啟並還原視窗，不用一個個重新點開。
//!
//! 寫入時機：cmd_restart_app 重啟前。
//! 讀取時機：app 啟動時（lib.rs setup），讀完即刪檔，避免下次正常啟動又還原。
use crate::error::WaypointError;
use crate::storage::paths::data_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OpenNoteRef {
    pub note_id: String,
    pub context_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSession {
    #[serde(default)]
    pub open_notes: Vec<OpenNoteRef>,
    #[serde(default)]
    pub list_open: bool,
}

fn app_session_path() -> PathBuf {
    data_dir().join("app_session.json")
}

pub fn save(session: &AppSession) -> Result<(), WaypointError> {
    let path = app_session_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(session)?)?;
    Ok(())
}

/// 讀取並刪檔：只在啟動後恢復一次，避免使用者下次正常啟動時又被恢復。
pub fn take() -> Option<AppSession> {
    let path = app_session_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let mut sess: AppSession = serde_json::from_str(&content).ok()?;
    sess.open_notes.retain(|r| crate::storage::notes::note_exists(r.context_id.as_deref(), &r.note_id));
    let _ = std::fs::remove_file(&path);
    Some(sess)
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
    fn save_then_take_roundtrip() {
        let (_dir, _guard) = setup();
        use crate::storage::notes;
        let note_a = notes::create_note(None, "a").unwrap();
        let note_b = notes::create_note(Some("steam"), "b").unwrap();
        let s = AppSession {
            open_notes: vec![
                OpenNoteRef { note_id: note_a.id.clone(), context_id: None },
                OpenNoteRef { note_id: note_b.id.clone(), context_id: Some("steam".into()) },
            ],
            list_open: true,
        };
        save(&s).unwrap();
        let loaded = take().unwrap();
        assert_eq!(loaded.open_notes.len(), 2);
        assert_eq!(loaded.open_notes[1].context_id.as_deref(), Some("steam"));
        assert!(loaded.list_open);
    }

    #[test]
    fn take_filters_missing_notes() {
        let (_dir, _guard) = setup();
        use crate::storage::notes;
        let n = notes::create_note(None, "alive").unwrap();
        let s = AppSession {
            open_notes: vec![
                OpenNoteRef { note_id: n.id.clone(), context_id: None },
                OpenNoteRef { note_id: "ghost".into(), context_id: None },
            ],
            list_open: false,
        };
        save(&s).unwrap();

        let loaded = take().unwrap();
        assert_eq!(loaded.open_notes.len(), 1);
        assert_eq!(loaded.open_notes[0].note_id, n.id);
    }

    #[test]
    fn take_consumes_file() {
        let (_dir, _guard) = setup();
        save(&AppSession { open_notes: vec![], list_open: false }).unwrap();
        assert!(take().is_some());
        assert!(take().is_none(), "take() 應在讀取後刪檔");
    }

    #[test]
    fn take_returns_none_when_no_file() {
        let (_dir, _guard) = setup();
        assert!(take().is_none());
    }
}
