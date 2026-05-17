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
    /// 每個 context 最後編輯的筆記 id：context_id（None=global → "_global_only_"）→ note_id。
    /// 用途：穿透模式關閉時，自動把焦點 + 游標移到該 note 最後，方便 user 接著打字。
    pub last_edited_per_context: Mutex<HashMap<String, String>>,
    /// active_mode：使用者呼叫 Waypoint 後為 true，foreground_watcher 才會發
    /// active-context-changed event 給前端切換筆記區域。手動收起（hotkey
    /// CollapseAll / list ✕）後為 false，使用者切換 app 不再觸發筆記跳動。
    /// 啟動時預設 true（vc 用戶體感「剛開就應該開始追蹤」）。
    pub active_mode: AtomicBool,
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
            last_edited_per_context: Mutex::new(HashMap::new()),
            active_mode: AtomicBool::new(true),
        }
    }
}
