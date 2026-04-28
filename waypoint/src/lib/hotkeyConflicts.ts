export interface HotkeyConflict {
  combo: string;
  app: string;
  description: string;
}

const CONFLICTS: HotkeyConflict[] = [
  { combo: "Ctrl+Shift+T", app: "瀏覽器（Edge/Chrome/Firefox）", description: "重新開啟最後關閉的分頁" },
  { combo: "Ctrl+Shift+N", app: "瀏覽器", description: "新無痕視窗" },
  { combo: "Ctrl+Shift+W", app: "瀏覽器", description: "關閉所有分頁" },
  { combo: "Ctrl+Alt+Del", app: "Windows 系統", description: "保留組合鍵" },
  { combo: "Win+L", app: "Windows 系統", description: "鎖定螢幕" },
  { combo: "Ctrl+Esc", app: "Windows 系統", description: "開始功能表" },
];

export function findConflict(combo: string): HotkeyConflict | null {
  const norm = combo.toLowerCase();
  return CONFLICTS.find(c => c.combo.toLowerCase() === norm) ?? null;
}
