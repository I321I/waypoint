use std::path::{Path, PathBuf};

/// 回傳 autostart 功能是否支援（僅 Linux）
pub fn is_supported() -> bool {
    cfg!(target_os = "linux")
}

/// 取得 autostart desktop file 路徑
/// ~/.config/autostart/waypoint.desktop
fn autostart_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("autostart").join("waypoint.desktop"))
}

/// 是否已啟用開機自動啟動
pub fn is_enabled() -> bool {
    #[cfg(target_os = "linux")]
    {
        autostart_path().map(|p| p.exists()).unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    false
}

/// Linux 端：在指定 path 啟用/停用 autostart。供單元測試注入 tempdir。
#[cfg(target_os = "linux")]
pub fn set_enabled_at(path: &Path, enabled: bool) -> Result<(), String> {
    if enabled {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("無法建立 autostart 目錄: {e}"))?;
        }
        let exec_cmd = if std::env::var("FLATPAK_ID").is_ok() {
            "flatpak run io.github.i321i.waypoint".to_string()
        } else {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "waypoint".to_string())
        };
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=Waypoint\nComment=浮動筆記應用程式\nExec={exec_cmd}\nIcon=io.github.i321i.waypoint\nX-GNOME-Autostart-enabled=true\nHidden=false\nNoDisplay=false\n"
        );
        std::fs::write(path, content)
            .map_err(|e| format!("無法寫入 autostart 設定: {e}"))?;
    } else if path.exists() {
        std::fs::remove_file(path)
            .map_err(|e| format!("無法移除 autostart 設定: {e}"))?;
    }
    Ok(())
}

/// 設定開機自動啟動（enabled=true 建立 desktop file，false 刪除）
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let path = autostart_path().ok_or("無法取得 config 目錄")?;
        set_enabled_at(&path, enabled)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = enabled;
        Ok(())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    fn unique_path(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!(
            "waypoint-autostart-test-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        p.push("autostart");
        p.push("waypoint.desktop");
        p
    }

    #[test]
    fn enabling_creates_desktop_file_and_disabling_removes_it() {
        let path = unique_path("enable-disable");
        let _ = std::fs::remove_file(&path);

        set_enabled_at(&path, true).expect("enable should succeed");
        assert!(path.exists(), "啟用後 .desktop 檔案應存在於 {:?}", path);

        let content = std::fs::read_to_string(&path).expect("讀回內容");
        assert!(content.contains("[Desktop Entry]"));
        assert!(content.contains("Name=Waypoint"));
        assert!(content.contains("X-GNOME-Autostart-enabled=true"));

        set_enabled_at(&path, false).expect("disable should succeed");
        assert!(!path.exists(), "停用後 .desktop 檔案應已刪除");
    }

    #[test]
    fn disabling_when_already_disabled_is_noop() {
        let mut path = std::env::temp_dir();
        path.push("waypoint-autostart-noop-test");
        let _ = std::fs::remove_file(&path);
        set_enabled_at(&path, false).expect("disable on absent file 應成功");
        assert!(!path.exists());
    }

    #[test]
    fn re_enabling_overwrites_existing_file() {
        let path = unique_path("re-enable");
        set_enabled_at(&path, true).unwrap();
        std::fs::write(&path, "stale content").unwrap();
        set_enabled_at(&path, true).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[Desktop Entry]"), "重新啟用應覆寫舊內容");
        let _ = std::fs::remove_file(&path);
    }
}
