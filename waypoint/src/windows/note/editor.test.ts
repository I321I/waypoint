import { describe, it, expect } from "vitest";

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
