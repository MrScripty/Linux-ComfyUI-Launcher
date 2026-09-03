# Governance and Verification Issues

The implementation-time findings below retain explicit evidence, ownership,
disposition, required verification, and revisit triggers.

## GOV-I01 — Silent model import and download degradation

- **Severity:** High.
- **Evidence:** EC-03, EC-04, EC-08, EC-18, EC-24, and EC-25 in the
  [error-contract disposition](reports/error-contract-gate-disposition.md).
- **Relationship:** Manual classification found real empty, stale, hidden, or
  ungrouped projections; the regex reports themselves cannot decide these
  behavior contracts.
- **Owner/boundary:** Frontend model import/download/link-health projection;
  decoded bridge outcomes remain owned by frontend Milestone 4 and its provider
  dependencies.
- **Disposition:** Transfer to the frontend plan or a focused frontend
  follow-up without broadening Governance Milestone 2.
- **Required verification:** Rejected-request tests proving explicit
  unavailable/degraded state, provenance of retained data, and safe recovery.
- **Revisit trigger:** Frontend Milestone 4 consumer inventory, or any earlier
  change to one of the named projection owners.

## GOV-I02 — Silent plugin and runtime refresh degradation

- **Severity:** Medium.
- **Evidence:** EC-02 and EC-27 through EC-30 in the
  [error-contract disposition](reports/error-contract-gate-disposition.md).
- **Relationship:** Refresh rejection retains empty, default, or stale state
  without a typed unavailable outcome. FE-I09 separately owns overlap and stale
  completion for three interval owners.
- **Owner/boundary:** Frontend plugin/runtime status projections.
- **Disposition:** Transfer to the frontend FE-I09 follow-up or a plugin UI
  projection follow-up; do not mistake an `instanceof` check for remediation.
- **Required verification:** Rejection, retained-state provenance, retry,
  supersession, and recovery tests for each retained runtime owner.
- **Revisit trigger:** FE-I09 scheduling or the next change to a named plugin
  status Module.

## GOV-I03 — User actions can fail without user-visible state

- **Severity:** Medium.
- **Evidence:** EC-03 and EC-22 in the
  [error-contract disposition](reports/error-contract-gate-disposition.md).
- **Relationship:** Native picker and installation-cancellation rejection are
  logged but not projected to the user.
- **Owner/boundary:** Frontend import interaction and the active installation
  lifecycle Module.
- **Disposition:** Transfer to frontend; cancellation belongs with its current
  Milestone 0 lifecycle repair, while picker failure may remain a focused
  follow-up.
- **Required verification:** User-observable rejection state and recovery in
  the representative interaction path.
- **Revisit trigger:** Frontend Milestone 0 acceptance or the next import-picker
  interaction change.

## GOV-I04 — Unused fallback helper erases failure distinctions

- **Severity:** Medium.
- **Evidence:** EC-31; `safeAPICall` has no production caller and its test proves
  arbitrary rejection becomes caller-supplied fallback success.
- **Relationship:** This is a real Core failure-semantics finding exposed by
  classification, but not by the checker's missing-`instanceof` oracle.
- **Owner/boundary:** Frontend renderer API adapter.
- **Disposition:** Already owned by FE-I01 and frontend Milestone 4's explicit
  fallback/pass-through deletion decision.
- **Required verification:** Consumer inventory proves no production caller;
  deleting the helper removes its test and leaves decoded typed outcomes intact.
- **Revisit trigger:** Frontend Milestone 4 consumer inventory.
