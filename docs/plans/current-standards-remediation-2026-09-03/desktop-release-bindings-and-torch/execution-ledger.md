# Execution Ledger: Desktop, Release, Bindings, and Torch

**Plan:** [plan.md](plan.md)

## Baseline

- Code: `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`
- Standards: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`
- Audit code: `a33c8c0efa7cd8783c7deeac9e608db205290d43`
- Audit standards: `52b096ded9c53afd439a3cf0efc4cc85252da570`

## Current State

- The explicit `start` operation was accepted on 2026-09-03 and moved the plan
  from `Planned` to `Active`.
- Milestone 0 is `Accepted`; the report and two canonical matrices agree and are
  visible versioned inputs.
- Milestone 1 is `Blocked`: Rust's credential-disclosure prerequisite is
  accepted, but the canonical producer projection still contains legacy
  operations and handler-owned event shapes.
- Current slice: Milestone 4 wrapper parity, explicit platform selection, and
  bounded launcher-owned process-tree outcomes.

## Entries

### 2026-09-03 — Start Milestone 0 authority investigation

- Operation: `start`.
- Prior/current plan state: `Planned` -> `Active`.
- Development decision: `investigate`.
- Coherent slice: determine the consumer, channel, artifact, host/target,
  version, promotion, dependency, licensing, and evidence facts needed before
  release or binding automation can be valid.
- Current exact write set:
  - `docs/plans/current-standards-remediation-2026-09-03/desktop-release-bindings-and-torch/plan.md`
  - `docs/plans/current-standards-remediation-2026-09-03/desktop-release-bindings-and-torch/execution-ledger.md`
  - `docs/plans/current-standards-remediation-2026-09-03/desktop-release-bindings-and-torch/issues.md`
  - `docs/plans/current-standards-remediation-2026-09-03/desktop-release-bindings-and-torch/reports/release-and-host-contract-decision.md`
- Shared-file handoff: `scripts/release/artifact-plan.json`,
  `bindings/support-matrix.json`, package scripts/manifests, generated output,
  CI, and durable shared docs remain held for serial ownership by the program
  coordinator.

### 2026-09-03 — Milestone 0 investigation reached the decision gate

- Inventory method: current repository/configuration review, historical package
  inspection, authenticated GitHub release/API inventory, and public-code search
  for concrete consumers.
- Result: the bounded investigation stopping condition passed. Every current
  released/generated output and public consumer claim has an observed consumer
  classification or proposed conservative disposition.
- Real consumers found: independent desktop release users, Pantograph through an
  exact Cargo Git revision, and Pixapillars transitively through Pantograph.
- No real direct host consumer was found for Pumas Python, Kotlin, Swift, Ruby,
  C#, Elixir/Erlang, or Go surfaces. Torch is source-only and not currently
  shipped.
- Evidence/report:
  [release-and-host-contract-decision.md](reports/release-and-host-contract-decision.md).
- Outcome: `unavailable`. Product/release-owner acceptance is required for the
  proposed preview channel, desktop tuples, `.crate`/binding removal,
  UniFFI/Rustler/Go dispositions, Torch non-shipped state, promotion owner, and
  licensing authority.
- Program impact: `PRG-A6` is blocked directly; `PRG-A3` and `PRG-A4` cannot
  fix their complete required-real environment matrices until these decisions
  are accepted.
- No source, tooling, configuration, shared manifest, CI, package, generated
  artifact, or durable shared-document file was changed.

### 2026-09-03 — Product/release contract accepted

- Operation: explicit user `continue` after the external decision resolved the
  `Blocked` state; plan and Milestone 0 returned to `Active`.
- Accepted channel: GitHub `preview` only, with manual promotion by a repository
  maintainer after all required evidence passes.
- Accepted desktop tuples: Linux x64 AppImage/Debian, Windows x64
  NSIS/portable, and macOS arm64 DMG. Build-only evidence cannot promote them.
- Removed release roles: standalone `.crate` and all Python, Kotlin, Swift,
  Ruby, and C# binding ZIPs.
- Removed surfaces: UniFFI, Rustler/Elixir, and Go; no hypothetical binding seam
  remains without a real consumer.
- Torch disposition: non-shipped source capability until Milestone 3 proves a
  real runtime/platform/device tuple.
- Version/consumer contract: one lockstep Pumas product version; Pantograph
  consumes the Rust interface by immutable Git revision.
- Licensing authority: repository maintainer reviews and accepts final
  third-party notice evidence.
- Next write set is the two Milestone 0 canonical matrix files plus this plan's
  plan, ledger, issues, and decision report, under granted serial ownership.
- Review reconciliation: release promotion explicitly requires program security
  `PRG-A1`, governance `PRG-A7`, all other `PRG-A*` claims, and all
  `DRBT-A*` claims. The versioned top-level third-party notice is the reviewed
  authority and each desktop artifact embeds byte-identical content under a
  stable internal filename.

### 2026-09-03 — Re-plan canonical binding-matrix visibility

- Trigger: `git check-ignore -v bindings/support-matrix.json` proved that the
  intended canonical matrix was excluded by `.gitignore`'s `bindings/*` rule;
  `git status --short --untracked-files=all` showed only the release artifact
  plan.
- Classification: systemic authority/integration defect. An ignored canonical
  file cannot own or project the accepted binding disposition.
- Composition result: unchanged. The same two accepted matrices remain the
  Interfaces; no additional registry, parser, or Adapter was introduced.
- Re-plan: `.gitignore` was admitted under explicit serial ownership solely to
  add `!bindings/support-matrix.json` while retaining every generated-output
  ignore. The Milestone 0 allowed write set now records it.
- Intended oracle: Git reports both canonical matrices as untracked change-set
  inputs, `git check-ignore` reports the support matrix as not ignored, and both
  documents parse and cross-reference the removed binding release role.

### 2026-09-03 — Accept Milestone 0

- Files: `.gitignore`, `scripts/release/artifact-plan.json`,
  `bindings/support-matrix.json`, and this plan's authority/evidence artifacts.
- Cross-review: program integration review accepted the preview/manual
  promotion contract, three target/five artifact closure, seven removed host
  surfaces, zero accepted host tuples, `.crate`/binding/Torch exclusions, all
  `DRBT-A1`–`DRBT-A9` and `PRG-A1`–`PRG-A7` gates, and notice identity rule.
- Visibility oracle: `git check-ignore` reports the support matrix is not
  ignored; `git status --short --untracked-files=all` shows both matrices.
- Contract oracle: Node JSON parsing plus relationship checks prove reciprocal
  references, unique/closed target and artifact IDs, complete removed-host
  inventory, acceptance-claim inclusion, and byte-identical notice embedding.
- Supporting checks: current plan structure and scoped diff checks pass.
- Result: Milestone 0 `Accepted`. Shared ownership of `.gitignore` and both
  matrices is released; later changes require a new serial handoff.

### 2026-09-03 — Start Milestone 4 launcher outcome slice

- Operation: `continue`; plan remains `Active`, Milestone 4 moves from
  `Planned` to `Active`, and Milestone 1 is explicitly `Blocked` rather than
  projecting an incomplete producer contract.
- Development decision: `build`, after bounded inspection confirmed three
  related failures at one existing ownership seam: Bash has a wrapper-specific
  release path, unknown platforms default to Linux, and bounded commands have
  no forced-termination deadline.
- Coherent slice: make both wrappers delegate every action to the shared Node
  launcher, make the platform factory closed over accepted OS values, and make
  launcher-owned child trees reach one observed completion after graceful then
  forced termination through an OS Adapter.
- Exact initial write set:
  - `launcher.sh`
  - `launcher.ps1`
  - `scripts/launcher/actions.mjs`
  - `scripts/launcher/commands.mjs`
  - `scripts/launcher/contract.mjs`
  - `scripts/launcher/platform-service.mjs`
  - `scripts/launcher/platform-linux.mjs`
  - `scripts/launcher/platform-macos.mjs`
  - `scripts/launcher/platform-windows.mjs`
  - `scripts/launcher/actions.test.mjs`
  - `scripts/launcher/commands.test.mjs`
  - `scripts/launcher/wrappers.test.mjs`
  - this plan, ledger, issues, and
    `reports/launcher-platform-evidence.md`
- Shared-file handoff: root serialized `README.md` and
  `docs/DEVELOPMENT.md` for launcher-only updates, but their mutation is held
  until the focused Linux source contract passes. `package.json` is excluded.
- Initial oracle order: first demonstrate the wrapper fast path, unknown-to-
  Linux fallback, and unbounded ignored-termination behavior; then implement.
- Evidence boundary: Linux x64 runs locally as required-real evidence. Windows
  x64 and macOS arm64 remain `unavailable` until the same suite executes on
  accepted real target runners; no result is inferred across OS boundaries.
- Deep-module review: keep validation, timers, and one-terminal-result policy
  inside `commands.mjs`; platform modules are mechanism-only Adapters for
  process-tree termination. No parallel action registry or generic process
  framework is admitted.

### 2026-09-03 — Re-plan typed platform failure at the CLI boundary

- Trigger: the initial unsupported-platform oracle showed that
  `createPlatformService()` is invoked before `cli.mjs` enters its
  `LauncherError` boundary. Fixing only the factory would emit an unhandled
  stack rather than the declared typed diagnostic and exit code.
- Re-plan: add the already-Milestone-4-owned `scripts/launcher/cli.mjs` to the
  current exact write set and move platform composition inside the existing
  error boundary. This is not a new Adapter or registry.
- Claim impact: DRBT-A6 only. No governance claim matrix or schedule changes.

### 2026-09-03 — Re-plan shared POSIX and bounded Windows mechanisms

- Trigger: focused cross-review found identical Linux/macOS process-group
  signal implementations and an unbounded Windows `taskkill.exe` helper nested
  inside the otherwise bounded command lifecycle.
- Re-plan: add `scripts/launcher/platform-posix-process.mjs` to the Milestone 4
  and current write sets, move the identical POSIX mechanism there, and give
  the Windows helper its own execution and forced-close observation deadlines
  inside the command's force window.
- Oracle correction: replace the loaded-runner-sensitive `<900ms` assertion
  with a controlled child self-exit marker that must remain absent after forced
  termination.
- Composition result: one process policy owner (`commands.mjs`), one shared
  POSIX mechanism Adapter, and one Windows mechanism Adapter. No target-specific
  behavior is hidden by duplication and no generic process framework is added.
- Claim impact: DRBT-A6 only. Windows/macOS required-real outcomes remain
  `unavailable` until target execution.

### 2026-09-03 — Reach focused Linux M4 source/test boundary

- Source result: Bash and PowerShell are policy-free delegates with the same
  dependency exit; the platform factory is closed over `linux`, `darwin`, and
  `win32`; the CLI catches unsupported-platform construction; bounded smoke
  commands own a per-platform tree through one max/grace/force/close policy.
- Design correction: Linux and macOS now share the single
  `platform-posix-process.mjs` process-group mechanism. Windows explicitly
  reports graceful tree termination unavailable, escalates to `/t /f`, and
  bounds plus force-observes the `taskkill.exe` helper inside the outer force
  window.
- Stable diagnostics: command arguments and absolute command paths are absent
  from launcher-generated child failure messages; spawn/termination failures
  expose stable codes rather than raw dependency text.
- Outcome-marker oracle: the force test proves the controlled child's
  self-exit marker remains absent, replacing the original wall-clock margin.
- Local real evidence: the Linux process test terminates a real SIGTERM-
  resistant parent and descendant process group; the native Bash wrapper
  returns exit 0 for help and exit 2 for invalid usage.
- Focused/full oracle: `npm run test:launcher` passes 41 tests outside the
  restricted sandbox required for nested Bash execution. `bash -n launcher.sh`
  and the scoped whitespace diff check pass.
- Composed ownership oracle: a controlled child closes before a deferred
  termination helper; the public command result remains pending until the
  helper completes. An ineffective Adapter produces the typed incomplete-
  cleanup result around the declared max/grace/force boundary and well before
  the fixture's five-second self-exit, after which the test explicitly removes
  the fixture.
- Windows helper seam: controlled spawn tests cover observed exit 0, stable
  nonzero mapping, helper timeout, forced helper kill, and observed helper
  close. The outer runner joins that Adapter outcome before settling.
- Cross-target boundary: Windows helper unit evidence is green, but it is not a
  substitute for real Windows process semantics. Windows x64 and macOS arm64
  remain `unavailable`; Milestone 4 and DRBT-A6 remain `Active`/`pending`.

## Reports

| Planned report | Milestone | Status |
| --- | --- | --- |
| `reports/release-and-host-contract-decision.md` | 0 | `accepted` |
| `reports/rpc-contract-conformance.md` | 1 | `pending` |
| `reports/desktop-lifecycle-evidence.md` | 2 | `pending` |
| `reports/torch-runtime-evidence.md` | 3 | `pending` |
| `reports/launcher-platform-evidence.md` | 4 | `pending` |
| `reports/binding-host-matrix.md` | 5 | `pending` |
| `reports/release-evidence.md` | 6 | `pending` |
| `reports/final-acceptance.md` | 6 | `pending` |
