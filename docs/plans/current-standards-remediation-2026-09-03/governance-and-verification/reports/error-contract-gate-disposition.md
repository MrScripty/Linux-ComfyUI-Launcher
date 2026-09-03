# Error-Contract Gate Disposition

**Date:** 2026-09-03

**Plan:** [Governance and verification](../plan.md)

**Slice:** Milestone 2 candidate classification

## Decision

Remove `check:errors` and its only package-command registration. Do not replace
it with another generic source checker.

The command has no adequate error-contract oracle and no marginal deciding
value:

- its `console.*` regex duplicates ESLint's `no-console` AST rule;
- its `throw new Error` regex duplicates the ESLint
  `no-restricted-syntax` AST rule but ignores that rule's reviewable exception;
- its catch rule treats `instanceof` text within ten lines as proof, although a
  catch may safely retain an `unknown`, delegate classification to an owning
  helper, or project a contract-specific failure without branching on a JavaScript
  error class; and
- it is red on 31 accepted source locations, is absent from the repository
  pre-commit hooks and CI, and is reachable only through `check:errors` or the
  otherwise unregistered frontend `precommit` composition.

Retain strict TypeScript, type-aware ESLint, the two existing AST rules, and
focused behavior tests. Those mechanisms prove only their named static or
behavior claims; they do not become a universal error-contract proof.

## Candidate Classification

The candidate identities below are function/site identities. Line numbers are
the checker output at the slice baseline and are diagnostic only.

| ID | Candidate | Reachable failure and required behavior | Contract owner and existing proof | Disposition |
| --- | --- | --- | --- | --- |
| EC-01 | `src/index.tsx:9`, missing root element | Desktop renderer cannot mount and must terminate with a causal local diagnostic. | Renderer composition root; ESLint's AST rule owns generic-throw policy and the source has a narrow reviewed fatal-invariant exception. | Remove regex report. It conflicts with the authoritative lint exception and finds no separate defect. |
| EC-02 | `usePlugins.checkStatus` | Plugin status request can reject; current behavior logs and retains the last Boolean status. | Plugin-status projection; strict TypeScript proves the caught value is not used unsafely. Async overlap is already routed as FE-I09; no rejection behavior test exists. | Remove regex report. Route stale/unavailable projection separately; `instanceof` would not repair it. |
| EC-03 | `useModelImportPicker.openImportPicker` | Native picker request can reject; the dialog stays closed and only a log is emitted. | Import interaction owner; success and unavailable-bridge tests exist, but rejection/user-feedback evidence does not. | Remove regex report and route the missing user-visible unavailable outcome to frontend. |
| EC-04 | `useModelDownloads.restoreDownloads` | Startup snapshot request can reject; current state remains empty or previously projected and only a warning is logged. | Download-state projection; snapshot, pushed-update, action, and resume-failure tests exist, but startup rejection/provenance evidence does not. | Remove regex report and route the degraded-state projection gap to frontend. |
| EC-05 | `useManagedProcess.startProcess` | Launch call or success callback can reject; transition clears and a generic user error is set. | Managed-process hook; `logProcessError` owns `APIError`/`Error`/unknown diagnostic classification and strict TypeScript checks the handoff. | Remove false positive. Focused thrown-rejection evidence may be added by the owning frontend change, not by a syntax gate. |
| EC-06 | `useManagedProcess.stopProcess` | Stop can reject; transition clears and a generic user error is set. | Managed-process hook and `logProcessError`; failed-response behavior is tested, while thrown rejection is not. | Remove false positive; helper delegation is valid and proximity is not an oracle. |
| EC-07 | `useManagedProcess.openLogPath` | Opening a log path can reject; the failure is diagnostic-only. | Managed-process hook and `logProcessError`; strict TypeScript checks the `unknown` handoff. | Remove false positive; no error-class branch is required at the catch site. |
| EC-08 | `useActiveModelDownload.loadSnapshot` | Initial active-download snapshot can reject; the header retains its initial or last projection and emits only a debug log. | Active-download projection; snapshot, push, empty-state, and unmount tests exist, but rejection/degraded-state evidence does not. | Remove regex report and route the unavailable/stale projection gap to frontend. |
| EC-09 | `VersionSelector.handleVersionSwitch` | Version switch can reject; the operation ends and emits a typed diagnostic. | `reportVersionSwitchError` owns `APIError`/`Error`/unknown classification; strict TypeScript proves the `unknown` handoff. | Remove false positive; classification is deliberately delegated. |
| EC-10 | `VersionSelector.handleOpenActiveInstall` | Opening the installation path can reject; the success indicator is not set and a typed diagnostic is emitted. | `reportOpenActiveInstallError` owns classification; strict TypeScript checks the handoff. | Remove false positive; the regex cannot follow the helper Interface. |
| EC-11 | `startRemoteModelDownload` | Starting a remote download can reject; an error is recorded and authentication opens for the typed auth case. | `reportRemoteDownloadError` owns classification; focused tests prove a thrown `APIError` produces the error and auth action. | Remove false positive; behavior evidence is stronger and the helper contains the class knowledge. |
| EC-12 | `MigrationReportsPanel.fetchReports` | Listing reports can reject; loading ends and an explicit user error replaces success. | Migration-report panel; strict TypeScript plus the explicit state transition. Existing tests cover normal rendering, not thrown rejection. | Remove false positive; add focused rejection evidence only if this behavior changes. |
| EC-13 | `MigrationReportsPanel.generateDryRun` | Dry-run generation can reject; the operation ends with an explicit user error. | Migration-report panel; strict TypeScript and explicit state transition. | Remove false positive; error-class discrimination is not required for the selected generic presentation. |
| EC-14 | `MigrationReportsPanel.executeMigration` | Execution can reject; the operation ends with an explicit user error. | Migration-report panel; strict TypeScript and explicit state transition. | Remove false positive. |
| EC-15 | `MigrationReportsPanel.deleteReport` | Delete can reject; the operation ends with an explicit user error. | Migration-report panel; strict TypeScript and explicit state transition. | Remove false positive. |
| EC-16 | `MigrationReportsPanel.pruneReports` | Prune can reject; the operation ends with an explicit user error. | Migration-report panel; strict TypeScript and explicit state transition. | Remove false positive. |
| EC-17 | `MigrationReportsPanel.handleOpenPath` | Opening a report can reject; the operation ends with an explicit user error. | Migration-report panel; strict TypeScript and explicit state transition; failed-result behavior is covered by the component fixture. | Remove false positive. |
| EC-18 | `LinkHealthStatus.fetchHealth` | Health fetch can reject; with no prior value the panel disappears, otherwise stale health remains without an unavailable state. | Link-health projection; success and unexpected-status tests exist, but rejection/provenance evidence does not. | Remove regex report and route the hidden unavailable/stale outcome to frontend. |
| EC-19 | `LinkHealthStatus.handleCleanBroken` | Cleanup can reject; the operation ends with an explicit `Error cleaning broken links` result. | Link-health interaction; strict TypeScript and explicit state transition. | Remove false positive. |
| EC-20 | `LinkHealthStatus.handleRemoveOrphans` | Removal can reject; the operation ends with an explicit `Error removing orphaned links` result. | Link-health interaction; strict TypeScript and explicit state transition. | Remove false positive. |
| EC-21 | `InstallDialog.handleInstall` | Installation start/refresh can reject; cancellation remains distinct, while failure sets the tag and message. | Install dialog; `isInstallationCancellation`, `reportInstallationError`, and `getErrorMessage` own classification/projection. | Remove false positive; helper delegation and distinct UI state are the relevant contracts. |
| EC-22 | `InstallDialog.cancelInstallation` | Cancellation can reject; current code records only a typed diagnostic and exposes no cancellation-failed state. | Install interaction; `reportCancelError` owns diagnostic classification. | Remove regex report and route the missing user-visible failure outcome to the active frontend installation owner. |
| EC-23 | `HuggingFaceAuthDialog.fetchAuthStatus` | Status request can reject; loading ends and a generic user error is displayed. | Authentication dialog; strict TypeScript and explicit UI error transition. | Remove false positive; logging the `unknown` needs no class branch. |
| EC-24 | `useShardedSetDetection.detectShards` | Detection can reject; entries remain ungrouped or retain a prior grouping and only a log is emitted. | Model-import projection; ordinary grouping tests exist, but rejection/invalidated-state evidence does not. | Remove regex report and route the silent unavailable/stale grouping outcome to frontend. |
| EC-25 | `runMetadataLookup`, early embedded metadata read | Embedded extraction can reject; the code continues to the Hugging Face lookup without recording embedded-metadata failure. | Model-import metadata workflow; fallback authority and degraded-state presentation are not documented or tested. | Remove regex report; route the missing fallback/degradation authority to frontend rather than assuming a class guard repairs it. |
| EC-26 | `runMetadataLookup`, outer item lookup | Validation or metadata lookup can reject; that item becomes `metadataStatus: error` and progress advances. | Model-import metadata workflow; invalid-file error/progress behavior is tested and strict TypeScript checks caught-value use. | Remove false positive; the selected state transition is independent of JavaScript error class. |
| EC-27 | `TorchServerConfigSection.fetchConfig` | Config fetch can reject; default or previous configuration remains visible without an unavailable marker. | Torch plugin UI projection; no focused rejection evidence was found. | Remove regex report and route stale/default projection to frontend/plugin follow-up. |
| EC-28 | `TorchModelSlotsSection.fetchTorchState` | Slot/device refresh can reject; previous or empty state remains without an unavailable marker. | Torch runtime projection; async overlap is already FE-I09 and preview-limit tests do not prove failure behavior. | Remove regex report and route unavailable/stale projection to frontend/plugin follow-up. |
| EC-29 | `OllamaModelSection.fetchOllamaState` | Model/runtime refresh can reject; previous or empty state remains without an unavailable marker. | Ollama runtime projection; preview-limit tests do not prove failure behavior. | Remove regex report and route unavailable/stale projection to frontend/plugin follow-up. |
| EC-30 | `ModelSelectorSection.fetchLoadedModels` | Plugin model refresh can reject; previous or empty state remains without an unavailable marker. | Generic plugin model projection; async overlap is already FE-I09 and no focused rejection evidence was found. | Remove regex report and route unavailable/stale projection to frontend/plugin follow-up. |
| EC-31 | `safeAPICall` | An arbitrary call can reject and be converted to caller-supplied fallback success. There is no production caller; only its own fallback test and documentation example are reachable. | Renderer API adapter; FE-I01/Milestone 4 already owns removal of fallback/pass-through helpers that erase outcome distinctions. | Remove regex report. Keep the real dead-helper/fake-success finding routed to frontend Milestone 4. |

## Real-Defect Handoff

The checker did not establish the defects below; manual contract review during
classification did. They are outside this milestone's write set and are
recorded in [issues.md](../issues.md):

- model import and download snapshot failures can become silent empty, stale,
  or ungrouped projections;
- plugin/runtime refresh failures can leave stale or default values without an
  unavailable outcome;
- picker and installation-cancellation rejection can lack user-visible failure
  state; and
- the unused `safeAPICall` helper intentionally converts failure into arbitrary
  fallback success and is already owned for deletion by frontend Milestone 4.

None justifies retaining a proximity regex. Each needs behavior evidence at
its frontend owner and, where applicable, the accepted decoded bridge
Interface.

## Stopping Condition

All 31 reports have a reachable-failure, owner/evidence, and disposition. The
three intended checker invariants have also been dispositioned:

| Intended invariant | Disposition | Adequate retained mechanism |
| --- | --- | --- |
| Production source does not call `console.*` directly | Remove duplicate regex | ESLint `no-console` AST rule |
| Generic `Error` construction requires an owned exception | Remove duplicate regex | ESLint `no-restricted-syntax` AST rule and reviewable inline exception |
| Catch handling preserves error meaning | Remove invalid universal rule | Strict TypeScript/type-aware ESLint plus focused behavior/contract evidence selected by each owner |

No replacement tooling is admitted. Source mutation may proceed by deleting
the checker and registrations after the serialized M2 review releases it.
