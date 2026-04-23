use crate::error::WaypointError;
use crate::storage::paths::{context_dir, global_dir, note_dir};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSettings {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    /// R4 已移除筆記專屬快捷鍵；保留欄位僅為向後相容（讀取舊 settings.json 不報錯）。
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default)]
    pub window_bounds: Option<WindowBounds>,
    #[serde(default)]
    pub passthrough: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn default_font_size() -> u32 { 14 }
fn default_opacity() -> f32 { 1.0 }

impl Default for NoteSettings {
    fn default() -> Self {
        NoteSettings {
            font_size: default_font_size(),
            opacity: default_opacity(),
            hotkey: None,
            window_bounds: None,
            passthrough: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub context_id: Option<String>,
    pub title: String,
    pub content: String,
    pub settings: NoteSettings,
}

pub fn create_note(context_id: Option<&str>, title: &str) -> Result<Note, WaypointError> {
    let id = Uuid::new_v4().to_string();
    let note = Note {
        id: id.clone(),
        context_id: context_id.map(|s| s.to_string()),
        title: title.to_string(),
        content: String::new(),
        settings: NoteSettings::default(),
    };
    let dir = note_dir(context_id, &id);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("content.md"), &note.content)?;
    let settings_json = serde_json::to_string_pretty(&note.settings)?;
    std::fs::write(dir.join("settings.json"), settings_json)?;
    Ok(note)
}

pub fn read_note(context_id: Option<&str>, note_id: &str) -> Result<Note, WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let content = std::fs::read_to_string(dir.join("content.md"))?;
    let settings_str = std::fs::read_to_string(dir.join("settings.json"))
        .unwrap_or_else(|_| "{}".to_string());
    let settings: NoteSettings = serde_json::from_str(&settings_str)?;
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    Ok(Note {
        id: note_id.to_string(),
        context_id: context_id.map(|s| s.to_string()),
        title,
        content,
        settings,
    })
}

pub fn save_content(context_id: Option<&str>, note_id: &str, content: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    std::fs::write(dir.join("content.md"), content)?;
    Ok(())
}

pub fn save_settings(context_id: Option<&str>, note_id: &str, settings: &NoteSettings) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(dir.join("settings.json"), json)?;
    Ok(())
}

pub fn delete_note(context_id: Option<&str>, note_id: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn list_notes(context_id: Option<&str>) -> Result<Vec<Note>, WaypointError> {
    let dir = match context_id {
        Some(ctx) => context_dir(ctx),
        None => global_dir(),
    };
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut notes = vec![];
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let note_id = path.file_name().unwrap().to_str().unwrap().to_string();
            if let Ok(note) = read_note(context_id, &note_id) {
                notes.push(note);
            }
        }
    }
    Ok(notes)
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
    fn create_and_read_global_note() {
        let (_dir, _guard) = setup();
        let note = create_note(None, "Test Note").unwrap();
        assert!(!note.id.is_empty());
        assert_eq!(note.context_id, None);
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.id, note.id);
    }

    #[test]
    fn create_and_read_context_note() {
        let (_dir, _guard) = setup();
        let note = create_note(Some("steam"), "Steam Note").unwrap();
        assert_eq!(note.context_id, Some("steam".to_string()));
        let loaded = read_note(Some("steam"), &note.id).unwrap();
        assert_eq!(loaded.context_id, Some("steam".to_string()));
    }

    #[test]
    fn save_and_read_content() {
        let (_dir, _guard) = setup();
        let note = create_note(None, "Note").unwrap();
        save_content(None, &note.id, "# Hello\nworld").unwrap();
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.content, "# Hello\nworld");
        assert_eq!(loaded.title, "Hello");
    }

    #[test]
    fn delete_note_removes_dir() {
        let (_dir, _guard) = setup();
        let note = create_note(None, "To Delete").unwrap();
        delete_note(None, &note.id).unwrap();
        assert!(read_note(None, &note.id).is_err());
    }

    #[test]
    fn list_notes_returns_all_in_context() {
        let (_dir, _guard) = setup();
        create_note(Some("steam"), "Note 1").unwrap();
        create_note(Some("steam"), "Note 2").unwrap();
        let notes = list_notes(Some("steam")).unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn save_settings_persists() {
        let (_dir, _guard) = setup();
        let note = create_note(None, "Note").unwrap();
        let mut settings = NoteSettings::default();
        settings.font_size = 18;
        settings.opacity = 0.8;
        save_settings(None, &note.id, &settings).unwrap();
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.settings.font_size, 18);
        assert!((loaded.settings.opacity - 0.8).abs() < 0.001);
    }

    #[test]
    fn note_settings_deserializes_without_passthrough_field() {
        let json = r#"{"fontSize":14,"opacity":1.0,"hotkey":null,"windowBounds":null}"#;
        let s: NoteSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.passthrough, false);
    }
}
