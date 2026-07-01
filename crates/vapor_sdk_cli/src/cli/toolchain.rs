//! Clap commands for SDK toolchain workflows.

use clap::Subcommand;
use vapor_sdk_core as core;

#[derive(Subcommand)]
pub(super) enum ToolchainCommand {
    Status,
    Install,
    Repair,
}

impl ToolchainCommand {
    pub(super) fn into_core(self) -> core::ToolchainCommand {
        match self {
            Self::Status => core::ToolchainCommand::Status,
            Self::Install => core::ToolchainCommand::Install,
            Self::Repair => core::ToolchainCommand::Repair,
        }
    }
}
