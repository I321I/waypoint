//! 筆記顯示順序。
//!
//! 儲存位置：
//!   global：$HOME/waypoint/global/_order.json
//!   context：$HOME/waypoint/contexts/{ctx}/_order.json
//!
//! 格式：`["noteId1","noteId2",...]`
//! list_notes 會根據此順序排序；新 note 會附加在尾端。
use crate::error::WaypointError;
use crate::storage::paths::{context_dir, global_dir};
use std::path::PathBuf;

fn order_path(context_id: Option<&str>) -> PathBuf {
    match context_id {
        Some(ctx) => context_dir(ctx).join("_order.json"),
        None => global_dir().join("_order.json"),
    }
}

pub fn load(context_id: Option<&str>) -> Vec<String> {
    let p = order_path(context_id);
    if !p.exists() { return vec![]; }
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
}

pub fn save(context_id: Option<&str>, order: &[String]) -> Result<(), WaypointError> {
    let p = order_path(context_id);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&p, serde_json::to_string(order)?)?;
    Ok(())
}

/// 將 order 應用到 note 列表：依 order 排序；order 外的 note 附加在尾端（穩定）。
pub fn apply_order<T, F>(order: &[String], notes: Vec<T>, id_of: F) -> Vec<T>
where
    F: Fn(&T) -> &str,
{
    let mut by_id: std::collections::HashMap<String, T> =
        notes.into_iter().map(|n| (id_of(&n).to_string(), n)).collect();
    let mut out: Vec<T> = Vec::with_capacity(by_id.len() + order.len());
    for id in order {
        if let Some(n) = by_id.remove(id) {
            out.push(n);
        }
    }
    // 將剩下未在 order 的 note 加回（依 id 穩定排序，避免檔案系統順序不定）
    let mut rest: Vec<(String, T)> = by_id.into_iter().collect();
    rest.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, n) in rest {
        out.push(n);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_utils::HOME_LOCK;
    use tempfile::TempDir;

    fn setup() -> (TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = HOME_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        (dir, guard)
    }

    #[test]
    fn load_empty_when_no_file() {
        let (_d, _g) = setup();
        assert!(load(None).is_empty());
        assert!(load(Some("steam")).is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (_d, _g) = setup();
        save(Some("steam"), &["b".into(), "a".into(), "c".into()]).unwrap();
        assert_eq!(load(Some("steam")), vec!["b", "a", "c"]);
    }

    #[test]
    fn apply_order_sorts_and_appends_unknown() {
        let notes = vec!["a", "b", "c", "d"];
        let out = apply_order(&["c".into(), "a".into()], notes, |s| s);
        assert_eq!(out, vec!["c", "a", "b", "d"]);
    }

    #[test]
    fn apply_order_ignores_unknown_ids() {
        let notes = vec!["a", "b"];
        let out = apply_order(&["x".into(), "a".into(), "y".into()], notes, |s| s);
        assert_eq!(out, vec!["a", "b"]);
    }
}
