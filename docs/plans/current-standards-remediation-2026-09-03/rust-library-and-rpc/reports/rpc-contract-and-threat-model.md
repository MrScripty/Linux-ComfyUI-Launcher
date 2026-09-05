# RPC And Local IPC Contract Inventory

## Scope And Decision

This report bounds the Rust producer surfaces before contract migration. The
desktop sidecar currently has 149 reachable JSON-RPC operations: 147 dispatcher
arms plus the separately handled `health_check` and `shutdown` operations. It
also has ten non-JSON-RPC routes: one health route, five event streams, and four
OpenAI-compatible routes. The local `pumas-core` server has 107 unary dispatcher
arms plus one separately handled update-stream operation.

The selected contract shape is a producer-owned closed operation enum with
operation-specific request DTOs and typed outcomes. The transport envelope,
method lookup, field policy, numeric validation, outcome validation, and public
error projection are one deep contract Module; HTTP, SSE, OpenAI, and local TCP
framing remain Adapters. Domain handlers receive validated commands rather than
arbitrary JSON.

Local IPC remains a separate smaller contract. Only the six operations exposed
by `PumasLocalClient` have an evidenced production Interface. The other 102
local IPC operations are obsolete reachability left from the removed
transparent-secondary design; they have no workspace production consumer and
most do not validate the local connection token. They will be removed from the
local transport instead of being preserved in a second giant protocol.

## Desktop JSON-RPC Inventory

The canonical producer owner for every retained entry is
`pumas-rpc::contract`. The named handler module owns domain execution only.
Every entry below is retained and migrated to a closed request/outcome variant
unless a different disposition is stated.

| Domain / handler | Count | Operations |
| --- | ---: | --- |
| Built-in lifecycle | 2 | `health_check`, `shutdown` |
| Status and system | 13 | `get_status`, `get_disk_space`, `get_system_resources`, `get_status_telemetry_snapshot`, `get_launcher_version`, `check_launcher_updates`, `apply_launcher_update`, `restart_launcher`, `get_sandbox_info`, `check_git`, `get_network_status`, `get_library_status`, `get_app_status` |
| Runtime profiles | 8 | `get_runtime_profiles_snapshot`, `list_runtime_profile_updates_since`, `upsert_runtime_profile`, `delete_runtime_profile`, `set_model_runtime_route`, `clear_model_runtime_route`, `launch_runtime_profile`, `stop_runtime_profile` |
| Serving | 5 | `get_serving_status`, `list_serving_status_updates_since`, `validate_model_serving_config`, `serve_model`, `unserve_model` |
| Version management | 23 | `get_available_versions`, `get_installed_versions`, `get_active_version`, `get_default_version`, `set_default_version`, `switch_version`, `install_version`, `remove_version`, `cancel_installation`, `get_installation_progress`, `validate_installations`, `get_version_status`, `get_version_info`, `get_release_size_info`, `get_release_size_breakdown`, `calculate_release_size`, `calculate_all_release_sizes`, `has_background_fetch_completed`, `reset_background_fetch_flag`, `get_github_cache_status`, `check_version_dependencies`, `install_version_dependencies`, `get_release_dependencies` |
| Model library | 34 | `get_models`, `refresh_model_index`, `import_model`, `download_model_from_hf`, `start_model_download_from_hf`, `get_model_download_status`, `cancel_model_download`, `pause_model_download`, `resume_model_download`, `list_model_downloads`, `resume_partial_download`, `search_hf_models`, `get_hf_download_details`, `get_related_models`, `search_models_fts`, `import_batch`, `import_external_diffusers_directory`, `classify_model_import_paths`, `lookup_hf_metadata_for_file`, `lookup_hf_metadata_for_bundle_directory`, `detect_sharded_sets`, `validate_file_type`, `get_embedded_metadata`, `get_library_model_metadata`, `resolve_model_execution_descriptor`, `resolve_model_artifact_load_target`, `resolve_model_package_facts`, `list_model_library_updates_since`, `resolve_model_package_facts_summary`, `model_package_facts_summary_snapshot`, `refetch_model_metadata_from_hf`, `adopt_orphan_models`, `import_model_in_place`, `scan_shared_storage` |
| Inference settings and model governance | 13 | `get_inference_settings`, `update_inference_settings`, `update_model_notes`, `resolve_model_dependency_requirements`, `audit_dependency_pin_compliance`, `list_models_needing_review`, `submit_model_review`, `reset_model_review`, `generate_model_migration_dry_run_report`, `execute_model_migration`, `list_model_migration_reports`, `delete_model_migration_report`, `prune_model_migration_reports` |
| Hugging Face authentication | 3 | `set_hf_token`, `clear_hf_token`, `get_hf_auth_status` |
| Process management | 9 | `launch_ollama`, `stop_ollama`, `is_ollama_running`, `launch_torch`, `stop_torch`, `is_torch_running`, `open_path`, `open_url`, `open_active_install` |
| Ollama model management | 11 | `ollama_list_models`, `ollama_list_models_for_profile`, `ollama_create_model`, `ollama_create_model_for_profile`, `ollama_delete_model`, `ollama_delete_model_for_profile`, `ollama_load_model`, `ollama_load_model_for_profile`, `ollama_unload_model`, `ollama_unload_model_for_profile`, `ollama_list_running` |
| Torch inference server | 6 | `torch_list_slots`, `torch_load_model`, `torch_unload_model`, `torch_get_status`, `torch_list_devices`, `torch_configure` |
| Link management | 9 | `get_link_health`, `clean_broken_links`, `remove_orphaned_links`, `get_links_for_model`, `delete_model_with_cascade`, `get_file_link_count`, `check_files_writable`, `set_model_link_exclusion`, `get_link_exclusions` |
| Conversion | 9 | `start_model_conversion`, `get_conversion_progress`, `cancel_model_conversion`, `list_model_conversions`, `check_conversion_environment`, `setup_conversion_environment`, `get_supported_quant_types`, `get_backend_status`, `setup_quantization_backend` |
| Plugin management | 4 | `get_plugins`, `get_plugin`, `call_plugin_endpoint`, `check_plugin_health` |

The default build reaches all 149 operations. The `inference-plugins` feature
controls 65 dispatcher operations: app status, runtime profiles, serving,
version management, Ollama/Torch process operations, Ollama/Torch management,
and plugin management. The 82 other dispatcher operations plus the two
built-ins remain in the library-only build.

The Electron consumer registry lists 152 names. `resolve_pumas_model_ref` has no
producer dispatch arm and is therefore not reachable. Its disposition is
**remove from the downstream registry/projection**; the platform plan owns that
consumer edit. The removed `list_interrupted_downloads` and `recover_download`
entries also require downstream removal. The remaining 149 names correspond to
the producer surface.

## Other Desktop HTTP Surfaces

| Adapter owner | Route(s) | Disposition |
| --- | --- | --- |
| Health | `GET /health` | Retain as a typed health outcome independent of JSON-RPC. |
| Model-library events | `GET /events/model-library-updates` | Retain; typed query, event, recovery marker, and public event error. |
| Download events | `GET /events/model-download-updates` | Retain; typed event and public event error. |
| Status telemetry events | `GET /events/status-telemetry-updates` | Retain; typed query/event and public event error. |
| Runtime-profile events | `GET /events/runtime-profile-updates` | Retain only with `inference-plugins`; typed query/event and public event error. |
| Serving events | `GET /events/serving-status-updates` | Retain only with `inference-plugins`; typed query/event and public event error. |
| JSON-RPC | `POST /rpc` | Retain as the Adapter for the closed desktop contract. |
| OpenAI discovery | `GET /v1/models` | Retain only with `inference-plugins`; validate its typed model-list outcome before serialization. |
| OpenAI inference | `POST /v1/chat/completions`, `POST /v1/completions`, `POST /v1/embeddings` | Retain only with `inference-plugins`; preserve endpoint-specific request bodies/status semantics and the bounded public-error contract. |

## Local Core IPC Inventory And Consumers

The canonical owner is `pumas-core::ipc::protocol`. `PrimaryState` is the
production executor, `IpcServer`/`IpcClient` are local TCP framing Adapters, and
the exported `PumasLocalClient` is the only supported caller Interface.

| Population | Count | Operations | Disposition |
| --- | ---: | --- | --- |
| Typed unary `PumasLocalClient` Interface | 5 | `model_library_selector_snapshot`, `resolve_model_artifact_load_target`, `resolve_model_package_facts_summaries`, `resolve_model_execution_descriptors_batch`, `get_inference_settings_batch` | Retain as closed commands/outcomes; require the connection token before domain execution. |
| Typed stream Interface | 1 | `subscribe_model_library_update_stream_since` | Retain as a separate typed streaming command/handshake/event contract; require the connection token. |
| Obsolete catalog/settings reachability | 9 | `list_models`, `search_models`, `get_model`, `delete_model_with_cascade`, `import_model`, `import_models_batch`, `rebuild_model_index`, `reclassify_model`, `reclassify_all_models` | Remove from local transport. |
| Obsolete single-item settings/status reachability | 4 | `get_inference_settings`, `update_inference_settings`, `update_model_notes`, `get_library_status` | Remove from local transport. |
| Obsolete resolution/update reachability | 18 | `resolve_model_dependency_requirements`, `resolve_model_execution_descriptor`, `resolve_model_package_facts`, `list_model_library_updates_since`, `get_runtime_profiles_snapshot`, `list_runtime_profile_updates_since`, `upsert_runtime_profile`, `delete_runtime_profile`, `set_model_runtime_route`, `clear_model_runtime_route`, `resolve_runtime_profile_endpoint`, `resolve_runtime_profile_endpoint_for_operation`, `resolve_model_runtime_profile_endpoint`, `resolve_model_runtime_profile_endpoint_for_operation`, `model_runtime_route_auto_load`, `resolve_model_package_facts_summary`, `model_package_facts_summary_snapshot`, `resolve_pumas_model_ref` | Remove from local transport. |
| Obsolete governance/import/link/migration reachability | 18 | `audit_dependency_pin_compliance`, `list_models_needing_review`, `submit_model_review`, `reset_model_review`, `get_effective_model_metadata`, `import_external_diffusers_directory`, `import_model_in_place`, `adopt_orphan_models`, `get_link_health`, `clean_broken_links`, `get_links_for_model`, `set_model_link_exclusion`, `get_link_exclusions`, `generate_model_migration_dry_run_report`, `execute_model_migration`, `list_model_migration_reports`, `delete_model_migration_report`, `prune_model_migration_reports` | Remove from local transport. |
| Obsolete HF/download/auth reachability | 19 | `search_hf_models`, `search_hf_models_with_hydration`, `get_hf_download_details`, `start_hf_download`, `get_hf_download_progress`, `cancel_hf_download`, `pause_hf_download`, `resume_hf_download`, `list_hf_downloads`, `list_interrupted_downloads`, `recover_download`, `resume_partial_download`, `refetch_metadata_from_hf`, `lookup_hf_metadata_for_file`, `lookup_hf_metadata_for_bundle_directory`, `set_hf_token`, `clear_hf_token`, `get_hf_auth_status`, `get_hf_repo_files` | Remove from local transport. |
| Obsolete network/process/version reachability | 25 | `is_online`, `connectivity_state`, `check_connectivity`, `network_status`, `get_network_status_response`, `get_disk_space`, `get_status_response`, `get_system_resources`, `is_ollama_running`, `stop_ollama`, `launch_ollama`, `launch_runtime_profile`, `stop_runtime_profile`, `is_torch_running`, `stop_torch`, `launch_torch`, `get_last_launch_log`, `get_last_launch_error`, `get_status`, `has_background_fetch_completed`, `reset_background_fetch_flag`, `get_launcher_version`, `check_launcher_updates`, `apply_launcher_update`, `ping` | Remove from local transport. |
| Obsolete conversion reachability | 9 | `start_conversion`, `get_conversion_progress`, `cancel_conversion`, `list_conversions`, `is_conversion_environment_ready`, `ensure_conversion_environment`, `supported_quant_types`, `backend_status`, `ensure_backend_environment` | Remove from local transport. |

The retained/removed totals are 6 and 102 respectively. No production Rust
workspace caller uses the generic `IpcClient::call`; only `PumasLocalClient`
and IPC tests do. The low-level client, request envelope, response envelope, and
dispatcher trait therefore become crate-internal implementation details. This
preserves the actual external consumer while removing unauthenticated and
untyped accidental capabilities.

## Representative Contract-Shape Comparison

| Shape | Credential command | Signed pagination / size | Typed collection outcome | Event error | Result |
| --- | --- | --- | --- | --- | --- |
| One method-string registry with runtime field descriptors | Registry describes `token: string`; handlers still receive JSON and the secret remains printable through generic values. | Descriptors can reject a negative integer, but conversion/default rules remain duplicated in handlers. | A descriptor can say `array` or `object` but cannot prove element or response-wrapper types. | Requires transport-specific ad hoc descriptors. | Rejected. It adds a shallow registry while retaining the existing knowledge leaks. |
| Closed producer command/outcome variants with reusable typed DTOs | `SetHfToken` owns a non-debuggable credential DTO and only the handler receives its value. | A checked `BoundedCount`/pagination DTO rejects negative, fractional, and over-limit values before dispatch. | The operation outcome contains its actual element type; serialization cannot substitute `{}`, `[]`, `false`, or `""`. | Event streams serialize a closed `EventOutcome<T>` or the shared bounded public error. | Selected. It is the smaller caller Interface even though the owning implementation contains more knowledge. |
| One enormous shared enum for desktop RPC and local IPC | Can type the credential. | Can type bounds. | Can type collections. | Must encode unrelated TCP stream and HTTP/SSE lifecycle variants. | Rejected. Sharing serialization machinery would couple two different callers, trust boundaries, operation sets, and lifecycles. |

The selected shape uses domain-grouped variants inside each producer contract,
not one DTO per transport. Reusable value types are admitted only when their
validation semantics are identical. Handler-local compatibility aliases and
the generic `wrapper.rs` are deleted as their callers migrate; they are not
kept beneath the new contract.

The field policy is explicit and uniform unless a named DTO opts into a
different representation:

- `jsonrpc` must be exactly `"2.0"`; a wrong or missing version is invalid
  request, not method-not-found.
- Unknown method names produce method-not-found/unsupported and never reach a
  domain handler.
- Params must be an object. Missing params maps to an empty object only for an
  explicitly empty request. `null` is not an empty object.
- Request DTOs deny unknown fields. Transitional snake/camel aliases are
  declared on the receiving field rather than probed manually.
- Signed/floating/oversized numeric input is rejected before conversion to
  `usize`; operation-specific maxima live with the value type.
- A handler outcome must match the operation's outcome variant before the
  transport serializes it. There is no null-to-empty, false, empty-string, or
  default-object recovery.

## Threat Model And Exposure Decision

Protected assets include the HF credential, model/library files and private
paths, update/install controls, local processes, plugin endpoints, inference
traffic, and event history. Adversaries include a compromised renderer, any
same-device process able to reach loopback, a hostile non-browser TCP client,
and—only if LAN mode exists—any network peer able to reach the bound port.
CORS is browser behavior and is not authentication.

Current controls are body and connection limits, loopback-only binding,
Electron-side partial validation, a connection token on only six local IPC
operations, and the Milestone 1 public-error/redaction projection. The producer
must still reject malformed requests independently of Electron.

`RUST-I1` is resolved. Repository inspection found no production consumer of
non-loopback desktop RPC; Electron connects to `127.0.0.1`. The accepted
standards-aligned decision removes `--allow-lan` and rejects every non-loopback
`--host`. A future LAN design cannot proceed from CORS alone: it must first
define authenticated capabilities, per-operation authorization, credential
provisioning/rotation/revocation, admission and rate policy, event-stream
authentication, and typed failure behavior for all protected operations and
routes.

## Slice Acceptance

- The producer populations are bounded: 149 desktop JSON-RPC operations, ten
  other routes, and 108 local IPC operations.
- Every desktop entry has a retained or downstream-removal disposition.
- Every local IPC entry has a retained or removed disposition, with the only
  supported consumer Interface named.
- Three shapes were compared using all four required representative cases; the
  closed producer command/outcome design is selected.
- The local IPC contract was eligible to close independently of the exposure
  choice; the accepted loopback-only policy is now enforced and proven below.

## Local IPC Implementation Evidence

The local IPC source slice implemented the selected smaller contract. The
public `PumasLocalClient` still exposes its five unary operations and one
update stream, while the wire parser recognizes no other operation. The
low-level request/response/client/server/dispatch machinery is crate-internal.
Each unary result is deserialized to its concrete domain type and reserialized
before a response frame is emitted. The streaming handshake and notifications
retain concrete types, correlation IDs, and typed public error frames.

The deciding focused run passed 34 IPC tests plus a real `PrimaryState` TCP
integration test. Negative cases covered malformed JSON, wrong protocol and
envelope shape, unknown/obsolete methods, missing/null/extra fields,
negative/oversized selector values, oversized batches, wrong outcome types,
invalid connection tokens, and secret-free public failures. Format, all-target
Clippy with warnings denied, and focused diff checks also passed. This closes
the local IPC portion of Milestone 2; the desktop RPC portion remains active.

## Aggregate Test Diagnosis

An aggregate run later reported two independent-looking roots: a read-only
SQLite failure while an unguarded unit test constructed `PumasApi`, and an
`EPERM` during `tests::test_api_creation`. Nine later failures were poisoned-
mutex cascades. The bounded diagnosis did not reproduce either root:

- both exact roots passed independently (1/1 each);
- the 859-test library unit suite passed serially, at default concurrency, and
  with 128 test threads;
- two simultaneous 859-test library processes both passed;
- twelve simultaneous exact API-construction processes passed (12/12); and
- the exact workspace test stage passed alone, including the relevant 78,
  859, and 34-test binaries and every subsequent workspace test binary.

The failure is therefore classified as environment-local IPC denial and
resulting shared-test-state interference, not as an accepted code defect. A
governance rerun reproduced the 848/11 pattern in its restricted sandbox, but
the independent `EPERM` moved to a migration test. The same isolated
`./scripts/rust/check.sh` then exited zero with the environment-required
elevated permissions after format, check, Clippy, all workspace
tests/doctests, and the no-default check. The crate-global registry-path
override and process environment override remain plausible cascade
amplifiers, but no red-capable command survived the correct execution
environment. No speculative fix was made. A future recurrence should preserve
the root failure's exact process overlap and registry path/lifetime evidence
before changing the owning test seam.

## Desktop Request Admission Evidence

The first desktop producer slice moved the exact JSON body and 18 commands
(17 without inference plugins) behind `AdmittedRpcRequest`. The admitted
commands cover built-ins, status/system operations, and HF credential state.
The contract distinguishes malformed JSON (`-32700`), invalid envelopes
(`-32600`), unsupported methods (`-32601`), and invalid typed params (`-32602`).
It rejects unknown envelope/selected-param fields, wrong/null param types,
invalid or oversized IDs/methods, duplicate snake/camel aliases, empty or
oversized identity/credential strings, and preserves the request ID only when
its form is valid.

The HF token is held by `SecretToken`, which has no `Debug` or `Display`
implementation, and is passed directly from the admitted command to the core
API. The generic auth handler and its legacy dispatcher branches were removed.
The real HTTP child-process fixture proved the error distinctions and that a
hostile extra-field credential request cannot echo its sentinel. Six direct
contract tests and the real adapter test passed under default and no-default
features; full default/no-default tests, checks, Clippy with warnings denied,
format, and focused diff checks also passed.

This is an accepted incremental request boundary, not final `RUST-A2` closure.
Unmigrated domains are explicitly represented by the temporary `Legacy`
command. Their typed params, typed results, and the generic response wrapper
remain pending domain-by-domain removal in Milestone 2D and later slices.

## Typed Outcome And Link-Domain Evidence

The initial 18 admitted commands now return closed typed `RpcOutcome` variants
instead of arbitrary JSON and bypass the compatibility wrapper. The status and
system handlers return concrete core response types; launcher-version JSON is
validated through an exact DTO; built-in, shutdown, sandbox, mutation, and auth
shapes are producer-owned. Complex payloads are boxed to keep the enum itself
small without weakening its type boundary.

The next complete non-plugin group migrated all nine link operations. Their
DTOs declare snake/camel aliases, reject duplicate/extra/wrong-typed fields,
bound non-empty IDs/tags/paths, and cap writable-file checks at 512 non-empty
items. Core link results and producer-specific cleanup/count/writability
results are typed, and the nine legacy dispatcher branches are gone. The
producer now owns 27 typed commands with inference plugins and 26 without.

Ten direct contract tests passed in both feature modes. The real HTTP adapter
passed with actual loopback binding in both modes, including hostile link
params and a valid typed link-health result. Full default/no-default suites,
check, Clippy with warnings denied, format, and focused diff checks passed.

## Conversion-Domain Evidence

All nine conversion operations now enter through exact request DTOs and return
closed outcomes. The contract enumerates accepted conversion directions and
quantization backends, bounds every provided identifier/path-like string, and
rejects duplicate aliases, extra fields, empty values, null/wrong types, and
unknown variants before dispatch. Domain handlers no longer inspect JSON, and
the conversion legacy arms and wrapper entries are removed.

Conversion worker diagnostics remain internal. Progress/list outcomes replace
the worker's arbitrary error string with one stable public sentence; a direct
credential/private-path sentinel test proves the result projection is bounded.
The contract now owns 36 typed commands with inference plugins and 35 without.
Direct contract tests, a real loopback child-process adapter test, full suites,
check, format, and all-target warnings-denied Clippy passed in both feature
modes. Remaining `Legacy` commands still block the final platform schema
handoff.

## OS-Open Evidence

`open_path` and `open_url` now use exact bounded producer requests and a shared
typed operation-status result. Environment-dependent path canonicalization and
HTTP(S)-scheme validation remain at the handler seam; handlers no longer parse
JSON, and OS-launch failures cannot return internal error text. The two legacy
dispatcher and wrapper entries are gone. Direct contract tests and real
child-process negative path/URL cases passed in both feature modes, followed by
full suites, format, and warnings-denied Clippy. The contract owns 38 typed
commands with inference plugins and 37 without them.

## Download-Lifecycle Evidence

All eight retained model-download commands use exact requests and closed outcomes. The
producer bounds required and optional identifiers, selected file collections,
and model-card JSON; rejects missing, duplicate, extra, null, and wrong-typed
fields before dispatch; and replaces worker and recovery detail with stable
public messages. Valid missing progress, missing mutation, and empty lists are
represented separately from subsystem and task failure.

Cross-review found and corrected two lower-layer default paths before this
contract was handed downstream. Disabled Hugging Face support now returns
`Result::Err` from the public `PumasApi` progress, mutation, and list methods
rather than `None`, `false`, or an empty vector. Those methods forward to one
internal primary-state owner, desktop RPC maps configuration failure to the
bounded `unavailable` error, and the transitional UniFFI caller also projects
`Result` without a fallback pending its accepted removal. The deliberately
smaller local IPC contract does not expose model-download methods: a real
listener rejected all six attempted lifecycle names as method-not-found with
no result.

The interrupted-directory scanner now awaits its blocking task through a
join-preserving helper. A deterministic task panic proves join failure becomes
an internal error rather than successful empty state; no production test
toggle was added, and RPC does not expose its detail. Direct public-API and RPC
contract/dispatch tests, the real desktop and local-IPC adapters, full core and
RPC suites, the transitional adapter suite, workspace check, format, and all-
target warnings-denied Clippy passed. The contract owns 48 typed commands with
inference plugins and 47 without; remaining `Legacy` commands still block the
stable platform projection handoff.

## Model-Catalog Evidence

The corrected catalog boundary is re-frozen for independent review. Reconciliation
admission distinguishes opportunistic from forced work and returns
`Started(run token)`, `Clean`, or `InFlight`. Admission clears the admitted
scope's preexisting dirty bit(s), and an opaque allocation identity owns the
active run. Concurrent dirty marks survive matching success; failure or
cancellation/drop restores dirty; allocation-identity comparison prevents a
stale token from settling a replacement run. Full and model scopes exclude one
another in both directions. Full dirty/failure/drop propagates to every known
model scope, while unseen scopes run on first access; targeted success does not
consume the pending full retry. `refresh_model_index` uses forced admission, so
overlap returns the stable typed `conflict` (`-32011`) rather than a successful
old count. A successful response is fresh for that request receipt; the
model-library update stream remains an independent cursor-driven event
contract and no false per-record/server timestamp was added.

The complete-list query no longer routes through paginated search. One stable
unbounded `ModelIndex::list_all` query returns all rows or propagates a row
decode error, after which the library applies its normal projections and
dedupe. A real 10,001-row fixture returns all 10,001; a corrupt direct-SQL row
is rejected rather than logged and omitted. Active dependency bindings always
replace the metadata projection, including an empty authoritative result.

The desktop response no longer serializes core `ModelRecord`. Its fallible
`CatalogModel` contains required `id`, `modelDir`, `displayName`, and
`modelType`; optional normalized `format`, `quantization`, JS-safe `sizeBytes`,
and `displayDate`; checked `dependencyCount`; a closed complete/partial
`artifact`; and a closed clean/duplicate `integrity` state. Partial state alone
may contain finite bounded `downloadProgressFraction` (`0.0..1.0`, below 1.0
while partial), fixed reason codes, and optional recovery identity. Recovery
repo ID is exact nonempty `owner/name`; selected files are bounded safe
relative paths and deterministically sorted. The private repo-ID smart
constructor mirrors the official `huggingface_hub` 1–96-byte character,
boundary, repeated-separator, and `.git` rules, narrowed to required
`owner/name`. The private selected-path constructor is host-independent: it
uses `/`, rejects POSIX/Windows root, traversal, drive, UNC, reserved-device,
invalid-character, empty/dot, and trailing-dot/space components, and bounds
both total path and individual component bytes. These are conservative common
Linux/Windows recovery constraints, not a universal filesystem claim. Every
emitted string and nested list is bounded. Raw metadata, nondeterministic
hashes, tags, update time, and unsupported related-state guesses are absent.
Exact-shape, malformed/cross-field, oversized, recovery-ID/path,
deterministic-byte, public conflict, and checked-u32 fixtures pass in both RPC
feature modes. Repository grammar authority:
[`huggingface_hub` validator](https://github.com/huggingface/huggingface_hub/blob/main/src/huggingface_hub/utils/_validators.py).
The contract counts are 48/47 typed and 101/37 `Legacy`
until independent acceptance.

Core `download_incomplete` is the readiness authority. In particular, the
supported mixed-quant shape with a complete displayable Q5 GGUF beside the
selected Q4 `.part` remains `artifact.state=complete`; subordinate part/missing
facts do not turn that model partial or invalidate the whole catalog. A direct
core-record-to-catalog regression preserves this cross-owner contract. That
realistic fixture also carries the selected Q4 repo/artifact/file/quant fields;
the complete Q5 projection remains exactly `{state: complete}` and never
exposes recovery. Recovery identity is parsed and emitted only for a partial
artifact.

Desktop RPC recovery actions no longer accept ambient repository or filesystem
authority.
The unconsumed desktop `list_interrupted_downloads` and `recover_download`
methods are method-not-found. The retained `resume_partial_download` request is
exact camel-case `{modelId,recoveryToken}`; snake-case, the old repo/path shape,
malformed model IDs, and anything outside the fixed `v1:` plus 64-lowercase-hex
token grammar are rejected before lookup.

One core recovery-snapshot module issues and verifies the token. It hashes an
explicit domain/version frame plus the model ID, canonical managed directory,
validated Hugging Face `owner/name`, selected artifact/quant, and the bounded
sorted-unique selected/expected file set. File reorder and duplicate metadata
therefore preserve the fingerprint, while file membership, repository,
artifact, quant, model identity/path, or complete/partial state changes make an
old action stale. Ordinary equality is sufficient because this unkeyed BLAKE3
value is only a collision-resistant stale-state precondition; it does not
authenticate or authorize a caller. Authority comes from fresh server-side
record lookup, filesystem projection, canonical-root/model-ID agreement, and
producer-derived repo/path/files.

Complete rows return before recovery metadata is parsed and always serialize
exactly `{state: complete}`. Filesystem-ineligible partial rows remain visible
but omit recovery: this includes outside-root imports, path aliases, symlinked
model directories, missing/uninspectable paths, non-UTF-8 canonical identity,
and missing provenance. A root canonicalization/configuration failure can still
fail the catalog. The action reindexes the resolved model before verification;
stale or tracked repo/file-context mismatch returns a closed public refusal.
Exact-context tracked paused/error work resumes, active work attaches, and an
indexed untracked partial starts with only the verified producer-derived file
set. Real loopback tests cover stale refusal, tracked resume and attach,
cache-backed untracked recovery, removal of both old methods, the hostile old
repo/path request, and omission of internal messages and locators.

Independent review rejected the first path-free action implementation because
it still delegated new recovery to generic download admission. That path could
accept only the remote intersection of the ticket's files, add unbound
repository auxiliaries, and race destination-only deduplication against an
unrelated context. The admitted replacement has one exact recovery resolver:
every member of the canonical ticket-bound set must exist remotely, and only
that set is scheduled. One downloads write-lock admission owner decides an
exact destination/repository/file-context attach or resume versus a new
recovery insertion. A missing member refuses before task or target mutation;
concurrent unrelated context cannot attach, filter the set, or become the
returned download.

The replacement also preserves filesystem authority through use. Core holds a
`cap_std::fs::Dir` for the canonical managed library root and performs recovery
preflight, parent creation, metadata, part open/length/removal, marker removal,
and final rename relative to that handle. It never reconstructs ambient
authority from the serialized/display destination. Existing symlink components
are refused when observed. Deterministic Linux tests replace a validated
`.part` path and the validated model-directory path with outside symlinks;
subsequent capability-relative operations refuse or remain confined, and no
outside file is mutated. The held root does not pin the original model-
directory inode and does not prevent a same-user replacement that resolves
elsewhere inside that same managed root. Windows and macOS runtime behavior
remains unavailable evidence; the design is cross-platform Rust but the
recorded runtime claim is Linux only.

The directory capability is a private non-serialized field in the existing
download-state aggregate. Recovery insertion and tracked resume prepare all
awaited dependencies first, then install state, capability, start-gated worker,
and `task_registered=true` synchronously under the downloads/task owner before
opening the gate. Caller cancellation before that commit leaves no new owner;
cancellation afterward leaves a real registered worker. Exact-context attach
requires that non-finished capability-backed task and never treats a persisted
ambient or unregistered row as active recovery. Error, pause, and reconciled
inactive states retain the capability; completion and observed final
cancellation clear it under the same state owner. Recovery tasks receive no
ordinary status-persistence owner or callback. A separate narrow terminal
cleanup authority verifies that no live resumable row remains while preserving
the durable revocation tombstone before capability release.

The preceding lifecycle candidates were not accepted: one awaited blocking
work under a download-state guard, and the next still reserved strict
revocation under a synthetic ID, let recovery data write/flush escape the
owner, detached nested custody from a cancelled drain waiter, and could publish
false cancellation after worker/nested/cleanup failure. Slice A establishes
the persistence publication prerequisite. The fourth corrected Slice B is
accepted: it reserves the
actual download ID as `RecoveryTransition` before revocation and promotes that
same opaque generation to `Worker`; generic resume and relocation refuse the
reservation, and stale generations cannot remove a successor.

The generation owner retains the outer Tokio task and every registered
blocking observer. Recovery and Ambient create, truncate/open, write, flush,
remove, rename, and marker mutations use its `TaskContext`; started blocking
filesystem work therefore cannot outlive owner drain and recreate artifacts
after terminal cancellation. Completed nested observers are reaped only after
the JoinHandle itself is terminal and its Join and semantic failure have been
consumed. The bounded archive preserves failures even if the request-side
result receiver is cancelled.

Finished observation atomically replaces the completed owner with an actual-ID
`TerminalProjection`. Non-owning generation-bound tickets share the one
caller-independent projector, so a dropped waiter cannot lose or duplicate
state/persistence publication. Sticky predecessor and projector failure is
captured before fallback awaits and transferred to a superseding finalizer;
fallback `RolledBack` is not mistaken for Error publication. If both the
projector and fallback panic, typed `FailureUnprojected` stops request-side
spinning while the actual-ID owner remains reserved. Cell custody is forwarded
across finalizer/projector replacement and acknowledged/settled only after a
finalizer publishes the fail-closed Error snapshot. Active clean-Join Workers
and clean-Join finalizers that left state `Cancelling` are unverified terminal
obligations, not successful completions. The same provenance remains in
non-serialized state after projector settlement so a later ownerless cancel
cannot overwrite `Error` with false `Cancelled`.

Caller-independent cancellation records `Absent | Observed` predecessor truth
and atomically captures whether the predecessor outer task had already ended at
the exact replacement. The finalizer cleans every bound part and marker for
Ambient and Recovery destinations, verifies the strict persistence disposition,
drains its owned work, and only then publishes terminal state. Any predecessor,
filesystem, persistence, nested, or finalizer failure becomes sticky `Error`;
recovery authority remains held and the revocation tombstone remains durable.
Reconciliation uses its own actual-ID terminal projector, requires exact
Ambient status-update success before memory `Paused`, and rechecks generation
and expected state. State-local ambient revocation disposition prevents a
transition that durably revoked and then disappeared from reopening generic
resume/relocate.

Strict successful-completion cleanup is a narrow Slice B exception for both
destination kinds: persistence cleanup and final drain precede `Completed`,
capability release, and the Ambient success callback. The admitted auxiliary
and completion callbacks execute as owned blocking work outside the destination
lease. Auxiliary continuation reacquires the lease and revalidates exact
generation, status, destination, and cancellation; its panic is an owned
preterminal failure. A completion-callback panic is observed after verified
completion and does not roll state back.

Deterministic production-topology barriers exercise the public cancel path with
real Ambient and Recovery state across held mutations and cleanup, including a
regular scanner-visible `.part` removal failure. Separate barriers cover strict
revocation versus generic resume/relocate/cancel, transition caller drop,
request-cancelled observation, dual observer ABA, post-memory/pre-persistence
reconciliation, projector panic/supersession, exact cancel replacement after a
Worker finishes, outer-finished/nested-held drain, rejected Worker and
RecoveryTransition retirement, and reentrant/panicking callbacks.

Slice C is active on the accepted Slice B owner in exactly nine admitted source
files. The product builder opens the selected model-library root once and
injects a crate-private held capability; public client construction and wire
outcomes do not change. A caller-independent, attempt-identified store-v3
transition durably owns the full request, non-authorizing destination identity,
domain, FIFO ordinal, and predecessor/release proof before an ID, active
snapshot, or effect is exposed. Only confirmed durability promotes that exact
attempt to the gated Worker; ambiguous publication remains hidden and parked.

Destination reservation and all marker/part/cleanup effects derive from the
same held root plus validated relative target. Raw or canonical path spelling,
nearest-existing ancestors, UUID/time/file order, and physical async mutexes do
not grant authority or FIFO. Missing targets and aliases retain one identity;
root/path replacement fails closed. Paused, recoverable Error, or Pending
cleanup retains state-lifetime custody. Release requires the exact current
generation plus strict durable terminal evidence, final drain, and matching
publication; a terminal projector can rescue a panic after that proof without
releasing stale or resumable state.

Store v3 strictly migrates legacy/v1/v2 Error as recoverable state and owns an
exclusive full-snapshot quarantine. Pending cleanup is independent of sticky
failure provenance: clean Pending uses an exact-attempt durable removal/release
proof before Cancelled, while sticky Pending-to-Verified retains Error and any
recovery tombstone. Visibility/durability uncertainty parks custody and never
authorizes verification, release, or empty restore. Capability-relative marker
publication reuses Slice A's typed atomic outcome algebra. Restoration,
reconciliation, pause, relocation, and callbacks remain caller-independent,
owned, and drained; snapshot dispatch is ordered and outside all guards.
Initial public reds reproduced ownerless ordinary setup/resume state, and the
first store red failed on the missing quarantine owner. The active matrix also
covers FIFO crash restore, alias/replacement confinement, exact removal proof,
stalled pause, terminal rescue, and callback release/drain ordering. Domain and
composed RPC corrections remain Slices D-E, and general client-drop drain
remains Milestone 4/RUST-A6.

Revised Slice A opens one pre-existing parent authority and retains it through
capability-relative unique staging, rename, exact-handle synchronization, and
configured-parent identity verification. `atomic_publish_json` distinguishes a
boxed pre-rename failure plus observed staging cleanup from visibility-unknown,
published-durability-unknown, and durable publication. A rename error stays
visibility-unknown because its filesystem effect cannot be inferred; only
`NotFound` from a direct read means absence. Existing `atomic_write_json`
callers retain their legacy Interface and acquire no new durability claim.

The accepted version-2 revocation document persists per-download revocation
attempt/disposition; version 3 extends that same strictly locked document with
durable admission/FIFO and exclusive lifecycle-quarantine ownership rather than
introducing another writer. Every versioned whole-document mutation
runs under the in-instance mutex plus `.downloads.lock`, strictly rereads the
store, and uses the durable publisher. The observer exposes separate attempting
and acquired test points immediately around the OS lock. Revocation publishes a
durability-unknown intent and a durable confirmation as separate durable
replacements even for an absent row. A fresh owner therefore cannot promote
bare absence or an uncertain attempt; stale save/status/relocate operations see
the persisted tombstone and fail closed. The OS lock coordinates store
read-modify-write and crash-reopen across constructors/processes, not task
lifetime; task admission assumes one active Pumas runtime per root. The HF
caller now uses Slice B's accepted caller-independent transition owner for
ambient-to-capability conversion. Slice C is tightening ordinary Worker
admission and publication without changing that revocation contract.

A confirmation publication that returns parent-sync or rename visibility
uncertainty never succeeds its initiating call. The same locked attempt already
durably published its unknown predecessor before preparing confirmation, so
revocation remains fail-closed: a fresh owner may reuse a visible durable
successor, while a pre-effect confirmation retries from the persisted unknown.
Pre-publication failures retain their closed stage/kind through the store outcome
instead of being reconstructed as rename failures.

Real focused runtime evidence is Linux on the exercised local ext-family
filesystem only. `Durable` records completed file and held-parent sync syscalls
plus configured-parent identity, not a universal hardware/power-loss guarantee.
macOS remains unverified. On Windows/non-Unix the publisher returns a closed
target-admission/unavailable pre-rename failure rather than false durability.
Network/distributed filesystems are unsupported and require rejection or
separate real evidence before integration with task admission.

The manifests declare caret-compatible `cap-std 4.0.3` with its default
features disabled; `Cargo.lock` resolves exactly `4.0.3`. The lock change is
limited to the resolved cap-std closure and its core package edge. Reverse and
no-default feature trees show only the intended direct core consumer, and the
Pumas correction contains no new `unsafe` block.
Downloaded package manifests report Apache-2.0/MIT-compatible licensing for
the closure; target-only `winx 0.36.4`, fetched separately because Cargo had
not downloaded it on Linux, reports `Apache-2.0 WITH LLVM-exception`.
`cargo-audit` and `cargo-deny` were unavailable and were not installed as
fallback, so no vulnerability-audit success is claimed.

Finally, `PartialDownloadOutcome` rejects blank or oversized download IDs and
impossible action/status/reason/ID combinations before serialization. The
public shape still contains no path, repository locator, or internal message.
This semantic correction does not add another command: provisional counts
remain 48/47 typed and 101/37 `Legacy`.

Remote LFS sizes are also part of closed recovery admission. The exact set may
contain at most 512 files, but even that bounded count can overflow a `u64`
total. Checked accumulation maps overflow to the same stable bound-files
refusal before downloads-state insertion, task registration, marker creation,
or part-file write. A deterministic `u64::MAX + 1` remote fixture first
reproduced the debug overflow panic and then proved the no-mutation refusal.

The recovery owner also maps lookup, reindex, token verification,
repository/network, capability, and admission errors into the closed desktop
action reason algebra. A real loopback cache-failure test returns
`success:false` with `recover_failed` and no internal text or JSON-RPC internal
error. A separate real loopback concurrency test issues two ticket-only
requests against a paused tracked context and observes exactly one `resume`
plus one `attach` to the same actually registered capability-backed owner.

This desktop boundary does not claim to have removed every lower legacy
surface. Public core/local-IPC and UniFFI
`recover_download(repo_id, dest_dir)` remain transitional ambient-authority
reachability. They are held for the accepted zero-`Legacy` local-IPC removal and
Milestone 6 binding deletion; they are not called by the new desktop action and
must not be described as secured by this ticket contract. Likewise, the current
Electron preload and renderer still send the old repo/path action shape. The
producer boundary therefore is not a standalone shippable app commit: its
platform decoder/preload and Frontend Milestone 4 consumer migration must land
as one coordinated integration before aggregate app behavior is green.

Disabled Hugging Face handling stays explicit. Progress, mutation, and list
APIs retain the accepted `Result::Err` contract. This action is the deliberate
closed-outcome exception: it returns `success:false` with
`hf_client_unavailable`, never false success or an empty default, and its
internal message is discarded by the public producer projection. The same
outcome test passes in default and no-default feature modes.

## Accepted Exposure Decision

`RUST-I1` is resolved: desktop RPC is loopback-only. `--allow-lan` is removed
and every non-loopback `--host` is rejected. This is the smallest contract
consistent with the only production consumer, which connects to
`127.0.0.1`; it avoids inventing an unauthenticated remote API or treating CORS
as authorization.

The listener accepts a private-field `LoopbackHost` value rather than an
unvalidated string. Construction accepts numeric IPv4/IPv6 loopback addresses
only, which makes a remote bind unrepresentable at the server interface. A real
RPC child process rejected `0.0.0.0` and the removed `--allow-lan` flag before
advertising a port in both feature modes. The production child-process adapter
also bound a real loopback listener and completed a request in both modes. Full
default/no-default RPC suites and their check, format, and warnings-denied
Clippy gates passed at this boundary.
