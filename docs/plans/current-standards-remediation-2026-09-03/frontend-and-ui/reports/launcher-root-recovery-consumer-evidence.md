# Launcher-Root Recovery Consumer Evidence

**Status:** Verifying

**Date:** 2026-09-03

**Claim:** FE-A8 / XR-S1

## Boundary

XR-S1 consumes the platform-owned, path-free launcher-root startup and
selection unions. It does not define producer recovery policy, edit Electron
IPC, start M4 catalog work, or add permanent renderer tooling.

The renderer owner is `LauncherRootRecoveryProvider` at the production
composition root. Browser mode is explicitly not applicable and renders its
children. Electron mode withholds backend-consuming children until startup
reaches `ready`; it then owns selection presentation without duplicating the
main process's single-flight, terminal-lock, or bounded visibility policy.

The Electron semantic owner remains
`electron/src/launcher-root-recovery.ts`. The frontend unions are a temporary,
separately compiled consumer projection, not independent schema authority;
Platform Milestone 1 invalidates and replaces them with generated contract
output. XR-S1 acceptance therefore requires every closed value to traverse the
actual compiled preload decoder into the frontend behavior below.

## Closed Behavior Matrix

| Producer state/outcome | Renderer behavior |
| --- | --- |
| `initializing` | Sequential single-owner polling; no window-visibility acknowledgement |
| `ready/select-library` | Render application; Change Library delegates to the provider |
| `ready/correct-launch-input` | Render application; Change Library shows dismissible launch-input guidance without invoking selection |
| startup recovery from persisted authority | Block application and offer Select Library |
| startup recovery from environment/argument | Block application, explain launch-input correction, and never invoke selection |
| `cancelled` | Restore the exact prior presentation |
| invalid selection, chooser unavailable, or persistence unchanged | Offer Try Again; offer Back only when the prior startup authority was ready |
| replacement visibility unknown, published durability unavailable, restart unavailable, or restarting | Block application and retry; require close/reopen or show restart state |
| invocation or decoder rejection | Commit path-free, non-retryable desktop-bridge unavailable state, then acknowledge it |
| main-process presentation timeout | Terminal-latch the current startup generation, commit bridge unavailable, acknowledge it once, and ignore late startup replies |
| Electron identity without a preload bridge | Synchronously render bridge unavailable, hide unsupported Minimize, and retain a browser Close fallback; main owns preload-error visibility |

All bridge-backed non-content presentations use one named region, an atomic
polite status, focused heading, and the existing frameless minimize/close
controls. A missing bridge cannot minimize, so that action is absent rather
than deceptive.

## TDD And Focused Evidence

The public provider/action/view seams captured failures before implementation:
premature child mount, missing recovery actions, incorrect explicit-authority
selection, duplicate StrictMode reads, absent sequential polling cleanup,
incorrect retry/back availability, unobserved rejection, overlapping selection
calls,
missing focus, and legacy direct bridge selection. The focused provider suite
then captured five handshake failures: no committed-presentation notification,
no timeout subscription, and no terminal ownership for a late startup reply.
The provider suite now passes 27 tests. It proves no acknowledgement while a
startup request is unresolved or repeatedly initializing; a direct post-layout,
one-shot acknowledgement for ready, recovery, and unavailable; StrictMode
subscription/notification ownership; timeout terminal latching; late-result
rejection; subscription cleanup; and observed acknowledgement rejection with
no renderer-invented fallback.

The startup timing regression required the representative oracle. A
provider-local `flushSync` experiment produced one zero-checking sample in each
mode, but independent review rejected it as proof: an asynchronous IPC reply
can still lose the first paint. That experiment and its timing table are
counterevidence, not acceptance. The construction-safe repair makes the main
process the window-visibility owner. The renderer never acknowledges
initializing and notifies main directly only after ready, recovery, or
unavailable DOM has committed. That notification is semantic; it does not
claim the compositor has presented the terminal DOM.

A subsequent proposed two-requestAnimationFrame delay was also rejected by
representative evidence. Electron 39.8.6 `beginFrameSubscription` NativeImage
callbacks captured a hidden Checking frame, and the latest delivered frame at
the synchronous production `showWindow` call was still Checking even after the
two callbacks. The oracle failed the normal delayed-default run and also
rejected a deliberately early acknowledgement. An invalidation followed by a
wait for another hidden presentation produced no callback under the production
preferences. The frontend therefore removed the disproved delay: 3 of 27
revised tests failed red, then all 27 passed with direct layout-commit
notification while retaining timeout/generation/unmount/StrictMode guards.

The next main-owned marker challenge did establish correct causal frame order:
repeated default and library-only runs captured hidden Checking, a canonical
marker frame, a later marker-free terminal frame, and only then native reveal.
Every reveal frame was model-list content with no later Checking. However, the
default renderer required roughly 1.05–1.28 seconds from semantic commit to
native show, which conflicts with the immediate-list intent and prevents that
mechanism from being accepted as the product boundary.

A temporary synchronous terminal-boot/flushSync alternative was then tested
without repository source changes by running Electron 39 with exact sandbox
preferences and no `--no-sandbox` against
`/tmp/pumas-boot-prototype.0hK9r5/run.cjs --mode=default --scenario=ready
--async-delay-ms=0`. In that case the renderer semantic
ack occurred at 640.3 milliseconds and `ready-to-show` at 691.3 milliseconds;
marker insertion resolved at 719.8 milliseconds. A hidden
`capturePage({ stayHidden: true })` did not resolve until 2019.4 milliseconds,
about 1.30 seconds after marker insertion, and the returned image did not
contain the marker: its ordinary dark UI samples were 26–31. The oracle
therefore reported `captureProof=false`, kept show count at zero, and exited
failure. Platform stopped before other cases. This is counterevidence, not a
partial acceptance of synchronous boot, flushSync, or hidden capture.

## Representative Electron Oracle Pending

The retained temporary harness previously launched fresh default and
library-only Vite production builds sequentially under Electron 39.8.6 with
the actual compiled preload, `sandbox:true`, context isolation, and no Node
integration. That sampling established environment reachability and exposed
the timing contradiction, but it does not decide FE-A8.

The cross-owner rerun must make visibility—not hidden DOM—the oracle. It must
require:

- an explicitly parsed build mode;
- delayed ready across multiple frames keeps the BrowserWindow hidden and
  makes model-list content the first visible frame in both build modes;
- repeated initializing remains hidden until the platform watchdog produces a
  committed bridge-unavailable first frame;
- startup recovery and missing preload each produce the correct focused,
  path-free first visible recovery frame;
- all nine startup and nine selection values traverse producer, compiled
  preload decoding, and frontend semantics; malformed and extra-field values
  are rejected before semantic presentation;
- signal failure reaches the platform-owned bounded native fatal/quit path;
  and every workflow has zero unexpected console/process failure or residue.

## Held Product Boundary

No other uncommitted frontend tranche is separable from XR-S1. Product startup
source remains frozen in exactly:

- `frontend/src/types/api-window.ts`;
- `frontend/src/types/api-bridge-utilities.ts`;
- `frontend/src/hooks/useLauncherRootRecovery.tsx` and its test;
- `frontend/src/components/LauncherRootRecoveryView.tsx` and its test;
- `frontend/src/hooks/useAppWindowActions.ts` and its test; and
- `frontend/src/index.tsx`.

Milestones 0 through 3 are already accepted in repository history. XR-S1 is
not commit-eligible until a re-planned owner satisfies both first-frame truth
and immediate model-list visibility.

## Supporting Gates

- Handshake red: provider suite failed 5 of 23 tests before the notification,
  timeout subscription, and terminal latch existed.
- Initial handshake green: provider suite passed 24 of 24 tests.
- Compositor-contract correction red: 3 of 27 provider tests failed against
  the obsolete two-frame delay.
- Corrected renderer green: provider suite passes 27 of 27 tests; notification
  is direct semantic commit and main owns actual in-frame presentation proof.
- The combined provider, recovery-view, and window-action suite passes 34 of
  34 tests; frontend lint and full TypeScript checking pass.
- The full frontend suite passes 111 files and 504 tests. Fresh default and
  library-only production builds pass in 3.40 and 2.48 seconds.
- Composed conformance, first-visible runtime evidence, and final plan checking
  remain required after the corrected platform producer freezes.

## Claim Limit

This evidence remains incomplete and does not yet decide FE-A8. Even after the
cross-owner rerun, it will not prove packaged installer execution, every
supported host, general startup performance, M4 catalog decoding, or M5
variant workflows. The temporary harness is deleted after decisive evidence is
captured; M5 retains ownership of the admitted permanent renderer runner.
