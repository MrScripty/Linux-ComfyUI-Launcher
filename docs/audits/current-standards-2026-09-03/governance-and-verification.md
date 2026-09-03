# Focused Audit: Standards Governance and Verification

## Scope and Result

This pass compared repository guidance, permanent gates, contract docs, plans,
and CI with the current standards engine. Result: **the repository's compliance
machinery is partly enforcing the standards version it was originally built
against, so a green check is not yet equivalent to current compliance**.

## Findings

### G-01 — Contributor routing points to legacy standards entrypoints

**Severity:** High — systemic standards-authority violation.

`CONTRIBUTING.md:29-40` names legacy files such as `CODING-STANDARDS.md`,
`PLAN-STANDARDS.md`, and `TESTING-STANDARDS.md` as the source of truth.
`docs/STANDARDS_ADOPTION.md:7-24` uses the same superseded ownership map.
The current standards require routing from `CORE-STANDARDS.md` through
`STANDARDS-ROUTER.md`; legacy indexes cannot override canonical owners.

Replace the local adoption map with the current route and record the standards
commit used for each future audit/refactor.

### G-02 — Count-based architecture and documentation gates are unauthorized

**Severity:** High — systemic tooling/architecture/documentation violation.

`frontend/eslint.config.js:83-100` and
`frontend/scripts/check-file-size.js:3-58` block files based on fixed line and
complexity counts and claim that this enforces modularity.
`scripts/dev/check-readme-coverage.sh:29-77` requires a README in every scanned
directory. The current standards explicitly say counts are diagnostic only and
directory/file counts do not establish a documentation obligation.

The README gate currently reports two missing directories; adding arbitrary
files would make the repository more compliant with the obsolete gate, not the
current standard. Retire or replace these checks with ownership-, contract-,
change-impact-, and test-seam evidence.

### G-03 — Error-policy verification has no coherent authority or schedule

**Severity:** Medium — enforceable tooling/verification violation.

`frontend/package.json:17-20` declares `check:errors` as part of precommit, but
`.pre-commit-config.yaml:6-34` does not run it and CI omits it
(`.github/workflows/build.yml:147-162`). It currently reports 31 locations.
`frontend/scripts/check-error-handling.js:38-59` uses source regexes whose
`instanceof` proximity and `throw new Error` checks do not prove the desired
typed boundary outcome.

Define the actual error-contract claims, inspect the 31 sites against those
claims, and keep only mechanisms with an authoritative or independent oracle
and marginal deciding value.

### G-04 — Human contract docs have drifted from executable behavior

**Severity:** Medium — enforceable documentation/contracts violation.

- `docs/contracts/desktop-rpc-methods.md:14-16` says all per-method schemas are
  deferred, while the registry now contains some schemas.
- Its method table at `:27-28` lists `preview_model_mapping`,
  `sync_with_resolutions`, and shortcut methods absent from the executable
  registry.
- `README.md:51-56` says legacy `PumasApi` can attach to an existing owner;
  `docs/architecture/SYSTEM_ARCHITECTURE.md:51-59` and implementation say
  constructors are owner-only.
- `docs/contracts/release-artifacts.md:27-78` promises release outputs and SBOMs
  not produced by `.github/workflows/build.yml:433-526`.

Select the executable owner for each contract and generate or verify human
projections from it. Do not preserve compatibility claims without a named
consumer and evidence.

### G-05 — Plan lifecycle authority is hard to determine

**Severity:** Medium — enforceable planning/documentation violation for active
plans, not a request to rewrite terminal history.

`docs/plans/README.md:1-25` lists many artifacts without a clear
active/accepted/completed/superseded classification. It also references the
standards from a stale filesystem path (`:51`) and claims there are no ADRs
(`:54-57`), despite `docs/adr/0001-onnx-runtime-provider-model.md`.
`docs/plans/backend-owned-status-resource-telemetry/plan.md:750-1171` contains a
long sequence of planned, completed, failed, and later completed status blocks,
making current authority expensive to recover.

Classify plan lifecycle in the index. Compact active plans to objective,
acceptance status, current phase, blockers, and exactly one next slice.
Historical terminal plans need no mechanical template rewrite.

### G-06 — Cross-platform and user-workflow evidence is thinner than support claims

**Severity:** Medium — enforceable verification-evidence gap.

CI provides valuable cross-platform builds, but real runtime evidence is
concentrated on a Linux Xvfb startup smoke. It does not exercise a substantive
user workflow, host-language loads, migration interruption, hostile RPC input,
or Windows/macOS runtime behavior. A build on a target and a startup smoke prove
only their named boundaries.

Create a claim-to-evidence matrix before adding more tests. Highest-value
missing scenarios are secret-free RPC diagnostics, unauthorized LAN calls,
invalid request/response/event contracts, interrupted persistence/migration,
default/library-only artifacts, host binding load/call, and keyboard/focus
workflows in Electron.

## Permanent Gate Disposition

| Mechanism | Current disposition | Reason |
| --- | --- | --- |
| TypeScript typecheck | Retain | Strong authoritative language proof |
| Type-aware ESLint | Retain, review local policy | Strong static oracle; some local rules need current ownership |
| Frontend Vitest | Retain | Valuable component/unit evidence |
| Fixed file-size/complexity gates | Retire or advisory | Counts do not decide architecture |
| README coverage gate | Retire | Directory presence does not decide documentation need |
| Regex error checker | Reauthorize or replace | Red, inconsistently scheduled, weak semantic oracle |
| Rust check script | Retain, map claims | Broad useful evidence; document each subcheck's boundary |
| Electron validation/tests | Retain and extend | Good initial allowlist/request evidence |
| Launcher tests | Retain and extend | Good shared behavior evidence; deadlines/wrapper parity missing |
| Release startup smoke | Retain as startup-only | Must not substitute for user workflow or platform-runtime evidence |
| Version/dependency ownership checks | Retain | Useful manifest consistency claims |

## Recommended Governance Slice

Before broad code refactoring:

1. update `CONTRIBUTING.md` and `docs/STANDARDS_ADOPTION.md` to the current
   standards router;
2. inventory every permanent gate with its named claim, proof boundary, oracle,
   overlap, marginal value, and schedule;
3. retire the fixed count and blanket README gates;
4. classify active versus historical plans;
5. reconcile public contract docs with their executable owners; and
6. convert the top boundary findings into separate implementation-ready plans.

This slice changes governance and documentation authority. It should not be
combined with the credential, RPC schema, persistence, or release fixes in one
undifferentiated refactor.
