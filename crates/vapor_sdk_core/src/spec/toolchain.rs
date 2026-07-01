//! SDK toolchain command specifications.

use super::{spec, CommandSpec, StateSurface};
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
            "Plan the pinned Rust/Cargo toolchain installation.",
            StateSurface::ReadOnly,
            &["toolchain pin is known"],
            &["resolve official Rust distribution archives without mutating local state"],
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
