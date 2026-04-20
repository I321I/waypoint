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
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default)]
    pub window_bounds: Option<WindowBounds>,
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
    // 套用使用者自訂順序（拖曳排序）；新 note 附加尾端。
    let order = crate::storage::note_order::load(context_id);
    let ordered = crate::storage::note_order::apply_order(&order, notes, |n| n.id.as_str());
    Ok(ordered)
}

/// 重新命名：將 content 首行的 "# title" 換成新的標題（無則插入）。
pub fn rename_note(context_id: Option<&str>, note_id: &str, new_title: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let content = std::fs::read_to_string(dir.join("content.md")).unwrap_or_default();
    let rest = if let Some(nl) = content.find('\n') {
        let first = &content[..nl];
        if first.starts_with("# ") {
            content[nl + 1..].to_string()
        } else {
            content.clone()
        }
    } else if content.starts_with("# ") {
        String::new()
    } else {
        content.clone()
    };
    let merged = if new_title.trim().is_empty() {
        rest
    } else {
        format!("# {}\n{}", new_title.trim(), rest)
    };
    std::fs::write(dir.join("content.md"), merged)?;
    Ok(())
}

/// 複製筆記到指定 context（含 global）。產生新 UUID；content / settings 一起複製。
/// 會把新 id 附加到目標 context 的 order 尾端。
pub fn duplicate_note(src_context_id: Option<&str>, src_note_id: &str, dst_context_id: Option<&str>) -> Result<Note, WaypointError> {
    let src_dir = note_dir(src_context_id, src_note_id);
    if !src_dir.exists() {
        return Err(WaypointError::NoteNotFound(src_note_id.to_string()));
    }
    let new_id = Uuid::new_v4().to_string();
    let dst_dir = note_dir(dst_context_id, &new_id);
    std::fs::create_dir_all(&dst_dir)?;
    let content = std::fs::read_to_string(src_dir.join("content.md")).unwrap_or_default();
    std::fs::write(dst_dir.join("content.md"), &content)?;
    let settings_str = std::fs::read_to_string(src_dir.join("settings.json"))
        .unwrap_or_else(|_| "{}".to_string());
    // 清掉 hotkey：避免兩個 note 共用同一個快捷鍵造成衝突。
    let mut settings: NoteSettings = serde_json::from_str(&settings_str).unwrap_or_default();
    settings.hotkey = None;
    std::fs::write(dst_dir.join("settings.json"), serde_json::to_string_pretty(&settings)?)?;
    // 附加到目標 order 尾端
    let mut order = crate::storage::note_order::load(dst_context_id);
    order.push(new_id.clone());
    let _ = crate::storage::note_order::save(dst_context_id, &order);
    read_note(dst_context_id, &new_id)
}

/// 把筆記搬到另一個 context（保留 UUID）。會同步更新 order：從來源移除，加到目標尾端。
pub fn move_note(src_context_id: Option<&str>, note_id: &str, dst_context_id: Option<&str>) -> Result<(), WaypointError> {
    if src_context_id == dst_context_id {
        return Ok(()); // noop
    }
    let src_dir = note_dir(src_context_id, note_id);
    if !src_dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let dst_dir = note_dir(dst_context_id, note_id);
    if let Some(parent) = dst_dir.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // 先嘗試 rename（同 fs 情況下最快）。跨 fs 失敗時退回 copy + remove。
    if std::fs::rename(&src_dir, &dst_dir).is_err() {
        copy_dir_recursive(&src_dir, &dst_dir)?;
        std::fs::remove_dir_all(&src_dir)?;
    }
    // 更新 order
    let mut src_order = crate::storage::note_order::load(src_context_id);
    src_order.retain(|id| id != note_id);
    let _ = crate::storage::note_order::save(src_context_id, &src_order);
    let mut dst_order = crate::storage::note_order::load(dst_context_id);
    if !dst_order.contains(&note_id.to_string()) {
        dst_order.push(note_id.to_string());
    }
    let _ = crate::storage::note_order::save(dst_context_id, &dst_order);
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
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
    fn rename_note_replaces_title_heading() {
        let (_d, _g) = setup();
        let n = create_note(None, "X").unwrap();
        save_content(None, &n.id, "# Old\nbody").unwrap();
        rename_note(None, &n.id, "New Title").unwrap();
        let reloaded = read_note(None, &n.id).unwrap();
        assert_eq!(reloaded.title, "New Title");
        assert!(reloaded.content.contains("body"));
    }

    #[test]
    fn rename_note_inserts_heading_when_missing() {
        let (_d, _g) = setup();
        let n = create_note(None, "X").unwrap();
        save_content(None, &n.id, "plain body").unwrap();
        rename_note(None, &n.id, "T").unwrap();
        let reloaded = read_note(None, &n.id).unwrap();
        assert_eq!(reloaded.title, "T");
        assert!(reloaded.content.contains("plain body"));
    }

    #[test]
    fn duplicate_note_creates_new_uuid() {
        let (_d, _g) = setup();
        let n = create_note(None, "orig").unwrap();
        save_content(None, &n.id, "# A\nb").unwrap();
        let dup = duplicate_note(None, &n.id, None).unwrap();
        assert_ne!(dup.id, n.id);
        assert!(dup.content.contains("# A"));
        // dup 應該附加到 order 尾端
        let order = crate::storage::note_order::load(None);
        assert!(order.contains(&dup.id));
    }

    #[test]
    fn duplicate_note_to_different_context() {
        let (_d, _g) = setup();
        let n = create_note(None, "g").unwrap();
        let dup = duplicate_note(None, &n.id, Some("steam")).unwrap();
        assert_eq!(dup.context_id.as_deref(), Some("steam"));
        // 原 note 仍在 global
        assert!(read_note(None, &n.id).is_ok());
    }

    #[test]
    fn move_note_changes_context() {
        let (_d, _g) = setup();
        let n = create_note(None, "g").unwrap();
        move_note(None, &n.id, Some("steam")).unwrap();
        assert!(read_note(None, &n.id).is_err());
        assert!(read_note(Some("steam"), &n.id).is_ok());
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
}
