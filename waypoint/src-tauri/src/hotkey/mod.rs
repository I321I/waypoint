use crate::context::derive_context_id;
use crate::context::detector::get_focused_window;
use crate::state::AppState;
use crate::storage::app_config;
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder, WebviewUrl};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, PartialEq)]
pub enum HotkeyAction {
    OpenAll,
    OpenList,
    CollapseAll,
}

pub fn determine_action(list_open: bool, any_note_open: bool) -> HotkeyAction {
    if list_open {
        HotkeyAction::CollapseAll
    } else if any_note_open {
        HotkeyAction::OpenList
    } else {
        HotkeyAction::OpenAll
    }
}

pub fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        let window_info = get_focused_window();
        let state = app.state::<AppState>();
        let list_open = *state.list_window_open.lock().unwrap();
        let any_note_open = app.webview_windows()
            .keys()
            .any(|label| label.starts_with("note-"));
        let action = determine_action(list_open, any_note_open);
        match action {
            HotkeyAction::OpenAll => {
                if let Some(info) = window_info {
                    let config = app_config::load().unwrap_or_default();
                    let ctx_id = derive_context_id(&info, &config);
                    *state.active_context_id.lock().unwrap() = Some(ctx_id);
                    *state.active_window_info.lock().unwrap() = Some(info);
                }
                let _ = open_list_window(app);
            }
            HotkeyAction::OpenList => {
                let _ = open_list_window(app);
            }
            HotkeyAction::CollapseAll => {
                // Emit event so frontend can save session before collapsing
                let _ = app.emit("waypoint://collapse-all-requested", ());
                // Also collapse after a brief delay to handle case
                // where list window is not mounted (safety fallback)
                let app2 = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    // Only collapse if list window still exists (frontend may have already handled it)
                    if let Some(list) = app2.get_webview_window("list") {
                        if list.is_visible().unwrap_or(false) {
                            collapse_all_waypoint_windows(&app2);
                        }
                    }
                });
            }
        }
    })?;
    Ok(())
}

pub fn open_list_window(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    if let Some(win) = app.get_webview_window("list") {
        win.show()?;
        win.set_focus()?;
        *state.list_window_open.lock().unwrap() = true;
        return Ok(());
    }
    let win = WebviewWindowBuilder::new(app, "list", WebviewUrl::App("/#view=list".into()))
        .title("Waypoint")
        .inner_size(220.0, 500.0)
        .min_inner_size(180.0, 300.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build()?;
    *state.list_window_open.lock().unwrap() = true;
    // 不使用 focus-based auto-hide：
    // 1. 拖曳時 Windows 會暫時奪走 focus，auto-hide 會中斷拖曳
    // 2. 開啟說明/設定視窗時也會觸發，造成列表意外消失
    // 使用者改靠快捷鍵（CollapseAll）、⇊ 或 ✕ 來關閉列表
    Ok(())
}

pub fn collapse_all_waypoint_windows(app: &AppHandle) {
    let windows = app.webview_windows();
    for (label, window) in &windows {
        if label.starts_with("note-") || label == "list" {
            let _ = window.hide();
        }
    }
    let state = app.state::<AppState>();
    *state.list_window_open.lock().unwrap() = false;
}

pub fn open_note_window(app: &AppHandle, note_id: &str, context_id: Option<&str>) -> tauri::Result<()> {
    let label = format!("note-{}", note_id);
    if let Some(win) = app.get_webview_window(&label) {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    let ctx_param = context_id.map(|c| format!("&contextId={}", c)).unwrap_or_default();
    let url = format!("/#view=note&noteId={}{}", note_id, ctx_param);
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title("Waypoint Note")
        .inner_size(420.0, 600.0)
        .min_inner_size(300.0, 200.0)
        .resizable(true)
        .decorations(false)
        .skip_taskbar(true)
        .build()?;
    Ok(())
}

pub fn register_note_hotkey(
    app: &AppHandle,
    hotkey: &str,
    note_id: String,
    context_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed { return; }
        let _ = open_note_window(app, &note_id, context_id.as_deref());
    })?;
    Ok(())
}

#[tauri::command]
pub fn cmd_open_note_window(app: AppHandle, note_id: String, context_id: Option<String>) -> Result<(), String> {
    // 必須在主執行緒建立視窗，否則 WebView2 的 PostMessageW 可能無法被即時處理 → 白屏
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        let _ = open_note_window(&app2, &note_id, context_id.as_deref());
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_collapse_all(app: AppHandle) {
    collapse_all_waypoint_windows(&app);
}

#[tauri::command]
pub fn cmd_close_note_window(app: AppHandle, note_id: String) -> Result<(), String> {
    let label = format!("note-{}", note_id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn cmd_register_note_hotkey(
    app: AppHandle,
    note_id: String,
    context_id: Option<String>,
    hotkey: String,
) -> Result<(), String> {
    register_note_hotkey(&app, &hotkey, note_id, context_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_unregister_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    app.global_shortcut()
        .unregister(hotkey.as_str())
        .map_err(|e| e.to_string())
}

/// 用 label 關閉視窗（close）——前端不依賴 getCurrentWindow()
#[tauri::command]
pub fn cmd_close_window(app: AppHandle, label: String) -> Result<(), String> {
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        if let Some(win) = app2.get_webview_window(&label) {
            let _ = win.close();
        }
    }).map_err(|e| e.to_string())
}

/// 用 label 隱藏視窗（hide）——前端不依賴 getCurrentWindow()
#[tauri::command]
pub fn cmd_hide_window(app: AppHandle, label: String) -> Result<(), String> {
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        if let Some(win) = app2.get_webview_window(&label) {
            let _ = win.hide();
            if label == "list" {
                let state = app2.state::<crate::state::AppState>();
                *state.list_window_open.lock().unwrap() = false;
            }
        }
    }).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_windows_open_triggers_open_all() {
        assert_eq!(determine_action(false, false), HotkeyAction::OpenAll);
    }

    #[test]
    fn notes_open_no_list_triggers_open_list() {
        assert_eq!(determine_action(false, true), HotkeyAction::OpenList);
    }

    #[test]
    fn list_open_triggers_collapse_all() {
        assert_eq!(determine_action(true, false), HotkeyAction::CollapseAll);
        assert_eq!(determine_action(true, true), HotkeyAction::CollapseAll);
    }
}
