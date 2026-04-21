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
  getAutostart: () => invoke<boolean>("get_autostart"),
  isAutostartSupported: () => invoke<boolean>("is_autostart_supported"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
};

export const windows = {
  openNote: (noteId: string, contextId: string | null) =>
    invoke<void>("cmd_open_note_window", { noteId, contextId }),
  collapseAll: () => invoke<void>("cmd_collapse_all"),
  closeNote: (noteId: string) =>
    invoke<void>("cmd_close_note_window", { noteId }),
  openSettings: () => invoke<void>("cmd_open_settings"),
  /** 用 label 關閉視窗（不依賴 getCurrentWindow()）*/
  closeWindow: (label: string) => invoke<void>("cmd_close_window", { label }),
  /** 用 label 隱藏視窗（不依賴 getCurrentWindow()）*/
  hideWindow: (label: string) => invoke<void>("cmd_hide_window", { label }),
  /** 最小化指定 label 視窗 */
  minimizeWindow: (label: string) => invoke<void>("cmd_minimize_window", { label }),
  /** 切換最大化狀態 */
  toggleMaximize: (label: string) => invoke<void>("cmd_toggle_maximize", { label }),
  /** 完全結束 Waypoint */
  exitApp: () => invoke<void>("cmd_exit_app"),
};
