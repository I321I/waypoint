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
}

export type ViewType = "list" | "note" | "help" | "settings";
