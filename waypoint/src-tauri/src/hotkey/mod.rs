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
        // OpenAll / OpenList 都要重新以「當前前景視窗」推導 context，
        // 否則列表會一直停留在第一次叫出時的 context（例如 msedge）。
        if matches!(action, HotkeyAction::OpenAll | HotkeyAction::OpenList) {
            if let Some(info) = window_info.clone() {
                let config = app_config::load().unwrap_or_default();
                let ctx_id = derive_context_id(&info, &config);
                *state.active_context_id.lock().unwrap() = Some(ctx_id);
                *state.active_window_info.lock().unwrap() = Some(info);
            }
        }
        match action {
            HotkeyAction::OpenAll => {
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
        win.unminimize().ok();
        win.show()?;
        win.set_focus()?;
        *state.list_window_open.lock().unwrap() = true;
        // 通知前端重新載入 context / session（再叫出時也會套用新 context）
        let _ = app.emit("waypoint://list-shown", ());
        return Ok(());
    }
    let _win = WebviewWindowBuilder::new(app, "list", WebviewUrl::App("/#view=list".into()))
        .title("Waypoint")
        .inner_size(220.0, 500.0)
        .min_inner_size(180.0, 300.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build()?;
    *state.list_window_open.lock().unwrap() = true;
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

// async：見 tray::cmd_open_help 的註解。WebView2 上 sync command + 同步建
// webview 會 deadlock 原 webview 的 IPC，造成兩邊白屏。
#[tauri::command]
pub async fn cmd_open_note_window(app: AppHandle, note_id: String, context_id: Option<String>) -> Result<(), String> {
    open_note_window(&app, &note_id, context_id.as_deref()).map_err(|e| e.to_string())
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
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 用 label 隱藏視窗（hide）——前端不依賴 getCurrentWindow()
#[tauri::command]
pub fn cmd_hide_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.hide().map_err(|e| e.to_string())?;
        if label == "list" {
            let state = app.state::<crate::state::AppState>();
            *state.list_window_open.lock().unwrap() = false;
        }
    }
    Ok(())
}

/// 最小化視窗
#[tauri::command]
pub fn cmd_minimize_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 切換最大化狀態
#[tauri::command]
pub fn cmd_toggle_maximize(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        if win.is_maximized().unwrap_or(false) {
            win.unmaximize().map_err(|e| e.to_string())?;
        } else {
            win.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 完全結束 Waypoint
#[tauri::command]
pub fn cmd_exit_app(app: AppHandle) {
    app.exit(0);
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

    /// R8: list autohide-on-blur 已移除（避免拖曳時被吃掉焦點而瞬間消失）。
    /// 編譯成功本身就是主要保證；此測試用源碼指紋雙保險。
    #[test]
    fn no_list_autohide_function() {
        let src = include_str!("mod.rs");
        // production 程式中的函式定義字串（拆開以免測試本身誤判）。
        let needle = concat!("fn ", "attach", "_list_autohide");
        assert!(
            !src.contains(needle),
            "list autohide 函式應已移除（R8）"
        );
    }
}
