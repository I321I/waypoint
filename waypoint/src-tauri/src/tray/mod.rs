use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let help_item = MenuItem::with_id(app, "help", "使用說明", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "結束 Waypoint", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&help_item, &sep, &quit_item])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Waypoint")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "help" => {
                let _ = open_help_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
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
        tauri::WebviewUrl::App("index.html?view=help".into()),
    )
    .title("Waypoint — 使用說明")
    .inner_size(600.0, 500.0)
    .resizable(true)
    .build()?;
    Ok(())
}
