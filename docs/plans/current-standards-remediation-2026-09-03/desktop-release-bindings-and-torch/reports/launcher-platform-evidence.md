# Launcher Platform Evidence

**Milestone:** 4 — Complete Launcher Platform and Process Outcomes

**Status:** `in-progress`

## Contract Under Test

- Both public wrappers delegate every action and argument to the same Node
  launcher Module.
- The platform factory accepts only `linux`, `darwin`, and `win32`; every other
  value returns a typed unsupported-platform outcome.
- `commands.mjs` owns spawn observation, deadlines, diagnostic mapping, and one
  terminal result for each launcher-owned process.
- Each platform module is a mechanism-only Adapter that terminates the owned
  process tree gracefully and then forcibly when the grace deadline expires.
- Release smoke keeps its minimum-uptime success rule; exceeding its maximum
  window is an observed failure after bounded cleanup, never success.

## Target Evidence Matrix

| Target | Wrapper | Platform rejection | Grace/force/tree lifecycle | Status |
| --- | --- | --- | --- | --- |
| Linux x64 | Real Bash help/invalid-exit test passed | Closed factory test passed | Real resistant parent/descendant group removed | `verified-local` |
| Windows x64 | Not executable in this environment | Not executable in this environment | Not executable in this environment | `unavailable` |
| macOS arm64 | Not executable in this environment | Not executable in this environment | Not executable in this environment | `unavailable` |

Linux results will be recorded from real local processes. Windows and macOS
remain unavailable until the same suite runs on accepted target runners; source
inspection and Linux behavior are not substitute evidence.

## Initial Failing Oracles

- Wrapper parity: `node scripts/launcher/wrappers.test.mjs` failed because
  `launcher.sh` contains `RELEASE_BACKEND_BINARY`, frontend/Electron artifact
  checks, environment policy, argument shifting, and direct Electron `exec`;
  PowerShell delegates the same action to the shared core.
- Platform closure: `node scripts/launcher/actions.test.mjs` failed with
  `Missing expected exception` for `createPlatformService('plan9')`; the
  factory returned Linux.
- Forced termination: `node scripts/launcher/commands.test.mjs` failed after
  about 1.23 seconds because a child that ignored `SIGTERM` remained open until
  its 1.2-second self-exit. The intended 200 ms maximum plus 100 ms grace did
  not force termination.
- The aggregate `node --test` invocation also returned nonzero for all three
  focused files. Direct per-file execution supplied the detailed assertions
  above.

## Implementation Evidence

- `npm run test:launcher`: 41/41 passed in the environment permitted to spawn
  the real wrapper and controlled process trees.
- `bash -n launcher.sh`: passed.
- Focused whitespace diff check over the M4 write set: passed.
- Bash and PowerShell contain only Node discovery, display-name projection, and
  delegation to `scripts/launcher/cli.mjs`; the prior Bash release fast path
  was deleted.
- The factory recognizes exactly `linux`, `darwin`, and `win32`; unsupported
  values produce `LauncherError` with exit code 5 inside the CLI error boundary.
- The bounded runner validates its deadlines and process-tree Adapter, observes
  spawn/error/exit/close once, requests graceful termination at maximum uptime,
  forces after grace, and returns an explicit incomplete-cleanup failure if the
  child still does not close.
- The runner joins active Adapter termination work before returning, including
  when the application child closes first. A composed deferred-helper test
  makes that ordering observable.
- An intentionally ineffective Adapter proves the max/grace/force terminal
  failure occurs well before a controlled five-second child fallback; its
  generous three-second scheduling tolerance avoids treating runner load as an
  implementation oracle, and the test explicitly force-cleans the fixture.
- Linux required-real process evidence used a detached process group containing
  a parent and descendant that both resisted `SIGTERM`; `SIGKILL` closed the
  command and the descendant PID was no longer alive.
- The force policy test uses a deterministic child self-exit marker. The marker
  remained absent, proving settlement followed forced termination rather than
  the child's fallback exit without relying on a loaded-runner timing margin.
- Windows lower-fidelity unit evidence proves that graceful process-tree
  termination is explicitly unavailable and that a hung `taskkill.exe` helper
  is force-terminated and observed within its supplied deadline. The external
  spawn seam also covers observed exit 0 and stable nonzero failure mapping.
  Real Windows target evidence remains unavailable.

## Design Review

The process-execution Module retains policy and time-based state behind one
Interface. Linux and macOS share one small POSIX process-group Adapter; Windows
hides only its unavoidable `taskkill.exe` mechanism and nested helper
lifecycle. The wrapper-specific release path is deleted rather than mirrored
into PowerShell, and unknown OS values are rejected rather than expanded into
another fallback registry. Deleting `commands.mjs` would distribute timer and
terminal-result policy to callers, while deleting either mechanism Adapter
would force OS details into that policy owner, so both seams pass the deletion
test without becoming pass-through layers.
