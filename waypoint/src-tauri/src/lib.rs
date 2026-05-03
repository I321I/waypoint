mod commands;
mod context;
mod error;
mod hotkey;
mod state;
mod storage;
pub mod taskbar;
mod tray;

use state::AppState;
use tauri::Listener;
use tauri::Manager;

/// 寫 panic / 重大錯誤訊息到 log 檔，Steam Deck 等無 console 環境下方便使用者回報。
///
/// 位置優先序：
/// 1. `$XDG_STATE_HOME/waypoint/error.log`
/// 2. `$HOME/.local/state/waypoint/error.log`
/// 3. `$HOME/waypoint/error.log`（最後 fallback，跟資料夾同位置）
fn resolve_log_path(xdg_state: Option<&std::ffi::OsStr>, home: Option<&std::ffi::OsStr>) -> Option<std::path::PathBuf> {
    let base = xdg_state
        .map(std::path::PathBuf::from)
        .or_else(|| home.map(|h| std::path::PathBuf::from(h).join(".local/state")))
        .or_else(|| home.map(std::path::PathBuf::from))?;
    Some(base.join("waypoint").join("error.log"))
}

fn waypoint_log_path() -> Option<std::path::PathBuf> {
    let xdg = std::env::var_os("XDG_STATE_HOME");
    let home = std::env::var_os("HOME");
    resolve_log_path(xdg.as_deref(), home.as_deref())
}

fn write_log_line(msg: &str) {
    if let Some(p) = waypoint_log_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
            let _ = writeln!(f, "[{}] {}", chrono_like_now(), msg);
        }
    }
    eprintln!("[waypoint] {msg}");
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => format!("t+{}s", d.as_secs()),
        Err(_) => "t?".to_string(),
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let loc = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let payload = info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "<non-string panic payload>".to_string()
        };
        write_log_line(&format!("PANIC at {loc}: {msg}"));
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_hook();
    write_log_line("startup: waypoint launching");
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_notes,
            commands::notes::create_note,
            commands::notes::read_note,
            commands::notes::save_content,
            commands::notes::save_note_settings,
            commands::notes::delete_note,
            commands::notes::rename_note,
            commands::notes::duplicate_note,
            commands::notes::move_note,
            commands::notes::get_note_order,
            commands::notes::set_note_order,
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
            hotkey::cmd_close_window,
            hotkey::cmd_hide_window,
            hotkey::cmd_minimize_window,
            hotkey::cmd_toggle_maximize,
            hotkey::cmd_get_window_position,
            hotkey::cmd_set_window_position,
            hotkey::cmd_start_dragging,
            hotkey::cmd_exit_app,
            hotkey::cmd_exit_app_with_flush,
            hotkey::cmd_restart_app,
            tray::cmd_open_help,
            tray::cmd_open_settings,
            commands::passthrough_cmd::cmd_set_passthrough,
            commands::passthrough_cmd::cmd_toggle_passthrough_global,
            commands::config_cmd::cmd_set_passthrough_hotkey,
            commands::config_cmd::cmd_set_show_in_taskbar,
            commands::config_cmd::get_transparent_includes_text,
            commands::config_cmd::set_transparent_includes_text,
        ])
        .setup(|app| {
            // 兩個初始化都用容錯方式：即使 tray 失敗（如 Steam Deck 無 StatusNotifier），
            // 至少 hotkey 仍可能工作；反之亦然。失敗原因寫入 log 檔供回報。
            if let Err(e) = tray::setup_tray(app) {
                write_log_line(&format!("setup_tray failed: {e}"));
            } else {
                write_log_line("setup_tray ok");
            }
            let config = storage::app_config::load().unwrap_or_default();
            match hotkey::register_hotkey(app.handle(), &config.hotkey) {
                Ok(()) => write_log_line(&format!("register_hotkey ok: {}", &config.hotkey)),
                Err(e) => write_log_line(&format!("register_hotkey failed ({}): {e}", &config.hotkey)),
            }
            match hotkey::register_passthrough_hotkey(app.handle(), &config.passthrough_hotkey) {
                Ok(()) => write_log_line(&format!("register_passthrough_hotkey ok: {}", &config.passthrough_hotkey)),
                Err(e) => {
                    write_log_line(&format!("register_passthrough_hotkey failed ({}): {e}", &config.passthrough_hotkey));
                    let state = app.handle().state::<crate::state::AppState>();
                    state.passthrough_hotkey_registered.store(false, std::sync::atomic::Ordering::SeqCst);
                    use tauri_plugin_notification::NotificationExt;
                    let _ = app.handle().notification()
                        .builder()
                        .title("Waypoint — 穿透快捷鍵註冊失敗")
                        .body(format!("「{}」可能已被其他程式占用。請至設定更換。", &config.passthrough_hotkey))
                        .show();
                }
            }
            {
                let handle = app.handle().clone();
                app.listen("waypoint://show-in-taskbar-changed", move |_| {
                    taskbar::refresh_taskbar_visibility(&handle);
                });
            }
            // Restart 後還原：若有 app_session.json（cmd_restart_app 留下的快照），
            // 依序開回之前的筆記視窗，並視需要叫出列表。讀完即刪檔。
            if let Some(snapshot) = storage::app_session::take() {
                write_log_line(&format!("restoring {} notes from app_session", snapshot.open_notes.len()));
                for n in &snapshot.open_notes {
                    let _ = hotkey::open_note_window(app.handle(), &n.note_id, n.context_id.as_deref());
                }
                if snapshot.list_open {
                    let _ = hotkey::open_list_window(app.handle());
                }
            }
            if std::env::var("WAYPOINT_E2E").is_ok() {
                write_log_line("WAYPOINT_E2E set: auto-opening list window");
                if let Err(e) = hotkey::open_list_window(app.handle()) {
                    write_log_line(&format!("e2e open_list_window failed: {e}"));
                }
            }
            Ok(())
        });

    // single-instance 只在 Windows 啟用
    // Linux/macOS 的 Flatpak/sandbox 環境下此 plugin 可能初始化失敗，
    // 造成 app 完全無聲 crash（tray icon 也不顯示）
    #[cfg(target_os = "windows")]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        let _ = hotkey::open_list_window(app);
    }));

    builder
        .run(tauri::generate_context!())
        .expect("error while running Waypoint");
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsStr;

    #[test]
    fn log_path_prefers_xdg_state_home() {
        let p = resolve_log_path(Some(OsStr::new("/tmp/xdg")), Some(OsStr::new("/tmp/home"))).unwrap();
        assert_eq!(p, std::path::PathBuf::from("/tmp/xdg/waypoint/error.log"));
    }

    #[test]
    fn log_path_falls_back_to_home_local_state() {
        let p = resolve_log_path(None, Some(OsStr::new("/tmp/home"))).unwrap();
        assert_eq!(p, std::path::PathBuf::from("/tmp/home/.local/state/waypoint/error.log"));
    }

    #[test]
    fn log_path_returns_none_without_env() {
        assert!(resolve_log_path(None, None).is_none());
    }
}
