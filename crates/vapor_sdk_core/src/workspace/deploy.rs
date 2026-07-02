use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::cargo::VaporCargo;
use super::error::WorkspaceCommandError;
use super::identity::require_current_repo_kind;
use super::report::WorkspaceDeployReport;

const DEV_ARTIFACT_DIR: &str = "debug";
const SDK_CLI_PACKAGE: &str = "vapor_sdk_cli";
const SDK_REPO_KIND: &str = "sdk";

pub(super) fn workspace_deploy() -> Result<WorkspaceDeployReport, WorkspaceCommandError> {
    require_current_repo_kind(SDK_REPO_KIND)?;

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
