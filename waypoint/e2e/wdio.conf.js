// WebdriverIO + tauri-driver 設定
// Windows CI：tauri-driver 代理 msedgedriver（WebView2），自動啟動 binary。
import { spawn, spawnSync } from "node:child_process";
import path from "node:path";
import os from "node:os";
import net from "node:net";

async function waitForPort(port, host, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const ok = await new Promise((resolve) => {
      const sock = net.connect(port, host);
      sock.once("connect", () => { sock.end(); resolve(true); });
      sock.once("error", () => resolve(false));
    });
    if (ok) return;
    await new Promise((r) => setTimeout(r, 200));
  }
  throw new Error(`tauri-driver did not listen on ${host}:${port} within ${timeoutMs}ms`);
}

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

  async beforeSession() {
    // 讓 Tauri 自動開列表視窗（tray-only 啟動不會建 WebView）
    process.env.WAYPOINT_E2E = "1";

    tauriDriver = spawn(tauriDriverBin, [], {
      stdio: [null, process.stdout, process.stderr],
      env: process.env,
    });

    tauriDriver.on("error", (err) => {
      console.error("[tauri-driver] spawn error:", err);
    });

    // spawn 是非同步的；第一個 spec 常在 port 4444 還沒 bind 前就 connect → ECONNREFUSED
    await waitForPort(4444, "127.0.0.1", 20_000);
  },

  afterSession() {
    if (tauriDriver && !tauriDriver.killed) tauriDriver.kill();
  },
};
