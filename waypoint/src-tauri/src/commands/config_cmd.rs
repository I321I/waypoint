use crate::error::WaypointError;
use crate::storage::app_config::{self, AppConfig, ContextConfig};
use crate::storage::autostart;
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigDto {
    pub hotkey: String,
    pub context_aliases: HashMap<String, String>,
    pub contexts: HashMap<String, ContextConfig>,
    pub passthrough_hotkey: String,
    pub show_in_taskbar: bool,
    pub transparent_includes_text: bool,
    #[serde(rename = "passthroughHotkeyRegistered")]
    pub passthrough_hotkey_registered: bool,
}

#[tauri::command]
pub fn get_app_config(app: AppHandle) -> Result<AppConfigDto, WaypointError> {
    let cfg: AppConfig = app_config::load()?;
    let registered = app
        .state::<crate::state::AppState>()
        .passthrough_hotkey_registered
        .load(std::sync::atomic::Ordering::SeqCst);
    Ok(AppConfigDto {
        hotkey: cfg.hotkey,
        context_aliases: cfg.context_aliases,
        contexts: cfg.contexts,
        passthrough_hotkey: cfg.passthrough_hotkey,
        show_in_taskbar: cfg.show_in_taskbar,
        transparent_includes_text: cfg.transparent_includes_text,
        passthrough_hotkey_registered: registered,
    })
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
pub fn get_transparent_includes_text() -> Result<bool, String> {
    let cfg = crate::storage::app_config::load().map_err(|e| e.to_string())?;
    Ok(cfg.transparent_includes_text)
}

#[tauri::command]
pub fn set_transparent_includes_text(value: bool) -> Result<(), String> {
    let mut cfg = crate::storage::app_config::load().map_err(|e| e.to_string())?;
    cfg.transparent_includes_text = value;
    crate::storage::app_config::save(&cfg).map_err(|e| e.to_string())
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
