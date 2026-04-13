# 專案開發規範

## 每階段實作後自動 Commit

完成任何一個實作階段後，**必須立即執行 git commit**，不需等待用戶要求。

### 何時觸發 Commit

以下情況視為「一個階段完成」，應立即 commit：

- 完成一個功能模組的實作（新增檔案、修改核心邏輯）
- 完成一個 Task 或子任務
- 修復一個 bug 並驗證完成
- 完成重構的一個獨立步驟
- 完成設計文件或規格撰寫

### Commit 流程

每次 commit 按以下步驟執行：

1. `git add` 相關檔案（只加本次修改的檔案，不用 `git add -A`）
2. `git commit -m "..."` 使用語意化 commit 訊息

### Commit 訊息格式

使用以下前綴：

| 前綴 | 使用時機 |
|------|----------|
| `feat:` | 新增功能 |
| `fix:` | 修復 bug |
| `refactor:` | 重構（不改變行為） |
| `docs:` | 文件、規格 |
| `chore:` | 設定、工具、雜項 |

範例：
```
feat: 實作 Waypoint 地點資料結構與基本操作
fix: 修正地點搜尋結果排序邏輯
docs: 新增 Waypoint API 設計規格
```

### 注意事項

- 每次 commit 只包含本階段相關的檔案，範圍要精確
- Commit 訊息要說明「做了什麼」，簡潔清楚
- 不需要等用戶說「幫我 commit」，階段完成即自動執行
- 如果有多個獨立的修改，分開 commit，不要打包成一個

## 每次修正後必須建立並執行 Test

**規則：任何 bug fix 或功能實作完成後，必須建立 test 並確認通過，才算完成。**

### 前端 Test（Vitest）

```bash
cd /data/games-note-AIgen/waypoint && npm test
```

- 測試檔放在 `src/**/*.test.ts`
- 針對修改的邏輯（函式、狀態、URL 解析等）撰寫單元測試
- 如果修改的是純 UI 元件（無可測邏輯），記錄原因並跳過

### 後端 Test（Rust）

```bash
cd /data/games-note-AIgen/waypoint/src-tauri && cargo test
```

- 針對修改的 Rust 函式撰寫 `#[test]`
- 放在對應模組的 `#[cfg(test)] mod tests {}` 區塊內

### 流程

1. 實作修正
2. 撰寫 test（能測什麼就測什麼）
3. 執行 test，確認全部通過
4. 執行 git commit（帶 test 檔案一起 commit）
