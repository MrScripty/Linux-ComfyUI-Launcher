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
- Current next slice: validate the Rust producer/projection handoff before
  starting Milestone 1 source changes.

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
