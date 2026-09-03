# Focused Audit: Rust Library and RPC

## Scope and Result

This pass inspected 242 Rust source files, workspace/crate manifests, tests,
RPC/IPC models, persistence, async ownership, feature gates, and UniFFI/Rustler
adapters. Result: **good local Rust hygiene, with critical diagnostics exposure
and high-priority boundary, persistence, lifecycle, and feature-contract gaps**.

`cargo metadata` and dependency-tree queries were used. The full Rust suite,
crash injection, hostile RPC clients, and host-language runtimes were not run by
this audit.

## Findings

### R-01 — RPC parameters can disclose credentials in logs

**Severity:** Critical — enforceable Security and Diagnostics violation.

`rust/crates/pumas-rpc/src/handlers/mod.rs:408-417` debug-logs the complete
method parameter object. `set_hf_token` receives the token in that object at
`rust/crates/pumas-rpc/src/handlers/models/auth.rs:7-10`. The same generic path
logs and returns raw internal errors at
`rust/crates/pumas-rpc/src/handlers/mod.rs:482-488`; error text can contain paths
and URLs (`rust/crates/pumas-core/src/error.rs:41-66,105-106,157-167`).

This reaches persistent logs in normal development/debug operation: Electron
enables backend debug at `electron/src/main.ts:499-503`, the bridge sets
`RUST_LOG=debug` and forwards tracing stderr as info
(`electron/src/python-bridge.ts:383-395,447-452`), and Electron's file transport
accepts info at `electron/src/main.ts:21-23`.

Stop logging request values, use a deny-by-default structured field allowlist,
and project typed, stable, redacted public errors.

### R-02 — LAN RPC has no caller authentication or authorization

**Severity:** High — enforceable network trust-boundary violation.

`rust/crates/pumas-rpc/src/main.rs:203-214` permits any non-loopback bind when
`--allow-lan` is present. `rust/crates/pumas-rpc/src/server.rs:170-214` adds
CORS, body-size, and concurrency middleware but no authentication layer. The
dispatcher includes credentials, deletion, migrations, path/URL opening, and
runtime control.

Either remove LAN mode from the supported contract or define authenticated
capabilities, per-operation authorization, admission/rate policy, and hostile
non-browser client tests. CORS is not authentication.

### R-03 — Durable mutation and update publication are not atomic

**Severity:** High — enforceable persistence violation.

`rust/crates/pumas-core/src/index/model_index.rs:419-469` commits a model upsert
before separately appending and publishing its event. Package-facts cache
updates follow the same pattern in
`rust/crates/pumas-core/src/index/model_index/package_facts_cache.rs:69-123`;
`rust/crates/pumas-core/src/index/model_index.rs:1046-1080` deletes rows before
recording removal events.

Commit authoritative mutation and durable event in one SQLite transaction,
publish only after commit, and prove injected failure, reopen, and replay.

### R-04 — Startup migration identity and interruption behavior are implicit

**Severity:** High — enforceable persistence migration violation.

Every normal open runs schema mutation
(`rust/crates/pumas-core/src/index/model_index.rs:271-376`). The code infers
whether particular repairs and rebuilds are needed by inspecting schema SQL,
then performs ad hoc table-rebuild sequences
(`rust/crates/pumas-core/src/index/model_index/governance.rs:191-337`). It does
not expose stable migration identity, accepted-artifact integrity, deterministic
ordering, or explicit interruption postconditions.

Define stable migration identity, integrity and ordering plus a durable record
of applied state appropriate to this store. Define transaction/recovery behavior
for each step and test all supported prior schemas plus interruption; the audit
does not prescribe a particular migration framework or version table.

### R-05 — Task and server owners discard terminal outcomes

**Severity:** High — enforceable concurrency and Rust async violation.

- `rust/crates/pumas-core/src/api/runtime_tasks.rs:22-42` removes finished
  handles without awaiting their results and aborts on shutdown without
  reporting outcomes.
- `rust/crates/pumas-rpc/src/server.rs:83-113` immediately aborts the server
  task.
- `rust/crates/pumas-core/src/ipc/server.rs:44-108` aborts accept/connection
  tasks.
- `rust/crates/pumas-core/src/api/builder.rs:72-87` maps a blocking-task join
  failure to an empty result.

Use an explicit supervisor, retain typed terminal results, close admission,
signal cancellation, drain under a bounded policy, and return
complete/incomplete/failed shutdown.

### R-06 — Core capability markers do not gate their dependency or API surface

**Severity:** High — enforceable library/dependency configuration violation.

HF, process, GPU, archive, and ONNX dependencies are unconditional in
`rust/crates/pumas-core/Cargo.toml:13-112`. The `hf-client`, `process-manager`,
and `gpu-monitor` entries at `:117-121` are empty compile-time markers, although
`process-manager` does alter builder runtime behavior. Related modules and
re-exports remain unconditional in `rust/crates/pumas-core/src/lib.rs:33-100`.
A no-default dependency tree still contained `reqwest`, `ort`, `tokenizers`,
`sysinfo`, `ctrlc`, archives, and watchers. The separate `uniffi` feature does
correctly enable its optional dependency.

Define actual consumer configurations, make optional dependencies real, gate
modules/re-exports consistently, and verify both dependency exclusion and
consumer compilation for each supported set.

### R-07 — IPC decoding and RPC response wrapping invent valid-looking defaults

**Severity:** High — enforceable IPC/contracts violation.

`rust/crates/pumas-core/src/ipc/server.rs:110-120` accepts method plus arbitrary
JSON. HTTP search/release handlers cast negative `i64` inputs to `usize`/`u64`
(`rust/crates/pumas-rpc/src/handlers/models/search.rs:7-13,32-35,73-76`;
`rust/crates/pumas-rpc/src/handlers/versions/release.rs:113-123,146-151`). Request
envelopes do not state an extra-field policy, and the HTTP handler decodes but
does not validate the `jsonrpc` version
(`rust/crates/pumas-core/src/ipc/protocol.rs:16-24`;
`rust/crates/pumas-rpc/src/handlers/mod.rs:85-94,408-490`).
`rust/crates/pumas-rpc/src/wrapper.rs:13-60,146-179` turns null or incorrectly
typed values into successful empty collections, strings, or booleans.

Decode each method into a strict DTO before dispatch, use checked conversions,
declare extra-field policy, and validate typed outbound outcomes.

### R-08 — Binding placement and support claims exceed their evidence

**Severity:** High — enforceable binding-boundary violation.

Core retains an optional UniFFI dependency/scaffolding and framework derives
(`rust/crates/pumas-core/Cargo.toml:101-122`,
`rust/crates/pumas-core/src/lib.rs:29-31`,
`rust/crates/pumas-core/src/models/model.rs:8-15`) even though the adapter does
define its exposed DTOs locally
(`rust/crates/pumas-uniffi/src/bindings.rs:299-305`). UniFFI documentation claims
Python, C#, Kotlin, Swift, Ruby, and Go support
(`rust/crates/pumas-uniffi/src/lib.rs:1-13`), but CI has neither Go generation
nor real-host runtime proof. A real C# host-smoke mechanism exists at
`scripts/check-uniffi-csharp-smoke.sh:114-168`, but CI does not schedule it.
UniFFI error conversion exposes raw paths and collapses paused into cancelled
(`rust/crates/pumas-uniffi/src/bindings.rs:88-141`).

The README also describes Rustler as a supported core binding, but
`rust/crates/pumas-rustler/src/lib.rs:268-442` exports local parsers and
constructors without calling `pumas-library`; unknown import stages are
fabricated as `Copying` at `:326-335`.

Keep framework conversion in the adapter, publish a truthful host/target
support matrix, run real-host cohort tests, and retain typed/redacted errors.

### R-09 — Default plugin startup silently substitutes an unrelated loader

**Severity:** High — enforceable resilience and degraded-outcome violation.

The RPC crate enables `inference-plugins` by default
(`rust/crates/pumas-rpc/Cargo.toml:51-57`). If the configured plugin loader fails,
`rust/crates/pumas-rpc/src/main.rs:136-152` silently substitutes an empty loader
rooted in the system temporary directory and then unwraps the fallback result.
The process can therefore look healthy with no configured plugins, or panic
while attempting a recoverable fallback.

Return a typed startup/degraded outcome tied to the configured plugin root.
Either fail startup or expose a deliberate plugin-disabled state; do not change
the authority path implicitly, and do not unwrap the recovery attempt.

## Strengths to Preserve

- Workspace `unsafe_code = "deny"` and `unsafe_op_in_unsafe_fn = "deny"`
  (`rust/Cargo.toml:15-19`).
- Credible adjacent safety reasoning for the few platform/FFI relaxations
  (`rust/crates/pumas-core/src/platform/process.rs:17-49`,
  `rust/crates/pumas-core/src/metadata/atomic.rs:117-130`).
- Loopback RPC default plus body and concurrency limits.
- Shared typed parameter decoding in several handlers
  (`rust/crates/pumas-rpc/src/handlers/shared.rs:12-22`).
- SQLite foreign keys, WAL, and distinct read-only opening
  (`rust/crates/pumas-core/src/index/model_index.rs:293-330`).
- A broad Rust verification script (`scripts/rust/check.sh:21-94`).

## Design Note

The roughly 13,000-line model-library implementation is a review hotspot, not
an automatic violation. A later deep-module review should use ownership,
contract depth, coupling, and test seams to decide whether and where to change
it.

## Next Focused Audits

1. RPC threat model, secret handling, and public error disclosure.
2. SQLite mutation/event atomicity and migration recovery.
3. Async task/shutdown postcondition inventory.
4. Core public API and Cargo feature/dependency matrix.
5. IPC/RPC DTO and outcome projection inventory.
6. UniFFI/Rustler support, adapter, and packaged-host evidence.
7. Plugin-loader startup and degraded-state contract.
