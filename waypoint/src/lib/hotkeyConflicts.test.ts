import { describe, it, expect } from "vitest";
import { findConflict } from "./hotkeyConflicts";

describe("findConflict", () => {
  it("returns conflict info for Ctrl+Shift+T", () => {
    const c = findConflict("Ctrl+Shift+T");
    expect(c).not.toBeNull();
    expect(c?.app).toContain("瀏覽器");
  });
  it("returns null for safe combo", () => {
    expect(findConflict("Ctrl+Alt+T")).toBeNull();
  });
  it("is case-insensitive on key", () => {
    expect(findConflict("ctrl+shift+t")).not.toBeNull();
  });
});
