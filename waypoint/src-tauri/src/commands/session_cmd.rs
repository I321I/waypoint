use crate::error::WaypointError;
use crate::storage::session::{self, Session};

#[tauri::command]
pub fn load_session(context_id: String) -> Result<Session, WaypointError> {
    session::load_session(&context_id)
}

#[tauri::command]
pub fn save_session(context_id: String, sess: Session) -> Result<(), WaypointError> {
    session::save_session(&context_id, &sess)
}
