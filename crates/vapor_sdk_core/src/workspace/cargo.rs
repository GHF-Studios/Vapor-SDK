use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::toolchain::{ToolchainInstallState, ToolchainStatus, toolchain_status};

use super::error::WorkspaceCommandError;
use super::identity::{WorkspaceIdentity, discover_workspace_identity};
use super::report::WorkspaceCargoReport;

const CARGO_HOME_DIR: &str = "cargo";
const CARGO_TARGET_DIR: &str = "cargo-target";

#[derive(Debug, Clone)]
pub(super) struct VaporCargo {
    pub(super) toolchain: ToolchainStatus,
    pub(super) target_dir: PathBuf,
    pub(super) identity: WorkspaceIdentity,
    cargo_home: PathBuf,
}

impl VaporCargo {
    pub(super) fn new() -> Result<Self, WorkspaceCommandError> {
        let toolchain = checked_toolchain_status()?;
        let identity = discover_workspace_identity()?;
        let cargo_home = toolchain.vapor_home.join(CARGO_HOME_DIR);
        let target_dir = toolchain.output_root.join(CARGO_TARGET_DIR);

        Ok(Self {
            toolchain,
            identity,
            cargo_home,
            target_dir,
        })
    }

    pub(super) fn run(&self, args: &[&str]) -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
        let mut command = Command::new(&self.toolchain.cargo_path);
        command
            .args(args)
            .current_dir(&self.identity.workspace_root)
            .env("CARGO_HOME", &self.cargo_home)
            .env("CARGO_TARGET_DIR", &self.target_dir)
            .env("RUSTC", &self.toolchain.rustc_path)
            .env_remove("RUSTUP_HOME")
            .env_remove("RUSTUP_TOOLCHAIN")
            .env_remove("RUSTC_WRAPPER")
            .env("PATH", vapor_path_prefix(&self.toolchain.cargo_path)?);

        let status = command.status()?;

        Ok(WorkspaceCargoReport {
            invocation_directory: self.identity.invocation_directory.clone(),
            workspace_root: self.identity.workspace_root.clone(),
            workspace_kind: self.identity.kind.clone(),
            workspace_id: self.identity.id.clone(),
            working_directory: self.identity.workspace_root.clone(),
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
