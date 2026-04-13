import { describe, it, expect } from "vitest";
import type { ViewType } from "./types";

// 測試 +page.svelte 中 URL params 解析邏輯
function parseRouteParams(search: string): {
  view: ViewType;
  noteId: string | null;
  contextId: string | null;
} {
  const params = new URLSearchParams(search);
  const view = (params.get("view") ?? "list") as ViewType;
  const noteId = params.get("noteId");
  const contextId = params.get("contextId");
  return { view, noteId, contextId };
}

describe("route URL param parsing", () => {
  it("預設 view 為 list", () => {
    const { view, noteId, contextId } = parseRouteParams("");
    expect(view).toBe("list");
    expect(noteId).toBeNull();
    expect(contextId).toBeNull();
  });

  it("?view=help 解析為 help", () => {
    const { view } = parseRouteParams("?view=help");
    expect(view).toBe("help");
  });

  it("?view=settings 解析為 settings", () => {
    const { view } = parseRouteParams("?view=settings");
    expect(view).toBe("settings");
  });

  it("?view=note&noteId=abc 解析正確", () => {
    const { view, noteId, contextId } = parseRouteParams(
      "?view=note&noteId=abc"
    );
    expect(view).toBe("note");
    expect(noteId).toBe("abc");
    expect(contextId).toBeNull();
  });

  it("?view=note&noteId=abc&contextId=game 解析正確", () => {
    const { view, noteId, contextId } = parseRouteParams(
      "?view=note&noteId=abc&contextId=game"
    );
    expect(view).toBe("note");
    expect(noteId).toBe("abc");
    expect(contextId).toBe("game");
  });

  it("全域筆記 contextId 為 null", () => {
    const { contextId } = parseRouteParams("?view=note&noteId=xyz");
    expect(contextId).toBeNull();
  });
});
