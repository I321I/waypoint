use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::hotkey::open_list_window;

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "開啟 Waypoint", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "設定", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let help_item = MenuItem::with_id(app, "help", "使用說明", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "結束 Waypoint", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &open_item, &sep1,
        &settings_item, &sep2,
        &help_item, &sep3,
        &quit_item,
    ])?;

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
    .build()?;
    Ok(())
}

#[tauri::command]
pub fn cmd_open_help(app: tauri::AppHandle) -> Result<(), String> {
    // 必須在主執行緒建立視窗，避免 WebView2 白屏競態
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        let _ = open_help_window(&app2);
    }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_open_settings(app: tauri::AppHandle) -> Result<(), String> {
    // 必須在主執行緒建立視窗，避免 WebView2 白屏競態
    let app2 = app.clone();
    app.run_on_main_thread(move || {
        let _ = open_settings_window(&app2);
    }).map_err(|e| e.to_string())
}
