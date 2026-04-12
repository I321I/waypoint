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
