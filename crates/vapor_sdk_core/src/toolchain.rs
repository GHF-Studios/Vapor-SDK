//! SDK-managed toolchain command intent and local status reporting.

mod dist;
mod install;
mod plan;

use std::env;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use vapor_core::{CanonicalToolchain, ManifestError, canonical_toolchain, current_host_triple};

pub use dist::DistError;
pub use install::{ToolchainInstallError, ToolchainInstallReport, toolchain_install};

pub use plan::{
    ToolchainArchivePlan, ToolchainInstallPlan, ToolchainPlanError, toolchain_install_plan,
};

/// Environment variable that overrides where Vapor stores executable-local state.
pub const VAPOR_HOME_ENV: &str = "VAPOR_HOME";

/// Directory under `VAPOR_HOME` that contains the single active Vapor toolchain.
pub const TOOLCHAIN_DIR: &str = "toolchain";

/// Directory under `VAPOR_HOME/toolchain` that contains the active Rust/Cargo tree.
pub const ACTIVE_TOOLCHAIN_DIR: &str = "active";

/// Directory under `VAPOR_HOME/toolchain` used only while bootstrapping a toolchain.
pub const TOOLCHAIN_BOOTSTRAP_DIR: &str = "bootstrap";

/// Directory under `VAPOR_HOME/toolchain/bootstrap` for official Rust archives.
pub const BOOTSTRAP_DOWNLOADS_DIR: &str = "downloads";

/// Directory under `VAPOR_HOME/toolchain/bootstrap` for temporary assembly roots.
pub const BOOTSTRAP_STAGING_DIR: &str = "staging";

/// Directory under `VAPOR_HOME` where builds are promoted for testing/packaging.
pub const DEPLOY_DIR: &str = "deploy";

/// Toolchain commands for the pinned SDK-managed Rust/Cargo toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolchainCommand {
    /// Inspect pinned toolchain state.
    Status,
    /// Install the pinned toolchain chosen by the project owner.
    Install,
    /// Repair a damaged or incomplete pinned toolchain installation.
    Repair,
}

/// Current local state for the canonical Vapor Rust/Cargo toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainStatus {
    pub toolchain: CanonicalToolchain,
    pub host_triple: &'static str,
    pub host_supported: bool,
    pub vapor_home_source: VaporHomeSource,
    pub vapor_home: PathBuf,
    /// Root for all active and bootstrap toolchain state.
    pub toolchain_home: PathBuf,
    /// Single active Rust/Cargo toolchain root for this Vapor install.
    pub toolchain_root: PathBuf,
    /// Toolchain-only bootstrap area for downloads and staging.
    pub bootstrap_root: PathBuf,
    /// Stable output root for future build/package promotion.
    pub deploy_root: PathBuf,
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub install_state: ToolchainInstallState,
}

impl ToolchainStatus {
    /// Supported host triples inferred from the canonical toolchain model.
    pub fn supported_host_triples(&self) -> &'static [&'static str] {
        self.toolchain.supported_host_triples()
    }

    /// Target triples inferred from the canonical toolchain model.
    pub fn supported_target_triples(&self) -> &'static [&'static str] {
        self.toolchain.supported_target_triples()
    }
}

/// Where `VAPOR_HOME` came from for this process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporHomeSource {
    Environment,
    ExecutableRoot,
}

impl VaporHomeSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Environment => VAPOR_HOME_ENV,
            Self::ExecutableRoot => "executable root",
        }
    }
}

/// Coarse local install state before real archive verification exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolchainInstallState {
    Missing,
    PresentUnverified,
    Broken { reason: String },
}

impl ToolchainInstallState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::PresentUnverified => "present_unverified",
            Self::Broken { .. } => "broken",
        }
    }
}

/// Inspect the canonical Vapor toolchain without mutating local state.
pub fn toolchain_status() -> Result<ToolchainStatus, ToolchainStatusError> {
    let toolchain = canonical_toolchain()?;
    let host_triple = current_host_triple();
    let (vapor_home_source, vapor_home) = vapor_home()?;
    let toolchain_home = vapor_home.join(TOOLCHAIN_DIR);
    let toolchain_root = toolchain_home.join(ACTIVE_TOOLCHAIN_DIR);
    let bootstrap_root = toolchain_home.join(TOOLCHAIN_BOOTSTRAP_DIR);
    let deploy_root = vapor_home.join(DEPLOY_DIR);
    let cargo_path = toolchain_root.join("bin").join(executable_name("cargo"));
    let rustc_path = toolchain_root.join("bin").join(executable_name("rustc"));
    let install_state = inspect_install_state(&toolchain_root, &cargo_path, &rustc_path);

    Ok(ToolchainStatus {
        host_supported: toolchain.supports_host(host_triple),
        toolchain,
        host_triple,
        vapor_home_source,
        vapor_home,
        toolchain_home,
        toolchain_root,
        bootstrap_root,
        deploy_root,
        cargo_path,
        rustc_path,
        install_state,
    })
}

fn vapor_home() -> Result<(VaporHomeSource, PathBuf), ToolchainStatusError> {
    if let Some(value) = env::var_os(VAPOR_HOME_ENV).filter(|value| !value.is_empty()) {
        return Ok((VaporHomeSource::Environment, PathBuf::from(value)));
    }

    let executable = env::current_exe().map_err(ToolchainStatusError::CurrentExecutable)?;
    let executable_dir = executable
        .parent()
        .ok_or(ToolchainStatusError::ExecutableHasNoParent)?;

    if executable_dir.file_name().is_some_and(|name| name == "bin") {
        let root = executable_dir
            .parent()
            .ok_or(ToolchainStatusError::ExecutableHasNoParent)?;
        Ok((VaporHomeSource::ExecutableRoot, root.to_path_buf()))
    } else {
        Ok((
            VaporHomeSource::ExecutableRoot,
            executable_dir.to_path_buf(),
        ))
    }
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

fn inspect_install_state(
    toolchain_root: &Path,
    cargo_path: &Path,
    rustc_path: &Path,
) -> ToolchainInstallState {
    if !toolchain_root.exists() {
        return ToolchainInstallState::Missing;
    }

    let mut missing = Vec::new();
    if !cargo_path.exists() {
        missing.push("cargo");
    }
    if !rustc_path.exists() {
        missing.push("rustc");
    }

    if missing.is_empty() {
        ToolchainInstallState::PresentUnverified
    } else {
        ToolchainInstallState::Broken {
            reason: format!("missing {}", missing.join(" and ")),
        }
    }
}

/// Error returned while inspecting local Vapor toolchain state.
#[derive(Debug)]
pub enum ToolchainStatusError {
    Manifest(ManifestError),
    CurrentExecutable(std::io::Error),
    ExecutableHasNoParent,
}

impl From<ManifestError> for ToolchainStatusError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl fmt::Display for ToolchainStatusError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "{error}"),
            Self::CurrentExecutable(error) => {
                write!(formatter, "failed to locate current executable: {error}")
            }
            Self::ExecutableHasNoParent => {
                write!(formatter, "current executable has no parent directory")
            }
        }
    }
}

impl Error for ToolchainStatusError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Manifest(error) => Some(error),
            Self::CurrentExecutable(error) => Some(error),
            Self::ExecutableHasNoParent => None,
        }
    }
}
