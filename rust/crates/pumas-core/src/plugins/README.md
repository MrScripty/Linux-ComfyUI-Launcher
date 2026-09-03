# Plugins

## Purpose

JSON-based inference-plugin configuration. Each descriptor specifies runtime
metadata, version-management capabilities, connection settings, API endpoints,
and UI panel layout.

## Contents

| File | Description |
|------|-------------|
| `mod.rs` | Module root, re-exports public API |
| `loader.rs` | `PluginLoader` - Discovers and loads plugin JSON files from the plugins directory |
| `schema.rs` | `PluginConfig`, `AppCapabilities`, `ConnectionConfig`, `ApiEndpoint`, and related types |

## Design Decisions

- **JSON over Rust code for runtime definitions**: Supported inference runtimes
  can be described in the plugins directory without hard-coding UI metadata.
- **Capability flags**: `AppCapabilities` uses boolean flags (e.g., `has_version_management`,
  `has_process_management`) so the frontend can conditionally render runtime UI.

## Dependencies

### Internal
- `crate::error` - `PumasError` / `Result`

### External
- `serde` / `serde_json` - Plugin JSON deserialization
