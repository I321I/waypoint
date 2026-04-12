use crate::error::WaypointError;
use crate::state::AppState;
use crate::storage::app_config::{self, ContextConfig, MatchBy};
use tauri::State;

#[tauri::command]
pub fn get_active_context(state: State<AppState>) -> Option<String> {
    state.active_context_id.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_context_match_by(context_id: String, match_by: String) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    let mb = if match_by == "title" { MatchBy::Title } else { MatchBy::Process };
    config.contexts.insert(context_id, ContextConfig { match_by: mb });
    app_config::save(&config)
}

#[tauri::command]
pub fn set_context_alias(from_context: String, to_context: String) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    config.context_aliases.insert(from_context, to_context);
    app_config::save(&config)
}

#[tauri::command]
pub fn rename_context(old_id: String, new_id: String) -> Result<(), WaypointError> {
    use crate::storage::paths::data_dir;
    let old_path = data_dir().join("contexts").join(&old_id);
    let new_path = data_dir().join("contexts").join(&new_id);
    if old_path.exists() {
        std::fs::rename(old_path, new_path)?;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_context(context_id: String) -> Result<(), WaypointError> {
    use crate::storage::paths::context_dir;
    let path = context_dir(&context_id);
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

#[tauri::command]
pub fn list_contexts() -> Result<Vec<String>, WaypointError> {
    use crate::storage::paths::data_dir;
    let ctx_dir = data_dir().join("contexts");
    if !ctx_dir.exists() {
        return Ok(vec![]);
    }
    let mut contexts = vec![];
    for entry in std::fs::read_dir(&ctx_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            if let Some(name) = entry.path().file_name() {
                contexts.push(name.to_string_lossy().to_string());
            }
        }
    }
    Ok(contexts)
}
