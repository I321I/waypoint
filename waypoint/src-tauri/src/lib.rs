mod commands;
mod context;
mod error;
mod foreground_watcher;
mod hotkey;
mod state;
mod storage;
pub mod taskbar;
mod tray;

use state::AppState;
use tauri::Listener;
use tauri::Manager;

/// 寫 panic / 重大錯誤訊息到 log 檔，Steam Deck / Windows 等無 console 環境下方便使用者回報。
///
/// 跨平台用 `dirs::data_local_dir()`（已涵蓋 Windows `%LOCALAPPDATA%`、macOS
/// `~/Library/Application Support`、Linux `$XDG_DATA_HOME`/`~/.local/share`、
/// Flatpak sandbox 內的對應 home）。Windows 用 HOME 找不到的 v0.2.x 老 bug 因此修掉。
fn resolve_log_path(
    data_local: Option<&std::path::Path>,
    xdg_state: Option<&std::ffi::OsStr>,
    home: Option<&std::ffi::OsStr>,
    date: &str,
) -> Option<std::path::PathBuf> {
    // 優先序：dirs::data_local_dir → XDG_STATE_HOME（Linux 慣例）→ HOME/.local/state → HOME
    let base = data_local
        .map(std::path::PathBuf::from)
        .or_else(|| xdg_state.map(std::path::PathBuf::from))
        .or_else(|| home.map(|h| std::path::PathBuf::from(h).join(".local/state")))
        .or_else(|| home.map(std::path::PathBuf::from))?;
    Some(base.join("waypoint").join(format!("error-{}.log", date)))
}

fn today_date_string() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn waypoint_log_path() -> Option<std::path::PathBuf> {
    let data_local = dirs::data_local_dir();
    let xdg = std::env::var_os("XDG_STATE_HOME");
    let home = std::env::var_os("HOME");
    resolve_log_path(data_local.as_deref(), xdg.as_deref(), home.as_deref(), &today_date_string())
}

pub(crate) fn write_log_line(msg: &str) {
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
        .on_page_load(|window, payload| {
            // 記錄 webview 載入事件，輔助定位「白屏」issue：可比對
            // 「open_list/note_window build OK」與「page load Started/Finished」之間是否中斷。
            write_log_line(&format!(
                "webview page-load: label={} event={:?} url={}",
                window.label(),
                payload.event(),
                payload.url()
            ));
        })
        .on_window_event(|window, event| {
            // 筆記視窗的 close-requested（含 Alt+F4）-> emit note-closed 給 list 做 session 記帳
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let label = window.label().to_string();
                if let Some(note_id) = label.strip_prefix("note-") {
                    let app = window.app_handle();
                    // Alt+F4 / WM close path：Rust 端直接保存幾何，避免依賴 JS event 時序
                    crate::hotkey::save_geometry_for_label(app, &label);
                    let context_id = {
                        let state = app.state::<crate::state::AppState>();
                        state
                            .open_notes_context
                            .lock()
                            .ok()
                            .and_then(|mut m| m.remove(note_id))
                            .flatten()
                    };
                    let is_global = context_id.is_none();
                    let _ = tauri::Emitter::emit(app, "note-closed", serde_json::json!({
                        "noteId": note_id,
                        "contextId": context_id,
                        "isGlobal": is_global,
                    }));
                }
            }
        })
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
            commands::passthrough_cmd::cmd_mark_note_edited,
            commands::config_cmd::cmd_set_passthrough_hotkey,
            commands::config_cmd::cmd_set_show_in_taskbar,
            commands::config_cmd::get_transparent_includes_text,
            commands::config_cmd::set_transparent_includes_text,
            commands::diag::cmd_log_diag,
        ])
        .setup(|app| {
            // 兩個初始化都用容錯方式：即使 tray 失敗（如 Steam Deck 無 StatusNotifier
            // 或 Flatpak runtime 缺 libayatana-appindicator3.so），至少 hotkey 仍可能
            // 工作；反之亦然。失敗原因寫入 log 檔供回報。
            //
            // catch_unwind 必要：libappindicator-sys 在 dlopen 失敗時是直接 panic!()
            // 而非回 Err，沒有 catch_unwind 整個 process 會死，連 fallback 開列表都來不及。
            let tray_ok = match std::panic::catch_unwind(
                std::panic::AssertUnwindSafe(|| tray::setup_tray(app))
            ) {
                Ok(Ok(())) => { write_log_line("setup_tray ok"); true }
                Ok(Err(e)) => { write_log_line(&format!("setup_tray failed: {e}")); false }
                Err(_) => {
                    // panic 已被 panic_hook 寫入 log（含 backtrace），這裡只記事件
                    write_log_line("setup_tray panicked (likely missing libappindicator); falling back");
                    false
                }
            };
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
            // 啟動前景視窗監聽，OS 前景變化時自動切換 active context（item 8）
            foreground_watcher::start(app.handle().clone());

            // Tray 失敗（例如 Steam Deck 無 StatusNotifier）時，自動開列表視窗作為入口，
            // 否則 process 啟動了卻完全沒有任何視覺入口，使用者只能 kill -9。
            // 注意：app_session 還原時若已開過列表，open_list_window 會走 reuse 分支，不會重複建窗。
            if !tray_ok {
                write_log_line("tray fallback: opening list window (no tray available)");
                if let Err(e) = hotkey::open_list_window(app.handle()) {
                    write_log_line(&format!("tray fallback open_list_window failed: {e}"));
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

    // 攔截 ExitRequested：tauri 預設「最後一個視窗關閉就退出」，但 Waypoint
    // 是常駐背景應用程式（靠 tray / hotkey 重新叫出 list）。沒攔截會在使用者
    // 把 list ✕ 掉後整個 process 退出，下次按 hotkey 沒反應。
    // 真正的退出走 cmd_exit_app / cmd_exit_app_with_flush（呼叫 app.exit(0)）。
    builder
        .build(tauri::generate_context!())
        .expect("error while building Waypoint")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    // code=None 表示「最後 window 關閉觸發」；阻止退出
                    api.prevent_exit();
                    write_log_line("ExitRequested (no code): preventing exit, app stays alive in background");
                }
                // code=Some(_) 表示 app.exit(n) 顯式呼叫；不攔截
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsStr;

    #[test]
    fn log_path_prefers_xdg_state_home() {
        let p = resolve_log_path(None, Some(OsStr::new("/tmp/xdg")), Some(OsStr::new("/tmp/home")), "2026-05-20").unwrap();
        assert_eq!(p, std::path::PathBuf::from("/tmp/xdg/waypoint/error-2026-05-20.log"));
    }

    #[test]
    fn log_path_falls_back_to_home_local_state() {
        let p = resolve_log_path(None, None, Some(OsStr::new("/tmp/home")), "2026-05-20").unwrap();
        assert_eq!(p, std::path::PathBuf::from("/tmp/home/.local/state/waypoint/error-2026-05-20.log"));
    }

    #[test]
    fn log_path_returns_none_without_env() {
        assert!(resolve_log_path(None, None, None, "2026-05-20").is_none());
    }

    #[test]
    fn log_path_uses_data_local_first() {
        let dl = std::path::PathBuf::from("/AppData/Local");
        let p = resolve_log_path(Some(&dl), Some(OsStr::new("/tmp/xdg")), Some(OsStr::new("/tmp/home")), "2026-05-20").unwrap();
        assert_eq!(p, std::path::PathBuf::from("/AppData/Local/waypoint/error-2026-05-20.log"));
    }

    #[test]
    fn log_path_works_with_only_data_local() {
        let dl = std::path::PathBuf::from("/AppData/Local");
        let p = resolve_log_path(Some(&dl), None, None, "2026-05-20").unwrap();
        assert_eq!(p, std::path::PathBuf::from("/AppData/Local/waypoint/error-2026-05-20.log"));
    }

    #[test]
    fn today_date_string_format_is_ymd() {
        let d = today_date_string();
        // 形如 2026-05-20
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }
}
