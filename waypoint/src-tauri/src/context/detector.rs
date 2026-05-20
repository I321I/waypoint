/// Information about the currently focused window, captured
/// BEFORE the hotkey triggers a Waypoint window to open.
#[derive(Debug, Clone)]
pub struct FocusedWindowInfo {
    pub process_name: String,
    pub window_title: String,
    /// 對方視窗的 process id（取得失敗 = None）。foreground_watcher
    /// 用此比對 std::process::id() 以「精確」識別自家視窗，避免 AppImage /
    /// Flatpak 環境下 process_name 不是 "waypoint" 而導致誤判。
    pub pid: Option<u32>,
}

/// 應排除於 context 切換的 Windows 系統 shell 視窗 class。
/// - Shell_TrayWnd / Shell_SecondaryTrayWnd：工作列（主/副螢幕）
/// - Progman / WorkerW：桌面背景視窗
/// - Windows.UI.Core.CoreWindow：開始選單、搜尋、通知中心等 UWP shell
/// 這些 hwnd 的 process 都是 explorer.exe / ShellExperienceHost.exe，
/// 不過濾的話使用者點工作列就會把 active context 切到 explorer，造成筆記亂跳。
#[cfg(target_os = "windows")]
fn is_shell_class(class: &str) -> bool {
    matches!(
        class,
        "Shell_TrayWnd"
            | "Shell_SecondaryTrayWnd"
            | "Progman"
            | "WorkerW"
            | "Windows.UI.Core.CoreWindow"
    )
}

#[cfg(target_os = "windows")]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND(std::ptr::null_mut()) { return None; }

        // 先讀 class name 過濾 shell 視窗（工作列 / 桌面 / 開始選單）。
        // 比 process_name 過濾更精確：explorer.exe 同時是檔案總管與工作列宿主，
        // 只能靠 class name 區分。
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        if class_len > 0 {
            let class = OsString::from_wide(&class_buf[..class_len as usize])
                .to_string_lossy()
                .to_string();
            if is_shell_class(&class) {
                return None;
            }
        }

        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = OsString::from_wide(&title_buf[..len as usize])
            .to_string_lossy()
            .to_string();

        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

        let mut name_buf = [0u16; 512];
        let mut size = name_buf.len() as u32;
        windows::Win32::System::Threading::QueryFullProcessImageNameW(
            handle,
            windows::Win32::System::Threading::PROCESS_NAME_WIN32,
            windows::core::PWSTR(name_buf.as_mut_ptr()),
            &mut size,
        ).ok()?;

        let full_path = OsString::from_wide(&name_buf[..size as usize])
            .to_string_lossy()
            .to_string();
        let process_name = std::path::Path::new(&full_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or(full_path);

        Some(FocusedWindowInfo { process_name, window_title, pid: Some(pid) })
    }
}

#[cfg(target_os = "macos")]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    use objc2_app_kit::NSWorkspace;

    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let app = unsafe { workspace.frontmostApplication() }?;

    let process_name = unsafe { app.localizedName() }
        .map(|s| s.to_string())
        .unwrap_or_default();

    // pid 不取：objc2-app-kit 0.2 的 NSRunningApplication.processIdentifier 在沒有 libc
    // feature 時不暴露（v0.2.1 起連續失敗 5 個 mac release 都栽在這）。
    // foreground_watcher::is_self 會用 process_name 比對自家視窗 fallback，效果一樣。
    Some(FocusedWindowInfo {
        process_name: process_name.clone(),
        window_title: process_name,
        pid: None,
    })
}

#[cfg(target_os = "linux")]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;
    use x11rb::rust_connection::RustConnection;

    let (conn, screen_num) = RustConnection::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let atom_net_active = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW").ok()?.reply().ok()?.atom;
    let reply = conn.get_property(false, root, atom_net_active, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
    let window_id = u32::from_ne_bytes(reply.value.try_into().ok()?);
    let window = Window::from(window_id);

    // PID 取得（_NET_WM_PID 不一定存在）。在 Flatpak sandbox 內這 PID 是 host PID
    // sandbox 看不到對應的 /proc/<pid>/comm，因此 process_name 改用 WM_CLASS 為主。
    let atom_pid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;
    let pid: Option<u32> = conn.get_property(false, window, atom_pid, AtomEnum::CARDINAL, 0, 1)
        .ok()
        .and_then(|c| c.reply().ok())
        .and_then(|r| <[u8; 4]>::try_from(r.value.as_slice()).ok())
        .map(u32::from_ne_bytes);

    // 視窗標題（_NET_WM_NAME UTF-8）
    let atom_name = conn.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
    let atom_utf8 = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;
    let window_title = conn.get_property(false, window, atom_name, atom_utf8, 0, 256)
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| String::from_utf8_lossy(&r.value).to_string())
        .unwrap_or_default();

    // process_name：優先 WM_CLASS（X11 atom，Flatpak sandbox 也能取得，不依賴 /proc）
    // WM_CLASS = "instance\0class\0"，class 通常是 app 名（"Konsole" "firefox" 等）。
    // 取不到 WM_CLASS 才 fallback /proc/<pid>/comm（非 sandbox 環境仍可用）。
    let process_name = read_wm_class(&conn, window)
        .or_else(|| pid.and_then(|p| {
            std::fs::read_to_string(format!("/proc/{}/comm", p))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        }))
        .unwrap_or_default();

    Some(FocusedWindowInfo { process_name, window_title, pid })
}

#[cfg(target_os = "linux")]
fn read_wm_class(
    conn: &x11rb::rust_connection::RustConnection,
    window: x11rb::protocol::xproto::Window,
) -> Option<String> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;
    let reply = conn
        .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
        .ok()?
        .reply()
        .ok()?;
    if reply.value.is_empty() { return None; }
    // "instance\0class\0\0" — split by \0，取 instance 與 class
    let parts: Vec<&[u8]> = reply.value.split(|&b| b == 0).filter(|s| !s.is_empty()).collect();
    // 優先 class（parts[1]），fallback instance（parts[0]）
    parts
        .get(1)
        .or_else(|| parts.first())
        .and_then(|s| std::str::from_utf8(s).ok())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    None
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::is_shell_class;

    #[test]
    fn taskbar_classes_excluded() {
        assert!(is_shell_class("Shell_TrayWnd"));
        assert!(is_shell_class("Shell_SecondaryTrayWnd"));
    }

    #[test]
    fn desktop_classes_excluded() {
        assert!(is_shell_class("Progman"));
        assert!(is_shell_class("WorkerW"));
    }

    #[test]
    fn start_menu_excluded() {
        assert!(is_shell_class("Windows.UI.Core.CoreWindow"));
    }

    #[test]
    fn explorer_window_not_excluded() {
        // 檔案總管的 hwnd class 是 CabinetWClass，不是 shell class，必須能正常觸發 context 切換
        assert!(!is_shell_class("CabinetWClass"));
        assert!(!is_shell_class("Chrome_WidgetWin_1"));
    }
}
