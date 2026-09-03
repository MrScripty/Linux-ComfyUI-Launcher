# Current Standards Audit — 2026-09-03

## Verdict

Pumas Library is **partially compliant and needs a staged standards refactor**.
It has a good local engineering baseline—strict TypeScript, broad unit coverage,
cross-platform builds, Rust unsafe defaults, Electron isolation, and shared
launcher checks—but those strengths do not yet establish trustworthy behavior
at several system boundaries.

The audit found one critical issue: the generic RPC handler can write Hugging
Face credentials to debug logs. The main systemic gaps are incomplete RPC
decoding and authorization, non-atomic persistence/event updates, implicit
migration and async shutdown contracts, stale release/SBOM evidence, and user
interfaces that can present cached or inaccessible state as authoritative.

This is an analysis artifact, not an implementation plan. No source, test,
configuration, generated artifact, or lockfile was changed by the audit.

## Audit Baseline

- Pumas commit: `a33c8c0efa7cd8783c7deeac9e608db205290d43`
- Standards commit: `52b096ded9c53afd439a3cf0efc4cc85252da570`
- Standards source: `/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/Coding-Standards/`
- Inventory: 892 tracked files from
  `scripts/dev/list-audit-files.sh --tracked-only`
- Inventory distribution: 328 Rust, 381 frontend, 21 Electron, 68 docs, 42
  scripts, and 52 other files

The review routed through `CORE-STANDARDS.md`, `STANDARDS-ROUTER.md`, the
Planning, Documentation, Implementation, Verification, Tooling, Build, and
Release workflows, and the applicable architecture, contracts, accessibility,
security, concurrency, resilience, cross-platform, dependency, licensing,
diagnostics, performance, frontend, library, launcher, Rust, TypeScript, IPC,
persistence, interop, generated-contract, and language-binding owners.

## Architecture and Code-Quality Assessment

The intended architecture is sound: Rust owns the model library and durable
state, Electron is a privileged desktop boundary, React renders projections,
and optional inference providers sit behind explicit build/runtime seams. Local
code quality is generally strongest inside each process, where the type system,
focused modules, and unit tests can enforce invariants.

The architecture becomes shallow at cross-process and release seams. Method
names, payload shapes, error meanings, update events, and binding claims are
partly duplicated across Rust, Electron, TypeScript, documentation, and CI.
Several adapters replace malformed or unavailable outcomes with empty/default
values, so consumers cannot reliably distinguish valid empty state from drift
or failure. Persistence similarly separates authoritative mutations from the
events that publish them.

The next architectural work should deepen these seams around one authoritative
contract and explicit invalid/degraded outcomes. Large implementation files may
be candidates for later review, but their size is not the compliance problem.

Severity means:

- **Critical:** a present security, data-integrity, or release-integrity threat.
- **High:** an enforceable standards failure at an important boundary or an
  absent proof that can permit incorrect behavior.
- **Medium:** material drift or weak evidence that should be corrected, but is
  not presently shown to corrupt data or expose a critical boundary.
- **Improvement:** useful design work that is not, by itself, a violation.

## Prioritized Findings

| ID | Severity | Finding | Principal evidence |
| --- | --- | --- | --- |
| CS-01 | Critical | RPC debug logging records complete request parameters, including `set_hf_token`; Electron development/debug mode forwards backend debug stderr into its persistent info-level log. Public/logged errors can also expose paths and URLs. | `rust/crates/pumas-rpc/src/handlers/mod.rs:408-417,482-488`; `rust/crates/pumas-rpc/src/handlers/models/auth.rs:7-10`; `electron/src/main.ts:21-23,499-503`; `electron/src/python-bridge.ts:383-395,447-452` |
| CS-02 | High | The RPC trust boundary is incomplete: LAN mode has no caller authentication, most methods lack operation-specific schemas, and responses/events are trusted or silently discarded. | `rust/crates/pumas-rpc/src/main.rs:203-214`; `rust/crates/pumas-rpc/src/server.rs:170-214`; `electron/src/rpc-method-registry.ts:1-9`; `electron/src/python-bridge.ts:66-102,746-760`; `frontend/src/api/adapter.ts:87-111` |
| CS-03 | High | Authoritative model mutations and their durable update records are separate commits, while startup migration identity, accepted-artifact integrity, ordering, and interruption behavior are implicit. | `rust/crates/pumas-core/src/index/model_index.rs:271-469,1046-1080`; `rust/crates/pumas-core/src/index/model_index/governance.rs:191-337` |
| CS-04 | High | Async work has incomplete current-invocation, admission, cancellation, and shutdown ownership across Rust tasks, Electron subscriptions, frontend installation polling, and Torch inference. | `rust/crates/pumas-core/src/api/runtime_tasks.rs:22-42`; `rust/crates/pumas-rpc/src/server.rs:83-113`; `electron/src/preload.ts:723-798`; `electron/src/main.ts:208-216`; `frontend/src/hooks/useInstallationProgress.ts:100-152`; `torch-server/openai_api.py:109-156,259-292` |
| CS-05 | High | The fast model snapshot has no timestamp, revision, or provenance, and unavailable refreshes leave cached data looking fresh. | `frontend/src/utils/modelLibrarySnapshot.ts:3-7,121-152`; `frontend/src/hooks/useModels.ts:41-92,285-295` |
| CS-06 | High | Release automation does not implement the documented artifact and SBOM contract; dependency, vulnerability, and license evidence is stale or non-reproducible. | `docs/contracts/release-artifacts.md:27-78`; `.github/workflows/build.yml:65-82,433-526`; `scripts/dev/generate-sbom.sh:12-41`; `torch-server/requirements.txt:1-7` |
| CS-07 | High | Binding support claims exceed real-host evidence, and binding concerns leak into the core crate despite the adapter-local contract. | `README.md:172-177`; `.github/workflows/build.yml:236-325,481-490`; `rust/crates/pumas-core/Cargo.toml:101-122`; `rust/crates/pumas-uniffi/src/bindings.rs:299-305` |
| CS-08 | High | Supported build configurations are not expressed or proven consistently: Rust HF/process/GPU capability markers retain their dependency/API graph, and the frontend library-only variant lacks representative CI and artifact checks. | `rust/crates/pumas-core/Cargo.toml:13-122`; `rust/crates/pumas-core/src/lib.rs:33-100`; `frontend/package.json:8-10`; `frontend/vite.config.ts:24-39`; `.github/workflows/build.yml:156-162` |
| CS-09 | High | Dialog, popup, progress, and dynamic-status accessibility is component-specific and incomplete rather than owned by shared interaction primitives. | `frontend/src/components/ModelImportDialog.tsx:65-85`; `frontend/src/components/HuggingFaceAuthDialog.tsx:113-150`; `frontend/src/components/ProgressDetailsView.tsx:77-120,206-253` |
| CS-10 | High | The repository still teaches and enforces the previous standards model. Fixed line counts and blanket per-directory README requirements are treated as architecture/documentation authority, contrary to the current standards. | `CONTRIBUTING.md:29-40`; `docs/STANDARDS_ADOPTION.md:7-32`; `frontend/eslint.config.js:83-100`; `frontend/scripts/check-file-size.js:3-58`; `scripts/dev/check-readme-coverage.sh:29-77` |
| CS-11 | High | Default plugin-enabled RPC startup replaces a failed configured loader with an empty temporary-directory loader and unwraps the fallback, obscuring degradation or panicking. | `rust/crates/pumas-rpc/Cargo.toml:51-57`; `rust/crates/pumas-rpc/src/main.rs:136-152` |
| CS-12 | High | Corrupt or torn persisted launcher-root configuration is treated as absent, after which startup silently selects another library root; writes are not atomic. | `electron/src/launcher-root.ts:21-89,126-143` |
| CS-13 | Medium | Verification mechanisms do not have one coherent schedule or clear marginal claims: `check:errors` is declared as precommit, is absent from hooks and CI, is currently red, and uses a weak regex oracle. | `frontend/package.json:17-20`; `.pre-commit-config.yaml:6-34`; `.github/workflows/build.yml:147-162`; `frontend/scripts/check-error-handling.js:38-59` |
| CS-14 | Medium | Current documentation authority is unclear: the public `PumasApi` ownership description conflicts with implementation, the RPC method document lists removed methods, and plan status is difficult to determine. | `README.md:51-56`; `docs/architecture/SYSTEM_ARCHITECTURE.md:51-59`; `docs/contracts/desktop-rpc-methods.md:14-28`; `docs/plans/README.md:1-25,46-56` |
| CS-15 | Medium | Launcher behavior maps unknown platforms to Linux, does not enforce a complete terminate/grace/kill deadline, and has wrapper-specific release behavior. | `scripts/launcher/platform-service.mjs:5`; `scripts/launcher/commands.mjs:56-150`; `launcher.sh:12`; `launcher.ps1:3` |

## What Is Already Strong

- TypeScript uses strict, type-aware compiler and lint settings, including
  unchecked-index and floating-promise protection.
- The frontend has 441 passing Vitest tests across 102 files, plus several good
  stale-result and persisted-snapshot decoders.
- Electron enables context isolation, sandboxing, web security, and disables
  renderer Node integration.
- Rust denies unsafe code by default and keeps the few platform/FFI relaxations
  beside explicit safety arguments.
- SQLite enables foreign keys and WAL and provides a distinct read-only opening
  path.
- RPC defaults to loopback and has request-size and concurrency bounds.
- Launcher, Electron, and Torch sidecar checks provide useful local evidence.
- CI builds/tests Rust and runs frontend static checks on Linux, Windows, and
  macOS; the frontend production build and tests run on Linux. Node installation
  uses the frozen lockfile.

These strengths should be retained. They do not close the boundary-specific
findings without the scenario evidence named above.

## Verification Snapshot

| Check | Audit result | Interpretation |
| --- | --- | --- |
| Frontend ESLint and TypeScript | Pass | Useful static evidence |
| Frontend Vitest | Pass: 102 files, 441 tests | Strong component/unit evidence; not a real Electron/browser workflow |
| Frontend file-size check | Pass | Not valid architectural acceptance under the current standards |
| Frontend `check:errors` | Fail: 31 reports | Gate and oracle both need reauthorization; reports are not automatically 31 code defects |
| Electron validate, lint, tests | Pass: 5 test files | Useful shell/validation evidence |
| Launcher tests | Pass: 25 tests | Useful shared parser/command evidence |
| Torch Ruff and unit tests | Pass: 13 tests | CI installs only development dependencies and the tests can substitute fake runtime modules; this is not real ASGI/Torch evidence |
| Dependency ownership and version alignment | Pass | Useful manifest-governance evidence |
| README coverage script | Fail: 2 directories | The script itself conflicts with the current documentation standard and should not be repaired by adding arbitrary READMEs |
| Full Rust, cross-platform runtime, bindings hosts, release publication | Not executed by this audit | Existing build machinery was inspected; its missing scenario proofs remain findings |

## Focused Audit Set

The domain passes supporting this overview are:

- [Frontend and UI](frontend-and-ui.md)
- [Rust library and RPC](rust-library-and-rpc.md)
- [Desktop, release, bindings, and Torch](desktop-release-bindings-and-torch.md)
- [Standards governance and verification](governance-and-verification.md)

Recommended remediation-oriented audits, in order:

1. **RPC threat model and diagnostics:** credential redaction, public error
   projection, LAN support decision, authentication, authorization, and hostile
   client evidence.
2. **Generated desktop RPC contract:** strict request, response, and event
   schemas; extra-field policy; negative producer/consumer tests.
3. **Persistence and recovery:** atomic mutation/event outbox, stable migration
   identity/integrity/order, interruption recovery, and replay acceptance.
4. **Release dependency closure:** exact artifact manifest, tag/version equality,
   locked dependency inputs, current SBOMs, generated notices, and final-byte
   checksum proof.
5. **Async lifecycle:** Rust task supervision, frontend polling/subscription
   ownership, Torch admission/worker/cancellation behavior, and shutdown
   postconditions.
6. **Supported configuration matrix:** Rust feature/dependency graph, frontend
   default/library-only artifacts, and real host-language binding cohorts.
7. **Accessible interaction system:** shared dialog/popup/progress/status
   primitives verified in a representative Electron/browser environment.
8. **Standards and documentation authority:** current routing, gate inventory,
   plan lifecycle, and public contract reconciliation.
9. **Plugin startup and degradation:** configured-root authority, explicit
   disabled/degraded outcomes, and fallible recovery without panic.
10. **Launcher-root durability:** atomic configuration replacement, explicit
    corrupt/unreadable outcomes, and proof that startup never silently changes
    the authoritative library.

Each item should become its own bounded audit before an implementation plan is
approved. Composed-design review applies to the eventual RPC, persistence,
release, async, and feature-boundary refactors. This audit does not admit a
specific design for those changes.

## Interpretation Notes

- The 13,000-line model-library module and other large files are useful review
  locators, not standards violations. Any decomposition must be justified by
  ownership, interface depth, change coupling, or testability—not a line count.
- Counts such as “51 of 152 RPC methods have schemas” establish inventory gaps;
  they do not decide the design.
- A passing build or startup smoke does not prove a real user workflow,
  cross-process contract, migration interruption, or host-language load.
- Historical terminal plans do not need mechanical rewrites solely to add new
  template fields. Active or still-authoritative plans do need a clear current
  status, acceptance claims, and exactly one next slice.
