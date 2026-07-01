//! SDK workspace workflows backed by the Vapor-managed Cargo binary.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use crate::toolchain::{ToolchainInstallState, ToolchainStatusError, toolchain_status};

/// Commands that operate on the current authoring workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// Run `cargo check` through the Vapor-managed Cargo binary.
    Check,
}

/// Result of running a Vapor-managed workspace check.
#[derive(Debug, Clone)]
pub struct WorkspaceCheckReport {
    pub working_directory: PathBuf,
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub status: ExitStatus,
}

/// Run `cargo check` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_check() -> Result<WorkspaceCheckReport, WorkspaceCheckError> {
    let toolchain = toolchain_status()?;

    if !toolchain.host_supported {
        return Err(WorkspaceCheckError::UnsupportedHost(
            toolchain.host_triple.to_owned(),
        ));
    }

    if !matches!(
        toolchain.install_state,
        ToolchainInstallState::PresentUnverified
    ) {
        return Err(WorkspaceCheckError::ToolchainNotInstalled(
            toolchain.install_state,
        ));
    }

    let working_directory = env::current_dir()?;
    let mut command = Command::new(&toolchain.cargo_path);
    command
        .arg("check")
        .current_dir(&working_directory)
        .env("CARGO_HOME", toolchain.vapor_home.join("cargo"))
        .env("RUSTC", &toolchain.rustc_path)
        .env_remove("RUSTUP_HOME")
        .env("PATH", vapor_path_prefix(&toolchain.cargo_path)?);

    let status = command.status()?;

    Ok(WorkspaceCheckReport {
        working_directory,
        cargo_path: toolchain.cargo_path,
        rustc_path: toolchain.rustc_path,
        status,
    })
}

fn vapor_path_prefix(cargo_path: &PathBuf) -> Result<OsString, WorkspaceCheckError> {
    let bin_dir = cargo_path
        .parent()
        .ok_or_else(|| WorkspaceCheckError::CargoPathHasNoParent(cargo_path.clone()))?;
    let mut paths = vec![bin_dir.to_path_buf()];

    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    Ok(env::join_paths(paths)?)
}

/// Error returned while running a Vapor-managed workspace check.
#[derive(Debug)]
pub enum WorkspaceCheckError {
    ToolchainStatus(ToolchainStatusError),
    Io(std::io::Error),
    JoinPaths(env::JoinPathsError),
    UnsupportedHost(String),
    ToolchainNotInstalled(ToolchainInstallState),
    CargoPathHasNoParent(PathBuf),
}

impl From<ToolchainStatusError> for WorkspaceCheckError {
    fn from(error: ToolchainStatusError) -> Self {
        Self::ToolchainStatus(error)
    }
}

impl From<std::io::Error> for WorkspaceCheckError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<env::JoinPathsError> for WorkspaceCheckError {
    fn from(error: env::JoinPathsError) -> Self {
        Self::JoinPaths(error)
    }
}

impl fmt::Display for WorkspaceCheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainStatus(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "workspace check failed: {error}"),
            Self::JoinPaths(error) => write!(formatter, "failed to build Vapor PATH: {error}"),
            Self::UnsupportedHost(host) => write!(
                formatter,
                "host triple `{host}` is not supported by this Vapor toolchain pin"
            ),
            Self::ToolchainNotInstalled(state) => write!(
                formatter,
                "Vapor toolchain is not installed: {}",
                state.as_str()
            ),
            Self::CargoPathHasNoParent(path) => write!(
                formatter,
                "Vapor cargo path has no parent directory: `{}`",
                path.display()
            ),
        }
    }
}

impl Error for WorkspaceCheckError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ToolchainStatus(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::JoinPaths(error) => Some(error),
            Self::UnsupportedHost(_)
            | Self::ToolchainNotInstalled(_)
            | Self::CargoPathHasNoParent(_) => None,
        }
    }
}
