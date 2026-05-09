// 共用 readLog：跟 lib.rs 的 resolve_log_path 對齊。
// v0.2.3 起 Windows / macOS / Linux 都改用 dirs::data_local_dir 為主，
// 舊路徑保留 fallback。
import fs from "node:fs";
import path from "node:path";
import os from "node:os";

export function logCandidates() {
  const home = os.homedir();
  const xdg = process.env.XDG_STATE_HOME;
  const localAppData = process.env.LOCALAPPDATA; // Windows
  const list = [];
  // dirs::data_local_dir() 等價路徑
  if (localAppData) list.push(path.join(localAppData, "waypoint", "error.log")); // Windows
  if (home && process.platform === "darwin") {
    list.push(path.join(home, "Library/Application Support/waypoint/error.log"));
  }
  if (home && process.platform === "linux") {
    list.push(path.join(home, ".local/share/waypoint/error.log"));
  }
  // 舊版 / fallback
  if (xdg) list.push(path.join(xdg, "waypoint", "error.log"));
  if (home) list.push(path.join(home, ".local/state/waypoint/error.log"));
  if (home) list.push(path.join(home, "waypoint", "error.log"));
  return list;
}

export function readLog() {
  for (const p of logCandidates()) {
    try { return fs.readFileSync(p, "utf8"); } catch {}
  }
  return "";
}
