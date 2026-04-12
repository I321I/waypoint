use crate::error::WaypointError;
use crate::storage::app_config::{self, AppConfig};

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
