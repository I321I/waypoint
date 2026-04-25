use tauri::{AppHandle, Manager};

pub fn should_show_taskbar(visible_count: usize, setting_on: bool) -> bool {
    setting_on && visible_count > 0
}

pub fn refresh_taskbar_visibility(app: &AppHandle) {
    let cfg = crate::storage::app_config::load().unwrap_or_default();
    let setting = cfg.show_in_taskbar;
    let windows = app.webview_windows();
    let visible_count = windows.values().filter(|w| w.is_visible().unwrap_or(false)).count();
    let show = should_show_taskbar(visible_count, setting);
    for (_, win) in windows.iter() {
        let _ = win.set_skip_taskbar(!show);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn visible_and_setting_on() {
        assert!(should_show_taskbar(1, true));
        assert!(should_show_taskbar(3, true));
    }
    #[test]
    fn no_window() {
        assert!(!should_show_taskbar(0, true));
    }
    #[test]
    fn setting_off() {
        assert!(!should_show_taskbar(5, false));
    }
}
