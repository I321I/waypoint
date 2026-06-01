use std::path::PathBuf;

/// 解析筆記資料目錄。
///
/// 優先序：
/// 1. `WAYPOINT_DATA_DIR` env var（測試 / 進階使用者強制覆寫）
/// 2. `$HOME` env var：Flatpak sandbox 內 `$HOME` 會被設成 sandbox app home
///    (`~/.var/app/<id>/`)，而 manifest 的 `--filesystem=~/waypoint` 將
///    host `~/waypoint` 透過 portal 暴露在 sandbox 看不到的位置。改用
///    `$HOME/waypoint` 寫入 sandbox 持久區可避免每次重啟資料消失。
///    用 env var 而不是 libc `getpwuid_r()` (dirs::home_dir 走那條) 是因
///    為 Flatpak passwd 表內 pw_dir 可能指向不可寫的位置。
/// 3. `dirs::home_dir()` fallback：傳統路徑，原行為。
pub fn data_dir() -> PathBuf {
    if let Some(override_dir) = std::env::var_os("WAYPOINT_DATA_DIR") {
        return PathBuf::from(override_dir);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join("waypoint");
    }
    dirs::home_dir()
        .expect("cannot find home dir")
        .join("waypoint")
}

pub fn global_dir() -> PathBuf {
    data_dir().join("global")
}

pub fn context_dir(context_id: &str) -> PathBuf {
    data_dir().join("contexts").join(context_id)
}

pub fn note_dir(context_id: Option<&str>, note_id: &str) -> PathBuf {
    match context_id {
        Some(ctx) => context_dir(ctx).join(note_id),
        None => global_dir().join(note_id),
    }
}

pub fn app_config_path() -> PathBuf {
    data_dir().join("app.json")
}

pub fn session_path(context_id: &str) -> PathBuf {
    context_dir(context_id).join("session.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_note_path_has_no_contexts_segment() {
        let p = note_dir(None, "abc123");
        assert!(p.to_str().unwrap().contains("global"));
        assert!(!p.to_str().unwrap().contains("contexts"));
    }

    #[test]
    fn context_note_path_contains_context_id() {
        let p = note_dir(Some("steam"), "abc123");
        let s = p.to_str().unwrap();
        assert!(s.contains("contexts"));
        assert!(s.contains("steam"));
        assert!(s.contains("abc123"));
    }

    #[test]
    fn session_path_is_inside_context_dir() {
        let p = session_path("steam");
        let s = p.to_str().unwrap();
        assert!(s.contains("contexts"));
        assert!(s.contains("steam"));
        assert!(s.ends_with("session.json"));
    }

    /// Regression：data_dir 必須能被 create_dir_all 真正建立（多層巢狀也行）。
    /// Flatpak Steam Deck 上 host ~/waypoint 不存在時，sandbox 寫入會落到
    /// overlay → 重啟資料消失；lib.rs 啟動時顯式 create_dir_all 之前提是
    /// data_dir 路徑本身可建。用獨佔 lock 避免與其他改 env 的測試打架。
    #[test]
    fn data_dir_is_creatable() {
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap();
        let prev_override = std::env::var_os("WAYPOINT_DATA_DIR");
        let tmp = std::env::temp_dir().join(format!("wp-create-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::env::set_var("WAYPOINT_DATA_DIR", &tmp);
        let d = data_dir();
        std::fs::create_dir_all(&d).expect("data_dir must be creatable");
        assert!(d.exists() && d.is_dir());
        let _ = std::fs::remove_dir_all(&tmp);
        match prev_override {
            Some(v) => std::env::set_var("WAYPOINT_DATA_DIR", v),
            None => std::env::remove_var("WAYPOINT_DATA_DIR"),
        }
    }

    /// WAYPOINT_DATA_DIR 應該蓋過所有其他來源（測試 / Flatpak 進階用戶用得到）。
    /// 用獨佔 lock 避免測試平行跑互相干擾 env state。
    #[test]
    fn data_dir_honors_override_env() {
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap();
        let prev_override = std::env::var_os("WAYPOINT_DATA_DIR");
        std::env::set_var("WAYPOINT_DATA_DIR", "/tmp/wp-test");
        assert_eq!(data_dir(), PathBuf::from("/tmp/wp-test"));
        match prev_override {
            Some(v) => std::env::set_var("WAYPOINT_DATA_DIR", v),
            None => std::env::remove_var("WAYPOINT_DATA_DIR"),
        }
    }
}
