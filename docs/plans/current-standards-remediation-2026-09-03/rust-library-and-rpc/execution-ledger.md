# Rust Library and RPC Execution Ledger

## Baseline

- Audit code: `a33c8c0efa7cd8783c7deeac9e608db205290d43`.
- Planning code: `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`.
- Audit standards: `52b096ded9c53afd439a3cf0efc4cc85252da570`.
- Planning standards: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`.

## Current State

- Plan lifecycle: `Active`.
- Acceptance: `partial`.
- Milestone 0 planning/ownership reconciliation is accepted.
- Milestone 1's RPC diagnostic-disclosure slice is accepted; `RUST-A1` is
  satisfied by the linked real-process and interface evidence.
- Milestone 2 is active for loopback desktop-RPC and local core-IPC producer
  contract and lifecycle work following explicit program admission. `RUST-I1`
  is resolved by loopback-only enforcement. Selected desktop contracts and
  Linux artifact-collision correction are accepted; full milestones remain open.

## Slice Log

### 2026-09-05 — Verified Ambient pre-settlement restore corrected

- Exact source: `hf/download.rs`, SHA-256
  `fb6a783526023f6080c7b2c5135fb78cf5027673530bf5ef472768d51df6c4c4`.
  Reuses the existing retained recovery task, blocking-operation owner, and
  exact-attempt queue settlement; no store schema or public API change.
- RED: `cargo test --offline -p pumas-library --lib
  restore_settles_verified_ambient_cleanup_before_restoring_follower` rejected
  the real current-format persisted cutpoint with the existing unresolved
  quarantine error. GREEN restores read-only Error history, settles only the
  verified head, preserves follower ordinal/predecessor, allows its completion,
  and reopens with only terminal history. Fixture bytes do not claim a network
  transfer or process-kill test.
- Added exact-diagnostic Pending refusal with unchanged queue/payload evidence,
  and a blocked real store transition proving caller cancellation does not
  discard settlement ownership. The initial test's Completed-task retirement
  wait was corrected to use the existing observation method; production
  completion was already reached and no lifecycle contract was weakened.
- Independent review found that the returned inventory also needed validation.
  Both snapshots now reject unresolved custody; the final one additionally
  rejects any quarantine still holding an active admission. Revised production
  review passes. This does not claim general concurrent restore or client Drop.
- Focused `--lib restore_`: 6/6 pass. Full
  `cargo test --offline -p pumas-library`, default and `--no-default-features`:
  each 1,191 pass (1,088 unit plus 103 integration/doc), 11 existing ignores.
  Evidence: `/tmp/pumas-verified-ambient-restore-focused.log` and
  `/tmp/pumas-verified-restore.dc87Q6/`. Runs use Linux and isolated fixtures;
  full package tests use the permitted real local socket/process environment.
- Final supporting gates pass: `cargo clippy --offline -p pumas-library
  --all-targets --all-features -- -D warnings`, the same command with
  `--no-default-features` instead of `--all-features`, `cargo fmt --all --
  --check`, diff whitespace, and all five plan contracts. No suppression or
  hook bypass. The checkpoint is accepted; whole C3 remains unaccepted.
- Live `launcher-data/downloads.json` SHA-256 remains
  `a0885e5fde0fc5f7c68f3c8726d8677bbbec73a9d030d92a945cce244d3b1575`.
  No live download, file migration, GUI rerun, or cross-target proof is claimed.
  Remaining Pending/Recovery/hidden restore and C4 importer ownership stay open.

### 2026-09-05 — Current authority reconciled; Verified Ambient restore admitted

- Source `2b9553a0`, standards `1609c304`. Continued the canonical focused plan;
  replaced stale selected-consumer, cutover, collision, and HF-unavailable holds
  with the recorded accepted boundaries. Remaining plugin-startup work is not
  complete merely because its product decision is resolved.
- Inspection found a current-format restart gap: Verified Ambient quarantine
  still carrying its exact admission is refused even though cleanup is already
  durably verified. Admit only `hf/download.rs` and its regressions, reusing
  retained restore-task ownership, exact settlement, and strict inventory reload.
- Required evidence: real temporary store operations reaching the persisted
  pre-settlement cutpoint; restored Error history without mutation capability;
  exact follower progress; unchanged Pending/Recovery/hidden refusal. This is
  neither process-kill evidence nor full unresolved-state recovery acceptance.
- C4 remains subsequent work: builder importer callbacks are not yet awaited
  before HF lifecycle settlement. No live download/migration tests or cross-target
  claims are admitted. Parent and all four focused plan-contract checks pass.

### 2026-09-03 — Start `plan.md`; Milestone 1 RPC disclosure containment

- Start operation: explicit implementation start of
  `docs/plans/current-standards-remediation-2026-09-03/rust-library-and-rpc/plan.md`.
- Source baseline: `453105780b1e5181d27dd1f20b234591bb6ead86`.
- Active allowed write set: `rust/crates/pumas-rpc/src/main.rs`,
  `rust/crates/pumas-rpc/src/handlers/**`, optional new
  `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, `docs/SECURITY.md`, and
  this focused plan's plan, ledger, issues, and
  `reports/rpc-disclosure-evidence.md`.
- Decision: the Rust RPC producer owns a deny-by-default public error
  projection; transports must not serialize internal `Display` text.
- Decisions: `contract.rs` is the producer seam; unknown/internal failures deny
  disclosure by default. JSON-RPC, RPC SSE, handler-local `PumasError` bodies,
  OpenAI-owned failures, and provider non-success bodies use bounded public
  messages/classes/codes. Complete request parameters never enter diagnostics.
- Verification: focused error mapping passed under default and no-default
  configurations; the real debug child-process credential/path/URL sentinel
  scenario passed under both; full default tests passed (70 unit, 9
  integration, 10 existing manual tests ignored); full no-default tests passed
  (33 unit, 6 integration, 10 existing manual tests ignored); format, default
  check, and default/no-default all-target Clippy with `-D warnings` passed.
- Corrected verification finding: the first default Clippy run rejected two
  collapsible conditionals introduced during diagnostic cleanup; the forms
  were collapsed and the exact Clippy command then passed.
- Supporting evidence: disclosure searches and `git diff --check` passed.
- Evidence: [RPC diagnostic disclosure evidence](reports/rpc-disclosure-evidence.md).
- Residual risk: LAN authentication and the complete closed RPC/IPC DTO
  contract remain explicitly pending Milestone 2; server task lifecycle
  diagnostics remain owned by Milestone 4.
- Handoff: `RUST-A1` is ready for program/governance/frontend sequencing. No
  shared manifests, CI, generated artifacts, or package scripts were changed.

### 2026-09-03 — Milestone 2A RPC/IPC inventory and contract-shape decision

- Admission: explicit `/root` direction after `RUST-A1` acceptance; loopback
  RPC and local IPC contract work may proceed independently of LAN policy.
- Baseline: shared worktree at the accepted Milestone 1 handoff; exact Rust
  source changes remain unstaged and uncommitted under the integration owner.
- Active allowed write set: this focused plan/ledger/issues and
  `reports/rpc-contract-and-threat-model.md` only.
- Excluded in this slice: all source, manifests, network exposure, CI,
  generated artifacts, package scripts, and shared project documentation.
- Goal: bound every reachable producer operation/route and local IPC consumer,
  assign its canonical owner/disposition, compare at least two contract-module
  shapes, and select the smallest deep interface for the next source slice.
- Evidence/results: accepted. The producer population is 151 desktop JSON-RPC
  operations, ten other desktop HTTP routes, and 108 local IPC operations.
  The Electron registry has one stale, unreachable name
  (`resolve_pumas_model_ref`) for downstream removal. Only six local IPC
  operations have an evidenced `PumasLocalClient` consumer; retain those and
  remove the other 102 from transport reachability. Three representative
  shapes were compared; closed producer command/outcome variants were selected
  over a shallow runtime registry or a cross-protocol mega-enum.
- Exposure decision: `RUST-I1` was escalated with the smallest recommended
  choice—remove `--allow-lan` and reject non-loopback hosts. No exposure source
  was edited.
- Evidence: [RPC and local IPC contract inventory](reports/rpc-contract-and-threat-model.md).
- Re-plan trigger: any route/consumer that cannot share the proposed invariant
  without leaking protocol-specific or lifecycle knowledge.

### 2026-09-03 — Milestone 2B closed local IPC contract

- Active allowed write set:
  `rust/crates/pumas-core/src/ipc/protocol.rs`,
  `rust/crates/pumas-core/src/ipc/server.rs`,
  `rust/crates/pumas-core/src/ipc/client.rs`,
  `rust/crates/pumas-core/src/ipc/local_client.rs`,
  `rust/crates/pumas-core/src/ipc/mod.rs`,
  `rust/crates/pumas-core/src/api/state.rs`,
  `rust/crates/pumas-core/tests/api_tests.rs`, this focused plan/ledger/issues,
  and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: desktop RPC source, all manifests, network exposure,
  CI, generated artifacts, package scripts, and shared project documentation.
- Goal: decode only the six retained local commands before dispatch, enforce
  exact field and numeric/collection bounds, validate typed outcomes before
  writing a frame, make the low-level TCP API crate-internal, and prove the
  contract plus real TCP adapter with negative cases.
- Evidence/results: accepted. `LocalIpcOperation` admits only the five unary
  and one streaming operations exposed by `PumasLocalClient`; all 102 legacy
  names now fail at the real server with method-not-found before `PrimaryState`
  dispatch. Commands enforce exact top-level/nested field sets, non-empty and
  bounded credentials/identifiers/cursors, batch size, selector numeric type,
  and selector limit. Typed results are decoded and normalized before a frame
  is written. Error frames carry stable codes/classes and bounded safe text;
  batch item and artifact diagnostics are redacted. Low-level client, protocol,
  server, and dispatch types are crate-internal, and public clients no longer
  implement `Debug` over registry credentials.
- Verification:
  - `cargo test -p pumas-library ipc --no-fail-fast` passed 34 IPC tests plus
    the real `PrimaryState` TCP hostile-client test selected by the filter.
  - The exact production-adapter invalid-token/obsolete-operation integration
    test passed independently.
  - The added response version/ID/ambiguous-outcome contract test passed in a
    focused rerun after its addition.
  - `cargo fmt --package pumas-library -- --check` passed.
  - `cargo clippy -p pumas-library --all-targets -- -D warnings` passed.
  - Focused `git diff --check` passed.
- Negative evidence includes malformed JSON, wrong protocol/envelope,
  missing/null/extra fields, negative/oversized numeric input, oversized
  batches, unknown/removed operation names, wrong dispatcher outcome types,
  invalid credentials, response correlation, and bounded redaction.
- Residual scope: `PrimaryState` still contains the obsolete internal match
  branches, but the only production transport parser and low-level caller are
  closed before dispatch. Removing those unreachable implementations is not
  required to preserve the supported six-operation Interface and can occur
  with later `PrimaryState` owner work without blocking this transport gate.

### 2026-09-03 — Milestone 2C closed desktop request admission

- Active allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`, selected files below
  `rust/crates/pumas-rpc/src/handlers/` only as their domain group migrates,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger/issues, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, network exposure, local core IPC, CI,
  generated artifacts, package scripts, and shared project documentation.
- Goal: close method/version/params admission at the producer before dispatch,
  then migrate built-in/status/system and HF credential operations through
  typed requests without serializing or debugging credentials.
- Evidence/results: accepted for request admission. `AdmittedRpcRequest`
  distinguishes JSON syntax, envelope, method, and params failures; validates
  version, exact envelope fields, bounded method/ID forms, and typed params;
  and owns 18 feature-complete commands (17 without inference plugins). Empty
  params, optional booleans with declared aliases, bounded non-empty app IDs,
  and bounded non-empty credentials have explicit policies. Unknown methods
  return method-not-found without entering a domain handler.
- Credential boundary: `SecretToken` implements neither `Debug` nor `Display`.
  `set_hf_token` passes it directly to the core API instead of reconstructing a
  generic handler `Value`; the obsolete generic auth handler module and legacy
  dispatcher branches were deleted.
- Adapter: `/rpc` now admits the exact request body itself and always returns a
  typed JSON-RPC parse/request/method/params error (`-32700`, `-32600`,
  `-32601`, or `-32602`) for the covered hostile cases. Selected commands are
  normalized before execution. Unmigrated methods are visibly represented by
  the temporary `Legacy` command and remain pending domain-by-domain removal.
- Verification: six direct contract tests passed; the real child-process HTTP
  negative adapter test passed with default and no-default features; default
  full tests passed 74 unit plus 10 integration (10 declared manual tests
  ignored); no-default full tests passed 37 unit plus 7 integration (10
  declared manual tests ignored). Default/no-default all-target check and
  Clippy with warnings denied passed, as did format and focused diff checks.
- Residual scope: typed result variants and deletion of the generic wrapper are
  explicitly Milestone 2D. Other domain params remain in `Legacy` until each
  owner group migrates. Network exposure remains untouched and gated by
  `RUST-I1`.

### 2026-09-03 — Aggregate Rust failure diagnosis boundary

- Trigger: the governance aggregate passed format, check, and Clippy, then
  reported 11 workspace-test failures. The two independent roots were
  `api::models::tests::get_inference_settings_batch_reports_per_model_errors`
  with SQLite `attempt to write a readonly database` and
  `tests::test_api_creation` with `Operation not permitted (os error 1)`; the
  other nine failures followed a poisoned registry-test lock.
- Diagnostic write set: no repository source or shared verification files.
  This focused ledger and `reports/rpc-contract-and-threat-model.md` record the
  bounded evidence. A supplemental probe under `/tmp` did not execute because
  its isolated Cargo dependency setup could not use the cache in the sandbox;
  it was abandoned rather than treated as evidence.
- Tight roots: each exact failing test passed independently once with
  `--exact --nocapture` (one test passed per command).
- Reproduction increases: the complete `pumas-library` unit suite passed once
  with `--test-threads=1`, once at default concurrency, and once with
  `--test-threads=128` (859/859 each). Two simultaneous complete library unit
  suites both passed (1,718 tests total). Twelve simultaneous exact
  `tests::test_api_creation` processes all passed (12/12).
- Original command: the exact aggregate workspace test stage
  `cargo test --manifest-path rust/Cargo.toml --workspace --exclude pumas_rustler --quiet`
  passed in isolation. Its relevant totals were 78 `pumas-app-manager` unit,
  859 `pumas-library` unit, and 34 `pumas-library` API integration tests; all
  subsequent workspace test binaries also passed, with only their declared
  ignored tests ignored.
- Classification: non-reproducible concurrent-environment interference. The
  crate-global registry override and the environment-variable override remain
  plausible test-isolation hazards, while the one `EPERM` remains consistent
  with transient process/socket policy pressure, but no candidate produced a
  red-capable repository command. The nine poisoned-lock failures are cascades,
  not independent roots.
- Independent environment check: governance reproduced the 848/11 pattern in
  its restricted sandbox, with the independent `EPERM` moving from API creation
  to a migration test. The exact isolated aggregate then passed with the
  environment-required elevated permissions: `./scripts/rust/check.sh` exited
  zero after format, check, Clippy, all workspace tests/doctests, and the
  no-default check.
- Decision: classify the aggregate failure as environment-local IPC denial and
  resulting shared-test-state interference. No speculative source repair.
  Governance released the stable boundary and Milestone 2C may resume.

### 2026-09-03 — Milestone 2D typed desktop outcomes

- Active allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/status.rs`, selected files below
  `rust/crates/pumas-rpc/src/handlers/models/` only if their next domain group
  is admitted, `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger/issues, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, network exposure, local core IPC, CI,
  generated artifacts, package scripts, and shared project documentation.
- Goal: construct and validate typed result variants for the 18 commands
  admitted by Milestone 2C, bypass/remove their generic wrapper knowledge, and
  only then admit the next non-plugin domain group.
- Evidence/results: accepted. All 18 commands from Milestone 2C now construct a
  closed `RpcOutcome`; complex domain payloads are boxed to keep the enum
  bounded. Status/system handlers return their concrete core types, launcher
  version JSON is decoded through an exact producer DTO, and local status,
  shutdown, sandbox, mutation, and auth shapes are explicit. These commands no
  longer enter `wrap_response`.
- Next exact domain: all nine link operations now have exact aliased param
  DTOs, bounded non-empty model/app/version/path identifiers, a non-empty
  512-item file-list maximum, and typed core/local outcomes. Their legacy
  dispatcher branches were removed. The contract therefore owns 27 typed
  commands with inference plugins and 26 without them.
- Verification: ten direct contract tests passed under default and no-default
  features. The production child-process HTTP adapter passed under both modes
  with actual loopback binding using the environment-required elevated test
  execution; its cases include invalid link params and a valid typed link-
  health result. Full default tests passed 78 unit plus 10 integration (10
  declared manual tests ignored); full no-default tests passed 41 unit plus 7
  integration (10 declared manual tests ignored). Format, default/no-default
  check, default/no-default all-target Clippy with warnings denied, and focused
  diff checks passed.
- Corrected verification finding: the first Clippy run rejected the initially
  direct large `RpcOutcome` payloads. Complex domain variants were boxed; both
  exact Clippy commands then passed without an allow.
- Residual scope: every remaining `Legacy` request/outcome and the wrapper are
  still pending domain-by-domain migration. Exposure was unchanged throughout
  this accepted slice.

### 2026-09-03 — Accept Milestone 2E loopback-only exposure

- Product decision: remove `--allow-lan` and reject every non-loopback
  `--host`. The user explicitly accepted the previously recommended smallest
  disposition; no remote authentication design is required.
- Allowed write set: `rust/crates/pumas-rpc/src/main.rs`,
  `rust/crates/pumas-rpc/src/server.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger/issues, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, local core IPC, other handlers, CI,
  generated artifacts, package scripts, and shared project documentation.
- Implementation: `LoopbackHost` has a private `IpAddr` field and can only be
  constructed by parsing a numeric loopback address. `start_server` accepts
  that type rather than a string, so a remote bind is not representable at the
  listener seam. The CLI parses it before runtime/API initialization and no
  longer defines `--allow-lan`.
- Negative system evidence: the real RPC binary rejected `--host 0.0.0.0`
  before printing `RPC_PORT`; it also rejected the removed `--allow-lan` flag.
  The same child-process test passed under default and no-default features.
- Positive system evidence: the production child-process HTTP adapter bound an
  actual loopback listener and completed a request under default and no-default
  features using the environment-required elevated execution.
- Verification: the focused host unit test passed; post-change full RPC suites
  passed 76 unit plus 11 integration under default features and 39 unit plus 8
  integration under no-default features (10 explicitly manual tests ignored in
  each mode). Default/no-default all-target Clippy with warnings denied,
  default/no-default check, format check, and focused diff check passed.
- Result: accepted; `RUST-A3` is satisfied. A future remote-access product must
  establish a new authenticated authorization and credential-lifecycle
  contract rather than reopening the removed flag.

### 2026-09-03 — Accept Milestone 2F conversion-domain contract

- Allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/conversion.rs`,
  `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, plugin commands/loaders, local core IPC,
  CI, generated artifacts, package scripts, and shared project documentation.
- Implementation: all nine conversion commands now use exact producer-owned
  DTOs and closed outcomes. Direction/backend names use an enumerated accepted
  spelling set; IDs, output names, quant names, and calibration paths are
  bounded and non-empty when present; aliases are declared and duplicates,
  extra fields, null/wrong types, and unknown enum values fail admission.
- The conversion handler receives validated domain values rather than JSON.
  All nine legacy dispatcher arms and the seven obsolete wrapper entries were
  removed. The producer contract now owns 36 commands with inference plugins
  and 35 without them.
- Disclosure correction: conversion workers retain internal error detail, but
  polling/list outcomes project only a stable public failure sentence. A direct
  sentinel test proves credential and private-path text cannot escape through
  a conversion-progress result.
- Verification: two focused contract tests passed under both feature modes.
  The production child-process HTTP adapter passed with actual loopback binding
  in both modes and covered invalid conversion direction, duplicate aliases,
  extra fields, and a typed missing-progress outcome. Full default tests passed
  78 unit plus 11 integration; full no-default tests passed 41 unit plus 8
  integration (10 explicitly manual tests ignored in each mode). Both all-
  target Clippy commands with warnings denied, default/no-default checks, and
  format passed.
- Result: accepted. Residual `Legacy` operations remain and prevent a stable
  platform projection handoff.

### 2026-09-03 — Accept Milestone 2G OS-open contract

- Allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/process.rs`,
  `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, plugin-gated process commands/loaders,
  local core IPC, CI, generated artifacts, package scripts, and shared project
  documentation.
- Implementation: `open_path` and `open_url` now enter through exact bounded,
  non-empty request DTOs. Their handler owns the environment-dependent
  canonical-path and HTTP(S)-scheme validation and receives validated strings
  rather than JSON. OS-launch failure projects the shared stable operation
  outcome; no internal `Display` text is returned.
- Both legacy dispatcher arms and wrapper entries were removed. The shared
  `OperationStatusOutcome` is used only for commands with the identical
  `{success, error?}` contract, and the producer now owns 38 typed commands
  with inference plugins and 37 without them.
- Verification: two direct request/outcome tests passed in both feature modes.
  The real child-process HTTP adapter rejected a non-web URL and a nonexistent
  local path with typed invalid-params errors under both modes. Full default
  tests passed 80 unit plus 11 integration; full no-default tests passed 43
  unit plus 8 integration (10 explicitly manual tests ignored in each mode).
  Both all-target warnings-denied Clippy commands, format, and focused diff
  checks passed.
- Result: accepted. Residual `Legacy` operations remain and prevent the stable
  platform projection handoff.

### 2026-09-03 — Accept corrected Milestone 2H model-download lifecycle contract

- Allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/models/downloads.rs`,
  `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, other model/search/metadata operations,
  plugin commands/loaders, local core IPC, CI, generated artifacts, package
  scripts, and shared project documentation.
- Implementation: all ten download lifecycle commands now use exact requests
  and closed outcomes. Required and optional identifiers are bounded and
  non-empty, selected file lists are non-empty and capped at 512, card JSON is
  capped at 1 MiB, aliases are declared, and duplicate/extra/null/wrong fields
  fail before dispatch. Handlers receive typed requests and no longer parse
  JSON. Ten legacy arms and eight obsolete wrapper entries were removed.
- Outcome validation: progress/list entries replace worker error detail with a
  stable public sentence. Partial recovery admits only `resume`, `recover`,
  `attach`, or `none`, validates the known reason-code set and success/ID/reason
  invariants, and never serializes its free-form message. Start/recovery and
  mutation failure outcomes also use bounded public messages.
- Verification: two direct request/outcome tests passed under both modes,
  including credential/private-path sentinels and invalid domain-outcome
  fixtures. The production child-process adapter passed with actual loopback
  binding under both modes and covered empty selection, duplicate aliases,
  exact list params, missing status, and empty list outcomes. Full default
  tests passed 82 unit plus 11 integration; full no-default tests passed 45
  unit plus 8 integration (10 explicitly manual tests ignored in each mode).
  Format, default/no-default check, warnings-denied all-target Clippy, and
  focused diff checks passed.
- Corrected verification finding: the first Clippy run rejected a 256-byte
  status enum variant. The found payload is boxed; both Clippy commands then
  passed without an allow.
- Cross-review finding before acceptance: the core interrupted-scan owner still
  converted a blocking-task join failure to an empty list. The corrective
  slice below repaired that owner rather than deferring a false-success seam.
- Cross-review correction: this slice is reopened. `with_hf_client(false)`
  currently lets progress, mutation, and list operations look like legitimate
  missing/false/empty results, while the interrupted-directory scan maps a
  blocking-task join failure to an empty list. Those lower failures violate
  the typed producer outcome even though request exactness, redaction, and the
  valid result shapes passed their original checks.
- Corrective active write set: the prior M2H files plus
  `rust/crates/pumas-core/src/api/hf.rs`,
  `rust/crates/pumas-core/src/api/state_hf.rs`,
  `rust/crates/pumas-core/src/api/state.rs`,
  `rust/crates/pumas-core/tests/api_tests.rs`, and the one discovered direct
  adapter caller, `rust/crates/pumas-uniffi/src/bindings/api_hf.rs`. The adapter
  is admitted only for a compile-truthful `Result` projection pending accepted
  Milestone 6 deletion. No manifest, generated binding, host workflow, or new
  binding behavior is admitted.
- Corrective goal: make disabled/uninitialized HF an explicit error distinct
  from valid missing/false/empty, preserve interrupted-scan join failure, map
  internal detail to bounded RPC errors, and prove both at the public
  `PumasApi` and producer contract seams. A private join-result helper may
  supply deterministic panic/join evidence without a production test toggle.
- Discovered-caller disposition:
  `pumas-core/src/api/state_hf.rs` is the canonical internal owner and was
  migrated to `Result`; `pumas-core/src/api/hf.rs` now forwards its public API
  without defaults; `pumas-core/src/api/state.rs` propagates those errors in
  its retained internal dispatch; RPC download handlers propagate them to the
  redacted producer error; the UniFFI direct caller projects `Result` without
  fallback and remains scheduled for removal under `RUST-I3`. Repository-wide
  Rust call-site search found no other consumers of the changed methods.
- Red evidence: the exact public API test failed because progress/list still
  returned `Option`/`Vec` and could not express `Err`; the exact private join
  test failed because no join-preserving finisher existed.
- Green evidence: the public API disabled-HF scenario passed 1/1 and the
  deterministic blocking-task panic/join scenario passed 1/1. Desktop dispatch
  returned the stable redacted unavailable error for progress, all three
  mutations, and list in both feature modes (1/1 each). A real local IPC server
  rejected all six HF lifecycle method names as method-not-found with no result
  (1/1), preserving the deliberately smaller local IPC operation set.
- Valid-state evidence: the existing real desktop child process still passed
  missing-progress and empty-list cases with HF compiled and initialized; a
  direct outcome fixture now distinguishes missing mutation as
  `Download not found`. Interrupted-scan `JoinError` is internal only and maps
  to the bounded RPC internal error rather than an empty list.
- Full verification: `pumas-library` passed 860 unit tests, 35 API tests, all
  contract/persistence fixture binaries, and doctests; `pumas-rpc` passed 83
  unit plus 11 integration tests by default and 46 unit plus 8 integration
  tests without defaults (10 declared manual tests ignored per mode); the
  transitional UniFFI adapter passed 14 tests and doctests. Workspace all-
  target warnings-denied Clippy, no-default RPC all-target warnings-denied
  Clippy, workspace all-target check, format, and focused diff checks passed.
- Result: accepted after correction. The producer owns 48 typed commands with
  inference plugins and 47 without; 103/39 respective commands remain
  `Legacy`.

### 2026-09-03 — Reopen Milestone 2I model-catalog contract

- Active allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/models/catalog.rs`,
  `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`, this focused
  plan/ledger, and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, download/search/import/metadata/governance
  operations, plugin commands/loaders, local core IPC, CI, generated artifacts,
  package scripts, and shared project documentation.
- Goal: close `get_models` and `refresh_model_index` request/outcome contracts
  without arbitrary JSON or wrapper defaults.
- Red evidence: the direct producer contract test failed because neither exact
  command nor either typed outcome existed; both methods still entered
  `Legacy` and the wrapper could invent an empty model map.
- Implementation: both methods now admit exact empty parameter objects.
  `get_models` returns a sorted `BTreeMap<String, ModelRecord>` inside the
  producer outcome and rejects duplicate IDs instead of silently overwriting a
  core record. `refresh_model_index` projects its count as a fixed-width `u64`
  after checked conversion. Both legacy dispatcher arms and both wrapper
  entries/tests were removed.
- Verification: the direct contract test passed in default and no-default
  modes, including null/array/extra-field rejection, exact result shapes, and
  duplicate-ID failure. The production child-process HTTP adapter passed in
  both modes with invalid params plus valid empty catalog and real index-
  refresh results. Full default tests passed 83 unit plus 11 integration;
  no-default passed 46 unit plus 8 integration (10 declared manual tests
  ignored per mode). Both all-target warnings-denied Clippy commands, format,
  and focused diff checks passed.
- Cross-review result: acceptance withheld. The wire types and adapter evidence
  remain provisional, but three lower-owner defects prevent a truthful catalog
  handoff: a forced refresh can report success while an all-model reconcile is
  already in flight; failed reconciliation records success freshness and clears
  retry state; and `list_models` silently caps an all-model response at 10,000
  rows without exposing truncation.
- Corrective active write set: the prior M2I files plus the exact lower owners
  `rust/crates/pumas-core/src/api/models.rs`,
  `rust/crates/pumas-core/src/api/mod.rs` for the crate-private required-refresh
  seam export,
  `rust/crates/pumas-core/src/api/reconciliation.rs`,
  `rust/crates/pumas-core/src/api/state.rs` for the exact local-IPC rebuild
  caller,
  `rust/crates/pumas-core/src/error.rs`,
  `rust/crates/pumas-core/src/index/model_index.rs`, and
  `rust/crates/pumas-core/src/model_library/library.rs`. The exhaustive
  compile-only transitional caller `rust/crates/pumas-uniffi/src/bindings.rs`
  may only map the new core error pending Milestone 6 deletion; it must not add
  binding behavior. Any further caller requires another exact re-plan.
- Corrective oracles: deterministic real coordinator state must prove an
  in-flight forced refresh cannot become success and a dirty mark made during
  that flight remains retryable; failure must clear only in-flight ownership
  while preserving/creating dirty retry state; the public `PumasApi` and RPC
  error projection must expose a typed conflict. Full-scope and model-scope
  work must exclude one another in both directions, cancellation/drop must not
  strand ownership, and a stale run token must not finish a newer run. The
  `ModelLibrary` interface must return more than 10,000 distinct records
  without truncation, while corrupt row data must propagate as failure rather
  than being logged and omitted.
- Corrective producer projection: `ModelsOutcome` must no longer serialize the
  open core `ModelRecord` directly. The exact existing M2I RPC write set owns a
  smaller fallible `CatalogModel` projection: required ID, model directory,
  producer-selected display name, and nonblank model type; optional normalized
  format/quantization, JS-safe size, and display date; checked dependency count;
  a closed complete/partial artifact state with partial-only validated recovery
  identity; and a closed duplicate-integrity state. Raw metadata, hashes, tags,
  per-row update time, and unsupported `related_available` guesses are omitted.
  The partial wire member is the explicit camel-case
  `downloadProgressFraction`, validated as finite `0.0..=1.0` and below `1.0`
  for partial state, with no legacy duplicate. Malformed typed fields,
  inconsistent state, or malformed selected-artifact paths fail the whole
  response; key/record ID agreement and deterministic serialized bytes are
  direct contract oracles. Refresh count uses checked `u32`, not a JS-unsafe
  platform-width value. Receipt-local refresh success is documented separately
  from the independent update-event stream, which remains independently
  cursor-driven.
- Corrective implementation: `ReconciliationCoordinator` now admits explicit
  opportunistic/forced intent as `Started(run token)`, `Clean`, or `InFlight`.
  Admission clears the admitted scope's preexisting dirty bit(s), and an opaque
  allocation identity owns the active run. Concurrent dirty marks survive
  matching success; failure/drop restores dirty; identity comparison prevents
  a stale token from settling a replacement run. Full/model ownership excludes
  in both directions. The public and local-IPC rebuild callers use the forced
  seam and map overlap to `ModelIndexRefreshInProgress`
  (`-32011`/`conflict`).
  `ModelIndex::list_all` is an unbounded stable-order query that propagates row
  errors; `ModelLibrary` then applies the established dependency/display and
  artifact dedupe projections. Active dependency bindings always replace the
  metadata projection, including authoritative empty.
- Corrective DTO: `CatalogModel` emits `id`, `modelDir`, `displayName`,
  `modelType`, optional `format`, `quantization`, `sizeBytes`, and
  `displayDate`, checked `dependencyCount`, closed `artifact`, and closed
  `integrity`. Partial artifact state alone can carry the validated
  `downloadProgressFraction`; complete and partial states cannot coexist.
  Recovery repo IDs require exact nonempty `owner/name`; selected artifact
  files are bounded safe relative paths and deterministically sorted. All
  emitted strings/nested lists are bounded; size and refresh count are checked
  to JS-safe/u32 representations. No open metadata, hash map, tag list,
  guessed related flag, or per-record update time crosses this list seam.
- Corrective red evidence: coordinator/public refresh tests initially failed to
  compile because no distinct start result, failure terminal transition, or
  typed in-progress error existed. The greater-than-cap oracle returned 10,000
  instead of 10,001. A stale dependency fixture returned one persisted binding
  instead of authoritative zero. The DTO exact-shape fixture received the open
  core record, and the oversized-record fixture was accepted.
- Corrective green evidence: 20 focused reconciliation tests passed, including
  failure retry, dirty-during-run, full/model exclusion, drop, and stale-token
  cases; the barrier-held public `PumasApi` conflict test passed. The direct
  SQLite 10,001-row and corrupt-row tests passed, with the same corrupt row
  demonstrably dropped by search but rejected by complete listing. Stale
  dependency projection passed. Five catalog DTO tests plus checked refresh
  count passed. Core library unit tests passed 868/868; the 34 unrelated API
  integration tests pass when the deliberately queued M2J red HF-unavailable
  test is skipped. The unmodified full `api_tests` target is intentionally not
  usable until M2J resumes because that one red assertion poisons its test
  guard and causes cascades; it was not weakened or ignored. RPC passed 88 unit
  plus 11 integration tests by default and 51 unit plus 8 integration tests
  without defaults (10 declared manual tests ignored in each mode). UniFFI
  passed 14 tests/doctests. Workspace all-target check, workspace all-target
  warnings-denied Clippy, no-default RPC all-target warnings-denied Clippy,
  format, and focused diff checks passed.
- Frozen corrective write set reconciled to the actual M2I edits: this focused
  `plan.md`, `execution-ledger.md`, `issues.md`, and
  `reports/rpc-contract-and-threat-model.md`; core `api/mod.rs`,
  `api/models.rs`, `api/reconciliation.rs`, `api/state.rs`, `error.rs`,
  `index/model_index.rs`, and `model_library/library.rs`; RPC `contract.rs`,
  `handlers/mod.rs`, `handlers/models/catalog.rs`, `wrapper.rs`, and
  `tests/integration_tests.rs`; plus compile-only transitional
  `pumas-uniffi/src/bindings.rs`. The unstaged queued M2J `api_tests.rs` red
  oracle and staged earlier `pumas-uniffi/src/bindings/api_hf.rs` change are not part of
  this M2I boundary. No manifest, lockfile, CI, generated artifact, package
  script, or shared project documentation changed in this corrective slice.
- Corrective result: frozen for independent cross-review, not yet accepted.
  The provisional typed counts remain 50/49 with 101/37 `Legacy`.
- First frozen-boundary review: not accepted because the RPC projection treated
  subordinate part/missing evidence as contradictory whenever
  `download_incomplete` was false. Core deliberately produces that shape when
  a selected Q4 artifact is partial but another complete Q5 GGUF remains
  displayable, so one valid mixed-quant row would fail the entire catalog.
- Mixed-quant red evidence: a core-record-to-`CatalogModel` regression using
  the exact Q5-complete/Q4-part metadata failed with `complete artifact
  contains partial evidence`.
- Mixed-quant correction: core `download_incomplete` is the authoritative
  readiness classification. False now projects `artifact.state=complete`
  even when subordinate selected-artifact part/missing evidence exists; it is
  never coerced to partial and no renderer-unneeded mixed state was added. The
  exact regression passed in both RPC feature modes; the owning real core
  fixture also passed. Full RPC passed 89 unit plus 11 integration tests by
  default and 52 unit plus 8 integration tests without defaults. The slice was
  re-frozen for independent review.
- Second frozen-boundary review: not accepted because full/model dirty state
  was not hierarchical in both directions, numeric revision/run counters used
  saturating identity, and recovery validation depended on the build host's
  path parser plus a two-segment-only repository check.
- Coordinator red evidence: after a clean full run, a model dirtied during a
  later full run was stranded when that run succeeded; after a clean targeted
  run, `mark_dirty_all` left the model scope `Clean`; and seeding the old run-ID
  counter at `u64::MAX - 1` let a stale token settle its replacement.
- Coordinator correction: dirtiness is an explicit per-scope bit. Admission
  clears the admitted preexisting bit(s); concurrent marks remain set; full
  dirty/failure/drop propagates to every known model scope; unseen scopes still
  reconcile on first access. Targeted success clears only its own admitted
  dirty state, so it does not consume the pending full retry. Active ownership
  uses allocation identity and `Arc::ptr_eq`, eliminating bounded numeric token
  identity and stale-token collision. Both hierarchy races and stale identity
  now pass deterministically without sleeps.
- Recovery red evidence: the projection accepted official-invalid repository
  IDs such as `owner/..`, `owner/name--variant`, and terminal `.git`, plus
  Windows drive, UNC/backslash traversal, reserved-device, invalid-character,
  and overlong-component selected-file paths when tested on Linux.
- Recovery correction: private serializable smart constructors apply the
  official Hugging Face 1–96-byte repository grammar, further narrowed to
  exact `owner/name`, and a host-independent repository-relative path grammar.
  Paths use `/`, reject traversal/root/drive/UNC and Windows-reserved or invalid
  components, bound the whole string and each component, and preserve exact
  serialized strings. The negative matrix and nested positive path pass in
  both feature modes. The slice is re-frozen for independent review.
- Final re-freeze verification: 22 focused reconciliation tests and 870/870
  core library unit tests passed; the 34 unrelated API integration tests pass
  with only the queued M2J red skipped. RPC passed 90 unit plus 11 integration
  tests by default and 53 unit plus 8 integration tests without defaults (10
  declared manual tests ignored in each mode). The first final no-default run
  hit the known global test-registry interference as SQLite `database is
  locked`/`disk I/O error` in two unrelated startup fixtures; both exact tests
  passed in isolation and the immediate isolated full rerun passed. Targeted
  all-feature and no-default warnings-denied Clippy, full format, and focused
  diff checks passed.
- Third frozen-boundary review: not accepted because complete catalog rows
  could still carry recovery identity. In the supported mixed-quant case, that
  would present a ready Q5 model as recoverable merely because subordinate Q4
  selection metadata remained present. A separate action-contract review also
  found that the accepted recovery methods combine producer repo identity with
  a caller-supplied destination that the core accepts when it is any existing
  directory; that second finding remains read-only pending an exact re-plan.
- Complete-state red evidence: the realistic Q5-complete/Q4-part projection,
  extended with `owner/model`, selected Q4 artifact identity, filename, and
  quant, serialized a `recovery` member on the complete artifact.
- Complete-state correction: `CatalogArtifactState::Complete` is a unit closed
  variant. Recovery parsing and validation occur only in the partial branch;
  complete models serialize exactly `{state: complete}` even if subordinate or
  stale selected-artifact recovery fields remain in core metadata. The exact
  regression is green and no typed/legacy command count changes.
- Action-contract investigation, no source mutation: model IDs are library-root
  relative; indexed records carry the producer path; `PumasApi::get_model`
  refreshes and resolves the indexed record; and legitimate in-place imports
  are documented as already inside the library tree. Existing recovery then
  reads exact selected/expected filenames from metadata, but it currently
  trusts a caller-selected existing directory and caller repo ID. The smallest
  corrective Interface is a model-ID-only action whose core owner resolves the
  indexed library directory and metadata server-side, rejects complete or
  unproven download provenance, and delegates to the existing tracked/resume
  mechanics with producer-derived repo, destination, and exact file set.
- Proposed action-contract write set, not yet admitted: RPC `contract.rs`,
  `handlers/mod.rs`, `handlers/models/downloads.rs`, and
  `tests/integration_tests.rs`; core `api/hf.rs`, `api/state_hf.rs`, and
  `api/state.rs`; transitional compile caller
  `pumas-uniffi/src/bindings/api_hf.rs`; plus this focused plan/ledger/issues
  and report. `pumas-core/tests/api_tests.rs` is required only if the public
  PumasApi seam cannot be proven in owning module tests, and currently remains
  excluded because it contains the queued M2J red. Electron/preload/frontend
  consumer projection is platform/frontend-owned and is not in this Rust write
  set. No manifest, lockfile, CI, generated, or shared-document edit is
  proposed.
- PRG-I19 admission: root admitted the exact proposal with canonical sorted-
  unique selected/expected file-set hashing. The token is only a collision-
  resistant stale-state fingerprint/precondition, never caller
  authentication. Partial rows that lack proven download provenance or a
  canonical managed-root model path remain displayable without a token; the
  path-free action rejects them. Model ID and the fixed token grammar are
  validated before lookup. Outside-root imports are not recovery-eligible.
- Active corrective write set: this focused plan/ledger/issues/report; new core
  `model_library/download_recovery.rs`; core `model_library/mod.rs`,
  `model_library/library.rs`, `model_library/hf/download.rs`, and `api/hf.rs`;
  RPC `contract.rs`, `handlers/mod.rs`, `handlers/models/downloads.rs`, and
  `tests/integration_tests.rs`. No core generic local-IPC, UniFFI, manifest,
  lockfile, CI, generated, shared-doc, platform, or frontend edit is admitted.
- Required TDD boundary: shared token issuance/verification owner; stable token
  under file reorder but stale on set membership/repo/artifact/quant/path/state
  changes; canonical-root/symlink/alias/provenance refusal; exact action params
  and fixed token grammar; tracked repository/file-context mismatch refusal;
  positive tracked resume/attach and indexed untracked recovery; deleted pair
  method-not-found through the real loopback process.
- PRG-I19 red evidence: the request fixture failed to compile after expecting
  the old repo/path fields from `ResumePartialDownload`, and the core recovery
  fixture failed before the shared issuer existed. The complete mixed-quant
  fixture also established that recovery metadata must never be parsed for a
  complete row. Exact invalid snake-case action params, malformed token/model
  IDs, stale tokens, unsafe repo/path metadata, and outside-root eligibility
  were then captured at the producer/core seams before the final green gates.
- PRG-I19 implementation: new `model_library/download_recovery.rs` is the one
  semantic owner for issuance and verification. Its `v1:` plus 64-lowercase-
  hex BLAKE3 value hashes explicit domain/version framing, model ID, canonical
  managed path, repo, artifact, quant, and a sorted-unique file set with
  length-delimited members. It is a collision-resistant stale-state
  fingerprint, not authentication. Model IDs, repo IDs, paths, per-component
  lengths, token grammar, text, and collections are bounded before use.
  Complete state returns before repo/artifact/file parsing. Partial rows that
  are outside the canonical root, aliased, symlinked, missing/uninspectable,
  non-UTF-8 after canonicalization, or without provenance stay displayable but
  receive no ticket.
- Action correction: `resume_partial_download` now accepts only exact
  camel-case `{modelId,recoveryToken}` at desktop admission. Core lookup derives
  the indexed path, reindexes the directory, rebuilds the same snapshot, and
  rejects stale state before any action. Tracked work must match destination,
  repo, and the canonical artifact file set; paused/error work resumes, active
  work attaches, mismatched context is stale, and an untracked eligible row
  starts from producer-derived repo/path/files. `list_interrupted_downloads`
  and `recover_download` are method-not-found at desktop admission. Retained
  low-level core/local branches were not widened or presented as a supported
  desktop Interface. In particular, public core/local-IPC and UniFFI
  `recover_download(repo_id, dest_dir)` remain transitional ambient-authority
  reachability pending the accepted zero-`Legacy` local-IPC and Milestone 6
  binding removals; this slice does not claim those surfaces are secured.
- Availability/error contract: progress/mutation/list retain the accepted
  public `Result::Err` behavior when HF is absent. The partial-recovery action
  is an explicit closed-outcome exception: absent HF is `success:false` with
  `hf_client_unavailable`, not false success/default data. A post-verification
  missing-path error maps to accepted `recovery_unavailable`; internal action
  messages and all locators are discarded by the RPC projection.
- Green evidence: core recovery and tracked-context tests passed 5/5; the four
  recovery error-classification tests passed; catalog tests passed 8/8 in both
  feature modes. Real loopback tests passed for stale refusal followed by exact
  tracked resume, deterministic active attach, cache-backed indexed untracked
  recovery, and the admission matrix covering removed methods plus hostile old
  repo/path and malformed token inputs. The first full default integration run
  exposed a nondeterministic test-only second-call attach assertion after the
  spawned task could already fail; the oracle was split into a persisted
  `pausing` state for deterministic attach, without changing production policy.
- Full verification: `pumas-library --lib` passed 876/876. `pumas-rpc` passed
  89 unit plus 14 integration tests by default and 52 unit plus 11 integration
  tests without defaults; ten declared manual tests remain ignored in each
  mode. Default all-target warnings-denied Clippy for `pumas-library` and
  `pumas-rpc`, and no-default all-target warnings-denied `pumas-rpc` Clippy,
  passed. The first final no-default rerun repeated the known concurrent global
  registry failure (`database is locked`/`disk I/O error`) in the disabled-HF
  and server-start tests; both exact roots passed, then the unchanged full
  no-default suite passed. Focused rustfmt, repository diff checks, and the
  focused plan-structure checker passed.
- Frozen PRG-I19 source write set: new core
  `model_library/download_recovery.rs`; core `model_library/mod.rs`,
  `model_library/hf/download.rs`, and `api/hf.rs`; RPC `contract.rs`,
  `handlers/mod.rs`, `handlers/models/downloads.rs`, and
  `tests/integration_tests.rs`; plus this focused plan/ledger/issues/report.
  The already-admitted M2I `model_library/library.rs` and catalog handler remain
  part of the cumulative boundary but received no PRG-I19-specific behavior.
  `model_library/hf/mod.rs` has no retained diff. No generic local IPC, UniFFI,
  manifest, lockfile, CI, generated, shared-doc, platform, frontend, index, or
  staging mutation belongs to this correction.
- Corrective result: re-frozen for independent review. `RUST-I7` is resolved;
  provisional command counts are now 48/47 typed and 101/37 `Legacy`, for
  149/84 reachable desktop operations. M2/RUST-A2 remain active until all
  remaining `Legacy` groups are converted; M2J remains paused pending this
  boundary's acceptance.

### 2026-09-03 — Reopen PRG-I19 for exact recovery authority through use

- Independent program and Standards review did not accept the preceding
  PRG-I19 boundary. The generic download start path could accept only the
  remote intersection of the ticket's files, add unbound auxiliary files, and
  race destination-only deduplication against an unrelated context. The token
  check also returned a plain absolute path before later pathname-based file
  mutation, so a replaced nested directory or `.part` symlink could escape the
  checked target. `PartialDownloadOutcome` still admitted invalid action,
  status, and download-ID combinations. These are systemic defects in the one
  recovery admission/lifecycle owner rather than isolated handler examples.
- Root and program admitted one replacement composition. The exact added write
  set is `rust/Cargo.toml`, `rust/crates/pumas-core/Cargo.toml`,
  `rust/Cargo.lock`, and
  `rust/crates/pumas-core/src/model_library/hf/types.rs`. The existing PRG-I19
  write set remains this focused plan/ledger/issues/report; core
  `model_library/download_recovery.rs`, `model_library/mod.rs`,
  `model_library/hf/download.rs`, and `api/hf.rs`; and RPC `contract.rs`,
  `handlers/mod.rs`, `handlers/models/downloads.rs`, and
  `tests/integration_tests.rs`. `model_library/library.rs` remains part of the
  cumulative M2I boundary but has no recovery-specific change. `hf/mod.rs`,
  generic local IPC, UniFFI, generated/platform/frontend files, CI, package
  scripts, and shared project documentation remain excluded.
- Red evidence first established each replacement invariant: exact selection
  had no owning resolver; a remote tree missing one bound member still reached
  generic start; concurrent unrelated contexts could share destination-only
  admission; invalid action/status/ID tuples projected as valid; existing
  nested and `.part` symlinks could receive authority; and replacing either
  after verification left later mutation on ambient pathnames. State-lifecycle
  reds then covered capability-free task registration, pause overwriting a
  terminal state, resume versus cancel, relocation mutation, restored
  authority, callback/persistence leakage, and retained descriptors after
  client drop.
- The replacement uses one private `DownloadRecoveryDestination` backed by a
  held `cap_std::fs::Dir` for the canonical managed library root. Exact
  recovery preflight, parent creation, metadata, part open/length/removal,
  marker removal, and final rename are all root-handle-relative. Existing
  symlink components are refused when observed, and capability-relative
  resolution contains a concurrent replacement within the held root instead
  of reacquiring ambient authority from the serialized/display path. This
  proves no mutation outside the held managed root in the tested Linux races;
  it does not pin the original model-directory inode or prevent a same-user
  replacement that resolves to a different directory inside that same root.
  No Windows or macOS runtime result is claimed.
- One exact recovery resolver requires every canonical ticket-bound remote
  file and schedules that sorted unique set only; it never adds repository
  auxiliaries. One downloads write-lock admission atomically decides exact
  tracked resume/attach versus new recovery insertion from destination,
  repository, and file context, then registers the task with a start gate.
  A missing member refuses before task or target mutation, and a concurrent
  unrelated context cannot attach, filter, or be returned.
- The held capability is a private, non-serialized member of `DownloadState`,
  so state and authority transition under the existing downloads lock. New
  recovery and tracked recovery resume install it atomically. Error, pause,
  and reconciled missing-task states retain it; completion and observed final
  cancel clear it. Recovery tasks receive neither ordinary status persistence
  nor application callbacks; Slice B later admits only a narrow terminal
  cleanup authority,
  restored/fresh clients never reconstruct authority, and relocation refuses
  before changing destination, request, or persistence. Pause now performs one
  conditional write-lock transition and cannot overwrite completion. Cancel
  removes task registration, aborts and awaits the outer task, then clears the
  state-held capability; an already-running blocking operation may retain a
  confined clone until that operation returns, so no synchronous OS-descriptor
  release is claimed.
- RPC projection now rejects blank/oversized download IDs and every impossible
  action/status/reason/ID tuple. Exact camel-case action params, stable public
  reason codes, path-free results, and no-secret/internal-message behavior are
  unchanged. Provisional command counts remain 48/47 typed and 101/37
  `Legacy`; this correction changes semantics inside an already counted typed
  command.
- A later independent numeric review reopened this candidate once more: the
  new exact-set admission used iterator `sum::<u64>()` over up to 512 remote
  LFS sizes. The focused `u64::MAX + 1` test first failed with debug panic
  `attempt to add with overflow`. Admission now uses checked accumulation and
  returns the stable `BoundFilesUnavailable` refusal before downloads-state
  insertion, task registration, marker creation, or part-file write. The exact
  red became green and the recovery-named group advanced from 17 to 18 tests.
- Dependency decision: workspace and core declare caret-compatible
  `cap-std = { version = "4.0.3", default-features = false }`. The lock change
  resolves `cap-std` exactly to `4.0.3` and is limited to `cap-std`,
  `cap-primitives`, `ambient-authority`,
  `fs-set-times`, `io-extras`, `io-lifetimes` 2 and 3, `maybe-owned`,
  `rustix-linux-procfs`, and target-only `winx`, plus the core package edge.
  Reverse and no-default feature trees show `pumas-library` as the direct
  consumer and no `cap-std` feature enabled. Local package manifests report
  Apache-2.0/MIT-compatible licensing for the closure; the initially absent
  target-only `winx 0.36.4` source was fetched with the admitted registry
  authority and reports `Apache-2.0 WITH LLVM-exception`. `cargo-audit` and
  `cargo-deny` are not installed; neither was installed as an unapproved
  fallback. The lock records exact checksums, and the admitted Pumas source
  adds no `unsafe` block.
- Green focused evidence: all 18 recovery-named core tests passed, including
  exact-set/no-aux, missing-member no-task/no-write, unrelated-context atomic
  admission, oversized remote-total no-task/no-write, capability-bearing
  registration, tracked resume, completion, error/pause retention,
  resume/cancel overlap, relocation refusal, restore, client drop, and
  symlink/replacement containment. The dedicated
  pause-versus-completion, ordinary ambient resume compatibility, and existing
  tracked-cancel tests each passed. The four real-loopback recovery tests pass,
  including a cache-backed exact untracked transfer with no auxiliary file or
  `downloads.json` persistence.
- Full source evidence: `cargo test -p pumas-library --lib` and its
  `--no-default-features` counterpart each passed 892/892. `cargo test -p
  pumas-rpc` passed 89 unit and 15 integration tests; the no-default run passed
  52 unit and 12 integration tests, with ten declared manual tests ignored in
  each mode. Default warnings-denied all-target Clippy passed for
  `pumas-library` and `pumas-rpc`; no-default warnings-denied all-target Clippy
  passed for `pumas-rpc`; workspace rustfmt passed. M2J remains held, and its
  queued red in `pumas-core/tests/api_tests.rs` is excluded from this evidence.
- Standards/diff evidence: Coding Standards current HEAD
  `17f418fbef05493c7aa02927834fe425846a0388` descends from the recorded
  `f3d2b8a3` lineage; its intervening changes migrate verifier/evidence files
  without changing a canonical normative or external plan-checker file. One
  live `check-plan-structure.sh` invocation over all five Pumas plans passed
  with no diagnostics. Root's hash-locked current-verifier environment passed
  the `planning-admission` and `execution-train` suites, four checks each.
  Workspace rustfmt check and repository `git diff --check` pass. Exact status
  confirms that the previously staged compile-only
  `pumas-uniffi/src/bindings/api_hf.rs` is unrelated and excluded; this slice
  stages and commits nothing.
- Current result: the checked-overflow correction and requested gates are
  complete, but the PRG-I19 boundary is reopened while independent governance
  review completes its announced cancellation/persistence findings. The exact
  correction write set remains the one recorded above; `hf/mod.rs` has no
  retained correction diff, and no discovered caller or shared file lies
  outside the admitted set. M2J remains held.

### 2026-09-03 — Re-plan PRG-I19 task and persistence authority

- Independent governance review rejected the candidate on four owning
  invariants. Recovery state/capability was installed before awaited task
  preparation, so caller cancellation could strand an unowned queued state;
  ambient persistence removal was best-effort and raced stale load/write;
  recovery validation canonicalized a root before separately opening its
  capability; and errors from the new admission path escaped the action's
  closed reason algebra. The earlier numeric-overflow correction remains
  admitted and green but is not an acceptance boundary.
- Root admitted one bounded re-plan. The existing PRG-I19 write set remains in
  force and adds exactly
  `rust/crates/pumas-core/src/model_library/download_store.rs` as the serialized
  persistence mutation/revocation owner. `hf/mod.rs`, generic local IPC,
  UniFFI, Electron/frontend consumers, generated files, CI, package scripts,
  and shared documentation remain excluded. M2J remains held.
- Required replacement evidence is deterministic: cancellation before and
  after the synchronous state/task commit; same-context attach and pause/cancel
  races; attach only to a registered capability-backed task; strict persistence
  revocation failure without state/task/target mutation; stale writer and
  restart generic-resume denial; one held-root identity across validation and
  use with a root-replacement sentinel; and real PumasApi plus loopback mapping
  of injected lookup/reindex/verification/network/capability/admission failures
  to closed action reasons.
- Dependency wording is narrowed: the manifest requirement is caret-compatible
  `4.0.3`; `Cargo.lock` resolves exactly `4.0.3`. No manifest change is admitted
  for this clarification.

### 2026-09-03 — Re-freeze PRG-I19 cancellation-safe recovery owner

- The final bounded implementation prepares every awaited recovery dependency
  before admission. Initial recovery and capability-backed resume then create
  a start-gated worker and synchronously commit state, held capability,
  non-finished `JoinHandle`, and `task_registered=true` while holding the
  downloads and task locks. Opening the gate happens before any further await.
  Cancellation before that commit leaves no new owner; cancellation after it
  leaves the registered worker. Exact-context attach now requires that real
  registered capability-backed owner and cannot attach to an ambient or
  unregistered state.
- Cancellation is also caller-independent after its state transition. Under
  the same state/task owner it sets `Cancelling`, aborts the owned worker, and
  installs a finalizer as the registered task. The finalizer observes worker
  termination, publishes `Cancelled`, and clears the state-held capability;
  dropping the cancel future before or during that work cannot detach it, and
  repeated cancel does not remove the finalizer. This is not a synchronous
  OS-descriptor-release claim: a blocking filesystem operation may retain a
  confined clone until it returns, and full client-drop drain remains owned by
  Milestone 4/RUST-A6.
- `DownloadPersistence` is the shared serialized mutation owner. Strict
  revocation removes the ambient persisted record before recovery admission
  and records an owner-local tombstone honored by save, status, relocation,
  load, and ambient resume. Corrupt-store/revocation failure leaves state,
  task, and target unchanged; a stale writer cannot recreate the row, and a
  newly constructed persistence owner finds no row to restore. Recovery work
  receives neither ordinary status persistence nor callbacks; Slice B later
  admits only terminal cleanup persistence. Ordinary ambient resume retains its
  existing callback/persistence behavior.
- Recovery opens the managed library root once, records the held descriptor's
  filesystem identity, and derives the model destination through that same
  `cap_std::fs::Dir`. The displayed root is rechecked against that identity,
  while all recovery file operations are handle-relative. A deterministic
  same-name root-replacement oracle now replaces the root after the handle is
  open but before model validation; admission refuses and neither original nor
  replacement sentinel is mutated. Existing nested-parent and `.part` symlink
  oracles remain green. This is Linux runtime evidence only and does not claim
  to pin the model-directory inode against every same-user replacement within
  the held root; Windows/macOS runtime behavior remains unverified.
- The public action maps lookup, reindex, verification, repository/network,
  capability, and admission failures into its closed reason algebra. The real
  loopback repository-lookup failure returns `success:false` with
  `recover_failed`, no locator/internal message, and no JSON-RPC internal
  error. The real loopback active-context test now performs two concurrent
  ticket-only requests and observes exactly one `resume` plus one `attach`
  with the same registered download ID; it no longer treats a persisted
  `pausing` row without a task as attachable.
- Focused green commands at re-freeze: `cargo test -p pumas-library --lib
  recovery_ -- --nocapture` passed 26/26; `... --lib revok ...` passed 3/3;
  `... --lib held_root ...` passed 1/1; the dedicated ordinary-resume,
  pause-versus-completion, and existing tracked-cancel commands each passed
  1/1. `cargo test -p pumas-rpc --test integration_tests
  desktop_rpc_partial_recovery -- --nocapture` passed 5/5, including the real
  loopback closed-error and registered-attach cases. The RPC typed-outcome
  unit test passed 1/1 in default and no-default modes; the no-default disabled
  HF and exact download-admission tests each passed 1/1. Full crate/aggregate,
  warnings-denied Clippy, dependency, Standards, and diff gates remain to run
  after independent source review; no success is claimed for them here.
- Exact re-frozen PRG-I19 source set: `rust/Cargo.toml`,
  `rust/crates/pumas-core/Cargo.toml`, `rust/Cargo.lock`; core
  `src/api/hf.rs`, `src/model_library/download_recovery.rs`,
  `src/model_library/download_store.rs`, `src/model_library/mod.rs`,
  `src/model_library/hf/download.rs`, and
  `src/model_library/hf/types.rs`; RPC `src/contract.rs`,
  `src/handlers/mod.rs`, `src/handlers/models/downloads.rs`, and
  `tests/integration_tests.rs`; plus this focused plan, ledger, issues, and RPC
  report. Cumulative M2I files outside that list retain their earlier admitted
  changes but received no new PRG-I19 behavior. `hf/mod.rs`, generic local IPC,
  UniFFI, Electron/frontend, generated, CI, package-script, and shared-doc
  files remain excluded.
- Current result: re-frozen for independent review, not accepted. The desktop
  producer cannot ship as a standalone commit because the reachable
  Electron/frontend action still sends the old repository/path shape; its
  decoder/preload/renderer migration must land atomically. Public core/local-
  IPC and UniFFI ambient recovery routes also remain transitional and outside
  this desktop Interface correction. No source or docs are staged or committed
  by this slice; the previously staged compile-only
  `pumas-uniffi/src/bindings/api_hf.rs` remains unrelated. M2J remains held.

### 2026-09-03 — PRG-I19 Slice A durable revocation publication

- The preceding whole-boundary re-freeze and first Slice A implementation were
  rejected. Besides the deferred task-ownership defects, the first Slice A
  could synthesize durable absence, kept uncertain disposition only in memory,
  coordinated only one constructor, reopened the parent after rename, and did
  not preserve rename visibility or staging-cleanup ambiguity. Root/program
  re-admitted a revised persistence-only Slice A.
- Exact Slice A source write set:
  `rust/crates/pumas-core/src/metadata/atomic.rs`, its exact crate-private
  re-export in `src/metadata/mod.rs`, and
  `src/model_library/download_store.rs`; plus this focused plan, ledger,
  issues, and RPC report. No HF lifecycle/state/task caller, RPC, manifest,
  consumer, generated, CI, or package-script file is admitted or changed by
  Slice A.
- `AtomicJsonTarget` requires a pre-existing parent, holds a capability-relative
  directory and sync-capable file opened from the same parent, creates a unique
  staging file with `create_new`, renames relative to that authority, syncs the
  exact held parent, and verifies that the configured parent still has the held
  identity. Direct reads treat only `NotFound` as absence. The closed result
  algebra distinguishes boxed pre-rename `AtomicPublishFailure` plus observed
  `StagingCleanup`, `VisibilityUnknown`, `PublishedDurabilityUnknown`, and
  `Durable`; rename errors remain visibility-unknown even if the Adapter
  injected failure before or after effect. Existing `atomic_write_json` callers
  retain their legacy Interface and gain no durability claim.
- `downloads.json` now has strict versioned schema 2 with persisted per-download
  revocation attempt/disposition. The unused store-generation field was removed
  rather than preserving a speculative wire surface. Legacy `{downloads}` is
  validated and migrates on the next mutation; malformed JSON is a typed JSON
  failure and unsupported/conflicting schema is a typed validation failure.
  Every versioned whole-document mutation uses the durable publisher. Revocation is mandatory
  two phase even when the row is absent: a durably published
  `durability_unknown` intent removes the row and fails closed, then a second
  durable publication confirms the same attempt. A fresh owner never promotes
  bare absence or an uncertain attempt to durable; only the persisted durable
  disposition may be reused.
- Every store read-modify-write acquires the in-instance mutex, opens the held
  target, takes `.downloads.lock` with the Rust 1.92 OS file-lock Interface,
  strictly rereads current bytes, then mutates and durably publishes while the
  locks remain owned. The test observer distinguishes the immediate pre-lock
  attempt from post-acquisition, so queued-order negatives no longer infer
  contention from a timeout alone. This coordinates independent constructors and processes
  for store RMW/crash-reopen only. Task admission retains the explicit
  single-active-Pumas-runtime-per-root precondition and is deferred to later
  slices.
- Red-to-green evidence included direct-open I/O misclassified as absence,
  missing-parent creation, Linux `EBADF` from an unsuitable directory handle,
  rename-before/after-effect ambiguity, cleanup failure, foreign staging-name
  collision in both the new publisher and legacy `atomic_write_json`, combined
  parent replacement plus sync failure, bare-absence and persisted-unknown promotion,
  strict schema failures, stale independent writers, and child-process
  interruption/lock-release phases. The first compile also exposed the new
  result algebra at its callers; the final five Clippy diagnostics were closed
  by boxing the large failure payload and removing a useless conversion without
  weakening the variants. A later classification red proved that the store
  discarded pre-publication stage/kind; `RecoveryRevocation::NotPublished` now
  retains that closed classification.
- Focused green evidence in both default and no-default modes: atomic module 19
  enumerated, 18 passed and one subprocess helper ignored; download-store module
  24 enumerated, 22 passed and two subprocess helpers ignored. These cover real
  Linux file/parent sync, typed pre-/post-publication uncertainty, cleanup,
  strict v1-to-v2 migration, two-phase absent/unknown/durable fresh-owner state,
  all three phase-two confirmation ambiguities, actual post-lock writer ordering,
  real child kill/lock release, and child exit after each revocation phase.
  Phase-two ambiguity never succeeds the initiating call. Its same-lock attempt
  already durably published the phase-one unknown tombstone; a visible durable
  successor may be reused, while a pre-effect outcome retries both publications.
  Default all-feature and no-default all-target
  `pumas-library` Clippy pass with `-D warnings`; workspace rustfmt check, exact
  diff check, and the five-plan structure gate pass at current Coding Standards
  revision `886c52d2b9502bc45a40049478ced0cc27eac240`. The delta from the prior
  `17f418fb` checkpoint changes verification-engine/planning records but no
  canonical normative standard or external plan-checker file, so it does not
  widen this slice's contract.
- Result: revised Slice A is re-frozen for independent review, not accepted and
  not wired into the HF caller. No full aggregate or downstream acceptance is
  claimed. `Durable` means file sync, held-parent sync syscall completion, and
  configured-parent identity match on the exercised Linux local ext-family
  filesystem; it is not a universal hardware/power-loss claim. macOS remains
  unverified and the durable publisher returns a closed target-admission/
  unavailable pre-rename failure on Windows/non-Unix rather than false
  `Durable`. Network/distributed filesystems remain unsupported and must be
  rejected or separately proved before a later slice wires task admission.
  Slices B-E and M2J remain held.
  This slice stages and commits nothing.

### 2026-09-03 — PRG-I19 Slice B owned download-task lifecycle

- Exact Slice B source write set: new
  `rust/crates/pumas-core/src/model_library/hf/lifecycle.rs` and existing
  `src/model_library/hf/mod.rs`, `src/model_library/hf/download.rs`, and
  `src/model_library/hf/types.rs`; plus this focused plan, ledger, issues, and
  RPC report. `hf/types.rs` now contains the non-serialized
  `lifecycle_failure_unverified` provenance required by Slice B. Slice A
  publication/store files, RPC, local IPC, manifests, consumers, generated
  files, CI, package scripts, and M2J remain excluded.
- Red evidence started at the new owner Interface: the exact gated-task test
  failed to compile while `PreparedTask` did not yet satisfy its completed
  result contract and production still stored raw outer handles. The first
  integrated recovery group then passed six of nine tests: two caller-dropped
  transitions remained finished but unobserved, and cancellation of a
  state-only active entry had no finalizer owner. The first broader HF run
  passed 75 of 76 tests and exposed the remaining incorrect expectation that a
  completed finalizer removed itself before terminal observation.
- Independent review reopened that first boundary. Deterministic correction
  reds reproduced five ownership failures: persisted recovery revocation used
  a synthetic transition ID and did not reserve the actual download; recovery
  data write/flush escaped through Tokio-internal blocking work; cancellation
  could detach nested custody from a dropped drain waiter; worker, nested, or
  cleanup failure still published `Cancelled`; and reconciliation could mark a
  newly installed owner inactive after an earlier task-ID snapshot. A final
  self-review red proved an outer task was observable while one of its
  registered blocking operations was still running.
- `DownloadTaskOwner` now owns an opaque generation, role, gated outer Tokio
  handle, bounded nested blocking-observer registry, sticky projection outcome,
  and retained rejected-task retirement per download. Start tokens only perform
  a bounded atomic `Gated -> Abandoned` transition on drop. Explicit post-lock
  rescue aborts and owner-retains the wrapper until its outer and nested handles
  have been joined and their terminal outcomes consumed. Worker and
  `RecoveryTransition` rejection/collision matrices prove that no work body runs
  and no retired observer remains. All start/abort signals occur after task and
  download-state guards are released.
- `TaskContext` registers mutating recovery and Ambient filesystem work,
  persistence work, and admitted external callbacks before awaiting it. Nested
  completion reaping requires actual observer-handle readiness, consumes Join
  failure, and archives semantic `Result::Err` even if the request-side receiver
  is cancelled. The retained registry remains constant-bounded across repeated
  operations. A held Ambient write cannot outlive cancellation cleanup and
  recreate a part file after terminal state.
- Existing-context recovery now installs `RecoveryTransition` under the actual
  download ID before strict revocation. Generic resume and relocation refuse
  that reservation; successful revocation promotes the same opaque generation
  to `Worker`, while stale transitions cannot remove a successor. Recovery
  `std::fs::File` write and flush operations run only through registered
  `TaskContext` blocking handles. Nested custody remains stored in the owner
  while any waiter drains it, and a task is observable as finished only after
  its outer and all nested observer handles finish.
- Finished-task observation atomically replaces the finished generation with an
  actual-ID `TerminalProjection`. Its non-owning ticket is generation-bound;
  duplicate observers share one owner, request cancellation cannot discard the
  outcome, and settlement distinguishes pending, already settled, stale, and
  missing. A terminal projector whose primary and failure fallback both panic
  reports `FailureUnprojected` instead of spinning as pending; the actual-ID
  owner and sticky provenance remain available to public cancel. Sticky
  inherited and projector failures are visible to a superseding cancel before
  any await. Their cell custody is forwarded across replacement and is
  acknowledged/settled only after the finalizer publishes its fail-closed Error
  snapshot. Active finished Workers and finished finalizers still in
  `Cancelling` are classified as unverified terminal obligations even after a
  clean Join.
- Cancellation uses the truthful predecessor algebra `Absent | Observed` and
  atomically captures whether the predecessor outer task had already finished
  at the exact replacement. A deterministic unfinished-at-precheck then
  finished-before-replacement race and an outer-finished/nested-held race both
  retain the unverified Worker obligation. The finalizer removes every bound
  part and marker for Ambient and Recovery destinations, performs the strict
  persistence cleanup, drains all owned work, and only then publishes terminal
  state. Any predecessor, filesystem, persistence, nested, or finalizer failure
  yields sticky `Error`; recovery capability is retained and its durable
  revocation tombstone is preserved.
- Reconciliation reserves its own terminal projector, persists Ambient
  `Paused` first, requires an exact updated-row result, then generation/status
  rechecks before memory publication. Resume/relocate cannot cross a durable
  revoke-and-transition-disappearance window because the state-local ambient
  authority block remains set. Concurrent observers and post-memory/pre-persist
  successor installation cannot apply a stale generation to a successor.
- Strict successful-completion cleanup is an admitted narrow Slice B exception
  for both destination kinds: persistence cleanup and final drain precede
  `Completed`, capability release, and the success callback. Ambient cleanup
  failure produces `Error` and no callback. Auxiliary and completion callbacks
  run as TaskContext-owned blocking work after releasing the destination lease;
  auxiliary continuation reacquires the lease and revalidates exact generation,
  Worker state, destination, and cancellation. Auxiliary panic is an owned
  semantic failure; completion-callback panic is observed after verified
  completion and does not roll back that terminal state.
- Production-topology barrier tests hold actual capability-relative create,
  truncate/open, write, flush, remove, rename, and marker operations while the
  public cancellation path runs against a real recovery state/capability. None
  publishes terminal cancellation, releases recovery capability, or mutates
  after terminal state before the operation is released and observed. Separate
  barriers cover recovery revocation versus generic resume/relocate/cancel,
  caller drop across transition-to-worker handoff, owner-drain cancellation,
  and snapshot-to-owner-install reconciliation.
- Focused results at the fourth corrected re-freeze: the lifecycle module passes
  18/18 and the download module passes 85/85 in default mode; the same 18/18 and
  85/85 pass with `--no-default-features`. Both affected all-target Clippy
  modes pass with `-D warnings`; workspace rustfmt check and scoped
  `git diff --check` pass. At committed Coding Standards revision
  `ef400727e2d81a467af64b95f64e7c631096faee`, the Python-only registered
  `planning-consolidation` suite and direct `validate_plan` evaluation of all
  five Pumas plan contents pass. The retired Bash helper and obsolete mixed
  checkpoint are not used. Final status/diff review confirms no Slice B source
  outside the exact four-file set, no newly staged path, and no commit.
- This remains a bounded download custody/observation slice. Slice C owns
  ordinary start publication before fallible preparation, ignored admission
  failure, ordinary resume admission order, snapshot broadcasts still sent
  while the destination lease is held, and the earlier direct ordinary
  auxiliary callback that is still synchronous/unisolated. General
  `HuggingFaceClient::drop` drain remains Milestone 4/RUST-A6; its current
  delegation requests abort but makes no synchronous drain claim. The admitted
  strict Ambient terminal-cleanup and callback boundary above are narrow
  exceptions, not a general ordinary-admission or callback-compliance claim. No
  full aggregate, consumer compatibility, or standalone-shippable result is
  claimed. Slice B is re-frozen for independent review and stages or commits
  nothing.

### 2026-09-03 — PRG-I19 Slice C atomic ordinary admission and publication

- Program and root formally accepted the fourth Slice B boundary, then admitted
  Slice C on exact source hashes `abfe0382` (`hf/lifecycle.rs`), `c69953b1`
  (`hf/mod.rs`), `2e75638f` (`hf/download.rs`), and `fdd00a3c`
  (`hf/types.rs`). Slice C may edit only those four source files plus this
  focused plan, ledger, issues record, and RPC contract/threat report.
- The active objective is one no-gap Worker admission for ordinary start and
  ordinary/recovery resume. Awaitable authentication, remote, preflight, and
  destination-authority work completes before mutation. Pure prepared download
  data becomes a gated lifecycle Worker only inside the no-await
  downloads-to-task critical section that also commits the exact destination,
  state, generation ownership, and `task_registered=true`. Guards are released
  before the start signal, and start occurs synchronously before another await
  or caller-cancellation point. Admission failure is never ignored.
- Directory, marker, configured persistence, and admitted callback setup belongs
  to the installed Worker and its `TaskContext`. A semantic, Join, panic, or
  missing-row result is observed and becomes sticky terminal failure rather
  than an ownerless active state. Ordinary resume has no request-owned late
  `Queued` writer. Pause is restricted to the exact started unfinished Worker;
  its persistence result is owned and verified before same-generation memory
  `Paused` publication.
- A private publication owner must serialize immutable state capture, revision
  allocation, and dispatch. Broadcast/callback signals occur outside download,
  task, and destination guards. The decisive concurrency oracle blocks an older
  active snapshot after construction but before dispatch, publishes a newer
  terminal state, then releases the older attempt and proves cursors and payload
  state never regress. The destination lease still intentionally serializes
  destination filesystem/network I/O; Slice C does not make a broader no-I/O
  claim for that lease.
- First red: `cargo test -p pumas-library
  ordinary_start_destination_setup_failure_is_owned_and_fail_closed --lib`
  initially failed because public start returned an I/O error after inserting a
  `Queued`, `task_registered=false` state. Its corrected contract is admitted
  ID followed by owner-projected sticky `Error` for execution-owned directory
  setup. The first green moves directory setup under the Worker and atomically
  installs the initial state plus real owner.
- Second red: `cargo test -p pumas-library
  cancelling_ordinary_resume_before_commit_preserves_paused_state --lib`
  initially observed `Queued` instead of the exact prior `Paused` state when the
  request was aborted while authentication preparation was held. The green
  prepares first and commits fresh flags, `Queued`, and the gated owner together.
- Remaining red-first matrix before freeze covers immediate pre/post-commit
  caller cancellation, deterministic owner-install rejection and retained
  retirement, stale same-destination contention, auth/remote/destination/setup/
  persistence/callback failure, ordinary and recovery resume cancellation,
  strict pause outcomes, retry/pause/cancel interleavings, Worker/callback panic,
  finished observation and repeat cancel, destination/snapshot/callback
  reentrancy, and reverse-release ordered snapshot publication.
- That initial four-file boundary was superseded after deterministic destination
  and restart counterexamples proved that a raw path queue plus volatile cleanup
  flag could not satisfy the admitted claim. Slice C remains active and
  unfrozen under the exact amendment below; Slice B semantics remain required
  regressions.

### 2026-09-04 — Amend Slice C for durable queue, quarantine, and held destination identity

- Root and program admitted exactly nine source files: the four HF lifecycle
  files above; `model_library/download_store.rs`; the existing untracked
  `model_library/download_recovery.rs`; narrow `api/builder.rs`; and narrow
  `metadata/{atomic,mod}.rs`. The same focused plan, ledger, issues, and RPC
  contract/threat report remain the only records. Admitted baselines are
  `020975b5` (store), `c7244b5a` (recovery), `95abe5fe` (builder), `c838e184`
  (atomic metadata), and `15adcd4f` (metadata module), in addition to the
  accepted Slice B hashes.
- The builder may only open the selected model-library root after its directory
  exists and inject a crate-private held authority; public `HuggingFaceClient`
  construction and wire outcomes do not change. The recovery file may only add
  the shared held destination authority/identity. Metadata may only generalize
  the accepted Slice A publisher to capability-relative marker bytes while
  preserving its typed pre-effect, visibility-unknown, durability-unknown, and
  durable outcomes.
- Store version 3 owns a two-phase durable admission attempt, non-authorizing
  destination identity, domain, FIFO ordinal, predecessor/release proof,
  ordinary row, and exclusive full-snapshot quarantine. Legacy/v1/v2 Error is
  recoverable. Pending cleanup is independent of sticky provenance; clean
  Pending removal and sticky Pending-to-Verified retain exact attempt/release
  evidence, Recovery keeps its durable revocation tombstone, and stale ordinary
  writers are rejected. Visibility or durability ambiguity parks custody and
  cannot authorize public success, verification, release, or empty restore.
- Destination reservations and effects must use the same configured-root handle
  plus validated relative target. Raw/canonical path spelling, nearest-existing
  ancestor identity, file order, UUID, creation time, or store write completion
  cannot define authority or FIFO. Restore strictly orders durable ordinals and
  blocks an orphan predecessor without release proof. Paused/recoverable Error
  and Pending retain custody; Verified sticky Error reconstructs without a
  claim but remains non-resumable. Fresh exact Recovery tickets may reattach
  Pending cleanup only, returning existing `Attached/Cancelling`; they never
  start a Worker.
- Red-first store evidence was captured before production APIs: the quarantine
  tests failed to compile on missing domain/disposition/load/begin/verify
  owners. The first implementation checkpoint passes 25 store tests with two
  subprocess helpers ignored, including explicit v1/v2 migration, Ambient and
  Recovery Pending-to-Verified roundtrip, tombstone retention, stale-writer
  rejection, and initiating-owner ambiguity. Review then exposed required
  sticky-versus-clean Pending and typed exact-attempt removal proof; those reds
  are active and the store is not frozen.
- Remaining mandatory evidence covers durable admission interruption/cross-
  writer ordering, reverse-save FIFO restore, orphan predecessor, exact marker
  publication ambiguity, root/alias/missing/replacement authority, restore and
  reconciliation custody, stalled-network pause, exact pause/terminal winners,
  atomic relocation, exact-generation terminal rescue, completion callback
  release/drain order, auxiliary-versus-payload identity, and every accepted
  Slice A/B regression in both feature modes.
- No other source/public/manifest/consumer file is admitted. Slices D-E, M2J,
  general client Drop, full aggregate, staging, commit, and standalone-
  shippable claims remain held.

### 2026-09-03 — Queue Milestone 2J model-search contract

- Active allowed write set: `rust/crates/pumas-rpc/src/contract.rs`,
  `rust/crates/pumas-rpc/src/handlers/mod.rs`,
  `rust/crates/pumas-rpc/src/handlers/models/search.rs`,
  `rust/crates/pumas-rpc/src/wrapper.rs`,
  `rust/crates/pumas-rpc/tests/integration_tests.rs`,
  `rust/crates/pumas-core/src/api/hf.rs`,
  `rust/crates/pumas-core/src/api/state_hf.rs`,
  `rust/crates/pumas-core/src/api/state.rs`,
  `rust/crates/pumas-core/tests/api_tests.rs`, and the compile-only direct
  caller `rust/crates/pumas-uniffi/src/bindings/api_hf.rs`, plus this focused
  plan/ledger and `reports/rpc-contract-and-threat-model.md`.
- Excluded in this slice: manifests, import/metadata/governance operations,
  plugin commands/loaders, local IPC operation expansion, CI, generated
  artifacts, package scripts, and shared project documentation.
- Goal: close `search_hf_models`, `get_hf_download_details`,
  `get_related_models`, and `search_models_fts`, including bounded numeric and
  collection inputs, typed outcomes, truthful HF-unavailable behavior, and no
  handler defaults that turn local/HF failures into empty success.
- Evidence/results: blocked on corrected M2I acceptance. One isolated red core
  test already proves disabled HF search currently returns successful empty
  results; no M2J production source edit is admitted until M2I closes.

### 2026-09-04 — Implement audit recommendations as C1–C4 checkpoints

- The user approved the read-only implementation audit and its recommendations.
  C1 now owns store repair before destination authority (C2), production
  lifecycle integration (C3), and actual importer ownership (C4). The focused
  and root plans preserve the full end-to-end acceptance obligations.
- Fixed the audited `MutexGuard`/`Clone::clone` compiler mismatch. The first
  executable store baseline then reported 29 passing tests, one failing
  admission/reopen test, and two ignored subprocess helpers. This replaces
  the compile-only red with a behavioral red; it is not C1 acceptance.
- The bounded persistence comparison selected JSON for this checkpoint.
  SQLite is already available but its authority, journal durability, and JSON
  handover would require separate proof; no database migration is introduced.
- Reopening explicitly reconciles recorded transitions through a successful
  publication barrier. Tests must not expect identical persisted bytes to
  reveal whether an earlier process observed a successful sync.
- Store implementation and independent review are scoped to C1. Current
  source was not frozen at that initial red; broad producer/consumer, GUI, and
  release claims remain pending.
- C1 repair freeze:
  `1d6335a31b434a16dfdbd4f1f9c93860a76f6a1f21ed5af32efbc2247ccbf33c`
  (`download_store.rs`). Independent source review accepted the corrected
  admission confirmation/reconciliation, physical FIFO, exact terminal release
  records, outgoing validation, immutable snapshot checks, attempt uniqueness,
  and sticky retry behavior. Each reported defect received a failing regression
  before correction.
- Final store suites: 38 passed, two subprocess helpers ignored in both default
  and no-default features. Root reproduced passing atomic-publication tests and
  155 HF tests in each feature configuration. `cargo check --offline -p
  pumas-library --lib` succeeds. Production compilation reports 17 unused
  integration warning groups; store test builds report five. No warning was
  suppressed and no warning-free integration claim is made.
- Root reran current `validate_plan`: all five plans valid. Scoped formatting
  and diff checks pass. C1 is internally verified; C2 is next. Release records
  are retained without GC. Queued legacy mutations refuse unsafe changes until
  C3 supplies dedicated transitions. Runtime owners still must complete effects
  before settlement and reconcile inventory before restart admission.

### 2026-09-04 — C2 held destination authority and marker publication

- Shared ordinary/recovery destination capabilities now derive equality from
  the held configured-root identity and validated relative target. Root aliases
  and missing-tail creation retain identity; nested symlinks are rejected.
- Destination effects retain model, creation-anchor, and nested file-parent
  directory identities. Independent review caught nested-parent replacement
  between partial-file writing and finalization. The regression now refuses
  rename/removal against replacement directories, and missing previously held
  parents are not recreated before rejection.
- Marker objects use the existing held-parent atomic publisher and preserve its
  durable/unknown outcome algebra. Newly created directory links are synced.
  A first test failure exposed Linux O_PATH directory handles being unsuitable
  for fsync; the correction opens `.` relative to the held directory for sync.
- Builder initialization establishes ModelLibrary before capturing authority.
  Authority-unavailable configuration keeps the initialized HF search client.
  Ordinary mutation refusal and runtime capability consumption remain C3.
- Independent source review accepted recovery `28ce9492`, atomic `3b49f195`,
  HF module `bd5c1e74`, and builder `1f5fa98b`. Root reproduced 155 HF, 38 store,
  14 recovery, and 22 atomic-publication tests in each of default and
  no-default-features configurations. Store suites ignore two subprocess
  helpers; atomic suites ignore one. Both library checks pass, with 23 unused
  integration warning groups; test builds report five. None were suppressed.
- Root corrected scoped formatting to the workspace's Rust 2021 edition after
  review and test reproduction. Final source hashes (format-only delta):
  `download_recovery.rs`
  `9cae63554ad555d3b469beedb3d0198c3419bbce250742b1d6cebe4a05846322`;
  `metadata/atomic.rs`
  `890df1df857db42f76726605e140434a5f8f81b59bd12d11f296b785241c3b2d`;
  `hf/mod.rs`
  `7a66dbfa2eb3e4abc0cc04a6789d8b536b8ef21f0599703a96308f5f051ab2a6`;
  `api/builder.rs`
  `af4a7fbf37a3389b4a838140d066afa061f2c58bd3ef6445c818609b29e67b02`.
  Earlier recursive formatting incidentally touched HF child modules; no
  semantic changes there were intended, and the HF suites above passed.
- After final formatting, the isolated public `api_tests test_api_creation`
  smoke suite passes all four tests: successful startup, automatic directory
  creation, nonexistent-path refusal, and idle clean startup. This does not
  prove the new missing-model-root/auto-create-disabled edge or unsupported
  platform behavior.
- C2 is internally verified; C3 is next. No full builder-startup, non-Linux,
  GUI, warning-free, or end-to-end producer/consumer acceptance is claimed.
  No changes were staged or committed; the prior staged UniFFI file remains.

### 2026-09-04 — C3 production admission integration started

- Continued the active focused plan under the user's instruction to proceed.
  HF runtime implementation owns `download.rs`, `lifecycle.rs`, `types.rs`, and
  `mod.rs`; the store owner supplies only concrete required transitions.
  Root owns serial plan/evidence updates and final verification. Existing
  unrelated changes and the staged UniFFI file are preserved.
- First production-start regression is red: `start_download` returns an ID
  while strict lifecycle inventory has no confirmed durable queue admission.
  This initial tracer injects marker setup failure and a fixture partial file;
  it does not yet prove actual byte transfer or the full C3 path.
- Added exact owned `update_admitted_status(id, attempt, status)` for confirmed
  active admissions, leaving immutable request/queue identity unchanged.
  Terminal/cancelling statuses require their dedicated lifecycle transitions;
  generic status mutation cannot bypass admission ownership. Default store
  suite passes 39 tests with two subprocess helpers ignored. Independent store
  review accepted source `72f93ae6`; composed runtime review and no-default
  reproduction remain pending; no C3 acceptance is claimed.
- The initial marker-failure/restart/cancel tracer passed after owned admission,
  exact status persistence, strict restore, and exact cancellation settlement
  were connected. Broader regressions and real byte-transfer evidence remain
  required; this narrow result does not close C3.
- `restore_persisted_downloads` now returns `Result<Vec<_>>`, and the builder
  propagates failure directly. Repository lookup found only the builder and
  colocated Rust tests consuming this method; callers migrate together. No
  wire outcome or new readiness state is introduced. Strict authoritative-store
  failure prevents successful initialization instead of claiming empty restore.

### 2026-09-04 — C3 narrow admission checkpoint verified

- Ordinary start now completes task-owned durable admission before returning
  an ID, publishing state, or running destination effects. Concurrent identical
  starts share the existing task owner's pending-admission completion signal.
  Missing required configuration is `Config`; failure preserves its cause and
  does not leave a provisional runtime queue reservation.
- Admitted status, resume, completion, and cancellation carry the exact attempt
  and held destination. Strict restore propagates errors through the builder.
  The tested restart/resume path uses controlled marker failure and known file
  bytes; it is not interrupted network-transfer or complete C3 evidence.
- Independent review found four introduced problems, each corrected with an
  observed failing regression: settlement before effect drain, stale inventory
  recreating released claims, hidden predecessors appearing during admission,
  and admitted resume returning to legacy authority/status mutation. Release
  facts now remain with the runtime queue until owner drop; they are recorded
  only by generation-validated release. Unknown or sticky state retains custody
  where complete quarantine reconciliation remains unavailable.
- Consumer tests also exposed absent-target cleanup: never-created target or
  nested payload directories are successful no-op cleanup, but lost held
  directories, replaced anchors, and invalid paths still fail. Both regressions
  were observed failing before correction; the 16-test recovery suite passed
  and independent source review accepted this capability change.
- Root reproduced both default and `--no-default-features` configurations with
  `cargo test --offline -p pumas-library --lib` and filters
  `model_library::hf` (168, comprising 162 HF and six cache tests),
  `model_library::download_store` (39 plus two ignored subprocess helpers),
  `model_library::download_recovery` (16), and `metadata::atomic::tests`
  (22 plus one ignored helper): 245 passed per configuration. Earlier counts
  using `model_library::hf::` exclude cache tests and are not interchangeable.
- Both `cargo check --offline -p pumas-library --lib` configurations passed
  with 11 unused-integration warning groups; test builds report six. Scoped
  Clippy completed and identified one new boolean-style warning, corrected
  without changing its predicate. Existing large-enum and filter-map advisories
  remain; no warning-free claim or suppression is made.
- The public `api_tests test_api_creation -- --test-threads=1` smoke initially
  failed with OS `EPERM` in the HF-disabled test, poisoning the shared test
  lock. The unchanged binary passed all four tests with approved local IPC
  permissions and its temporary launcher/registry fixtures. This is environment
  qualification, not a product fallback or a weakened test.
- Independently reviewed source hashes: HF `download.rs` `f18a3e8ac2f68625030e5cb6cb2a36c016f188798cc67e86b3013bcd8d87df83`,
  `lifecycle.rs` `c00fd86f7de7e2fce6010b616df0ac2254fd28bc2a974b3fe6aea99ec31bb93b`,
  `types.rs` `aba9f05db91b65256aa5d357664d028555d66c60e533f4c8b9d4e95a407a55ae`,
  `mod.rs` `7a66dbfa2eb3e4abc0cc04a6789d8b536b8ef21f0599703a96308f5f051ab2a6`,
  recovery `10db29a32ba5b7b07f25ec0de2d9a1e86abfc3d18d8f3994f21b417653f12065`.
  Store review accepted `72f93ae6`; final equivalent lint simplification yields
  `adf196472911b4746314b4de45912d0633b1aa93a120b8b835982e1227705320`.
  Root reran both store suites after that simplification (39 passed each) and
  Clippy (completed with 11 integration warnings and two existing advisories).
  Builder's direct restore-error propagation is
  `d6c2f362cd983273d28e140d121580c825773e8df54f9718a8c68c467b1970b2`.
- Full C3 stays active. Remaining obligations include real transferred-byte
  evidence, stable runtime queue identity, removal of the physical effect
  mutex, owned comprehensive restore, legacy/hidden/quarantined/recovery state,
  stalled pause, and relocation. C4 importer ownership, consumer migration,
  GUI, platform, and full-program acceptance remain held. No changes were
  staged or committed; the pre-existing staged UniFFI file remains untouched.
  Final scoped Rust formatting, diff checks, and all five current plan-contract
  validations passed.

### 2026-09-04 — C3 Real Interrupted-Response Checkpoint

- Added `transferred_partial_survives_interrupted_response_and_fresh_owner_cancel`
  through public start, progress, restore, and cancellation operations. The
  loopback fixture sends seven bytes of a twelve-byte body, truncates the
  response after observed progress, checks the retry's `Range: bytes=7-`, and
  returns a terminal HTTP refusal. A fresh owner restores 7/12-byte progress;
  cancellation removes the partial file and marker, and another fresh owner
  restores no download. Payload bytes are not seeded on disk.
- The only new endpoint override is private and `cfg(test)` in `hf/mod.rs` and
  `hf/download.rs`; production retains the canonical HF endpoint. Root review
  replaced single-read HTTP assumptions with bounded complete-header reads and
  replaced a private queue-map assertion with public fresh-owner evidence.
- `cargo test --offline -p pumas-library --lib model_library::hf::` passed
  163 tests in both default and `--no-default-features` configurations. A
  sandbox run failed at loopback bind with EPERM; approved socket-access reruns
  passed without suppressing the failure. Both `cargo check --offline -p
  pumas-library --lib` modes passed with the existing 11 integration-warning
  groups. Scoped Rustfmt (`--edition 2021 --check --config skip_children=true`)
  passed for the two changed files.
- Verified SHA-256: `hf/download.rs`
  `507e6a66f664bc2ed3319ae00769248a2f46c136d01a715b4cc50fb83d803eef`;
  `hf/mod.rs`
  `25ab92ce5c4d063e9b8e744ab4ab4f87d5a16f90a3b3741ebb666972e626785b`.
- This is interrupted HTTP followed by orderly Error settlement and reopening,
  not a hard process crash, stalled pause, live HF-service, complete queue/restore,
  or importer claim. No new production defect/red-to-green fix is claimed.
  C3 remains active; this source cannot be committed independently of its
  still-uncommitted lifecycle dependencies and compatible consumer integration.

### 2026-09-04 — Queue Identity Draft and Relocation Scope Blocker

- Operation: continue C3. TDD at the admitted public restore/start/cancel seam
  exposed a legacy paused row restored through a root symlink alias bypassed by
  a canonical-path successor. The original code reached destination preparation;
  the new regression reported that bypass explicitly.
- The unaccepted draft replaces path-keyed queues and physical-lock lookup with
  held `DestinationIdentity`, consolidates state capability into explicit
  Managed/Recovery provenance, and reuses capability-relative operations during
  legacy restoration. The alias test is green and also proves successor progress
  after the incumbent is cancelled. The design skill guided removal of duplicate
  capability storage and reuse of the existing finalization policy owner.
- Supporting scope correction: `partial_download.rs` joins the existing C3
  source set solely for a shared filesystem Interface. This replaces the prior
  nine-file count restriction, without changing the held consumer, public wire,
  manifest, or migration-caller boundary. Its independent exact candidate over
  `acbc726c` passed `cargo test --offline -p pumas-library --lib
  model_library::library::tests` (131 tests) in default and no-default modes,
  default `cargo clippy --offline -p pumas-library --lib --tests -- -D warnings`,
  scoped Rustfmt, source review, staged review, message validation, and enabled
  hooks. Committed as `09fd0777`; SHA-256
  `2fe574ac496e426f3ddbb396a1406020b98e6b705c10f2f4a5551abff22d0af9`.
- The queue draft is **not accepted**. Final default `cargo test --offline -p
  pumas-library --lib model_library::hf::` reports 163 passed and one failed:
  `test_relocate_download_destination_updates_state_and_persistence`. The
  original post-relocation lookup assertion remains intact and the fixture now
  supplies a real configured root/held old destination. A no-default run before
  that fixture/configuration and formatting cleanup likewise reported 163/1;
  it is not final-hash acceptance. Production checks and scoped formatting pass,
  but do not override the failed behavioral gate.
- RUST-I8/PRG-I24 blocks further queue integration. The existing relocation
  path changes display/persisted metadata without transferring retained
  capability and queue ownership. The actual migration caller moves files
  first, ignores `false`, and attempts rollback on `Err`; the store does not
  retain expected-old/publication-outcome proof. A generic task wrapper, raw-path
  lookup fallback, silently disabled legacy move, or weaker assertion is not a
  repair. The user was asked to admit coordinated migration/preflight/owned
  relocation work; no caller modification or guessed rollback was made.
- Queue-checkpoint draft SHA-256 (before the probe safety correction below): `hf/download.rs`
  `18b42552372c80759f3c7117e8b9763f089b361497281cc551e2286b7751c953`;
  `hf/types.rs`
  `6e154a28bba03ea0c08528ae529ab586b4b561564d452ba8e8506cf79fc7db45`;
  `hf/mod.rs`
  `2ea1946f13ae34eccdc6e3247b1e3706dc4cd30f9ef5aa0c5eb46f07579a5e02`;
  `download_recovery.rs`
  `73083c3c011dc711ff0b0444f26bb9eacc49079d132262438c653855eda38087`.
  These are uncommitted draft subjects, not accepted integration hashes. Do not
  use this draft for model migration. The original staged UniFFI change remains
  separately owned and unchanged.
- Independent review then found that file-size probes converted lost held-parent
  authority into ordinary absence, allowing restore to discard its persisted
  record as empty. The new `file_probes_distinguish_uncreated_paths_from_lost_authority`
  regression failed at the lost nested-parent assertion before the fix.
  `regular_file_len` now uses the existing `file_parent_if_present` distinction:
  never-created paths are absent; lost authority propagates an error. Both final
  and partial probes are covered for nested-parent and model-directory loss.
  `cargo test --offline -p pumas-library --lib
  model_library::download_recovery::tests` passes all 17 tests, also with
  `--no-default-features`; six existing unused-integration warnings remain.
  Scoped Rustfmt and independent narrow source review pass. The corrected
  `download_recovery.rs` SHA-256 is
  `2652a7e5eb89039e48200cc0a465a97d61a63e9dac636e5e17ec55b20f35ab96`.
  This safety correction remains in the uncommitted draft; the HF results above
  precede it, and no relocation or full-C3 acceptance is claimed.

### 2026-09-04 — Approved Relocation Composition

- The user's affirmative response approves coordinated model migration and
  owned relocation. This supersedes RUST-I8's scope hold, not any failed gate.
  Continue `docs/plans/current-standards-remediation-2026-09-03/rust-library-and-rpc/plan.md`;
  the plan is Active and the source draft remains unaccepted.
- The first implementation preserves existing admitted/recovery refusal before
  effects and supplies the missing legacy physical-move owner. A Pending store
  intent precedes movement and blocks both destinations. After proven durable
  filesystem/marker completion, finish atomically publishes target row and
  removes intent; a visible target-only document is then valid after restart.
  A remaining Pending document stays blocked. No automatic rollback, Applied
  receipt registry, or guessed filesystem reconciliation is introduced.
- Root owns migration orchestration and records. Its two consumers pass the
  real model library and optional HF client explicitly; the `api/state.rs`
  expansion is limited to that call site and changes no IPC representation.
  Store, capability, and HF source owners are separate; Cargo runs serialize.
  Independent review checks the store's entire mutation family.
- No-replace movement uses the existing Linux capability/libc mechanisms;
  unsupported kernels/filesystems/targets return explicit failure, not copy or
  replacing rename. Pumas excludes its own competing directory mutations.
  External source-name swaps are checked before/after but cannot be atomically
  prevented by renameat2; changed postconditions remain indeterminate without
  cleanup. This is not a hostile-writer exclusion or cross-platform release claim.

- Implementation and independent narrow reviews now agree on the legacy
  composition. Store review reproduced a foreign ordinary/quarantined row
  bypass at either path; the shared validator now rejects all four cases.
  Capability review added durable ancestor links, post-move source checks, and
  nonblocking/no-follow marker reads that reject FIFOs and non-object JSON.
  HF review moved queue notification outside the state guard, added pending
  destination rechecks, and retained ownership when the caller is dropped.
  Cancellation waits for relocation rather than aborting its physical work.
- The migration regression failed before replacement: the caller reported
  `moved_partial` despite lifecycle refusal. It now delegates to HF with the
  expected source identity, preserves bytes on refusal, and proves successful
  movement through the real caller, marker, reopened store, and model index.
  Post-move index/resume errors retain `moved_partial` plus an error; report
  counts truthfully record both. Complete models keep their existing path;
  untracked partials are explicitly skipped. No global runtime registry or live
  Hugging Face service is used by this fixture.
- Final reviewed draft SHA-256: `hf/download.rs`
  `082187ffb0b4acbe603aef343c3d8e1c9cb0777f0072e836c82402cda33ea2c3`;
  `hf/lifecycle.rs`
  `3d7ea27ec5aac9eda7e2dc176b4a50b9506efde01b2d4fc543e4da987d4b053f`;
  `hf/types.rs`
  `219271b3dbd13aac6f3299e9d432085bf383a783e7341a013888061a7018a770`;
  `download_store.rs`
  `172c3d0d4050e787c2b084c2310fc396763827e6dadfb15b0b66e6f4cef3293b`;
  `download_recovery.rs`
  `530004beca609217a22ec4e68d78da6be0619bdf2d140d861283b994ccbe234c`;
  root-owned `api/migration.rs`
  `1d5346b8c370333aae8b3d466045d3e3a7be85aa63c7ecf35c1cd6de8743ebd3`.
  Reviews accept this narrow source protocol, not full C3 or integration.
- An unregistered archive candidate at
  `/tmp/pumas-relocation-candidate.jZ3nzd` is based on `f0f89ed2`. It includes
  the workspace/core manifests and lockfile, metadata atomic module and export,
  model-library export, store/capability, four HF owner files, builder, migration,
  and only the migration argument hunk in `api/state.rs`. It excludes the held
  API/RPC recovery wire, bindings, Electron/UI, and unrelated formatting changes.
  Builds serialize using the repository's existing Cargo target directory;
  the candidate itself was compiled, not merely the mixed working tree.
- Final isolated behavioral matrix: `cargo test --offline -q -p pumas-library
  --lib <filter>` passes with both default and `--no-default-features` for
  `model_library::hf::` (168), `api::migration::` (3),
  `model_library::download_recovery::` (23), `model_library::download_store::`
  (43, two helper tests ignored), and `metadata::atomic::` (22, one helper
  ignored): 259 passes per configuration. The initial sandboxed HF run had
  167 passes and a loopback-bind permission failure; the approved socket-access
  rerun passes all 168. This is automated local contract evidence, not live
  service or hard-process-crash acceptance. Scoped formatting, diff checks,
  and all five plan-contract checks pass.
- Isolated production `cargo check --offline -p pumas-library` passes in default
  and no-default configurations, with unused-integration warnings (35 in the
  default check). Strict
  `cargo clippy --offline -p pumas-library --all-targets --all-features -- -D warnings`
  fails (39 library and 14 library-test diagnostics). Recovery consumers excluded
  from the slice leave dead code/imports; other findings include large enum
  variants, bool-then filtering, and question-mark simplifications. This is an
  affected-package failure of the repository's strict policy, not a successful
  workspace gate. No lint was suppressed and no source commit is accepted.
- Next: inventory the production core recovery ticket/validation, admission,
  and real `api/hf.rs` caller contracts before admitting that exact integration
  boundary; repair lints in their existing owners and reverify the isolated
  result. Do not manufacture callers, publish private helpers, or silently add
  held RPC/binding/UI consumers to make the gate green. Current standards route
  revision is `0b10cb84`; prior revision evidence remains historical.
- Limits remain explicit: one active runtime per root; pending restart blocks
  both locations without guessing reconciliation; only legacy dormant moves are
  supported here. Notification occurs outside the state guard after snapshot,
  but no snapshot-before-successor-first-poll guarantee is claimed. Admitted
  relocation, comprehensive owned restore/quarantine, stalled pause,
  guard-free physical effects, hard-process-crash proof, and C4 importer work
  remain open. The GUI was not validated. The original staged UniFFI blob
  `2af018c1e1c27effc7fb3ddf2c77f0570cb78fb9` remains separately owned.

### 2026-09-04 — Core Recovery Integration

- The user explicitly approved finishing core recovery integration and lint
  repairs before source commit. Continue the focused plan. The additive
  `PumasApi::resume_partial_download_with_ticket` consumes an indexed model ID
  and previously issued ticket; it resolves repository/filesystem authority in
  the core. The old repo/path method remains byte-identical to HEAD for real
  RPC and binding consumers. It does not acquire stale-ticket protection or
  synthesize a fresh ticket. Later transports must explicitly select the new
  method; this checkpoint changes no wire shape.
- The isolated candidate excludes held Result-shape, reconciliation, RPC,
  binding, and GUI changes. Its API tests construct real local primary state
  without registry overrides, IPC listeners, watchers, or background probes. A
  bounded loopback server proves exact selected-file Range recovery, followed
  by public cancellation and unrelated-artifact preservation. Stale artifact
  selection refuses admission without touching payloads. The legacy missing-
  directory result is retained. Filesystem ticket inspection uses the existing
  HF task/blocking owner, not detached API work.
- Store projections no longer duplicate unused persisted attempts, snapshots,
  release maps, or live queue positions. Canonical durable records, exact
  replay/publication, and predecessor validation remain. The write-only release
  confirmation cache and obsolete private relocation adapter were removed;
  tests now assert the actual store transition and reopened inventory.
  Review rejected an initial generic-error conversion; the corrected conversion
  preserves typed I/O paths/sources and publication/cleanup context.
- Cancellation now begins durable full-snapshot quarantine before payload
  cleanup, drains effects before verification/settlement, and preserves sticky
  failure provenance. The first regression was red because a fresh client
  restored the row while cancellation was held before cleanup. A later review
  reproduced a second red: cancellation after durable recovery revocation but
  before snapshot handoff lost quarantine evidence. The snapshot must be in
  cancellation-visible custody before revocation; post-drain domain selection
  still requires the store's durable revocation validation before effects.
- The first isolated strict all-target/all-feature Clippy run passed, but its
  full library run had 1,070 passes, one outdated ordinary-row assertion, and
  three ignored helpers. This precedes the revocation-window correction and is
  not final-source acceptance. The assertion is replaced by exact quarantine
  preservation and fresh-restore refusal, not a weaker empty-state check.
- Standards revision `b8805364` changes standards tooling/audit artifacts since
  `0b10cb84`, not the applicable normative route. The design and TDD skills kept
  the new work at the existing public recovery/cancel and store transitions.
  Final verification and the accepted incremental boundary follow below.
- Final narrow source review accepts the revocation handoff and restore policy:
  Pending/hidden state and quarantines with active queue admissions still
  refuse startup; confirmed Verified Recovery history stays hidden behind its
  durable tombstone; Verified Ambient history is a read-only Error with no
  destination capability. No queue release is inferred from visible cleanup.
  Both-domain tests prove Pending preservation, successful retry, and fresh
  restoration. The verified-only Ambient fixture also caught and corrected an
  early return when no ordinary downloads existed.
- Candidate `/tmp/pumas-core-integration.oBhAM2` is an unregistered archive based
  on `a17c5b32`, not a branch or worktree. It adds the recovery-only API hunks to
  the prior 15-file boundary; `api/state.rs` still contains only migration
  arguments. Final reviewed SHA-256: `hf/download.rs`
  `5969615e42b5a5bd3968fcc6f77ca6e6d1aefccedb944aaa040d974d2d2cd20d`;
  `hf/lifecycle.rs`
  `39947dda9c08bfb67908e809e767977c55af3629764e8bd66b83551199b44288`;
  `hf/types.rs`
  `dcc7f480eda800a3bf071f5ad45b2da8bcb9feed1d6102ec0dccfa54040efa00`;
  `hf/mod.rs`
  `c4468ec218b61ecf483b58580af83c86d8d506122c17da133786944866f7c307`;
  `download_store.rs`
  `230f68caf2b55c6b73f7ebf5e7bd3ddd05a3725521a7ff11edf81b728a0ae3be`;
  isolated `api/hf.rs`
  `73fa9bb6d507c5b982d9d4b3f262bde2f3c0952ba713917b609619baf98285ac`.
  Capability and migration retain the prior checkpoint's reviewed hashes.
- Final candidate verification on Linux with Rust/Cargo 1.92.0:
  `cargo test --offline -p pumas-library`, also with `--no-default-features`,
  passes 1,175 checks per configuration: 1,073 library tests, 96 integration
  tests, and six doctests. Three existing helper tests and eight existing
  doctests remain ignored; no new ignore or assertion weakening was added.
  Loopback socket access was explicitly enabled for real local HTTP/IPC tests.
- Strict core Clippy passes with `--all-targets --all-features -- -D warnings`
  and separately with `--all-targets --no-default-features -- -D warnings`.
  `cargo check --offline --workspace --exclude pumas_rustler --all-targets
  --all-features` passes against unchanged HEAD consumers. The same supported
  workspace's strict all-target/all-feature Clippy passes. Rustler remains
  excluded by the repository's existing BEAM-tooling policy, not a new bypass.
  Scoped Rustfmt and diff checks pass; the plan-contract checks are rerun on
  the exact staged records before commit. No warning suppression was used for
  these lint repairs.
- Accept this coherent 16-source-file incremental boundary, plus its six plan,
  issue, and ledger records. Public migration obligations are recorded in
  Rustdoc and the breaking-change commit: standalone HF clients lack mutation
  authority, restore is fallible, and relocation owns the physical move.
  The original staged UniFFI blob remains separately owned. No push, transport
  migration, GUI verification, cross-platform release, full C3, or C4 acceptance
  follows. Unknown admissions/Pending state, stalled pause, admitted relocation,
  guard-free effects, actual importer ownership, and hard-process-crash proof
  remain with their existing owners; stalled pause is the next slice.

### 2026-09-04 — Stalled Network-Wait Pause

- Continued C3 at the admitted public pause/status seam. Three separate reds
  held response headers, held a body after two transferred bytes, and entered
  retry backoff: each remained Pausing past the fixture deadline. The fix
  wakes the exact worker generation at those waits, not during file writes.
  The atomic flag remains request truth; registration precedes its check and
  notification occurs outside the state guard. Body pause awaits owned flush;
  existing settlement owns persistence and cancellation precedence.
- Five public regressions preserve partial bytes, restore durable Paused,
  cancel during held pause persistence without stale publication/restart
  resurrection, and immediately resume without first draining the old owner.
  Fresh-owner and immediate resume issue real `Range: bytes=5-` requests and
  finish the exact `abcdefgh` payload. All 15 focused `pause_` checks pass.
  TDD supplied the red–green sequence; design review kept wakeup custody in
  the existing lifecycle owner rather than adding a public control surface,
  separate registry, polling loop, or shorter network timeout.
- Independent narrow review accepted the frozen two-file source. Candidate
  `/tmp/pumas-pause-candidate.6QCrpx` is an unregistered archive of `d8c12f4e`
  plus only `hf/download.rs` and `hf/lifecycle.rs`, excluding held API/RPC/UI
  drafts. Reviewed SHA-256: download
  `acdb4ba35f9571d2b85e9269544d07a5a388abb38798e3338ee52f6d1c94759c`;
  lifecycle
  `f5ed2d4c08e855a7a6b6ab8a1b35a81206133550e4285c2ce1ba6adadd1cf2ef`.
- Root verification on Linux, Rust/Cargo 1.92.0:
  `cargo test --offline -p pumas-library`, also with `--no-default-features`,
  passes 1,180 checks per configuration: 1,078 library tests, 96 integration
  tests, and six doctests. The existing three helper and eight doctest ignores
  are unchanged. Strict core Clippy passes with `--all-targets --all-features
  -- -D warnings` and separately `--all-targets --no-default-features
  -- -D warnings`. Scoped Rustfmt and diff checks pass. Local loopback access
  enables real socket fixtures; no live Hugging Face service is required.
- `cargo clippy --offline --workspace --exclude pumas_rustler --all-targets
  --all-features -- -D warnings` also passes, compiling unchanged committed
  consumers. Rustler retains the existing BEAM-tooling exclusion. Accept this
  two-source-file incremental checkpoint with its plan/issue/ledger updates;
  all five plan contracts pass before commit. The standards revision is
  `97789165`; applicable normative Markdown is unchanged since
  `b8805364`. No dependency, public signature, persistence schema, or held
  consumer changes are included.
- Network waits are the bounded result, not complete C3 acceptance. Read-only
  review identified reachable queued-pause starvation behind a paused FIFO
  predecessor; `RUST-I9` owns the next slice and its public regression.
  Comprehensive unresolved-state restore, admitted relocation, guard-free
  effects/publication, hard-process-crash proof, and C4 importer ownership
  remain open. No GUI, release, or cross-platform acceptance is claimed.

### 2026-09-04 — Queued Pause Exposes Missing Durable Legacy Ownership

- The public queued-pause regression failed while a paused predecessor held
  the destination. A draft select on the existing generation wakeup settles
  pause without entering destination work or releasing the FIFO claim.
- The next real HTTP regression found that pre-execution pause discards the
  pending marker setup. The draft regenerates admitted-resume markers after
  queue acquisition from the exact strict admission snapshot and canonical
  `requested_payload_files`, using the same serializer as original start.
  Restored per-file sizes are not authoritative selection data. Same-client
  pause/resume now passes: no request before head release, exact marker during
  held headers, and successful real payload transfer afterward.
- A strengthened fresh-client regression remains red: a successor issues HTTP
  before its legacy predecessor is released. The initial immediate Queued
  assertion missed this race and is not FIFO evidence. Store admission ordinals
  omit legacy incumbents; restore sorts admitted entries ahead of the legacy
  fallback. Independent review confirms the missing durable predecessor proof.
- Freeze the source draft and retain the failing restart oracle. `RUST-I10`
  requires a decision on durable legacy ownership/migration before extending
  the two-file implementation. A guessed legacy-first sort, replacement with
  an easier all-admitted fixture, or silent restore refusal cannot satisfy the
  selected transparent restart claim. Source is not committed or accepted;
  full package and lint verification remain pending after that decision.
- Standards revision `bf9eafa6` has no applicable normative Markdown changes
  since `97789165`. The isolated archive
  `/tmp/pumas-queued-pause-candidate.9CQOAU` currently contains only committed
  `c04dd119`; it has not been populated or verified with this source draft.

### 2026-09-04 — Current-Only Cutover Implementation

- The user explicitly authorized updating the existing library once and
  dropping legacy support. Runtime schema v4 rejects earlier documents and
  unadmitted ordinary snapshots without conversion. This changes the selected
  contract; the preceding legacy restart failure remains causal evidence.
- Read-only inspection found two paused unversioned records in distinct
  destinations, with explicit payload selections matching their markers.
  Source SHA-256 is
  `060e438e72dd86683398d8a5cb7dc1cfc3c33dbf90e07be6b4093291eb83beb0`.
  File-preservation hashes were captured outside the repository. No live
  publication has occurred. The converter is temporary operator tooling,
  not a startup fallback or shipped compatibility layer.
- Store and lifecycle owners removed unowned ordinary mutation/relocation
  paths and introduced an exact admitted recovery handoff preserving queue
  position and snapshot provenance. Partial-download relocation reports
  unsupported; ordinary completed-model migration remains unchanged.
- The isolated candidate has now been populated with selected source and
  the exact migration caller hunk, excluding unrelated working-tree drafts.
  Its first production check found a removed constructor still used by the
  committed caller; the constructor was restored. Initial strict Clippy found
  a cancellation loop made obsolete by relocation removal; it is now a direct
  block. Neither diagnostic was suppressed.
- Store tests pass: 39 tests and two existing ignored subprocess helpers.
  Current automatic finishing first reproduced a missing-implementation RED,
  then passed with a no-network owned finalizer and exact queue settlement.
  Independent review found a second race: public status observation could
  retire that owner before restore collected its importer result. The
  deterministic regression reproduced zero completion records instead of one;
  the retained task now hands off its own result, preserving a same-ID successor.
  The isolated regression is GREEN, and independent review reports no further
  narrow blocker. Full core package tests now pass in both default and
  no-default-feature configurations: 1,064 library tests, 96 integration tests,
  and six doctests per configuration (1,166 passes; three existing library
  helpers and eight existing doctests ignored). The reduced count reflects
  removal of obsolete compatibility/relocation tests, not new ignores.
  Strict core Clippy passes for both `--all-targets --all-features` and
  `--all-targets --no-default-features`; strict supported-workspace Clippy
  passes with `--workspace --all-targets --all-features --exclude pumas_rustler`.
  Every invocation used `-D warnings`, offline dependencies, and the isolated
  candidate. No unrelated working-tree drafts were used as acceptance evidence.
- The temporary converter compiles and its three fixture tests pass. Live
  dry-run produced an exact synced backup and a schema-v4 candidate preserving
  both original snapshots field-for-field. Candidate hash:
  `cfdfd4c1be9c38d247052f497b5628c04bdce4f0098e81ec26b5509d3d4fb7d5`.
  Private artifacts and operator provenance are retained outside application
  data and excluded from the source commit. After final stopped-writer and hash
  checks, publication returned `durable_verified`. Readback exactly matches the
  candidate; both snapshots and the original backup are preserved. All 22
  existing files retain their hashes and all 38 previously absent paths remain
  absent. No model, auxiliary, partial-file, marker, or metadata bytes changed;
  both downloads remain paused. The temporary converter is not shipped and its
  execute permissions were removed after proof; private source/provenance remains.
- The development `pumas-rpc` binary builds from the accepted isolated source,
  and its `--help` entry point passes. Release/package binaries were not rebuilt
  and must be rebuilt before using schema-v4 data. No GUI, live Hugging Face,
  cross-platform, hard-process-crash, comprehensive unresolved-state restore,
  admitted relocation, or C4 importer-ownership acceptance is claimed.
- Accept RUST-I9/RUST-I10 as an incremental C3 checkpoint. Frozen critical
  source hashes: `download_store.rs`
  `51077c83c5e79ae8418b3fed80cd1663295073511a25bfec59801189a1b17193`;
  `hf/download.rs`
  `db0a897735b8375e1a00d817f10d328d3bff750de3af08296ed3f3d258ddbf88`.
  Standards revision `1609c304` has no applicable normative changes from the
  previously routed revision. Five plan contracts, scoped rustfmt, and diff
  whitespace checks pass. The isolated archive is not a registered worktree;
  no branch or commit reachability was changed. Live personal tracking data,
  private backup artifacts, unrelated drafts, and the original staged UniFFI
  change are excluded from the source commit. Full C3/Milestone 2 stay pending.

### 2026-09-04 — GUI Startup Identity Correction

- Real GUI verification after `3745a049` indexed 83 models, then failed with
  `Persisted download destination identity changed`. The two saved roots used
  device 66312; the current root uses 66310 with the same path and inode.
  The cause/time of that device-number change is unknown. The renderer
  misleadingly displayed an empty library, and closing the window accessed a
  destroyed native object; failed startup also left a backend restart timer.
- `7d979e28` separates persisted UUID identity from live physical capability
  identity. The configured root owner durably initializes the strict bounded
  marker; recovery inspection remains read-only. Missing/changed held identity
  fails closed. Copying the marker preserves logical identity, not filesystem
  authority. No runtime acceptance or migration of physical IDs remains.
  Review moved the new filesystem reads into existing owned blocking work.
- Final isolated source passed both full core configurations: 1,068 library,
  96 integration, and six doctests per configuration (1,170 passes and 11
  existing ignores). Strict supported-workspace all-feature Clippy and
  no-default core all-target Clippy passed with warnings denied. The development
  RPC backend was rebuilt from that source; no release build is claimed.
- `ee9d38fa` makes library loading/unavailability explicit in both frontend
  compositions without hiding saved rows or presenting an unknown count as
  zero. Isolated frontend tests/types/lint passed (480 tests); the complete
  working frontend, including held drafts, passed 511 tests.
  `1a36c1a4` stops failed bridge ownership and fences post-stop port allocation.
  Isolated Electron behavior tests/build/lint passed (63 tests); the working
  Electron tree passed 125 behavior tests with one existing skip. The native
  closed-window guard is retained in the uncommitted presentation draft whose
  callback introduced the failure; unrelated drafts were not folded in.
- The selected offline repair changes only two queue destination root fields,
  preserving snapshots, attempts, ordering and files. An exact backup must
  precede marker initialization and complete-store validation precedes atomic
  publication. The first operator attempt safely refused an absent nested
  payload directory before backup/marker creation; a regression reproduced it
  and the temporary operator was corrected. Six operator fixtures and strict
  lint passed. Publication returned `durable_verified`; readback exactly
  matched candidate SHA-256
  `a0885e5fde0fc5f7c68f3c8726d8677bbbec73a9d030d92a945cce244d3b1575`.
  Both records remain paused and all 60 recorded file facts are unchanged.
  Private backup, candidate, operator source and provenance are retained in
  `.download-identity-repair.0yLHd9`; no operator code is shipped.
- Real Linux/X11 Electron verification used the rebuilt development RPC binary
  and current working-tree renderer/main builds, including held UI drafts.
  The loaded search count is 83; scrolling shows partial percentages, not
  “ready to finish.” Captured renderer errors are empty, and closing through
  the application's actual close control exits successfully after bridge
  cleanup, without the destroyed-window exception or a post-cleanup restart.
  Cold-profile reconciliation still takes time; loading is now truthful.
  A native X11 warm-start capture after window visibility confirms cached rows
  are painted while loading. Early CDP captures before painting and a close
  request whose evaluation reply was destroyed are not timing/shutdown proof;
  the final harness observes target closure and successful process exit.
  An initial early capture and external-window-close attempt were not treated
  as loaded-list or shutdown acceptance. Deciding screenshots/console logs
  are retained under `.download-identity-repair.0yLHd9/verification`.
  One existing Qwen embedding reclassification destination collision remains
  a non-blocking warning (`RUST-I12`); no merge, deletion, or move was attempted.
- Source commits used isolated indexes and enabled hooks; the original staged
  UniFFI blob and unrelated working-tree changes remain intact. Standards
  revision is `1609c304`; all five plan contracts and diff checks pass. The
  temporary operator's execute permissions are removed after acceptance.
  This correction does not close full C3, C4, the
  producer/consumer migration, or cross-platform/release acceptance.

### 2026-09-05 — Reconciliation Caller Ownership Accepted Incrementally

- An actual public rebuild test held blocking duplicate cleanup, cancelled its
  requester, and reproduced a second rebuild entering before cleanup finished.
  Reconciliation now runs under the existing runtime task owner. Requester
  cancellation drops only its result receiver; the run retains exclusion until
  settlement. Required rebuilds report the new `ModelIndexRefreshInProgress`
  error when occupied, while dirty marks arriving during a run survive it.
- Public cancellation and blocking-panic/retry regressions pass. The shared HF
  client is retained through `Arc`, without cloning its lifecycle implementation.
  New API fixtures reuse the existing configured, registry-free test owner.
- Root verified an isolated candidate based on `f61025dd`: exactly eight source
  files covering reconciliation/API wiring, the public error, and exhaustive
  UniFFI error projection. Public HF Result changes, staged HF binding adapters,
  RPC contracts, generator dependencies, and desktop consumers were excluded.
  Both full core package feature configurations pass 1,179 tests with 11 existing
  ignores. Strict all-target/all-feature supported-workspace lint (excluding
  BEAM-loaded `pumas_rustler`) and no-default core all-target lint pass, as do
  scoped edition-2021 formatting and diff checks.
- This adds a public Rust error variant; exhaustive external matches must
  handle it. The candidate keeps existing RPC and native consumers compiling.
  General application Drop/drain, complete C3/C4, desktop integration, and
  RUST-I12 collision remediation remain unaccepted by this checkpoint.

### 2026-09-05 — RUST-I12 Collision Correction Admitted

- Following coordinated desktop commit `2b081fba`, real GUI startup still
  reproduces the Qwen reclassification collision. Read-only inspection proves
  distinct Q4_K_M and Q8_0 artifacts in the same repository; `cleaned_name`
  loses the source directory's repo/quant identity when forming the target.
- Selected source scope: preserve the exact artifact basename while changing
  category/family, retain display and selected-artifact metadata, and remove
  reclassification's content-based deletion. An occupied exact target refuses
  without changing either entry; no suffix guessing or automatic merging.
- The existing exists-check followed by rename cannot exclude target creation
  during the gap. The move therefore gains a small platform-owned no-replace
  primitive, called through awaited blocking work. Linux/macOS use the safe
  `rustix` rename-with-flags API; Windows uses its non-replacing native move.
  `rustix` 1.1.3 is selected for the required Linux/macOS semantics and safe
  wrapper; standard Unix rename overwrites an empty directory and the current
  nix wrapper covers only GNU Linux. Promote the already pinned resolution to
  an explicit core target dependency with filesystem support, not a new public
  feature. No unsafe domain code or weaker rename fallback is admitted.
- Verification requires naming and identical-bytes/different-repo regressions,
  atomic occupied-target refusal, both core configurations and strict lint,
  exact live payload preservation, and repeat real GUI startup without the
  warning. Linux/X11 and the current local filesystem provide runtime evidence;
  Windows/macOS target runtime evidence remains unavailable, not passed.
  This does not claim full path containment, general move crash recovery,
  broader C3/C4 completion, or cross-platform release acceptance.
- Both public reclassification failures reproduced before correction: distinct
  quants collided at a shared display-name target, and identical bytes from
  different repositories caused source deletion. The corrected reclassification
  group passes 13/13. Three real filesystem checks prove successful movement
  and refusal of an occupied empty directory or dangling symlink. Independent
  review confirms the native flags have no replacing fallback.
- Reconciliation now classifies `AlreadyExists` from the retained IO cause;
  a native-error regression reproduced the old message-parsing failure.
  Both full core feature configurations pass 1,188 tests with 11 existing
  ignores. Strict supported-workspace all-target/all-feature lint, no-default
  core all-target lint, full formatting, diff checks, and all five plan
  contracts pass. The final default RPC backend builds.
- Live Linux/X11 verification accepts the bounded correction: first startup
  reports `1/83 reclassified, 0 errors`; repeat startup reports
  `0/83 reclassified, 0 errors`. Both render 83 catalog models, two labelled
  paused activities, nine partial percentages, zero renderer errors, and clean
  window-control shutdown. Q4_K_M is now at
  `embedding/qwen3/qwen--qwen3-embedding-8b-gguf__q4_k_m`; Q8_0 remains at
  `embedding/qwen3/qwen3-embedding-8b-gguf`. Neither model was merged or deleted.
- Both full payload SHA256 values are unchanged (`3fcd3febec8b` and
  `d20ddc71e8a5` prefixes). Repo, display-name, selected-file, and quant metadata
  remain unchanged for both entries. The two paused download records retain
  store hash `a0885e5f` and all 60 tracked payload paths. Exact hashes, metadata
  backups, test logs, and GUI captures remain in the private
  `/tmp/pumas-draft-integration.tEfbTc` evidence directory. Accept RUST-I12 for
  the verified Linux environment; Windows/macOS runtime and broader release
  evidence remain unavailable rather than inferred from these host checks.

## Reports

- [RPC diagnostic disclosure evidence](reports/rpc-disclosure-evidence.md):
  Milestone 1 accepted; `RUST-A1` satisfied.
- `reports/rpc-contract-and-threat-model.md`: pending Milestone 2.
- `reports/model-index-recovery-evidence.md`: pending Milestone 3.
- `reports/rust-lifecycle-evidence.md`: pending Milestone 4.
- `reports/rust-feature-matrix.md`: pending Milestone 5.
- `reports/rust-binding-boundary.md`: pending Milestone 6.
- Final objective evidence reconciliation: pending Milestone 7.
