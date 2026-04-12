mod commands;
mod context;
mod error;
mod hotkey;
mod state;
mod storage;
mod tray;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_notes,
            commands::notes::create_note,
            commands::notes::read_note,
            commands::notes::save_content,
            commands::notes::save_note_settings,
            commands::notes::delete_note,
            commands::context_cmd::get_active_context,
            commands::context_cmd::set_context_match_by,
            commands::context_cmd::set_context_alias,
            commands::context_cmd::rename_context,
            commands::context_cmd::delete_context,
            commands::context_cmd::list_contexts,
            commands::session_cmd::load_session,
            commands::session_cmd::save_session,
            commands::config_cmd::get_app_config,
            commands::config_cmd::set_hotkey,
            commands::config_cmd::get_autostart,
            commands::config_cmd::is_autostart_supported,
            commands::config_cmd::set_autostart,
            hotkey::cmd_open_note_window,
            hotkey::cmd_collapse_all,
            hotkey::cmd_close_note_window,
            hotkey::cmd_register_note_hotkey,
            hotkey::cmd_unregister_hotkey,
            tray::cmd_open_help,
            tray::cmd_open_settings,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            let config = storage::app_config::load().unwrap_or_default();
            hotkey::register_hotkey(app.handle(), &config.hotkey)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Waypoint");
}
