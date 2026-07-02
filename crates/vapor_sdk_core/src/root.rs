//! Root Vapor package and SteamPipe publishing workflows.
//!
//! Root publishing is intentionally about the Steam-distributed Vapor app root,
//! not about arbitrary authored content. It packages the executable-local SDK
//! surface that Steam should redistribute, then can hand that package to
//! SteamCMD with a generated SteamPipe app build script. The installed Rust
//! toolchain is not packaged here; Vapor should wrap/install Rust tooling
//! deliberately instead of shipping the active toolchain tree as app content.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crate::steam::{
    SteamCommandError, SteamRunAppBuildRequest, steam_run_app_build, steam_run_app_build_plan,
};
use crate::toolchain::{ToolchainStatusError, toolchain_status};

/// Steam AppID for the first-party Loo Cast/Vapor root application.
pub const ROOT_STEAM_APP_ID: u32 = 2_122_620;

/// Default Steam DepotID for the first-party root application files.
pub const ROOT_STEAM_DEPOT_ID: u32 = 2_122_621;

const ROOT_OUTPUT_DIR: &str = "root";
const CONTENT_DIR: &str = "content";
const SCRIPTS_DIR: &str = "scripts";
const STEAM_BUILD_DIR: &str = "steam-build";
const BIN_DIR: &str = "bin";
const SDK_CLI_ALIAS: &str = "sdk_cli";
const SDK_CLI_BINARY: &str = "vapor_sdk_cli";
const LAUNCHER_CLI_ALIAS: &str = "launcher_cli";
const LAUNCHER_CLI_BINARY: &str = "vapor_launcher_cli";
const ACTIVATION_SCRIPT_UNIX: &str = "vapor_env.sh";
const ACTIVATION_SCRIPT_WINDOWS: &str = "vapor_env.cmd";

/// Root SDK workflows for the Steam-distributed app itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootCommand {
    /// Assemble the Steam redistributable root package and SteamPipe scripts.
    Package(RootPackageRequest),
    /// Assemble the root package and invoke SteamCMD with `+run_app_build`.
    Publish(RootPublishRequest),
}

/// Parameters for root packaging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootPackageRequest {
    /// Report what would be packaged without mutating the package output.
    pub plan: bool,
    /// Steam AppID receiving the root package.
    pub app_id: u32,
    /// Steam DepotID receiving the root package files.
    pub depot_id: u32,
    /// Internal Steamworks build description.
    pub description: String,
    /// Optional beta branch to set live after upload. Steam does not allow
    /// automatically setting the default branch live from the build script.
    pub set_live: Option<String>,
}

/// Parameters for root publishing through SteamCMD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootPublishRequest {
    /// Report what would be packaged and uploaded without mutating or invoking SteamCMD.
    pub plan: bool,
    /// Steam AppID receiving the root package.
    pub app_id: u32,
    /// Steam DepotID receiving the root package files.
    pub depot_id: u32,
    /// Internal Steamworks build description.
    pub description: String,
    /// Steam account used by SteamCMD. SteamCMD prompts for password/Steam Guard
    /// when the local login token is missing.
    pub account: String,
    /// SteamCMD executable path, or `steamcmd` to resolve through PATH.
    pub steamcmd: PathBuf,
    /// Optional beta branch to set live after upload. Steam does not allow
    /// automatically setting the default branch live from the build script.
    pub set_live: Option<String>,
}

/// Report for root package assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootPackageReport {
    pub planned: bool,
    pub app_id: u32,
    pub depot_id: u32,
    pub host_triple: String,
    pub description: String,
    pub set_live: Option<String>,
    pub vapor_home: PathBuf,
    pub package_root: PathBuf,
    pub content_root: PathBuf,
    pub scripts_root: PathBuf,
    pub steam_build_root: PathBuf,
    pub app_build_script: PathBuf,
    pub included_roots: Vec<PathBuf>,
    pub copied_files: usize,
}

/// Report for a SteamCMD root publish attempt.
#[derive(Debug, Clone)]
pub struct RootPublishReport {
    pub planned: bool,
    pub package: RootPackageReport,
    pub steamcmd: PathBuf,
    pub account: String,
    pub steam_args: Vec<String>,
    pub status: Option<ExitStatus>,
    pub set_live: Option<String>,
}

/// Assemble the Steam redistributable root package.
pub fn root_package(request: &RootPackageRequest) -> Result<RootPackageReport, RootCommandError> {
    let layout = RootPackageLayout::new(request)?;
    layout.package(request)
}

/// Assemble the root package and publish it through SteamCMD.
pub fn root_publish(request: &RootPublishRequest) -> Result<RootPublishReport, RootCommandError> {
    validate_set_live(request.set_live.as_deref())?;

    let package_request = RootPackageRequest {
        plan: request.plan,
        app_id: request.app_id,
        depot_id: request.depot_id,
        description: request.description.clone(),
        set_live: request.set_live.clone(),
    };
    let package = root_package(&package_request)?;
    let steam_request = SteamRunAppBuildRequest {
        account: request.account.clone(),
        steamcmd: request.steamcmd.clone(),
        app_build_script: package.app_build_script.clone(),
    };

    if request.plan {
        let (steam_status, steam_args) = steam_run_app_build_plan(&steam_request)?;
        return Ok(RootPublishReport {
            planned: true,
            package,
            steamcmd: steam_status
                .resolved_steamcmd
                .unwrap_or_else(|| request.steamcmd.clone()),
            account: request.account.clone(),
            steam_args,
            status: None,
            set_live: request.set_live.clone(),
        });
    }

    let steam_report = steam_run_app_build(&steam_request)?;

    Ok(RootPublishReport {
        planned: false,
        package,
        steamcmd: steam_report
            .status
            .resolved_steamcmd
            .unwrap_or_else(|| request.steamcmd.clone()),
        account: request.account.clone(),
        steam_args: steam_report.steam_args,
        status: Some(steam_report.exit_status),
        set_live: request.set_live.clone(),
    })
}

fn validate_ids(app_id: u32, depot_id: u32) -> Result<(), RootCommandError> {
    if app_id == 0 {
        return Err(RootCommandError::InvalidAppId(app_id));
    }

    if depot_id == 0 {
        return Err(RootCommandError::InvalidDepotId(depot_id));
    }

    Ok(())
}

fn validate_set_live(set_live: Option<&str>) -> Result<(), RootCommandError> {
    if set_live.is_some_and(|branch| branch == "default") {
        return Err(RootCommandError::DefaultBranchCannotBeSetLive);
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct RootPackageLayout {
    vapor_home: PathBuf,
    host_triple: String,
    package_root: PathBuf,
    content_root: PathBuf,
    scripts_root: PathBuf,
    steam_build_root: PathBuf,
    app_build_script: PathBuf,
    included_roots: Vec<PackageRoot>,
}

impl RootPackageLayout {
    fn new(request: &RootPackageRequest) -> Result<Self, RootCommandError> {
        validate_ids(request.app_id, request.depot_id)?;
        validate_set_live(request.set_live.as_deref())?;

        let status = toolchain_status()?;
        let package_root = status.output_root.join(ROOT_OUTPUT_DIR);
        let content_root = package_root.join(CONTENT_DIR);
        let scripts_root = package_root.join(SCRIPTS_DIR);
        let steam_build_root = package_root.join(STEAM_BUILD_DIR);
        let app_build_script = scripts_root.join(format!("app_build_{}.vdf", request.app_id));
        let included_roots = vec![
            PackageRoot::new(
                status.vapor_home.join(alias_name(SDK_CLI_ALIAS)),
                PathBuf::from(alias_name(SDK_CLI_ALIAS)),
            ),
            PackageRoot::new(
                status.vapor_home.join(alias_name(LAUNCHER_CLI_ALIAS)),
                PathBuf::from(alias_name(LAUNCHER_CLI_ALIAS)),
            ),
            PackageRoot::new(
                status
                    .vapor_home
                    .join(BIN_DIR)
                    .join(executable_name(SDK_CLI_BINARY)),
                PathBuf::from(BIN_DIR).join(executable_name(SDK_CLI_BINARY)),
            ),
            PackageRoot::new(
                status
                    .vapor_home
                    .join(BIN_DIR)
                    .join(executable_name(LAUNCHER_CLI_BINARY)),
                PathBuf::from(BIN_DIR).join(executable_name(LAUNCHER_CLI_BINARY)),
            ),
            PackageRoot::new(
                status.vapor_home.join(activation_script_name()),
                PathBuf::from(activation_script_name()),
            ),
        ];

        for root in &included_roots {
            if !root.source.exists() {
                return Err(RootCommandError::MissingPackageInput(root.source.clone()));
            }
        }

        Ok(Self {
            vapor_home: status.vapor_home,
            host_triple: status.host_triple.to_owned(),
            package_root,
            content_root,
            scripts_root,
            steam_build_root,
            app_build_script,
            included_roots,
        })
    }

    fn package(&self, request: &RootPackageRequest) -> Result<RootPackageReport, RootCommandError> {
        let mut copied_files = 0;

        if !request.plan {
            reset_dir(&self.content_root)?;
            reset_dir(&self.scripts_root)?;
            fs::create_dir_all(&self.steam_build_root)?;
        }

        for root in &self.included_roots {
            let target = self.content_root.join(&root.depot_path);
            copied_files += copy_tree(&root.source, &target, request.plan)?;
        }

        if !request.plan {
            write_app_build_script(
                &self.app_build_script,
                request,
                &self.content_root,
                &self.steam_build_root,
            )?;
        }

        Ok(RootPackageReport {
            planned: request.plan,
            app_id: request.app_id,
            depot_id: request.depot_id,
            host_triple: self.host_triple.clone(),
            description: request.description.clone(),
            set_live: request.set_live.clone(),
            vapor_home: self.vapor_home.clone(),
            package_root: self.package_root.clone(),
            content_root: self.content_root.clone(),
            scripts_root: self.scripts_root.clone(),
            steam_build_root: self.steam_build_root.clone(),
            app_build_script: self.app_build_script.clone(),
            included_roots: self
                .included_roots
                .iter()
                .map(|root| root.depot_path.clone())
                .collect(),
            copied_files,
        })
    }
}

#[derive(Debug, Clone)]
struct PackageRoot {
    source: PathBuf,
    depot_path: PathBuf,
}

impl PackageRoot {
    fn new(source: PathBuf, depot_path: PathBuf) -> Self {
        Self { source, depot_path }
    }
}

fn reset_dir(path: &Path) -> Result<(), RootCommandError> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }

    fs::create_dir_all(path)?;
    Ok(())
}

fn copy_tree(source: &Path, target: &Path, plan: bool) -> Result<usize, RootCommandError> {
    let metadata = fs::symlink_metadata(source)?;

    if metadata.is_dir() {
        if !plan {
            fs::create_dir_all(target)?;
        }

        let mut copied_files = 0;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copied_files += copy_tree(&entry.path(), &target.join(entry.file_name()), plan)?;
        }
        return Ok(copied_files);
    }

    if metadata.file_type().is_symlink() {
        return copy_symlink_target(source, target, plan);
    }

    if metadata.is_file() {
        if !plan {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, target)?;
        }
        return Ok(1);
    }

    Ok(0)
}

fn copy_symlink_target(
    source: &Path,
    target: &Path,
    plan: bool,
) -> Result<usize, RootCommandError> {
    let link_target = fs::read_link(source)?;
    let resolved_target = if link_target.is_absolute() {
        link_target
    } else {
        source
            .parent()
            .ok_or_else(|| RootCommandError::SymlinkHasNoParent(source.to_path_buf()))?
            .join(link_target)
    };

    copy_tree(&resolved_target, target, plan)
}

fn write_app_build_script(
    path: &Path,
    request: &RootPackageRequest,
    content_root: &Path,
    steam_build_root: &Path,
) -> Result<(), RootCommandError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let set_live = request.set_live.as_deref().map_or(String::new(), |branch| {
        format!("    \"SetLive\" \"{}\"\n", vdf_escape(branch))
    });
    let script = format!(
        "\"AppBuild\"\n{{\n    \"AppID\" \"{}\"\n    \"Desc\" \"{}\"\n{}    \"ContentRoot\" \"{}\"\n    \"BuildOutput\" \"{}\"\n    \"Depots\"\n    {{\n        \"{}\"\n        {{\n            \"FileMapping\"\n            {{\n                \"LocalPath\" \"*\"\n                \"DepotPath\" \".\"\n                \"recursive\" \"1\"\n            }}\n        }}\n    }}\n}}\n",
        request.app_id,
        vdf_escape(&request.description),
        set_live,
        vdf_escape(&content_root.display().to_string()),
        vdf_escape(&steam_build_root.display().to_string()),
        request.depot_id,
    );

    fs::write(path, script)?;
    Ok(())
}

fn vdf_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn alias_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.cmd")
    } else {
        stem.to_owned()
    }
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}

fn activation_script_name() -> &'static str {
    if cfg!(windows) {
        ACTIVATION_SCRIPT_WINDOWS
    } else {
        ACTIVATION_SCRIPT_UNIX
    }
}

/// Error returned by root package or SteamPipe publish workflows.
#[derive(Debug)]
pub enum RootCommandError {
    ToolchainStatus(ToolchainStatusError),
    Steam(SteamCommandError),
    Io(std::io::Error),
    InvalidAppId(u32),
    InvalidDepotId(u32),
    DefaultBranchCannotBeSetLive,
    MissingPackageInput(PathBuf),
    SymlinkHasNoParent(PathBuf),
}

impl From<ToolchainStatusError> for RootCommandError {
    fn from(error: ToolchainStatusError) -> Self {
        Self::ToolchainStatus(error)
    }
}

impl From<std::io::Error> for RootCommandError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<SteamCommandError> for RootCommandError {
    fn from(error: SteamCommandError) -> Self {
        Self::Steam(error)
    }
}

impl fmt::Display for RootCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainStatus(error) => write!(formatter, "{error}"),
            Self::Steam(error) => write!(formatter, "{error}"),
            Self::Io(error) => write!(formatter, "root package command failed: {error}"),
            Self::InvalidAppId(app_id) => {
                write!(formatter, "Steam AppID must be non-zero, found `{app_id}`")
            }
            Self::InvalidDepotId(depot_id) => {
                write!(
                    formatter,
                    "Steam DepotID must be non-zero, found `{depot_id}`"
                )
            }
            Self::DefaultBranchCannotBeSetLive => write!(
                formatter,
                "SteamPipe cannot automatically set the default branch live; upload first, then set default live in Steamworks"
            ),
            Self::MissingPackageInput(path) => write!(
                formatter,
                "root package input is missing: `{}`",
                path.display()
            ),
            Self::SymlinkHasNoParent(path) => {
                write!(
                    formatter,
                    "symlink path has no parent: `{}`",
                    path.display()
                )
            }
        }
    }
}

impl Error for RootCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ToolchainStatus(error) => Some(error),
            Self::Steam(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::InvalidAppId(_)
            | Self::InvalidDepotId(_)
            | Self::DefaultBranchCannotBeSetLive
            | Self::MissingPackageInput(_)
            | Self::SymlinkHasNoParent(_) => None,
        }
    }
}
