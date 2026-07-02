//! Vapor app-root shell environment reporting.
//!
//! The SDK executes Rust/Cargo through explicit paths and process-local
//! environment variables. This module reports the equivalent app-root shell
//! setup for humans and future GUI frontends.

use std::path::PathBuf;

use crate::toolchain::{ToolchainStatusError, toolchain_status};

const CARGO_HOME_DIR: &str = "cargo-home";
const RUSTUP_HOME_DIR: &str = "rustup-home";
const STEAM_HOME_DIR: &str = "steam";
const STEAMCMD_DIR: &str = "steamcmd";
const RUST_TOOLCHAIN_ACTIVE_BIN: &str = "rust-toolchain/active/bin";
const RUSTUP_BIN: &str = "rustup/bin";
const BIN_DIR: &str = "bin";

/// Environment commands for app-root shell/tool discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentCommand {
    /// Show the app-root shell environment Vapor expects humans to activate.
    Status,
}

/// Report for Vapor's app-root shell environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentReport {
    pub vapor_home: PathBuf,
    pub activation_script: PathBuf,
    pub cargo_home: PathBuf,
    pub rustup_home: PathBuf,
    pub steam_home: PathBuf,
    pub path_entries: Vec<PathBuf>,
}

/// Inspect the app-root shell environment without mutating local state.
pub fn environment_status() -> Result<EnvironmentReport, ToolchainStatusError> {
    let toolchain = toolchain_status()?;
    let vapor_home = toolchain.vapor_home;

    Ok(EnvironmentReport {
        activation_script: vapor_home.join(activation_script_name()),
        cargo_home: vapor_home.join(CARGO_HOME_DIR),
        rustup_home: vapor_home.join(RUSTUP_HOME_DIR),
        steam_home: vapor_home.join(STEAM_HOME_DIR),
        path_entries: vec![
            vapor_home.clone(),
            vapor_home.join(BIN_DIR),
            vapor_home.join(RUST_TOOLCHAIN_ACTIVE_BIN),
            vapor_home.join(RUSTUP_BIN),
            vapor_home.join(STEAM_HOME_DIR).join(STEAMCMD_DIR),
        ],
        vapor_home,
    })
}

fn activation_script_name() -> &'static str {
    if cfg!(windows) {
        "vapor_env.cmd"
    } else {
        "vapor_env.sh"
    }
}
