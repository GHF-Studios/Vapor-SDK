//! SDK workspace command specifications.

use super::{CommandSpec, StateSurface, spec};
use crate::workspace::WorkspaceCommand;

pub(super) fn describe(command: &WorkspaceCommand) -> CommandSpec {
    match command {
        WorkspaceCommand::Check => spec(
            "sdk check",
            "Check the current workspace through the Vapor-managed Cargo binary.",
            StateSurface::BuildArtifact,
            &["Vapor toolchain is installed"],
            &["run cargo check without falling back to system Cargo"],
        ),
    }
}
