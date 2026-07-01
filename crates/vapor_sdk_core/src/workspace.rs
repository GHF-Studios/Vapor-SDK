//! SDK workspace workflows backed by the Vapor-managed Cargo binary.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::toolchain::{
    ToolchainInstallState, ToolchainStatus, ToolchainStatusError, toolchain_status,
};

const CARGO_HOME_DIR: &str = "cargo";
const CARGO_TARGET_DIR: &str = "cargo-target";
const DEV_ARTIFACT_DIR: &str = "debug";
const SDK_CLI_PACKAGE: &str = "vapor_sdk_cli";

/// Commands that operate on the current authoring workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// Run `cargo check` through the Vapor-managed Cargo binary.
    Check,
    /// Run `cargo fmt` through the Vapor-managed Cargo binary.
    Fmt,
    /// Run `cargo build --workspace` through the Vapor-managed Cargo binary.
    Build,
    /// Build and promote the SDK CLI into the executable-root `bin` directory.
    Deploy,
}

/// Result of running one Vapor-managed Cargo command.
#[derive(Debug, Clone)]
pub struct WorkspaceCargoReport {
    pub working_directory: PathBuf,
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub cargo_home: PathBuf,
    pub target_dir: PathBuf,
    pub cargo_args: Vec<String>,
    pub status: ExitStatus,
}

/// Result of promoting a built SDK executable into the deploy root.
#[derive(Debug, Clone)]
pub struct WorkspaceDeployReport {
    pub build: WorkspaceCargoReport,
    pub source_executable: PathBuf,
    pub deployed_executable: PathBuf,
}

/// Run `cargo check --workspace` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_check() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    VaporCargo::new()?.run(&["check", "--workspace"])
}

/// Run `cargo fmt` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_fmt() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    VaporCargo::new()?.run(&["fmt"])
}

/// Run `cargo build --workspace` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_build() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    VaporCargo::new()?.run(&["build", "--workspace"])
}

/// Build and promote `vapor_sdk_cli` into `$VAPOR_HOME/bin`.
pub fn workspace_deploy() -> Result<WorkspaceDeployReport, WorkspaceCommandError> {
    let cargo = VaporCargo::new()?;
    let build = cargo.run(&["build", "-p", SDK_CLI_PACKAGE])?;

    if !build.status.success() {
        return Err(WorkspaceCommandError::BuildFailedBeforeDeploy(build.status));
    }

    let executable_name = executable_name(SDK_CLI_PACKAGE);
    let source_executable = cargo
        .target_dir
        .join(DEV_ARTIFACT_DIR)
        .join(&executable_name);
    let deployed_executable = cargo.toolchain.vapor_home.join("bin").join(executable_name);

    promote_file(&source_executable, &deployed_executable)?;

    Ok(WorkspaceDeployReport {
        build,
        source_executable,
        deployed_executable,
    })
}

#[derive(Debug, Clone)]
struct VaporCargo {
    toolchain: ToolchainStatus,
    cargo_home: PathBuf,
    target_dir: PathBuf,
}

impl VaporCargo {
    fn new() -> Result<Self, WorkspaceCommandError> {
        let toolchain = checked_toolchain_status()?;
        let cargo_home = toolchain.vapor_home.join(CARGO_HOME_DIR);
        let target_dir = toolchain.output_root.join(CARGO_TARGET_DIR);

        Ok(Self {
            toolchain,
            cargo_home,
            target_dir,
        })
    }

    fn run(&self, args: &[&str]) -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
        let working_directory = env::current_dir()?;
        let mut command = Command::new(&self.toolchain.cargo_path);
        command
            .args(args)
            .current_dir(&working_directory)
            .env("CARGO_HOME", &self.cargo_home)
            .env("CARGO_TARGET_DIR", &self.target_dir)
            .env("RUSTC", &self.toolchain.rustc_path)
            .env_remove("RUSTUP_HOME")
            .env_remove("RUSTUP_TOOLCHAIN")
            .env_remove("RUSTC_WRAPPER")
            .env("PATH", vapor_path_prefix(&self.toolchain.cargo_path)?);

        let status = command.status()?;

        Ok(WorkspaceCargoReport {
            working_directory,
            cargo_path: self.toolchain.cargo_path.clone(),
            rustc_path: self.toolchain.rustc_path.clone(),
            cargo_home: self.cargo_home.clone(),
            target_dir: self.target_dir.clone(),
            cargo_args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            status,
        })
    }
}

fn checked_toolchain_status() -> Result<ToolchainStatus, WorkspaceCommandError> {
    let toolchain = toolchain_status()?;

    if !toolchain.host_supported {
        return Err(WorkspaceCommandError::UnsupportedHost(
            toolchain.host_triple.to_owned(),
        ));
    }

    if !matches!(
        toolchain.install_state,
        ToolchainInstallState::PresentUnverified
    ) {
        return Err(WorkspaceCommandError::ToolchainNotInstalled(
            toolchain.install_state,
        ));
    }

    Ok(toolchain)
}

fn vapor_path_prefix(cargo_path: &PathBuf) -> Result<OsString, WorkspaceCommandError> {
    let bin_dir = cargo_path
        .parent()
        .ok_or_else(|| WorkspaceCommandError::CargoPathHasNoParent(cargo_path.clone()))?;
    let mut paths = vec![bin_dir.to_path_buf()];

    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    Ok(env::join_paths(paths)?)
}

fn promote_file(source: &Path, destination: &Path) -> Result<(), WorkspaceCommandError> {
    if !source.is_file() {
        return Err(WorkspaceCommandError::MissingBuiltExecutable(
            source.to_path_buf(),
        ));
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let temporary = temporary_destination(destination)?;
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }

    fs::copy(source, &temporary)?;

    if destination.exists() {
        fs::remove_file(destination)?;
    }

    fs::rename(temporary, destination)?;
    Ok(())
}

fn temporary_destination(destination: &Path) -> Result<PathBuf, WorkspaceCommandError> {
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| WorkspaceCommandError::ExecutableHasNoFileName(destination.to_path_buf()))?;

    Ok(destination.with_file_name(format!("{file_name}.tmp-{}", std::process::id())))
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

/// Error returned while running a Vapor-managed workspace command.
#[derive(Debug)]
pub enum WorkspaceCommandError {
    ToolchainStatus(ToolchainStatusError),
    Io(std::io::Error),
    JoinPaths(env::JoinPathsError),
    UnsupportedHost(String),
    ToolchainNotInstalled(ToolchainInstallState),
    CargoPathHasNoParent(PathBuf),
    BuildFailedBeforeDeploy(ExitStatus),
    MissingBuiltExecutable(PathBuf),
    ExecutableHasNoFileName(PathBuf),
}

impl From<ToolchainStatusError> for WorkspaceCommandError {
    fn from(error: ToolchainStatusError) -> Self {
        Self::ToolchainStatus(error)
    }
}

impl From<std::io::Error> for WorkspaceCommandError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<env::JoinPathsError> for WorkspaceCommandError {
    fn from(error: env::JoinPathsError) -> Self {
        Self::JoinPaths(error)
    }
}

impl fmt::Display for WorkspaceCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainStatus(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "workspace command failed: {error}"),
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
            Self::BuildFailedBeforeDeploy(status) => write!(
                formatter,
                "Vapor-managed cargo build failed before deploy with {status}"
            ),
            Self::MissingBuiltExecutable(path) => {
                write!(formatter, "missing built executable `{}`", path.display())
            }
            Self::ExecutableHasNoFileName(path) => write!(
                formatter,
                "deployed executable path has no file name: `{}`",
                path.display()
            ),
        }
    }
}

impl Error for WorkspaceCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ToolchainStatus(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::JoinPaths(error) => Some(error),
            Self::UnsupportedHost(_)
            | Self::ToolchainNotInstalled(_)
            | Self::CargoPathHasNoParent(_)
            | Self::BuildFailedBeforeDeploy(_)
            | Self::MissingBuiltExecutable(_)
            | Self::ExecutableHasNoFileName(_) => None,
        }
    }
}
