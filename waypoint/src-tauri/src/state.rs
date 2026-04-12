use crate::context::detector::FocusedWindowInfo;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct AppState {
    pub active_context_id: Mutex<Option<String>>,
    pub active_window_info: Mutex<Option<FocusedWindowInfo>>,
    pub list_window_open: Mutex<bool>,
}
