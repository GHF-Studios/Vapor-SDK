//! Clap parser and conversion into SDK-core command requests.

mod args;
mod leaf;
mod packagepack;
mod pack;
mod repair;
mod template;
mod toolchain;

use clap::{Parser, Subcommand};
use leaf::LeafCommand;
use packagepack::PackagepackCommand;
use pack::PackCommand;
use repair::RepairCommand;
use template::TemplateCommand;
use toolchain::ToolchainCommand;
use vapor_sdk_core as core;

/// Parsed top-level SDK invocation.
#[derive(Parser)]
#[command(name = "vapor-sdk")]
#[command(version, about = "Authoring workflows for Vapor content.")]
pub(crate) struct Cli {
    /// Show operation planning, diagnostics, historical context, and live detail.
    #[arg(long, global = true)]
    verbose: bool,
    /// Accept non-destructive prompts such as template or dependency setup.
    #[arg(long, global = true)]
    yes: bool,
    /// Permit destructive or risk-bearing operations when that command supports it.
    #[arg(long, global = true)]
    force: bool,
    /// Reject authoring mutations that would leave authored content invalid.
    #[arg(long, global = true)]
    strict: bool,
    /// Keep old unused installed versions after update, lock, repair, or cleanup.
    #[arg(long, global = true)]
    keep_unused_versions: bool,
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    pub(crate) fn into_parts(self) -> (core::GlobalOptions, core::SdkCommand) {
        let globals = core::GlobalOptions {
            verbose: self.verbose,
            yes: self.yes,
            force: self.force,
            strict: self.strict,
            keep_unused_versions: self.keep_unused_versions,
        };
        (globals, self.command.into_core())
    }
}

/// Root SDK workflows.
#[derive(Subcommand)]
enum Command {
    Version,
    Status,
    Repair { #[command(subcommand)] command: RepairCommand },
    Toolchain { #[command(subcommand)] command: ToolchainCommand },
    Template { #[command(subcommand)] command: TemplateCommand },
    Packagepack { #[command(subcommand)] command: PackagepackCommand },
    Enginepack { #[command(subcommand)] command: PackCommand },
    Gamepack { #[command(subcommand)] command: PackCommand },
    Modpack { #[command(subcommand)] command: PackCommand },
    Engine { #[command(subcommand)] command: LeafCommand },
    Game { #[command(subcommand)] command: LeafCommand },
    #[command(name = "engine_mod")]
    EngineMod { #[command(subcommand)] command: LeafCommand },
    #[command(name = "game_mod")]
    GameMod { #[command(subcommand)] command: LeafCommand },
    #[command(name = "extension_mod")]
    ExtensionMod { #[command(subcommand)] command: LeafCommand },
}

impl Command {
    fn into_core(self) -> core::SdkCommand {
        match self {
            Self::Version => core::SdkCommand::Version,
            Self::Status => core::SdkCommand::Status,
            Self::Repair { command } => core::SdkCommand::Repair(command.into_core()),
            Self::Toolchain { command } => core::SdkCommand::Toolchain(command.into_core()),
            Self::Template { command } => core::SdkCommand::Template(command.into_core()),
            Self::Packagepack { command } => core::SdkCommand::Packagepack(command.into_core()),
            Self::Enginepack { command } => pack_command(core::ContentType::Enginepack, command),
            Self::Gamepack { command } => pack_command(core::ContentType::Gamepack, command),
            Self::Modpack { command } => pack_command(core::ContentType::Modpack, command),
            Self::Engine { command } => leaf_command(core::ContentType::Engine, command),
            Self::Game { command } => leaf_command(core::ContentType::Game, command),
            Self::EngineMod { command } => leaf_command(core::ContentType::EngineMod, command),
            Self::GameMod { command } => leaf_command(core::ContentType::GameMod, command),
            Self::ExtensionMod { command } => leaf_command(core::ContentType::ExtensionMod, command),
        }
    }
}

fn pack_command(pack_type: core::ContentType, command: PackCommand) -> core::SdkCommand {
    core::SdkCommand::Pack { pack_type, command: command.into_core() }
}

fn leaf_command(content_type: core::ContentType, command: LeafCommand) -> core::SdkCommand {
    core::SdkCommand::Leaf { content_type, command: command.into_core() }
}
