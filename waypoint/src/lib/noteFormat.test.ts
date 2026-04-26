import { describe, it, expect } from "vitest";
import { parseTitleContent, joinTitleContent } from "./noteFormat";

describe("noteFormat.parseTitleContent", () => {
  it("parses '# title\\nbody'", () => {
    const r = parseTitleContent("# Hello\nworld\nline2");
    expect(r.title).toBe("Hello");
    expect(r.body).toBe("world\nline2");
  });

  it("treats content without '# ' prefix as body only", () => {
    const r = parseTitleContent("just text");
    expect(r.title).toBe("");
    expect(r.body).toBe("just text");
  });

  it("handles empty string", () => {
    const r = parseTitleContent("");
    expect(r.title).toBe("");
    expect(r.body).toBe("");
  });

  it("handles only heading line", () => {
    const r = parseTitleContent("# Only");
    expect(r.title).toBe("Only");
    expect(r.body).toBe("");
  });
});

describe("noteFormat.joinTitleContent", () => {
  it("joins title and body with '# ' prefix", () => {
    expect(joinTitleContent("Hello", "body")).toBe("# Hello\nbody");
  });

  it("omits heading when title empty", () => {
    expect(joinTitleContent("", "body")).toBe("body");
    expect(joinTitleContent("   ", "body")).toBe("body");
  });

  it("round-trips", () => {
    const src = "# My Note\nsome content\nand more";
    const parsed = parseTitleContent(src);
    expect(joinTitleContent(parsed.title, parsed.body)).toBe(src);
  });
});
