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
    let self_pid = std::process::id();
    crate::write_log_line(&format!(
        "foreground_watcher started; self_pid={self_pid} self_proc={self_proc}"
    ));
    thread::spawn(move || run(app, self_proc, self_pid));
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

fn run(app: AppHandle, self_proc: String, self_pid: u32) {
    crate::write_log_line(&format!(
        "foreground_watcher run loop started; self_proc={self_proc}"
    ));
    let mut last_logged_proc: Option<String> = None;
    let mut none_count: u32 = 0;
    loop {
        thread::sleep(Duration::from_millis(200));
        let info = match get_focused_window() {
            Some(i) => i,
            None => {
                none_count = none_count.saturating_add(1);
                if none_count == 1 || none_count == 50 || none_count % 250 == 0 {
                    crate::write_log_line(&format!(
                        "foreground_watcher: get_focused_window returned None (count={none_count})"
                    ));
                }
                continue;
            }
        };
        none_count = 0;
        // process_name 為空 = detector 抓不到 PID 對應 process 名（Linux _NET_WM_PID 缺、
        // /proc/<pid>/comm 讀不到等）。略過避免把 active_context 設成 ""。
        if info.process_name.is_empty() {
            crate::write_log_line(&format!(
                "foreground_watcher: empty process_name (title={:?}); skip",
                info.window_title
            ));
            continue;
        }
        // 自家視窗識別優先用 PID 精確比對；fallback 用 process_name（涵蓋
        // detector 抓不到 PID 的場景，例如 Linux _NET_WM_PID 缺失）。
        // AppImage / Flatpak 環境下 process_name 不見得是 "waypoint"，PID 比對
        // 才是權威。
        if info.pid == Some(self_pid) || is_self(&self_proc, &info.process_name) {
            continue;
        }
        // 觀察用：每次 process_name 變化都 log（限同一 proc 不重複 log）
        if last_logged_proc.as_deref() != Some(&info.process_name) {
            crate::write_log_line(&format!(
                "foreground_watcher: foreground proc={} title={:?}",
                info.process_name, info.window_title
            ));
            last_logged_proc = Some(info.process_name.clone());
        }
        let config = app_config::load().unwrap_or_default();
        let ctx_id = derive_context_id(&info, &config);
        if ctx_id.is_empty() {
            continue;
        }

        let state = app.state::<crate::state::AppState>();
        let changed = {
            let mut active = state.active_context_id.lock().unwrap();
            if active.as_deref() == Some(&ctx_id) {
                false
            } else {
                *active = Some(ctx_id.clone());
                *state.active_window_info.lock().unwrap() = Some(info.clone());
                true
            }
        };
        if changed {
            crate::write_log_line(&format!(
                "active-context-changed: emit ctx_id={ctx_id}"
            ));
            let _ = app.emit("waypoint://active-context-changed", &ctx_id);
        }
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
