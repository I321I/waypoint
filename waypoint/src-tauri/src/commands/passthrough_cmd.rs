use tauri::{AppHandle, Manager, Emitter};

pub fn target_state(states: &[bool]) -> bool {
    // empty → target on; all-on → target off; otherwise → target on
    states.is_empty() || !states.iter().all(|&s| s)
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
    }
    let state = app.state::<crate::state::AppState>();
    state.passthrough_state.lock().unwrap().insert(note_label.clone(), on);
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
    for l in labels {
        cmd_set_passthrough(app.clone(), l, target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::target_state;
    #[test] fn mixed_targets_on() { assert_eq!(target_state(&[true, false, true]), true); }
    #[test] fn all_on_targets_off() { assert_eq!(target_state(&[true, true, true]), false); }
    #[test] fn all_off_targets_on() { assert_eq!(target_state(&[false, false]), true); }
    #[test] fn empty_targets_on() { assert_eq!(target_state(&[]), true); }
}
