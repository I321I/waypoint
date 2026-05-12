use crate::context::derive_context_id;
use crate::context::detector::get_focused_window;
use crate::state::AppState;
use crate::storage::app_config;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder, WebviewUrl};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// 序列化主 hotkey callback：拿不到鎖 = inflight，把 pending counter +1。
/// 當前 cycle 結束時逐次消耗 counter 再跑同樣次數的 cycle，實現
/// 「連按 N 次 = 跑 N 次 toggle」（cap=8 避免 key-repeat runaway）。
fn hotkey_inflight() -> &'static Mutex<()> {
    static M: OnceLock<Mutex<()>> = OnceLock::new();
    M.get_or_init(|| Mutex::new(()))
}

static HOTKEY_PENDING: AtomicU8 = AtomicU8::new(0);
const HOTKEY_PENDING_CAP: u8 = 8;

/// Debounce：同一個物理按壓被 OS auto-repeat 連射成 20+ 次 callback，
/// 必須以 timestamp 過濾。實測 keyboard repeat rate ≈ 30Hz (33ms 間隔)，
/// 設 250ms 足以吃掉一次 hold，又不影響使用者連續快速兩次有意按壓。
/// 並把 frontend handleCollapseAll 的 async IPC + hide 完成時間也包進來。
static LAST_FIRE_MS: AtomicU64 = AtomicU64::new(0);
const HOTKEY_DEBOUNCE_MS: u64 = 250;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, PartialEq)]
pub enum HotkeyAction {
    OpenAll,
    OpenList,
    CollapseAll,
}

pub fn determine_action(list_open: bool, any_note_open: bool) -> HotkeyAction {
    // 純 toggle：任何 waypoint 視窗（list 或 note）可見就 collapse，全收起才 open。
    // 舊邏輯「list 已關但 note 開 → 只開 list」造成「note 開著按 hotkey 不收起反而多開列表」
    // （v0.2.7 user 回報），改為純 toggle 比較直覺。
    if list_open || any_note_open {
        HotkeyAction::CollapseAll
    } else {
        HotkeyAction::OpenAll
    }
}

pub fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        // Debounce key-repeat：250ms 內的 callback 直接丟棄，不進 inflight、不累 pending。
        // 不這樣做的話一個 hold/repeat 會被當成 20+ 次 toggle，最終狀態取決於奇偶數，
        // 使用者體感「按了沒反應」或「自己跳回去」（v0.2.7 user 回報）。
        let now = now_ms();
        let last = LAST_FIRE_MS.load(Ordering::SeqCst);
        if now.saturating_sub(last) < HOTKEY_DEBOUNCE_MS {
            crate::write_log_line(&format!(
                "hotkey debounced: now={} last={} dt={}ms",
                now, last, now - last
            ));
            return;
        }
        LAST_FIRE_MS.store(now, Ordering::SeqCst);
        let _guard = match hotkey_inflight().try_lock() {
            Ok(g) => g,
            Err(_) => {
                // inflight：累加 pending（cap 防 key-repeat runaway）
                let _ = HOTKEY_PENDING.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                    if v < HOTKEY_PENDING_CAP { Some(v + 1) } else { None }
                });
                crate::write_log_line(&format!(
                    "hotkey queued: inflight, pending={}",
                    HOTKEY_PENDING.load(Ordering::SeqCst)
                ));
                return;
            }
        };
        run_hotkey_cycle(app);
        // 逐次消耗 pending counter
        loop {
            let prev = HOTKEY_PENDING.load(Ordering::SeqCst);
            if prev == 0 { break; }
            if HOTKEY_PENDING
                .compare_exchange(prev, prev - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                crate::write_log_line(&format!("hotkey replay: pending was {}", prev));
                run_hotkey_cycle(app);
            }
        }
    })?;
    Ok(())
}

fn run_hotkey_cycle(app: &AppHandle) {
        let window_info = get_focused_window();
        let state = app.state::<AppState>();
        let list_open = *state.list_window_open.lock().unwrap();
        // 注意：collapse 只 .hide() 不 close，hidden window 仍會留在 webview_windows 裡。
        // 必須以 is_visible() 判定，否則第二次按 hotkey 會持續判定 any_note=true → 一直 collapse，無法復原。
        let any_note_open = app.webview_windows()
            .iter()
            .any(|(label, win)| label.starts_with("note-") && win.is_visible().unwrap_or(false));
        let action = determine_action(list_open, any_note_open);
        crate::write_log_line(&format!("hotkey fired: action={:?} list_open={} any_note={}", action, list_open, any_note_open));
        // 注意：active_context_id 由 foreground_watcher 持續維護，hotkey path 不再
        // 自行覆寫（v0.2.x 的舊邏輯會在「按 hotkey 時 waypoint 自家視窗剛好前景」
        // 場景下污染 state 變成 ctx="waypoint"，閃一下才被 watcher 修回）。
        let _ = window_info;
        match action {
            HotkeyAction::OpenAll => {
                let _ = open_list_window(app);
            }
            HotkeyAction::OpenList => {
                let _ = open_list_window(app);
            }
            HotkeyAction::CollapseAll => {
                // Emit event so frontend can save session before collapsing
                let _ = app.emit("waypoint://collapse-all-requested", ());
                // Also collapse after a brief delay to handle case
                // where list window is not mounted (safety fallback)
                let app2 = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    // Only collapse if list window still exists (frontend may have already handled it)
                    if let Some(list) = app2.get_webview_window("list") {
                        if list.is_visible().unwrap_or(false) {
                            collapse_all_waypoint_windows(&app2);
                        }
                    }
                });
            }
        }
}

pub fn register_passthrough_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        let _ = crate::commands::passthrough_cmd::cmd_toggle_passthrough_global(app.clone());
    })?;
    Ok(())
}

pub fn reregister_passthrough_hotkey(app: &AppHandle, old_hotkey: &str, new_hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    let _ = app.global_shortcut().unregister(old_hotkey);
    register_passthrough_hotkey(app, new_hotkey)
}

pub fn open_list_window(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    if let Some(win) = app.get_webview_window("list") {
        win.unminimize().ok();
        win.show()?;
        // raise 到最上：KDE/GNOME focus stealing prevention 會擋一般 set_focus，
        // 用 set_always_on_top toggle + set_focus 強制 raise（toggle off 確保下次能再 raise）。
        let _ = win.set_always_on_top(true);
        win.set_focus()?;
        *state.list_window_open.lock().unwrap() = true;
        // 通知前端重新載入 context / session（再叫出時也會套用新 context）
        let _ = app.emit("waypoint://list-shown", ());
        return Ok(());
    }
    let _win = WebviewWindowBuilder::new(app, "list", WebviewUrl::App("/#view=list".into()))
        .title("Waypoint")
        .inner_size(220.0, 500.0)
        .min_inner_size(180.0, 300.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build()?;
    *state.list_window_open.lock().unwrap() = true;
    crate::taskbar::refresh_taskbar_visibility(app);
    Ok(())
}

pub fn collapse_all_waypoint_windows(app: &AppHandle) {
    let windows = app.webview_windows();
    for (label, window) in &windows {
        if label.starts_with("note-") || label == "list" {
            let _ = window.hide();
        }
    }
    let state = app.state::<AppState>();
    *state.list_window_open.lock().unwrap() = false;
    crate::taskbar::refresh_taskbar_visibility(app);
}

pub fn open_note_window(app: &AppHandle, note_id: &str, context_id: Option<&str>) -> tauri::Result<()> {
    let label = format!("note-{}", note_id);
    // 記錄 noteId→contextId，給 on_window_event 在 close-requested 時用
    if let Ok(mut map) = app
        .state::<crate::state::AppState>()
        .open_notes_context
        .lock()
    {
        map.insert(note_id.to_string(), context_id.map(|s| s.to_string()));
    }
    if let Some(win) = app.get_webview_window(&label) {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    let ctx_param = context_id.map(|c| format!("&contextId={}", c)).unwrap_or_default();
    let url = format!("/#view=note&noteId={}{}", note_id, ctx_param);
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title("Waypoint Note")
        .inner_size(420.0, 600.0)
        .min_inner_size(300.0, 200.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true);

    // 套用 settings.window_bounds（item 7：每次 move/resize 即時記憶）
    if let Ok(note) = crate::storage::notes::read_note(context_id, note_id) {
        if let Some(b) = note.settings.window_bounds {
            builder = builder
                .position(b.x as f64, b.y as f64)
                .inner_size(b.width as f64, b.height as f64);
        }
    }

    builder.build()?;
    crate::taskbar::refresh_taskbar_visibility(app);
    Ok(())
}

// async：見 tray::cmd_open_help 的註解。WebView2 上 sync command + 同步建
// webview 會 deadlock 原 webview 的 IPC，造成兩邊白屏。
#[tauri::command]
pub async fn cmd_open_note_window(app: AppHandle, note_id: String, context_id: Option<String>) -> Result<(), String> {
    open_note_window(&app, &note_id, context_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_collapse_all(app: AppHandle) {
    collapse_all_waypoint_windows(&app);
}

#[tauri::command]
pub fn cmd_close_note_window(app: AppHandle, note_id: String) -> Result<(), String> {
    let label = format!("note-{}", note_id);
    if let Some(win) = app.get_webview_window(&label) {
        // 關窗前 Rust 端直接讀 OS 位置/大小寫進 settings.window_bounds。
        // 不依賴 JS event 時序（onMoved 對 programmatic set_position 不一定 fire；
        // emit_to + flushPendingSave 也有 async timing 問題）。
        save_geometry_to_settings(&win, &note_id);
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn save_geometry_to_settings(win: &tauri::WebviewWindow, note_id: &str) {
    let pos = match win.outer_position() { Ok(p) => p, Err(_) => return };
    let size = match win.outer_size() { Ok(s) => s, Err(_) => return };
    let url = win.url().map(|u| u.to_string()).unwrap_or_default();
    let ctx = crate::commands::passthrough_cmd::parse_context_id_from_url(&url);
    if let Ok(mut note) = crate::storage::notes::read_note(ctx.as_deref(), note_id) {
        note.settings.window_bounds = Some(crate::storage::notes::WindowBounds {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
        });
        let _ = crate::storage::notes::save_settings(ctx.as_deref(), note_id, &note.settings);
    }
}

/// 公開給 lib.rs 的 on_window_event 在 CloseRequested 時呼叫，覆蓋 Alt+F4 / WM close 路徑。
pub fn save_geometry_for_label(app: &AppHandle, label: &str) {
    let Some(note_id) = label.strip_prefix("note-") else { return };
    let Some(win) = app.get_webview_window(label) else { return };
    save_geometry_to_settings(&win, note_id);
}

/// 用 label 關閉視窗（close）——前端不依賴 getCurrentWindow()
#[tauri::command]
pub fn cmd_close_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 用 label 隱藏視窗（hide）——前端不依賴 getCurrentWindow()
#[tauri::command]
pub fn cmd_hide_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.hide().map_err(|e| e.to_string())?;
        if label == "list" {
            let state = app.state::<crate::state::AppState>();
            *state.list_window_open.lock().unwrap() = false;
        }
    }
    Ok(())
}

/// 最小化視窗
#[tauri::command]
pub fn cmd_minimize_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        win.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 取得視窗外部位置（physical pixels, 螢幕座標）。
/// 用途：E2E 測試驗證拖曳前後位置是否改變。
#[tauri::command]
pub fn cmd_get_window_position(app: AppHandle, label: String) -> Result<(i32, i32), String> {
    let win = app.get_webview_window(&label).ok_or_else(|| format!("window not found: {label}"))?;
    let pos = win.outer_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

/// 設定視窗外部位置（physical pixels）。
/// 用途：E2E 測試為視窗指定已知起點。
#[tauri::command]
pub fn cmd_set_window_position(app: AppHandle, label: String, x: i32, y: i32) -> Result<(), String> {
    let win = app.get_webview_window(&label).ok_or_else(|| format!("window not found: {label}"))?;
    win.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|e| e.to_string())?;
    Ok(())
}

/// 啟動拖曳（讓 webview 以外的地方也能觸發原生 drag）。
/// 用途：前端 JS 在 titlebar mousedown 上呼叫，作為 data-tauri-drag-region 的 fallback；
/// 以及 E2E 測試直接觸發 drag + 配合 set_window_position 驗證。
#[tauri::command]
pub fn cmd_start_dragging(app: AppHandle, label: String) -> Result<(), String> {
    let win = app.get_webview_window(&label).ok_or_else(|| format!("window not found: {label}"))?;
    win.start_dragging().map_err(|e| e.to_string())?;
    Ok(())
}

/// 切換最大化狀態
#[tauri::command]
pub fn cmd_toggle_maximize(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&label) {
        if win.is_maximized().unwrap_or(false) {
            win.unmaximize().map_err(|e| e.to_string())?;
        } else {
            win.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// 完全結束 Waypoint
#[tauri::command]
pub fn cmd_exit_app(app: AppHandle) {
    app.exit(0);
}

/// 完全結束 Waypoint 前先廣播 flush-and-save-now 給所有 note 視窗，
/// 等指定毫秒讓前端寫入再退出。300ms 足夠 debounce(100ms) + atomic write 完成。
#[tauri::command]
pub fn cmd_exit_app_with_flush(app: AppHandle) {
    let _ = app.emit("waypoint://flush-and-save-now", ());
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        app.exit(0);
    });
}

/// 快照當下所有 waypoint 視窗狀態（note-* 與 list）。
pub fn snapshot_open_windows(app: &AppHandle) -> crate::storage::app_session::AppSession {
    use crate::storage::app_session::{AppSession, OpenNoteRef};
    let mut open_notes: Vec<OpenNoteRef> = Vec::new();
    let mut list_open = false;
    for (label, win) in app.webview_windows() {
        if label == "list" {
            if win.is_visible().unwrap_or(false) { list_open = true; }
        } else if let Some(note_id) = label.strip_prefix("note-") {
            if win.is_visible().unwrap_or(false) {
                // 從 URL hash 解析 contextId（開窗時帶入）
                let ctx = win
                    .url()
                    .ok()
                    .and_then(|u| {
                        let s = u.to_string();
                        let hash = s.split_once('#').map(|(_, h)| h.to_string())?;
                        let q = hash.trim_start_matches('/');
                        // view=note&noteId=xxx&contextId=yyy
                        for kv in q.split('&') {
                            if let Some(v) = kv.strip_prefix("contextId=") {
                                return Some(v.to_string());
                            }
                        }
                        None
                    });
                open_notes.push(OpenNoteRef {
                    note_id: note_id.to_string(),
                    context_id: ctx,
                });
            }
        }
    }
    AppSession { open_notes, list_open }
}

/// 重新啟動 Waypoint：
/// 1. 把當下開啟的筆記視窗寫入 app_session.json
/// 2. 以目前 binary 執行新 process
/// 3. exit(0)
///
/// 啟動時 lib.rs 會讀 app_session.json 並還原這些視窗。
#[tauri::command]
pub fn cmd_restart_app(app: AppHandle) -> Result<(), String> {
    let snapshot = snapshot_open_windows(&app);
    crate::storage::app_session::save(&snapshot).map_err(|e| e.to_string())?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // 以獨立 process 啟動，避免成為 child（父死 child 受影響）。
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = std::process::Command::new(&exe).process_group(0).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        let _ = std::process::Command::new(&exe)
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_windows_open_triggers_open_all() {
        assert_eq!(determine_action(false, false), HotkeyAction::OpenAll);
    }

    #[test]
    fn notes_open_triggers_collapse_all() {
        // 純 toggle：note 開著按 hotkey 應該 collapse，不是開列表
        assert_eq!(determine_action(false, true), HotkeyAction::CollapseAll);
    }

    #[test]
    fn list_open_triggers_collapse_all() {
        assert_eq!(determine_action(true, false), HotkeyAction::CollapseAll);
        assert_eq!(determine_action(true, true), HotkeyAction::CollapseAll);
    }

    /// R8: list autohide-on-blur 已移除（避免拖曳時被吃掉焦點而瞬間消失）。
    /// 編譯成功本身就是主要保證；此測試用源碼指紋雙保險。
    #[test]
    fn note_window_builder_sets_always_on_top() {
        let src = include_str!("mod.rs");
        // 找到 open_note_window 區段，要求 .always_on_top(true)
        let start = src.find("pub fn open_note_window").expect("open_note_window must exist");
        let end = src[start..].find("\n}\n").expect("function must close") + start;
        let body = &src[start..end];
        assert!(
            body.contains(".always_on_top(true)"),
            "open_note_window must call .always_on_top(true) on its WebviewWindowBuilder (R3)"
        );
    }

    #[test]
    fn open_note_window_sets_transparent() {
        let src = include_str!("./mod.rs");
        let start = src.find("pub fn open_note_window").expect("open_note_window must exist");
        let end = src[start..].find("\n}\n").map(|i| start + i).unwrap_or(src.len());
        let body = &src[start..end];
        assert!(
            body.contains(concat!(".trans", "parent(true)")),
            "open_note_window must call .transparent(true) for R5"
        );
    }

    /// Regression：any_note_open 必須以 is_visible() 過濾，
    /// 否則 collapse 後 hidden 的 note window 仍算數，
    /// 第二次按 hotkey 永遠走 CollapseAll，無法復原（v0.2.7 user 回報）。
    #[test]
    fn any_note_open_must_check_visibility() {
        let src = include_str!("mod.rs");
        let start = src.find("fn run_hotkey_cycle").expect("run_hotkey_cycle must exist");
        let end = src[start..].find("\n}\n").map(|i| start + i).unwrap_or(src.len());
        let body = &src[start..end];
        assert!(
            body.contains("any_note_open") && body.contains("is_visible"),
            "run_hotkey_cycle 必須以 is_visible() 過濾 any_note_open，不可只看 webview_windows().keys()"
        );
    }

    #[test]
    fn no_list_autohide_function() {
        let src = include_str!("mod.rs");
        // production 程式中的函式定義字串（拆開以免測試本身誤判）。
        let needle = concat!("fn ", "attach", "_list_autohide");
        assert!(
            !src.contains(needle),
            "list autohide 函式應已移除（R8）"
        );
    }
}
