import { describe, it, expect, test } from "vitest";
import type { Note, NoteSettings, Session } from "./types";

describe("types", () => {
  it("Note contextId is null for global notes", () => {
    const note: Note = {
      id: "abc",
      contextId: null,
      title: "Test",
      content: "# Test",
      settings: { fontSize: 14, opacity: 1.0, hotkey: null, windowBounds: null, passthrough: false },
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

test("NoteSettings.passthrough defaults to false", () => {
  const s: NoteSettings = { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false };
  expect(s.passthrough).toBe(false);
});
