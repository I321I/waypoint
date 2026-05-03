use crate::error::WaypointError;
use crate::storage::notes::{self, Note, NoteSettings};
use tauri::{AppHandle, Emitter};

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
pub async fn delete_note(
    app: AppHandle,
    context_id: Option<String>,
    note_id: String,
) -> Result<(), WaypointError> {
    notes::delete_note(context_id.as_deref(), &note_id)?;
    let _ = app.emit("waypoint://note-deleted", serde_json::json!({
        "noteId": note_id,
        "contextId": context_id,
    }));
    Ok(())
}

#[tauri::command]
pub fn rename_note(context_id: Option<String>, note_id: String, new_title: String) -> Result<(), WaypointError> {
    notes::rename_note(context_id.as_deref(), &note_id, &new_title)
}

#[tauri::command]
pub fn duplicate_note(src_context_id: Option<String>, src_note_id: String, dst_context_id: Option<String>) -> Result<Note, WaypointError> {
    notes::duplicate_note(src_context_id.as_deref(), &src_note_id, dst_context_id.as_deref())
}

#[tauri::command]
pub fn move_note(src_context_id: Option<String>, note_id: String, dst_context_id: Option<String>) -> Result<(), WaypointError> {
    notes::move_note(src_context_id.as_deref(), &note_id, dst_context_id.as_deref())
}

#[tauri::command]
pub fn get_note_order(context_id: Option<String>) -> Vec<String> {
    crate::storage::note_order::load(context_id.as_deref())
}

#[tauri::command]
pub fn set_note_order(context_id: Option<String>, order: Vec<String>) -> Result<(), WaypointError> {
    crate::storage::note_order::save(context_id.as_deref(), &order)
}
