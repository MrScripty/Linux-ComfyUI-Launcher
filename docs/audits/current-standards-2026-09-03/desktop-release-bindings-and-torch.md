# Focused Audit: Desktop, Release, Bindings, and Torch

## Scope and Result

This pass reviewed Electron, launchers, build/release automation, Torch sidecar,
bindings, dependency/SBOM/licensing evidence, and cross-platform claims. Result:
**useful isolation and build foundations, but the shipped-contract evidence is
not yet authoritative or reproducible**.

No package installation, GUI, release build, model inference, cross-platform
runtime, or host binding suite was executed.

## Findings

### P-01 — Most desktop RPC methods lack operation-specific validation

**Severity:** High — enforceable IPC/security violation.

`electron/src/rpc-method-registry.ts:1-9` defaults request and response schemas
to `deferred`. Only 51 of 152 registered methods have request schemas and 19 are
declared empty. Imports, downloads, token changes, deletion, migration, and
quantization include deferred operations. Without a method schema,
`electron/src/ipc-validation.ts:50` accepts a generic record; its
`unknown-record` type at `:164` checks only object shape.

Generate closed per-operation decoders and negative producer/consumer tests.
Treat the counts as inventory evidence, not design authority.

### P-02 — Responses and streamed events are not contract-decoded

**Severity:** High — enforceable generated-contract and TypeScript-boundary
violation.

`electron/src/python-bridge.ts:746-760` parses JSON and asserts `RPCResponse`
without validating the envelope, correlation ID, or result. SSE handling at
`:66-102` logs and discards malformed events. `electron/src/preload.ts:132`
accepts arrays and nested payloads without decoding their elements.

Use the same canonical contract owner for requests, responses, and events, and
surface invalid-contract outcomes instead of dropping them.

### P-03 — Release automation and the release artifact contract disagree

**Severity:** High — enforceable release/reproducibility violation.

`docs/contracts/release-artifacts.md:27-78` requires standalone RPC artifacts,
separate native archives, generated binding packages, three SBOMs, and final
checksums. `.github/workflows/build.yml:65-82` ignores missing required Rust/RPC
outputs, while `:433-526` assembles a different set and generates no SBOMs. It
derives a tag version without comparing it to manifests and marks every `v0.*`
release prerelease.

Define one exact machine-readable artifact manifest, fail on missing or
unexpected files, verify tag/version equality, generate final-byte SBOMs and
checksums, and make release-channel policy explicit.

### P-04 — Binding publication claims exceed host evidence

**Severity:** High — enforceable language-binding/release violation.

`README.md:172-177` claims Python, C#, Kotlin, Swift, Ruby, and Elixir support.
`.github/workflows/build.yml:272-325,481-490` generates and packages five UniFFI
languages but runs no host runtime suites. `scripts/check-uniffi-csharp-smoke.sh:68-74`
text-checks async signatures, while
`bindings/csharp/Pumas.NativeSmoke/Program.cs:7-40` makes only a synchronous
native version call and constructs generated records.

Binding generation can also install mutable tools implicitly:
`scripts/generate-bindings.sh:64-72,98-106,130-137,163-170,195-202` installs
generators during generation, including unversioned `uniffi-bindgen-cli`; CI
installs the C# generator from a mutable Git tag
(`.github/workflows/build.yml:266-270`).

Publish an explicit host/target/tier matrix and test load, call, async, and
error behavior against the exact packaged native cohort.

### P-05 — SBOM, dependency, security, and license evidence is stale

**Severity:** High — enforceable dependency/build/security/licensing violation.

`scripts/dev/generate-sbom.sh:12-41` expects a nonexistent root Python lock,
sources ambient NVM state, and invokes unpinned `npx`. The checked-in Python
SBOM describes an obsolete desktop stack instead of Torch.
`torch-server/requirements.txt:1-7` contains lower bounds without a resolved
lock. `docs/SECURITY.md:14,93-119` describes nonexistent manifests and CI
scanning/pinning practices that are not scheduled.
`docs/THIRD-PARTY-NOTICES.md:5-15,49-51` lists obsolete Python packages and
legacy Linux tools while omitting the current Torch and broader shipped
dependency inventory; `electron/package.json:48-76` does not include a notice
artifact in the packaged application.

Audit the final shipped dependency closure for all three ecosystems, use locked
inputs and pinned producers, regenerate notices from evidence, and run current
vulnerability/license checks in a named schedule.

### P-06 — Torch accepts unsupported semantics and blocks async routes

**Severity:** High — enforceable boundary/concurrency violation.

`torch-server/openai_api.py:25` leaves roles, prompts, sampling, and token
counts largely unconstrained; `:163-198` drops unknown roles, accepts but ignores
`stop`, and returns zero usage. Async handlers at `:109-156,259-292` execute
synchronous generation on the event loop.

CI installs only `torch-server/requirements-dev.txt`
(`.github/workflows/build.yml:212-213`). The suite substitutes local FastAPI,
Torch, Uvicorn, and psutil fakes when those packages are absent
(`torch-server/tests/test_validation_and_app.py:15-136`), so its 13 passing
tests do not prove real ASGI routing, middleware, dependency compatibility, or
Torch integration.

Define the supported OpenAI subset, reject unsupported values explicitly, bound
inputs, report truthful usage, and add worker/queue/cancellation/overload
ownership so generation cannot block health and control traffic.

### P-07 — Electron subscription failures and close ownership are incomplete

**Severity:** Medium — enforceable TypeScript async/lifecycle violation.

`electron/src/preload.ts:723-798` launches several subscribe/unsubscribe
promises without observing failures. Window close resets serving, runtime, and
telemetry subscription owners but omits the model-download subscription
(`electron/src/main.ts:208-216`). Async Electron lifecycle callbacks are backed
only by log-level global rejection handling at `electron/src/main.ts:601-620`.

Give each stream one main-process owner, observe subscription transitions, and
define idempotent close/unsubscribe outcomes with focused lifecycle tests.

### P-08 — Corrupt launcher-root state silently changes library authority

**Severity:** High — enforceable persistence/resilience violation.

`electron/src/launcher-root.ts:126-143` maps malformed, unreadable, or invalid
persisted configuration to `null`; `:21-64` then silently chooses another root.
The owner is written directly at `:66-89`, so interruption can leave a torn
file. Startup may therefore display or mutate a different library while looking
healthy.

Write the configuration through atomic replacement, distinguish absent from
invalid/unreadable state, and require an explicit recovery outcome before
changing the authoritative library root.

### P-09 — Launcher outcomes and deadlines are incomplete

**Severity:** Medium — enforceable launcher/cross-platform violation.

`scripts/launcher/platform-service.mjs:5` maps any unknown platform to Linux.
`scripts/launcher/commands.mjs:56-150` sends `SIGTERM` at timeout but has no
second deadline or kill escalation while awaiting `close`. `launcher.sh:12`
contains a release fast path absent from `launcher.ps1:3`.

Reject unknown platforms, define TERM/grace/KILL outcomes, and add wrapper
equivalence tests for supported launch/release actions.

### P-10 — Repository standards governance still targets legacy owners

**Severity:** Medium here; elevated to High systemically in the overview.

`CONTRIBUTING.md:29-40` and `docs/STANDARDS_ADOPTION.md:7-32` route contributors
to the old monolithic standards and count thresholds.
`scripts/dev/check-readme-coverage.sh:29-77` makes directory presence a blanket
documentation requirement, contrary to the current contract/impact rule.

Replace these with Core + Router navigation and reauthorize each permanent gate
against a named claim, proof boundary, oracle, and marginal value.

## Strengths to Preserve

- Electron enables context isolation, sandboxing, web security, and disables
  Node integration (`electron/src/main.ts:152`).
- The RPC allowlist and initial schemas are a useful base for generation.
- Launcher process creation is centralized and generally uses structured,
  non-shell spawning.
- Torch has loopback defaults, LAN token gating, constant-time token comparison,
  path containment, and model-slot locking.
- CI uses frozen pnpm/Cargo inputs for the parts that have locks and calculates
  checksums over emitted release files.

## Next Focused Audits

1. Generated desktop RPC contracts and invalid-payload outcomes.
2. Exact release artifact/SBOM/license closure.
3. Host binding support matrix and real-runtime cohorts.
4. Torch API subset, admission, inference workers, and shutdown.
5. Electron stream subscription ownership and failure outcomes.
6. Launcher-root persistence, recovery, and authority.
7. Launcher deadlines, outcomes, and wrapper equivalence.
