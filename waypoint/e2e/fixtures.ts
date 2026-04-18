import { test as base, chromium, type Browser, type Page } from "@playwright/test";
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";

const CDP_PORT = 9222;
const CDP_URL = `http://127.0.0.1:${CDP_PORT}`;
const CONNECT_TIMEOUT_MS = 30_000;
const CONNECT_POLL_MS = 500;

async function waitForCdp(): Promise<Browser> {
  const start = Date.now();
  let lastErr: unknown;
  while (Date.now() - start < CONNECT_TIMEOUT_MS) {
    try {
      return await chromium.connectOverCDP(CDP_URL);
    } catch (err) {
      lastErr = err;
      await new Promise((r) => setTimeout(r, CONNECT_POLL_MS));
    }
  }
  throw new Error(`CDP connect timeout after ${CONNECT_TIMEOUT_MS}ms: ${lastErr}`);
}

export type AppFixture = {
  browser: Browser;
  proc: ChildProcess;
  findPage: (hashPattern: RegExp) => Promise<Page>;
};

export const test = base.extend<{ app: AppFixture }>({
  app: async ({}, use) => {
    const binary = process.env.WAYPOINT_BINARY;
    if (!binary) throw new Error("WAYPOINT_BINARY env var is required");
    if (!existsSync(binary)) throw new Error(`binary not found: ${binary}`);

    const proc = spawn(binary, [], {
      env: {
        ...process.env,
        WAYPOINT_E2E: "1",
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${CDP_PORT}`,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    proc.stdout?.on("data", (d) => process.stdout.write(`[app] ${d}`));
    proc.stderr?.on("data", (d) => process.stderr.write(`[app:err] ${d}`));

    const browser = await waitForCdp();

    const findPage = async (hashPattern: RegExp): Promise<Page> => {
      const deadline = Date.now() + 15_000;
      while (Date.now() < deadline) {
        for (const ctx of browser.contexts()) {
          for (const page of ctx.pages()) {
            if (hashPattern.test(page.url())) return page;
          }
        }
        await new Promise((r) => setTimeout(r, 300));
      }
      const urls = browser.contexts().flatMap((c) => c.pages().map((p) => p.url()));
      throw new Error(`no page matched ${hashPattern}. Open pages: ${JSON.stringify(urls)}`);
    };

    await use({ browser, proc, findPage });

    await browser.close().catch(() => {});
    if (!proc.killed) proc.kill("SIGTERM");
  },
});

export { expect } from "@playwright/test";
