// WebdriverIO + tauri-driver 設定
// Windows CI：tauri-driver 代理 msedgedriver（WebView2），自動啟動 binary。
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import os from "node:os";

let tauriDriver;

const binary = process.env.WAYPOINT_BINARY;
if (!binary) throw new Error("WAYPOINT_BINARY env var is required");

const tauriDriverBin = path.join(
  os.homedir(),
  ".cargo",
  "bin",
  process.platform === "win32" ? "tauri-driver.exe" : "tauri-driver",
);

export const config = {
  runner: "local",
  specs: ["./specs/**/*.spec.js"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application: binary },
    },
  ],
  hostname: "127.0.0.1",
  port: 4444,
  logLevel: "info",
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 60_000 },
  reporters: ["spec"],
  waitforTimeout: 15_000,

  beforeSession() {
    // 讓 Tauri 自動開列表視窗（tray-only 啟動不會建 WebView）
    process.env.WAYPOINT_E2E = "1";

    tauriDriver = spawn(tauriDriverBin, [], {
      stdio: [null, process.stdout, process.stderr],
      env: process.env,
    });

    tauriDriver.on("error", (err) => {
      console.error("[tauri-driver] spawn error:", err);
    });
  },

  afterSession() {
    if (tauriDriver && !tauriDriver.killed) tauriDriver.kill();
  },
};
