# Frontend Async Owner Inventory

**Date:** 2026-09-03

**Plan:** [Frontend and UI standards remediation](../plan.md)

**Slice:** M0-S1

## Scope And Method

This inventory bounds the production installation-progress path and the timer
owners under `frontend/src/hooks/` and `frontend/src/components/`. It inspected
all production references to `useInstallationProgress`,
`useInstallationManager`, `get_installation_progress`,
`fetchInstallationProgress`, `installationProgress`, `installingTag`,
`setInterval`, and `setTimeout` at revision `45310578`.

The deciding distinction is ownership, not timer syntax. A timer that only
changes local presentation state is not another installation owner. A Module
that requests installation progress or supplies the current installation tag
is part of the installation synchronization path.

## Installation Synchronization Path

The production path is:

`useSelectedAppVersions` -> three app-scoped `useVersions` instances ->
`useInstallationManager` -> desktop `get_installation_progress`.

For the selected app, `VersionManagementPanel` passes manager state and a
manual refresh action to `InstallDialog`. `InstallDialog` then invokes
`useInstallationProgress`, which currently retains a second direct desktop
polling implementation when that manual action is absent. The production
caller supplies the action, so the second implementation is unreachable in
production but remains executable in tests and part of the presentation
Interface. The dialog also calls `cancel_installation` directly without its
app identity, so a non-Ollama dialog can request cancellation for the default
app instead of its current owner.

`useVersions` also owns `localInstallingTag`, populated through
`useVersionFetching` and `useAvailableVersionState` from a release row's
`installing` flag. It merges that value with the manager-owned tag. This is a
second current-installation identity owner even though it does not fetch
progress itself.

## Consumer Dispositions

| Owner / consumer | Current role | Disposition | Reason |
| --- | --- | --- | --- |
| `useInstallationManager.ts` | Starts an 800 ms `setInterval`, requests progress, normalizes it, refreshes versions, and owns most installation actions | `migrate` | Deepen this as the sole installation synchronization Module. Hide serialized self-scheduling, app/tag generation, completion classification, active-release discovery, and app-scoped cancellation behind its result/actions Interface. |
| `useInstallationManager.test.ts` | Covers ordinary active, completed, failed, request-failed, and not-yet-initialized results | `migrate` | Replace interval-shape assertions with Interface evidence for maximum one request in flight, app/tag supersession, disable/unmount, success, failure, and cancellation. |
| `useInstallationProgress.ts` | Projects external progress into dialog state, but also owns a second one-second direct progress poll | `delete` / `migrate` | Delete its desktop request and polling implementation. Retain only dialog presentation state such as sticky failure and cancellation notice, with external progress as its input. |
| `useInstallationProgress.test.ts` | Makes the otherwise unreachable local polling branch executable | `delete` / `migrate` | Delete fallback-poll tests and prove the narrowed presentation Interface instead. |
| `InstallDialog.tsx` | Mirrors the installing tag, invokes the presentation hook, manually refetches after `onInstallVersion`, and calls unscoped desktop cancellation directly | `migrate` | Consume manager-owned progress/cancellation only and remove the manual refresh prop/call so callers do not coordinate the manager's hidden scheduling. |
| `InstallDialog.test.tsx` | Covers current visible dialog behavior | `migrate` | Add evidence that externally supplied success, failure, and cancellation remain distinct after narrowing. |
| `VersionManagementPanel.tsx` | Sole production `InstallDialog` caller; passes manager progress and `fetchInstallationProgress` | `migrate` | Replace the superseded manual-refresh prop with manager-owned cancellation. This caller was outside the original M0 write set, so the plan now names it explicitly. |
| `useVersions.ts` | Composes fetching and installation Modules and merges manager tag with `localInstallingTag` | `migrate` | Remove the second tag state and project manager-owned cancellation. The manager can derive an active-install hint from its existing `availableVersions` input while remaining the sole state owner. |
| `useVersions.test.ts` | Proves the two-owner merge | `migrate` | Delete the merge expectation and prove that only manager state is projected. |
| `useVersionFetching.ts` and test | Transports `onInstallingTagUpdate` into available-version fetching | `delete` | Remove the now-unused callback from the Interface rather than retaining a compatibility path with no production consumer. These files were added to the exact M0 write set. |
| `useAvailableVersionState.ts` and test | Finds a release row marked `installing` and calls the tag callback; separately polls GitHub cache state | `delete` / `follow-up` | Delete only the installation-tag callback path in M0. Its separate cache-status polling risk is routed below and does not become installation behavior. |
| `appVersionState.ts` | Supplies the typed unsupported version-state Adapter | `migrate` | Add the no-op cancellation action required by the manager-owned Interface. This file was added to the exact M0 write set. |
| `useSelectedAppVersions.ts` | Composes one manager per supported app and selects the visible projection | `already safe` | It does not start progress requests or merge installation identities. No edit is required for M0. |
| `Header`, `StatusFooter`, `VersionSelector`, and their state helpers | Render manager-owned progress | `already safe` | These are read-only presentation consumers. Progress semantics are owned by Milestone 3. |
| `api/versions.ts` | Contains an unused `VersionsAPI.getInstallationProgress` wrapper | `follow-up` | It is not on the production installation path. The renderer contract inventory in Milestone 4 owns API-wrapper deletion decisions. |
| `api-bridge-runtime.ts` | Declares the renderer-side desktop method | `external owner` | The platform/Rust contract chain owns the accepted method representation; Milestone 4 consumes it. |

## Target Module And Interface

`useInstallationManager` is the single Module at the renderer-lifecycle to
desktop-installation Seam. Callers provide app identity, availability,
release metadata, and version-refresh behavior. Callers receive installation
state and actions; they do not learn timer ownership, request generation,
coalescing, or completion ordering.

The production desktop bridge and the controlled deferred test bridge are the
two Adapters at this Seam. Tests exercise the same hook Interface as callers.
The deletion test is positive: removing the manager would redistribute
serialization, identity, normalization, completion refresh, and error
classification across `useVersions` and the dialog. Deleting the dialog-local
poll and installation-tag callback instead removes duplicated machinery.

## Unrelated Timer Dispositions

| Timer owner | Classification | Disposition |
| --- | --- | --- |
| `useAvailableVersionState` cache-status interval | Async polling can overlap and a late result can update a superseded app lifecycle | `follow-up issue FE-I09`; different state and acceptance path from installation progress |
| `usePlugins` app-status interval | Async polling can overlap and a late result can update a superseded app lifecycle | `follow-up issue FE-I09`; plugin lifecycle owner |
| `StatsSection` statistics interval | Async polling can overlap and a late result can update a superseded app lifecycle | `follow-up issue FE-I09`; runtime statistics owner |
| `ModelSelectorSection` loaded-model interval | Async polling can overlap and a late result can update a superseded app lifecycle | `follow-up issue FE-I09`; plugin model-selection owner |
| `TorchModelSlotsSection` slot/device interval | Async polling can overlap and a late result can update a superseded runtime lifecycle | `follow-up issue FE-I09`; Torch runtime owner |
| `AppIndicator` spinner/error intervals | Synchronous presentation animation with cleanup | `already safe` for async ownership; reduced-motion behavior remains Milestone 3 |
| `useModels` debounced search | Async completion is guarded by a request sequence and cleanup invalidates pending work | `already safe` for this inventory; model projection remains Milestone 4 |
| `useRemoteModelSearch` debounce | Async completion is guarded by active state and generation | `already safe` |
| model-library/runtime-profile subscription debounces | Latest-event timers are cleared on replacement and unmount | `already safe` |
| API-readiness and delayed UI-notice timeouts | Self-scheduling or one-shot presentation work with local cleanup where currently claimed | `already safe` for installation ownership; no broader timer-safety claim is made |

The unrelated async interval owners share a mechanical risk but not the
installation invariant, write set, or acceptance path. Expanding M0 to them
would turn a bounded installation repair into a cross-domain lifecycle
program. FE-I09 preserves the evidence and routes a focused follow-up.

## Stopping Condition

Every production installation-progress requester, tag owner, caller, and
renderer consumer has one disposition above. All production timer owners in
the searched hooks/components population are either covered by the grouped
dispositions or routed to FE-I09. No additional production caller of
`InstallDialog` or `useInstallationProgress` was found.

The program integration owner accepted RUST-A1 and released M0-S1 source work
after this inventory was recorded.
