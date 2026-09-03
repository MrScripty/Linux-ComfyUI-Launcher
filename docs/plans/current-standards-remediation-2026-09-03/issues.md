# Current Standards Remediation Issues

## Open Decision Dependencies

| ID | Canonical owner | Program effect | Required disposition |
| --- | --- | --- | --- |
| `RUST-I1` | [Rust issues](rust-library-and-rpc/issues.md) | Blocks remote-exposure portion of PRG-A1/PRG-A2 | Remove LAN support or accept complete authentication/authorization contract |
| `RUST-I2` | [Rust issues](rust-library-and-rpc/issues.md) | Blocks plugin portion of PRG-A4 | Decide fail-startup versus explicit disabled/degraded plugin state |
| `RUST-I3` | Platform Milestone 0 and [Rust issues](rust-library-and-rpc/issues.md) | Blocks binding portion of PRG-A6 | Decide Rustler host/support disposition |
| `PLAT-I1` | [Platform Milestone 0](desktop-release-bindings-and-torch/plan.md) | Blocks target/host/release portions of PRG-A3/PRG-A4/PRG-A6 | Accept consumers, channels, matrices, and evidence/legal owners |

No implementation-time issue has been recorded. New issues belong first with
their focused owner and are promoted here only when they change program
sequence, shared ownership, objective scope, or acceptance.
