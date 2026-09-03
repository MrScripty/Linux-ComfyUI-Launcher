//! Launcher management module.
//!
//! Provides functionality for:
//! - Launcher self-updates via git
//! - System binary detection

mod updater;

pub use updater::{LauncherUpdater, UpdateApplyResult, UpdateCheckResult};
