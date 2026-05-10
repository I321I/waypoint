/// 前端寫 diag log。讓 ListWindow / NoteWindow 等 webview 端能把 listener 是否觸發、
/// 流程是否完成等 frontend-only 的事件寫進與 Rust 同一份 error.log，方便實機 debug
/// 時拿一份就能完整看到「Rust emit → frontend listen → frontend done」整條鏈。
#[tauri::command]
pub fn cmd_log_diag(scope: String, msg: String) {
    crate::write_log_line(&format!("[js:{scope}] {msg}"));
}
