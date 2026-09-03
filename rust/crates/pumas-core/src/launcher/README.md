# Launcher

## Purpose

Launcher self-management: checking for and applying updates via git.

## Contents

| File | Description |
|------|-------------|
| `mod.rs` | Module root, re-exports public API |
| `updater.rs` | `LauncherUpdater` - Git-based self-update: fetch, compare commits, pull changes |

## Design Decisions

- **Git-based updates**: The launcher updates itself by pulling from its git remote, keeping
  the update mechanism simple and leveraging git's merge/conflict handling. Update checks
  compare local vs remote HEAD commit SHAs.

## Dependencies

### Internal
- `crate::error` - `PumasError` / `Result`
- `crate::models` - `CommitInfo` for update check results

### External
- `std::process::Command` - Git subprocess execution
