//! SteamCMD tooling and authentication workflows.
//!
//! Vapor never accepts or stores Steam passwords. Login delegates to SteamCMD so
//! SteamCMD owns password prompts, Steam Guard, and its own persisted login
//! token/config behavior.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::root::ROOT_STEAM_APP_ID;
use crate::toolchain::{ToolchainStatusError, toolchain_status};

const STEAM_HOME_DIR: &str = "steam";
const STEAMCMD_STATE_DIR: &str = "steamcmd";

/// SDK Steam workflows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamCommand {
    /// Inspect SteamCMD availability and Vapor Steam state paths.
    Status(SteamStatusRequest),
    /// Run SteamCMD login without Vapor seeing the password or Steam Guard code.
    Login(SteamLoginRequest),
}

/// Parameters for Steam status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamStatusRequest {
    /// SteamCMD executable path, or `steamcmd` to resolve through PATH.
    pub steamcmd: PathBuf,
}

/// Parameters for SteamCMD login.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLoginRequest {
    /// Steam account used by SteamCMD.
    pub account: String,
    /// SteamCMD executable path, or `steamcmd` to resolve through PATH.
    pub steamcmd: PathBuf,
}

/// Parameters for running one SteamPipe app build through SteamCMD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamRunAppBuildRequest {
    pub account: String,
    pub steamcmd: PathBuf,
    pub app_build_script: PathBuf,
}

/// Resolved SteamCMD and Vapor Steam state paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamStatusReport {
    pub app_id: u32,
    pub vapor_home: PathBuf,
    pub steam_home: PathBuf,
    pub steamcmd_state_root: PathBuf,
    pub requested_steamcmd: PathBuf,
    pub resolved_steamcmd: Option<PathBuf>,
    pub steamcmd_source: SteamCmdSource,
    pub auth_model: &'static str,
}

/// Report for SteamCMD login.
#[derive(Debug, Clone)]
pub struct SteamLoginReport {
    pub status: SteamStatusReport,
    pub account: String,
    pub steam_args: Vec<String>,
    pub exit_status: ExitStatus,
}

/// Report for SteamCMD app build upload.
#[derive(Debug, Clone)]
pub struct SteamRunAppBuildReport {
    pub status: SteamStatusReport,
    pub account: String,
    pub steam_args: Vec<String>,
    pub exit_status: ExitStatus,
}

/// How a SteamCMD path was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamCmdSource {
    ExplicitPath,
    PathLookup,
    Unavailable,
}

impl SteamCmdSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPath => "explicit_path",
            Self::PathLookup => "path_lookup",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Inspect SteamCMD availability and Vapor's Steam state root.
pub fn steam_status(request: &SteamStatusRequest) -> Result<SteamStatusReport, SteamCommandError> {
    steam_status_for(&request.steamcmd)
}

/// Run SteamCMD login and let SteamCMD handle password and Steam Guard prompts.
pub fn steam_login(request: &SteamLoginRequest) -> Result<SteamLoginReport, SteamCommandError> {
    validate_account(&request.account)?;

    let status = steam_status_for(&request.steamcmd)?;
    let steamcmd = status
        .resolved_steamcmd
        .clone()
        .ok_or_else(|| SteamCommandError::SteamCmdUnavailable(request.steamcmd.clone()))?;
    let steam_args = login_args(&request.account);
    let exit_status = run_steamcmd(&steamcmd, &steam_args)?;

    Ok(SteamLoginReport {
        status,
        account: request.account.clone(),
        steam_args,
        exit_status,
    })
}

/// Run a SteamPipe app build through SteamCMD.
pub fn steam_run_app_build(
    request: &SteamRunAppBuildRequest,
) -> Result<SteamRunAppBuildReport, SteamCommandError> {
    validate_account(&request.account)?;

    let status = steam_status_for(&request.steamcmd)?;
    let steamcmd = status
        .resolved_steamcmd
        .clone()
        .ok_or_else(|| SteamCommandError::SteamCmdUnavailable(request.steamcmd.clone()))?;
    let steam_args = run_app_build_args(&request.account, &request.app_build_script);
    let exit_status = run_steamcmd(&steamcmd, &steam_args)?;

    Ok(SteamRunAppBuildReport {
        status,
        account: request.account.clone(),
        steam_args,
        exit_status,
    })
}

/// Build the SteamCMD command line without executing it.
pub fn steam_run_app_build_plan(
    request: &SteamRunAppBuildRequest,
) -> Result<(SteamStatusReport, Vec<String>), SteamCommandError> {
    validate_account(&request.account)?;

    let status = steam_status_for(&request.steamcmd)?;
    let steam_args = run_app_build_args(&request.account, &request.app_build_script);
    Ok((status, steam_args))
}

fn steam_status_for(steamcmd: &Path) -> Result<SteamStatusReport, SteamCommandError> {
    let toolchain = toolchain_status()?;
    let (resolved_steamcmd, steamcmd_source) = resolve_steamcmd(steamcmd);
    let steam_home = toolchain.vapor_home.join(STEAM_HOME_DIR);
    let steamcmd_state_root = steam_home.join(STEAMCMD_STATE_DIR);

    Ok(SteamStatusReport {
        app_id: ROOT_STEAM_APP_ID,
        vapor_home: toolchain.vapor_home,
        steam_home,
        steamcmd_state_root,
        requested_steamcmd: steamcmd.to_path_buf(),
        resolved_steamcmd,
        steamcmd_source,
        auth_model: "SteamCMD owns password prompts, Steam Guard, and saved login tokens.",
    })
}

fn validate_account(account: &str) -> Result<(), SteamCommandError> {
    if account.trim().is_empty() {
        return Err(SteamCommandError::MissingAccount);
    }

    Ok(())
}

fn login_args(account: &str) -> Vec<String> {
    vec!["+login".to_owned(), account.to_owned(), "+quit".to_owned()]
}

fn run_app_build_args(account: &str, app_build_script: &Path) -> Vec<String> {
    vec![
        "+login".to_owned(),
        account.to_owned(),
        "+run_app_build".to_owned(),
        app_build_script.display().to_string(),
        "+quit".to_owned(),
    ]
}

fn run_steamcmd(steamcmd: &Path, steam_args: &[String]) -> Result<ExitStatus, SteamCommandError> {
    Command::new(steamcmd)
        .args(steam_args)
        .status()
        .map_err(SteamCommandError::SteamCmdIo)
}

fn resolve_steamcmd(steamcmd: &Path) -> (Option<PathBuf>, SteamCmdSource) {
    if steamcmd.components().count() > 1 || steamcmd.is_absolute() {
        if is_executable_file(steamcmd) {
            return (Some(steamcmd.to_path_buf()), SteamCmdSource::ExplicitPath);
        }
        return (None, SteamCmdSource::Unavailable);
    }

    let Some(path) = env::var_os("PATH") else {
        return (None, SteamCmdSource::Unavailable);
    };

    for root in env::split_paths(&path) {
        for candidate in executable_candidates(&root, steamcmd) {
            if is_executable_file(&candidate) {
                return (Some(candidate), SteamCmdSource::PathLookup);
            }
        }
    }

    (None, SteamCmdSource::Unavailable)
}

fn executable_candidates(root: &Path, command: &Path) -> Vec<PathBuf> {
    let candidate = root.join(command);
    let mut candidates = vec![candidate.clone()];

    if env::consts::EXE_SUFFIX.is_empty() {
        return candidates;
    }

    if candidate.extension().is_none() {
        let mut with_suffix: OsString = candidate.into_os_string();
        with_suffix.push(env::consts::EXE_SUFFIX);
        candidates.push(PathBuf::from(with_suffix));
    }

    candidates
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// Error returned by SteamCMD tooling workflows.
#[derive(Debug)]
pub enum SteamCommandError {
    ToolchainStatus(ToolchainStatusError),
    SteamCmdIo(std::io::Error),
    SteamCmdUnavailable(PathBuf),
    MissingAccount,
}

impl From<ToolchainStatusError> for SteamCommandError {
    fn from(error: ToolchainStatusError) -> Self {
        Self::ToolchainStatus(error)
    }
}

impl fmt::Display for SteamCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ToolchainStatus(error) => write!(formatter, "{error}"),
            Self::SteamCmdIo(error) => write!(formatter, "failed to run SteamCMD: {error}"),
            Self::SteamCmdUnavailable(path) => {
                write!(
                    formatter,
                    "SteamCMD is not available at `{}`",
                    path.display()
                )
            }
            Self::MissingAccount => write!(formatter, "Steam account must not be empty"),
        }
    }
}

impl Error for SteamCommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ToolchainStatus(error) => Some(error),
            Self::SteamCmdIo(error) => Some(error),
            Self::SteamCmdUnavailable(_) | Self::MissingAccount => None,
        }
    }
}
