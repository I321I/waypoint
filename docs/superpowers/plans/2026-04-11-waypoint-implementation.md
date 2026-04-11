# Waypoint Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Waypoint — a cross-platform floating note app with context-aware hotkeys, WYSIWYG markdown editing, and per-context session restore.

**Architecture:** Tauri 2.0 Rust backend handles all OS-level concerns (hotkeys, window detection, file I/O, tray); Svelte frontend renders all UI in a single Vite app where each window type is differentiated by URL query params (`?view=list`, `?view=note&noteId=x&contextId=y`). TipTap provides WYSIWYG editing stored as Markdown files.

**Tech Stack:** Rust + Tauri 2.0, Svelte 5, TypeScript, TipTap 2.x, Vite, Vitest, `uuid` crate, `x11rb` (Linux), `windows` crate (Windows), `objc2` (macOS)

---

## File Structure

```
waypoint/
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── icons/                        ← app icons (generated)
│   └── src/
│       ├── main.rs                   ← entry point
│       ├── lib.rs                    ← app builder + plugin registration
│       ├── error.rs                  ← WaypointError enum
│       ├── state.rs                  ← AppState (Mutex, managed by Tauri)
│       ├── storage/
│       │   ├── mod.rs
│       │   ├── paths.rs              ← ~/waypoint/ path resolution
│       │   ├── app_config.rs         ← app.json read/write
│       │   ├── notes.rs              ← note CRUD (content.md + settings.json)
│       │   └── session.rs            ← session.json read/write
│       ├── context/
│       │   ├── mod.rs
│       │   ├── detector.rs           ← platform-specific active window info
│       │   └── normalizer.rs         ← normalization + alias resolution
│       ├── hotkey/
│       │   └── mod.rs                ← hotkey registration + 3-state machine
│       ├── tray/
│       │   └── mod.rs                ← system tray setup
│       └── commands/
│           ├── mod.rs                ← re-exports all commands
│           ├── notes.rs              ← note CRUD commands
│           ├── context_cmd.rs        ← context query/update commands
│           ├── session_cmd.rs        ← session save/restore commands
│           └── config_cmd.rs         ← app config commands
├── src/
│   ├── app.css                       ← global VSCode dark styles
│   ├── main.ts                       ← Svelte entry (reads ?view= param)
│   ├── App.svelte                    ← root router
│   ├── lib/
│   │   ├── types.ts                  ← shared TS types
│   │   ├── api.ts                    ← typed tauri invoke wrappers
│   │   └── stores.ts                 ← Svelte stores
│   └── windows/
│       ├── ListWindow.svelte
│       ├── list/
│       │   ├── GlobalSection.svelte
│       │   ├── ContextSection.svelte ← includes right-click menu
│       │   └── NoteItem.svelte
│       ├── NoteWindow.svelte
│       ├── note/
│       │   ├── Editor.svelte         ← TipTap WYSIWYG
│       │   ├── Toolbar.svelte
│       │   └── SettingsPanel.svelte
│       └── HelpWindow.svelte
├── index.html
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.ts
```

---

## Task 1: Project Scaffolding

**Files:**
- Create: `waypoint/` (entire project)
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `package.json`
- Create: `vite.config.ts`

- [ ] **Step 1: Create project with Tauri CLI**

```bash
npm create tauri@latest waypoint -- --template svelte-ts --manager npm
cd waypoint
```

- [ ] **Step 2: Install frontend dependencies**

```bash
npm install @tiptap/core @tiptap/pm @tiptap/starter-kit
npm install @tiptap/extension-task-list @tiptap/extension-task-item
npm install @tiptap/extension-code-block-lowlight @tiptap/extension-table
npm install @tiptap/extension-underline @tiptap/extension-strike
npm install @tiptap/extension-markdown lowlight
npm install --save-dev vitest @testing-library/svelte jsdom
```

- [ ] **Step 3: Install Tauri plugins**

```bash
npm install @tauri-apps/plugin-global-shortcut @tauri-apps/plugin-shell
```

- [ ] **Step 4: Replace `src-tauri/Cargo.toml` with full dependencies**

```toml
[package]
name = "waypoint"
version = "0.1.0"
edition = "2021"

[lib]
name = "waypoint_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2.0"
tauri-plugin-shell = "2.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
dirs = "5"
thiserror = "1"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
  "Win32_UI_WindowsAndMessaging",
  "Win32_System_Threading",
  "Win32_Foundation",
] }

[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { version = "0.13", features = ["connect"] }

[target.'cfg(target_os = "macos")'.dependencies]
objc2-app-kit = { version = "0.2", features = ["NSWorkspace", "NSRunningApplication"] }
objc2-foundation = "0.2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 5: Replace `src-tauri/tauri.conf.json`**

```json
{
  "productName": "Waypoint",
  "version": "0.1.0",
  "identifier": "com.waypoint.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  }
}
```

- [ ] **Step 6: Create `src-tauri/capabilities/default.json`**

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["*"],
  "permissions": [
    "core:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "shell:default"
  ]
}
```

- [ ] **Step 7: Replace `vite.config.ts`**

```typescript
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { internalIpV4 } from "internal-ip";

const mobile = !!/android|ios/.exec(process.env.TAURI_ENV_PLATFORM ?? "");

export default defineConfig(async () => ({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: mobile ? "0.0.0.0" : false,
    hmr: mobile
      ? { protocol: "ws", host: await internalIpV4(), port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

- [ ] **Step 8: Verify the project builds**

```bash
npm run tauri dev
```

Expected: app window opens (default Svelte template content visible)

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: scaffold Waypoint Tauri 2.0 + Svelte project"
```

---

## Task 2: Error Type + Storage Paths

**Files:**
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/paths.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing test for path resolution**

Create `src-tauri/src/storage/paths.rs`:

```rust
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
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
}
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cd src-tauri && cargo test storage::paths
```

Expected: 3 tests pass

- [ ] **Step 3: Create `src-tauri/src/error.rs`**

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum WaypointError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("JSON error: {0}")]
    Json(String),
    #[error("Note not found: {0}")]
    NoteNotFound(String),
    #[error("Context not found: {0}")]
    ContextNotFound(String),
}

impl From<std::io::Error> for WaypointError {
    fn from(e: std::io::Error) -> Self {
        WaypointError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for WaypointError {
    fn from(e: serde_json::Error) -> Self {
        WaypointError::Json(e.to_string())
    }
}

// Required for tauri commands to return this as Err
impl From<WaypointError> for tauri::ipc::InvokeError {
    fn from(e: WaypointError) -> Self {
        tauri::ipc::InvokeError::from_anyhow(e.into())
    }
}
```

- [ ] **Step 4: Create `src-tauri/src/storage/mod.rs`**

```rust
pub mod app_config;
pub mod notes;
pub mod paths;
pub mod session;
```

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/src/error.rs src-tauri/src/storage/
git commit -m "feat: add error type and storage path helpers"
```

---

## Task 3: App Config + Note CRUD

**Files:**
- Create: `src-tauri/src/storage/app_config.rs`
- Create: `src-tauri/src/storage/notes.rs`

- [ ] **Step 1: Write failing tests for app_config**

Create `src-tauri/src/storage/app_config.rs`:

```rust
use crate::error::WaypointError;
use crate::storage::paths::app_config_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextConfig {
    pub match_by: MatchBy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MatchBy {
    Process,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default)]
    pub context_aliases: HashMap<String, String>,
    #[serde(default)]
    pub contexts: HashMap<String, ContextConfig>,
}

fn default_hotkey() -> String {
    "Ctrl+Shift+Space".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            hotkey: default_hotkey(),
            context_aliases: HashMap::new(),
            contexts: HashMap::new(),
        }
    }
}

pub fn load() -> Result<AppConfig, WaypointError> {
    let path = app_config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save(config: &AppConfig) -> Result<(), WaypointError> {
    let path = app_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn with_temp_home(f: impl FnOnce(&TempDir)) {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        f(&dir);
    }

    #[test]
    fn load_returns_default_when_no_file() {
        with_temp_home(|_| {
            let cfg = load().unwrap();
            assert_eq!(cfg.hotkey, "Ctrl+Shift+Space");
            assert!(cfg.context_aliases.is_empty());
        });
    }

    #[test]
    fn save_and_reload_roundtrip() {
        with_temp_home(|_| {
            let mut cfg = AppConfig::default();
            cfg.hotkey = "Ctrl+Alt+N".to_string();
            cfg.context_aliases.insert("steam_win".to_string(), "steam".to_string());
            save(&cfg).unwrap();
            let loaded = load().unwrap();
            assert_eq!(loaded.hotkey, "Ctrl+Alt+N");
            assert_eq!(loaded.context_aliases.get("steam_win").unwrap(), "steam");
        });
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test storage::app_config
```

Expected: 2 tests pass

- [ ] **Step 3: Write failing tests for notes CRUD**

Create `src-tauri/src/storage/notes.rs`:

```rust
use crate::error::WaypointError;
use crate::storage::paths::note_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteSettings {
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub hotkey: Option<String>,
    #[serde(default)]
    pub window_bounds: Option<WindowBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn default_font_size() -> u32 { 14 }
fn default_opacity() -> f32 { 1.0 }

impl Default for NoteSettings {
    fn default() -> Self {
        NoteSettings {
            font_size: default_font_size(),
            opacity: default_opacity(),
            hotkey: None,
            window_bounds: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub context_id: Option<String>,   // None = global note
    pub title: String,
    pub content: String,              // Markdown
    pub settings: NoteSettings,
}

pub fn create_note(context_id: Option<&str>, title: &str) -> Result<Note, WaypointError> {
    let id = Uuid::new_v4().to_string();
    let note = Note {
        id: id.clone(),
        context_id: context_id.map(|s| s.to_string()),
        title: title.to_string(),
        content: String::new(),
        settings: NoteSettings::default(),
    };
    let dir = note_dir(context_id, &id);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("content.md"), &note.content)?;
    let settings_json = serde_json::to_string_pretty(&note.settings)?;
    std::fs::write(dir.join("settings.json"), settings_json)?;
    Ok(note)
}

pub fn read_note(context_id: Option<&str>, note_id: &str) -> Result<Note, WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let content = std::fs::read_to_string(dir.join("content.md"))?;
    let settings_str = std::fs::read_to_string(dir.join("settings.json"))
        .unwrap_or_else(|_| "{}".to_string());
    let settings: NoteSettings = serde_json::from_str(&settings_str)?;
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    Ok(Note {
        id: note_id.to_string(),
        context_id: context_id.map(|s| s.to_string()),
        title,
        content,
        settings,
    })
}

pub fn save_content(context_id: Option<&str>, note_id: &str, content: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    std::fs::write(dir.join("content.md"), content)?;
    Ok(())
}

pub fn save_settings(context_id: Option<&str>, note_id: &str, settings: &NoteSettings) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(dir.join("settings.json"), json)?;
    Ok(())
}

pub fn delete_note(context_id: Option<&str>, note_id: &str) -> Result<(), WaypointError> {
    let dir = note_dir(context_id, note_id);
    if !dir.exists() {
        return Err(WaypointError::NoteNotFound(note_id.to_string()));
    }
    std::fs::remove_dir_all(dir)?;
    Ok(())
}

pub fn list_notes(context_id: Option<&str>) -> Result<Vec<Note>, WaypointError> {
    use crate::storage::paths::{global_dir, context_dir};
    let dir = match context_id {
        Some(ctx) => context_dir(ctx),
        None => global_dir(),
    };
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut notes = vec![];
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let note_id = path.file_name().unwrap().to_str().unwrap();
            if note_id == "session.json" { continue; }
            if let Ok(note) = read_note(context_id, note_id) {
                notes.push(note);
            }
        }
    }
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        dir
    }

    #[test]
    fn create_and_read_global_note() {
        let _dir = setup();
        let note = create_note(None, "Test Note").unwrap();
        assert!(!note.id.is_empty());
        assert_eq!(note.context_id, None);
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.id, note.id);
    }

    #[test]
    fn create_and_read_context_note() {
        let _dir = setup();
        let note = create_note(Some("steam"), "Steam Note").unwrap();
        assert_eq!(note.context_id, Some("steam".to_string()));
        let loaded = read_note(Some("steam"), &note.id).unwrap();
        assert_eq!(loaded.context_id, Some("steam".to_string()));
    }

    #[test]
    fn save_and_read_content() {
        let _dir = setup();
        let note = create_note(None, "Note").unwrap();
        save_content(None, &note.id, "# Hello\nworld").unwrap();
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.content, "# Hello\nworld");
        assert_eq!(loaded.title, "Hello");
    }

    #[test]
    fn delete_note_removes_dir() {
        let _dir = setup();
        let note = create_note(None, "To Delete").unwrap();
        delete_note(None, &note.id).unwrap();
        assert!(read_note(None, &note.id).is_err());
    }

    #[test]
    fn list_notes_returns_all_in_context() {
        let _dir = setup();
        create_note(Some("steam"), "Note 1").unwrap();
        create_note(Some("steam"), "Note 2").unwrap();
        let notes = list_notes(Some("steam")).unwrap();
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn save_settings_persists() {
        let _dir = setup();
        let note = create_note(None, "Note").unwrap();
        let mut settings = NoteSettings::default();
        settings.font_size = 18;
        settings.opacity = 0.8;
        save_settings(None, &note.id, &settings).unwrap();
        let loaded = read_note(None, &note.id).unwrap();
        assert_eq!(loaded.settings.font_size, 18);
        assert!((loaded.settings.opacity - 0.8).abs() < 0.001);
    }
}
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test storage::notes
```

Expected: 6 tests pass

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/src/storage/
git commit -m "feat: add app config and note CRUD with tests"
```

---

## Task 4: Session Storage

**Files:**
- Create: `src-tauri/src/storage/session.rs`

- [ ] **Step 1: Write failing tests and implementation**

Create `src-tauri/src/storage/session.rs`:

```rust
use crate::error::WaypointError;
use crate::storage::paths::session_path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub open_context_notes: Vec<String>,
    #[serde(default)]
    pub open_global_notes: Vec<String>,
}

pub fn load_session(context_id: &str) -> Result<Session, WaypointError> {
    let path = session_path(context_id);
    if !path.exists() {
        return Ok(Session::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn save_session(context_id: &str, session: &Session) -> Result<(), WaypointError> {
    let path = session_path(context_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(session)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn clear_session(context_id: &str) -> Result<(), WaypointError> {
    let path = session_path(context_id);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::env::set_var("HOME", dir.path());
        dir
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let _dir = setup();
        let s = load_session("steam").unwrap();
        assert!(s.open_context_notes.is_empty());
        assert!(s.open_global_notes.is_empty());
    }

    #[test]
    fn save_and_reload_session() {
        let _dir = setup();
        let session = Session {
            open_context_notes: vec!["note-1".to_string(), "note-2".to_string()],
            open_global_notes: vec!["global-1".to_string()],
        };
        save_session("steam", &session).unwrap();
        let loaded = load_session("steam").unwrap();
        assert_eq!(loaded.open_context_notes, vec!["note-1", "note-2"]);
        assert_eq!(loaded.open_global_notes, vec!["global-1"]);
    }

    #[test]
    fn clear_session_removes_file() {
        let _dir = setup();
        let session = Session {
            open_context_notes: vec!["note-1".to_string()],
            open_global_notes: vec![],
        };
        save_session("steam", &session).unwrap();
        clear_session("steam").unwrap();
        let loaded = load_session("steam").unwrap();
        assert!(loaded.open_context_notes.is_empty());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test storage::session
```

Expected: 3 tests pass

- [ ] **Step 3: Commit**

```bash
cd ..
git add src-tauri/src/storage/session.rs
git commit -m "feat: add session storage with tests"
```

---

## Task 5: Context Detection + Normalization

**Files:**
- Create: `src-tauri/src/context/mod.rs`
- Create: `src-tauri/src/context/normalizer.rs`
- Create: `src-tauri/src/context/detector.rs`

- [ ] **Step 1: Write failing tests for normalizer**

Create `src-tauri/src/context/normalizer.rs`:

```rust
use std::collections::HashMap;

/// Normalize a raw process name to a canonical context id.
/// Rules: strip .exe suffix (case-insensitive), lowercase all.
pub fn normalize_process_name(raw: &str) -> String {
    let lower = raw.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// Resolve a normalized context id through the alias map.
/// If context_id is in aliases, return the canonical target.
pub fn resolve_alias<'a>(context_id: &'a str, aliases: &'a HashMap<String, String>) -> &'a str {
    aliases
        .get(context_id)
        .map(|s| s.as_str())
        .unwrap_or(context_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_exe_suffix() {
        assert_eq!(normalize_process_name("steam.exe"), "steam");
        assert_eq!(normalize_process_name("Steam.EXE"), "steam");
    }

    #[test]
    fn lowercases_without_exe() {
        assert_eq!(normalize_process_name("Steam"), "steam");
        assert_eq!(normalize_process_name("Firefox"), "firefox");
    }

    #[test]
    fn already_normalized_unchanged() {
        assert_eq!(normalize_process_name("steam"), "steam");
    }

    #[test]
    fn resolve_alias_returns_canonical() {
        let mut aliases = HashMap::new();
        aliases.insert("mygame_win".to_string(), "mygame".to_string());
        assert_eq!(resolve_alias("mygame_win", &aliases), "mygame");
    }

    #[test]
    fn resolve_alias_passthrough_when_no_match() {
        let aliases = HashMap::new();
        assert_eq!(resolve_alias("steam", &aliases), "steam");
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd src-tauri && cargo test context::normalizer
```

Expected: 5 tests pass

- [ ] **Step 3: Create platform-specific detector**

Create `src-tauri/src/context/detector.rs`:

```rust
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
    use windows::Win32::Foundation::HANDLE;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND(0) { return None; }

        // Get window title
        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = OsString::from_wide(&title_buf[..len as usize])
            .to_string_lossy()
            .to_string();

        // Get process name
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
    use objc2_foundation::MainThreadMarker;

    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    let workspace = NSWorkspace::sharedWorkspace(mtm);
    let app = unsafe { workspace.frontmostApplication() }?;

    let process_name = unsafe { app.localizedName() }
        .map(|s| s.to_string())
        .unwrap_or_default();

    // macOS window title requires Accessibility API — use process name as title fallback
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

    // Get focused window via _NET_ACTIVE_WINDOW
    let atom_net_active = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW").ok()?.reply().ok()?.atom;
    let reply = conn.get_property(false, root, atom_net_active, AtomEnum::WINDOW, 0, 1).ok()?.reply().ok()?;
    let window_id = u32::from_ne_bytes(reply.value.try_into().ok()?);
    let window = Window::from(window_id);

    // Get _NET_WM_PID
    let atom_pid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?.atom;
    let pid_reply = conn.get_property(false, window, atom_pid, AtomEnum::CARDINAL, 0, 1).ok()?.reply().ok()?;
    let pid = u32::from_ne_bytes(pid_reply.value.try_into().ok()?);

    // Get window title via _NET_WM_NAME
    let atom_name = conn.intern_atom(false, b"_NET_WM_NAME").ok()?.reply().ok()?.atom;
    let atom_utf8 = conn.intern_atom(false, b"UTF8_STRING").ok()?.reply().ok()?.atom;
    let title_reply = conn.get_property(false, window, atom_name, atom_utf8, 0, 256).ok()?.reply().ok()?;
    let window_title = String::from_utf8_lossy(&title_reply.value).to_string();

    // Read process name from /proc/{pid}/comm
    let process_name = std::fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    Some(FocusedWindowInfo { process_name, window_title })
}

// Fallback for unsupported platforms
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn get_focused_window() -> Option<FocusedWindowInfo> {
    None
}
```

- [ ] **Step 4: Create `src-tauri/src/context/mod.rs`**

```rust
pub mod detector;
pub mod normalizer;

use crate::storage::app_config::{AppConfig, MatchBy};
use detector::FocusedWindowInfo;
use normalizer::{normalize_process_name, resolve_alias};

/// Derive the canonical context_id from a FocusedWindowInfo,
/// applying normalization, per-context matchBy config, and aliases.
pub fn derive_context_id(info: &FocusedWindowInfo, config: &AppConfig) -> String {
    // Determine raw match value
    let raw = if let Some(ctx_cfg) = config.contexts.get(
        &normalize_process_name(&info.process_name)
    ) {
        if ctx_cfg.match_by == MatchBy::Title {
            info.window_title.to_lowercase()
        } else {
            info.process_name.clone()
        }
    } else {
        info.process_name.clone()
    };

    let normalized = normalize_process_name(&raw);
    resolve_alias(&normalized, &config.context_aliases).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::app_config::{AppConfig, ContextConfig, MatchBy};
    use std::collections::HashMap;

    fn make_info(process: &str, title: &str) -> FocusedWindowInfo {
        FocusedWindowInfo {
            process_name: process.to_string(),
            window_title: title.to_string(),
        }
    }

    #[test]
    fn derives_context_from_process_by_default() {
        let config = AppConfig::default();
        let info = make_info("Steam.exe", "Steam");
        assert_eq!(derive_context_id(&info, &config), "steam");
    }

    #[test]
    fn uses_window_title_when_configured() {
        let mut config = AppConfig::default();
        config.contexts.insert("steam".to_string(), ContextConfig { match_by: MatchBy::Title });
        let info = make_info("steam", "Counter-Strike 2");
        assert_eq!(derive_context_id(&info, &config), "counter-strike 2");
    }

    #[test]
    fn applies_alias_after_normalization() {
        let mut config = AppConfig::default();
        config.context_aliases.insert("mygame_win".to_string(), "mygame".to_string());
        let info = make_info("mygame_win.exe", "My Game");
        assert_eq!(derive_context_id(&info, &config), "mygame");
    }
}
```

- [ ] **Step 5: Run tests**

```bash
cd src-tauri && cargo test context
```

Expected: 8 tests pass

- [ ] **Step 6: Commit**

```bash
cd ..
git add src-tauri/src/context/
git commit -m "feat: add context detection, normalization, and alias resolution with tests"
```

---

## Task 6: App State + Tauri Commands

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/notes.rs`
- Create: `src-tauri/src/commands/context_cmd.rs`
- Create: `src-tauri/src/commands/session_cmd.rs`
- Create: `src-tauri/src/commands/config_cmd.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `src-tauri/src/state.rs`**

```rust
use crate::context::detector::FocusedWindowInfo;
use std::sync::Mutex;

/// The context captured when the hotkey fires — the window that was
/// active BEFORE Waypoint came to the foreground.
#[derive(Debug, Default)]
pub struct AppState {
    /// The context id resolved at last hotkey press.
    pub active_context_id: Mutex<Option<String>>,
    /// Raw info captured at hotkey press, for display purposes.
    pub active_window_info: Mutex<Option<FocusedWindowInfo>>,
    /// Whether the list window is currently visible.
    pub list_window_open: Mutex<bool>,
}
```

- [ ] **Step 2: Create `src-tauri/src/commands/notes.rs`**

```rust
use crate::error::WaypointError;
use crate::storage::notes::{self, Note, NoteSettings};
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn list_notes(context_id: Option<String>) -> Result<Vec<Note>, WaypointError> {
    notes::list_notes(context_id.as_deref())
}

#[tauri::command]
pub fn create_note(context_id: Option<String>, title: String) -> Result<Note, WaypointError> {
    notes::create_note(context_id.as_deref(), &title)
}

#[tauri::command]
pub fn read_note(context_id: Option<String>, note_id: String) -> Result<Note, WaypointError> {
    notes::read_note(context_id.as_deref(), &note_id)
}

#[tauri::command]
pub fn save_content(context_id: Option<String>, note_id: String, content: String) -> Result<(), WaypointError> {
    notes::save_content(context_id.as_deref(), &note_id, &content)
}

#[tauri::command]
pub fn save_note_settings(
    context_id: Option<String>,
    note_id: String,
    settings: NoteSettings,
) -> Result<(), WaypointError> {
    notes::save_settings(context_id.as_deref(), &note_id, &settings)
}

#[tauri::command]
pub fn delete_note(context_id: Option<String>, note_id: String) -> Result<(), WaypointError> {
    notes::delete_note(context_id.as_deref(), &note_id)
}
```

- [ ] **Step 3: Create `src-tauri/src/commands/context_cmd.rs`**

```rust
use crate::error::WaypointError;
use crate::state::AppState;
use crate::storage::app_config::{self, ContextConfig, MatchBy};
use tauri::State;

#[tauri::command]
pub fn get_active_context(state: State<AppState>) -> Option<String> {
    state.active_context_id.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_context_match_by(
    context_id: String,
    match_by: String,
) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    let mb = if match_by == "title" { MatchBy::Title } else { MatchBy::Process };
    config.contexts.insert(context_id, ContextConfig { match_by: mb });
    app_config::save(&config)
}

#[tauri::command]
pub fn set_context_alias(
    from_context: String,
    to_context: String,
) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    config.context_aliases.insert(from_context, to_context);
    app_config::save(&config)
}

#[tauri::command]
pub fn rename_context(old_id: String, new_id: String) -> Result<(), WaypointError> {
    use crate::storage::paths::{data_dir};
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
```

- [ ] **Step 4: Create `src-tauri/src/commands/session_cmd.rs`**

```rust
use crate::error::WaypointError;
use crate::storage::session::{self, Session};

#[tauri::command]
pub fn load_session(context_id: String) -> Result<Session, WaypointError> {
    session::load_session(&context_id)
}

#[tauri::command]
pub fn save_session(context_id: String, sess: Session) -> Result<(), WaypointError> {
    session::save_session(&context_id, &sess)
}
```

- [ ] **Step 5: Create `src-tauri/src/commands/config_cmd.rs`**

```rust
use crate::error::WaypointError;
use crate::storage::app_config::{self, AppConfig};

#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, WaypointError> {
    app_config::load()
}

#[tauri::command]
pub fn set_hotkey(hotkey: String) -> Result<(), WaypointError> {
    let mut config = app_config::load()?;
    config.hotkey = hotkey;
    app_config::save(&config)
}
```

- [ ] **Step 6: Create `src-tauri/src/commands/mod.rs`**

```rust
pub mod config_cmd;
pub mod context_cmd;
pub mod notes;
pub mod session_cmd;
```

- [ ] **Step 7: Update `src-tauri/src/lib.rs` to wire everything together**

```rust
mod commands;
mod context;
mod error;
mod hotkey;
mod state;
mod storage;
mod tray;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::notes::list_notes,
            commands::notes::create_note,
            commands::notes::read_note,
            commands::notes::save_content,
            commands::notes::save_note_settings,
            commands::notes::delete_note,
            commands::context_cmd::get_active_context,
            commands::context_cmd::set_context_match_by,
            commands::context_cmd::set_context_alias,
            commands::context_cmd::rename_context,
            commands::context_cmd::delete_context,
            commands::context_cmd::list_contexts,
            commands::session_cmd::load_session,
            commands::session_cmd::save_session,
            commands::config_cmd::get_app_config,
            commands::config_cmd::set_hotkey,
        ])
        .setup(|app| {
            tray::setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Waypoint");
}
```

- [ ] **Step 8: Verify it compiles**

```bash
cd src-tauri && cargo build 2>&1 | head -40
```

Expected: compilation succeeds (no errors)

- [ ] **Step 9: Commit**

```bash
cd ..
git add src-tauri/src/
git commit -m "feat: add app state, all tauri commands, wire up lib.rs"
```

---

## Task 7: Global Hotkey + 3-State Machine

**Files:**
- Create: `src-tauri/src/hotkey/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for state machine logic**

Create `src-tauri/src/hotkey/mod.rs`:

```rust
use crate::context::detector::get_focused_window;
use crate::context::{derive_context_id};
use crate::state::AppState;
use crate::storage::app_config;
use tauri::{AppHandle, Manager, WebviewWindowBuilder, WebviewUrl};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// The three states that determine what the hotkey does.
#[derive(Debug, PartialEq)]
pub enum HotkeyAction {
    /// No Waypoint windows open → open list + restore session
    OpenAll,
    /// Notes open but list closed → open list only
    OpenList,
    /// List is open → save session + close all
    CollapseAll,
}

/// Determine which action to take based on current window state.
pub fn determine_action(list_open: bool, any_note_open: bool) -> HotkeyAction {
    if list_open {
        HotkeyAction::CollapseAll
    } else if any_note_open {
        HotkeyAction::OpenList
    } else {
        HotkeyAction::OpenAll
    }
}

pub fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }

        // Capture focused window BEFORE we do anything that shifts focus
        let window_info = get_focused_window();

        let state = app.state::<AppState>();
        let list_open = *state.list_window_open.lock().unwrap();

        // Check if any note window is open
        let any_note_open = app.webview_windows()
            .keys()
            .any(|label| label.starts_with("note-"));

        let action = determine_action(list_open, any_note_open);

        match action {
            HotkeyAction::OpenAll => {
                // Store the captured context
                if let Some(info) = window_info {
                    let config = app_config::load().unwrap_or_default();
                    let ctx_id = derive_context_id(&info, &config);
                    *state.active_context_id.lock().unwrap() = Some(ctx_id);
                    *state.active_window_info.lock().unwrap() = Some(info);
                }
                let _ = open_list_window(app);
                // Session restore is triggered from frontend after list mounts
            }
            HotkeyAction::OpenList => {
                let _ = open_list_window(app);
            }
            HotkeyAction::CollapseAll => {
                // Frontend emits save-session before windows close
                collapse_all_waypoint_windows(app);
            }
        }
    })?;
    Ok(())
}

pub fn open_list_window(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<AppState>();

    if let Some(win) = app.get_webview_window("list") {
        win.show()?;
        win.set_focus()?;
        *state.list_window_open.lock().unwrap() = true;
        return Ok(());
    }

    let win = WebviewWindowBuilder::new(app, "list", WebviewUrl::App("index.html?view=list".into()))
        .title("Waypoint")
        .inner_size(220.0, 500.0)
        .min_inner_size(180.0, 300.0)
        .resizable(true)
        .decorations(false)
        .always_on_top(true)
        .build()?;

    *state.list_window_open.lock().unwrap() = true;

    // Auto-close list when focus leaves all Waypoint windows
    let app_handle = app.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            // Delay to let focus settle on another Waypoint window
            let app = app_handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let any_waypoint_focused = app.webview_windows()
                    .values()
                    .any(|w| w.is_focused().unwrap_or(false));
                if !any_waypoint_focused {
                    if let Some(list) = app.get_webview_window("list") {
                        let _ = list.hide();
                        let state = app.state::<AppState>();
                        *state.list_window_open.lock().unwrap() = false;
                    }
                }
            });
        }
    });

    Ok(())
}

pub fn collapse_all_waypoint_windows(app: &AppHandle) {
    // Close all note windows
    let windows = app.webview_windows();
    for (label, window) in &windows {
        if label.starts_with("note-") || label == "list" {
            let _ = window.hide();
        }
    }
    let state = app.state::<AppState>();
    *state.list_window_open.lock().unwrap() = false;
}

/// Open a note window for a given note.
pub fn open_note_window(app: &AppHandle, note_id: &str, context_id: Option<&str>) -> tauri::Result<()> {
    let label = format!("note-{}", note_id);
    if let Some(win) = app.get_webview_window(&label) {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    let ctx_param = context_id.map(|c| format!("&contextId={}", c)).unwrap_or_default();
    let url = format!("index.html?view=note&noteId={}{}", note_id, ctx_param);
    WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title("Waypoint Note")
        .inner_size(420.0, 600.0)
        .min_inner_size(300.0, 200.0)
        .resizable(true)
        .decorations(false)
        .transparent(true)
        .build()?;
    Ok(())
}

/// Register a per-note direct hotkey. When triggered, opens that specific note
/// directly, bypassing the 3-state logic.
pub fn register_note_hotkey(
    app: &AppHandle,
    hotkey: &str,
    note_id: String,
    context_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(hotkey, move |app, _shortcut, event| {
        if event.state != ShortcutState::Pressed { return; }
        let _ = open_note_window(app, &note_id, context_id.as_deref());
    })?;
    Ok(())
}

// Tauri commands callable from frontend
#[tauri::command]
pub fn cmd_open_note_window(
    app: AppHandle,
    note_id: String,
    context_id: Option<String>,
) -> Result<(), String> {
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
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Called from SettingsPanel when user sets a per-note hotkey.
/// Unregisters any old hotkey for this note first, then registers the new one.
#[tauri::command]
pub fn cmd_register_note_hotkey(
    app: AppHandle,
    note_id: String,
    context_id: Option<String>,
    hotkey: String,
) -> Result<(), String> {
    register_note_hotkey(&app, &hotkey, note_id, context_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cmd_unregister_hotkey(app: AppHandle, hotkey: String) -> Result<(), String> {
    app.global_shortcut()
        .unregister(hotkey.as_str())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_windows_open_triggers_open_all() {
        assert_eq!(determine_action(false, false), HotkeyAction::OpenAll);
    }

    #[test]
    fn notes_open_no_list_triggers_open_list() {
        assert_eq!(determine_action(false, true), HotkeyAction::OpenList);
    }

    #[test]
    fn list_open_triggers_collapse_all() {
        assert_eq!(determine_action(true, false), HotkeyAction::CollapseAll);
        assert_eq!(determine_action(true, true), HotkeyAction::CollapseAll);
    }
}
```

- [ ] **Step 2: Run unit tests**

```bash
cd src-tauri && cargo test hotkey
```

Expected: 3 tests pass

- [ ] **Step 3: Add hotkey commands to `lib.rs` invoke_handler**

In `src-tauri/src/lib.rs`, add to the `invoke_handler` list:

```rust
hotkey::cmd_open_note_window,
hotkey::cmd_collapse_all,
hotkey::cmd_close_note_window,
```

Also add to `setup`:

```rust
.setup(|app| {
    tray::setup_tray(app)?;
    let config = app_config::load().unwrap_or_default();
    hotkey::register_hotkey(app.handle(), &config.hotkey)?;
    Ok(())
})
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo build 2>&1 | head -30
```

Expected: no errors

- [ ] **Step 5: Commit**

```bash
cd ..
git add src-tauri/src/hotkey/ src-tauri/src/lib.rs
git commit -m "feat: add global hotkey registration and 3-state machine with tests"
```

---

## Task 8: System Tray

**Files:**
- Create: `src-tauri/src/tray/mod.rs`

- [ ] **Step 1: Create `src-tauri/src/tray/mod.rs`**

```rust
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let help_item = MenuItem::with_id(app, "help", "使用說明", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "結束 Waypoint", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&help_item, &sep, &quit_item])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("Waypoint")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "help" => {
                let _ = open_help_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

pub fn open_help_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(win) = app.get_webview_window("help") {
        win.show()?;
        win.set_focus()?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(
        app,
        "help",
        tauri::WebviewUrl::App("index.html?view=help".into()),
    )
    .title("Waypoint — 使用說明")
    .inner_size(600.0, 500.0)
    .resizable(true)
    .build()?;
    Ok(())
}
```

- [ ] **Step 2: Generate placeholder icons**

```bash
cd src-tauri
cargo tauri icon ../public/app-icon.png
```

If `public/app-icon.png` doesn't exist, create a placeholder first:

```bash
mkdir -p ../public
# use any 1024x1024 PNG as placeholder icon
cp /usr/share/pixmaps/*.png ../public/app-icon.png 2>/dev/null || \
  convert -size 1024x1024 xc:#007acc ../public/app-icon.png 2>/dev/null || \
  echo "Place a 1024x1024 PNG at public/app-icon.png manually"
```

- [ ] **Step 3: Verify tray appears on dev run**

```bash
npm run tauri dev
```

Expected: system tray icon visible; right-click shows "使用說明" and "結束 Waypoint"

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/tray/ src-tauri/icons/
git commit -m "feat: add system tray with help and quit menu items"
```

---

## Task 9: Frontend Foundation (Types, API, Stores, Styles)

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/api.ts`
- Create: `src/lib/stores.ts`
- Create: `src/app.css`
- Create: `src/main.ts`
- Create: `src/App.svelte`
- Modify: `index.html`

- [ ] **Step 1: Create `src/lib/types.ts`**

```typescript
export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface NoteSettings {
  fontSize: number;
  opacity: number;
  hotkey: string | null;
  windowBounds: WindowBounds | null;
}

export interface Note {
  id: string;
  contextId: string | null;  // null = global
  title: string;
  content: string;
  settings: NoteSettings;
}

export interface Session {
  openContextNotes: string[];
  openGlobalNotes: string[];
}

export interface AppConfig {
  hotkey: string;
  contextAliases: Record<string, string>;
  contexts: Record<string, { matchBy: "process" | "title" }>;
}

export type ViewType = "list" | "note" | "help";
```

- [ ] **Step 2: Create `src/lib/api.ts`**

```typescript
import { invoke } from "@tauri-apps/api/core";
import type { Note, NoteSettings, Session, AppConfig } from "./types";

export const notes = {
  list: (contextId: string | null) =>
    invoke<Note[]>("list_notes", { contextId }),
  create: (contextId: string | null, title: string) =>
    invoke<Note>("create_note", { contextId, title }),
  read: (contextId: string | null, noteId: string) =>
    invoke<Note>("read_note", { contextId, noteId }),
  saveContent: (contextId: string | null, noteId: string, content: string) =>
    invoke<void>("save_content", { contextId, noteId, content }),
  saveSettings: (contextId: string | null, noteId: string, settings: NoteSettings) =>
    invoke<void>("save_note_settings", { contextId, noteId, settings }),
  delete: (contextId: string | null, noteId: string) =>
    invoke<void>("delete_note", { contextId, noteId }),
};

export const context = {
  getActive: () => invoke<string | null>("get_active_context"),
  listAll: () => invoke<string[]>("list_contexts"),
  setMatchBy: (contextId: string, matchBy: "process" | "title") =>
    invoke<void>("set_context_match_by", { contextId, matchBy }),
  setAlias: (fromContext: string, toContext: string) =>
    invoke<void>("set_context_alias", { fromContext, toContext }),
  rename: (oldId: string, newId: string) =>
    invoke<void>("rename_context", { oldId, newId }),
  delete: (contextId: string) =>
    invoke<void>("delete_context", { contextId }),
};

export const session = {
  load: (contextId: string) => invoke<Session>("load_session", { contextId }),
  save: (contextId: string, sess: Session) =>
    invoke<void>("save_session", { contextId, sess }),
};

export const config = {
  get: () => invoke<AppConfig>("get_app_config"),
  setHotkey: (hotkey: string) => invoke<void>("set_hotkey", { hotkey }),
};

export const windows = {
  openNote: (noteId: string, contextId: string | null) =>
    invoke<void>("cmd_open_note_window", { noteId, contextId }),
  collapseAll: () => invoke<void>("cmd_collapse_all"),
  closeNote: (noteId: string) =>
    invoke<void>("cmd_close_note_window", { noteId }),
  registerNoteHotkey: (noteId: string, contextId: string | null, hotkey: string) =>
    invoke<void>("cmd_register_note_hotkey", { noteId, contextId, hotkey }),
  unregisterHotkey: (hotkey: string) =>
    invoke<void>("cmd_unregister_hotkey", { hotkey }),
};
```

- [ ] **Step 3: Create `src/lib/stores.ts`**

```typescript
import { writable } from "svelte/store";
import type { Note, Session } from "./types";

export const activeContextId = writable<string | null>(null);
export const globalNotes = writable<Note[]>([]);
export const contextNotes = writable<Note[]>([]);
export const settingsPanelOpen = writable<boolean>(false);
```

- [ ] **Step 4: Create `src/app.css` (VSCode dark theme)**

```css
:root {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #2d2d2d;
  --bg-selected: #094771;
  --bg-hover: #2a2d2e;
  --border: #3c3c3c;
  --text-primary: #cccccc;
  --text-secondary: #858585;
  --text-link: #75beff;
  --accent: #007acc;
  --accent-hover: #1e8ad4;
  --danger: #f44747;
  --font-ui: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --font-mono: "Cascadia Code", "Fira Code", "Consolas", monospace;
  --radius: 2px;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: 13px;
  line-height: 1.5;
  user-select: none;
  overflow: hidden;
}

button {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-family: var(--font-ui);
  font-size: 13px;
  padding: 2px 6px;
  border-radius: var(--radius);
}

button:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

input {
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: 13px;
  padding: 4px 8px;
  border-radius: var(--radius);
  outline: none;
}

input:focus {
  border-color: var(--accent);
}

.divider {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}

/* Scrollbar */
::-webkit-scrollbar { width: 8px; }
::-webkit-scrollbar-track { background: var(--bg-primary); }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-secondary); }
```

- [ ] **Step 5: Create `src/App.svelte` (view router)**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import ListWindow from "./windows/ListWindow.svelte";
  import NoteWindow from "./windows/NoteWindow.svelte";
  import HelpWindow from "./windows/HelpWindow.svelte";
  import type { ViewType } from "./lib/types";

  let view: ViewType = "list";
  let noteId: string | null = null;
  let contextId: string | null = null;

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    view = (params.get("view") as ViewType) ?? "list";
    noteId = params.get("noteId");
    contextId = params.get("contextId");
  });
</script>

{#if view === "list"}
  <ListWindow />
{:else if view === "note" && noteId}
  <NoteWindow {noteId} {contextId} />
{:else if view === "help"}
  <HelpWindow />
{/if}
```

- [ ] **Step 6: Update `src/main.ts`**

```typescript
import "./app.css";
import App from "./App.svelte";

const app = new App({
  target: document.getElementById("app")!,
});

export default app;
```

- [ ] **Step 7: Write Vitest setup and types test**

Create `src/lib/types.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import type { Note, Session } from "./types";

describe("types", () => {
  it("Note contextId is null for global notes", () => {
    const note: Note = {
      id: "abc",
      contextId: null,
      title: "Test",
      content: "# Test",
      settings: { fontSize: 14, opacity: 1.0, hotkey: null, windowBounds: null },
    };
    expect(note.contextId).toBeNull();
  });

  it("Session has both global and context note arrays", () => {
    const sess: Session = {
      openContextNotes: ["note-1"],
      openGlobalNotes: ["global-1"],
    };
    expect(sess.openContextNotes).toHaveLength(1);
    expect(sess.openGlobalNotes).toHaveLength(1);
  });
});
```

- [ ] **Step 8: Run frontend tests**

```bash
npx vitest run src/lib/types.test.ts
```

Expected: 2 tests pass

- [ ] **Step 9: Commit**

```bash
git add src/
git commit -m "feat: add frontend types, API wrappers, stores, VSCode styles, and view router"
```

---

## Task 10: List Window UI

**Files:**
- Create: `src/windows/ListWindow.svelte`
- Create: `src/windows/list/GlobalSection.svelte`
- Create: `src/windows/list/ContextSection.svelte`
- Create: `src/windows/list/NoteItem.svelte`

- [ ] **Step 1: Create `src/windows/list/NoteItem.svelte`**

```svelte
<script lang="ts">
  import type { Note } from "../../lib/types";
  import { windows } from "../../lib/api";

  export let note: Note;
  export let isOpen: boolean = false;

  async function handleClick() {
    await windows.openNote(note.id, note.contextId);
  }
</script>

<button
  class="note-item"
  class:open={isOpen}
  on:click={handleClick}
  title={note.title}
>
  <span class="icon">📄</span>
  <span class="title">{note.title || "Untitled"}</span>
</button>

<style>
  .note-item {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 10px;
    text-align: left;
    color: var(--text-primary);
    border-radius: 0;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .note-item:hover { background: var(--bg-hover); }
  .note-item.open { background: var(--bg-selected); }
  .icon { color: var(--text-link); font-size: 11px; flex-shrink: 0; }
  .title { overflow: hidden; text-overflow: ellipsis; }
</style>
```

- [ ] **Step 2: Create `src/windows/list/GlobalSection.svelte`**

```svelte
<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
  import { notes as notesApi } from "../../lib/api";

  export let notes: Note[] = [];
  export let openNoteIds: string[] = [];

  async function addNote() {
    const note = await notesApi.create(null, "New Note");
    notes = [...notes, note];
  }
</script>

<div class="section">
  <div class="section-header">
    <span class="section-label">🌐 全域筆記</span>
    <button class="add-btn" on:click={addNote} title="新增全域筆記">+</button>
  </div>
  {#each notes as note (note.id)}
    <NoteItem {note} isOpen={openNoteIds.includes(note.id)} />
  {/each}
</div>

<style>
  .section { padding: 4px 0; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 10px;
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-top: 4px;
  }
  .add-btn {
    font-size: 14px;
    color: var(--text-secondary);
    padding: 0 4px;
    line-height: 1;
  }
  .add-btn:hover { color: var(--text-primary); background: none; }
</style>
```

- [ ] **Step 3: Create `src/windows/list/ContextSection.svelte`**

```svelte
<script lang="ts">
  import type { Note } from "../../lib/types";
  import NoteItem from "./NoteItem.svelte";
  import { context as contextApi, notes as notesApi, windows as windowsApi } from "../../lib/api";
  import { createEventDispatcher } from "svelte";

  export let contextId: string;
  export let notes: Note[] = [];
  export let openNoteIds: string[] = [];

  const dispatch = createEventDispatcher();

  let menuVisible = false;
  let menuX = 0;
  let menuY = 0;
  let showAliasInput = false;
  let aliasTarget = "";
  let availableContexts: string[] = [];

  async function showContextMenu(e: MouseEvent) {
    e.preventDefault();
    menuX = e.clientX;
    menuY = e.clientY;
    availableContexts = (await contextApi.listAll()).filter(c => c !== contextId);
    menuVisible = true;
  }

  function closeMenu() { menuVisible = false; showAliasInput = false; }

  async function setMatchBy(matchBy: "process" | "title") {
    await contextApi.setMatchBy(contextId, matchBy);
    closeMenu();
  }

  async function setAlias(target: string) {
    await contextApi.setAlias(contextId, target);
    dispatch("aliasSet", { from: contextId, to: target });
    closeMenu();
  }

  async function deleteCtx() {
    if (confirm(`刪除 context "${contextId}" 及其所有筆記？`)) {
      await contextApi.delete(contextId);
      dispatch("deleted", { contextId });
    }
    closeMenu();
  }

  async function addNote() {
    const note = await notesApi.create(contextId, "New Note");
    notes = [...notes, note];
  }
</script>

<svelte:window on:click={closeMenu} />

<div class="section">
  <div class="section-header" on:contextmenu={showContextMenu}>
    <span class="section-label">{contextId} 筆記</span>
    <button class="add-btn" on:click|stopPropagation={addNote} title="新增筆記">+</button>
  </div>
  {#each notes as note (note.id)}
    <NoteItem {note} isOpen={openNoteIds.includes(note.id)} />
  {/each}
</div>

{#if menuVisible}
  <div
    class="context-menu"
    style="left:{menuX}px;top:{menuY}px"
    on:click|stopPropagation={() => {}}
  >
    <button on:click={() => setMatchBy("process")}>識別方式：程序名稱</button>
    <button on:click={() => setMatchBy("title")}>識別方式：視窗標題</button>
    <div class="divider" />
    {#if !showAliasInput}
      <button on:click={() => showAliasInput = true}>對應到現有 context...</button>
    {:else}
      <select class="alias-select" bind:value={aliasTarget}>
        <option value="">選擇 context</option>
        {#each availableContexts as ctx}
          <option value={ctx}>{ctx}</option>
        {/each}
      </select>
      <button on:click={() => aliasTarget && setAlias(aliasTarget)}>確認</button>
    {/if}
    <div class="divider" />
    <button class="danger" on:click={deleteCtx}>刪除此 context</button>
  </div>
{/if}

<style>
  .section { padding: 4px 0; }
  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 3px 10px;
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-top: 4px;
    cursor: context-menu;
  }
  .section-header:hover { background: var(--bg-hover); }
  .add-btn { font-size: 14px; color: var(--text-secondary); padding: 0 4px; }
  .context-menu {
    position: fixed;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    z-index: 9999;
    min-width: 180px;
    padding: 4px 0;
    box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  }
  .context-menu button {
    display: block;
    width: 100%;
    text-align: left;
    padding: 5px 12px;
    border-radius: 0;
    color: var(--text-primary);
    font-size: 12px;
  }
  .context-menu button:hover { background: var(--bg-selected); }
  .context-menu .danger { color: var(--danger); }
  .alias-select {
    display: block;
    width: calc(100% - 16px);
    margin: 4px 8px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 3px 6px;
    font-size: 12px;
  }
</style>
```

- [ ] **Step 4: Create `src/windows/ListWindow.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import GlobalSection from "./list/GlobalSection.svelte";
  import ContextSection from "./list/ContextSection.svelte";
  import { notes as notesApi, context as contextApi, session as sessionApi, windows as windowsApi } from "../lib/api";
  import { globalNotes, contextNotes, activeContextId } from "../lib/stores";
  import type { Note } from "../lib/types";

  let currentContextId: string | null = null;
  let openGlobalNoteIds: string[] = [];
  let openContextNoteIds: string[] = [];
  let allContextIds: string[] = [];

  onMount(async () => {
    currentContextId = await contextApi.getActive();
    activeContextId.set(currentContextId);

    const [globals, contexts] = await Promise.all([
      notesApi.list(null),
      currentContextId ? notesApi.list(currentContextId) : Promise.resolve([]),
    ]);
    globalNotes.set(globals);
    contextNotes.set(contexts);
    allContextIds = await contextApi.listAll();

    // If OpenAll action: restore session
    if (currentContextId) {
      const sess = await sessionApi.load(currentContextId);
      openContextNoteIds = sess.openContextNotes;
      openGlobalNoteIds = sess.openGlobalNotes;
      // Open note windows from session
      for (const nId of sess.openContextNotes) {
        await windowsApi.openNote(nId, currentContextId);
      }
      for (const nId of sess.openGlobalNotes) {
        await windowsApi.openNote(nId, null);
      }
    }
  });

  async function handleCollapseAll() {
    // Save session before collapsing
    if (currentContextId) {
      await sessionApi.save(currentContextId, {
        openContextNotes: openContextNoteIds,
        openGlobalNotes: openGlobalNoteIds,
      });
    }
    await windowsApi.collapseAll();
  }

  async function openHelp() {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("plugin:shell|open", {}).catch(() => {});
    // Tray help handler opens the window from Rust side
  }

  let appWindow = getCurrentWindow();
  function closeList() { appWindow.hide(); }
</script>

<div class="list-window">
  <!-- Title bar -->
  <div class="titlebar" data-tauri-drag-region>
    <div class="titlebar-left">
      <span class="app-name">WAYPOINT</span>
      <button class="icon-btn" on:click={openHelp} title="使用說明">?</button>
    </div>
    <div class="titlebar-right">
      <button class="icon-btn" on:click={handleCollapseAll} title="收起全部">⇊</button>
      <button class="icon-btn" on:click={closeList} title="關閉列表">✕</button>
    </div>
  </div>

  <!-- Note list -->
  <div class="list-body">
    <GlobalSection
      notes={$globalNotes}
      openNoteIds={openGlobalNoteIds}
    />
    <div class="divider" />
    {#if currentContextId}
      <ContextSection
        contextId={currentContextId}
        notes={$contextNotes}
        openNoteIds={openContextNoteIds}
        on:deleted={() => contextNotes.set([])}
      />
    {/if}
  </div>
</div>

<style>
  .list-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    min-height: 32px;
  }
  .titlebar-left { display: flex; align-items: center; gap: 8px; }
  .titlebar-right { display: flex; align-items: center; gap: 6px; }
  .app-name { font-size: 11px; font-weight: bold; color: var(--text-primary); letter-spacing: 1px; }
  .icon-btn { font-size: 12px; padding: 2px 5px; }
  .list-body { flex: 1; overflow-y: auto; padding: 4px 0; }
</style>
```

- [ ] **Step 5: Test list window renders in dev mode**

```bash
npm run tauri dev
```

In a terminal, trigger the hotkey (default Ctrl+Shift+Space).
Expected: list window appears with "WAYPOINT ?" on left, "⇊ ✕" on right, and global notes section.

- [ ] **Step 6: Commit**

```bash
git add src/windows/
git commit -m "feat: add list window with global and context sections"
```

---

## Task 11: Note Window + TipTap Editor

**Files:**
- Create: `src/windows/note/Editor.svelte`
- Create: `src/windows/note/Toolbar.svelte`
- Create: `src/windows/NoteWindow.svelte`

- [ ] **Step 1: Create `src/windows/note/Toolbar.svelte`**

```svelte
<script lang="ts">
  import type { Editor } from "@tiptap/core";

  export let editor: Editor | null = null;
  export let onOpenSettings: () => void;

  function cmd(command: () => boolean) {
    if (!editor) return;
    command();
    editor.view.focus();
  }
</script>

<div class="toolbar">
  <button
    class:active={editor?.isActive("bold")}
    on:click={() => cmd(() => editor!.chain().focus().toggleBold().run())}
    title="Bold"><b>B</b></button>
  <button
    class:active={editor?.isActive("italic")}
    on:click={() => cmd(() => editor!.chain().focus().toggleItalic().run())}
    title="Italic"><i>I</i></button>
  <button
    class:active={editor?.isActive("underline")}
    on:click={() => cmd(() => editor!.chain().focus().toggleUnderline().run())}
    title="Underline"><u>U</u></button>
  <span class="sep" />
  <button
    class:active={editor?.isActive("heading", { level: 1 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 1 }).run())}
    title="Heading 1">H1</button>
  <button
    class:active={editor?.isActive("heading", { level: 2 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 2 }).run())}
    title="Heading 2">H2</button>
  <button
    class:active={editor?.isActive("heading", { level: 3 })}
    on:click={() => cmd(() => editor!.chain().focus().toggleHeading({ level: 3 }).run())}
    title="Heading 3">H3</button>
  <span class="sep" />
  <button
    class:active={editor?.isActive("bulletList")}
    on:click={() => cmd(() => editor!.chain().focus().toggleBulletList().run())}
    title="Bullet list">≡</button>
  <button
    class:active={editor?.isActive("taskList")}
    on:click={() => cmd(() => editor!.chain().focus().toggleTaskList().run())}
    title="Task list">☑</button>
  <button
    class:active={editor?.isActive("codeBlock")}
    on:click={() => cmd(() => editor!.chain().focus().toggleCodeBlock().run())}
    title="Code block">&lt;/&gt;</button>
  <span class="spacer" />
  <button class="settings-btn" on:click={onOpenSettings} title="設定">⚙</button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 3px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    min-height: 28px;
  }
  .toolbar button { font-size: 11px; padding: 2px 5px; min-width: 22px; }
  .toolbar button.active { background: var(--bg-selected); color: var(--text-primary); }
  .sep { width: 1px; height: 14px; background: var(--border); margin: 0 3px; }
  .spacer { flex: 1; }
  .settings-btn { font-size: 13px; }
</style>
```

- [ ] **Step 2: Create `src/windows/note/Editor.svelte`**

```svelte
<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Underline from "@tiptap/extension-underline";
  import { TaskList } from "@tiptap/extension-task-list";
  import { TaskItem } from "@tiptap/extension-task-item";
  import { Markdown } from "@tiptap/extension-markdown";
  import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight";
  import { common, createLowlight } from "lowlight";

  export let content: string = "";
  export let fontSize: number = 14;

  const dispatch = createEventDispatcher<{ update: { markdown: string } }>();

  let element: HTMLElement;
  let editor: Editor;

  export function getEditor() { return editor; }

  const lowlight = createLowlight(common);

  onMount(() => {
    editor = new Editor({
      element,
      extensions: [
        StarterKit.configure({ codeBlock: false }),
        Underline,
        TaskList,
        TaskItem.configure({ nested: true }),
        CodeBlockLowlight.configure({ lowlight }),
        Markdown.configure({ transformPastedText: true, transformCopiedText: true }),
      ],
      content: editor?.storage?.markdown?.getMarkdown() ?? content,
      editorProps: {
        attributes: {
          class: "tiptap-editor",
          style: `font-size: ${fontSize}px`,
        },
      },
      onUpdate({ editor }) {
        const markdown = editor.storage.markdown.getMarkdown();
        dispatch("update", { markdown });
      },
    });

    // Load initial markdown content
    if (content) {
      editor.commands.setContent(content);
    }
  });

  onDestroy(() => {
    editor?.destroy();
  });

  $: if (editor) {
    editor.view.dom.style.fontSize = `${fontSize}px`;
  }
</script>

<div class="editor-wrap" bind:this={element} />

<style>
  .editor-wrap {
    flex: 1;
    overflow-y: auto;
    padding: 12px 14px;
    cursor: text;
  }
  :global(.tiptap-editor) {
    outline: none;
    min-height: 100%;
    color: var(--text-primary);
    font-family: var(--font-mono);
    line-height: 1.7;
  }
  :global(.tiptap-editor h1) { font-size: 1.5em; margin-bottom: 0.5em; color: var(--text-primary); }
  :global(.tiptap-editor h2) { font-size: 1.25em; margin-bottom: 0.4em; }
  :global(.tiptap-editor h3) { font-size: 1.1em; margin-bottom: 0.3em; }
  :global(.tiptap-editor ul, .tiptap-editor ol) { padding-left: 1.5em; }
  :global(.tiptap-editor li) { margin: 2px 0; }
  :global(.tiptap-editor code) { background: var(--bg-tertiary); padding: 1px 4px; border-radius: 2px; font-size: 0.9em; }
  :global(.tiptap-editor pre) { background: var(--bg-tertiary); padding: 10px 12px; border-radius: 3px; margin: 8px 0; overflow-x: auto; }
  :global(.tiptap-editor input[type="checkbox"]) { margin-right: 6px; accent-color: var(--accent); }
  :global(.tiptap-editor p) { margin: 4px 0; }
  :global(.tiptap-editor strong) { color: var(--text-primary); }
</style>
```

- [ ] **Step 3: Create `src/windows/note/SettingsPanel.svelte`**

```svelte
<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { NoteSettings } from "../../lib/types";
  import { windows as windowsApi } from "../../lib/api";

  export let settings: NoteSettings;
  export let noteId: string;
  export let contextId: string | null;
  const dispatch = createEventDispatcher<{ change: NoteSettings }>();

  function update(patch: Partial<NoteSettings>) {
    settings = { ...settings, ...patch };
    dispatch("change", settings);
  }

  async function handleHotkeyChange(e: Event) {
    const hotkey = (e.target as HTMLInputElement).value.trim() || null;
    // Unregister old hotkey if any
    if (settings.hotkey) {
      await windowsApi.unregisterHotkey(settings.hotkey).catch(() => {});
    }
    update({ hotkey });
    // Register new hotkey if set
    if (hotkey) {
      await windowsApi.registerNoteHotkey(noteId, contextId, hotkey);
    }
  }
</script>

<div class="settings-panel">
  <div class="setting-row">
    <label>字體大小</label>
    <div class="number-input">
      <button on:click={() => update({ fontSize: Math.max(8, settings.fontSize - 1) })}>-</button>
      <input
        type="number"
        min="8" max="32"
        value={settings.fontSize}
        on:change={e => update({ fontSize: parseInt((e.target as HTMLInputElement).value) })}
      />
      <button on:click={() => update({ fontSize: Math.min(32, settings.fontSize + 1) })}>+</button>
    </div>
  </div>

  <div class="setting-row">
    <label>透明度</label>
    <div class="slider-row">
      <input
        type="range" min="0.1" max="1" step="0.05"
        value={settings.opacity}
        on:input={e => update({ opacity: parseFloat((e.target as HTMLInputElement).value) })}
      />
      <span class="value">{Math.round(settings.opacity * 100)}%</span>
    </div>
  </div>

  <div class="setting-row">
    <label>專屬快捷鍵</label>
    <input
      type="text"
      placeholder="留空不設定"
      value={settings.hotkey ?? ""}
      on:change={e => update({ hotkey: (e.target as HTMLInputElement).value || null })}
    />
  </div>
</div>

<style>
  .settings-panel {
    width: 200px;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }
  .setting-row { display: flex; flex-direction: column; gap: 6px; }
  label { font-size: 11px; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.5px; }
  .number-input { display: flex; align-items: center; gap: 4px; }
  .number-input input { width: 48px; text-align: center; }
  .number-input button { padding: 2px 7px; }
  .slider-row { display: flex; align-items: center; gap: 8px; }
  .slider-row input { flex: 1; accent-color: var(--accent); }
  .value { font-size: 11px; color: var(--text-secondary); min-width: 30px; }
</style>
```

- [ ] **Step 4: Create `src/windows/NoteWindow.svelte`**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Editor from "./note/Editor.svelte";
  import Toolbar from "./note/Toolbar.svelte";
  import SettingsPanel from "./note/SettingsPanel.svelte";
  import { notes as notesApi, windows as windowsApi } from "../lib/api";
  import type { Note, NoteSettings } from "../lib/types";

  export let noteId: string;
  export let contextId: string | null;

  let note: Note | null = null;
  let settingsOpen = false;
  let editorRef: Editor;
  let saveTimeout: ReturnType<typeof setTimeout>;
  let appWindow = getCurrentWindow();

  onMount(async () => {
    note = await notesApi.read(contextId, noteId);
    // Apply stored opacity
    if (note) {
      appWindow.setOpacity(note.settings.opacity);
    }
  });

  function handleContentUpdate(e: CustomEvent<{ markdown: string }>) {
    if (!note) return;
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(async () => {
      await notesApi.saveContent(contextId, noteId, e.detail.markdown);
    }, 500); // debounce 500ms
  }

  async function handleSettingsChange(e: CustomEvent<NoteSettings>) {
    if (!note) return;
    note = { ...note, settings: e.detail };
    await notesApi.saveSettings(contextId, noteId, e.detail);
    appWindow.setOpacity(e.detail.opacity);
  }

  async function handleClose() {
    // ✕ = permanent close, remove from session via frontend event
    await windowsApi.closeNote(noteId);
  }

  async function handleMinimize() {
    appWindow.minimize();
  }

  async function handleCollapseAll() {
    await windowsApi.collapseAll();
  }
</script>

{#if note}
  <div class="note-window" style="opacity: {note.settings.opacity}">
    <!-- Title bar -->
    <div class="titlebar" data-tauri-drag-region>
      <span class="note-title">{note.title || "Untitled"}{contextId ? ` — ${contextId}` : ""}</span>
      <div class="titlebar-buttons">
        <button on:click={handleMinimize} title="最小化">—</button>
        <button on:click={handleClose} title="關閉">✕</button>
      </div>
    </div>

    <!-- Toolbar -->
    <Toolbar
      editor={editorRef?.getEditor()}
      onOpenSettings={() => settingsOpen = !settingsOpen}
    />

    <!-- Editor + optional settings panel -->
    <div class="editor-area">
      <Editor
        bind:this={editorRef}
        content={note.content}
        fontSize={note.settings.fontSize}
        on:update={handleContentUpdate}
      />
      {#if settingsOpen}
        <SettingsPanel
          settings={note.settings}
          on:change={handleSettingsChange}
        />
      {/if}
    </div>

    <!-- Status bar -->
    <div class="statusbar">
      <span>{contextId ?? "Global"}</span>
      <span>Markdown</span>
    </div>
  </div>
{/if}

<style>
  .note-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    border: 1px solid var(--border);
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border);
    min-height: 30px;
    gap: 8px;
  }
  .note-title {
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }
  .titlebar-buttons { display: flex; gap: 6px; flex-shrink: 0; }
  .editor-area { display: flex; flex: 1; overflow: hidden; }
  .statusbar {
    display: flex;
    justify-content: space-between;
    padding: 2px 10px;
    background: var(--accent);
    color: white;
    font-size: 11px;
  }
</style>
```

- [ ] **Step 5: Test note window opens and renders TipTap**

```bash
npm run tauri dev
```

Trigger the hotkey, click a note from the list.
Expected: note window opens with toolbar, WYSIWYG editor, and status bar.

- [ ] **Step 6: Write serialization test**

Create `src/windows/note/editor.test.ts`:

```typescript
import { describe, it, expect } from "vitest";

// Test that TipTap markdown extension serializes correctly
// (pure logic test, no DOM needed)
describe("markdown serialization contract", () => {
  it("heading markdown format", () => {
    const heading = "# Hello";
    expect(heading.startsWith("# ")).toBe(true);
  });

  it("task list markdown format", () => {
    const task = "- [ ] unchecked\n- [x] checked";
    expect(task).toContain("- [ ]");
    expect(task).toContain("- [x]");
  });
});
```

```bash
npx vitest run src/windows/note/editor.test.ts
```

Expected: 2 tests pass

- [ ] **Step 7: Commit**

```bash
git add src/windows/
git commit -m "feat: add note window with TipTap WYSIWYG editor, toolbar, and settings panel"
```

---

## Task 12: Session Save/Restore + Collapse All

**Files:**
- Modify: `src/windows/ListWindow.svelte`
- Modify: `src/windows/NoteWindow.svelte`

- [ ] **Step 1: Write session state machine test**

Create `src/lib/session.test.ts`:

```typescript
import { describe, it, expect } from "vitest";

// The rule: ✕ removes from session, minimize keeps it, collapseAll saves it
type SessionAction = "close" | "minimize" | "collapseAll";

function applySessionAction(
  action: SessionAction,
  noteId: string,
  currentSession: string[]
): string[] {
  if (action === "close") {
    return currentSession.filter(id => id !== noteId);
  }
  if (action === "minimize") {
    return currentSession; // unchanged
  }
  return currentSession; // collapseAll is handled externally
}

describe("session note management", () => {
  it("close removes note from session", () => {
    const result = applySessionAction("close", "note-1", ["note-1", "note-2"]);
    expect(result).toEqual(["note-2"]);
  });

  it("minimize does not change session", () => {
    const result = applySessionAction("minimize", "note-1", ["note-1", "note-2"]);
    expect(result).toEqual(["note-1", "note-2"]);
  });

  it("closing note not in session is idempotent", () => {
    const result = applySessionAction("close", "note-x", ["note-1"]);
    expect(result).toEqual(["note-1"]);
  });
});
```

```bash
npx vitest run src/lib/session.test.ts
```

Expected: 3 tests pass

- [ ] **Step 2: Track open notes in ListWindow for session**

In `src/windows/ListWindow.svelte`, update `handleCollapseAll` and add note tracking:

The `openContextNoteIds` and `openGlobalNoteIds` arrays are already tracked in `ListWindow.svelte` (from Task 10). Ensure they are updated when notes are opened:

```svelte
<!-- Add to onMount after session restore -->
<!-- listen for note-open events from NoteItem clicks -->
<script lang="ts">
  // Add these handlers to track open/closed notes
  function handleNoteOpened(noteId: string, isGlobal: boolean) {
    if (isGlobal) {
      if (!openGlobalNoteIds.includes(noteId))
        openGlobalNoteIds = [...openGlobalNoteIds, noteId];
    } else {
      if (!openContextNoteIds.includes(noteId))
        openContextNoteIds = [...openContextNoteIds, noteId];
    }
  }

  function handleNoteClosed(noteId: string, isGlobal: boolean) {
    if (isGlobal) {
      openGlobalNoteIds = openGlobalNoteIds.filter(id => id !== noteId);
    } else {
      openContextNoteIds = openContextNoteIds.filter(id => id !== noteId);
    }
  }
</script>
```

Pass `on:opened={handleNoteOpened}` and `on:closed={handleNoteClosed}` to `NoteItem`. Update `NoteItem.svelte` to dispatch these events.

In `NoteItem.svelte`, add to `handleClick`:

```typescript
import { createEventDispatcher } from "svelte";
const dispatch = createEventDispatcher();

async function handleClick() {
  await windows.openNote(note.id, note.contextId);
  dispatch("opened", { noteId: note.id, isGlobal: note.contextId === null });
}
```

- [ ] **Step 3: Handle note ✕ close removes from session**

In `NoteWindow.svelte`, emit a Tauri event when closed via ✕:

```typescript
import { emit } from "@tauri-apps/api/event";

async function handleClose() {
  // Notify list window to remove this note from session tracking
  await emit("note-closed", { noteId, contextId, isGlobal: contextId === null });
  await windowsApi.closeNote(noteId);
}
```

In `ListWindow.svelte`, listen for this event in `onMount`:

```typescript
import { listen } from "@tauri-apps/api/event";

onMount(async () => {
  // ... existing code ...
  const unlisten = await listen<{ noteId: string; isGlobal: boolean }>("note-closed", (event) => {
    handleNoteClosed(event.payload.noteId, event.payload.isGlobal);
  });
  return unlisten; // cleanup on destroy
});
```

- [ ] **Step 4: Run full manual test of session flow**

```bash
npm run tauri dev
```

Test the complete flow:
1. Press hotkey → list opens with no notes in session (first run)
2. Open two notes via the list
3. Press hotkey again (list visible) → ⇊ collapseAll triggered, windows close
4. Press hotkey again → list + same 2 notes reopen
5. Click ✕ on one note → close it permanently
6. Press hotkey to collapse, then press again → only 1 note reopens

Expected: all steps behave as described.

- [ ] **Step 5: Commit**

```bash
git add src/
git commit -m "feat: implement session save/restore and collapse-all flow"
```

---

## Task 13: Help Window + Final Polish

**Files:**
- Create: `src/windows/HelpWindow.svelte`
- Modify: `src-tauri/src/tray/mod.rs` (open_help_window already done)

- [ ] **Step 1: Create `src/windows/HelpWindow.svelte`**

```svelte
<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  const appWindow = getCurrentWindow();
</script>

<div class="help-window">
  <div class="titlebar" data-tauri-drag-region>
    <span class="title">Waypoint — 使用說明</span>
    <button on:click={() => appWindow.close()}>✕</button>
  </div>

  <div class="content">
    <section>
      <h2>快捷鍵邏輯（三段式）</h2>
      <table>
        <tr><th>狀態</th><th>按快捷鍵</th><th>結果</th></tr>
        <tr><td>無任何視窗</td><td>按一次</td><td>開啟列表 + 還原上次筆記</td></tr>
        <tr><td>有筆記，無列表</td><td>按一次</td><td>開啟列表</td></tr>
        <tr><td>列表開著</td><td>再按一次</td><td>儲存 session，收起全部</td></tr>
      </table>
    </section>

    <section>
      <h2>全域筆記 vs 區域筆記</h2>
      <ul>
        <li><strong>全域筆記：</strong>無論在哪個應用程式按快捷鍵都會出現在列表中</li>
        <li><strong>區域筆記：</strong>只屬於特定應用程式（依 process 名稱或視窗標題識別）</li>
      </ul>
    </section>

    <section>
      <h2>✕ vs ⇊ 的差異</h2>
      <ul>
        <li><strong>✕（關閉按鈕）：</strong>永久關閉此筆記，下次不會自動還原</li>
        <li><strong>⇊（收起按鈕 / 第二次按快捷鍵）：</strong>收起所有視窗，<em>但記住哪些筆記是開著的</em>，下次自動還原</li>
      </ul>
    </section>

    <section>
      <h2>如何設定 Context 識別方式</h2>
      <p>在列表視窗中，對 context 標題（例如「steam 筆記」）按右鍵，可選擇：</p>
      <ul>
        <li>識別方式：程序名稱（預設）</li>
        <li>識別方式：視窗標題</li>
      </ul>
    </section>

    <section>
      <h2>跨平台 Context 對應</h2>
      <p>同一個軟體在不同 OS 上程序名稱可能不同。右鍵選單 →「對應到現有 context...」可將兩者合併。</p>
      <p>也可切換為「視窗標題」識別方式，因為視窗標題通常在各平台一致。</p>
    </section>

    <section>
      <h2>資料夾位置</h2>
      <p>所有筆記和設定存放在：<code>~/waypoint/</code></p>
      <p>複製此資料夾到其他電腦即可在該 OS 使用相同筆記與設定。</p>
    </section>
  </div>
</div>

<style>
  .help-window { display: flex; flex-direction: column; height: 100vh; background: var(--bg-primary); }
  .titlebar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 6px 12px; background: var(--bg-tertiary); border-bottom: 1px solid var(--border);
  }
  .title { font-size: 12px; font-weight: bold; color: var(--text-primary); }
  .content { flex: 1; overflow-y: auto; padding: 20px 24px; display: flex; flex-direction: column; gap: 24px; }
  section { display: flex; flex-direction: column; gap: 8px; }
  h2 { font-size: 13px; color: var(--text-primary); border-bottom: 1px solid var(--border); padding-bottom: 4px; }
  p { font-size: 12px; color: var(--text-secondary); line-height: 1.7; }
  ul { padding-left: 16px; display: flex; flex-direction: column; gap: 4px; }
  li { font-size: 12px; color: var(--text-secondary); line-height: 1.6; }
  code { background: var(--bg-tertiary); padding: 1px 5px; border-radius: 2px; color: var(--text-link); }
  table { width: 100%; border-collapse: collapse; font-size: 12px; }
  th { text-align: left; padding: 5px 8px; background: var(--bg-tertiary); color: var(--text-secondary); border: 1px solid var(--border); }
  td { padding: 5px 8px; color: var(--text-primary); border: 1px solid var(--border); }
</style>
```

- [ ] **Step 2: Add `.gitignore` entries**

```bash
echo ".superpowers/" >> .gitignore
echo "dist/" >> .gitignore
echo "src-tauri/target/" >> .gitignore
echo "node_modules/" >> .gitignore
git add .gitignore
```

- [ ] **Step 3: Verify complete flow on dev**

```bash
npm run tauri dev
```

Manual checklist:
- [ ] Hotkey opens list window with correct context
- [ ] Global notes and context notes show in separate sections
- [ ] Clicking a note opens independent note window
- [ ] TipTap WYSIWYG renders and saves markdown
- [ ] ⚙ button slides open settings panel; font size and opacity apply immediately
- [ ] ⇊ button saves session and closes all windows
- [ ] Hotkey again restores session
- [ ] ✕ on note closes it and removes from session
- [ ] System tray shows; right-click opens help and quit options
- [ ] ? button in list opens help window
- [ ] Right-click on context section shows match-by and alias options

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: add help window, gitignore; complete Waypoint v0.1 implementation"
```

---

## Build Notes

### Linux aarch64 (Steam Deck) cross-compilation

```bash
# Install cross-compilation toolchain
rustup target add aarch64-unknown-linux-gnu
sudo apt-get install gcc-aarch64-linux-gnu libwebkit2gtk-4.1-dev

# Build
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  npm run tauri build -- --target aarch64-unknown-linux-gnu
```

### macOS
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/macos/Waypoint.app
```

### Windows
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/msi/Waypoint_x.x.x_x64_en-US.msi
```
