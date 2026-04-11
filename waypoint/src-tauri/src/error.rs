use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum WaypointError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Note not found: {0}")]
    NoteNotFound(String),
    #[error("Context not found: {0}")]
    ContextNotFound(String),
}

impl From<std::io::Error> for WaypointError {
    fn from(e: std::io::Error) -> Self {
        WaypointError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for WaypointError {
    fn from(e: serde_json::Error) -> Self {
        WaypointError::Json(e.to_string())
    }
}

// Note: tauri already implements From<T> for InvokeError where T: Serialize,
// so WaypointError (which derives Serialize) is automatically usable as a
// Tauri command return error without an explicit impl.
