use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager,
};

use crate::hotkey::open_list_window;

/// 用於從事件 listener 更新 tray 穿透選單項目文字
pub struct PassthroughMenuItem<R: tauri::Runtime>(pub MenuItem<R>);

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "開啟 Waypoint", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
    let passthrough_item = MenuItem::with_id(app, "passthrough", "○ 穿透：關", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let help_item = MenuItem::with_id(app, "help", "使用說明", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "結束 Waypoint", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &open_item, &sep1,
        &settings_item, &passthrough_item, &sep2,
        &help_item, &sep3,
        &quit_item,
    ])?;

    // 把 passthrough_item clone 存入 app state，供 listener 更新文字
    let passthrough_item_clone = passthrough_item.clone();
    app.manage(Mutex::new(PassthroughMenuItem(passthrough_item_clone)));

    // 訂閱穿透狀態變化事件，更新選單文字
    let app_handle = app.app_handle().clone();
    app.listen("waypoint://passthrough-changed", move |_event| {
        let state = app_handle.state::<crate::state::AppState>();
        let map = state.passthrough_state.lock().unwrap();
        let any_on = map.values().any(|&v| v);
        drop(map);
        let text = if any_on { "● 穿透：開" } else { "○ 穿透：關" };
        if let Some(item_state) = app_handle.try_state::<Mutex<PassthroughMenuItem<tauri::Wry>>>() {
            let guard = item_state.lock().unwrap();
            let _ = guard.0.set_text(text);
        }
    });

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        // Windows：左鍵開視窗，右鍵才跑選單
        // Linux/macOS：左鍵保持平台預設（GTK/KDE 通常左鍵顯示選單）
        .show_menu_on_left_click(cfg!(not(target_os = "windows")))
        .tooltip("Waypoint")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                let _ = open_list_window(app);
            }
            "settings" => {
                let _ = open_settings_window(app);
            }
            "passthrough" => {
                let _ = crate::commands::passthrough_cmd::cmd_toggle_passthrough_global(app.clone());
            }
            "help" => {
                let _ = open_help_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 左鍵單擊 tray icon 直接開啟列表
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = open_list_window(app);
            }
        })
        .build(app)?;

    Ok(())
}

pub fn open_help_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("help") {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        app,
        "help",
        tauri::WebviewUrl::App("/#view=help".into()),
    )
    .title("Waypoint — 使用說明")
    .inner_size(600.0, 500.0)
    .resizable(true)
    .decorations(false)
    .build()?;
    Ok(())
}

pub fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("settings") {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("/#view=settings".into()),
    )
    .title("Waypoint — 設定")
    .inner_size(400.0, 300.0)
    .resizable(false)
    .decorations(false)
    .build()?;
    Ok(())
}

// async：sync command 在 main thread 跑，會被 WebviewWindowBuilder::build
// 阻塞 → 原 webview 的 IPC reply 卡住 → "Timed out receiving message from
// renderer" → 兩邊白屏（WebView2 特有）。
// async 命令在 tokio thread 跑，build 內部會 dispatch 回 main thread，
// 不阻塞原 webview 的 IPC channel。
#[tauri::command]
pub async fn cmd_open_help(app: tauri::AppHandle) -> Result<(), String> {
    open_help_window(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_open_settings(app: tauri::AppHandle) -> Result<(), String> {
    open_settings_window(&app).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn tray_menu_includes_passthrough_item() {
        let src = include_str!("./mod.rs");
        assert!(src.contains(concat!("\"", "passthrough", "\"")));
        assert!(src.contains("穿透"));
    }
}
