# Steam Deck Flatpak 支援 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 GitHub Release 中新增 aarch64 `.flatpak` bundle，讓 Steam Deck 用戶可直接安裝 Waypoint。

**Architecture:** 在現有 `ubuntu-22.04-arm` CI job 尾端新增打包步驟：tauri-action 建出 binary 後，以 flatpak-builder 依 manifest 打包成 `.flatpak` bundle，再上傳到 GitHub Release。不修改 app 程式碼。

**Tech Stack:** Flatpak / flatpak-builder、org.freedesktop.Platform 23.08、GitHub Actions（softprops/action-gh-release@v2）

---

## 檔案清單

| 動作 | 路徑 | 說明 |
|------|------|------|
| 新增 | `waypoint/flatpak/io.github.i321i.waypoint.yml` | Flatpak manifest |
| 修改 | `.github/workflows/release.yml` | 新增三個 step + 更新 releaseBody |

---

### Task 1：新增 Flatpak manifest

**Files:**
- Create: `waypoint/flatpak/io.github.i321i.waypoint.yml`

- [ ] **Step 1：建立目錄**

```bash
mkdir -p waypoint/flatpak
```

- [ ] **Step 2：建立 manifest 檔案**

建立 `waypoint/flatpak/io.github.i321i.waypoint.yml`，內容如下：

```yaml
app-id: io.github.i321i.waypoint
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: waypoint

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --filesystem=~/waypoint
  - --talk-name=org.kde.StatusNotifierWatcher

modules:
  - name: waypoint
    buildsystem: simple
    build-commands:
      - install -Dm755 waypoint /app/bin/waypoint
    sources:
      - type: file
        path: waypoint
```

> 注意：`path: waypoint` 是相對於 flatpak-builder 執行時的工作目錄（repo root），binary 會在 CI 中複製到此位置（見 Task 2）。

- [ ] **Step 3：Commit**

```bash
git add waypoint/flatpak/io.github.i321i.waypoint.yml
git commit -m "feat: 新增 Flatpak manifest for Steam Deck"
```

---

### Task 2：修改 CI workflow — 新增 Flatpak 建置步驟

**Files:**
- Modify: `.github/workflows/release.yml`

目前 workflow 最後一個步驟是 `Build Tauri`（使用 `tauri-apps/tauri-action@v0`）。在其**之後**新增以下三個步驟。

- [ ] **Step 1：確認 tauri-action 產出的 binary 路徑**

tauri-action 在 `ubuntu-22.04-arm` 上使用 `--target aarch64-unknown-linux-gnu`，binary 輸出至：
```
waypoint/src-tauri/target/aarch64-unknown-linux-gnu/release/waypoint
```

- [ ] **Step 2：在 `Build Tauri` 步驟之後新增「Install flatpak-builder」步驟**

在 `.github/workflows/release.yml` 的 `Build Tauri` uses 區塊後面（與它同縮排層級）加入：

```yaml
      - name: Install flatpak-builder
        if: matrix.platform == 'ubuntu-22.04-arm'
        run: |
          sudo apt-get install -y flatpak flatpak-builder
          flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
          flatpak install --user -y flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08
```

- [ ] **Step 3：新增「Build Flatpak」步驟**

緊接在上一步之後加入：

```yaml
      - name: Build Flatpak
        if: matrix.platform == 'ubuntu-22.04-arm'
        run: |
          cp waypoint/src-tauri/target/aarch64-unknown-linux-gnu/release/waypoint waypoint
          flatpak-builder --repo=flatpak-repo --force-clean flatpak-build \
            waypoint/flatpak/io.github.i321i.waypoint.yml
          flatpak build-bundle flatpak-repo \
            io.github.i321i.waypoint_aarch64.flatpak \
            io.github.i321i.waypoint --arch=aarch64
```

> 說明：先把 binary 複製到 repo root 命名為 `waypoint`，對應 manifest 中的 `path: waypoint`。

- [ ] **Step 4：新增「Upload Flatpak to Release」步驟**

緊接在上一步之後加入：

```yaml
      - name: Upload Flatpak to Release
        if: matrix.platform == 'ubuntu-22.04-arm'
        uses: softprops/action-gh-release@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          files: io.github.i321i.waypoint_aarch64.flatpak
```

- [ ] **Step 5：更新 releaseBody**

找到 `releaseBody:` 區塊，將 Steam Deck 那一行改為：

```yaml
          releaseBody: |
            ## 安裝方式
            - **Windows**：下載 `.msi` 檔案安裝
            - **Steam Deck（Flatpak，推薦）**：下載 `_aarch64.flatpak`，在 Desktop Mode 執行 `flatpak install --user io.github.i321i.waypoint_aarch64.flatpak`
            - **Steam Deck / Linux（AppImage）**：下載 `_aarch64.AppImage`，賦予執行權限後執行
            - **Linux (x86_64)**：下載 `_amd64.AppImage`，賦予執行權限後執行
            - **macOS**：下載 `.dmg` 檔案安裝
```

- [ ] **Step 6：Commit**

```bash
git add .github/workflows/release.yml
git commit -m "feat: 在 CI 新增 aarch64 Flatpak 建置與上傳步驟"
```

---

### Task 3：驗證

由於 Flatpak 建置只在推送 tag 時觸發，本地無法直接測試完整流程。以下是驗證方式：

- [ ] **Step 1：Lint 檢查 workflow 語法**

```bash
# 確認 YAML 格式正確（需安裝 yq 或用 python）
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" && echo "YAML OK"
```

預期輸出：`YAML OK`

- [ ] **Step 2：Lint 檢查 manifest 語法**

```bash
python3 -c "import yaml; yaml.safe_load(open('waypoint/flatpak/io.github.i321i.waypoint.yml'))" && echo "Manifest YAML OK"
```

預期輸出：`Manifest YAML OK`

- [ ] **Step 3：推送測試 tag 觸發 CI**

```bash
git tag v0.0.1-flatpak-test
git push origin v0.0.1-flatpak-test
```

在 GitHub Actions 頁面觀察 `ubuntu-22.04-arm` job，確認：
1. `Install flatpak-builder` 步驟成功
2. `Build Flatpak` 步驟成功並產出 `.flatpak` 檔
3. `Upload Flatpak to Release` 步驟成功
4. Release 頁面出現 `io.github.i321i.waypoint_aarch64.flatpak` 附件

- [ ] **Step 4：在 Steam Deck Desktop Mode 實機測試**

下載 `.flatpak` 檔後執行：
```bash
flatpak install --user io.github.i321i.waypoint_aarch64.flatpak
flatpak run io.github.i321i.waypoint
```

確認 app 正常啟動，`~/waypoint/` 資料目錄可正常讀寫。

- [ ] **Step 5：刪除測試 tag（避免留下無意義的 release）**

```bash
git tag -d v0.0.1-flatpak-test
git push origin --delete v0.0.1-flatpak-test
```
