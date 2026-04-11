# Waypoint — 設計規格文件

**日期**：2026-04-11  
**版本**：1.0

---

## 1. 專案概覽

Waypoint 是一款跨平台浮動筆記軟體，主要用於遊戲情境，但適用於所有應用程式視窗。核心理念：在任何軟體上按快捷鍵，立即呼叫與該軟體相關的筆記。

### 目標平台
- Linux (aarch64，主要目標：Steam Deck / SteamOS + KDE Wayland，透過 XWayland 相容層運作)
- macOS
- Windows

### 技術棧
- **後端**：Tauri 2.0（Rust）
- **前端**：Svelte
- **編輯器**：TipTap（WYSIWYG，底層儲存為 Markdown）
- **系統整合**：Tauri 全域快捷鍵 API + 各平台原生視窗焦點 API

---

## 2. 資料儲存

### 預設路徑
```
~/waypoint/
```

### 目錄結構
```
~/waypoint/
├── global/
│   └── {note-id}/
│       ├── content.md        ← 筆記內容（Markdown 格式）
│       └── settings.json     ← 此筆記的獨立設定
├── contexts/
│   └── {context-id}/         ← 正規化後的 process/title 名稱
│       ├── session.json      ← 此 context 上次開著的筆記清單
│       └── {note-id}/
│           ├── content.md
│           └── settings.json
└── app.json                  ← 全域設定（快捷鍵、context 對應關係等）
```

### 可攜性
- 資料夾內只有純文字（`.md`、`.json`），無二進位資料
- 直接複製 `~/waypoint/` 到另一台電腦即可在該 OS 使用相同設定與筆記
- 軟體本身透過各平台正常方式安裝，獨立於資料夾之外

### app.json 結構
```json
{
  "hotkey": "Ctrl+Shift+Space",
  "contextAliases": {
    "mygame_win": "mygame"
  },
  "contexts": {
    "steam": { "matchBy": "process" },
    "somegame": { "matchBy": "title" }
  }
}
```

### 筆記設定 settings.json 結構
```json
{
  "fontSize": 14,
  "opacity": 1.0,
  "hotkey": null,
  "windowBounds": { "x": 100, "y": 100, "width": 400, "height": 600 }
}
```

---

## 3. Context 識別系統

### 筆記分類
- **全域筆記**：不管在哪個應用程式按快捷鍵都會顯示
- **區域筆記（context 筆記）**：只屬於特定應用程式視窗

### Context 識別方式（每個 context 可獨立設定）
- **預設**：Process name（程序名稱）
- **可切換為**：Window title（視窗標題）

### 跨平台自動正規化（第一層防護）
Process name 偵測到後自動處理：
1. 去掉 `.exe` 副檔名（Windows）
2. 全部轉為小寫

| 偵測到的原始名稱 | 正規化後 | 資料夾 |
|----------------|---------|--------|
| `steam.exe` | `steam` | `contexts/steam/` |
| `Steam` | `steam` | `contexts/steam/` |
| `steam` | `steam` | `contexts/steam/` |

### 手動 Context 對應（第二層防護）
當同一軟體在不同 OS 上 process name 完全不同時，透過 UI 操作建立對應：
1. 在列表視窗對 context 標題**右鍵** → 「對應到現有 context...」
2. 選擇目標 context
3. 對應關係寫入 `app.json` 的 `contextAliases`

### 平台識別 API
| 平台 | 取得當前視窗 API |
|------|----------------|
| Windows | `GetForegroundWindow()` + `GetWindowThreadProcessId()` |
| macOS | `NSWorkspace.shared.frontmostApplication` |
| Linux (XWayland) | `XGetInputFocus()` + `/proc/{pid}/comm` |

---

## 4. 視窗系統

### 所有視窗完全獨立
每個視窗是獨立的 OS 視窗，可自由移動、調整大小、最小化。

### 視窗類型

#### 4.1 背景程序（System Tray）
- Waypoint 啟動後常駐系統托盤
- 監聽全域快捷鍵
- 右鍵選單：
  - 使用說明
  - 設定
  - 結束 Waypoint

#### 4.2 筆記列表視窗（List Window）
- 快捷鍵觸發時開啟
- 切換至非 Waypoint 視窗時**自動關閉**（不影響 session）
- 無最小化按鈕

**標題列佈局：**
```
[ WAYPOINT  ? ]              [ ⇊  ✕ ]
```
- 左側：軟體名稱 + `?`（使用說明）
- 右側：`⇊`（收起全部）、`✕`（關閉列表）

**列表內容（分區顯示）：**
```
🌐 全域筆記                    [+]
  📄 每日待辦
  📄 重要聯絡人

────────────────────
Steam 筆記           [⚙ 右鍵]  [+]
  📄 攻略備忘
  📄 成就清單
```

**Context 區塊右鍵選單：**
```
識別方式：程序名稱 ✓
識別方式：視窗標題
─────────────────
對應到現有 context...
重新命名
─────────────────
刪除此 context
```

#### 4.3 筆記視窗（Note Window，每個筆記獨立）
- 點擊列表中的筆記開啟，或透過 session 還原自動開啟
- 切換至其他視窗時**保持開啟**（不自動關閉）
- 記憶視窗位置、大小、所有設定

**視窗結構：**
```
┌─ 筆記標題 — Context ────────────[ — ][ ✕ ]─┐
├─ B  I  U  |  H1  H2  |  ≡  ☑  ─────[ ⚙ ]─┤
│                                             │
│   TipTap WYSIWYG 編輯區                     │
│                                             │
├─ Context 名稱 ──────────────── Markdown ────┤
└─────────────────────────────────────────────┘
```

**按鈕行為：**
| 按鈕 | 行為 |
|------|------|
| `—` | 最小化至工作列，session 保留 |
| `✕` | 永久關閉，從 session 移除 |
| `⚙` | 開關右側設定面板 |

**右側設定面板（滑出）：**
- 字體大小（數字輸入或 +/- 按鈕）
- 視窗透明度（滑桿 0%–100%）
- 此筆記專屬快捷鍵（留空則不設定）：按下後**直接開啟這個筆記**，繞過全域快捷鍵的三段邏輯，適合最常用的筆記
- 設定即時生效並自動儲存至 `settings.json`

---

## 5. 快捷鍵與 Session 邏輯

### 快捷鍵三段式行為

| 當前狀態 | 按快捷鍵 | 結果 |
|---------|---------|------|
| 無任何 Waypoint 視窗 | 按一次 | 開啟列表 + 還原此 context 的 session 筆記 |
| 有筆記開著，列表已關閉 | 按一次 | 只開啟列表，筆記不動 |
| 列表開著 | 再按一次 | 收起全部（⇊），儲存 session |

### ⇊ 收起全部（按鈕或快捷鍵）
- 儲存目前所有開著的筆記視窗為 session
- 關閉所有 Waypoint 視窗
- 下次按快捷鍵時自動還原

### Session 範圍
每個 context 的 session **同時記錄**當時開著的全域筆記與該 context 的區域筆記。
按快捷鍵還原時，兩者一起恢復。

### Session 格式
```json
// ~/waypoint/contexts/steam/session.json
{
  "openContextNotes": ["note-abc123", "note-def456"],
  "openGlobalNotes": ["note-ghi789"]
}
```

同一個全域筆記可以出現在多個 context 的 session 裡，這是正常的。

### 快捷鍵偵測時序
1. 使用者在非 Waypoint 視窗按下快捷鍵
2. Rust backend 攔截快捷鍵
3. **在切換焦點前**，立即讀取當前焦點視窗（process name / title）
4. 正規化 context ID，查詢 `contextAliases`
5. 依三段式邏輯決定開啟或收起

---

## 6. 編輯器（TipTap WYSIWYG）

- 所見即所得：打字時直接顯示格式（粗體即粗體、標題即大字）
- 底層儲存為標準 Markdown（`.md`），人類可讀
- 支援功能：
  - 標題 H1–H3
  - 粗體、斜體、底線、刪除線
  - 有序清單、無序清單、任務清單（checkbox）
  - 程式碼區塊（帶語法高亮）
  - 表格
  - 圖片插入（儲存至筆記資料夾的 `assets/` 子目錄）
- 工具列固定於編輯區頂部

---

## 7. UI 風格

- 參考 VSCode 深色主題
- 色彩：深灰背景（`#1e1e1e`）、中灰面板（`#252526`）、藍色強調（`#007acc`）
- 字型：系統預設等寬字型（編輯器）、系統 UI 字型（介面）
- 邊角：直角或極小圓角（2–3px）
- 不使用大圓角設計

---

## 8. 測試策略

由於要求「修正錯誤時同步寫 test 確保不再發生」，測試分兩層：

### 後端（Rust）單元測試
- Context 正規化邏輯（process name → normalized id）
- Context alias 解析
- Session 讀寫
- 檔案系統操作（note CRUD）

### 前端（Svelte）元件測試（Vitest + Testing Library）
- 列表視窗渲染（全域 / context 分區）
- 快捷鍵三段邏輯狀態機
- 設定面板開關
- TipTap 內容序列化（WYSIWYG → Markdown）

### 整合測試（Tauri test harness）
- 快捷鍵觸發 → 視窗開關行為
- Context 偵測 → 正確筆記顯示
- Session 儲存與還原

---

## 9. 使用說明（? 按鈕內容）

說明視窗需涵蓋：
1. 快捷鍵三段式邏輯圖解
2. 全域筆記 vs 區域筆記的差異
3. ✕ vs ⇊ 的差異（永久關閉 vs 收起保留 session）
4. 如何設定 context 識別方式
5. 如何跨平台對應 context
6. 資料夾位置與可攜性說明
