import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { runInNewContext } from 'node:vm';
import test from 'node:test';
import { setImmediate } from 'node:timers/promises';

const mainUrl = new URL('../dist/main.js', import.meta.url);
const requireMain = createRequire(mainUrl);

function createMainHarness({ root, bridgeRuntime } = {}) {
  const app = new EventEmitter();
  const timers = new Set();
  const windows = [];
  const exits = [];
  const errors = [];
  let ready;
  const readiness = new Promise((resolve) => { ready = resolve; });
  Object.assign(app, {
    isPackaged: false,
    commandLine: { appendSwitch() {} },
    requestSingleInstanceLock: () => true,
    whenReady: () => readiness,
    getPath: () => root,
    quit() { app.emit('before-quit', { preventDefault() {} }); },
    exit(code) {
      exits.push(code);
      for (const window of windows) if (!window.destroyed) window.destroy();
    },
  });
  class BrowserWindow extends EventEmitter {
    destroyed = false;
    contents = new EventEmitter();
    constructor() {
      super();
      this.contents.isDestroyed = () => this.destroyed;
      windows.push(this);
    }
    static getAllWindows() { return windows.filter((window) => !window.destroyed); }
    get webContents() {
      if (this.destroyed) throw new TypeError('Object has been destroyed');
      return this.contents;
    }
    isDestroyed() { return this.destroyed; }
    async loadFile() {}
    destroy() {
      this.destroyed = true;
      this.emit('closed');
    }
  }
  const logger = {
    transports: { file: {}, console: {} },
    info() {}, warn() {}, error(...args) { errors.push(args); },
  };
  const runtimeProcess = {
    argv: [],
    env: root ? { PUMAS_RPC_BINARY: process.execPath } : {},
    platform: 'linux',
    on() {},
  };
  runInNewContext(readFileSync(mainUrl, 'utf8'), {
    exports: {},
    __dirname: root ? join(root, 'electron', 'dist') : fileURLToPath(new URL('../dist', import.meta.url)),
    process: runtimeProcess,
    setTimeout(callback) { timers.add(callback); return callback; },
    clearTimeout(callback) { timers.delete(callback); },
    require(specifier) {
      if (specifier === 'electron') return {
        app, BrowserWindow, ipcMain: { handle() {} },
      };
      if (specifier === 'electron-log') return logger;
      if (specifier === './python-bridge' && bridgeRuntime) return bridgeRuntime.module;
      if (specifier === './launcher-root' && root) {
        const exports = {};
        runInNewContext(readFileSync(new URL('../dist/launcher-root.js', import.meta.url), 'utf8'), {
          exports, require: requireMain, process: runtimeProcess,
        });
        return exports;
      }
      return requireMain(specifier);
    },
  });
  return { app, timers, windows, ready, exits, errors };
}

test('closing the native window settles presentation without accessing its destroyed getter', async () => {
  const { app, timers, windows } = createMainHarness();
  const activation = app.listeners('activate')[0]();
  // Let the native load finish so the closed lifecycle is installed.
  await setImmediate();
  const [window] = windows;
  try {
    assert.doesNotThrow(() => window.destroy());
    await activation;
    assert.equal(timers.size, 0, 'closed presentation must release its deadline');
    // A later activation must be able to own a new window.
    const replacement = app.listeners('activate')[0]();
    await setImmediate();
    assert.equal(windows.length, 2);
    windows[1].destroy();
    await replacement;
  } finally {
    timers.clear();
  }
});

function createFailedProcessRuntime() {
  const timers = new Set();
  const requests = [];
  const children = [];
  let now = 0;
  const module = {};
  const server = new EventEmitter();
  Object.assign(server, {
    listen(_port, _host, callback) { queueMicrotask(callback); },
    address: () => ({ port: 49152 }),
    close(callback) { callback(); },
  });
  runInNewContext(readFileSync(new URL('../dist/python-bridge.js', import.meta.url), 'utf8'), {
    exports: module,
    Buffer,
    process: { env: {} },
    Date: { now: () => now },
    setTimeout(callback, delayMs) {
      const timer = { callback, delayMs };
      timers.add(timer);
      return timer;
    },
    clearTimeout(timer) { timers.delete(timer); },
    require(specifier) {
      if (specifier === 'electron-log') return { info() {}, warn() {}, error() {} };
      if (specifier === 'net') return { createServer: () => server };
      if (specifier === 'child_process') return {
        spawn() {
          const child = new EventEmitter();
          children.push(child);
          return child;
        },
      };
      if (specifier === 'http') return {
        request() {
          const request = new EventEmitter();
          Object.assign(request, { write() {}, end() {} });
          requests.push(request);
          return request;
        },
      };
      return requireMain(specifier);
    },
  });
  return { module, timers, requests, children, expireStartup() { now = 30_000; } };
}

test('failed backend startup stops its pending restart before application cleanup completes', async () => {
  const root = mkdtempSync(join(tmpdir(), 'pumas-main-lifecycle-'));
  mkdirSync(join(root, 'shared-resources', 'models'), { recursive: true });
  const runtime = createFailedProcessRuntime();
  const harness = createMainHarness({ root, bridgeRuntime: runtime });
  try {
    harness.ready();
    await setImmediate();
    assert.equal(runtime.children.length, 1);
    assert.equal(runtime.requests.length, 1);
    runtime.children[0].emit('exit', 1, null);
    assert.equal(runtime.timers.size, 1, 'crash has a pending restart before startup fails');
    runtime.expireStartup();
    runtime.requests[0].emit('error', new Error('connection refused'));
    await setImmediate();
    assert.equal(harness.exits.length, 1, 'failed startup must finish application cleanup');
    assert.equal(runtime.timers.size, 0, 'failed initialization must retain its bridge until restart is stopped');
    assert.ok(harness.errors.some((args) => args.some((value) =>
      value?.message === 'RPC server failed to start within timeout'
    )), 'cleanup must preserve the original startup failure');
  } finally {
    runtime.timers.clear();
    harness.timers.clear();
    rmSync(root, { recursive: true, force: true });
  }
});
