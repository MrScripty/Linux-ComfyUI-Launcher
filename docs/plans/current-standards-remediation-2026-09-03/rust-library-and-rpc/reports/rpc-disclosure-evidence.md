# RPC Diagnostic Disclosure Evidence

## Claim And Oracle

- Claim: `RUST-A1`.
- Source baseline: `453105780b1e5181d27dd1f20b234591bb6ead86`.
- Deciding oracle: an actual debug-enabled `pumas-rpc` child process bound to
  loopback with a temporary launcher root, plus direct tests of the Rust public
  error interface.
- Negative condition: a synthetic Hugging Face credential and synthetic private
  path/URL fragments must not occur in captured stdout, stderr, or JSON-RPC
  responses. Stable numeric codes and public classes must remain observable.
- Unsupported by this evidence: Electron log persistence, authentication for
  LAN RPC, the complete strict DTO contract, and arbitrary future diagnostic
  sinks.

## Request And Outward-Site Inventory

The JSON-RPC envelope contains protocol version, method, optional parameters,
and optional correlation ID. Parameter fields fall into these disclosure
classes:

| Class | Representative fields | Diagnostic disposition |
| --- | --- | --- |
| Secret | `set_hf_token.token` | Never log or echo |
| Private locator | local/import/model paths, destination directories, URLs, endpoints, connection URLs, download URLs | Never log from received parameters; internal error locators are projected |
| Safe identifier | registered method, numeric request ID, model/repository/download/app/profile IDs | Only allowlisted method labels and unsigned numeric request IDs enter the central request diagnostic |
| Opaque/free-form | search text, notes, metadata, runtime/plugin configuration, provider bodies | Never log from the received parameter object |
| Bounded scalar | counts, limits, offsets, booleans, enum-like selections | Not logged by the central request diagnostic |

The systemic outward-site search found and dispositioned:

- the central JSON-RPC diagnostic and error response;
- RPC shutdown diagnostics/results;
- model-library, model-download, status, runtime-profile, and serving-status SSE
  startup/serialization errors;
- 15 handler-local `PumasError` result bodies in model download, import, and
  search handlers;
- handler-local `PumasError` bodies in process, status, version, plugin, and
  Torch handlers;
- OpenAI gateway errors originating from `PumasError`, request transport,
  response transport, ONNX runtime, and non-success provider bodies; and
- launcher-root, plugin-loader, version-manager, process-launch, and serving
  diagnostics that previously emitted locators or arbitrary error text.

Provider success bodies remain the requested OpenAI-compatible result. Provider
non-success bodies are no longer passed through because they are not a safe
public diagnostic authority. Domain messages with fixed, owned wording remain
public. The server-task lifecycle diagnostic in `server.rs` is outside the
Milestone 1 write set and remains assigned to Milestone 4; it cannot receive
request parameters and does not affect this claim.

## Implemented Contract

`contract.rs` is the producer-owned seam. `PublicError` maps internal
`PumasError` codes to a closed public class and bounded static message. Unknown
and internal cases deny disclosure by default. JSON-RPC adds the public class
under `error.data.class`; SSE uses `error`, `error_code`, and `error_class`;
OpenAI-owned failures use the same bounded projection while retaining the
protocol's HTTP status.

Diagnostics identify the operation with an allowlisted method label, accept
only an unsigned numeric request ID as correlation data, and report the public
code/class. Complete request parameters and internal `Display`/`Debug` text are
not emitted.

## Results

All commands ran from `rust/` unless stated otherwise.

| Command | Result |
| --- | --- |
| `cargo test --package pumas-rpc public_error_projection` | Passed: 2 focused mapping tests |
| `cargo test --package pumas-rpc public_error_projection --no-default-features` | Passed: 2 focused mapping tests |
| `cargo test --package pumas-rpc debug_rpc_process_does_not_disclose_credentials_or_private_locators -- --nocapture` | Passed: real debug process, default features |
| `cargo test --package pumas-rpc --no-default-features debug_rpc_process_does_not_disclose_credentials_or_private_locators -- --nocapture` | Passed: real debug process, no default features |
| `cargo test --package pumas-rpc` | Passed: 70 unit and 9 integration; 10 manual external-process tests ignored by existing policy |
| `cargo test --package pumas-rpc --no-default-features` | Passed: 33 unit and 6 integration; 10 manual external-process tests ignored by existing policy |
| `cargo check --package pumas-rpc` | Passed |
| `cargo fmt --package pumas-rpc -- --check` | Passed |
| `cargo clippy --package pumas-rpc --all-targets -- -D warnings` | Passed |
| `cargo clippy --package pumas-rpc --all-targets --no-default-features -- -D warnings` | Passed |

An earlier default Clippy run correctly rejected two collapsible conditional
forms introduced while removing provider error text. The conditionals were
collapsed without changing their outcomes, and the passing rerun above is the
deciding supporting result.

Repository-root supporting searches found no complete-parameter RPC diagnostic,
raw launcher-root diagnostic, direct internal-error `Display` conversion in a
JSON-RPC error, or direct `e`/`err`/`error.to_string()` projection in a
production handler error/message field. `git diff --check` passed at the
evidence point.

## Acceptance

`RUST-A1` is satisfied. The real-process oracle and focused interface tests
decide the disclosure claim; static searches and toolchain checks are supporting
evidence.
