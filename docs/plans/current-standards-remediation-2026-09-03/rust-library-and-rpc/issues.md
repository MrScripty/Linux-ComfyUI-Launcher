# Rust Library and RPC Issues

## Open Decision Dependencies

| ID | Relationship and evidence | Owner | Current disposition | Required verification | Revisit trigger |
| --- | --- | --- | --- | --- | --- |
| `RUST-I11` | **High, resolved:** Real GUI startup indexed 83 models then rejected two persisted physical root identities after the device number changed. | Core destination/persistence owner | **Accepted 2026-09-04:** durable logical UUID identity is separate from live physical capability identity. The two exact paused records were updated offline after backup; no runtime fallback or payload mutation. | Marker initialization/reopen/refusal fixtures; six operator fixtures; dual full-package tests and strict lint; exact two-field live diff and 60-path preservation; real GUI list, console, scroll and shutdown. | Identity marker mutation, unsupported filesystem durability, or a new independently deployed persistence contract |
| `RUST-I12` | **Medium, resolved on Linux:** Display-name destination naming lost distinct Q4_K_M/Q8_0 identity. | Model-library classification/naming owner | **Accepted 2026-09-05:** retain artifact basename and display metadata, remove reclassification dedupe, refuse occupied targets with a non-replacing move. Q4_K_M relocated without merging/deleting either model. | Naming/no-loss/native-refusal regressions, dual core suites and strict lint pass. Exact payload hashes unchanged; two GUI startups show 83 models and zero reclassification/renderer errors. Windows/macOS runtime evidence remains unavailable. | A missing model, changed move authority, or any proposed automatic merge/deletion |
| `RUST-I10` | **High, resolved for the selected cutover:** Old paused records lacked durable predecessor ownership; a real HTTP regression exposed successor execution before head release after restart. | Rust lifecycle/store and migration owners | **Accepted 2026-09-04:** user selected a one-time library update and removal of runtime compatibility. Schema v4 rejects old formats and unowned ordinary rows. Two distinct local records received new independent custody after exact backup; durable publication and unchanged-file proof passed. The temporary converter is retired. | Three converter fixtures, fresh current-store reopen fixture, live exact candidate/backup and 60-path preservation proof; dual full-package tests; strict core/workspace lint; independent review. Exact evidence is retained in the ledger. | Ambiguous destination history, unsupported live record shape, unresolved publication, or expansion beyond the selected one-time cutover |
| `RUST-I9` | **High, resolved incrementally for C3:** A queued successor accepted pause but could not settle while a paused destination head retained its claim. | Rust download lifecycle owner | **Accepted 2026-09-04:** generation wake settles queued pause without destination effects, retains exact admission/FIFO position, and restores marker selection only after its turn. Current-only restart and automatic-finalization regressions pass. | Public queued pause, same-client and fresh-client resume, implicit-selection marker provenance, cancellation/failure custody, and complete-follower blocking; dual full-package tests and strict lint. | Changing queued-pause, restart ordering, or completion-handoff authority |
| `RUST-I8` | **Critical, historical checkpoint retired:** Migration previously moved before admission, ignored refusal, and guessed rollback. Its owned replacement was verified before the user dropped legacy support. | Rust lifecycle/store and model-migration owners | RUST-I10 removes that obsolete partial-relocation implementation and its tests. Current partial moves report unsupported without filesystem effects; completed-model migration remains supported. | Historical exact-hash evidence remains in the ledger. Current no-effect partial reporting and unchanged complete-model migration pass; admitted relocation and broader C3 remain open. | A newly selected admitted-relocation contract |
| `RUST-I1` | R-02: unauthenticated `--allow-lan` materially changes the desktop RPC trust boundary. | Product/program owner; Milestone 2 consumes | **Accepted 2026-09-03:** remove `--allow-lan` and reject every non-loopback `--host`; desktop RPC is loopback-only. | RUST-A3 real negative host/bind and loopback-positive system evidence | Revisit only with a new authenticated remote-access product contract |
| `RUST-I2` | R-09: configured plugin-loader failure currently substitutes a temporary root and may panic. | Product/program owner; Milestone 4 consumes | **Accepted 2026-09-03:** compiled-out plugins report explicit disabled/unavailable; compiled-in configured loader/root failure fails startup. Per-plugin discovery/load failures may be typed unavailable only when the public contract supports it. | RUST-A6 invalid-root, compiled-out capability, and startup/shutdown system evidence | Re-plan if source shows no clean configured-vs-discovery failure seam |
| `RUST-I3` | R-08: Rustler declares a core dependency but exports local conversions; host support is unproved. | Desktop/platform plan; Milestone 6 consumes | **Accepted 2026-09-03:** remove UniFFI, Rustler/Elixir, and false Go surfaces; retain the public Rust library used by Pantograph's exact-Git dependency. | RUST-A8 Rust removal proof; host/release proof stays downstream | Re-plan if a real supported host consumer is supplied before removal |

## Resolved Implementation-Time Issues

### RUST-I4 — Valid mixed-quant model rejected by catalog projection

- **Severity:** High.
- **Evidence:** `ModelLibrary` deliberately projects `download_incomplete=false`
  when a complete GGUF remains displayable beside the selected artifact's
  `.part`, while the first frozen RPC projection rejected the subordinate
  `download_has_part_files` or missing-file evidence.
- **Relationship:** RUST-A2 requires a truthful closed result; rejection made a
  valid installed model fail the whole `get_models` response.
- **Owner/boundary:** Core model-library download projection is readiness
  authority; desktop RPC owns the closed catalog projection.
- **Disposition:** Resolved in Milestone 2I. False `download_incomplete`
  projects complete even with subordinate partial-selected-artifact evidence;
  it is never coerced to partial. Complete is a unit wire state and never
  carries recovery, even when subordinate selected-artifact recovery metadata
  remains present.
- **Required verification:** Existing real Q5-complete/Q4-part core fixture plus
  direct core-record-to-`CatalogModel` red-to-green regression and both RPC
  feature-mode suites.
- **Revisit trigger:** Any change to core download readiness semantics or to
  the closed catalog artifact-state variants.

### RUST-I5 — Reconciliation hierarchy and bounded identity could lose retry ownership

- **Severity:** High.
- **Evidence:** A model dirtied during a full run remained dirty after success,
  but the next full admission inspected only full-scope state; conversely,
  full dirtiness was invisible to a previously clean model scope. Numeric
  revision/run IDs used `saturating_add`, allowing stale/current identities to
  collide at `u64::MAX`.
- **Relationship:** RUST-A2 requires refresh/read success to represent observed
  state rather than stranded stale work.
- **Owner/boundary:** Core `ReconciliationCoordinator` admission and run-token
  lifecycle.
- **Disposition:** Resolved in Milestone 2I. Per-scope dirty bits are cleared on
  admission, post-admission marks survive, and full dirty/failure/drop
  propagates to known model scopes. Opaque allocation identity replaces
  bounded counters; only the matching identity may finish or abandon a run.
- **Required verification:** Deterministic full-to-model and model-to-full
  dirty races, failure/drop retry, dirty-during-success, scope exclusion, and
  stale-token replacement tests.
- **Revisit trigger:** Any scheduler hierarchy, cooldown, or run-token
  lifecycle change.

### RUST-I6 — Recovery identity admitted invalid repositories and host-dependent paths

- **Severity:** High.
- **Evidence:** Two nonempty repo segments accepted `owner/..`, forbidden
  punctuation/repetition, and `.git`; Linux `Path::components` accepted Windows
  drive, UNC, and backslash traversal strings as relative paths.
- **Relationship:** RUST-A2 requires validated recovery identity; invalid data
  must not enable an action that predictably fails or escapes its target seam.
- **Owner/boundary:** Desktop RPC `CatalogRecoveryIdentity` projection.
- **Disposition:** Resolved in Milestone 2I with private serializable smart
  constructors: official Hugging Face repo grammar narrowed to exact
  `owner/name`, plus a platform-neutral repository-relative path grammar with
  total and per-component bounds.
- **Required verification:** Official-invalid repo matrix; POSIX and Windows
  traversal/root/drive/UNC, reserved-device, invalid-character, and overlong
  component negatives; nested valid path; default and no-default RPC suites.
- **Revisit trigger:** Hugging Face changes its public repo-ID grammar, recovery
  gains a different upstream, or selected-artifact storage semantics change.

### RUST-I7 — Recovery action trusts a caller-selected existing directory

- **Severity:** Critical.
- **Evidence:** Desktop `recover_download` and `resume_partial_download` accept
  `repo_id` plus `dest_dir`; the handler/core canonicalize only that the path is
  an existing directory. The action can therefore combine producer-shaped
  catalog identity with an unrelated caller-selected directory.
- **Relationship:** RUST-A2 requires the producer's closed action contract to
  preserve catalog identity and filesystem authority rather than trusting a
  renderer-provided locator.
- **Owner/boundary:** Core PumasApi/model-library resolution owns model ID to
  indexed directory, recovery snapshot, and metadata; desktop RPC owns the
  model-ID-plus-opaque-token adapter.
- **Disposition:** Open. Reopened after the first Milestone 2I/M2H correction failed
  independent exact-set, atomic-admission, filesystem-authority-through-use,
  caller-cancellation, and persistence-revocation review. The replacement
  PRG-I19 fix failed its subsequent independent lifecycle/durability review.
  Revised Slice A is re-frozen separately: canonical atomic publication holds
  one pre-existing parent authority and reports pre-publication failure,
  visibility-unknown, published-durability-unknown, or durable. The strict v2
  download store persists two-phase unknown/durable revocation disposition and
  serializes every read-modify-write across constructors/processes with an OS
  file lock; it never derives durable revocation from absence. Confirmation
  ambiguity cannot succeed its initiating call, while its already-durable
  unknown predecessor remains fail-closed. Linux local/ext-family runtime
  evidence is green, macOS is unverified, and Windows/non-Unix durable
  publication is closed target-unavailable. Network/distributed filesystems
  remain unsupported and must be rejected or proved before integration. It
  now has an accepted, fourth-corrected Slice B lifecycle owner. The
  actual download ID is reserved across strict revocation,
  generation-matched worker handoff, terminal projection, and cancellation.
  Each installed generation owns its gated outer task, mutating filesystem and
  persistence work, callbacks admitted in this slice, and rejected-task
  retirement until terminal Join observation. Sticky state-local provenance
  prevents a later cancel from erasing unresolved Worker, transition,
  projection, finalizer, nested, or cleanup failure. Terminal state and
  recovery-capability release follow filesystem cleanup, strict persistence
  cleanup, and the final owned drain. Attach requires a real non-finished
  capability-backed owner; reconciliation reserves a terminal projector and
  rechecks exact owner/state after durable publication. Ordinary admission
  Slice C is active in exactly nine source files. The configured model root is
  opened once and injected as a held capability; one two-phase store-v3
  admission owner persists the complete request, physical destination identity,
  domain, FIFO ordinal, and predecessor/release proof before an ID, active
  snapshot, or effect becomes public. The same held root plus validated relative
  target owns reservations, marker/part effects, cleanup, restore, and
  relocation; raw/canonical path strings and a physical async mutex cannot
  substitute. Store v3 migrates legacy/v1/v2 state, separates clean Pending
  cleanup from sticky Pending/Verified failure, preserves recovery tombstones,
  and rejects stale ordinary writers. Unknown publication parks custody.
  Pause belongs to the exact started generation and uses owner-visible wakeups;
  snapshots are linearly ordered and guard-free; terminal release is exact-
  generation, durable, published, drained, and panic-rescuable. Initial reds
  proved ownerless ordinary setup/resume state and missing durable quarantine
  owners; later review added exact-attempt clean removal, FIFO crash restore,
  alias/replacement confinement, restoration custody, and callback-order gates.
  General client-drop draining remains Milestone 4/RUST-A6, and Slices D-E
  remain pending. Desktop
  `list_interrupted_downloads` and `recover_download` were deleted. The retained
  action accepts exact camel-case `{modelId,recoveryToken}` only. One core
  snapshot owner issues and verifies a domain/version-framed BLAKE3 stale-state
  fingerprint over the model ID, canonical managed directory, repository,
  artifact, quant, and canonical sorted-unique file set. The core freshly
  resolves and reindexes the model, rejects stale or mismatched context, and
  derives every filesystem/repository locator itself. The token is a
  collision-resistant precondition, not caller authentication. In-library
  indexed imports with download provenance remain eligible; outside-root,
  aliased/symlinked, unavailable, or unproven rows remain displayable without a
  recovery action. The mutation requires the complete token-bound remote file
  set and adds no auxiliary file. One downloads-lock owner atomically selects
  exact-context attach/resume or new admission. A non-serialized state-local
  `cap_std::fs::Dir` capability roots every recovery filesystem operation at
  the held managed library root; recovery tasks do not use ordinary ambient
  status persistence or application callbacks, while their narrow terminal
  cleanup authority preserves the revocation tombstone. Start-gated admission
  commits the capability and a real task
  owner synchronously; caller-independent cancellation observes the worker
  before clearing state authority; strict serialized tombstones prevent stale
  persistence writers or generic resume from restoring ambient authority; and
  relocation is refused without mutation. Capability-relative operations
  prevent the tested replacement races from mutating outside that held root,
  but do not claim to pin the original model-directory inode against a
  same-user replacement within the same managed root.
  This disposition is limited to the desktop RPC Interface. Public core/local-
  IPC and UniFFI `recover_download(repo_id, dest_dir)` remain transitional
  ambient-authority surfaces pending the accepted zero-`Legacy` and Milestone
  6 removals. The selected Electron/frontend ticket consumer migration is
  accepted at `2b081fba`; its former repo/path mismatch is no longer a hold.
- **Required verification:** exact request/token grammar; token stability under
  file reorder/duplication and refusal after semantic change; managed-root,
  symlink, alias, missing-path, and provenance cases; existing nested and part
  symlinks plus verification/use replacement with no outside mutation; exact
  remote set with no auxiliary files; atomic unrelated-context refusal; state
  capability retention/clearance, no recovery status persistence/application
  callbacks, terminal cleanup with tombstone retention, restore,
  drop, relocation, pause/complete and resume/cancel races; exact tracked-
  context match/mismatch; malformed result algebra; real loopback stale
  refusal, tracked resume/attach, indexed untracked recovery, and method-not-
  found for both deleted methods; checked remote-size overflow refusal before
  task, state, marker, or target mutation; pre/post-commit caller cancellation,
  registered-owner attach, strict-revocation failure, stale-writer/restart
  denial, same-name root replacement after held-handle acquisition, and closed
  public error mapping through real loopback.
- **Revisit trigger:** A supported caller requires recovery of a non-indexed or
  outside-root import, the catalog identity contract changes, or the token is
  proposed as an authorization credential.

New issues must include severity, exact evidence, relationship to an
objective, canonical owner, disposition, adequate verification, and a concrete
revisit trigger.
