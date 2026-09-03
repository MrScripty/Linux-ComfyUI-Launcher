# Renderer Harness Admission

## Decision

Admit the already-installed Electron runtime and Chromium DevTools Protocol
(CDP) accessibility/emulation surfaces behind one frontend-owned orchestration
command. No browser-runner dependency is justified.

The permanent harness should have three narrow owners:

- `run.mjs` builds each real Vite mode into a fresh temporary directory,
  starts Electron, collects the structured result, and removes the directory in
  `finally`;
- `electron-main.cjs` owns the isolated BrowserWindow, real preload, CDP
  accessibility/media Adapter, input dispatch, watchdog, and process shutdown;
- `fixtures.cjs` owns deterministic desktop-operation responses and scenario
  state, without replacing browser layout, focus, or accessibility behavior.

This keeps the Interface at one package command and hides build directories,
IPC setup, CDP commands, and cleanup. Removing the harness would otherwise
redistribute those policies across every representative workflow, so it is a
substantive Module rather than a script-name wrapper.

## Experiment Environment

- Date: 2026-09-03
- Host: Linux x86-64
- Node: 24.12.0
- Electron: 39.8.6
- Chromium: 142.0.7444.265
- Subject: production Vite `library-only` bundle and the compiled production
  Electron preload
- Browser state: hidden headless BrowserWindow, `contextIsolation: true`,
  `nodeIntegration: false`, `sandbox: true`, dedicated user-data directory
- External operations: deterministic `ipcMain` fixtures; no backend sidecar,
  operator library, network, or inherited browser profile

The repository sandbox rejected Chromium's sandbox-host startup even with the
normal no-sandbox flags. Re-running the installed Electron binary with the
approved process permission succeeded. This is an execution-environment
constraint, not a renderer failure; the permanent command must retain the
current release-smoke flags and run in the same process-capable job class.

## Command Prototype

The bounded experiment used no repository tooling or dependency changes:

```text
frontend/node_modules/.bin/vite build --mode library-only \
  --outDir /tmp/pumas-renderer-admission/dist --emptyOutDir

electron/node_modules/electron/dist/electron \
  --headless --no-sandbox --disable-setuid-sandbox \
  --disable-gpu --disable-dev-shm-usage \
  /tmp/pumas-renderer-admission/main.cjs
```

The Vite build completed in 8.29 seconds. The Electron workflow completed in
1.554 seconds after fixture correction. A two-mode permanent command should
remain comfortably bounded; record its actual duration in Milestone 5 rather
than promising a guessed threshold.

## Workflow And Observations

The experiment loaded
`file:///tmp/pumas-renderer-admission/dist/index.html`, waited for the model
import trigger, used Chromium input dispatch to open the import interaction,
read the full accessibility tree and DOM focus, sent Escape, closed through the
visible control, and emulated `prefers-reduced-motion: reduce`.

Observed positive capabilities:

- the initial Chromium accessibility tree contained 97 nodes and exposed the
  import trigger as a button named `Import models`;
- the production preload invoked deterministic status, model, download,
  preference, link-health, report, classification, and shard-detection
  fixtures;
- a selected fixture path drove the production renderer into the visible
  `Import Models` interaction;
- CDP media emulation made
  `matchMedia('(prefers-reduced-motion: reduce)').matches` true;
- the corrected fixture run emitted no renderer warning/error messages;
- Electron exited with code 0, the BrowserWindow was destroyed, the debugger
  detached, and a process-table check found no surviving experiment process.

## Reachable Negative Failure

The current `ModelImportDialog` deliberately exposed the failure Milestone 2
is expected to repair:

- visible dialog content had no `role="dialog"` and no `aria-modal="true"`;
- the Chromium accessibility tree contained zero dialog nodes;
- focus remained on the background `Import models` trigger after open;
- Escape did not dismiss the interaction;
- closing with its button left focus on `BODY`, rather than restoring the
  trigger.

These are externally observed browser outcomes, not implementation-text
checks. The same assertions become positive acceptance oracles after the modal
Module exists, and deleting any one of its lifecycle policies will make the
representative test fail.

## Isolation And Cleanup Protocol

The experiment used only `/tmp/pumas-renderer-admission` for the renderer
bundle, harness, Chromium profile, caches, cookies, and session storage. No
workspace build output was changed. The permanent runner must use `mkdtemp`,
pass its absolute paths to the Electron process, enforce a watchdog, destroy
the window and detach CDP on both success and failure, wait for process exit,
and recursively remove only that resolved temporary directory in `finally`.

The manual experiment directory was removed after this report was recorded.

## Marginal Value And Limits

Vitest/jsdom remains the faster oracle for component state, async sequencing,
and exhaustive edge cases, but it cannot decide Chromium accessibility-tree,
real focus/input, media-emulation, or built-entry behavior. The existing
release smoke launches Electron with production assets, but does not drive a
workflow, isolate desktop-operation outcomes, or inspect accessibility and
focus. This harness fills that specific gap and should not duplicate their
claims.

The admission proves a capable browser oracle and one built mode. It does not
yet prove corrected modal behavior, progress announcements, cache provenance,
both modes, packaged artifacts, a real backend, or screen-reader product
certification. Those remain with Milestones 2–5 and the platform plan.

## Permanent Write-Set Consequence

Milestone 5 is revised before implementation to replace speculative Vitest
renderer files with the three runtime-harness files named above. The selected
runner uses existing Electron/Vite/Node dependencies, so no lockfile or new
dependency is admitted. The package command remains subject to the program's
shared-manifest serialization.

## Stopping Condition

Satisfied. The experiment observed accessible role/name, focus, Escape and
restore outcomes, media preference, and a real built entry; it had a reachable
negative failure, deterministic fixtures, bounded measured duration, explicit
shutdown, and no surviving child process.
