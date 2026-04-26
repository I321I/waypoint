// 筆記內容 <-> (標題, 正文) 的轉換。
//
// 儲存格式（延續舊檔案相容性）：
//   # {title}\n{body}
//
// 若第一行不是 "# " 前綴，視為沒有標題（title="", body=全部內容）。
// 空標題序列化時不寫 "# " 以免首行被誤認為 heading。

export interface TitleContent {
  title: string;
  body: string;
}

export function parseTitleContent(raw: string): TitleContent {
  if (!raw) return { title: "", body: "" };
  const nl = raw.indexOf("\n");
  const firstLine = nl === -1 ? raw : raw.slice(0, nl);
  const rest = nl === -1 ? "" : raw.slice(nl + 1);
  if (firstLine.startsWith("# ")) {
    return { title: firstLine.slice(2).trim(), body: rest };
  }
  return { title: "", body: raw };
}

export function joinTitleContent(title: string, body: string): string {
  const t = title.trim();
  if (!t) return body;
  return `# ${t}\n${body}`;
}
