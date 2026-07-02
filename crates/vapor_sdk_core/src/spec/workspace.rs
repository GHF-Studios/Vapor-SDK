//! SDK workspace command specifications.

use super::{CommandSpec, StateSurface, spec};
use crate::workspace::WorkspaceCommand;

pub(super) fn describe(command: &WorkspaceCommand) -> CommandSpec {
    match command {
        WorkspaceCommand::Status => spec(
            "sdk workspace status",
            "Inspect the current Vapor workspace identity and managed structure.",
            StateSurface::ReadOnly,
            &["Vapor.toml exists at or above the invocation directory"],
            &["display workspace kind, id, root, and managed structure state"],
        ),
        WorkspaceCommand::Sync => spec(
            "sdk workspace sync",
            "Create or update SDK-managed custom-content workspace structure.",
            StateSurface::AuthoredSource,
            &["workspace kind is custom-content"],
            &["create missing managed workspace files and directories"],
        ),
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
            "Build and promote the current first-party tool CLI into the executable-root bin directory.",
            StateSurface::BuildArtifact,
            &[
                "Vapor toolchain is installed",
                "current workspace kind is sdk or launcher",
            ],
            &["replace the deployed tool CLI from the Vapor output root"],
        ),
    }
}
