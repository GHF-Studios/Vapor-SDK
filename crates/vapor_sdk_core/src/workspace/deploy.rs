use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::GlobalOptions;

use super::cargo::VaporCargo;
use super::error::WorkspaceCommandError;
use super::identity::{WorkspaceIdentity, discover_workspace_identity};
use super::report::WorkspaceDeployReport;

const DEV_ARTIFACT_DIR: &str = "debug";

pub(super) fn workspace_deploy(
    globals: &GlobalOptions,
) -> Result<WorkspaceDeployReport, WorkspaceCommandError> {
    let identity = discover_workspace_identity(globals)?;
    let target = deploy_target(&identity)?;

    let cargo = VaporCargo::new(globals)?;
    let build = cargo.run(&["build", "-p", target.package])?;

    if !build.status.success() {
        return Err(WorkspaceCommandError::BuildFailedBeforeDeploy(build.status));
    }

    let executable_name = executable_name(target.package);
    let source_executable = cargo
        .target_dir
        .join(DEV_ARTIFACT_DIR)
        .join(&executable_name);
    let deployed_executable = cargo.toolchain.vapor_home.join("bin").join(executable_name);
    let alias_executable = cargo.toolchain.vapor_home.join(alias_name(target.alias));
    let activation_script = cargo.toolchain.vapor_home.join(activation_script_name());

    promote_file(&source_executable, &deployed_executable)?;
    promote_alias(&deployed_executable, &alias_executable)?;
    write_activation_script(&cargo.toolchain.vapor_home, &activation_script)?;

    Ok(WorkspaceDeployReport {
        build,
        source_executable,
        deployed_executable,
        alias_executable,
        activation_script,
    })
}

#[derive(Debug, Clone, Copy)]
struct DeployTarget {
    package: &'static str,
    alias: &'static str,
}

fn deploy_target(identity: &WorkspaceIdentity) -> Result<DeployTarget, WorkspaceCommandError> {
    match identity.kind.as_deref() {
        Some("sdk") => Ok(DeployTarget {
            package: "vapor_sdk_cli",
            alias: "sdk_cli",
        }),
        Some("launcher") => Ok(DeployTarget {
            package: "vapor_launcher_cli",
            alias: "launcher_cli",
        }),
        _ => Err(WorkspaceCommandError::WrongWorkspaceKind {
            expected: "sdk or launcher".to_owned(),
            actual: identity.kind.clone(),
        }),
    }
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

fn promote_alias(target: &Path, alias: &Path) -> Result<(), WorkspaceCommandError> {
    if let Some(parent) = alias.parent() {
        fs::create_dir_all(parent)?;
    }

    if fs::symlink_metadata(alias).is_ok() {
        fs::remove_file(alias)?;
    }

    remove_legacy_bin_alias(target, alias)?;
    create_platform_alias(target, alias)
}

fn remove_legacy_bin_alias(target: &Path, alias: &Path) -> Result<(), WorkspaceCommandError> {
    let Some(target_parent) = target.parent() else {
        return Ok(());
    };
    let Some(alias_name) = alias.file_name() else {
        return Ok(());
    };
    let legacy_alias = target_parent.join(alias_name);

    if legacy_alias != alias && fs::symlink_metadata(&legacy_alias).is_ok() {
        fs::remove_file(legacy_alias)?;
    }

    Ok(())
}

#[cfg(windows)]
fn create_platform_alias(target: &Path, alias: &Path) -> Result<(), WorkspaceCommandError> {
    let target_path = relative_alias_target(target, alias)?;
    let target_path = target_path.display().to_string().replace('/', "\\");
    let content = format!("@echo off\r\n\"%~dp0{target_path}\" %*\r\n");
    fs::write(alias, content)?;
    Ok(())
}

#[cfg(not(windows))]
fn create_platform_alias(target: &Path, alias: &Path) -> Result<(), WorkspaceCommandError> {
    let target_path = relative_alias_target(target, alias)?;
    let content = format!(
        "#!/usr/bin/env sh\nSCRIPT_DIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nexec \"$SCRIPT_DIR/{}\" \"$@\"\n",
        target_path.display()
    );
    fs::write(alias, content)?;

    let mut permissions = fs::metadata(alias)?.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(alias, permissions)?;
    Ok(())
}

fn relative_alias_target(target: &Path, alias: &Path) -> Result<PathBuf, WorkspaceCommandError> {
    let alias_parent = alias
        .parent()
        .ok_or_else(|| WorkspaceCommandError::ExecutableHasNoFileName(alias.to_path_buf()))?;

    target
        .strip_prefix(alias_parent)
        .map(Path::to_path_buf)
        .map_err(|_| WorkspaceCommandError::ExecutableHasNoFileName(target.to_path_buf()))
}

fn alias_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.cmd")
    } else {
        stem.to_owned()
    }
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

fn activation_script_name() -> &'static str {
    if cfg!(windows) {
        "vapor_env.cmd"
    } else {
        "vapor_env.sh"
    }
}

#[cfg(windows)]
fn write_activation_script(
    vapor_home: &Path,
    destination: &Path,
) -> Result<(), WorkspaceCommandError> {
    let content = format!(
        "@echo off\r\nset \"VAPOR_HOME={}\"\r\nset \"CARGO_HOME=%VAPOR_HOME%\\cargo-home\"\r\nset \"RUSTUP_HOME=%VAPOR_HOME%\\rustup-home\"\r\nset \"VAPOR_STEAM_HOME=%VAPOR_HOME%\\steam\"\r\nset \"PATH=%VAPOR_HOME%;%VAPOR_HOME%\\bin;%VAPOR_HOME%\\rust-toolchain\\active\\bin;%VAPOR_HOME%\\rustup\\bin;%VAPOR_HOME%\\steam\\steamcmd;%PATH%\"\r\n",
        vapor_home.display()
    );
    fs::write(destination, content)?;
    Ok(())
}

#[cfg(not(windows))]
fn write_activation_script(
    _vapor_home: &Path,
    destination: &Path,
) -> Result<(), WorkspaceCommandError> {
    let content = r#"# Source this file from the Vapor app root: . ./vapor_env.sh
VAPOR_HOME=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE:-$0}")" && pwd)
export VAPOR_HOME
export CARGO_HOME="$VAPOR_HOME/cargo-home"
export RUSTUP_HOME="$VAPOR_HOME/rustup-home"
export VAPOR_STEAM_HOME="$VAPOR_HOME/steam"

vapor_prepend_path() {
    case ":$PATH:" in
        *":$1:"*) ;;
        *) PATH="$1${PATH:+:$PATH}" ;;
    esac
}

vapor_prepend_path "$VAPOR_HOME/steam/steamcmd"
vapor_prepend_path "$VAPOR_HOME/rustup/bin"
vapor_prepend_path "$VAPOR_HOME/rust-toolchain/active/bin"
vapor_prepend_path "$VAPOR_HOME/bin"
vapor_prepend_path "$VAPOR_HOME"
export PATH
unset -f vapor_prepend_path
"#;
    fs::write(destination, content)?;

    let mut permissions = fs::metadata(destination)?.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    fs::set_permissions(destination, permissions)?;
    Ok(())
}
