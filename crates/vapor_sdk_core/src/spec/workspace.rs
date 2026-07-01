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
        WorkspaceCommand::Fmt => spec(
            "sdk fmt",
            "Format the current workspace through the Vapor-managed Cargo binary.",
            StateSurface::BuildArtifact,
            &["Vapor toolchain is installed"],
            &["run cargo fmt without falling back to system Cargo or rustup"],
        ),
        WorkspaceCommand::Build => spec(
            "sdk build",
            "Build the current workspace through the Vapor-managed Cargo binary.",
            StateSurface::BuildArtifact,
            &["Vapor toolchain is installed"],
            &["write build artifacts under the Vapor output root"],
        ),
        WorkspaceCommand::Deploy => spec(
            "sdk deploy",
            "Build and promote the SDK CLI into the executable-root bin directory.",
            StateSurface::BuildArtifact,
            &[
                "Vapor toolchain is installed",
                "current workspace contains vapor_sdk_cli",
            ],
            &["replace the deployed SDK CLI from the Vapor output root"],
        ),
    }
}
