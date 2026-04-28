use crate::context::detector::FocusedWindowInfo;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub active_context_id: Mutex<Option<String>>,
    pub active_window_info: Mutex<Option<FocusedWindowInfo>>,
    pub list_window_open: Mutex<bool>,
    pub passthrough_state: Mutex<HashMap<String, bool>>,
    pub passthrough_hotkey_registered: AtomicBool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active_context_id: Mutex::new(None),
            active_window_info: Mutex::new(None),
            list_window_open: Mutex::new(false),
            passthrough_state: Mutex::new(HashMap::new()),
            passthrough_hotkey_registered: AtomicBool::new(true),
        }
    }
}
