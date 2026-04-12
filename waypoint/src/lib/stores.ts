import { writable } from "svelte/store";
import type { Note } from "./types";

export const activeContextId = writable<string | null>(null);
export const globalNotes = writable<Note[]>([]);
export const contextNotes = writable<Note[]>([]);
export const settingsPanelOpen = writable<boolean>(false);
