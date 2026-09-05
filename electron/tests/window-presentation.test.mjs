import assert from 'node:assert/strict';
import test from 'node:test';
import { createWindowPresentationOwner } from '../dist/window-presentation.js';

function createManualScheduler() {
  let nextId = 1;
  const callbacks = new Map();

  return {
    schedule(callback, delayMs) {
      const id = nextId;
      nextId += 1;
      callbacks.set(id, { callback, delayMs });
      return id;
    },
    clearScheduled(id) {
      callbacks.delete(id);
    },
    fireDelay(delayMs) {
      const match = [...callbacks.entries()].find(([, scheduled]) =>
        scheduled.delayMs === delayMs
      );
      assert.ok(match, `expected a pending ${delayMs}ms callback`);
      callbacks.delete(match[0]);
      match[1].callback();
    },
    pendingDelays() {
      return [...callbacks.values()].map(({ delayMs }) => delayMs).sort();
    },
  };
}

function createHarness(adapterOverrides = {}) {
  let authoritativeStatus = 'ready';
  let frameCallback;
  let markerPresent = false;
  const calls = {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 0,
    destroy: 0,
    quit: 0,
  };
  const markerCalls = [];
  const scheduler = createManualScheduler();
  const owner = createWindowPresentationOwner({
    ...scheduler,
    getAuthoritativeStatus: () => authoritativeStatus,
    showWindow: () => {
      calls.show += 1;
    },
    focusWindow: () => {
      calls.focus += 1;
    },
    reportFocusUnavailable: () => {},
    sendVisibilityTimeout: () => {
      calls.timeout += 1;
    },
    subscribeToPresentationFrames: (callback) => {
      markerCalls.push('subscribe');
      frameCallback = callback;
    },
    unsubscribeFromPresentationFrames: () => {
      markerCalls.push('unsubscribe');
      frameCallback = undefined;
    },
    insertPresentationMarker: (onInserted) => {
      markerCalls.push('insert');
      markerPresent = true;
      onInserted('marker-key');
    },
    removePresentationMarker: (_markerKey, onRemoved) => {
      markerCalls.push('remove');
      markerPresent = false;
      onRemoved();
    },
    invalidatePresentationFrame: () => {
      markerCalls.push('invalidate');
      frameCallback?.({ markerPresent });
    },
    frameContainsPresentationMarker: (frame) => frame.markerPresent,
    showNativeFatal: () => {
      calls.fatal += 1;
    },
    destroyWindow: () => {
      calls.destroy += 1;
    },
    quitApplication: () => {
      calls.quit += 1;
    },
    ...adapterOverrides,
  }, {
    presentationDeadlineMs: 30_000,
    fallbackGraceMs: 2_000,
  });
  owner.documentReady();

  return {
    calls,
    emitFrame(frame) {
      frameCallback?.(frame);
    },
    markerCalls,
    owner,
    scheduler,
    setAuthoritativeStatus(status) {
      authoritativeStatus = status;
    },
  };
}

test('reveal requires marker-present then strictly later marker-free presentation frames', () => {
  const harness = createHarness({
    invalidatePresentationFrame: () => {
      harness.markerCalls.push('invalidate');
    },
  });

  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();

  assert.equal(harness.calls.show, 0);
  assert.deepEqual(harness.markerCalls, ['subscribe', 'insert', 'invalidate']);

  harness.emitFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 0, 'queued pre-marker frame must not reveal');

  harness.emitFrame({ markerPresent: true });
  assert.equal(harness.calls.show, 0, 'marker proof must be removed before reveal');
  assert.deepEqual(harness.markerCalls, [
    'subscribe',
    'insert',
    'invalidate',
    'remove',
    'invalidate',
  ]);

  harness.emitFrame({ markerPresent: true });
  assert.equal(harness.calls.show, 0, 'a queued marker frame must not reveal');

  harness.emitFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 1);
  assert.deepEqual(harness.markerCalls, [
    'subscribe',
    'insert',
    'invalidate',
    'remove',
    'invalidate',
    'unsubscribe',
  ]);
});

test('navigation cancels the old frame challenge and requires a fresh document proof', () => {
  const frameCallbacks = [];
  const harness = createHarness({
    subscribeToPresentationFrames: (callback) => {
      harness.markerCalls.push('subscribe');
      frameCallbacks.push(callback);
    },
    unsubscribeFromPresentationFrames: () => {
      harness.markerCalls.push('unsubscribe');
    },
    invalidatePresentationFrame: () => {
      harness.markerCalls.push('invalidate');
    },
  });

  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  const staleFrame = frameCallbacks[0];
  assert.ok(staleFrame);

  harness.owner.documentChanged();
  staleFrame({ markerPresent: true });
  staleFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 0);

  harness.owner.documentReady();
  harness.owner.browserReady();
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  const currentFrame = frameCallbacks[1];
  assert.ok(currentFrame);
  currentFrame({ markerPresent: true });
  currentFrame({ markerPresent: false });

  assert.equal(harness.calls.show, 1);
  assert.equal(frameCallbacks.length, 2);
  assert.deepEqual(
    harness.markerCalls.filter((call) => call === 'subscribe'),
    ['subscribe', 'subscribe']
  );
});

test('deadline fallback cancels the old challenge and proves unavailable in a fresh challenge', () => {
  const frameCallbacks = [];
  let owner;
  const harness = createHarness({
    subscribeToPresentationFrames: (callback) => {
      harness.markerCalls.push('subscribe');
      frameCallbacks.push(callback);
    },
    unsubscribeFromPresentationFrames: () => {
      harness.markerCalls.push('unsubscribe');
    },
    invalidatePresentationFrame: () => {
      harness.markerCalls.push('invalidate');
    },
    sendVisibilityTimeout: () => {
      harness.calls.timeout += 1;
      owner.rendererCommitted({
        currentTopDocument: true,
        presentation: 'bridge-unavailable',
      });
    },
  });
  owner = harness.owner;
  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  owner.browserReady();
  const staleFrame = frameCallbacks[0];

  harness.scheduler.fireDelay(30_000);
  assert.equal(frameCallbacks.length, 2);
  staleFrame({ markerPresent: true });
  staleFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 0);

  frameCallbacks[1]({ markerPresent: true });
  frameCallbacks[1]({ markerPresent: false });
  assert.equal(harness.calls.show, 1);
  assert.equal(harness.calls.timeout, 1);
  assert.deepEqual(harness.scheduler.pendingDelays(), []);
});

for (const [stage, adapterOverrides, drive] of [
  [
    'subscription',
    { subscribeToPresentationFrames: () => { throw new Error('subscription unavailable'); } },
    () => {},
  ],
  [
    'marker insertion',
    { insertPresentationMarker: (_onInserted, onUnavailable) => onUnavailable() },
    () => {},
  ],
  [
    'marker-frame invalidation',
    { invalidatePresentationFrame: () => { throw new Error('invalidation unavailable'); } },
    () => {},
  ],
  [
    'frame decoding',
    { frameContainsPresentationMarker: () => { throw new Error('image unavailable'); } },
    (harness) => harness.emitFrame({ markerPresent: true }),
  ],
  [
    'marker removal',
    {
      invalidatePresentationFrame: () => {},
      removePresentationMarker: (_markerKey, _onRemoved, onUnavailable) => onUnavailable(),
    },
    (harness) => harness.emitFrame({ markerPresent: true }),
  ],
  [
    'marker-free invalidation',
    {
      invalidatePresentationFrame: (() => {
        let calls = 0;
        return () => {
          calls += 1;
          if (calls === 2) {
            throw new Error('clear invalidation unavailable');
          }
        };
      })(),
    },
    (harness) => harness.emitFrame({ markerPresent: true }),
  ],
  [
    'subscription disposal',
    {
      invalidatePresentationFrame: () => {},
      unsubscribeFromPresentationFrames: () => {
        throw new Error('subscription disposal unavailable');
      },
    },
    (harness) => {
      harness.emitFrame({ markerPresent: true });
      harness.emitFrame({ markerPresent: false });
    },
  ],
]) {
  test(`${stage} failure reaches one native fatal terminal without showing`, () => {
    const harness = createHarness(adapterOverrides);
    harness.owner.rendererCommitted({
      currentTopDocument: true,
      presentation: 'ready',
    });
    harness.owner.browserReady();
    drive(harness);

    assert.deepEqual(harness.calls, {
      show: 0,
      focus: 0,
      timeout: 0,
      fatal: 1,
      destroy: 1,
      quit: 1,
    });
    assert.deepEqual(harness.scheduler.pendingDelays(), []);
  });
}

test('cleanup failure cannot replace an earlier frame-challenge failure', () => {
  const harness = createHarness({
    invalidatePresentationFrame: () => {
      throw new Error('primary invalidation failure');
    },
    removePresentationMarker: () => {
      throw new Error('secondary marker cleanup failure');
    },
  });

  assert.doesNotThrow(() => {
    harness.owner.rendererCommitted({
      currentTopDocument: true,
      presentation: 'ready',
    });
    harness.owner.browserReady();
  });
  assert.deepEqual(harness.calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
});

test('a matching callback before marker insertion settles cannot satisfy proof', () => {
  let callback;
  let inserted;
  const harness = createHarness({
    subscribeToPresentationFrames: (nextCallback) => {
      harness.markerCalls.push('subscribe');
      callback = nextCallback;
    },
    insertPresentationMarker: (onInserted) => {
      harness.markerCalls.push('insert');
      callback({ markerPresent: true });
      inserted = onInserted;
    },
    invalidatePresentationFrame: () => {
      harness.markerCalls.push('invalidate');
    },
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();

  inserted('marker-key');
  callback({ markerPresent: false });
  assert.equal(harness.calls.show, 0);

  callback({ markerPresent: true });
  callback({ markerPresent: false });
  assert.equal(harness.calls.show, 1);
});

test('deadline cancellation cleanup failure terminates before fallback work starts', () => {
  const harness = createHarness({
    invalidatePresentationFrame: () => {},
    removePresentationMarker: () => {
      throw new Error('marker cleanup unavailable');
    },
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();

  harness.scheduler.fireDelay(30_000);

  assert.deepEqual(harness.calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(harness.scheduler.pendingDelays(), []);
});

test('active marker removal settles once when an adapter reports conflicting callbacks', () => {
  let onRemoved;
  let onUnavailable;
  const harness = createHarness({
    invalidatePresentationFrame: () => {},
    removePresentationMarker: (_markerKey, nextRemoved, nextUnavailable) => {
      onRemoved = nextRemoved;
      onUnavailable = nextUnavailable;
    },
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  harness.emitFrame({ markerPresent: true });

  onRemoved();
  onUnavailable();
  onRemoved();
  harness.emitFrame({ markerPresent: false });

  assert.equal(harness.calls.show, 1);
  assert.equal(harness.calls.fatal, 0);
});

test('active marker removal failure remains terminal when a late success arrives', () => {
  let onRemoved;
  let onUnavailable;
  const harness = createHarness({
    invalidatePresentationFrame: () => {},
    removePresentationMarker: (_markerKey, nextRemoved, nextUnavailable) => {
      onRemoved = nextRemoved;
      onUnavailable = nextUnavailable;
    },
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  harness.emitFrame({ markerPresent: true });

  onUnavailable();
  onRemoved();

  assert.equal(harness.calls.show, 0);
  assert.equal(harness.calls.fatal, 1);
  assert.equal(harness.calls.destroy, 1);
  assert.equal(harness.calls.quit, 1);
});

test('cancelled marker cleanup settles once and late conflicting callbacks are inert', () => {
  let onRemoved;
  let onUnavailable;
  const harness = createHarness({
    invalidatePresentationFrame: () => {},
    removePresentationMarker: (_markerKey, nextRemoved, nextUnavailable) => {
      onRemoved = nextRemoved;
      onUnavailable = nextUnavailable;
    },
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  harness.owner.documentChanged();

  onRemoved();
  onUnavailable();
  onRemoved();

  assert.equal(harness.calls.show, 0);
  assert.equal(harness.calls.fatal, 0);
  assert.deepEqual(harness.scheduler.pendingDelays(), [30_000]);
});

test('late marker insertion after navigation is cleaned before a fresh challenge starts', () => {
  const insertions = [];
  const removed = [];
  const harness = createHarness({
    insertPresentationMarker: (onInserted, onUnavailable) => {
      insertions.push({ onInserted, onUnavailable });
    },
    removePresentationMarker: (markerHandle, onRemoved) => {
      removed.push(markerHandle);
      onRemoved();
    },
    invalidatePresentationFrame: () => {},
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  assert.equal(insertions.length, 1);

  harness.owner.documentChanged();
  insertions[0].onInserted('stale-marker');
  assert.deepEqual(removed, ['stale-marker']);

  harness.owner.documentReady();
  harness.owner.browserReady();
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  assert.equal(insertions.length, 2);
  insertions[1].onInserted('current-marker');
  harness.emitFrame({ markerPresent: true });
  harness.emitFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 1);
});

test('late marker insertion after deadline is cleaned before fallback challenge starts', () => {
  const insertions = [];
  const removed = [];
  let owner;
  const harness = createHarness({
    insertPresentationMarker: (onInserted, onUnavailable) => {
      insertions.push({ onInserted, onUnavailable });
    },
    removePresentationMarker: (markerHandle, onRemoved) => {
      removed.push(markerHandle);
      onRemoved();
    },
    invalidatePresentationFrame: () => {},
    sendVisibilityTimeout: () => {
      harness.calls.timeout += 1;
      owner.rendererCommitted({
        currentTopDocument: true,
        presentation: 'bridge-unavailable',
      });
    },
  });
  owner = harness.owner;
  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  owner.browserReady();
  harness.scheduler.fireDelay(30_000);
  assert.equal(insertions.length, 1);

  insertions[0].onInserted('stale-marker');
  assert.deepEqual(removed, ['stale-marker']);
  assert.equal(insertions.length, 2);

  insertions[1].onInserted('fallback-marker');
  harness.emitFrame({ markerPresent: true });
  harness.emitFrame({ markerPresent: false });
  assert.equal(harness.calls.show, 1);
  assert.equal(harness.calls.timeout, 1);
});

test('authority is re-correlated after marker proof and before native show', () => {
  const harness = createHarness({
    invalidatePresentationFrame: () => {},
  });
  harness.owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  harness.owner.browserReady();
  harness.emitFrame({ markerPresent: true });
  harness.setAuthoritativeStatus('recovery-required');
  harness.emitFrame({ markerPresent: false });

  assert.equal(harness.calls.show, 0);
  assert.equal(harness.calls.fatal, 0);
});

test('synchronous construction deadline callback owns and clears returned handles', () => {
  const cleared = [];
  const harness = createHarness({
    schedule: (callback, delayMs) => {
      if (delayMs === 30_000) {
        callback();
      }
      return `timer-${delayMs}`;
    },
    clearScheduled: (handle) => {
      cleared.push(handle);
    },
  });

  assert.equal(harness.calls.timeout, 1);
  assert.equal(harness.calls.fatal, 0);
  assert.deepEqual(cleared, ['timer-30000']);
});

test('synchronous fallback-grace callback cannot send timeout work after fatal', () => {
  const cleared = [];
  const scheduler = createManualScheduler();
  const harness = createHarness({
    schedule: (callback, delayMs) => {
      if (delayMs === 2_000) {
        callback();
        return `timer-${delayMs}`;
      }
      return scheduler.schedule(callback, delayMs);
    },
    clearScheduled: (handle) => {
      cleared.push(handle);
      if (typeof handle === 'number') {
        scheduler.clearScheduled(handle);
      }
    },
  });

  scheduler.fireDelay(30_000);
  assert.equal(harness.calls.timeout, 0);
  assert.equal(harness.calls.fatal, 1);
  assert.deepEqual(cleared, [1, 'timer-2000']);
});

test('current ready presentation and browser readiness reveal one window', () => {
  const { calls, owner } = createHarness();

  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'accepted' });
  assert.equal(calls.show, 0);

  owner.browserReady();
  owner.browserReady();
  assert.equal(calls.show, 1);

  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'terminal' });
  assert.equal(calls.show, 1);
});

test('only a closed current presentation correlated to launcher authority is accepted', () => {
  const { calls, owner, setAuthoritativeStatus } = createHarness();
  owner.browserReady();

  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: false,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'stale-document' });
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'checking',
  }), { status: 'invalid' });
  setAuthoritativeStatus('initializing');
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'authority-mismatch' });
  assert.equal(calls.show, 0);

  setAuthoritativeStatus('recovery-required');
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'recovery-required',
  }), { status: 'accepted' });
  assert.equal(calls.show, 1);
});

test('visibility deadline accepts only a committed unavailable fallback', () => {
  const { calls, owner, scheduler } = createHarness();
  owner.browserReady();

  scheduler.fireDelay(30_000);
  assert.equal(calls.timeout, 1);
  assert.deepEqual(scheduler.pendingDelays(), [2_000]);
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'fallback-required' });
  assert.equal(calls.show, 0);

  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'bridge-unavailable',
  }), { status: 'accepted' });
  assert.equal(calls.show, 1);
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('missing fallback acknowledgement reaches one native fatal terminal', () => {
  const { calls, owner, scheduler } = createHarness();
  owner.browserReady();

  scheduler.fireDelay(30_000);
  scheduler.fireDelay(2_000);

  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 1,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(scheduler.pendingDelays(), []);
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'bridge-unavailable',
  }), { status: 'ignored', reason: 'terminal' });
});

test('preload failure reveals only after the synchronous fallback is browser-ready', () => {
  for (const order of ['preload-first', 'browser-first']) {
    const { calls, owner, scheduler } = createHarness();

    if (order === 'preload-first') {
      owner.preloadFailed();
      assert.equal(calls.show, 0);
      owner.browserReady();
    } else {
      owner.browserReady();
      assert.equal(calls.show, 0);
      owner.preloadFailed();
    }

    assert.equal(calls.show, 1);
    assert.deepEqual(scheduler.pendingDelays(), []);
    owner.preloadFailed();
    owner.browserReady();
    assert.equal(calls.show, 1);
  }
});

test('focus requests never bypass readiness and are applied after the one reveal', () => {
  const { calls, owner } = createHarness();

  owner.focusRequested();
  owner.browserReady();
  assert.deepEqual({ show: calls.show, focus: calls.focus }, { show: 0, focus: 0 });

  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  assert.deepEqual({ show: calls.show, focus: calls.focus }, { show: 1, focus: 1 });

  owner.focusRequested();
  assert.deepEqual({ show: calls.show, focus: calls.focus }, { show: 1, focus: 2 });
});

test('window disposal cancels deadlines and makes late work terminal', () => {
  const { calls, owner, scheduler } = createHarness();

  owner.focusRequested();
  owner.dispose();
  assert.deepEqual(scheduler.pendingDelays(), []);
  owner.browserReady();
  owner.preloadFailed();
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'terminal' });

  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 0,
    destroy: 0,
    quit: 0,
  });
});

test('load failure reaches the same bounded native fatal terminal immediately', () => {
  const { calls, owner, scheduler } = createHarness();

  owner.loadFailed();

  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(scheduler.pendingDelays(), []);
  owner.loadFailed();
  assert.equal(calls.fatal, 1);
});

test('authority is re-correlated at reveal instead of trusting a commit snapshot', () => {
  const { calls, owner, setAuthoritativeStatus } = createHarness();
  setAuthoritativeStatus('ready');
  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });

  setAuthoritativeStatus('recovery-required');
  owner.browserReady();

  assert.equal(calls.show, 0);
});

test('main-frame navigation invalidates commit and readiness without extending deadline', () => {
  const { calls, owner, scheduler } = createHarness();

  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  owner.documentChanged();
  owner.browserReady();
  assert.equal(calls.show, 0);
  assert.deepEqual(scheduler.pendingDelays(), [30_000]);

  owner.documentReady();
  assert.equal(calls.show, 0);
  assert.deepEqual(scheduler.pendingDelays(), [30_000]);

  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  assert.equal(calls.show, 1);
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('pre-navigation browser readiness cannot reveal a later document', () => {
  const { calls, owner, scheduler } = createHarness();
  owner.browserReady();
  owner.documentChanged();
  owner.documentReady();

  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });

  assert.equal(calls.show, 0);
  assert.deepEqual(scheduler.pendingDelays(), [30_000]);
  owner.browserReady();
  assert.equal(calls.show, 1);
});

test('pre-navigation readiness cannot authorize a later preload-failure fallback', () => {
  const { calls, owner, scheduler } = createHarness();
  owner.browserReady();
  owner.documentChanged();
  owner.documentReady();
  owner.preloadFailed();

  assert.equal(calls.show, 0);
  assert.deepEqual(scheduler.pendingDelays(), [30_000]);
  owner.browserReady();
  assert.equal(calls.show, 1);
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('a failed timeout signal reaches native fatal instead of stranding the window', () => {
  const { calls, scheduler } = createHarness({
    sendVisibilityTimeout: () => {
      calls.timeout += 1;
      throw new Error('renderer timeout channel unavailable');
    },
  });

  assert.doesNotThrow(() => scheduler.fireDelay(30_000));
  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 1,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('a synchronous timeout fallback commit clears its already-owned grace timer', () => {
  let owner;
  const harness = createHarness({
    sendVisibilityTimeout: () => {
      harness.calls.timeout += 1;
      owner.rendererCommitted({
        currentTopDocument: true,
        presentation: 'bridge-unavailable',
      });
    },
  });
  owner = harness.owner;
  owner.browserReady();

  harness.scheduler.fireDelay(30_000);

  assert.deepEqual(harness.scheduler.pendingDelays(), []);
  assert.equal(harness.calls.show, 1);
  assert.equal(harness.calls.timeout, 1);
  assert.equal(harness.calls.fatal, 0);
});

test('a failed fallback-grace schedule reaches native fatal immediately', () => {
  const baseScheduler = createManualScheduler();
  const { calls } = createHarness({
    schedule: (callback, delayMs) => {
      if (delayMs === 2_000) {
        throw new Error('fallback grace timer unavailable');
      }
      return baseScheduler.schedule(callback, delayMs);
    },
    clearScheduled: baseScheduler.clearScheduled,
  });

  assert.doesNotThrow(() => baseScheduler.fireDelay(30_000));
  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
});

test('a failed native show reaches fatal cleanup instead of claiming visibility', () => {
  const { calls, owner, scheduler } = createHarness({
    showWindow: () => {
      calls.show += 1;
      throw new Error('native window show failed');
    },
  });
  owner.browserReady();

  assert.doesNotThrow(() => owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }));
  assert.deepEqual(calls, {
    show: 1,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('authority inspection failure reaches native fatal without escaping IPC work', () => {
  const { calls, owner, scheduler } = createHarness({
    getAuthoritativeStatus: () => {
      throw new Error('authority state unavailable');
    },
  });
  owner.browserReady();

  assert.doesNotThrow(() => owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }));
  assert.deepEqual(calls, {
    show: 0,
    focus: 0,
    timeout: 0,
    fatal: 1,
    destroy: 1,
    quit: 1,
  });
  assert.deepEqual(scheduler.pendingDelays(), []);
});

test('focus failure is observed as harmless after the window is visible', () => {
  let reportedUnavailable = 0;
  const { calls, owner } = createHarness({
    focusWindow: () => {
      calls.focus += 1;
      throw new Error('window manager denied focus');
    },
    reportFocusUnavailable: () => {
      reportedUnavailable += 1;
    },
  });

  assert.deepEqual(owner.focusRequested(), { status: 'queued' });
  owner.browserReady();
  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  assert.equal(calls.show, 1);
  assert.equal(reportedUnavailable, 1);

  assert.deepEqual(owner.focusRequested(), { status: 'unavailable' });
  assert.equal(reportedUnavailable, 2);
  assert.equal(calls.destroy, 0);
  assert.equal(calls.quit, 0);
});

test('initial deadline scheduling failure reaches native fatal during construction', () => {
  const calls = { fatal: 0, destroy: 0, quit: 0 };
  const owner = createWindowPresentationOwner({
    getAuthoritativeStatus: () => 'initializing',
    schedule: () => {
      throw new Error('timer scheduling unavailable');
    },
    clearScheduled: () => {},
    showWindow: () => assert.fail('failed construction must not show'),
    focusWindow: () => assert.fail('failed construction must not focus'),
    reportFocusUnavailable: () => {},
    sendVisibilityTimeout: () => assert.fail('failed construction has no deadline'),
    showNativeFatal: () => {
      calls.fatal += 1;
    },
    destroyWindow: () => {
      calls.destroy += 1;
    },
    quitApplication: () => {
      calls.quit += 1;
    },
  }, {
    presentationDeadlineMs: 30_000,
    fallbackGraceMs: 2_000,
  });

  assert.deepEqual(calls, { fatal: 1, destroy: 1, quit: 1 });
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'bridge-unavailable',
  }), { status: 'ignored', reason: 'terminal' });
});

test('timer-clear failure cannot revive work after the window is shown', () => {
  const scheduler = createManualScheduler();
  const { calls, owner } = createHarness({
    schedule: scheduler.schedule,
    clearScheduled: () => {
      throw new Error('timer cancellation unavailable');
    },
  });

  owner.browserReady();
  owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  });
  assert.equal(calls.show, 1);

  assert.doesNotThrow(() => scheduler.fireDelay(30_000));
  assert.equal(calls.timeout, 0);
  assert.equal(calls.fatal, 0);
});

test('fatal diagnostic and cleanup adapter failures are contained and all attempted', () => {
  const attempts = [];
  const { owner } = createHarness({
    showNativeFatal: () => {
      attempts.push('fatal');
      throw new Error('native dialog unavailable');
    },
    destroyWindow: () => {
      attempts.push('destroy');
      throw new Error('window destroy unavailable');
    },
    quitApplication: () => {
      attempts.push('quit');
      throw new Error('app quit unavailable');
    },
  });

  assert.doesNotThrow(() => owner.loadFailed());
  assert.deepEqual(attempts, ['fatal', 'destroy', 'quit']);
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'terminal' });
});

test('the first correlated commit is one-shot before browser readiness', () => {
  const { calls, owner } = createHarness();

  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'accepted' });
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'ready',
  }), { status: 'ignored', reason: 'already-committed' });
  assert.deepEqual(owner.rendererCommitted({
    currentTopDocument: true,
    presentation: 'bridge-unavailable',
  }), { status: 'ignored', reason: 'already-committed' });

  owner.browserReady();
  assert.equal(calls.show, 1);
});
