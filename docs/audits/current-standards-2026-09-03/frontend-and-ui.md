# Focused Audit: Frontend and UI

## Scope and Result

This pass reviewed `frontend/**`, its package/configuration, tests, CI surface,
and frontend documentation. Result: **strong static and unit-test foundations,
with high-priority runtime-contract, cached-state, async, and accessibility
gaps**.

No representative Electron/browser session was used, so visual, focus-order,
and assistive-technology claims remain evidence gaps.

## Findings

### F-01 — IPC response values cross the boundary without runtime decoding

**Severity:** High — enforceable contracts and TypeScript boundary violation.

`frontend/src/api/adapter.ts:87-111` creates the bridge proxy from a type
assertion and forwards results. Domain wrappers such as
`frontend/src/api/models.ts:31-48` and `frontend/src/api/import.ts:39-74` also
return independently produced values without decoding them from `unknown`.

Create one canonical operation-schema owner, retain `unknown` until decoding,
and test invalid as well as valid producer/consumer payloads.

### F-02 — Startup snapshot state has no freshness or degradation contract

**Severity:** High — enforceable authoritative-projection and degraded-outcome
violation.

`frontend/src/utils/modelLibrarySnapshot.ts:3-7,121-152` stores only model
groups. `frontend/src/hooks/useModels.ts:41-92,285-295` can initialize from that
snapshot and leave it in place when the backend is absent, returns failure, or
throws. The model-list state itself cannot distinguish fresh backend state from
an old projection. `NetworkStatusBanner` can generically report cached data
(`frontend/src/components/NetworkStatusBanner.tsx:33-51`), but it does not
establish this snapshot's revision or freshness.

Store revision/timestamp/provenance and expose a typed
`fresh | cached | degraded` outcome with visible recovery behavior.

### F-03 — Installation polling permits overlapping stale completions

**Severity:** High — enforceable TypeScript async/current-invocation violation.

`frontend/src/hooks/useInstallationProgress.ts:100-131` applies every async
completion, while `:134-152` starts the fetch on a one-second interval. Cleanup
stops future ticks but does not cancel or generation-check in-flight requests.

Serialize the loop or move it to events, give each invocation an identity or
abort signal, and test overlap, dependency changes, completion, and unmount.

### F-04 — Dialog and popup semantics are inconsistent

**Severity:** High — enforceable accessibility violation.

- `frontend/src/components/ModelImportDialog.tsx:65-85` lacks owned dialog
  semantics, naming, Escape behavior, focus entry/trap, and restoration.
- `frontend/src/components/HuggingFaceAuthDialog.tsx:113-150` has the same gaps
  and an unlabeled icon close control.
- `frontend/src/components/VersionSelectorTrigger.tsx:88-96` and
  `VersionSelectorDropdown.tsx:111-149` do not establish a complete popup
  role/state/keyboard contract.

Build a shared dialog primitive and choose one explicit disclosure, popover, or
menu interaction contract for the version selector based on its nested actions.
The primitives should own the applicable semantics, focus, keyboard, and
dismissal behavior.

### F-05 — Progress and dynamic outcomes are visual-only

**Severity:** Medium — enforceable accessibility/documentation violation.

Progress bars and completion/failure messages in
`frontend/src/components/ProgressDetailsView.tsx:77-120,206-253` lack progress,
status, or alert semantics. No global reduced-motion implementation was found,
while `frontend/README.md:282-292` claims reduced-motion support.

Add reusable progress/status announcement behavior and either implement the
documented reduced-motion policy or correct the claim.

### F-06 — The error-handling gate is red and has a weak oracle

**Severity:** Medium — enforceable tooling-governance violation.

`frontend/package.json:17-20` includes `check:errors` in `precommit`, but the
command reports 31 findings. `.pre-commit-config.yaml:6-34` and
`.github/workflows/build.yml:147-162` do not schedule it. Its regex accepts a
catch based on `instanceof` appearing within ten lines
(`frontend/scripts/check-error-handling.js:38-48`), which does not prove a
correct boundary error contract.

Audit the 31 locations semantically, replace the regex with type-aware rules or
retire it, and schedule only a gate with a named, independently decidable claim.

### F-07 — Supported frontend variants lack representative evidence

**Severity:** Medium — enforceable library-profile and verification gap.

`frontend/package.json:8-10` advertises a library-only build and
`frontend/vite.config.ts:24-39` changes aliases and entrypoints for it. Vitest
always uses the default/plugin aliases in jsdom
(`frontend/vitest.config.ts:5-13,27-32`), while CI builds only the default
variant (`.github/workflows/build.yml:156-162`).

Build and smoke both variants in CI, inspect library-only artifacts for absent
plugin code/contracts, and add a small representative Electron/browser suite.

### F-08 — Fixed line counts are being used as design authority

**Severity:** Medium — enforceable standards-policy violation, not a finding
that current files are necessarily badly designed.

`frontend/scripts/check-file-size.js:3-58`, `frontend/eslint.config.js:83-100`,
and `frontend/README.md:331-343` treat 300 lines as a modularity decision. The
current Core and Architecture standards explicitly make counts diagnostic
only.

Retire the blocking count gate or make it advisory. Review modules based on
ownership, cohesion, interface depth, change locality, and testability.

## Strengths to Preserve

- Strict compiler settings, including nullability, unchecked-index, unused, and
  implicit-return checks (`frontend/tsconfig.json:38-55`).
- Type-aware strict ESLint and `jsx-a11y`
  (`frontend/eslint.config.js:11-26,133-135`).
- Good model-search stale-result and timer cleanup
  (`frontend/src/hooks/useModels.ts:163-176,200-283`).
- Runtime decoding for persisted model snapshots
  (`frontend/src/utils/modelLibrarySnapshot.ts:46-107`).
- Build-time plugin separation through aliases
  (`frontend/vite.config.ts:5-40`).
- Passing frontend evidence: ESLint, TypeScript, the current size checker, and
  441 Vitest tests across 102 files.

## Next Focused Audits

1. Renderer/Electron/Rust response and event decoder inventory.
2. Polling, subscription, timer, and stale-completion lifecycle inventory.
3. Keyboard, focus, status announcement, and reduced-motion audit in Electron.
4. Default versus library-only artifact and feature verification.
5. Frontend verification-gate ownership and error-contract cleanup.
