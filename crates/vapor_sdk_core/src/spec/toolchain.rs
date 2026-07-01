//! SDK toolchain command specifications.

use super::{CommandSpec, StateSurface, spec};
use crate::toolchain::ToolchainCommand;

pub(super) fn describe(command: &ToolchainCommand) -> CommandSpec {
    match command {
        ToolchainCommand::Status => spec(
            "sdk toolchain status",
            "Inspect pinned Rust/Cargo/toolchain state.",
            StateSurface::ReadOnly,
            &[],
            &["display pinned toolchain state"],
        ),
        ToolchainCommand::Install => spec(
            "sdk toolchain install",
            "Install the pinned Rust/Cargo toolchain.",
            StateSurface::Toolchain,
            &["toolchain pin is known"],
            &[
                "download official Rust archives",
                "verify archive hashes",
                "promote staged toolchain",
            ],
        ),
        ToolchainCommand::Repair => spec(
            "sdk toolchain repair",
            "Repair the pinned Rust/Cargo toolchain installation.",
            StateSurface::Toolchain,
            &["toolchain pin is known"],
            &["repair or reinstall damaged toolchain components"],
        ),
    }
}
