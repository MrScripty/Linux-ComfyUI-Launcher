# Plan: Frontend and UI Standards Remediation

**Plan status:** `Active`

**Current phase:** Milestones 0 through 3 are accepted. The user's 2026-09-05
decision accepts the main-owned causal marker barrier's roughly one-second
cost and admits bounded XR-S1/M4 integration with the candidate catalog/search
producer and generated platform decoder. Renderer-frame timing and hidden
capture remain rejected reveal authorities. This is source admission, not
acceptance of the unfinished M4 milestone.

**Next slice:** Complete the coordinated producer/preload/renderer conformance
matrix, retaining the approved marker barrier. Preserve FE-I10/FE-I11/FE-I14
consumer classifications: catalog/search use the closed projection, scan/import
keep their distinct raw record contract, cached rows are root-scoped and
display-only, and downloads without exact association remain orphan rows.
No cached path, ticket, activity, or action authority is admitted.

**Acceptance status:** `partial`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

**Reports:** Planned report artifacts are indexed in the [execution ledger](execution-ledger.md#reports).

**Audit source:** [Frontend and UI audit](../../../audits/current-standards-2026-09-03/frontend-and-ui.md)

## Objective

Make the renderer consume proof-bearing desktop contracts, preserve fast model
startup without misrepresenting cached state, keep asynchronous installation
state current, provide consistent keyboard/focus/status/motion behavior, and
prove both supported renderer variants in a representative runtime.

This plan preserves the existing strict TypeScript, type-aware lint, snapshot
decoder, model-search stale-result guard, and build-time plugin aliases. It
changes only owners whose current behavior cannot support the audited claims.

## Baseline

- Audit code baseline: `a33c8c0efa7cd8783c7deeac9e608db205290d43`.
- Planning code baseline: `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`.
- Audit standards baseline: `52b096ded9c53afd439a3cf0efc4cc85252da570`.
- Planning standards baseline: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`.
- Audit evidence at its baseline: frontend ESLint, TypeScript, the then-current
  size check, and 441 Vitest tests in 102 files passed. `check:errors` reported
  31 candidates; the sibling governance plan owns that mechanism. No
  representative browser/Electron workflow or production build was run in the
  audit, so neither is baseline acceptance evidence.
- The uncommitted `docs/plans/**` tree is shared program-planning work. This
  plan owns only its `frontend-and-ui/` directory.

## Objective Acceptance

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| FE-A1 | Every desktop RPC response or event used by renderer code arrives through the accepted platform decoder Interface; invalid, unsupported, unavailable, and operation-failure outcomes remain distinct and cannot become domain values through assertions or fallback substitution. | `contract` | `simulated` (generated-decoder test Adapter and malformed payload corpus) | `automated` | `pending` | Pending Milestone 4; producer proof remains linked to the Rust and platform plans |
| FE-A2 | A valid saved model list renders immediately with visible cached provenance and real or explicitly unknown age; failed refresh remains visibly degraded and retryable; successful refresh replaces it with fresh authoritative state. | `user-workflow` | `representative` (built renderer in its supported desktop browser runtime) | `automated` | `pending` | Pending Milestones 1 and 4 |
| FE-A3 | Installation progress has at most one request in flight for its current owner, begins only after the backend admits a requested lifecycle, and completions superseded by app/tag changes or unmount cannot mutate current state; success, failure, and cancellation remain distinguishable. | `integration` | `simulated` (controlled deferred request Adapter and fake clock) | `automated` | `satisfied` | [M0-S1 evidence](execution-ledger.md#2026-09-03--m0-s1-installation-progress-owner-accepted) plus accepted [PRG-I12 repair evidence](execution-ledger.md#2026-09-03--m3-s3a-status-reachability-terminal-retention-and-admission-order-accepted) |
| FE-A4 | Affected modal and popup workflows expose names and state programmatically, support pointer and keyboard operation, contain and restore focus where modal, dismiss predictably, and preserve nested-dialog focus order. | `user-workflow` | `representative` (built renderer with keyboard and accessibility-tree observations) | `automated` | `satisfied` | [M2-S4 Chromium evidence](execution-ledger.md#2026-09-03--m2-s4-representative-chromium-evidence-accepted) |
| FE-A5 | Installation progress and terminal outcomes are programmatically announced without duplicate noise, and the operating-system reduced-motion preference suppresses nonessential CSS and Framer Motion movement. | `user-workflow` | `representative` (built renderer with accessibility-tree and media-preference control) | `automated` | `satisfied` | [M3-S3b real-entry evidence](execution-ledger.md#2026-09-03--m3-s3b-popover-motion-and-terminal-semantics-accepted) |
| FE-A6 | Both frontend build modes start through their real entry point; the default renderer exposes supported inference-plugin UI, while the library-only renderer omits that UI and still completes the core model-library workflow. | `user-workflow` | `representative` (built default and library-only renderers in the supported desktop browser runtime) | `automated` | `pending` | Pending Milestone 5; packaged-artifact claims remain in the platform plan |
| FE-A7 | Frontend behavior documentation describes only the accepted cache provenance, keyboard/focus, reduced-motion, and variant behavior, and routes verification policy to the governance owner. | `focused` | `not-applicable` | `manual` | `pending` | Pending Milestones 3 through 5 |
| FE-A8 | In Electron, one launcher-root owner prevents backend-consuming renderer content from mounting before a decoded ready state, presents every startup/selection recovery outcome without path disclosure or unsafe retry, and gives both normal-ready entries a first visible model-list frame without a painted checking frame; browser mode remains explicitly not applicable. | `integration` | `representative` (production builds and preload in sandboxed Electron) | `automated` | `pending` | [XR-S1 evidence](reports/launcher-root-recovery-consumer-evidence.md); main-owned in-frame compositor proof and composed conformance remain active |

## Scope

### In Scope

- Renderer consumption of the canonical decoded RPC/result Interface.
- The model-list cached projection, its provenance/freshness/degraded state,
  its presentation, and recovery behavior.
- Installation-progress polling ownership and hook lifecycle.
- Shared modal and popup interaction Modules for current affected consumers.
- Progress/status announcement and reduced-motion behavior.
- Frontend-owned representative workflow machinery for default and
  library-only built renderers.
- Frontend behavior documentation changed by those outcomes.

### Out Of Scope

- Defining Rust RPC DTOs, the canonical error taxonomy, wire schemas, or
  compatibility policy; the Rust focused plan owns them.
- Electron IPC handler validation, generated decoder implementation, preload
  exposure, or process-boundary transport tests; the platform focused plan owns
  them.
- Packaged installer contents, plugin binary inclusion, release signing, or
  packaged-artifact inspection; the platform focused plan owns them.
- Retiring `check:errors`, fixed line/count gates, changing CI schedules, or
  defining the repository verification inventory; the
  [governance and verification plan](../governance-and-verification/plan.md)
  owns F-06 and F-08.
- A general WCAG conformance claim, every screen in the application, or an
  assistive-technology certification claim.
- Opportunistic decomposition based only on file length or complexity counts.

## Constraints And Assumptions

### Constraints

- Renderer code must not duplicate canonical DTO/error schemas or accept
  asserted producer values as proof. It consumes the platform-generated
  Interface once that dependency is accepted.
- The Rust/backend model catalog is authoritative. Browser storage is a
  disposable projection used to satisfy the product requirement that the model
  list appear immediately at startup.
- Existing version-1 snapshots have no capture time. They must be presented as
  cached with unknown age; migration must not invent a timestamp or revision.
- The producer currently exposes no catalog revision. Capture time and source
  may be recorded locally, but must not be described as producer revision.
- Underlying IPC calls are not assumed cancellable. Superseded async work must
  still be observed and classified, while its completion is prevented from
  mutating current state.
- Popup roles must follow actual interaction. Nested action collections must
  not be mislabeled as listboxes or menus merely to obtain a familiar role.
- A test command or dependency is not selected until the bounded renderer
  harness admission proves the environment and oracle it can provide.
- Governance edits to `frontend/package.json` and CI land before Milestone 5,
  or the overlapping package-script change is rebased and reviewed explicitly.

### Assumptions

- The existing build aliases remain the production mechanism for plugin
  separation unless Milestone 5 produces contradictory runtime evidence.
- The current `VersionManagementPanel` is the sole production installation
  dialog caller and already supplies manager-owned progress. The local fallback
  has no reachable production consumer at this baseline.
- A single built-renderer harness can decide focus, semantics, cache
  provenance, motion preference, and mode visibility; Milestone 1 must disprove
  this before admitting a second permanent harness.

## Binding Decisions

| Decision | Module / Interface / Seam | Production and test Adapters | Evidence / consequence |
| --- | --- | --- | --- |
| Deepen `useInstallationManager` as the sole installation synchronization Module for the resolved app/tag identity. Its result/actions Interface owns current progress and every installation lifecycle action, including app-scoped cancellation; serialized polling and generation ownership stay hidden. | Seam between renderer lifecycle and desktop installation operations. | Production Adapter is the accepted desktop API; tests use controlled deferred operations and a fake clock through the hook Interface. | Deletes the unused dialog-local polling owner and unscoped dialog cancellation instead of coordinating lifecycle across callers. |
| Deepen `useModels` as the model-library projection Module. Its small renderer Interface adds provenance/freshness/degraded outcome and retry beside current model groups/actions. | Seam between authoritative catalog refresh, disposable local projection, and presentation. | Production Adapter is the accepted decoded models operation plus browser storage; tests use valid/legacy/malformed snapshots and controlled refresh outcomes. | One owner decides immediate display, freshness, replacement, and recovery. No second store is added. |
| Consume the platform-generated decoded desktop Interface. Keep a frontend API wrapper only when it adds renderer-domain composition or recovery; delete fallback/pass-through helpers that erase outcome distinctions. | Existing Electron/preload process boundary; this plan owns only its renderer consumer side. | Platform owns the production decoder Adapter. Frontend tests use its proof-bearing test Adapter, not hand-authored parallel schemas. | Prevents schema drift and removes `{ } as DesktopBridgeAPI`/arbitrary fallback authority from renderer paths. |
| Add one `ModalDialog` Module whose Interface owns accessible role/name, initial focus, containment, Escape/backdrop policy, nested restoration, and cleanup. Feature content owns titles, messages, and actions. | Seam between modal lifecycle policy and feature content. | No external production Adapter is needed; component tests supply focusable content, and representative tests drive the public component behavior. | Replaces partial duplicated focus hooks/frames. The Module is deeper than a styling wrapper. |
| Add one `Popover` Module whose Interface owns trigger relationship/state, outside/Escape dismissal, focus entry/return, and cleanup. Feature content selects semantics appropriate to its actual actions. | Seam between non-modal popup lifecycle and selector/search/download content. | No external production Adapter is needed; tests drive trigger/content behavior through the Interface. | Does not introduce a generic menu/listbox abstraction or hide feature semantics. |
| Implement progress/status semantics in the existing progress view and motion preference at the frontend composition root/CSS boundary. | Existing presentation owners; no shared Module until a second consumer needs the full lifecycle. | Representative harness controls progress and media preference. | Avoids a hypothetical announcement framework for a single current lifecycle. |
| Use the installed Electron runtime and Chromium DevTools Protocol behind one package command; a frontend runner owns temporary builds/process cleanup, an Electron main Adapter owns browser input/accessibility/media observation, and deterministic fixtures own only external operations. | Seam between built renderer behavior and acceptance evidence. | Real built renderer and production preload are the production subject; IPC fixtures replace only external operation providers. | [Admission evidence](reports/renderer-harness-admission.md) reached the intended modal failure without a new dependency; jsdom and release smoke cannot decide the same outcomes. |
| Deepen one `LauncherRootRecoveryProvider` at the renderer composition root as the sole owner of startup admission and library-root selection presentation. | Seam between the platform-decoded launcher-root Interface and backend-consuming renderer trees. | Production Adapter is the accepted sandboxed preload API; tests supply controlled startup/selection promises, and the representative oracle supplies exact IPC fixtures. | Browser mode renders children as not applicable. Electron withholds visibility acknowledgement through initializing, then acknowledges exactly once after ready, recovery, or unavailable presentation commits; the main process owns actual window visibility and its bounded native fallback. |

## Dependencies And Ownership

| Dependency | Provider | Consumer milestone | Ready condition |
| --- | --- | --- | --- |
| Canonical RPC DTO/error taxonomy and compatibility policy | Rust focused remediation plan | Milestone 4 | Producer contract is accepted and versioned; invalid/unsupported/unavailable outcomes are named. |
| Electron/generated response and event decoders | Platform focused remediation plan | Milestone 4 | Renderer-visible Interface is proof-bearing and has invalid-payload tests. |
| Packaged variant/artifact proof | Platform focused remediation plan | Program acceptance only | Platform plan owns installer/package inspection; FE-A6 does not claim it. |
| Count/error gate and CI schedule cleanup | [Governance and verification plan](../governance-and-verification/plan.md) | Milestone 5 | Its `frontend/package.json`/CI edits are integrated before frontend adds the selected package-local representative command. |
| Renderer harness selection | Milestone 1 of this plan | Milestones 2, 3, 4, and 5 | Admission report identifies one capable tool, cleanup protocol, duration, and independently observable oracle. |

Integration order is Milestone 0, Milestone 1, Milestones 2–3, then Milestone
4 when the Rust/platform contracts are ready, and Milestone 5 after governance
package-script changes. Milestones 2 and 3 may be reviewed independently after
Milestone 1, but their shared UI primitive/export files must be serialized.

## Evidence And Oracle Plan

| Claim | Deciding oracle | Independent authority | Deliberately unsupported by that evidence | Intended negative failure |
| --- | --- | --- | --- | --- |
| FE-A1 | Generated decoder contract tests plus renderer consumer tests that inject malformed, unsupported, unavailable, and operation-failure outcomes | Accepted Rust schema/error contract and platform decoder output | Backend implementation correctness and transport delivery | Any invalid payload reaching model or presentation state fails |
| FE-A2 | Built-renderer workflow observes immediate list content, explicit cached/unknown-age state, degradation/retry, and fresh replacement | Controlled authoritative-response Adapter and storage fixture | Cross-device cache validity or producer revision | Removing the provenance indicator or allowing a failed refresh to appear fresh fails |
| FE-A3 | Hook test with deferred operations records maximum active count and state after supersession/unmount | Invocation generation and resolved app/tag identity observed outside the hook | Cancellation of non-cancellable IPC work | A second request overlaps or an old completion mutates current state |
| FE-A4 | Built-renderer keyboard workflow and accessibility-tree/focus observations | Browser/Electron focus and accessibility behavior | Screen-reader product certification | Missing name/state, escaped modal focus, broken Escape, or failed restoration fails |
| FE-A5 | Accessibility-tree outcome observations and reduced-motion media emulation against the built renderer | Browser accessibility tree and operating-system media preference | Every decorative transition in third-party content | Silent terminal state, duplicate announcement, or nonessential motion under reduce fails |
| FE-A6 | Separate production builds launched through their real entry points and driven through a core model workflow | Build mode/entry configuration and runtime-visible controls | Installer contents or absence of strings in minified bundles | Plugin UI appears in library-only mode or the core library workflow fails in either mode |
| FE-A8 | Provider tests drive the closed startup/selection state matrix; real sandboxed Electron runs the default and library-only production entries with compositor presentation-frame capture at the native reveal boundary | Accepted platform decoder contract plus browser frame and renderer-console observations | Packaged installer execution and general startup performance | Backend content mounts before ready, a typed state gains the wrong action, startup reads overlap, the native reveal can expose checking, model-list content never reaches a presented frame, or a renderer/preload error is emitted |

For new permanent renderer verification machinery, Milestone 1 must record its
claim, reachable negative failure, independent oracle, marginal value versus
Vitest/release smoke tests, expected runtime, cleanup behavior, and retention
trigger. The package-local command is scheduled by governance rather than
silently added to every local hook.

## Development Proportionality

### Admitted Investigation: Representative Renderer Harness

- **Uncertainty:** Whether the existing Electron/Chromium launch path can be
  driven to observe accessibility roles, focus, media preference, and both
  built variants, or whether one bounded browser-driving dependency is needed.
- **Decision unlocked:** Extend the existing launch tooling or admit one
  frontend-owned runner and fixture protocol.
- **Consequence of guessing:** The project could retain a test that exercises
  jsdom rather than the real browser behavior, or acquire a costly duplicate
  runtime harness.
- **Cheapest discriminating check:** Build one mode, launch it with an isolated
  profile and deterministic desktop-operation fixture, and drive one modal
  open/focus/Escape/restore workflow while reading the browser accessibility
  representation.
- **Stopping condition:** The approach observes accessible role/name/focus and
  built-entry mode behavior, terminates all child processes deterministically,
  and has a bounded measured duration; otherwise the report records a typed
  unsupported/unavailable result and the plan is revised before tool changes.

### Deferred decisions

- The package-local command name remains integration-owned until Milestone 5;
  the admitted implementation uses existing Electron/Vite/Node dependencies
  and the three-file owner split recorded in the admission report.
- The exact decoded desktop Interface remains provider-owned until the Rust and
  platform plans accept it. This plan will not draft a temporary schema.
- Existing version-1 cache retention ends only when the minimum supported
  upgrade policy excludes pre-remediation snapshots or an explicit release
  decision authorizes removal.

## Systemic Finding Audit

- **Invariant families and canonical owners:** decoded desktop values are owned
  by the Rust/platform contract chain; cached model provenance by `useModels`;
  current installation progress by `useInstallationManager`; modal lifecycle
  by `ModalDialog`; non-modal popup lifecycle by `Popover`; announcement/motion
  by their presentation/composition owners; variant behavior by the frontend
  build entry and aliases.
- **Bounded population:** all files under `frontend/src/api/`; model snapshot,
  `useModels`, and its two app composition roots; installation manager/progress
  hooks and sole dialog caller; current dialog frames and feature dialogs;
  version/search/download popups; progress view; frontend entry/CSS; default
  and library-only Vite entry/alias paths.
- **Expansion facts:** expand only when a searched renderer consumer accepts the
  same undecoded value, another production owner starts the same installation
  poll, or another overlay/progress consumer shares the full audited lifecycle.
  Record the evidence and re-plan its exact files before editing.
- **Consumer dispositions:** migrate reachable consumers to the canonical
  owner; delete unused fallback owners and duplicated focus hooks; leave
  unrelated hook timers/popups with an explicit inventory disposition rather
  than silently widening the repair.
- **Alternatives considered:** event delivery before polling; existing launch
  tooling before a new test dependency; direct semantic HTML before a generic
  component; existing bridge Interface before another API facade.
- **Evidence-backed stopping condition:** every member of each bounded
  population has `migrate`, `already safe`, `delete`, `external owner`, or
  `follow-up issue` recorded in the applicable report/ledger, and every
  acceptance claim has one deciding oracle.
- **Repaired-composition comparison:** the target deletes one polling owner and
  duplicated focus machinery, adds only two reusable interaction Modules and
  one admitted renderer harness, and does not add schemas, stores, or generic
  service layers.

## Simplicity And Ownership Review

**Applicability:** `applicable`

- Independent concepts and dimensions: decoded transport values, authoritative versus cached catalog state, current async invocation, modal policy, popup policy, announcements, motion preference, and build mode can change independently and retain separate owners.
- State, identity, value, time, policy, and mechanism: model groups are values; source/freshness and installation phase are state; app/tag and request generation are identity; snapshot capture and invocation generation own time; accessibility/variant rules are policy; storage, IPC, focus management, polling, CSS, and the harness are mechanisms hidden behind their owning Interfaces.
- Caller and composition-root knowledge: app roots select the model projection and motion policy; feature dialogs/popups provide content and actions without reimplementing focus policy; renderer consumers know decoded operation outcomes but not wire validation internals.
- Representative change paths and forced owners: changing cache freshness touches the projection Module and its notice; changing modal dismissal touches `ModalDialog` and its Interface tests; adding an RPC field starts at the provider schema/decoder and reaches only consuming renderer behavior; adding a supported mode extends the build-mode matrix and representative workflow.
- Stable Interfaces versus hidden knowledge: hook results, modal/popover props, and the platform proof-bearing API are Interfaces; snapshot envelope details, request generations, focus sentinels, wire parsing, and harness process control remain hidden knowledge.
- Independent evolution, testing, failure, and replacement: each Module is tested through its Interface; production and test Adapters meet the same external Seam; platform decoders, storage fixtures, and renderer driver can be replaced without feature components learning their internals.
- Necessary complexity and containment: only external desktop/storage/runtime Seams receive Adapters; modal and popup Modules contain real repeated lifecycle policy; status/motion stay local until reuse is evidenced; no registries, pass-through facades, or speculative abstraction are admitted.
- Deletion and cumulative machinery result: delete dialog-local polling, arbitrary fallback helpers without domain meaning, and duplicated focus lifecycle code; replace them with fewer canonical owners, while retaining one renderer harness only if it supplies evidence existing Vitest and release smoke checks cannot.

## Risks

| Risk | Control |
| --- | --- |
| Immediate cache display is mistaken for current backend truth | Unknown-age legacy migration, explicit cached/degraded presentation, and successful-refresh replacement test |
| Late installation completion mutates a new app/tag | Serialized loop plus generation/current-owner checks and deferred-promise tests |
| Shared modal/popover primitive changes feature behavior | Migrate one representative consumer first, test through the Interface, then migrate only matching consumers |
| Nested dialogs restore focus to a removed element | Stack-aware restoration with connectivity fallback, exercised in representative runtime |
| Accessibility tests pass in jsdom but fail in Chromium/Electron | Representative harness is a prerequisite for acceptance, not optional corroboration |
| Generated contract timing blocks model remediation | Milestone 4 remains dependency-gated; no local duplicate schema is introduced |
| Governance and frontend both edit package scripts | Governance integrates first; Milestone 5 rebases and owns only the package-local renderer command |
| A new harness becomes slow or flaky | Admission requires measured duration, deterministic fixtures/process cleanup, unique oracle, and a retention trigger |

## Milestones

### Milestone 0: Own Installation Progress Lifecycle

**Goal:** Establish one serialized, current installation-progress owner and
remove the unused second polling loop.

**Allowed write set:**

- `frontend/src/hooks/useInstallationManager.ts`
- `frontend/src/hooks/useInstallationManager.test.ts`
- `frontend/src/hooks/useInstallationProgress.ts`
- `frontend/src/hooks/useInstallationProgress.test.ts`
- `frontend/src/components/InstallDialog.tsx`
- `frontend/src/components/InstallDialog.test.tsx`
- `frontend/src/hooks/useVersions.ts`
- `frontend/src/hooks/useVersions.test.ts`
- `frontend/src/hooks/useSelectedAppVersions.test.ts`
- `frontend/src/hooks/useVersionFetching.ts`
- `frontend/src/hooks/useVersionFetching.test.ts`
- `frontend/src/hooks/useAvailableVersionState.ts`
- `frontend/src/hooks/useAvailableVersionState.test.ts`
- `frontend/src/components/app-panels/VersionManagementPanel.tsx`
- `frontend/src/utils/appVersionState.ts`
- `reports/frontend-async-owner-inventory.md`
- this plan, ledger, and issues files

**Tasks:**

- [x] Record the bounded installation/timer consumer inventory and disposition;
  unrelated polling owners become follow-up issues unless they share the exact
  current-invocation defect.
- [x] Replace interval overlap with a serialized self-scheduling loop whose
  next request starts only after the current request settles.
- [x] Start polling a requested install only after the backend accepts that
  lifecycle; retain immediate polling only for an existing release-discovery
  lifecycle.
- [x] Give each enabled app/tag lifecycle a generation and prevent superseded
  or unmounted completions from mutating state.
- [x] Keep non-cancellable completions observed and classified; do not abandon
  rejected promises.
- [x] Remove the unreachable dialog-local polling fallback and narrow its
  presentation Interface.
- [x] Route cancellation through the manager with the resolved app identity;
  keep cancellation distinct from failure while its terminal progress is
  observed.
- [x] Add controlled overlap, app/tag change, disable/unmount, success,
  failure, and cancellation tests.

**Acceptance gate:** FE-A3 plus passing affected typecheck, lint, and focused
Vitest evidence.

**Status:** `Accepted` after PRG-I12 repair review

### Milestone 1: Admit Representative Renderer Evidence

**Goal:** Select the smallest permanent runtime harness that can decide the
renderer-only user-workflow claims.

**Allowed write set:**

- `reports/renderer-harness-admission.md`
- this plan, ledger, and issues files

**Tasks:**

- [x] Run the admitted one-workflow experiment without changing dependencies
  or permanent tooling.
- [x] Record environment, command prototype, isolated-state fixture, process
  cleanup, observed browser/accessibility surfaces, runtime, reachable negative
  failure, and comparison with existing Vitest/release smoke tests.
- [x] Select existing tooling, one new runner, or `unsupported`; if a new runner
  or different write set is selected, revise Milestone 5 before implementation.

**Acceptance gate:** Reviewed admission report meets the stopping condition and
names one deciding environment/oracle or a typed blocker.

**Status:** `Accepted`

### Milestone 2: Establish Modal And Popup Interaction Modules

**Goal:** Centralize repeated overlay lifecycle policy and migrate the audited
modal and popup consumers without changing feature-domain content.

**Allowed write set:**

- `frontend/src/components/ui/ModalDialog.tsx`
- `frontend/src/components/ui/ModalDialog.test.tsx`
- `frontend/src/components/ui/OverlayEscapeStack.ts`
- `frontend/src/components/ui/Popover.tsx`
- `frontend/src/components/ui/Popover.test.tsx`
- `frontend/src/components/ui/index.ts`
- `frontend/src/components/ConfirmationDialog.tsx`
- `frontend/src/components/ConfirmationDialog.test.tsx`
- `frontend/src/components/InstallDialogFrame.tsx`
- `frontend/src/components/InstallDialogFrame.test.tsx`
- `frontend/src/components/ModelMetadataModalFrame.tsx`
- `frontend/src/components/ModelMetadataModalFrame.test.tsx`
- `frontend/src/components/ModelServeDialog.tsx`
- `frontend/src/components/ModelServeDialog.test.tsx`
- `frontend/src/components/model-serve/ModelServeDialogContent.tsx`
- `frontend/src/components/model-serve/useDialogFocusTrap.ts`
- `frontend/src/components/ModelImportDialog.tsx`
- `frontend/src/components/ModelImportDialog.test.tsx`
- `frontend/src/components/HuggingFaceAuthDialog.tsx`
- `frontend/src/components/HuggingFaceAuthDialog.test.tsx`
- `frontend/src/components/VersionSelector.tsx`
- `frontend/src/components/VersionSelector.test.tsx`
- `frontend/src/components/VersionSelectorTrigger.tsx`
- `frontend/src/components/VersionSelectorTrigger.test.tsx`
- `frontend/src/components/VersionSelectorDropdown.tsx`
- `frontend/src/components/VersionSelectorDropdown.test.tsx`
- `frontend/src/components/ModelSearchBar.tsx`
- `frontend/src/components/ModelSearchBar.test.tsx`
- `frontend/src/components/RemoteModelDownloadMenu.tsx`
- `frontend/src/components/RemoteModelDownloadMenu.test.tsx`
- `frontend/src/components/RemoteModelListItemActions.tsx`
- `frontend/src/components/RemoteModelListItemActions.test.tsx`
- `reports/frontend-overlay-consumer-inventory.md`
- this plan, ledger, and issues files

**Tasks:**

- [x] Record the bounded overlay consumer matrix, actual interaction semantics,
  and migrate/already-safe/delete/follow-up disposition.
- [x] Implement `ModalDialog` and migrate one representative nested-capable
  consumer; prove name, entry, containment, Escape/backdrop, cleanup, and focus
  restoration through the Interface before migrating matching dialogs.
- [x] Implement `Popover`, select feature-correct semantics, and prove trigger
  relationship/state, keyboard/pointer dismissal, focus entry/return, and
  cleanup before migrating matching popups.
- [x] Delete superseded focus lifecycle machinery after its last consumer
  migrates; do not retain compatibility wrappers without a consumer.
- [x] Run representative focus/accessibility workflows selected in Milestone 1.

**Acceptance gate:** FE-A4 with focused component tests and representative
runtime evidence; every inventoried consumer has a disposition.

**Status:** `Accepted`

### Milestone 3: Make Progress, Outcomes, And Motion Perceivable

**Goal:** Expose dynamic state programmatically and make the documented motion
preference true without adding an unsupported general accessibility claim.

**Allowed write set:**

- `frontend/src/components/ProgressDetailsView.tsx`
- `frontend/src/components/ProgressDetailsView.test.tsx`
- `frontend/src/components/ui/Popover.tsx`
- `frontend/src/components/ui/Popover.test.tsx`
- `frontend/src/components/ModelMetadataModal.test.tsx`
- `frontend/src/components/InstallDialog.tsx`
- `frontend/src/components/InstallDialog.test.tsx`
- `frontend/src/hooks/useInstallationManager.ts`
- `frontend/src/hooks/useInstallationManager.test.ts`
- `frontend/src/hooks/useInstallationState.ts`
- `frontend/src/hooks/useInstallationState.test.ts`
- `frontend/src/index.tsx`
- `frontend/src/index.css`
- `frontend/README.md`
- this plan, ledger, and issues files

**Slice sequence:**

- M3-S1: `ProgressDetailsView.tsx` and its focused test only.
- M3-S2: composition-root and CSS reduced-motion policy only.
- M3-S3a: program-approved FE-I13 repair through only the existing
  installation manager, state, and dialog source/tests, including the PRG-I12
  requested-install admission-order regression that reopened FE-A3.
- M3-S3b: representative normal/reduced runtime evidence and affected README
  behavior claims, after M3-S3a acceptance and serialized documentation
  ownership confirmation.

**M3-S3b re-plan gate:** Real Chromium proved the CSS policy but found that
`MotionConfig reducedMotion="user"` still lets the Popover's initial
`translateY(-6px)` persist into nonzero-opacity frames before snapping to its
resting position. Program review admitted only `Popover.tsx` and its colocated
test: use the operating-system preference to select zero entry and exit
translation under reduce while retaining `-6px` in normal mode and preserving
opacity behavior. The deciding rerun must record positive entry and dismissal
samples in all four default/library-only × normal/reduced scenarios and reject
any visible reduced transform. The root also admitted a test-only M2 follow-up
in `ModelMetadataModal.test.tsx`: replace its deleted backdrop-label query with
the actual backdrop/document interactions and await initial async state without
act warnings. The full frontend suite must be green before README or FE-A5
acceptance.

**M3-S3 re-plan gate:** Runtime admission found that no production caller sets
the progress view to `details`, successful terminal progress is cleared by the
manager, and the dialog clears its presentation tag on any terminal payload.
Resolve `FE-I13` through the existing installation owner chain with the exact
M3-S3a files and focused behavior evidence before representative runtime
resumes. Integration review additionally found PRG-I12: a requested lifecycle
could poll before backend admission and consume a prior terminal payload.
M3-S3a included the deferred-admission regression and the accepted repair that
re-satisfied FE-A3/M0. The library-only entry has no installation feature by
design; installation-status
evidence is not applicable there, while motion-policy evidence still covers
both real entries.

**Tasks:**

- [x] Give determinate progress a programmatic name/value and announce terminal
  success/failure at the appropriate politeness without duplicate updates.
- [x] Apply the operating-system reduced-motion preference to Framer Motion at
  the composition root and to nonessential CSS animation/transition behavior.
- [x] Verify normal and reduced modes in the representative runtime.
- [x] Update only the affected behavior claims in the frontend README; leave
  gate/count policy cleanup to governance.

**Acceptance gate:** FE-A5 and the applicable portion of FE-A7.

**Status:** `Accepted`

### Cross-Plan Slice XR-S1: Gate Renderer On Launcher-Root Recovery

**Goal:** Integrate the accepted desktop launcher-root startup and selection
contract without mounting backend consumers against an unresolved root or
making the normal ready path feel slower.

**Dependency gate:** The platform owner accepted and froze the closed,
path-free startup/selection unions, main-process single-flight policy, and
sandbox-compatible preload decoder before this consumer was integrated.

**Allowed write set:**

- `frontend/src/types/api-window.ts`
- `frontend/src/types/api-bridge-utilities.ts`
- `frontend/src/hooks/useLauncherRootRecovery.tsx`
- `frontend/src/hooks/useLauncherRootRecovery.test.tsx`
- `frontend/src/hooks/useAppWindowActions.ts`
- `frontend/src/hooks/useAppWindowActions.test.ts`
- `frontend/src/components/LauncherRootRecoveryView.tsx`
- `frontend/src/components/LauncherRootRecoveryView.test.tsx`
- `frontend/src/index.tsx`
- `reports/launcher-root-recovery-consumer-evidence.md`
- this plan, ledger, and issues files

The held product source at this boundary is exactly the two type files, the
provider and its test, the recovery view and its test, the window-actions hook
and its test, and `index.tsx` listed above. No other uncommitted frontend source
belongs to an independently acceptable tranche.

**Tasks:**

- [x] Replace the legacy optional-field selection response with the exact
  startup and selection unions exposed by the decoded preload Interface.
- [x] Add one composition-root provider that treats browser mode as not
  applicable, gates Electron children until ready, serializes startup/selection
  attempts, keeps initializing polling sequential, and cleans up timers and
  late outcomes; the main process owns the only visibility deadline.
- [x] Present checking, explicit-authority guidance, retryable unchanged,
  ambiguous/published terminal, restarting, and bridge-unavailable states with
  accessible status/actions and frameless window controls.
- [x] Delegate the existing Change Library action to the provider and remove
  the legacy direct bridge/logging path.
- [x] Prove StrictMode ownership, every state transition, action availability,
  cancellation restoration, focus, path non-disclosure, both renderer aliases,
  and absence of legacy response fields through public seams.
- [x] Run both production entries with the actual sandboxed preload and prove
  that a fast ready response can reach a model-list content frame; this sample
  exposed that renderer-only timing cannot prove the first visible frame.
- [x] Add the renderer half of the construction-safe visibility handshake:
  never acknowledge initializing, terminal-latch the platform watchdog, and
  acknowledge ready, recovery, or unavailable exactly once from the terminal
  layout commit. This is semantic notification only; it makes no compositor-
  presentation claim.
- [ ] Prove all closed producer startup and selection values through the actual
  compiled preload and frontend semantics, reject malformed/extra payloads,
  and run first-visible ready/recovery/unavailable/no-preload workflows through
  a re-planned compositor owner. The marker challenge satisfied frame truth but
  not immediate startup, and the hidden-capture alternative failed its marker
  proof.

**Acceptance gate:** FE-A8, the accepted platform producer/decoder evidence,
focused and full frontend gates, both production builds, and the representative
two-entry frame oracle.

**Status:** `Verifying`

### Milestone 4: Adopt Decoded Contracts And Honest Model Projection

**Goal:** Consume the provider-owned decoded operation Interface and retain
instant model display with explicit provenance, degradation, and recovery.

**Dependency gate:** Accepted Rust DTO/error contract and platform-generated
decoder Interface. This milestone must not start by inventing either locally.

**Pre-start re-plan gate:** Classify the link-health refresh rejection
(`FE-I10`) and model-import picker rejection (`FE-I11`) against this
milestone's decoded bridge/projection Seam. When an outcome shares that owner,
add its exact consumers and behavior evidence to this milestone before source
edits. Otherwise add one exact focused slice with its own write set and gate;
neither finding may be deferred outside this program.

**Allowed write set:**

- `frontend/src/api/adapter.ts`
- `frontend/src/api/adapter.test.ts`
- `frontend/src/api/import.ts`
- `frontend/src/api/models.ts`
- `frontend/src/api/versions.ts`
- `frontend/src/utils/modelLibrarySnapshot.ts`
- `frontend/src/utils/modelLibrarySnapshot.test.ts`
- `frontend/src/hooks/useModels.ts`
- `frontend/src/hooks/useModels.test.ts`
- `frontend/src/App.tsx`
- `frontend/src/components/LibraryOnlyApp.tsx`
- `frontend/src/components/AppShellState.ts`
- `frontend/src/components/AppShellState.test.ts`
- `frontend/src/components/ModelManager.tsx`
- `frontend/src/components/ModelLibraryProjectionNotice.tsx`
- `frontend/src/components/ModelLibraryProjectionNotice.test.tsx`
- `frontend/src/components/ModelManagerIntegrityRefresh.test.tsx`
- `reports/renderer-contract-consumer-inventory.md`
- `frontend/README.md`
- this plan, ledger, and issues files

**Tasks:**

- [ ] Inventory every renderer API consumer and record decoded/already-safe,
  migrate, delete, or external-owner disposition against the accepted Interface.
- [ ] Remove assertions or fallback substitution that let invalid/unavailable
  results masquerade as domain values; retain wrappers only where the deletion
  test shows real renderer-domain composition or recovery.
- [ ] Evolve the snapshot envelope to record local capture time and provenance,
  while decoding version-1 snapshots as cached with unknown age.
- [ ] Expose one typed projection outcome from `useModels` and render cached,
  degraded, retrying, and fresh state without hiding the cached list.
- [ ] On successful authoritative refresh, atomically replace the projection;
  on failure, preserve visible cached data and expose recovery.
- [ ] Test valid/legacy/malformed storage, unavailable/invalid/failure results,
  stale completion, retry, and fresh replacement through the Module Interface.
- [ ] Run the representative immediate-startup and degradation workflow.

**Acceptance gate:** FE-A1, FE-A2, and the applicable portion of FE-A7. Link
provider contract/decoder evidence without claiming ownership of it.

**Status:** `Blocked` pending accepted Rust DTO/error and platform decoder handoff

### Milestone 5: Prove Default And Library-Only Renderer Behavior

**Goal:** Add the admitted package-local representative command and prove both
supported built-renderer workflows without claiming packaged release contents.

**Dependency gate:** Milestone 1 harness decision and integrated governance
changes to shared package scripts.

**Allowed write set:**

- `frontend/package.json`
- `frontend/tests/renderer/run.mjs`
- `frontend/tests/renderer/electron-main.cjs`
- `frontend/tests/renderer/fixtures.cjs`
- `frontend/README.md`
- this plan, ledger, and issues files

**Tasks:**

- [ ] Revise this exact write set first if the admission report selects a
  different bounded location or no capable tool.
- [ ] Add one package-local representative command with deterministic external
  operation fixtures, isolated profile/state, and process cleanup.
- [ ] Build and launch the default and library-only modes through their real
  entry points.
- [ ] Prove plugin UI presence in default mode, its absence in library-only
  mode, and a core model-library workflow in both.
- [ ] Run FE-A2, FE-A4, and FE-A5 workflows in the same harness and document its
  unique claim and schedule for governance consumption.
- [ ] Keep bundle/package inspection in the platform plan; minified string
  absence is not the renderer behavior oracle.

**Acceptance gate:** FE-A2, FE-A4, FE-A5, FE-A6, and FE-A7 with a passing
representative command in both build modes.

**Status:** `Planned`

### Milestone 6: Frontend Acceptance Review

**Goal:** Reconcile all renderer claims, reports, and evidence, then hand off
remaining producer, packaged-artifact, and governance evidence without
overclaiming frontend acceptance.

**Allowed write set:**

- this plan, ledger, issues, and reports

**Tasks:**

- [ ] Run affected frontend lint, typecheck, focused unit/integration tests,
  both production builds, and the admitted representative command.
- [ ] Review each objective row against linked evidence and its stated
  environment; lower-fidelity results remain corroboration only.
- [ ] Confirm each systemic consumer inventory has a disposition and each
  retained permanent test has a unique claim and schedule owner.
- [ ] Link unresolved producer, packaged-artifact, CI, platform, or broader
  accessibility claims to their sibling owner rather than broadening this plan.

**Acceptance gate:** FE-A1 through FE-A7 are satisfied in their stated
environments, or the plan remains non-accepted with explicit issues.

**Status:** `Planned`

## Blockers

- No blocker prevents M0-S1. The program integration owner accepted the
  critical Rust RPC disclosure evidence and released frontend source work.
- Milestone 4 is dependency-gated on accepted Rust and platform contract work.
- No harness blocker remains for Milestones 2, 3, and 5; the accepted report
  admits the installed Electron/CDP environment without a new dependency.

## Re-Plan Triggers

- The harness experiment cannot observe the required browser accessibility,
  focus, media, or build-mode behavior with deterministic cleanup.
- Contract providers expose a materially different consumer Interface or omit
  an audited invalid/unavailable outcome.
- Consumer inventories find another active owner with the same invariant or a
  required file outside a milestone's exact write set.
- A popup's real interaction requires a different semantic/lifecycle Module
  than the proposed `Popover` Interface.
- Snapshot compatibility or product policy requires a producer revision or
  retention period not available in current contracts.
- Governance or platform integration changes shared package/build files before
  the dependent milestone starts.
- A proposed Module fails the deletion test, merely forwards calls, or forces
  unrelated callers to learn hidden transport/storage/focus details.
- Representative runtime evidence contradicts jsdom/unit behavior.

## Final Acceptance

- Acceptance status: `partial`
- Deferred follow-ups: Rust schema/error production, Electron/generated decoder
  production, packaged-artifact verification, CI scheduling, and any broader
  accessibility conformance remain with their named sibling owners.
- Final status: `Active`
