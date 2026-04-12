use crate::error::WaypointError;
use crate::storage::notes::{self, Note, NoteSettings};

#[tauri::command]
pub fn list_notes(context_id: Option<String>) -> Result<Vec<Note>, WaypointError> {
    notes::list_notes(context_id.as_deref())
}

#[tauri::command]
pub fn create_note(context_id: Option<String>, title: String) -> Result<Note, WaypointError> {
    notes::create_note(context_id.as_deref(), &title)
}

#[tauri::command]
pub fn read_note(context_id: Option<String>, note_id: String) -> Result<Note, WaypointError> {
    notes::read_note(context_id.as_deref(), &note_id)
}

#[tauri::command]
pub fn save_content(context_id: Option<String>, note_id: String, content: String) -> Result<(), WaypointError> {
    notes::save_content(context_id.as_deref(), &note_id, &content)
}

#[tauri::command]
pub fn save_note_settings(
    context_id: Option<String>,
    note_id: String,
    settings: NoteSettings,
) -> Result<(), WaypointError> {
    notes::save_settings(context_id.as_deref(), &note_id, &settings)
}

#[tauri::command]
pub fn delete_note(context_id: Option<String>, note_id: String) -> Result<(), WaypointError> {
    notes::delete_note(context_id.as_deref(), &note_id)
}
