use std::path::PathBuf;

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

/// 設定開機自動啟動（enabled=true 建立 desktop file，false 刪除）
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let path = autostart_path().ok_or("無法取得 config 目錄")?;
        if enabled {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("無法建立 autostart 目錄: {e}"))?;
            }
            // 偵測是否在 Flatpak 環境
            let exec_cmd = if std::env::var("FLATPAK_ID").is_ok() {
                "flatpak run io.github.i321i.waypoint".to_string()
            } else {
                // 嘗試取得可執行檔完整路徑
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "waypoint".to_string())
            };
            let content = format!(
                "[Desktop Entry]\nType=Application\nName=Waypoint\nComment=浮動筆記應用程式\nExec={exec_cmd}\nIcon=io.github.i321i.waypoint\nX-GNOME-Autostart-enabled=true\nHidden=false\nNoDisplay=false\n"
            );
            std::fs::write(&path, content)
                .map_err(|e| format!("無法寫入 autostart 設定: {e}"))?;
        } else {
            if path.exists() {
                std::fs::remove_file(&path)
                    .map_err(|e| format!("無法移除 autostart 設定: {e}"))?;
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = enabled;
        Ok(())
    }
}
