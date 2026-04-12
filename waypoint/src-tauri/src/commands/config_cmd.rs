use crate::error::WaypointError;
use crate::storage::app_config::{self, AppConfig};
use crate::storage::autostart;

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
