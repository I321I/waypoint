/// Information about the currently focused window, captured
/// BEFORE the hotkey triggers a Waypoint window to open.
#[derive(Debug, Clone)]
pub struct FocusedWindowInfo {
    pub process_name: String,
    pub window_title: String,
}

#[cfg(target_os = "windows")]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND(std::ptr::null_mut()) { return None; }

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

        Some(FocusedWindowInfo { process_name, window_title })
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

    Some(FocusedWindowInfo {
        process_name: process_name.clone(),
        window_title: process_name,
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

    let atom_pid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;
    let pid_reply = conn.get_property(false, window, atom_pid, AtomEnum::CARDINAL, 0, 1).ok()?.reply().ok()?;
    let pid = u32::from_ne_bytes(pid_reply.value.try_into().ok()?);

    let atom_name = conn.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
    let atom_utf8 = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;
    let title_reply = conn.get_property(false, window, atom_name, atom_utf8, 0, 256).ok()?.reply().ok()?;
    let window_title = String::from_utf8_lossy(&title_reply.value).to_string();

    let process_name = std::fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    Some(FocusedWindowInfo { process_name, window_title })
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    None
}
