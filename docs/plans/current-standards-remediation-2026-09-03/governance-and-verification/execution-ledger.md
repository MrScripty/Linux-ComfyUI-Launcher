# Governance and Verification Execution Ledger

## Baseline

- Plan created from code `d84e2b3520ce3da3f39cc3df953301fa9d6d3d50`.
- Current standards: `7bf74bb5a8cb0ffccaff3ec86550051f900fb4bb`.
- Audit evidence remains fixed to its recorded older baselines.

## Accepted Prerequisite

| Result | Evidence |
| --- | --- |
| Current documentation routing, ownership, and plan lifecycle cleanup | `d84e2b35` |

## Slice Log

### 2026-09-03 — Plan admission and Milestone 1 inventory

- Operation: `start`.
- Lifecycle: plan and Milestone 1 transitioned from `Planned` to `Active`.
- Development decision: `investigate` only for the bounded, read-only inventory
  needed to resolve the exact count-gate consumer population. The critical RPC
  disclosure repair remains the program's first source-code slice.
- Current write set: this focused plan, ledger, and issues file only.
- Source, package, hook, CI, and shared-document edits remain unstarted pending
  critical RPC closure and serialized shared-file ownership.
- Re-plan trigger: `frontend/README.md:85` is a current guide that advertises
  both `check:size` and `check:errors`, but it was absent from the applicable
  milestone write sets. Removing either command without changing this guide
  would leave current documentation stale.
- Disposition: the integration owner serialized `frontend/README.md` to this
  plan for Milestones 1 and 2, limited to keeping those command descriptions
  consistent with their accepted dispositions. Both allowed write sets now
  include the guide; its content remains unchanged until the source gate opens.
- Inventory stopping condition met. The bounded mechanism population is:
  - ESLint rules `max-lines`, `max-lines-per-function`, and `complexity`, plus
    the test-file `max-lines` override in `frontend/eslint.config.js`;
  - `frontend/scripts/check-file-size.js` and its empty
    `frontend/scripts/file-size-baseline.json`;
  - `check:size` and the `precommit` command composition in
    `frontend/package.json`;
  - the frontend CI `Check file-size ratchet` step; and
  - current command descriptions in `docs/DEVELOPMENT.md` and
    `frontend/README.md`.
- No `.pre-commit-config.yaml` hook invokes `check:size`; no other live
  non-historical consumer was found.
- Baseline evidence before removal: `check:size`, `check:types`, and all 441
  frontend tests passed; ESLint also passed with the count rules enabled.
- Next development decision: `implement` once the program source gate opens.
  The change is reversible, its exact consumers are bounded, and no remaining
  uncertainty could change the removal decision.
- Lifecycle correction: after the inventory stopped, the required next source
  slice could not begin until the Rust plan's critical RPC disclosure claim
  `RUST-A1` is accepted. The focused plan and Milestone 1 therefore
  transitioned from `Active` to `Blocked`; the conditional next slice remains
  the bounded Milestone 1 removal.
- External-state change: the Rust owner and integration owner confirmed
  `RUST-A1` accepted with default/no-default debug-process sentinel evidence
  and the full affected Rust verification suite. The program source gate was
  released, so this plan and Milestone 1 resumed as `Active` and began the
  already bounded removal slice.

### 2026-09-03 — Milestone 1 accepted: retire count-based blocking gates

- Removed ESLint's blocking line-count and cyclomatic-count rules, the file-size
  checker and empty baseline, the `check:size` package command/composition, and
  its CI step.
- Updated both current development guides so neither advertises the deleted
  command. Historical audit and plan evidence remains unchanged.
- `rg` found no live non-historical references to `check:size`, its files, or
  the removed ESLint rule keys.
- `node --check frontend/eslint.config.js` and JSON parsing of
  `frontend/package.json` passed.
- PyYAML parsed `.github/workflows/build.yml` successfully; provider-level
  Actionlint remains represented by the unchanged CI `lint-workflows` job.
- Frontend ESLint and TypeScript checks passed.
- Frontend Vitest passed: 102 files and 441 tests.
- Focused `git diff --check` passed.
- Acceptance: GOV-A2 is satisfied and Milestone 1 is `Accepted`.
- Next slice: Milestone 2 error-contract candidate classification and mechanism
  disposition.

### 2026-09-03 — Milestone 2 classification complete

- `npm run -w frontend check:errors` produced 31 reports: one generic-Error
  report and 30 catch reports.
- The [disposition report](reports/error-contract-gate-disposition.md) records
  a reachable failure, contract owner/existing proof, and disposition for every
  report and all three intended checker invariants.
- Recommended mechanism disposition: remove `check:errors` and its package
  registration without replacement. Existing AST lint retains the direct
  console/generic-Error claims; strict types, type-aware lint, and focused
  behavior tests remain the selected error-handling evidence.
- Four groups of real frontend behavior gaps discovered by manual review were
  recorded in `issues.md` and handed to the frontend owner. The regex is not an
  oracle for those defects and is not retained to proxy them.
- Source/tooling mutation remains held until the serialized classification
  review accepts or rejects the recommendation.

### 2026-09-03 — Milestone 2 accepted: remove the regex error checker

- The integration and program owners reviewed and accepted the classification
  report's remove/no-replacement disposition.
- Deleted `frontend/scripts/check-error-handling.js`, removed `check:errors`
  from `frontend/package.json` and its `precommit` composition, and removed the
  obsolete current-guide descriptions.
- Preserved ESLint `no-console` and the generic-Error AST restriction, strict
  TypeScript, type-aware lint, and focused behavior ownership without claiming
  that any one is a universal error-contract proof.
- A live non-plan/audit search found no remaining checker command, filename, or
  checker-only `noqa` markers.
- `frontend/package.json` parsed successfully and focused `git diff --check`
  passed.
- The post-change frontend `precommit` composition passed ESLint and strict
  TypeScript. Frontend Vitest passed 102 files and 446 tests; after a concurrent
  frontend fixture-only lint repair, its seven focused files and 33 tests also
  passed.
- The focused plan structure checker passed.
- Acceptance: GOV-A3 is satisfied and Milestone 2 is `Accepted`.
- Next slice: Milestone 3 claim-to-evidence inventory and registration review.

### 2026-09-03 — Milestone 3 inventory and re-plan

- The retained hook/package/CI population was inventoried from the three
  package manifests, hook configuration and implementation, CI workflow,
  aggregate Rust check, and current contributor/development guidance.
- Re-plan trigger: the registered hardcoded-color hook implementation was
  outside the planned write set. Its line regex and substring exceptions do
  not decide semantic-theme use, it returns success on read/decode failure,
  and its failure output links a deleted guide. Deregistration without deletion
  would leave unowned machinery.
- The integration and program owners approved adding only
  `.pre-commit-hooks/check_hardcoded_colors.py` to Milestone 3 and deleting it
  without replacement.
- Reviewed reconciliation dispositions: remove the unowned 1 MB file-size
  hook, unregistered frontend `precommit` aggregate, Electron double compile,
  redundant post-test Electron build, and redundant non-Linux frontend
  lint/type-only matrix work; keep platform-specific Electron tests/packages.
- Exact shared ownership remains serialized to Governance for the declared
  Milestone 3 write set.

### 2026-09-03 — Milestone 3 accepted: publish the verification inventory

- Deleted the hardcoded-color regex hook/file and the unowned 1 MB added-file
  hook without replacement. The retained optional hooks now state bounded local
  claims rather than repository acceptance.
- Removed the unregistered frontend `precommit` composition and Electron's
  redundant `prebuild` compile.
- Consolidated deterministic frontend lint/type/test/build on Linux, retained
  target-specific Rust and Electron test/package evidence, ran Electron lint
  once on Linux, and removed the second unchanged Electron compile after tests.
- `docs/DEVELOPMENT.md` now records every retained semantic gate's claim,
  oracle, overlap/marginal value, blocking authority, environment, and schedule.
  Setup/cache/artifact transport is explicitly not independent acceptance.
- `CONTRIBUTING.md` links to that owner and no longer advertises deleted gates.
- Pending contract, system, user-workflow, required-real, and release-artifact
  evidence is linked to the Rust, frontend, or platform plan that owns it.
- Three package manifests and both YAML files parsed; hook shell syntax and
  isolated valid/invalid commit-subject fixtures passed; obsolete references
  were absent; focused `git diff --check` passed.
- Integration-owner configuration/documentation cross-review found the hook,
  manifests, workflow, contributor guide, and evidence inventory internally
  consistent.
- Acceptance: GOV-A4 is satisfied and Milestone 3 is `Accepted`.
- Shared M3 ownership is released. Source work is `Implemented`; Milestone 4
  begins objective verification, transitioning the plan to `Verifying`.

### 2026-09-03 — Milestone 4 accepted: governance acceptance

- Root ownership/release-version checks and 25 launcher tests passed; both
  launcher CLI help and the Rust aggregate help returned successfully.
- Frontend ESLint, strict TypeScript, default/library-only builds, and 102
  Vitest files with 448 tests passed.
- Electron ESLint and validation passed. Its retained `test` command performed
  the one owned TypeScript build and passed all five Node test files.
- Torch Ruff lint/format and 13 unit tests passed.
- The first two default-sandbox Rust aggregates reached 848 passing library
  tests before local-IPC permission denial caused two independent failures and
  nine poisoned-lock cascades. Both roots passed exactly, and the Rust owner
  passed the full workspace plus serial/default/128-thread and multi-process
  stress checks without reproducing a repository defect.
- The decisive isolated `scripts/rust/check.sh` run with required local-IPC
  permission exited zero. Formatting, all-target/all-feature checking, Clippy
  with warnings denied, complete workspace tests, doctests, and no-default
  compilation passed.
- Actionlint 1.7.8 passed the edited workflow after its archive matched the
  official checksum. Both YAML files and all three manifests parsed; the
  commit-message hook passed shell syntax plus positive/negative fixtures.
- No live non-historical consumer remains for any removed mechanism or
  duplicate registration. Focused diff and plan-structure checks passed.
- [Final governance evidence](reports/final-governance-evidence.md) reconciles
  GOV-A1 through GOV-A5 and preserves the unsupported system, user-workflow,
  non-Linux, binding-host, installer, and release-artifact claims under their
  focused owners.
- Acceptance: GOV-A5 and Milestone 4 are `Accepted`; the focused plan is
  `Accepted` with all objective claims satisfied and no next slice.

## Reports

- Error-contract gate disposition:
  [reports/error-contract-gate-disposition.md](reports/error-contract-gate-disposition.md).
- Final governance evidence:
  [reports/final-governance-evidence.md](reports/final-governance-evidence.md).
