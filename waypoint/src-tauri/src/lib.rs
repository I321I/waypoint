mod commands;
mod context;
mod error;
// mod hotkey;  // TODO Task 7
mod state;
mod storage;
// mod tray;    // TODO Task 8

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
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running Waypoint");
}
