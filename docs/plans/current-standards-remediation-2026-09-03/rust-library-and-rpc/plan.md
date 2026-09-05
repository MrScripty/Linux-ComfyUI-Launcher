# Plan: Rust Library and RPC Standards Remediation

**Plan status:** `Active`

**Current phase:** Milestone 1 is accepted. Milestone 2's C3 queue candidate is
active following the user's approval to coordinate migration and relocation
in RUST-I8. The shared
finalization-policy extraction is independently verified; the queue draft is
not accepted. Milestone 2 otherwise remains producer
contract work that is valid for loopback desktop RPC and local core IPC.
`RUST-I1` is resolved: desktop RPC is loopback-only and LAN support is removed.

**Next slice:** Establish the exact production core recovery integration boundary
needed to make the isolated C3 candidate reachable and lint-clean. Legacy owned
relocation and its migration caller are implemented and narrowly reviewed, but
the isolated candidate fails strict Clippy. Inventory the held core caller and
its contracts before admitting it; do not suppress warnings or pull RPC/UI
consumers into this slice. The real transferred-byte interrupted
response → settled Error → fresh-owner restore → cancel path is now verified;
hard process-crash recovery remains unproved. C1 store repairs and C2 destination
authority passed independent source review and targeted regressions in both
feature configurations. C3's ordinary admission, exact status/settlement, and
strict restore integration passed independent review and targeted regressions
in both feature configurations; final supporting gates are recorded in the
ledger. Full lifecycle integration is not yet accepted. Unused integration warnings remain visible.
Slice B remains the accepted lifecycle regression boundary. C3 lifecycle integration and C4 importer integration proceed in
that order; they are reviewable checkpoints, not independently shippable claims.
The accepted Slice B
generation, terminal-projection, cancellation, cleanup, tombstone, and sticky
failure semantics remain invariants. General client-drop draining remains
Milestone 4/RUST-A6, Slices D-E remain held, and Milestone 2J remains queued
with only its isolated red HF-unavailable oracle recorded.

**Acceptance status:** `pending`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

**Reports:** Pending report slots are listed in the
[execution ledger](execution-ledger.md#reports).

**Audit source:** [Rust library and RPC](../../../audits/current-standards-2026-09-03/rust-library-and-rpc.md)

## Active PRG-I19 Slice C Boundary

### Checkpoints after the implementation audit

| Checkpoint | Owned result | Acceptance evidence |
| --- | --- | --- |
| C1 (internally verified) | Durable admission, restart reconciliation, quarantine, and exact queue settlement behind the store Interface | 38 store tests and atomic/HF regressions pass in both feature configurations; independent source review accepted |
| C2 (internally verified) | Held configured-root destination authority and marker publisher ready for runtime integration | Independent source review accepted; root reproduced 14 recovery, 22 atomic-publication, 38 store, and 155 HF tests in both feature configurations |
| C3 (active; queue draft unaccepted) | Start, pause, resume, cancellation, restore, and relocation consume the store and destination Interfaces | Earlier admission/interrupted-transfer checkpoints remain verified. RUST-I8 scope is approved; legacy relocation and its migration caller must pass before draft acceptance |
| C4 | Importer mutations are awaited before settlement; notifications follow release | Real async importer held during cancellation/completion; successor progress and terminal-state tests |

The complete producer/consumer boundary still requires the existing later
acceptance gates. Do not broaden C1 while its regression suite is failing.
Each checkpoint reports current evidence separately from historical passing
counts. The user authorized these checkpoints after reviewing the implementation
audit; they replace the single expanding implementation step, not its required
product outcomes.

**Persistence mechanism decision:** retain JSON for C1. SQLite is already an
available dependency and could simplify atomic updates and referential integrity,
but it would not remove download-to-filesystem recovery, destination authority,
or task ownership. The existing index database's path-based opening and
WAL/`synchronous=NORMAL` configuration do not prove this store's authority or
durability contract. A separate replacement requires a bounded real-file proof
of database/journal authority, interruption/reopening, and idempotent JSON
handover, and must demonstrate a net reduction in maintained machinery. C1
does not add a second authoritative store or a speculative database adapter.

**Restart reconciliation:** persisted bytes cannot reveal whether a previous
process observed a successful final sync. A fresh owner must explicitly confirm
the recorded transition through the store before using it to authorize work.
Tests must distinguish raw inventory from reconciled inventory; an in-memory
confirmation cache alone is not restart proof.

**C1 integration limits:** terminal settlement retains exact release records
without garbage collection. Queue-owned generic save/remove/revoke/relocate
operations now refuse mutation; C3 must use dedicated owned transitions rather
than bypass those guards. Runtime owners must complete destination effects
before store settlement and explicitly reconcile before restart admission.
Some private integration APIs remain unused during C3; compilation is not a claim
of warning-free or end-to-end acceptance.

**C2 integration limits:** the builder establishes the library directory before
opening its held download authority, and preserves HF search when that authority
is unavailable. Root-relative targets and absolute configured-root aliases share
identity; nested symlinks are rejected. Model, creation-anchor, and nested file
parent replacement cannot redirect capability effects. Marker publication uses
the existing atomic outcome algebra through a held parent, including directory
creation sync. C3 now routes ordinary start and admitted resume/cancellation
through this capability and rejects unconfigured starts. Legacy mutation paths
and runtime reservation identity still need migration.
Only Linux execution is evidenced; this is not cross-platform acceptance or a
full builder-startup test.

**Current C3 limits:** the new transfer path uses a real loopback HTTP response
and real partial-file writes, then an orderly Error before reopening; it is not
a process-crash or live Hugging Face service claim. The earlier marker-failure
and seeded-final-file regressions retain their narrower meaning. The unaccepted
queue draft uses held identities and one retained capability per state, but
legacy relocation now transfers path and retained authority together, with
Pending source/target custody on uncertainty. The physical
destination mutex remains; comprehensive restore still
needs task ownership, legacy migration, and hidden/quarantined/recovery-state
reconciliation. Stalled pause and admitted relocation remain pending. Unknown admissions
fail closed but do not yet have the complete recovery path. Runtime release
facts are retained until owner drop to prevent stale-inventory resurrection.
The real asynchronous importer and callback ordering remain C4. This admission
checkpoint is neither full C3 acceptance nor a producer/consumer or GUI handoff.

- **Exact source write set:**
  `rust/crates/pumas-core/src/model_library/hf/download.rs`,
  `rust/crates/pumas-core/src/model_library/hf/lifecycle.rs`,
  `rust/crates/pumas-core/src/model_library/hf/types.rs`,
  `rust/crates/pumas-core/src/model_library/hf/mod.rs`,
  `rust/crates/pumas-core/src/model_library/download_store.rs`,
  the existing untracked
  `rust/crates/pumas-core/src/model_library/download_recovery.rs`,
  `rust/crates/pumas-core/src/api/builder.rs`,
  `rust/crates/pumas-core/src/api/migration.rs`,
  `rust/crates/pumas-core/src/api/state.rs` (migration call-site arguments only),
  `rust/crates/pumas-core/src/model_library/partial_download.rs`,
  `rust/crates/pumas-core/src/metadata/atomic.rs`, and
  `rust/crates/pumas-core/src/metadata/mod.rs`.
- **Exact record set:** this plan, `execution-ledger.md`, `issues.md`, and
  `reports/rpc-contract-and-threat-model.md`.
- **Accepted base:** Slice B source hashes `abfe0382` (`lifecycle.rs`),
  `c69953b1` (`mod.rs`), `2e75638f` (`download.rs`), and `fdd00a3c`
  (`types.rs`). Additional admitted baselines are `020975b5`
  (`download_store.rs`), `c7244b5a` (`download_recovery.rs`), `95abe5fe`
  (`api/builder.rs`), `c838e184` (`metadata/atomic.rs`), and `15adcd4f`
  (`metadata/mod.rs`).
- **Admission invariant:** the builder opens the selected model-library root
  once and injects a crate-private held authority; an unconfigured client may
  search but destination mutation is typed unavailable. One caller-independent
  admission transition durably persists the complete request, non-authorizing
  destination identity, domain, FIFO ordinal, and predecessor/release proof
  before returning an ID, publishing active state, or performing an effect.
  Only a confirmed durable attempt promotes to the gated Worker in one no-await
  downloads-to-task commit; ambiguous publication parks hidden custody.
- **Destination invariant:** reservation identity and every effect derive from
  the same held configured-root capability plus validated relative target, not
  a raw/canonical path string or nearest-existing ancestor. Missing targets and
  aliases retain one identity; root/path replacement fails closed. The private
  state-lifetime queue retains Paused, recoverable Error, or Pending quarantine
  authority, orders restore by durable ordinal, and wakes only after exact
  generation plus durable/published terminal release. No physical async mutex
  is held across effect work, signals, or callbacks.
- **Persistence invariant:** version 3 strictly migrates legacy/v1/v2 rows as
  recoverable state, owns ordinary row/admission/FIFO truth, and exclusively
  owns full-snapshot lifecycle quarantine. Pending cleanup is independent of
  sticky provenance. Clean Pending removal and sticky Pending-to-Verified use
  exact attempt/release proofs; Recovery quarantine preserves the durable
  revocation tombstone. Unknown publication never authorizes verification,
  cancellation, queue release, or empty restore. Stale save/status/relocation
  rejects every quarantined ID.
- **Execution/publication invariant:** capability-relative marker staging,
  file sync, atomic rename, and parent sync use the accepted Slice A outcome
  algebra. Directory, marker, persistence, pause, restore, relocation, and
  callback work remains task-owned and drained. A private publisher linearizes
  immutable snapshot capture/revision/dispatch and signals outside all guards.
  Pause uses owner-visible wakeups for stalled headers/body/retry and only the
  exact started generation can durably project Paused. The real asynchronous
  importer mutation is owned and awaited while the logical destination claim
  is held; importer failure preserves resumable finalization state. Completion releases
  logical destination custody after durable/published terminal state and drain,
  before an owned callback-only phase whose panic cannot roll back Completed.
- **Earlier regression evidence:** a cached public ordinary start against a path
  occupied by a regular file returned setup `Err` after leaving a published
  ownerless `Queued` entry. A public ambient resume cancelled while awaiting
  authentication similarly left its prior `Paused` state as ownerless Queued.
  Store reds now also cover missing v1/v2 migration, exclusive quarantine,
  sticky-versus-clean Pending, typed removal proof, and ambiguous publication.
  Destination identity, durable FIFO admission/restore, capability marker,
  stalled pause, relocation, terminal rescue, and callback ordering remain
  red-first work before freeze.
- **Supporting finalization seam:** the queue-identity regression also reaches
  legacy restore's shared size-inference and finalization policy. The existing
  `partial_download.rs` owner is added to this slice solely to share that policy
  between its existing path adapter and the held-capability adapter. Existing
  library callers retain their behavior; HF restoration must not regain ambient
  filesystem authority or duplicate the policy. This replaces the prior
  nine-file count limit, not the product, consumer, or dependency boundaries.
- **Relocation scope approved:** the user approved coordinated migration and
  owned relocation after PRG-I24. The existing public relocation entry point
  owns preflight, physical movement, marker publication, persistence, and
  state/queue transfer. Its only production caller, `api/migration.rs`, delegates
  those effects and must not ignore refusal or roll back an unknown outcome.
  A partial directory without a tracked owner remains explicitly skipped with
  a report reason; no ambient move substitutes for missing lifecycle authority.
  Complete-model migration is unchanged.
  This checkpoint supports legacy dormant downloads; existing admitted/recovery
  refusal moves before all effects. Admitted queue-graph evolution remains C3
  work, not an accepted capability or a new silent disablement.
  A durable intent precedes movement and reserves both identities. Unknown
  outcomes preserve records and block conflicting work, including after reopen;
  no automatic rollback or guessed restart placement is admitted.
  Cancellation waits for the owned relocation result and re-reads authority.
  Root owns caller/records, store_checkpoint owns `download_store.rs`,
  destination_checkpoint owns the four HF files, and a separately assigned
  capability owner may edit only `download_recovery.rs`. Cargo runs serialize;
  no worker commits or edits another owner's files.
  Acceptance: real temporary-directory/store relocation and public migration
  orchestration, exact refusal without movement, cancellation, collision and
  publication-failure preservation, then affected default/no-default suites.
  These are automated integration/contract claims on the local Linux filesystem;
  other-platform and hard-process-crash acceptance remain separate.
  Composed-design review is applicable: filesystem authority stays in the held
  capability, durable intent in the store, task/cancellation and queue custody
  in HF, and report/index projection in migration. Moving a model necessarily
  coordinates these owners; callers no longer know rename/rollback ordering.
  Filesystem mechanism changes stay in the capability and publication mechanics
  in the store; migration consumes a settled result, not either representation.
  The intent is necessary to preserve bytes and exclude both destinations after
  interruption; deleting it moves that obligation back to callers and restore.
  Reuse existing task, atomic publisher, capability and queue owners; no general
  migration framework, additional runtime or independent registry is admitted.
- **Held boundaries:** no further source expansion, public constructor/wire outcome,
  manifest, RPC/IPC/UniFFI, frontend/Electron, package/generated/CI, or shared-
  document mutation. The metadata files expose only the existing atomic writer
  to a held capability-relative marker target. Builder changes inject the
  selected root in C2, propagate strict restore failure in C3, and wire owned
  asynchronous importer hooks in C4. The Rust restore method returns
  `Result<Vec<DownloadCompletionInfo>>`; its builder and test callers migrate
  together. Corrupt or uncertain authoritative download inventory prevents
  successful API initialization rather than being reported as empty restore.
  Slices D-E, M2J, full aggregate verification,
  general client Drop, consumer implementation, and standalone-shippable
  claims remain excluded. The [root incremental-commit decision](../plan.md#binding-decisions)
  permits coherent verified candidates with compatible reachable contracts;
  it does not permit incompatible producer-only integration or full C3
  acceptance from the narrower admission checkpoint.

## Objective

Bring the Rust library, its local IPC, the desktop RPC server, and the Rust side
of native adapters into compliance with the current security, contract,
persistence, concurrency, dependency, and verification standards. The result
must expose explicit invalid, failed, degraded, and shutdown outcomes; keep
secrets and internal locators out of public diagnostics; make durable model
mutation/event history recoverable; and make each accepted build capability
truthful.

This plan owns the canonical Rust/server desktop-RPC DTO and public
error/redaction contracts. It also owns the distinct local `pumas-core` IPC
contract. It does not own Electron/TypeScript projections or host-language
release support.

## Baseline

- Audit code baseline: `a33c8c0efa7cd8783c7deeac9e608db205290d43`.
- Planning code baseline: `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`.
- Audit standards baseline: `52b096ded9c53afd439a3cf0efc4cc85252da570`.
- Planning standards baseline: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`.
- The planning baseline changed documentation, not the audited Rust source; the
  finding evidence was rechecked against the planning baseline.
- No implementation or verification result is inherited from file counts,
  dependency counts, a green compile, or the audit's read-only inspection.

The routed authorities are `CORE-STANDARDS.md`, `STANDARDS-ROUTER.md`, the
Planning, Development Proportionality, Implementation, Verification,
Documentation, Tooling, and Build workflows; the Library Application, IPC, and
Persistence profiles; the Rust base, API, Async, Dependencies, Tooling,
Security, Cross-Platform, Interop, Language Bindings, and Unsafe standards;
and the Architecture, Contracts, Concurrency, Persistence, Resilience,
Security, Diagnostics, Performance, Dependencies, and Cross-Platform topic
owners.

## Objective Acceptance

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| RUST-A1 | An actual debug-enabled `pumas-rpc` process receives a sentinel HF token and representative failures containing sentinel paths/URLs; captured diagnostics and public responses contain none of those values, while stable typed error codes remain observable. | `system` | `representative` — isolated launcher root and loopback process | `automated` | `satisfied` | [RPC disclosure evidence](reports/rpc-disclosure-evidence.md) |
| RUST-A2 | Every reachable desktop JSON-RPC method and local core IPC operation is admitted through its owning closed DTO contract; wrong protocol versions, unknown operations, missing/extra fields, negative or oversized values, and wrong result types remain distinct typed failures instead of successful defaults. | `contract` | `not-applicable` — deterministic Rust DTO/dispatch fixtures | `automated` | `pending` | Pending Milestone 2 |
| RUST-A3 | The accepted desktop-RPC exposure policy is enforced: either non-loopback binding is unavailable, or an unauthenticated hostile client cannot invoke/read any protected operation or event while an authorized caller can. | `system` | `representative` — isolated real TCP listeners and hostile-client fixtures | `automated` | `satisfied` | [RPC contract and threat model](reports/rpc-contract-and-threat-model.md#accepted-exposure-decision): typed loopback host, real negative CLI process, and real positive listener evidence |
| RUST-A4 | For every event-producing model-index mutation, injected failure cannot commit authoritative state without its durable event; after reopen, event replay and authoritative rows converge before an ephemeral notification is published. | `integration` | `representative` — real temporary SQLite files with controlled fault injection | `automated` | `pending` | Pending Milestone 3 |
| RUST-A5 | Every declared supported prior index schema migrates in deterministic identity/order with integrity checks; interrupted/repeated execution is safe, and unknown or corrupt state returns a typed recovery outcome. | `integration` | `representative` — real SQLite fixtures for every supported prior state | `automated` | `pending` | Pending Milestone 3 |
| RUST-A6 | Rust task and server owners return typed terminal outcomes; shutdown closes admission, signals cancellation, drains under the accepted bound, distinguishes complete/incomplete/failed work, and plugin startup reports the configured root as ready, explicitly disabled/degraded, or failed without path substitution or panic. | `system` | `representative` — real Tokio runtime, loopback servers, active requests, and invalid plugin root | `automated` | `pending` | Pending Milestone 4 and product decision `RUST-I2` |
| RUST-A7 | Each accepted core/RPC feature configuration compiles and runs its applicable tests with the promised public modules and dependency graph present or absent, on every supported Rust target. | `contract` | `required-real` — supported targets supplied by the platform plan | `automated` | `pending` | Pending Milestone 5 and platform evidence |
| RUST-A8 | `pumas-library` contains no UniFFI/Rustler framework dependency, scaffolding, or derives; Rust adapters own their DTO conversions and preserve typed/redacted errors, including distinct paused and cancelled outcomes. | `contract` | `not-applicable` — Cargo graph/source contract and Rust adapter tests | `automated` | `pending` | Pending Milestone 6 |
| RUST-A9 | The affected Rust format, compile, Clippy, test, documentation, feature-matrix, and isolation checks pass without touching a developer launcher root. | `integration` | `representative` — isolated Linux development workspace; target-specific portions use required real targets | `automated` | `pending` | Pending Milestone 7 |

Static search is supporting inventory evidence only. It cannot satisfy secret
non-disclosure, transaction recovery, network trust, lifecycle, or
cross-process claims.

## Scope

### In Scope

- `pumas-rpc` request/outcome DTOs, public error projection, diagnostics,
  transport exposure, admission, and shutdown.
- The separate `pumas-core` primary/client IPC request, response, error, and
  shutdown contract.
- SQLite model-index mutations, durable update events, startup migrations,
  reopen/replay behavior, and their real local test adapter.
- Rust-owned background tasks and fallible blocking-task join outcomes.
- Plugin-loader startup semantics at the Rust RPC composition root.
- `pumas-library` and `pumas-rpc` capability features, dependency ownership,
  public re-exports, and accepted consumer configurations.
- Core-versus-adapter placement for UniFFI/Rustler and Rust-side typed/redacted
  error semantics.
- Rust source/config/tests, focused Rust documentation, and Rust verification
  tooling needed to prove the owned claims.

### Out Of Scope

- Electron/TypeScript generated DTO projections and negative desktop-consumer
  tests; these consume the canonical Rust/server contract and belong to the
  desktop/platform plan.
- Binding host support matrices, generators, generated sources, packaged
  cohorts, host-runtime tests, and release evidence; these belong to the
  desktop/platform plan.
- CI scheduling and the repository-wide permanent-gate inventory, owned by the
  [governance plan](../governance-and-verification/plan.md).
- Frontend model freshness/presentation, accessibility, and UI changes.
- Torch/Python lifecycle or launcher-root durability outside the Rust-owned
  seams.
- General decomposition of the large model-library implementation. Size alone
  does not authorize refactoring.

## Constraints And Assumptions

### Constraints

- The first slice is a security repair and may not wait for the broader RPC
  redesign, but it must establish the error/redaction interface that the next
  slice can deepen rather than add a disposable filter.
- Secret values may not be written to test failure messages, snapshots,
  reports, or logs; use clearly synthetic sentinel values and assert absence.
- Public wire errors are stable, bounded, and redacted. Internal `PumasError`
  may retain operational context inside the owning process but is never a wire
  representation by `Display` conversion.
- SQLite behavior is tested with real temporary SQLite files. Do not add a
  storage trait or in-memory fake that hides transaction, WAL, locking, reopen,
  or migration behavior.
- No dependency is added until the owning milestone records its purpose,
  version authority, feature reachability, security posture, and removal path.
- Tests use temporary launcher roots and isolated ports; they may not discover
  or mutate the user's real library, credentials, processes, or registry.
- Shared Rust owners are changed serially. A milestone may not overlap another
  plan's edits to `rust/Cargo.toml`, `rust/Cargo.lock`, or shared CI config.
- A lower-fidelity unit/compile result does not substitute for a required real
  target, process, network, SQLite recovery, or host-boundary result.

### Assumptions To Validate

- Loopback-only desktop RPC is the accepted product boundary; LAN mode is
  removed and cannot be restored without a new authenticated remote contract.
- The existing `ModelIndex` public surface can remain the storage module
  interface while transaction/event and migration knowledge moves behind it.
- A bounded inventory will identify the accepted `pumas-library` consumer
  configurations; feature names and combinations are not assumed correct merely
  because they already exist.
- The platform plan will provide the binding host disposition and supported
  target matrix before Milestones 5 and 6 close.

## Owned Outcomes

1. One canonical Rust desktop-RPC contract module owns decoded commands,
   typed outcomes, stable public errors, protocol version, and redaction policy.
2. HTTP/SSE/OpenAI transports are protocol adapters and cannot invent defaults
   or expose internal `Display` text. Local core IPC retains its own smaller
   contract because its callers, trust, operations, and lifecycle differ.
3. The model index commits each authoritative mutation with its durable event
   in one recovery unit and publishes ephemeral notification only after commit.
4. Migrations have explicit supported inputs, stable identity/integrity/order,
   and defined interruption/re-entry postconditions without an unneeded generic
   framework.
5. Each Rust task owner has explicit admission, cancellation, join, and typed
   shutdown results; plugin initialization truthfully reports its configured
   authority.
6. Capability names correspond to real dependency, source, public-interface,
   and verification differences for accepted consumers.
7. Binding framework knowledge and wire-safe conversions stay in adapter crates;
   Rust-side errors remain typed and redacted.

## Binding Decisions

| Decision | Owner | Evidence/Reason | Status |
| --- | --- | --- | --- |
| The canonical desktop-RPC DTO and public-error/redaction contract lives in a Rust source module under `rust/crates/pumas-rpc/src/`; `handlers` and HTTP/SSE/OpenAI routing adapt to that interface. | This plan, Milestones 1-2 | R-01/R-07 and RPC/IPC standards; Rust is the producing authority | `accepted` |
| `rust/crates/pumas-core/src/ipc/protocol.rs` remains the canonical local primary/client IPC contract and is not mirrored into the desktop RPC contract. | This plan, Milestone 2 | Different caller, trust, operation, framing, and lifecycle contracts | `accepted` |
| Internal `PumasError` is not serialized by `Display`; each public adapter maps it to a closed error class/code and a bounded safe message. | This plan, Milestones 1-2 and 6 | Security, Diagnostics, IPC, and Language Bindings standards | `accepted` |
| The existing `ModelIndex` interface is the test/caller seam; real local SQLite is the production and test adapter, with test-only fault control kept internal to the module. | This plan, Milestone 3 | R-03/R-04; transaction/recovery behavior must not be faked | `accepted` |
| No universal task supervisor is presumed. Each task-owning module gets the smallest lifecycle interface its actual spawned work requires; consolidation needs a proven shared contract. | This plan, Milestone 4 | R-05 and Rust Async/Concurrency owners | `accepted` |
| UniFFI and Rustler types/conversions/errors belong to their adapter crates; the core stays framework-free. | This plan, Milestone 6 | R-08 and Rust Language Bindings standards | `accepted` |
| Desktop RPC is loopback-only; remove `--allow-lan` and reject every non-loopback `--host`. CORS is not caller authentication. | Product/program owner, consumed by Milestone 2 | R-02 | `accepted` (`RUST-I1`, 2026-09-03) |
| Compiled-out plugin support reports disabled/unavailable; compiled-in configured subsystem root/loader initialization failure fails startup without root substitution or degraded success. | Product/program owner, consumed by Milestone 4 | R-09 | `accepted` (`RUST-I2`, 2026-09-03) |
| Remove Pumas-owned UniFFI, Rustler/Elixir, and false Go surfaces while preserving the public Rust library used by Pantograph's exact-Git dependency. | Desktop/platform plan, consumed by Milestone 6 | R-08 and accepted support matrix | `accepted` (`RUST-I3`, 2026-09-03) |

## Evidence And Oracle Plan

| Claims | Deciding oracle | Independent authority | Unsupported by that evidence | Intended negative failure |
| --- | --- | --- | --- | --- |
| RUST-A1 | Capture an actual debug-enabled RPC binary's diagnostics and responses around synthetic secrets/locators; directly test the public-error mapping. | Public error/redaction interface plus process output | Electron persistence and arbitrary future log sinks | Sentinel appears in stderr/stdout/response, or an internal error becomes public text |
| RUST-A2/A3 | Exhaustive reachable-operation inventory, direct contract fixtures, real HTTP/local-IPC calls, and hostile non-browser clients. | Canonical Rust contract and selected exposure policy | Desktop consumer decoding | Invalid input reaches a domain handler; malformed output becomes success; unauthorized request succeeds |
| RUST-A4/A5 | Real SQLite files, controlled failures at transaction/migration stages, reopen, row/event comparison, and replay. | Model-index interface and migration manifest/identity | Production-disk hardware faults beyond the declared policy | Row without event, event without row, duplicate effect, unknown state accepted, or pre-commit publish |
| RUST-A6 | Active-work lifecycle scenarios observe admission closure, cancellation, joins, deadlines, terminal results, and configured-root startup result. | Owning task/server/plugin interfaces | Electron/frontend/Torch shutdown | Accepted task vanishes, shutdown hangs/claims false completion, root changes, or panic occurs |
| RUST-A7 | Cargo metadata/tree assertions plus compile and applicable behavior tests for every matrix cell on required real targets. | Accepted consumer/configuration matrix | Release artifact composition and binding hosts | Disabled capability remains linked/exported, enabled consumer fails, or target-specific cell is absent |
| RUST-A8 | Core dependency/source scan and adapter conversion/error tests through the adapter interface. | Core and adapter crate manifests/interfaces | Generated host sources, native loading, packaging, and host workflows | Core graph contains binding framework, or adapter collapses/prints internal error state |

## Systemic Finding Audit

- **Invariant family and canonical owners:** desktop RPC disclosure/DTOs are
  owned by the Rust RPC contract; local IPC by `pumas-core::ipc`; durable index
  state by `ModelIndex`; task lifecycle by each spawn owner; feature truth by
  Cargo plus the public module root; host conversion by each binding adapter.
- **Bounded reachable populations:** all registered RPC/HTTP/SSE/OpenAI routes
  and outward error sites; all core IPC methods; all SQL mutation/event and
  startup migration paths; all Rust-owned `spawn`/`spawn_blocking` handles; all
  feature-controlled dependencies/modules/re-exports and workspace consumers;
  all exported UniFFI/Rustler functions and error conversions.
- **Expansion facts:** expand a milestone only when a newly found site shares
  the same invariant and canonical owner. A frontend consumer, host generator,
  release artifact, or Python lifecycle site is handed to its owning plan.
- **Consumer dispositions:** every population entry is migrated, explicitly
  retained with evidence, removed, or assigned to another named plan; counts
  locate entries but do not decide their design.
- **Alternatives considered:** remove LAN instead of adding authentication;
  delete the response wrapper instead of layering validation around it; use one
  SQLite transaction instead of reconciliation after separate commits; keep
  migration machinery store-specific; remove unsupported binding surfaces
  instead of manufacturing support evidence.
- **Evidence-backed stopping condition:** each bounded population has one
  recorded disposition and every owned acceptance claim has its deciding
  scenario evidence in the declared environment.
- **Repaired-composition comparison:** the accepted design must reduce duplicated
  parsing/error/default/migration/lifecycle knowledge. A new registry, adapter,
  or supervisor that merely forwards the old surface fails admission.

## Simplicity And Ownership Review

**Applicability:** `applicable`

- Independent concepts and dimensions: RPC trust, wire decoding, public error disclosure, durable mutation/event consistency, schema evolution, task lifecycle, capability selection, and host conversion can change independently and retain separate module owners.
- State, identity, value, time, policy, and mechanism: SQLite rows/events/migration identities own durable state and order; request IDs/protocol versions own wire identity; task admission/cancellation/deadlines own time; exposure/redaction/capability support are policy; HTTP, local IPC, SQLite, Tokio, and FFI crates are mechanisms behind their owning interfaces.
- Caller and composition-root knowledge: `pumas-rpc::main` chooses exposure, plugin state, and production adapters; handlers know only decoded commands/outcomes; `PumasApi` callers know `ModelIndex` behavior rather than SQL sequencing; binding composition roots choose adapter crates without teaching core about frameworks.
- Representative change paths and forced owners: adding an RPC operation changes the Rust contract plus one handler and downstream generated projection; adding a durable mutation changes the `ModelIndex` implementation and interface test; adding a capability changes its matrix/manifests/module gate; adding a host mapping changes only its adapter unless the domain contract genuinely changes.
- Stable Interfaces versus hidden knowledge: stable surfaces are closed command/outcome/error types, `ModelIndex` operations/recovery promises, typed shutdown results, and accepted feature names; auth internals, SQL statements, tracing text, Tokio handles, serde glue, and host-framework annotations stay hidden.
- Independent evolution, testing, failure, and replacement: contract tests call the same interfaces as production adapters; HTTP and local fixture adapters exercise the RPC seam; real temporary SQLite exercises the index seam; each task owner and binding adapter can fail/test independently without bypassing typed results.
- Necessary complexity and containment: one desktop RPC contract module, the existing distinct local IPC module, the existing model-index module, and owner-local lifecycle interfaces are sufficient; do not add a generic schema registry, storage abstraction, migration framework, or universal supervisor without evidence from two real adapters/consumers.
- Deletion and cumulative machinery result: deleting the RPC contract would redistribute validation/error knowledge across every handler/transport, deleting index transaction ownership would redistribute SQL/event recovery across mutations, and deleting adapter-local conversion would reinfect core; conversely `wrapper.rs`, core UniFFI annotations, false feature markers, temporary plugin fallback, and obsolete pass-through parsing should disappear rather than gain another layer.

## Milestones

### Milestone 0: Reconcile Baseline And Ownership

**Goal:** Turn the audit into one current, routed, executable authority with
cross-plan boundaries and adequate acceptance claims.

**Allowed write set:**

- `docs/plans/current-standards-remediation-2026-09-03/rust-library-and-rpc/plan.md`
- `docs/plans/current-standards-remediation-2026-09-03/rust-library-and-rpc/execution-ledger.md`
- `docs/plans/current-standards-remediation-2026-09-03/rust-library-and-rpc/issues.md`

**Tasks:**

- [x] Recheck audit evidence against the planning baseline.
- [x] Route the current standards and complete the composed-design probe.
- [x] Assign Rust/server DTO and error/redaction ownership here and downstream
  desktop/host evidence to the platform plan.
- [x] Record unresolved decisions without allowing them to block the critical
  first slice.

**Acceptance gate:** The current standards plan checker passes, local links and
referenced source paths resolve, and the ledger records that no implementation
slice has started.

**Status:** `Accepted`

### Milestone 1: Close Critical Diagnostic Disclosure

**Goal:** Establish one usable public error/redaction interface and remove the
known credential disclosure through the real RPC process.

**Allowed write set:**

- `rust/crates/pumas-rpc/src/main.rs`
- `rust/crates/pumas-rpc/src/handlers/**`
- `rust/crates/pumas-rpc/src/contract.rs` (new, if this remains the accepted
  module filename after source reconciliation)
- `rust/crates/pumas-rpc/tests/integration_tests.rs`
- `docs/SECURITY.md`
- this plan, ledger, issues, and `reports/rpc-disclosure-evidence.md`

**Tasks:**

- [x] Inventory request fields and every outward `PumasError`/handler error path
  reachable through the RPC process; classify secret, private locator, safe
  identifier, and bounded public context.
- [x] Define the closed Rust public error class/code and deny-by-default safe
  message projection; keep internal causes/locators in private diagnostics only
  when they are not secrets.
- [x] Remove complete parameter logging. Emit method, request ID, outcome class,
  and correlation data only from an explicit allowlist.
- [x] Route generic JSON-RPC and protocol-specific public errors through the
  redaction interface wherever the inventory proves the same disclosure
  invariant; record non-RPC owners rather than widening silently.
- [x] Modify the real-process test harness to capture bounded stderr, exercise
  `set_hf_token` with a synthetic sentinel and controlled locator failures, and
  assert absence from diagnostics and responses plus presence of stable codes.
- [x] Run focused mapping tests, the real binary scenario, and affected
  `pumas-rpc` format/check/Clippy/default/no-default tests.

**Acceptance gate:** RUST-A1 is satisfied by the process scenario and focused
mapping tests; text search is supporting evidence only.

**Status:** `Accepted`

### Milestone 2: Deepen RPC And IPC Contracts At The Trust Seam

**Goal:** Replace arbitrary JSON/default projection with explicit Rust-owned
contracts and enforce the accepted network exposure policy.

**Allowed write set:**

- `rust/crates/pumas-rpc/Cargo.toml`
- `rust/crates/pumas-rpc/src/main.rs`
- `rust/crates/pumas-rpc/src/server.rs`
- `rust/crates/pumas-rpc/src/contract.rs`
- `rust/crates/pumas-rpc/src/handlers/**`
- `rust/crates/pumas-rpc/src/wrapper.rs`
- `rust/crates/pumas-rpc/tests/integration_tests.rs`
- `rust/crates/pumas-core/src/ipc/**`
- `rust/crates/pumas-core/src/api/hf.rs`
- `rust/crates/pumas-core/src/api/mod.rs` for the exact Milestone 2I
  crate-private required-refresh seam export
- `rust/crates/pumas-core/src/api/models.rs` for the exact Milestone 2I
  refresh/reconciliation caller correction
- `rust/crates/pumas-core/src/api/reconciliation.rs` for the exact Milestone
  2I single-flight admission and success/failure terminal-state correction
- `rust/crates/pumas-core/src/api/state.rs`
- `rust/crates/pumas-core/src/api/state_hf.rs`
- `rust/crates/pumas-core/src/error.rs` for the exact Milestone 2I typed
  operation-in-progress outcome
- `rust/crates/pumas-core/src/index/model_index.rs` for the exact Milestone 2I
  uncapped catalog query
- `rust/crates/pumas-core/src/model_library/download_recovery.rs` for the exact
  PRG-I19 recovery snapshot, stale fingerprint, and managed-target authority
- `rust/crates/pumas-core/src/model_library/download_store.rs` for the exact
  PRG-I19 serialized persistence mutation/revocation owner
- `rust/crates/pumas-core/src/metadata/atomic.rs` and `src/metadata/mod.rs` for
  the exact PRG-I19 Slice A durable-publication Interface and crate-private
  re-export
- `rust/crates/pumas-core/src/model_library/mod.rs` for that bounded public
  recovery-ticket export
- `rust/crates/pumas-core/src/model_library/hf/download.rs` for exact tracked
  destination/repository/file-context matching in the PRG-I19 correction
- `rust/crates/pumas-core/src/model_library/hf/lifecycle.rs` for the exact
  PRG-I19 Slice B download-generation, outer-task, blocking-task, and
  cancellation-finalizer owner
- `rust/crates/pumas-core/src/model_library/hf/types.rs` for the exact private
  state-local recovery capability aggregate and its non-persisted lifecycle
- `rust/crates/pumas-core/src/model_library/library.rs` for the exact Milestone
  2I all-model listing owner and its greater-than-10,000-record oracle
- `rust/crates/pumas-core/tests/api_tests.rs`
- `rust/crates/pumas-uniffi/src/bindings.rs` only as a compile-truthful direct
  exhaustive error projection pending its accepted removal in Milestone 6; no
  new binding behavior is admitted
- `rust/crates/pumas-uniffi/src/bindings/api_hf.rs` only as a compile-truthful
  direct caller pending its accepted removal in Milestone 6; no new binding
  behavior is admitted
- `rust/Cargo.toml`, `rust/crates/pumas-core/Cargo.toml`, and `rust/Cargo.lock`
  for the accepted exact `cap-std = { version = "4.0.3", default-features =
  false }` held-directory dependency and resolved closure only
- `rust/README.md`, `rust/crates/pumas-core/README.md`, `docs/ARCHITECTURE.md`,
  and `docs/SECURITY.md`
- this plan, ledger, issues, and `reports/rpc-contract-and-threat-model.md`

**Tasks:**

- [x] Resolve `RUST-I1` before the first exposure edit: remove LAN support or
  record authenticated capability, operation authorization, credential
  lifecycle, admission/rate, event-stream, and failure contracts.
- [x] Inventory every desktop RPC method and separate HTTP/SSE/OpenAI route,
  plus every local IPC operation and consumer. Record one canonical owner and
  migrate/remove/retain disposition per entry.
- [x] Compare at least two representative contract-module shapes using a
  credential method, a signed pagination/size method, a typed collection
  result, and an event error; select the smaller interface that prevents
  transport/handler knowledge leakage.
- [ ] Decode protocol version and method-specific params before dispatch using
  closed request variants, checked numeric conversion and bounds, declared
  missing/null/extra-field policy, and distinct unknown/unsupported outcomes.
- [ ] Validate typed outcome variants before serialization. Delete the generic
  default-inventing wrapper when its callers are migrated; do not retain it as
  a pass-through compatibility layer.
- [ ] Apply the safe public-error contract to JSON-RPC and the distinct
  protocol adapters. Preserve internal errors for operations without exposing
  `Display` text.
- [ ] Test the contract interface directly and the production HTTP/local-IPC
  adapters with valid, malformed, hostile, unauthorized, and wrong-outcome
  cases.
- [ ] Publish the stable Rust DTO/error handoff to the platform plan. That plan
  owns generated TypeScript projection and negative Electron consumer proof.

**Acceptance gate:** RUST-A2 and RUST-A3 are satisfied, and the platform plan
has an explicit handoff revision for downstream projections.

**Status:** `Active`

### Milestone 3: Make Index Mutation, Events, And Evolution Recoverable

**Goal:** Put authoritative SQLite mutations and durable events in one recovery
unit and make every supported schema transition identifiable and re-entrant.

**Allowed write set:**

- `rust/crates/pumas-core/src/index/model_index.rs`
- `rust/crates/pumas-core/src/index/model_index/**`
- `rust/crates/pumas-core/src/index/mod.rs`
- `rust/crates/pumas-core/tests/api_tests.rs`
- `rust/crates/pumas-core/tests/model_index_persistence.rs` (new)
- `rust/crates/pumas-core/README.md` and `docs/ARCHITECTURE.md`
- this plan, ledger, issues, and `reports/model-index-recovery-evidence.md`

**Tasks:**

- [ ] Inventory all SQL write paths, including upsert/delete, dependency
  profiles, package-facts cache, overlays, repairs, and every durable/ephemeral
  update publication. Give each one a transaction/event disposition.
- [ ] Move each authoritative mutation and its durable event into one SQLite
  transaction behind the existing `ModelIndex` interface; publish broadcast
  updates only after successful commit.
- [ ] Build supported-prior-schema fixtures from repository history and current
  upgrade code. Explicitly declare the oldest supported state and reject
  unknown/corrupt states rather than guessing.
- [ ] Select the smallest store-specific migration identity/integrity/applied-
  state mechanism after probing the real current schemas. Record the decision
  before implementation; do not assume a universal version table or framework.
- [ ] Define transaction and interruption postconditions for every migration,
  including table rebuilds, and make retry/reopen deterministic.
- [ ] Test through the `ModelIndex` interface with real temporary SQLite files
  and internal controlled failure points at mutation, event, commit, migration,
  reopen, and publish stages.

**Acceptance gate:** RUST-A4 and RUST-A5 are satisfied; the recovery report maps
every supported input and mutation family to deciding evidence.

**Status:** `Planned`

### Milestone 4: Make Startup, Task Ownership, And Shutdown Truthful

**Goal:** Ensure accepted work and configured plugin state cannot disappear
behind aborted handles, empty results, false readiness, or an unrelated root.

**Allowed write set:**

- `rust/crates/pumas-core/src/api/runtime_tasks.rs`
- `rust/crates/pumas-core/src/api/builder.rs`
- `rust/crates/pumas-core/src/ipc/server.rs`
- `rust/crates/pumas-core/src/plugins/**`
- `rust/crates/pumas-core/src/error.rs`
- `rust/crates/pumas-core/tests/api_tests.rs`
- `rust/crates/pumas-rpc/src/main.rs`
- `rust/crates/pumas-rpc/src/server.rs`
- `rust/crates/pumas-rpc/src/handlers/plugins.rs`
- `rust/crates/pumas-rpc/tests/integration_tests.rs`
- `rust/README.md`, `rust/crates/pumas-core/README.md`, and
  `docs/ARCHITECTURE.md`
- this plan, ledger, issues, and `reports/rust-lifecycle-evidence.md`

**Tasks:**

- [ ] Inventory each Rust-owned `spawn`/`spawn_blocking`, its admission point,
  handle owner, cancellation signal, natural completion, error/panic path,
  deadline, and shutdown postcondition. Expand only within the same owner.
- [ ] Replace pruning/aborting/discarding with owner-local typed terminal
  outcomes. Preserve blocking-task join failure rather than fabricating an
  empty result.
- [ ] Close server admission, signal connection/request tasks, drain within a
  recorded operational bound, await owned handles, and return
  complete/incomplete/failed shutdown. Keep `Drop` best-effort and separate
  from the explicit async contract.
- [ ] Resolve `RUST-I2`, then make plugin startup use only the configured root
  and return the selected ready/disabled/degraded/failed state without unwrap.
- [ ] Exercise task success/error/panic/cancel, spawn versus shutdown races,
  active RPC/IPC connections, repeated shutdown, timeout/incomplete outcome,
  and invalid/unreadable plugin roots on a real Tokio runtime.

**Acceptance gate:** RUST-A6 is satisfied; every inventoried handle has a
terminal disposition and the plugin state is observable through its owner.

**Status:** `Planned`

### Milestone 5: Make Core And RPC Capabilities Real Configurations

**Goal:** Align capability names with dependency, source, public-interface, and
consumer behavior for every accepted configuration.

**Allowed write set:**

- `rust/Cargo.toml` and `rust/Cargo.lock`
- `rust/crates/pumas-core/Cargo.toml`
- `rust/crates/pumas-core/src/lib.rs`
- `rust/crates/pumas-core/src/api/**`
- `rust/crates/pumas-core/src/model_library/hf/**`
- `rust/crates/pumas-core/src/process/**`
- `rust/crates/pumas-core/src/system/**`
- `rust/crates/pumas-core/src/onnx_runtime/**`
- other core source files only after they are named in the approved
  feature-to-code inventory in the ledger
- `rust/crates/pumas-app-manager/Cargo.toml`
- `rust/crates/pumas-rpc/Cargo.toml`, `rust/crates/pumas-rpc/src/main.rs`, and
  `rust/crates/pumas-rpc/src/server.rs`
- `scripts/rust/check.sh`
- `rust/README.md` and `rust/crates/pumas-core/README.md`
- this plan, ledger, issues, and `reports/rust-feature-matrix.md`

**Tasks:**

- [ ] Inventory real workspace/published consumers and map each accepted
  capability to dependencies, modules, public re-exports, runtime behavior,
  platform restrictions, tests, and incompatible combinations.
- [ ] Decide whether current names (`hf-client`, `process-manager`,
  `gpu-monitor`, `full`, and RPC `inference-plugins`) express those consumers;
  record any public compatibility change before editing.
- [ ] Make optional dependencies and source/public gates agree. Do not call a
  marker optional when heavy HF, archive, ONNX, process, GPU, watcher, or
  platform dependencies remain reachable without an accepted reason.
- [ ] Preserve a working `pumas-rpc --no-default-features` library-only build
  and prove plugin-enabled behavior separately.
- [ ] Add matrix commands that check both dependency exclusion and consumer
  compilation/behavior; coordinate their permanent CI schedule with governance.
- [ ] Obtain required-real results for each supported Rust target from the
  platform target matrix; missing environments leave RUST-A7 pending.

**Acceptance gate:** RUST-A7 is satisfied for every accepted matrix cell, with
Cargo tree/metadata and applicable behavior proof rather than compilation alone.

**Status:** `Planned`

### Milestone 6: Restore The Rust Core/Binding Adapter Seam

**Goal:** Remove host-framework knowledge from core and make Rust adapter error
semantics truthful without claiming unsupported hosts.

**Allowed write set:**

- `rust/Cargo.toml` and `rust/Cargo.lock`
- `rust/crates/pumas-core/Cargo.toml`
- `rust/crates/pumas-core/src/lib.rs`
- `rust/crates/pumas-core/src/models/**`
- `rust/crates/pumas-core/src/model_library/types.rs`
- `rust/crates/pumas-core/src/model_library/dependencies.rs`
- `rust/crates/pumas-uniffi/Cargo.toml` and `rust/crates/pumas-uniffi/src/**`
- `rust/crates/pumas-rustler/Cargo.toml` and
  `rust/crates/pumas-rustler/src/lib.rs`
- `scripts/rust/check.sh`
- `rust/README.md`, `rust/crates/pumas-core/README.md`, and
  `docs/native-bindings.md`
- this plan, ledger, issues, and `reports/rust-binding-boundary.md`

**Tasks:**

- [ ] Consume the platform plan's supported-host/disposition matrix before
  changing public binding claims; record unsupported/deferred hosts explicitly.
- [ ] Remove the core UniFFI dependency/feature/scaffolding/derives and complete
  adapter-local records/conversions through the actual UniFFI interface.
- [ ] Define typed, bounded, redacted Rust adapter errors. Preserve paused,
  cancelled, invalid, unavailable, and internal outcomes distinctly.
- [ ] Resolve `RUST-I3`: remove Rustler's false core dependency/claim if no real
  host consumer is accepted, or implement only the bounded core adapter surface
  accepted by the platform plan. Do not build speculative host breadth.
- [ ] Test Rust conversions and error projection through adapter interfaces and
  assert the core dependency/source graph is framework-free.
- [ ] Hand host generation, native loading, packaged cohort, async host call,
  negative host input, and release proof back to the platform plan.

**Acceptance gate:** RUST-A8 is satisfied. Host support/release claims remain
outside this plan and cannot be inferred from Rust-only tests.

**Status:** `Planned`

### Milestone 7: Rust Objective Acceptance

**Goal:** Re-run all owned deciding evidence, reconcile documentation and
downstream handoffs, and accept only the claims actually proved.

**Allowed write set:**

- `scripts/rust/check.sh`
- `rust/README.md`
- `rust/crates/pumas-core/README.md`
- `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, and `docs/native-bindings.md`
- this plan, ledger, issues, and `reports/**`

**Tasks:**

- [ ] Run each objective's deciding scenario in its declared environment and
  link exact commands/results without copying raw secrets or unbounded logs.
- [ ] Run affected Rust format, compile, Clippy, test, doc, feature-matrix, and
  isolation checks; classify failures instead of weakening gates.
- [ ] Reconcile source module documentation and the concise project guides with
  accepted contracts/configurations.
- [ ] Confirm the platform plan has the canonical RPC revision and binding/target
  handoffs; do not mark its downstream claims satisfied here.
- [ ] Review all issues, reports, re-plan triggers, and objective rows before
  changing lifecycle state.

**Acceptance gate:** RUST-A1 through RUST-A9 are satisfied with linked evidence;
no required-real environment is represented by a lower-fidelity substitute.

**Status:** `Planned`

## Dependencies And Coordination

- The [governance plan](../governance-and-verification/plan.md) owns permanent
  gate inventory and CI scheduling. Its CI edits must land before this plan adds
  new schedules to avoid shared-file conflict.
- The desktop/platform plan (planned sibling
  `../desktop-release-bindings-and-torch/plan.md`) consumes the canonical Rust
  DTO/error revision and owns Electron/TypeScript generation, negative desktop
  consumer tests, host support, packaged cohorts, and release evidence. It
  supplies the target/host matrix and Rustler disposition back to Milestones
  5-6.
- Frontend/UI remediation consumes downstream desktop projections; it does not
  edit the Rust contracts.
- Milestones 1-6 run serially because they overlap Rust composition roots,
  manifests, public errors, or tests. Independent downstream work starts only
  after a versioned handoff, not against a moving schema.

## Risks

- **Credential regression:** another method or protocol-specific error can
  bypass the first projection. Mitigation: systemic outward-site inventory plus
  process-level sentinel evidence.
- **Consumer breakage:** strict DTOs expose latent Electron/TypeScript drift.
  Mitigation: versioned Rust handoff and downstream generated/negative tests;
  no silent compatibility defaults.
- **Remote exposure ambiguity:** retaining LAN mode materially expands auth,
  credential lifecycle, rate, and event-stream work. Mitigation: decide remove
  versus authenticate before Milestone 2 edits.
- **Migration/data loss:** unknown deployed schemas or interrupted rebuilds may
  exceed current fixtures. Mitigation: derive supported states from history and
  block acceptance on unknown/corrupt recovery evidence.
- **Shutdown hangs or loss:** draining without accepted bounds may hang, while
  aborting may lose work. Mitigation: explicit admission/cancellation/deadline
  policy and typed incomplete outcomes.
- **Feature compatibility:** making markers real can change public API and
  transitive dependencies. Mitigation: consumer matrix and compatibility
  decision before source gating.
- **Binding scope expansion:** host claims can pull generator/release work into
  Rust refactoring. Mitigation: platform-owned matrix and strict adapter-only
  ownership here.

## Blockers

- None for the admitted Milestone 1 slice.
- `RUST-I1` is resolved; Milestone 2 enforces the accepted loopback-only
  boundary.
- `RUST-I2` is resolved; Milestone 4 must prove the accepted startup behavior.
- `RUST-I3` is resolved; Milestone 6 owns Rust source/manifests removal while
  the platform plan owns binding scripts, docs, and release projection.

## Re-Plan Triggers

- The disclosure inventory shows secret propagation outside the current
  `pumas-rpc` owner or requires changing a downstream consumer contract in the
  first slice.
- The LAN decision retains remote access but lacks an accepted credential and
  authorization owner, or introduces browser-session assumptions.
- Representative RPC design shows one closed contract would combine unrelated
  HTTP/SSE/OpenAI/local-IPC promises or merely move the existing dispatcher.
- SQLite capability/history cannot support the proposed atomic event or stable
  migration mechanism without a data-format compatibility decision.
- Lifecycle inventory finds an owner outside this plan or an operation whose
  cancellation/deadline semantics require product policy.
- Feature inventory identifies external consumers or incompatible combinations
  absent from the accepted matrix.
- The platform plan changes supported targets/hosts, rejects the Rust contract
  handoff, or requires core domain changes beyond adapter conversion.
- A proposed module fails the deletion test, introduces a hypothetical seam,
  or increases cumulative parsing/default/supervision machinery.

## Final Acceptance

- Acceptance status: `pending`
- Deferred follow-ups: Electron/TypeScript projections and negative consumer
  tests, real binding-host cohorts, generators, packaged artifacts, and release
  evidence remain with the desktop/platform plan; frontend presentation remains
  with the frontend/UI plan.
- Final status: `Active`
