# Issues: Desktop, Release, Bindings, and Torch

**Plan:** [plan.md](plan.md)

## Current State

- The product/release owner accepted the Milestone 0 decision bundle. Its
  former unavailable issues are resolved and implementation/evidence work is
  routed to the named downstream milestones.

## Active Issues

### DRBT-I6 — Windows and macOS launcher evidence unavailable locally

- Severity: High for release promotion; no effect on bounded Linux
  implementation.
- Evidence: the current execution environment is Linux x64. Source review or
  Linux process behavior cannot prove Windows console/process-tree semantics or
  macOS runtime behavior.
- Relationship: leaves DRBT-A6 pending and blocks promotion of the accepted
  Windows x64 and macOS arm64 artifacts until required-real evidence exists.
- Owner/boundary: Milestone 4 owns the shared launcher contract and per-OS
  Adapters; Milestone 6 supplies accepted target runners and release gates.
- Disposition: implement one portable launcher contract with platform-specific
  mechanisms and record non-local target outcomes as `unavailable`, not passed.
- Verification: execute the same wrapper/platform/process integration suite on
  accepted Windows x64 and macOS arm64 runners and observe no orphaned child or
  false success.
- Revisit trigger: target CI execution or an accepted target-matrix change.

## Resolved Issues

### DRBT-I1 — Release channel, target matrix, and promotion authority unavailable

- Severity: High.
- Evidence: six public GitHub releases are all marked prerelease; current CI
  packages Linux x64, Windows x64, and macOS arm64, but only an unpackaged Linux
  launcher smoke exists. SemVer `0.x`, workflow shape, and prior publication do
  not select a channel or support promise.
- Relationship: blocks Milestone 0, `PRG-A6`, and `DRBT-A8`/`DRBT-A9`; fixes the
  required environments for `PRG-A3`, `PRG-A4`, `DRBT-A3`, `DRBT-A4`, and
  `DRBT-A6`.
- Owner/boundary: product/release owner at the release artifact-plan seam.
- Disposition: re-plan/accept-now. Decide the proposed GitHub preview channel,
  three desktop target tuples, manual promotion authority, and support/retention
  policy before encoding a release plan.
- Verification: reviewed decision followed by required-real exact-package
  evidence for every accepted tuple.
- Revisit trigger: explicit owner acceptance or a new real channel/target
  consumer.
- Resolution: accepted GitHub preview-only channel, the three current desktop
  tuples, and repository-maintainer manual promotion. Evidence remains owned by
  Milestones 2, 4, and 6.

### DRBT-I2 — Host binding disposition unavailable

- Severity: High.
- Evidence: public releases contain Python, Kotlin, Swift, Ruby, and C# bundles,
  but no real host consumer or release host suite was found. The C# smoke proves
  only a Linux debug native `version()` call and record construction. Rustler
  has no core Pumas interface or BEAM host evidence, and Go has only a source
  comment.
- Relationship: blocks Milestone 0, `PRG-A6`, and `DRBT-A7`/`DRBT-A9`.
- Owner/boundary: product owner selects the host promise; Rust owns semantic
  Adapter source and this plan owns host cohort/release projection.
- Disposition: re-plan/accept-now. Proposed removal of all binding ZIPs from
  releases, Rustler/Go removal, and either internal-experimental or removed
  UniFFI machinery.
- Verification: absence across advertised/registered/released surfaces, or a
  new accepted tuple with exact-cohort real-host evidence.
- Revisit trigger: explicit owner decision or a named real host consumer.
- Resolution: remove all host-binding releases plus UniFFI, Rustler/Elixir, and
  Go surfaces. Rust owns source-seam removal; this plan owns release/script/doc
  projections.

### DRBT-I3 — Published Rust `.crate` role unavailable

- Severity: Medium.
- Evidence: Pantograph is the real direct Rust consumer and pins an immutable
  Git revision; no consumer of the GitHub `.crate` asset or package registry was
  found.
- Relationship: blocks exact release-artifact closure in Milestone 0/6 and
  `PRG-A6`.
- Owner/boundary: product/release owner at Rust source distribution.
- Disposition: proposed removal from GitHub releases while preserving the
  producer-owned source contract and Pantograph exact-revision evidence.
- Verification: release-plan absence plus Pantograph's locked exact-revision
  consumer build/tests.
- Revisit trigger: accepted direct `.crate` or registry consumer.
- Resolution: remove the GitHub `.crate` artifact while preserving Pantograph's
  exact Git-revision source contract.

### DRBT-I4 — Torch deployment target unavailable

- Severity: High.
- Evidence: Torch has lower-bounded production requirements and source-only
  development instructions; it is not present in current desktop, Rust, or
  binding release artifacts. No supported Python/platform/device tuple or
  deployment channel exists.
- Relationship: fixes the environment for Milestone 3, `PRG-A4`, `DRBT-A5`,
  and any later `PRG-A6` release claim.
- Owner/boundary: product owner selects deployment; this plan owns the Torch
  request/work Module and evidence.
- Disposition: proposed non-shipped status until Milestone 3 proves the smallest
  accepted real stack.
- Verification: real resolved ASGI/model/device evidence for the accepted tuple,
  or absence from every release promise.
- Revisit trigger: explicit owner disposition or a real deployment consumer.
- Resolution: Torch remains non-shipped until Milestone 3 proves an accepted
  real tuple.

### DRBT-I5 — Third-party license/notice acceptance authority unavailable

- Severity: High.
- Evidence: current releases lack an accepted final dependency notice/SBOM
  closure; a local historical Debian package exposes Electron/Chromium license
  files but not a separately identifiable Pumas MIT license or reviewed final
  third-party inventory.
- Relationship: blocks `DRBT-A8`, `DRBT-A9`, and `PRG-A6` release acceptance.
- Owner/boundary: designated licensing/release authority.
- Disposition: block promotion until the owner and per-artifact obligation
  review are named.
- Verification: final extracted-byte license/notice/SBOM inspection with
  recorded reviewer authority.
- Revisit trigger: named authority and accepted artifact plan.
- Resolution: repository maintainer owns final third-party notice review and
  acceptance.
