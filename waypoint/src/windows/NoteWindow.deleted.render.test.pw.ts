import { test, expect, type Page } from '@playwright/test';

async function setupMock(page: Page, extraInvoke?: (cmd: string, args: unknown) => unknown) {
  await page.addInitScript((extraInvokeStr) => {
    const listeners: Array<{ event: string; handler: (payload: unknown) => void }> = [];

    (window as any).__tauri_event_listeners__ = listeners;

    (window as any).__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: 'note-test-id' },
        currentWebview: { label: 'note-test-id', windowLabel: 'note-test-id' },
      },
      invoke: (cmd: string, args: unknown) => {
        // plugin:event|listen — store the handler callback
        if (cmd === 'plugin:event|listen') {
          const a = args as { event: string; handler: number };
          // The Tauri JS SDK calls transformCallback to wrap the handler,
          // then passes handler id. We store the event name for later dispatch.
          listeners.push({ event: a.event, handler: a.handler as any });
          return Promise.resolve(0);
        }
        if (cmd === 'plugin:event|unlisten') return Promise.resolve();
        if (cmd === 'plugin:event|emit') return Promise.resolve();
        if (cmd === 'read_note') {
          return Promise.resolve({
            id: 'test-id', contextId: null, title: 'T', content: '',
            settings: { fontSize: 14, opacity: 1, hotkey: null, windowBounds: null, passthrough: false },
          });
        }
        if (cmd === 'get_transparent_includes_text') return Promise.resolve(true);
        if (cmd === 'save_content') {
          (window as any).__saveCount = ((window as any).__saveCount ?? 0) + 1;
          return Promise.resolve();
        }
        if (cmd === 'plugin:window|close') {
          (window as any).__windowClosed = true;
          return Promise.resolve();
        }
        // extra invoke override from test
        if (extraInvokeStr) {
          try {
            const fn = new Function('cmd', 'args', extraInvokeStr);
            const result = fn(cmd, args);
            if (result !== undefined) return result;
          } catch (_) {}
        }
        return Promise.resolve(null);
      },
      transformCallback: (fn: (...args: unknown[]) => unknown, once?: boolean) => {
        // Return a numeric id; store fn under that id so we can call it later
        const id = Math.floor(Math.random() * 1e9);
        (window as any).__tauri_callbacks__ = (window as any).__tauri_callbacks__ ?? {};
        (window as any).__tauri_callbacks__[id] = fn;
        return id;
      },
      unregisterCallback: (id: number) => {
        if ((window as any).__tauri_callbacks__) {
          delete (window as any).__tauri_callbacks__[id];
        }
      },
      convertFileSrc: (s: string) => s,
    };
  }, extraInvoke?.toString() ?? '');
}

test('note window registers listener for waypoint://note-deleted on mount', async ({ page }) => {
  // Track which events are listened to
  await page.addInitScript(() => {
    (window as any).__listenedEvents__ = [];
    const origInvoke = (window as any).__TAURI_INTERNALS__?.invoke;
    // We'll patch after TAURI_INTERNALS is set by hooking Object.defineProperty
    const descriptor = {
      configurable: true,
      get() { return this.__tauriInternalsValue__; },
      set(val: unknown) {
        this.__tauriInternalsValue__ = val;
        if (val && typeof (val as any).invoke === 'function') {
          const originalInvoke = (val as any).invoke.bind(val);
          (val as any).invoke = (cmd: string, args: unknown) => {
            if (cmd === 'plugin:event|listen') {
              (window as any).__listenedEvents__.push((args as any)?.event);
            }
            return originalInvoke(cmd, args);
          };
        }
      }
    };
    Object.defineProperty(window, '__TAURI_INTERNALS__', descriptor);
  });

  await setupMock(page);
  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  await page.locator('.note-window').waitFor({ state: 'visible' });

  const listenedEvents = await page.evaluate(() => (window as any).__listenedEvents__ ?? []);
  expect(listenedEvents).toContain('waypoint://note-deleted');
});

test('note window 收到 note-deleted 自身事件後不再呼叫 save_content', async ({ page }) => {
  await setupMock(page);

  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  await page.locator('.note-window').waitFor({ state: 'visible' });

  // Find the registered callback id for waypoint://note-deleted and fire it
  await page.evaluate(() => {
    const internals = (window as any).__TAURI_INTERNALS__;
    const listeners = (window as any).__tauri_event_listeners__ as Array<{
      event: string;
      handler: number;
    }>;
    if (!listeners) return;
    const entry = listeners.find((l) => l.event === 'waypoint://note-deleted');
    if (!entry) return;
    // handler is the callback id stored via transformCallback
    const callbacks = (window as any).__tauri_callbacks__;
    if (!callbacks) return;
    const fn = callbacks[entry.handler];
    if (typeof fn === 'function') {
      fn({ event: 'waypoint://note-deleted', payload: { noteId: 'test-id', contextId: null } });
    }
  });

  await page.waitForTimeout(300);

  const saveCount = await page.evaluate(() => (window as any).__saveCount ?? 0);
  expect(saveCount).toBe(0);
});

test('note window 收到不同 noteId 的 note-deleted 不關閉', async ({ page }) => {
  await setupMock(page);

  await page.goto('http://localhost:4173/#view=note&noteId=test-id');
  await page.waitForLoadState('networkidle');
  await page.locator('.note-window').waitFor({ state: 'visible' });

  // Fire with a different noteId
  await page.evaluate(() => {
    const listeners = (window as any).__tauri_event_listeners__ as Array<{
      event: string;
      handler: number;
    }>;
    if (!listeners) return;
    const entry = listeners.find((l) => l.event === 'waypoint://note-deleted');
    if (!entry) return;
    const callbacks = (window as any).__tauri_callbacks__;
    if (!callbacks) return;
    const fn = callbacks[entry.handler];
    if (typeof fn === 'function') {
      fn({ event: 'waypoint://note-deleted', payload: { noteId: 'other-id', contextId: null } });
    }
  });

  await page.waitForTimeout(300);

  // Window should NOT be closed
  const windowClosed = await page.evaluate(() => (window as any).__windowClosed ?? false);
  expect(windowClosed).toBe(false);
  // Note window should still be visible
  await expect(page.locator('.note-window')).toBeVisible();
});
