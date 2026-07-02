//! SteamCMD tooling and authentication workflows.
//!
//! Vapor never accepts or stores Steam passwords. Login delegates to SteamCMD so
//! SteamCMD owns password prompts, Steam Guard, and its own persisted login
//! token/config behavior.

mod error;
mod process;
mod status;
mod types;

pub use error::SteamCommandError;
pub use process::{steam_login, steam_run_app_build, steam_run_app_build_plan};
pub use status::steam_status;
pub use types::{
    SteamCmdSource, SteamCommand, SteamLoginReport, SteamLoginRequest, SteamRunAppBuildReport,
    SteamRunAppBuildRequest, SteamStatusReport, SteamStatusRequest,
};
