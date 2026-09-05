# Desktop Authority and Lifecycle Evidence

**Milestone:** 2 — Preserve Desktop Authority and Stream Lifecycle

**Status:** `active; sandbox-compatible renderer-recovery producer green pending coordinated review`

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

### Atomic-persistence contract

The accepted next slice retains one writer through Electron's instance lock and
keeps `persistLauncherRootOverride` as the public Interface. It prepares one
unpredictable adjacent temporary file with exclusive `wx` creation and mode
`0600`, writes and synchronizes the complete serialized record, closes the temp,
renames it over the authority, then synchronizes and closes an already-open
parent directory. The [official Node filesystem API](https://nodejs.org/api/fs.html)
provides these mechanisms but does not by itself prove their semantics on every
supported filesystem or OS.

Pre-publication failure preserves the prior authority and cleans only the temp
owned by that attempt. A rename failure has unknown replacement visibility.
After rename, parent sync or close failure reports the complete new destination
as published with unavailable durability; it never rolls back, deletes, retries,
or reports relaunch success. Cleanup failure is secondary bounded state and does
not replace the primary phase/cause.

The admitted evidence uses stage-named injected failures plus local Linux
reopen/byte/mode checks and real subprocess termination immediately before and
after publication. It can prove namespace old-or-new behavior only for that
local filesystem. Power-loss guarantees, every Linux filesystem, Windows and
macOS directory flushing, remote/removable filesystems, orphan cleanup, and
concurrent-writer policy remain explicitly open.

The implemented Module preserves the existing success result and main-process
consumer. Its typed failure has one path-free message and correlated stage,
authority, and cleanup states:

| Failure phase | Authority state | Cleanup/consumer result |
| --- | --- | --- |
| parent or temp preparation through temp close | `unchanged` | Close owned descriptors and unlink only the owned unpublished temp; cleanup incompleteness is secondary |
| rename | `replacement-visibility-unknown` | Never unlink the destination, copy, retry, or select a fallback |
| parent sync or close after rename | `published-durability-unavailable` | Complete new destination remains visible; caller receives failure and does not relaunch as success |

The default Adapter executes ten ordered stages: ensure directory, open parent,
create the typed unpredictable temporary name, open that temp exclusively, write
temp, sync temp, close temp, rename, sync parent, and close parent. Temporary-
name generation is inside the typed operation boundary. Its successful local
Linux result has mode `0600`, parses as the exact returned config, leaves no
operation-owned temp, and resolves through the public reader as the new authority.

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
Fifteen persistence cases cover every selected injected phase, primary-cause
preservation when cleanup throws a legal non-Error value, successful default
publication, and real subprocess `SIGKILL` barriers immediately before and after
rename. The focused module therefore has 30 passing cases total. The main
consumer remains fail-closed and path-free, but explicit renderer recovery for
replacement-unknown and published-durability-unavailable remains open.
The Electron TypeScript build and the six-file Electron test command pass. This
is local unit/build evidence, not required-real packaged Linux, Windows, or
macOS filesystem evidence.

## Renderer Recovery Handoff

The detailed launcher-root Module remains the semantic owner of resolution and
persistence facts. The admitted pure recovery Module projects only two closed,
path-free values across Electron IPC:

- startup is `initializing`, `ready`, or `recovery-required`; ready carries only
  `selectionAction: select-library|correct-launch-input`, while recovery also
  distinguishes invalid versus unavailable and persisted versus explicit
  authority; and
- selection is `cancelled`, `restarting`, `not-selectable/correct-launch-input`,
  or `recovery-required`. Recovery distinguishes invalid selection, chooser
  unavailable, persistence unavailable, and restart unavailable, correlated
  with `unchanged`, the two indeterminate publication states, or `published`.

Persisted startup failure can expose the native chooser. Environment or argument
authority cannot, whether it is invalid, unavailable, or already resolved:
writing persisted state would not outrank that explicit input on the next
launch. Normal desktop startup keeps the window available only for a typed
launcher-root recovery outcome. Release smoke and unrelated backend failure
remain terminal.

Main owns the current startup state and one selection-attempt lifecycle. Active
callers share one attempt. Proven-unchanged outcomes permit a later explicit
retry; indeterminate publication, published recovery, and restarting outcomes
lock and replay without more work. The restart Adapter invokes
`app.relaunch()` synchronously after durable persistence and delays only quit;
it cannot return restarting after that request or timer scheduling throws.
Preload owns runtime decoding at the native IPC boundary. Neither exposes
launcher paths, selected values, persistence stages, cleanup detail, raw causes,
or source chains. Failure never automatically retries or rolls back.

The frontend consumer remains a separately serialized boundary. It will use one
provider/hook for startup query and terminal presentation while the main owner
retains authoritative chooser deduplication and retry exclusion. One recovery
view will be shared by default and library-only entrypoints.
Until that consumer is accepted, this producer tranche cannot integrate because
keeping the Electron window open would otherwise leave recovery unreachable.

### Producer red to green

| Oracle | Red observation | Green observation |
| --- | --- | --- |
| recovery Module public seam | module/export absent | closed startup and selection projection/decoder Interface |
| malformed/extra/correlated IPC values | raw preload forwarding accepted any value | exact-shape decoders reject unknown keys, option bags, and invalid source/action or reason/state pairs |
| renderer data minimization | chooser response included selected and launcher paths; persistence failures collapsed to text | projections contain only the declared status, reason, source/action, and authority state |
| explicit startup authority | resolved env/argv collapsed to chooser-capable ready; persisted selection would lose on relaunch | ready carries path-free selection action; only non-explicit ready and persisted select-library recovery allow selection |
| unavailable chooser | no-window reused persistence failure; native dialog rejection escaped as raw IPC rejection | no-window and dialog rejection return chooser-unavailable with authority unchanged and no private detail |
| overlapping chooser calls | each call opened a dialog and could publish/relaunch independently | concurrent callers share exactly one owned Promise, dialog, persistence operation, restart request, and terminal outcome |
| retry boundary | chooser stayed open after ambiguity, publication, and restart scheduling | unchanged outcomes reset to idle; ambiguous, published, and restarting outcomes lock and replay without new work |
| relaunch request | `restarting` returned before detached callback invoked `app.relaunch()` | relaunch is invoked synchronously after publication; request/scheduling failure returns locked restart-unavailable/published |
| normal versus smoke lifecycle | no public disposition oracle | typed root rejection becomes normal desktop recovery but remains the exact fatal error in release smoke; unrelated failure is fatal |
| preload boundary | launcher-root IPC values were not decoded | startup and selection invocations pass through their closed runtime decoders |

The first red run failed the new recovery module import and preload decoder
source oracle while five of seven Electron test files passed. The lifecycle red
then failed on the absent initialization-outcome classifier. The first 11-check
green boundary was rejected when review exposed collapsed resolved-explicit
policy and absent selection-attempt ownership. The corrective red could not
import the selected handler Interface. The corrected focused run passes 18
checks across recovery and preload, including handler-level overlap, re-entry,
terminal-lock, dialog, publication, restart-request, and main composition cases.
The full Electron command builds TypeScript and passes all seven test files.

This evidence does not claim a reachable recovery experience: the frontend
consumer is deliberately held for separate ownership and atomic integration.
It also does not satisfy required-real packaged target behavior, stream
lifecycle acceptance, or final DRBT-A3 acceptance.

## Sandboxed Preload Composition

The initial producer boundary passed TypeScript and Node tests but failed in the
real production composition. The BrowserWindow enables sandbox and context
isolation and disables Node integration. The emitted CommonJS preload attempted
to load `./launcher-root-recovery`; Electron 39.8.6's sandboxed preload loader
could not resolve that local module, so the bridge never reached the renderer.

The corrected ownership keeps producer DTOs, authority-to-action projection,
and chooser lifecycle in `launcher-root-recovery.ts`. Runtime inbound decoding
belongs only to the preload trust boundary. Its producer import is type-only,
so the canonical build emits a standalone preload whose only runtime module
load is Electron. No security preference or browser fallback changed.

| Oracle | Red observation | Green observation |
| --- | --- | --- |
| canonical build output | `dist/preload.js` contained `require('./launcher-root-recovery')` | exact runtime allowlist contains only Electron |
| compiled decoder | Node tests exercised a separately importable helper | compiled standalone preload exposes the API in a constrained VM and accepts/rejects the complete closed value set |
| real sandboxed preload | unable-to-load-preload/module-not-found; `window.electronAPI` absent | actual built preload loads with exact production preferences and exposes the bridge |
| renderer-main-world values | no bridge, so no decoder path | nine startup and nine selection values round-trip exactly; three startup and six selection malformed/extra values reject with stable messages |
| lifecycle | bridge failure could be missed by uptime smoke | oracle fails on preload/load/render/unresponsive error, mismatch, rejection, nonzero exit, or 12-second child bound; an owned process group is observed gone before unique temp cleanup; successful real case completed in 551 ms |

The deciding command used pinned Electron 39.8.6 as uid 1000 on Linux with
`DISPLAY=:0` and no `--no-sandbox`. It passed all nine tests in the focused
preload module. The full command rebuilt Electron and passed all 77 tests with
the real oracle enabled and no skip; lint, `tsc --noEmit`, plan structure, and
scoped diff checks also pass. This is representative real Electron evidence for
sandboxed preload loading and decoding, not proof for packaged Linux bytes or
required-real Windows/macOS targets. The frontend consumer and composed
recovery UI remain separate and block integration.

## First-Visible-Frame Decision Evidence

The first-visible-frame problem crosses three owners: main owns native window
visibility and the current document generation, preload owns the sandboxed
bridge, and the frontend provider owns the terminal semantic presentation. A
renderer DOM commit is not a compositor presentation. Real Electron disproved
the attempted two-animation-frame approximation: the renderer acknowledged
terminal content while the newest hidden presentation remained Checking.

The frozen correctness implementation uses an in-frame challenge instead of a
timing claim. After browser readiness, current-document readiness, and a
correlated terminal semantic acknowledgement, main begins one presentation
subscription. It injects a fixed opaque 3×3 grayscale checker with no input or
private data, ignores frames until a scale-safe nine-point NativeImage match,
removes the exact inserted CSS, and ignores queued checker frames until a
strictly later valid image lacks the marker. Authority, document, and terminal
state are re-correlated before the one show. Navigation, fallback, preload/load
failure, deadline, disposal, and close cancel the subscription and exact marker;
cleanup or image-decoding failure reaches the existing path-free native fatal
terminal rather than guessing visibility.

### Correctness and performance observations

| Evidence | Result | Claim boundary |
| --- | --- | --- |
| Focused owner suite | 43/43 pass: pre-insert and queued-frame exclusion, marker/clear order, stale document, fresh fallback challenge, late insert cleanup, duplicate/reentrant callbacks, authority re-correlation, all Adapter failures, and no post-fatal continuation | deterministic Module/Adapter lifecycle only |
| Permanent real sandbox case | actual Electron 39 hidden BrowserWindow observes canonical marker then later marker-free image and one show; compiled preload still loads only Electron | representative Linux loader/presentation mechanics, not packaged targets |
| Default real ready matrix | immediate commit→marker/clear/show 1216.6/1279.5/1283.0 ms; delayed samples end at 1049.5, 1247.9, and 1049.4 ms | causal correctness green; latency is observational and materially conflicts with immediate startup |
| Library-only real ready matrix | immediate show 663.9 ms; delayed samples show at 363.5, 438.2, and 396.3 ms | same boundary; no threshold is used as correctness proof |

Every completed ready case made one authority request, captured hidden Checking,
showed only after a content frame, observed no later Checking, emitted no
console/preload failure, and exited zero. Those results do not accept the
latency or substitute for the held recovery, missing-preload, fatal, reload, and
complete producer/preload/frontend conformance matrix.

### Rejected faster mechanisms

1. **Synchronous terminal boot plus ready-to-show.** A throwaway prototype used
   one versioned, bounded, path-free `additionalArguments` envelope, exact
   sandboxed-preload decoding, synchronous provider seeding, and one initial
   React `flushSync`. It constructed no Checking state and made the semantic
   acknowledgement precede ready-to-show. Nevertheless, the only delivered
   pre-show NativeImage was a uniform opaque fuchsia 1280×874 surface with
   repeated interior BGRA samples `[138,0,255]`, not terminal app content.
   Layout acknowledgement plus ready-to-show is not sufficient presentation
   proof and is rejected.
2. **Hidden `capturePage` challenge.** With the same boot construction,
   acknowledgement preceded ready-to-show. Marker insertion completed at
   719.8 ms, but `capturePage(undefined,{stayHidden:true})` completed at
   2019.4 ms and did not contain the marker. Because the first required capture
   was already non-causal and slower than the subscription approach, the probe
   stopped without retries, sleeps, invalidation guesses, or product changes.

The product implementation and composed frontend remain frozen at this
decision gate. The causal marker path is correctness evidence, not an accepted
product tradeoff. Direct boot and hidden capture must not be reintroduced as
support assumptions; a later re-plan must either accept the measured cost
explicitly or provide a different deciding real-frame oracle.
