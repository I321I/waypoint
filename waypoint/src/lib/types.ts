export interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface NoteSettings {
  fontSize: number;
  opacity: number;
  /** @deprecated R4：已不再使用筆記專屬快捷鍵；保留欄位以相容舊 JSON 設定。 */
  hotkey: string | null;
  windowBounds: WindowBounds | null;
  passthrough: boolean;
}

export interface Note {
  id: string;
  contextId: string | null;
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
  passthroughHotkey: string;
  showInTaskbar: boolean;
}

export type ViewType = "list" | "note" | "help" | "settings";
