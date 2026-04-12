import { describe, it, expect } from "vitest";

type SessionAction = "close" | "minimize" | "collapseAll";

function applySessionAction(
  action: SessionAction,
  noteId: string,
  currentSession: string[]
): string[] {
  if (action === "close") {
    return currentSession.filter(id => id !== noteId);
  }
  return currentSession;
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
