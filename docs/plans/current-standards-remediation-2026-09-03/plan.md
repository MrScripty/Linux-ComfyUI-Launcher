# Plan: Current Standards Remediation Program

**Plan status:** `Planned`

**Current phase:** Focused plans are reconciled and await an explicit `start` operation.

**Next slice:** On explicit `start`, execute the critical disclosure slice in [Rust library and RPC](rust-library-and-rpc/plan.md), Milestone 1; transition that focused plan through its own explicit `start` before source edits.

**Acceptance status:** `pending`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

**Reports:** Program evidence is indexed in the [ledger](execution-ledger.md#reports).

**Audit source:** [Current standards audit](../../audits/current-standards-2026-09-03/README.md)

## Objective

Remediate every finding in the 2026-09-03 standards audit through four focused
owners, so Pumas preserves authoritative model state, rejects invalid or
unauthorized cross-process input, completes asynchronous work truthfully,
presents state accessibly, and ships only configurations and artifacts backed
by the evidence their support claims require.

This plan owns program sequence, cross-plan handoffs, and objective-level
acceptance. It does not duplicate the implementation decisions or write sets
owned by the focused plans.

## Baseline

- Audit code baseline: `a33c8c0efa7cd8783c7deeac9e608db205290d43`.
- Planning code baseline: `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`.
- Audit standards baseline: `52b096ded9c53afd439a3cf0efc4cc85252da570`.
- Planning standards baseline: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`.
- Documentation cleanup `d84e2b35` is an accepted prerequisite and is not
  repeated by this program.
- No source implementation or objective acceptance evidence is created by
  authoring these plans.

## Objective Acceptance

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| PRG-A1 | A hostile or malformed caller cannot disclose credentials/internal locators or invoke protected remote RPC operations, while authorized supported operations retain typed results. | `system` | `representative` (real debug RPC process and isolated network clients) | `automated` | `pending` | Rust RUST-A1/RUST-A3 and platform DRBT-A2 |
| PRG-A2 | Requests, responses, errors, and events traverse Rust, Electron, preload, and renderer through one producer-owned contract; invalid, unsupported, unavailable, and failed outcomes never become valid-looking defaults. | `system` | `required-real` (built RPC and Electron process path) | `automated` | `pending` | Rust RUST-A2, platform DRBT-A1/DRBT-A2, frontend FE-A1 |
| PRG-A3 | Interrupted model mutations/events, supported schema migrations, and launcher-root updates recover without missing durable history or silently selecting another library authority. | `system` | `required-real` (real SQLite and every accepted desktop filesystem/OS target) | `either` | `pending` | Rust RUST-A4/RUST-A5 and platform DRBT-A3 |
| PRG-A4 | Rust, Electron, frontend, launcher, and Torch work owners observe admission, supersession, cancellation, failure, deadlines, and bounded shutdown without detached work, false completion, or starvation of control traffic. | `system` | `required-real` (real runtimes, accepted OS targets, and resolved Torch stack) | `automated` | `pending` | Rust RUST-A6, platform DRBT-A4/DRBT-A5/DRBT-A6, frontend FE-A3 |
| PRG-A5 | Cached model state, recovery, dialogs, popups, progress, motion preference, and both renderer modes behave truthfully and accessibly through representative built-renderer workflows. | `user-workflow` | `representative` (built renderer in supported Electron/Chromium runtime) | `automated` | `pending` | Frontend FE-A2/FE-A4/FE-A5/FE-A6 |
| PRG-A6 | Every accepted feature configuration, host binding tuple, desktop target, and release artifact is supported by an explicit consumer matrix and matching real-target/cohort/final-byte evidence. | `release-artifact` | `required-real` (every accepted target, host/runtime, and assembly environment) | `either` | `pending` | Rust RUST-A7/RUST-A8 and platform DRBT-A7/DRBT-A8/DRBT-A9 |
| PRG-A7 | Contributor guidance and every retained permanent gate follow the current standards route and state their exact claim, oracle, schedule, overlap, and blocking authority. | `contract` | `representative` (repository and Linux pinned toolchain) | `either` | `pending` | Governance GOV-A1 through GOV-A5 |

## Scope

### In Scope

- All CS-01 through CS-15 findings and their focused-audit expansions.
- The producer/consumer contracts, persistence, lifecycle, user workflows,
  configuration matrices, bindings, release evidence, launchers, documentation,
  and governance needed to close those findings.
- Bounded design investigations admitted by the focused plans where missing
  product, compatibility, environment, or support facts can change a
  high-consequence implementation decision.
- Cross-plan sequencing and objective-level integration/release evidence.

### Out Of Scope

- New inference providers or unrelated product features.
- Decomposition justified only by file size, complexity counts, or repository
  shape.
- Expanding LAN, platform, host-language, OpenAI-compatibility, or release
  promises without an accepted consumer and adequate evidence.
- Signing, notarization, registry publication, or GitHub release publication
  unless an accepted release contract later brings it into scope.
- General accessibility certification beyond the audited workflows.
- Rewriting historical audit evidence to describe post-audit code.

## Focused Plan Ownership

| Plan | Canonical ownership | Principal audit findings |
| --- | --- | --- |
| [Governance and verification](governance-and-verification/plan.md) | Standards routing, count/error gate disposition, permanent-gate claims and schedules | G-01–G-06; CS-10, CS-13, CS-14 and G-06 evidence routing |
| [Rust library and RPC](rust-library-and-rpc/plan.md) | Rust/server RPC and local IPC contracts, public errors/redaction, SQLite state/events/migrations, Rust lifecycle/features, Rust binding placement, plugin startup | R-01–R-09; CS-01–CS-03, Rust portions of CS-04/CS-07/CS-08, CS-11 |
| [Desktop, release, bindings, and Torch](desktop-release-bindings-and-torch/plan.md) | Generated Electron projections/decoding, desktop authority/lifecycle, Torch, launcher, host cohorts, release artifact/dependency evidence | P-01–P-10; desktop portions of CS-02/CS-04/CS-06/CS-07, CS-12, CS-15 |
| [Frontend and UI](frontend-and-ui/plan.md) | Renderer consumption, cached model provenance, installation lifecycle, interaction Modules, status/motion, renderer variants | F-01–F-08; CS-05, CS-09, renderer portions of CS-02/CS-04/CS-08 |

Shared findings close only when every named producer and consumer claim passes;
one focused plan cannot accept another owner's behavior by agreement alone.

## Constraints And Assumptions

### Constraints

- Security containment precedes broader refactoring. The Critical credential
  disclosure slice may not wait for the complete RPC redesign.
- Each implementation invocation names one canonical focused `plan.md` and an
  explicit `start`, `continue`, or `verify` operation.
- The focused plan owns its source write set and lifecycle. This program plan
  records only handoffs and aggregate state.
- Shared manifests, CI, package scripts, generated artifacts, schemas, and
  current documentation are serial integration-owner writes.
- Required-real evidence cannot be replaced by compilation, fakes, generated
  freshness, startup smoke, or another plan's local tests.
- Unsupported capabilities are removed or made explicitly unavailable rather
  than retained behind empty/default behavior.

### Assumptions

- The product owner will decide LAN support and plugin-loader criticality when
  the Rust plan reaches those decision gates.
- Platform Milestone 0 can establish the actual release consumers, channels,
  target matrix, binding host matrix, and evidence/legal owners without first
  changing release automation.
- The focused plans' bounded populations cover the audited systemic families;
  each expands only when a newly discovered site shares the same authority or
  consumer promise.
- Existing useful strict typing, Electron isolation, Rust unsafe defaults,
  launcher structure, and focused tests are preserved unless direct evidence
  invalidates them.

## Binding Decisions

| Decision | Owner | Evidence | Supersedes |
| --- | --- | --- | --- |
| Critical RPC diagnostic disclosure is the first implementation slice. | Program plan | CS-01/R-01 severity and independent reversibility | Broad governance-first recommendation in the audit |
| Rust/server owns desktop RPC semantics and public error/redaction; Electron generated code is an Adapter/projection, and the renderer consumes only decoded outcomes. | Rust, platform, and frontend plans in that order | CS-01/CS-02 and cross-plan review | Hand-maintained, asserted, and fallback contract copies |
| SQLite index state/event/migration recovery and launcher-root authority remain separate Modules because their stores, consumers, and lifecycles differ. | Rust and platform plans | CS-03/CS-12 | One generic persistence framework |
| Feature, host, target, channel, and artifact support is selected from real consumers before build/release machinery is admitted. | Rust feature milestone and platform Milestone 0 | CS-06–CS-08 and Release/Binding standards | Incumbent CI/generator output as support authority |
| Governance removes weak proxy gates before later plans add claim-directed permanent evidence to shared schedules. | Governance plan | CS-10/CS-13 | Repairing legacy gate machinery by convention |
| Historical audit text remains fixed to its baseline; plans and current guides own forward implementation authority. | Documentation/program owner | Audit baseline and documentation workflow | Editing evidence into current instructions |

## Evidence And Oracle Plan

| Claim | Domain | Deciding oracle | Independent authority | Unsupported domain | Intended negative failure |
| --- | --- | --- | --- | --- | --- |
| PRG-A1 | Security/diagnostics | Captured real-process responses and diagnostics plus hostile-client outcomes | Safe public-error and accepted exposure contracts | Arbitrary future log sinks | Sentinel secret/path appears, or unauthorized operation succeeds |
| PRG-A2 | Cross-process semantics | Real Rust-to-Electron-to-renderer scenario plus closed negative contract corpus | Producer contract and independently observed consumer result | Domain correctness unrelated to transport | Malformed value reaches presentation or becomes empty/default success |
| PRG-A3 | Durable authority | Controlled interruption, cold reopen, and authoritative row/event/root comparison | Accepted store/root formats and recovery policy | Hardware failures outside declared filesystem contract | Missing event, duplicate effect, guessed migration, or silent root switch |
| PRG-A4 | Lifecycle | Owners expose and tests observe every applicable terminal state at real runtime seams | Accepted state machines, deadlines, and external responsiveness | Unclaimed hardware/provider behavior | Detached work, stale completion, starvation, hang, or false shutdown success |
| PRG-A5 | User workflows | Built-renderer keyboard/accessibility/state/mode observations | Browser accessibility tree, controlled backend outcomes, and build-mode configuration | General certification or packaged contents | Cached state appears fresh, focus/status/motion contract fails, or wrong mode UI appears |
| PRG-A6 | Shipped support | Real target/host execution and exact extracted final artifact inspection | Accepted consumer/support matrices and final resolved bytes | Unadvertised tuple/channel | Mismatch, missing/extra file, incomplete provenance/notices, or absent target evidence |
| PRG-A7 | Governance | Executable config cross-review and affected retained command results | Current standards and named gate claims | Product behavior owned by focused plans | Count/regex proxy or unmapped scheduled gate remains |

## Systemic Finding Audit

- Invariant family and canonical owner: diagnostic safety/RPC semantics,
  durable state, lifecycle, renderer truth/accessibility, support/release
  evidence, and governance each have the focused owner named above.
- Bounded authority, representation, and reachable consumer population: the
  four focused plans enumerate their Rust, Electron, frontend, Torch, launcher,
  binding, release, documentation, and tooling populations. The program owns
  only cross-plan handoffs and combined acceptance paths.
- Expansion facts: add a population only for a new semantic owner, reachable
  consumer, persisted/public promise, supported tuple, or material risk in the
  same invariant family.
- Consumer dispositions: migrate, already-safe, delete, explicit unsupported/
  unavailable, or named follow-up owner; no unclassified consumer can close a
  systemic claim.
- Deletion, consolidation, smaller-Interface, stronger-proof, and evidence-
  replacement alternatives: each focused plan prefers deletion of unsupported
  paths/proxy tests, one producer contract, existing deep Modules, and direct
  scenario evidence before new registries, Adapters, or frameworks.
- Evidence-backed stopping condition: all CS-01 through CS-15 mappings have
  accepted focused claims and every program acceptance path has its stated
  objective-level evidence.
- Repaired-composition comparison: runtime Modules stay independently owned;
  contract and release knowledge propagates through small Interfaces and
  generated Adapters rather than synchronized hand copies.

## Simplicity And Ownership Review

**Applicability:** `applicable`

- Independent concepts and dimensions: security, wire semantics, durable
  authority, async lifecycle, renderer interaction, support configuration,
  release evidence, and governance change independently.
- State, identity, value, time, policy, and mechanism: focused plans keep these
  roles separate within their Modules; this plan owns only sequence, handoff
  identity, finding disposition, and aggregate acceptance.
- Caller and composition-root knowledge: implementers select one focused plan
  and learn only its next slice; the program integration owner learns the
  dependency graph and acceptance claims, not every implementation mechanism.
- Representative change paths and forced owners: an RPC field flows producer
  to generated Adapter to renderer consumer; a release-target change flows
  support matrix to build/host evidence to final assembly; local changes do not
  force unrelated plans.
- Stable Interfaces versus hidden knowledge: accepted DTO/error revisions,
  decoded outcomes, persisted-state results, lifecycle results, support
  matrices, and focused-plan evidence are stable Interfaces; SQL, generator,
  transport, focus, queue, and packaging mechanics remain hidden.
- Independent evolution, testing, failure, and replacement: each focused plan
  can verify and fail its Module locally, while program claims add only the
  cross-process, user-workflow, or release path that local evidence cannot
  prove.
- Necessary complexity and containment: four plans match four independently
  owned audit domains; no program-level runtime Module, schema, test runner, or
  release registry is added.
- Deletion and cumulative machinery result: deleting this coordination plan
  would scatter ordering, shared-write ownership, and objective acceptance
  across four plans; deleting a focused plan would redistribute its detailed
  contract and evidence knowledge into this plan, so both levels earn their
  distinct Interfaces without duplicating implementation.

## Milestones

Each milestone delegates source changes to the exact write sets and gates in
the linked focused plans. This plan and its ledger/issues are the only
additional program-level write set.

### Milestone 1: Contain Critical RPC Disclosure

**Goal:** Prevent credentials and internal locators from reaching backend or
public diagnostics before broad contract refactoring.

**Allowed write set:** Rust plan Milestone 1's exact write set, plus this plan,
ledger, and issues.

**Tasks:**

- [ ] Explicitly start the Rust focused plan and execute only its Milestone 1.
- [ ] Require real debug-process sentinel evidence and safe typed public errors.
- [ ] Update program acceptance state without starting later Rust work
  implicitly.

**Acceptance gate:** Rust RUST-A1; PRG-A1 remains pending until exposure
evidence also closes.

**Status:** `Planned`

### Milestone 2: Establish Governance and Support Authority

**Goal:** Remove invalid gate authority and decide actual release/host support
before adding permanent evidence or packaging machinery.

**Allowed write set:** Governance Milestones 1–3 and platform Milestone 0 exact
write sets, plus this plan, ledger, and issues.

**Tasks:**

- [ ] Complete governance count/error gate dispositions and evidence inventory.
- [ ] Complete the bounded release/host contract investigation.
- [ ] Obtain product decisions for LAN support and plugin startup before their
  dependent Rust milestones.
- [ ] Serialize shared package/CI/documentation changes and record handoffs.

**Acceptance gate:** Governance GOV-A1 through GOV-A4 and accepted platform
Milestone 0 matrices; unresolved tuples are typed and block/remove only their
own claims.

**Status:** `Planned`

### Milestone 3: Deepen the RPC Contract Path

**Goal:** Carry one producer-owned request/response/error/event contract through
Rust, Electron, preload, and renderer without default invention.

**Allowed write set:** Rust Milestone 2, then platform Milestone 1, then
frontend Milestone 4 exact write sets, plus this plan, ledger, and issues.

**Tasks:**

- [ ] Accept the Rust DTO/error/exposure contract before generating consumers.
- [ ] Generate and enforce Electron projections and observable invalid outcomes.
- [ ] Migrate renderer consumers and cached model projection to decoded results.
- [ ] Run the real cross-process and immediate/degraded model-list paths.

**Acceptance gate:** PRG-A1, PRG-A2, and the cache portion of PRG-A5.

**Status:** `Planned`

### Milestone 4: Restore Durable and Async Authority

**Goal:** Make storage/root recovery and each runtime's accepted work lifecycle
atomic, current, bounded, and observable.

**Allowed write set:** Rust Milestones 3–4, platform Milestones 2–4, and
frontend Milestone 0 exact write sets, plus this plan, ledger, and issues.

**Tasks:**

- [ ] Accept SQLite mutation/event and migration recovery.
- [ ] Accept launcher-root atomicity and explicit corrupt/unavailable outcomes.
- [ ] Accept Rust, Electron, Torch, launcher, and installation-progress
  admission/cancellation/shutdown outcomes in their required environments.
- [ ] Run combined cold-reopen and runtime lifecycle paths where ownership
  crosses focused plans.

**Acceptance gate:** PRG-A3 and PRG-A4.

**Status:** `Planned`

### Milestone 5: Accept Renderer Interaction and Variants

**Goal:** Prove truthful cached state and the audited keyboard, focus, status,
motion, and default/library-only behavior in a representative renderer.

**Allowed write set:** Frontend Milestones 1–3 and 5–6 exact write sets, plus
this plan, ledger, and issues.

**Tasks:**

- [ ] Admit a representative renderer harness only if it supplies deciding
  value beyond current tests/smoke.
- [ ] Migrate audited modal/popup consumers through deep interaction Modules.
- [ ] Implement progress/status and reduced-motion semantics.
- [ ] Exercise both renderer modes through real entry points.

**Acceptance gate:** PRG-A5 and frontend FE-A1 through FE-A7.

**Status:** `Planned`

### Milestone 6: Accept Configurations, Bindings, and Release Artifacts

**Goal:** Make supported Rust/build variants, binding cohorts, and final release
bytes agree with accepted consumer and evidence matrices.

**Allowed write set:** Rust Milestones 5–7 and platform Milestones 5–6 exact
write sets, plus this plan, ledger, and issues.

**Tasks:**

- [ ] Make Rust feature/dependency/public-interface configurations real.
- [ ] Remove binding framework leakage and unsupported host claims.
- [ ] Provision pinned generators and test exact host/native cohorts.
- [ ] Assemble exact target artifacts with final-byte version, dependency,
  SBOM/provenance, checksum, license/notice, and extracted-content proof.

**Acceptance gate:** PRG-A6 plus all required-real target/host results.

**Status:** `Planned`

### Milestone 7: Program Acceptance

**Goal:** Reconcile all focused outcomes and run only the objective-level paths
not already proved by adequate focused evidence.

**Allowed write set:** This plan, ledger, issues, focused plan lifecycle/evidence
links, and their already-declared final documentation/report write sets.

**Tasks:**

- [ ] Verify all non-deferred focused milestones are Accepted or Superseded.
- [ ] Re-run cross-plan path claims in their declared environments.
- [ ] Record blocked required-real evidence without substituting weaker checks.
- [ ] Reconcile current architecture, development, security, release, binding,
  frontend, Rust, Electron, Torch, launcher, and plugin documentation.
- [ ] Close or explicitly disposition every issue and acceptance row.

**Acceptance gate:** PRG-A1 through PRG-A7 are satisfied with linked evidence.

**Status:** `Planned`

## Blockers

- `none` for the next critical disclosure slice.
- Rust LAN exposure (`RUST-I1`) requires a product decision before Rust
  Milestone 2 exposure edits.
- Plugin-loader criticality (`RUST-I2`) requires a product decision before the
  plugin lifecycle part of Rust Milestone 4.
- Host/target/channel and Rustler decisions require platform Milestone 0 before
  feature, binding, and release acceptance.
- Any required legal interpretation remains blocked until its designated owner
  accepts the evidence.

## Re-Plan Triggers

- A focused plan changes its canonical owner, objective, shared write set, or
  acceptance environment.
- The first disclosure repair cannot establish the error/redaction Interface
  needed by the broader RPC contract.
- A product decision retains a capability without an authorization, lifecycle,
  dependency, support, or evidence owner.
- Another consumer expands a systemic population or contradicts the current
  producer/consumer handoff.
- Required-real infrastructure is unavailable for an accepted support claim.
- A focused replacement introduces pass-through Modules, hypothetical Seams,
  duplicate semantic registries, or cumulative machinery beyond its admitted
  composed design.
- A lower-fidelity result was being used to close a higher-fidelity program
  claim.

## Final Acceptance

- Acceptance status: `pending`
- Deferred follow-ups: `none`; any explicit deferment added during execution
  requires an owner, reason, consequence, and revisit trigger and cannot satisfy
  an affected acceptance claim.
- Final status: `Planned`
