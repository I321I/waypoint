use tauri::{AppHandle, Manager, Emitter};

pub fn target_state(states: &[bool]) -> bool {
    // empty → target on; all-on → target off; otherwise → target on
    states.is_empty() || !states.iter().all(|&s| s)
}

/// 從 webview URL 解析 contextId 參數
/// 範例：tauri://localhost/#view=note&noteId=xxx&contextId=steam → Some("steam")
///        tauri://localhost/#view=note&noteId=xxx&contextId=null  → None
pub fn parse_context_id_from_url(url: &str) -> Option<String> {
    let q = url.split("contextId=").nth(1)?;
    let v = q.split('&').next()?;
    if v.is_empty() || v == "null" {
        None
    } else {
        Some(v.to_string())
    }
}

fn collect_note_labels(app: &AppHandle) -> Vec<String> {
    app.webview_windows()
        .into_iter()
        .filter(|(label, _)| label.starts_with("note-"))
        .map(|(label, _)| label)
        .collect()
}

#[tauri::command]
pub fn cmd_set_passthrough(app: AppHandle, note_label: String, on: bool) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&note_label) {
        win.set_ignore_cursor_events(on).map_err(|e| e.to_string())?;
        // 持久化：解析 note_id 與 contextId，寫回 settings.json
        if let Some(note_id) = note_label.strip_prefix("note-") {
            let url_str = win.url().map(|u| u.to_string()).unwrap_or_default();
            let ctx = parse_context_id_from_url(&url_str);
            if let Ok(mut note) = crate::storage::notes::read_note(ctx.as_deref(), note_id) {
                note.settings.passthrough = on;
                let _ = crate::storage::notes::save_settings(ctx.as_deref(), note_id, &note.settings);
            }
        }
    }
    {
        let state = app.state::<crate::state::AppState>();
        state.passthrough_state.lock().unwrap().insert(note_label.clone(), on);
    }
    let _ = app.emit("waypoint://passthrough-changed", (note_label, on));
    Ok(())
}

#[tauri::command]
pub fn cmd_toggle_passthrough_global(app: AppHandle) -> Result<(), String> {
    let labels = collect_note_labels(&app);
    let states: Vec<bool> = {
        let state = app.state::<crate::state::AppState>();
        let map = state.passthrough_state.lock().unwrap();
        labels.iter().map(|l| *map.get(l).unwrap_or(&false)).collect()
    };
    let target = target_state(&states);
    let state = app.state::<crate::state::AppState>();
    if target {
        // going on：snapshot 當下 list_open，Rust 端直接 hide list（不走
        // cmd_hide_window，避免 active_mode 被設 false 導致 foreground_watcher
        // 不再 emit context-changed，穿透中切視窗無法切 context）。
        let list_open = *state.list_window_open.lock().unwrap();
        state.pre_passthrough_list_open.store(list_open, std::sync::atomic::Ordering::SeqCst);
        if let Some(win) = app.get_webview_window("list") {
            let _ = win.hide();
        }
        *state.list_window_open.lock().unwrap() = false;
        crate::taskbar::refresh_taskbar_visibility(&app);
        let _ = app.emit("waypoint://passthrough-globally-on", ());
    } else {
        // going off：依 snapshot 還原 list（直接 Rust 端開回，不依賴前端）
        let should_show = state.pre_passthrough_list_open.load(std::sync::atomic::Ordering::SeqCst);
        if should_show {
            let _ = crate::hotkey::open_list_window(&app);
        }
        let _ = app.emit("waypoint://passthrough-globally-off", should_show);
    }
    for l in labels {
        cmd_set_passthrough(app.clone(), l, target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{target_state, parse_context_id_from_url};
    #[test] fn mixed_targets_on() { assert_eq!(target_state(&[true, false, true]), true); }
    #[test] fn all_on_targets_off() { assert_eq!(target_state(&[true, true, true]), false); }
    #[test] fn all_off_targets_on() { assert_eq!(target_state(&[false, false]), true); }
    #[test] fn empty_targets_on() { assert_eq!(target_state(&[]), true); }

    #[test]
    fn parse_context_null_returns_none() {
        let url = "tauri://localhost/#view=note&noteId=abc&contextId=null";
        assert_eq!(parse_context_id_from_url(url), None);
    }
    #[test]
    fn parse_context_value_returns_some() {
        let url = "tauri://localhost/#view=note&noteId=abc&contextId=steam&extra=x";
        assert_eq!(parse_context_id_from_url(url), Some("steam".to_string()));
    }
    #[test]
    fn parse_context_missing_returns_none() {
        let url = "tauri://localhost/#view=note&noteId=abc";
        assert_eq!(parse_context_id_from_url(url), None);
    }
}
