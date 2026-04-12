# Flatpak Steam Deck 支援 — 設計規格文件

**日期**：2026-04-13
**版本**：1.0

---

## 1. 目標

在 GitHub Release 中新增 aarch64 `.flatpak` bundle 檔，讓 Steam Deck 用戶可以在 Desktop Mode 直接安裝 Waypoint，無需透過 AppImage。

## 2. 範圍

- 僅建置 aarch64 Flatpak（目標：Steam Deck / SteamOS）
- 發佈方式：GitHub Release 附件（非 Flathub）
- 不修改 app 本身程式碼，不更動資料目錄路徑

---

## 3. Flatpak 基本資訊

| 欄位 | 值 |
|------|-----|
| App ID | `io.github.i321i.waypoint` |
| Runtime | `org.freedesktop.Platform//23.08` |
| SDK | `org.freedesktop.Sdk//23.08` |
| Command | `waypoint` |
| 輸出檔名 | `io.github.i321i.waypoint_aarch64.flatpak` |

---

## 4. Flatpak Manifest

路徑：`waypoint/flatpak/io.github.i321i.waypoint.yml`

```yaml
app-id: io.github.i321i.waypoint
runtime: org.freedesktop.Platform
runtime-version: '23.08'
sdk: org.freedesktop.Sdk
command: waypoint

finish-args:
  - --share=ipc
  - --socket=fallback-x11        # XWayland 支援（Steam Deck 主要走這條）
  - --socket=wayland              # 原生 Wayland（備用）
  - --filesystem=~/waypoint       # 資料目錄存取
  - --talk-name=org.kde.StatusNotifierWatcher  # System tray

modules:
  - name: waypoint
    buildsystem: simple
    build-commands:
      - install -Dm755 waypoint /app/bin/waypoint
    sources:
      - type: file
        path: ../../target/aarch64-unknown-linux-gnu/release/waypoint
```

### 設計決策

- **Runtime 選擇**：使用 `org.freedesktop.Platform` 而非 GNOME/KDE SDK，體積較小且已涵蓋所需的基礎函式庫
- **`--socket=fallback-x11`**：Steam Deck 主要透過 XWayland 執行桌面應用，此為必要權限
- **`--filesystem=~/waypoint`**：只開放 app 的資料目錄，沙箱其餘部分保持完整；因為是 sideload 發布（非 Flathub），硬編碼路徑可接受
- **binary 來源**：直接引用 tauri-action 建出的 Rust 執行檔，不在 Flatpak 內重新編譯

---

## 5. CI Workflow 修改

在 `.github/workflows/release.yml` 的 `ubuntu-22.04-arm` job 中，在 `Build Tauri` 步驟之後新增三個步驟：

### 步驟 1：安裝 flatpak-builder

```yaml
- name: Install flatpak-builder
  if: matrix.platform == 'ubuntu-22.04-arm'
  run: |
    sudo apt-get install -y flatpak flatpak-builder
    flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
    flatpak install --user -y flathub org.freedesktop.Platform//23.08 org.freedesktop.Sdk//23.08
```

### 步驟 2：建置 Flatpak bundle

```yaml
- name: Build Flatpak
  if: matrix.platform == 'ubuntu-22.04-arm'
  run: |
    flatpak-builder --repo=flatpak-repo --force-clean flatpak-build \
      waypoint/flatpak/io.github.i321i.waypoint.yml
    flatpak build-bundle flatpak-repo \
      io.github.i321i.waypoint_aarch64.flatpak \
      io.github.i321i.waypoint --arch=aarch64
```

### 步驟 3：上傳到 Release

```yaml
- name: Upload Flatpak to Release
  if: matrix.platform == 'ubuntu-22.04-arm'
  uses: softprops/action-gh-release@v2
  with:
    files: io.github.i321i.waypoint_aarch64.flatpak
```

---

## 6. Release 說明文字更新

在 `releaseBody` 新增安裝說明：

```
- **Steam Deck（Flatpak，推薦）**：下載 `_aarch64.flatpak`，在 Desktop Mode 執行：
  `flatpak install --user io.github.i321i.waypoint_aarch64.flatpak`
- **Steam Deck / Linux（AppImage）**：下載 `_aarch64.AppImage`，賦予執行權限後執行
```

---

## 7. 新增檔案清單

| 檔案 | 說明 |
|------|------|
| `waypoint/flatpak/io.github.i321i.waypoint.yml` | Flatpak manifest |

## 8. 修改檔案清單

| 檔案 | 修改內容 |
|------|---------|
| `.github/workflows/release.yml` | 新增三個步驟 + 更新 releaseBody |
