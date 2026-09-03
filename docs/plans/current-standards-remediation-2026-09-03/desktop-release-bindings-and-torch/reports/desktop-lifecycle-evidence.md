# Desktop Authority and Lifecycle Evidence

**Milestone:** 2 — Preserve Desktop Authority and Stream Lifecycle

**Status:** `active; corrected persisted-authority slice green pending review`

## Investigation Boundary

This report inventories the independent launcher-root authority and Electron
stream lifecycle Modules. It does not treat a source test as real packaged OS
evidence, and it does not redefine the still-changing Rust request, response,
error, cursor, or event contract.

## Persisted Launcher-Root Authority

At investigation start, `resolveLauncherRoot` had one production caller and
returned a bare string. Its precedence was environment/argument override, then
a persisted record, AppImage portable root, ancestor discovery, non-existing
AppImage portable default, packaged user-data default, and development root.

The accepted explicit precedence is environment before argument. Presence is
authoritative: a blank/whitespace environment value does not reveal an argument
or persisted fallback, and a missing/blank argument value does not reveal
persisted authority or discovery.

The baseline persisted reader first checked existence, then caught every read,
parse, shape, and root-validation failure as `null`. Consequently all of these
materially different states invoked discovery:

| State | Baseline observation | Required disposition |
| --- | --- | --- |
| absent | `null`; discovery/default continues | Supported initialization path |
| valid | configured existing launcher root | Supported authoritative path |
| invalid | `null`; discovery/default continues | Stable recovery-required result; no discovery |
| unavailable | `null`; discovery/default continues | Stable recovery-required result; no discovery |
| explicit argv/environment | unvalidated resolved path | Validate as an existing launcher root or reject |

The backend creates a missing working directory during startup. At baseline,
an invalid explicit override or fallback could therefore create a new root
rather than preserving the intended owner. Persistence failures also exposed
raw filesystem errors and the selected path.

Persistence uses recursive parent creation followed by direct synchronous write
to `launcher-root.json`. It does not prepare a same-directory temporary file,
write exclusively, synchronize the file, atomically replace the record,
synchronize the parent directory where supported, or prove interruption
behavior. Those mechanics require their own cross-platform contract and are
not bundled into the state-classification slice.

## Stream Lifecycle Inventory

`PythonBridge` owns five `NamedSseStreamOwner` instances: model-library,
model-download, runtime-profile, serving-status, and status-telemetry. Each
stores one listener, optional error listener, current request, buffer, cursor,
and reconnect timer. Backend process exit closes requests and retains listeners
for reconnect after restart; bridge stop clears listeners, cursors, requests,
and reconnect timers.

The end-to-end lifecycle is nevertheless incomplete:

- model-library forwarding is always started and has no renderer subscribe or
  unsubscribe owner;
- the other streams use process-global counts, not the subscribing renderer's
  identity and destruction lifecycle;
- window close resets and stops serving, runtime-profile, and telemetry, but
  omits model-download state and the always-on model-library stream;
- preload observes only serving-status subscribe rejection; other subscribe and
  every unsubscribe promise are fire-and-forget;
- start overwrites the listener and resets the cursor; duplicate or stale
  transitions have no typed outcome;
- destroying a request is synchronous from the owner perspective and terminal
  socket close is not joined before bridge shutdown reports completion;
- a late end/error callback from an older request can clear current state or
  schedule a reconnect after a newer request exists;
- partial SSE input has no explicit capacity, ordering, or drop contract; and
- malformed events are logged and discarded, which remains a Milestone 1
  producer/decoder issue rather than authority for this lifecycle slice.

## First Slice

The public `resolveLauncherRoot` Interface returns one closed discriminated
result. Resolved outcomes include the launcher root, source, and persisted-state
provenance (`not-consulted`, `absent`, or `valid`). Failures include a stable
path-free code and message, the explicit/persisted source, and `invalid` or
`unavailable` state. Main-process composition will refuse backend initialization
for a failure result.

Explicit environment and argument roots traverse the same existing-root
validator as persisted roots. Only persisted absence retains discovery/default
behavior. This keeps validation and precedence knowledge inside one Module; a
caller learns one result and never reinterprets filesystem state.

The persisted JSON policy is intentionally narrow by authority rather than by
metadata. The top-level value must be a runtime-checked record and its
`launcherRoot` must be a string that resolves through the canonical validator.
`selectedPath`, `updatedAt`, and unknown fields are non-authoritative metadata;
the resolver ignores them and never lets them select or alter the library root.
This policy is characterized with wrong-type metadata, an unknown field, and a
non-record top-level value rather than relying on a TypeScript cast.

Authoritative root shape is also runtime evidence, not path existence. A
narrow filesystem Adapter supplies `readFileSync` and `statSync`. Validation
requires the root markers to be directories, classifies `ENOENT`/`ENOTDIR` as
invalid or missing, and classifies other I/O failures as unavailable. The same
typed validator governs environment, argument, and persisted authority;
best-effort discovery is invoked only after persisted absence and cannot supply
proof for an explicit or persisted selection.

Authoritative normalization is bounded to three chooser forms: the exact
launcher root, its exact `shared-resources` directory, or its exact
`shared-resources/models` directory. It never walks arbitrary ancestors.
Ancestor walking is a different mechanism used only for discovery after the
persisted record is absent. Malformed path-domain errors
(`ERR_INVALID_ARG_VALUE`, `EINVAL`, and `ENAMETOOLONG`) are invalid alongside
`ENOENT`/`ENOTDIR`; access and I/O capability failures remain unavailable.

Startup diagnostics project only the selected source or the stable typed
recovery code/source/message. They do not log the absolute launcher root or the
raw typed recovery Error/stack. Selection persistence likewise emits stable
path-free success/failure diagnostics; its atomicity remains a later slice.

Backend initialization is converted immediately into a closed `fulfilled` or
`rejected` outcome before window creation is awaited. This preserves the
original typed failure while preventing a delayed window from routing an early
rejection through the process-level unhandled-rejection logger. After the
window boundary, normal startup emits one projected failure and quits;
release-smoke rethrows the preserved failure and never reports success. The
path-free projection is specific to typed launcher-root recovery; unexpected
backend failures retain the existing general error diagnostic.

The deletion test passes: removing this resolver Interface would push override
precedence, record classification, root-shape validation, discovery, defaults,
and recovery policy into the Electron composition root and its tests. Removing
the startup-observation seam would reopen the timing-dependent global rejection
path. The implementation adds one narrow production/test filesystem Adapter
and one backend-specific closed-task seam, not a general filesystem or task
framework.

## Evidence and Remaining Gates

The first red/green evidence uses real temporary filesystem layouts through the
public resolver, including a valid discovery decoy that never wins over invalid
or unavailable authority. It supplies incremental DRBT-A3 evidence only. Atomic
replacement, interruption safety, durability by accepted OS, explicit UI
recovery, and required-real Linux x64, Windows x64, and macOS arm64 filesystem
behavior remain pending.

Stream changes remain held until a separate exact ownership slice is admitted.
DRBT-A4 also remains dependent on the canonical Rust event projection for
decoder semantics and on required-real Electron lifecycle evidence.

### Focused red to green

| Oracle | Red observation | Green observation |
| --- | --- | --- |
| malformed persisted authority with valid discovery decoy | discovery decoy returned | `launcher_root_invalid`, persisted `invalid`, recovery required |
| unavailable persisted authority with valid discovery decoy | discovery decoy returned | `launcher_root_unavailable`, persisted `unavailable`, recovery required |
| nonexistent argument/environment override | missing path returned as resolved | canonical validation returns source-specific `launcher_root_invalid` |
| missing/flag/blank argument value and blank environment value | treated as no override; discovery returned | presence is `invalid` with persisted state `not-consulted` |
| absent and valid persisted authority | formerly returned an unclassified string | resolved result records `absent`/discovery or `valid`/persisted provenance |
| metadata and JSON shape | authority projected through an unchecked cast | runtime record check; only `launcherRoot` carries authority |
| marker is a regular file | `existsSync` accepted it as a root marker | directory type is required; explicit authority is invalid |
| deterministic validation `EIO` | valid path was returned because injected filesystem behavior had no seam | environment, argument, and persisted authority return path-free unavailable results |
| NUL-bearing explicit/persisted path | unavailable capability result | invalid path-domain result |
| arbitrary descendant beneath a valid root | climbed to and selected the ancestor for argument, environment, and persisted sources | invalid authority; only exact chooser forms normalize |
| startup/persistence diagnostics | absolute root and raw recovery/filesystem Error text | stable path-free source/code/message only |
| immediate recovery rejection while window creation is delayed | rejection had no handler until the window completed and could reach the global raw-error logger | initialization is observed at creation; no `unhandledRejection`; exact typed failure and path-free projection preserved |

Fourteen public resolver cases pass against real temporary Linux filesystem
layouts and the narrow deterministic failure Adapter. One additional public
startup-task case passes for immediate rejection with delayed window
consumption.
The Electron TypeScript build and the six-file Electron test command pass. This
is local unit/build evidence, not required-real packaged Linux, Windows, or
macOS filesystem evidence.
