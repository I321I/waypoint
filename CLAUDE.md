# 專案開發規範

## Git 工作分支策略

**規則：日常開發在 `dev/main`（或其他 `dev/*`）分支進行，不要每個 commit 都推 master。**

原因：`e2e-windows.yml` 的 trigger 是 `push: branches:[master]` + `pull_request`。每推 master 就會跑 Windows runner（耗時且消耗 CI 額度）。

流程：
1. 平常 commit / push 到 `dev/main`（不開 PR）→ **不會**觸發 e2e-windows
2. 階段完成、想驗 Windows 行為時 → `git checkout master && git merge --ff-only dev/main && git push origin master` → 觸發 e2e-windows
3. 純文件 / CI 設定 commit 若不影響邏輯，可用 `[skip ci]` 訊息（即使在 master 也不跑）
4. 發 tag 前：master 末梢 e2e-windows 綠 + 本機 `act -j e2e-linux` 綠 → `git tag vX.Y.Z && git push --tags`

注意：tag 也要從 master 上打（release.yml 假設 release 內容在 master）。

⚠️ tag 落點的 commit message **不可含 `[skip ci]`**：tag push 觸發的 release.yml 也會被同一個 commit 上的 `[skip ci]` 標記擋掉。若 master 末梢剛好是 `[skip ci]` 的純文件 / CI commit，請先加一個 `git commit --allow-empty -m "chore: trigger release for vX.Y.Z"` 再打 tag。

## Release 內容說明 + README 同步

**規則：每次 `git tag -a vX.Y.Z -m "..."` 都要在 message 中列出本版改動的「項目」，但不要解釋實作細節。**

原因：
- release.yml 已設定為自動把 tag annotation 抽出當作 GitHub release notes 的「本版修正」區塊（`Build release notes from tag annotation` step）
- 使用者需要看的是「這版修了什麼 / 加了什麼」，不是「如何修的」（後者用 commit log 與 PR 即可）

格式：bullet list，動詞開頭，描述功能變化或修正項目。**不寫**內部實作（不講「重構某 module」「改用 v3 API」之類）。

範例（好）：
```
- 修正開啟筆記時 markdown 沒 render 的 bug（# Heading、**bold** 不生效）
- 列表筆記刪除確認改成 in-app 對話框（不再是 OS confirm 爆寬）
- 穿透模式預設快捷鍵改為 Ctrl+Shift+Q
- 新增視窗透明度滑桿移到 titlebar
```

範例（不好）：
```
- 修改 Editor.svelte 的 setContent 呼叫加上 contentType:'markdown'  ← 太實作層
- refactor NoteWindow 的 flushPendingSave 路徑                       ← 對使用者無意義
```

**附加規則：每次發 release 前，依本版改動項目檢查 `README.md` 是否需要更新（功能列表、預設快捷鍵、安裝指引、操作步驟）。** 若有 user-facing 變化沒反映在 README，先補 README 再打 tag。

## Docker / act 環境變數

**規則：本機（WSL2）執行任何 `docker` 或 `act` 指令前，必須先設定 `DOCKER_HOST`，連到 Windows 端 Docker Desktop 的 TCP daemon。**

```bash
export DOCKER_HOST=tcp://host.docker.internal:2375
```

原因：WSL2 內沒有本地 docker daemon，socket `/var/run/docker.sock` 不存在；Docker Desktop 在 Windows 端透過 TCP 2375 暴露。

用法：

- 單次：`export DOCKER_HOST=tcp://host.docker.internal:2375 && docker ps`
- 跑 act：`export DOCKER_HOST=tcp://host.docker.internal:2375 && act -j e2e-linux`
- 或在 shell session 開頭一次設定，後續指令直接用

## CI 分工：act vs GitHub 真 runner

act 只能跑 Linux container，Windows/macOS runner 與 ARM 矩陣必須交給 GitHub 真 runner。

| Workflow / Job | 本機 act | GitHub 真 runner |
|---|---|---|
| `e2e-linux.yml`（ubuntu-22.04 + WebKitGTK） | ✅ `act -j e2e-linux` | ✅ |
| `e2e-windows.yml`（windows-latest + WebView2） | ❌ 跑不了 | ✅ 必跑 |
| `release.yml` matrix `ubuntu-22.04` | ✅ `act -j release --matrix platform:ubuntu-22.04` | ✅ |
| `release.yml` matrix `ubuntu-22.04-arm` | ❌（QEMU 慢且常卡） | ✅ |
| `release.yml` matrix `windows-latest` | ❌ | ✅ 必跑 |
| `release.yml` matrix `macos-latest` | ❌ | ✅ 必跑 |

原則：本機開發迭代用 act 快速驗證 Linux 那條；發 release tag 前除了 GitHub 全平台必須綠燈，**本機 act 也必須跑一次並確認成功**（避免 GitHub 綠燈但 act 環境壞掉的盲點，act 失敗即不可發 tag）。

## 每次對話必須建立並顯示任務列表

**規則：當使用者要求執行任務（修正、新增功能、重構等）時，必須：**

1. **開始前**：將任務拆解為列表，使用 TaskCreate 建立追蹤。
2. **執行中**：每完成或切換一項，立即 TaskUpdate 狀態（`pending` / `in_progress` / `completed`），讓任務面板即時反映進度。
3. **完成所有任務後**：在回覆最後以單一列表呈現所有任務，**每項前面加符號表示狀態**：
   - `⬜` 未完成
   - `🔄` 進行中
   - `✅` 已完成（後面簡述處理方式：做了什麼、怎麼做的、結果）

### 範例格式

```
## 任務摘要

- ✅ 修正白屏問題 — 改用 URL hash 路由，在 app.html 設 body 深色背景防閃爍
- ✅ 新增 Playwright 渲染測試 — 針對三個視窗建立 14 個按鈕/標題測試，全部通過
- 🔄 優化快捷鍵回應速度
- ⬜ 加入暗色模式切換
```

### 注意事項

- 即使是單一任務也要建立列表，維持一致性
- 任務描述要具體（「修 bug」太籠統，「修正筆記視窗白屏」才夠精確）
- 只有在使用者要求「做事」時觸發；純問答、諮詢不需要建任務列表

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

## 每次 UI 修正後必須執行渲染測試（Playwright）

**規則：凡是修改視窗渲染、路由、CSS、元件顯示相關的 bug 或功能，必須執行渲染測試確認通過，才能發布。**

這是因為 Vitest/cargo test 無法測試真實瀏覽器渲染行為。白屏、元件未掛載、樣式未套用等問題只有渲染測試才能提早發現。

### 渲染 Test（Playwright）

```bash
# 先建置前端
cd /data/games-note-AIgen/waypoint && npm run build

# 再執行渲染測試（會自動啟動 preview server）
cd /data/games-note-AIgen/waypoint && npm run test:render
```

- 測試檔放在 `src/**/*.render.test.pw.ts`
- 測試覆蓋：各視窗 hash routing 正確渲染、body 不是白色、元件關鍵文字出現
- 每次修改視窗元件、路由邏輯、CSP 或 app.html 時必須重跑

### 完整測試流程（涵蓋邏輯 + 渲染）

1. 實作修正
2. 撰寫 Vitest/Rust 單元測試（邏輯層）
3. `npm test` + `cargo test`，確認全部通過
4. `npm run build && npm run test:render`，確認渲染正常
5. 執行 git commit（帶所有 test 檔案一起 commit）
6. 修正好就發布

## 每次發 release 前必須 E2E 綠燈（Windows WebView2）

**規則：任何修改推上 master 後，`.github/workflows/e2e-windows.yml` 必須綠燈才能發 release tag。**

### E2E Test（WebdriverIO + tauri-driver）

- 位置：`waypoint/e2e/`（`wdio.conf.js` + `specs/*.spec.js`）
- 跑法（本機 Windows）：
  ```powershell
  cd waypoint
  cargo install tauri-driver --locked
  npm install
  npx tauri build --no-bundle
  $env:WAYPOINT_BINARY = "$PWD\src-tauri\target\release\waypoint.exe"
  npm run test:e2e
  ```
- CI：push master / PR 自動觸發 `e2e-windows` job，runner windows-latest

### 觸發條件

- 修改 UI、路由、CSS、視窗、Rust tauri 指令 — 必須確保 E2E 綠燈後才 tag release
- 修改 Rust 啟動流程（setup、tray、hotkey）— 同樣 E2E 必綠
- 純文件修改、CI/workflow 微調 — 仍會觸發，但關注點是主邏輯

### 確認 Windows E2E 狀態（agent 可直接執行，不依賴 gh CLI）

repo `I321I/waypoint` 是 public，走公開 GitHub REST API，不需 token、不需 `gh auth login`。

**workflow id**：`262677203`（對應 `.github/workflows/e2e-windows.yml`，一次確定就不會變）。

**查最新 master run**：

```bash
curl -s "https://api.github.com/repos/I321I/waypoint/actions/workflows/262677203/runs?branch=master&per_page=1" \
  | python3 -c "import json,sys; r=json.load(sys.stdin)['workflow_runs'][0]; print(f\"id={r['id']} sha={r['head_sha'][:7]} status={r['status']} conclusion={r['conclusion']} url={r['html_url']}\")"
```

**判讀**：
- `status=queued` / `in_progress` → 還在跑，輪詢等待。
- `status=completed conclusion=success` → ✅ 綠燈，可 tag release。
- `status=completed conclusion=failure` / `cancelled` / `timed_out` → ❌ 紅燈，不可 tag。
- `sha` 必須等於 `git rev-parse master | cut -c1-7`，否則你看到的是更早的 run。

**輪詢指令**（blocking，直到 conclusion 出來才退出）：

```bash
RUN_ID=$(curl -s "https://api.github.com/repos/I321I/waypoint/actions/workflows/262677203/runs?branch=master&per_page=1" \
  | python3 -c "import json,sys;print(json.load(sys.stdin)['workflow_runs'][0]['id'])")
while true; do
  result=$(curl -s "https://api.github.com/repos/I321I/waypoint/actions/runs/$RUN_ID" \
    | python3 -c "import json,sys;r=json.load(sys.stdin);print(r.get('status'),r.get('conclusion'))")
  echo "$result"
  [[ "$result" != *"None"* && "$result" != in_progress* && "$result" != queued* ]] && break
  sleep 45
done
```

`e2e-linux` 同法，workflow id `262754229`。

### 失敗時

- CI 紅燈 → 立刻看 log。本 repo 有 fail-safe 機制：`e2e-windows.yml` 失敗時會把 runner log **push 到 `ci-logs` 分支**（commit 85e5a39），可直接：
  ```bash
  git fetch origin ci-logs && git log origin/ci-logs -1 --stat
  ```
  也可抓 artifact `wdio-logs`（需 token，麻煩，優先用 ci-logs 分支）。
- 定位失敗原因，補修後再推
- 不要用「skip E2E」或「暫時關掉」來繞過

### Linux / Steam Deck 代理測試

- `.github/workflows/e2e-linux.yml`：ubuntu-22.04 + `webkit2gtk-driver` + `xvfb`
- 跟 Windows 用同一份 wdio 設定與測試檔（tauri-driver 抽象 OS 差異）
- 作為 Steam Deck 行為代理（同為 WebKitGTK 引擎）

### 已知盲點

- 白屏檢測：若 `app.html` 的 inline `body style` 還是深色，即使 CSS 壞了測試也不紅。
  詳見 `docs/superpowers/plans/2026-04-18-windows-e2e-webview2.md` 與驗證紀錄。
