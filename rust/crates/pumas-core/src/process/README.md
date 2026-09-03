# Process

## Purpose

Process lifecycle management for inference runtimes. It launches binaries with
log capture and health checks, tracks managed PID files, and stops process trees.

## Contents

| File | Description |
|------|-------------|
| `mod.rs` | Module root, re-exports public API |
| `launcher.rs` | `ProcessLauncher` / `BinaryLaunchConfig` - Spawns detached binaries with stdout/stderr capture and health polling |
| `manager.rs` | `ProcessManager` - High-level orchestrator for supported inference runtimes |

## Design Decisions

- **Managed-process scope**: PID files are authoritative for processes launched
  by Pumas; external processes are observed through provider health APIs.
- **Detached process spawning**: Processes are launched in their own process group
  (`setsid` on Unix, `CREATE_NEW_PROCESS_GROUP` on Windows) so they survive launcher restarts.
- **Policy-free launch helpers**: Provider/profile policy is owned by higher
  runtime services. This module accepts explicit launch config, PID paths,
  environment, and health URLs; it does not decide model routes, provider
  capabilities, or CPU/GPU placement.

## Dependencies

### Internal
- `crate::platform` - Cross-platform process termination and cmdline scanning
- `crate::system` - `ResourceTracker` for per-process CPU/RAM/GPU monitoring
- `crate::error` - `PumasError` / `Result`

## Runtime Profile Boundary

Managed local runtime profiles may use this module for profile-scoped process
spawn/stop mechanics, but profile identity and provider-specific launch
arguments are derived before this boundary. Broad singleton cleanup remains a
  app-level behavior and must not be reused for profile-scoped stop operations.

### External
- `sysinfo` - Process table access
