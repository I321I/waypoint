// 共用 readLog：跟 lib.rs 的 resolve_log_path 對齊。
// v0.2.3 起 Windows / macOS / Linux 都改用 dirs::data_local_dir 為主，
// 舊路徑保留 fallback。
// v0.2.32 起檔名改為 error-YYYY-MM-DD.log（每日 rotation），
// 因此 candidate 改為「目錄」並讀目錄裡最新的 error-*.log；
// 舊單檔 error.log 路徑也保留 fallback 以相容尚未 rotate 的舊安裝。
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

function dirCandidates() {
  const home = os.homedir();
  const xdg = process.env.XDG_STATE_HOME;
  const localAppData = process.env.LOCALAPPDATA;
  const list = [];
  if (localAppData) list.push(path.join(localAppData, "waypoint"));
  if (home && process.platform === "darwin") {
    list.push(path.join(home, "Library/Application Support/waypoint"));
  }
  if (home && process.platform === "linux") {
    list.push(path.join(home, ".local/share/waypoint"));
  }
  if (xdg) list.push(path.join(xdg, "waypoint"));
  if (home) list.push(path.join(home, ".local/state/waypoint"));
  if (home) list.push(path.join(home, "waypoint"));
  return list;
}

export function logCandidates() {
  const files = [];
  for (const dir of dirCandidates()) {
    let entries = [];
    try { entries = fs.readdirSync(dir); } catch { continue; }
    // 取所有 error-*.log（按檔名遞減 = 日期遞減 → 最新優先）
    const rotated = entries
      .filter((n) => /^error-\d{4}-\d{2}-\d{2}\.log$/.test(n))
      .sort()
      .reverse();
    for (const n of rotated) files.push(path.join(dir, n));
    // 舊單檔 fallback
    if (entries.includes("error.log")) files.push(path.join(dir, "error.log"));
  }
  return files;
}

export function readLog() {
  // 把目前候選名單裡「全部存在」的檔案內容 concat 起來，
  // 跨日 rotation 時測試在意的事件可能落在新舊兩個檔。
  let out = "";
  for (const p of logCandidates()) {
    try { out += fs.readFileSync(p, "utf8"); } catch {}
  }
  return out;
}
