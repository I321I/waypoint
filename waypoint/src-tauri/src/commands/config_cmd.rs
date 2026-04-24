use crate::error::WaypointError;
use crate::storage::app_config::{self, AppConfig};
use crate::storage::autostart;
use tauri::AppHandle;

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, WaypointError> {
    app_config::load()
}

#[tauri::command]
pub fn set_hotkey(hotkey: String) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    config.hotkey = hotkey;
    app_config::save(&config)
}

#[tauri::command]
pub fn get_autostart() -> bool {
    autostart::is_enabled()
}

#[tauri::command]
pub fn is_autostart_supported() -> bool {
    autostart::is_supported()
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    autostart::set_enabled(enabled)
}

#[tauri::command]
pub fn cmd_set_show_in_taskbar(show: bool) -> Result<(), String> {
    let mut config = app_config::load().map_err(|e| e.to_string())?;
    config.show_in_taskbar = show;
    app_config::save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_set_passthrough_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    let mut config = app_config::load().map_err(|e| e.to_string())?;
    let old_hotkey = config.passthrough_hotkey.clone();
    config.passthrough_hotkey = hotkey.clone();
    app_config::save(&config).map_err(|e| e.to_string())?;
    crate::hotkey::reregister_passthrough_hotkey(&app, &old_hotkey, &hotkey)
        .map_err(|e| e.to_string())
}
