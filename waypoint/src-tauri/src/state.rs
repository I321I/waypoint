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
    /// 進入穿透模式前 list 是否開著，用以離開穿透時還原。
    pub pre_passthrough_list_open: AtomicBool,
    /// 開啟中的筆記視窗：noteId -> contextId（None=global）。close-requested 時用以 emit 正確 isGlobal。
    pub open_notes_context: Mutex<HashMap<String, Option<String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active_context_id: Mutex::new(None),
            active_window_info: Mutex::new(None),
            list_window_open: Mutex::new(false),
            passthrough_state: Mutex::new(HashMap::new()),
            passthrough_hotkey_registered: AtomicBool::new(true),
            pre_passthrough_list_open: AtomicBool::new(false),
            open_notes_context: Mutex::new(HashMap::new()),
        }
    }
}
