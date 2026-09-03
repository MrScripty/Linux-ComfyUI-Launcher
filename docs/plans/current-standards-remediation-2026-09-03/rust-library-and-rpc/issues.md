# Rust Library and RPC Issues

## Open Decision Dependencies

| ID | Relationship and evidence | Owner | Current disposition | Required verification | Revisit trigger |
| --- | --- | --- | --- | --- | --- |
| `RUST-I1` | R-02: unauthenticated `--allow-lan` materially changes the desktop RPC trust boundary. | Product/program owner; Milestone 2 consumes | Decide remove LAN or define authenticated capabilities, authorization, credential lifecycle, admission/rate, event-stream, and failure contracts. Not a blocker for Milestone 1. | RUST-A3 hostile-client system evidence | Before any Milestone 2 exposure edit |
| `RUST-I2` | R-09: configured plugin-loader failure currently substitutes a temporary root and may panic. | Product/program owner; Milestone 4 consumes | Decide whether plugins are required (startup fails) or optional (explicit disabled/degraded state). | RUST-A6 invalid-root and startup/shutdown system evidence | Before plugin startup implementation in Milestone 4 |
| `RUST-I3` | R-08: Rustler declares a core dependency but exports local conversions; host support is unproved. | Desktop/platform plan; Milestone 6 consumes | Supply the supported-host disposition, then remove the false core claim/dependency or implement only the accepted bounded adapter. | RUST-A8 Rust-side proof; host/release proof stays downstream | Before Rustler edits in Milestone 6 |

No implementation-time issue has been recorded. New issues must include
severity, exact evidence, relationship to an objective, canonical owner,
disposition, adequate verification, and a concrete revisit trigger.
