//! 偵測 OS 前景視窗變化，自動切換 active context。
//!
//! 設計：跨平台用 200ms 輪詢 `detector::get_focused_window()`。事件式
//! hook（X11 _NET_ACTIVE_WINDOW、Win SetWinEventHook、macOS NSWorkspace）
//! 各平台 API 差異大，輪詢實作最簡且 detector 已抽象掉跨平台差異。
//!
//! 自家視窗識別：detector 取得的 process_name 比對「本程式 process name」。
//! 比對 PID 較精確但 detector 內部已以 PID 推導 process_name，這裡用 name
//! 是雙重保險（不過於 Linux _NET_WM_PID 可能找不到 PID，detector 會回傳空
//! process_name，此情況視為非自家，不過濾）。
//!
//! 變化偵測：active_context_id 與 derive 出的新 ctx 不同才 emit，避免每次
//! 輪詢都 flood 事件。

use crate::context::derive_context_id;
use crate::context::detector::get_focused_window;
use crate::storage::app_config;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// 啟動前景視窗監聽 thread。lib.rs setup() 呼叫一次。
pub fn start(app: AppHandle) {
    let self_proc = current_process_name();
    crate::write_log_line(&format!(
        "foreground_watcher started; self process_name={self_proc}"
    ));
    thread::spawn(move || run(app, self_proc));
}

fn current_process_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_default()
}

/// 比對「對方視窗的 process_name」是否屬於自家。
/// 注意：current_exe.file_name() 在 Linux 通常是 "waypoint"，在 Win 是 "waypoint.exe"。
/// detector 在 Linux 用 /proc/<pid>/comm（無副檔名），在 Win 用 QueryFullProcessImageNameW.file_name。
/// 兩端 stem 比對：去掉 .exe 後比 lowercase。
fn is_self(self_name: &str, other: &str) -> bool {
    if other.is_empty() { return false; }
    fn stem(s: &str) -> String {
        s.trim_end_matches(".exe").to_lowercase()
    }
    stem(self_name) == stem(other)
}

fn run(app: AppHandle, self_proc: String) {
    let mut last_ctx: Option<String> = None;
    loop {
        thread::sleep(Duration::from_millis(200));
        let info = match get_focused_window() {
            Some(i) => i,
            None => continue,
        };
        if is_self(&self_proc, &info.process_name) {
            // 自家視窗，不換 context（保留先前狀態）
            continue;
        }
        let config = app_config::load().unwrap_or_default();
        let ctx_id = derive_context_id(&info, &config);

        let state = app.state::<crate::state::AppState>();
        let changed = {
            let mut active = state.active_context_id.lock().unwrap();
            let old = active.clone();
            if old.as_deref() == Some(&ctx_id) {
                false
            } else {
                *active = Some(ctx_id.clone());
                *state.active_window_info.lock().unwrap() = Some(info.clone());
                last_ctx = Some(ctx_id.clone());
                let _ = old;
                true
            }
        };
        if changed {
            let _ = app.emit("waypoint://active-context-changed", &ctx_id);
        }
        let _ = last_ctx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_self_matches_with_or_without_exe() {
        assert!(is_self("waypoint", "waypoint"));
        assert!(is_self("waypoint.exe", "waypoint"));
        assert!(is_self("waypoint", "waypoint.exe"));
        assert!(is_self("Waypoint", "waypoint"));
    }

    #[test]
    fn is_self_rejects_other_apps() {
        assert!(!is_self("waypoint", "firefox"));
        assert!(!is_self("waypoint.exe", "chrome.exe"));
    }

    #[test]
    fn is_self_handles_empty() {
        assert!(!is_self("waypoint", ""));
    }
}
