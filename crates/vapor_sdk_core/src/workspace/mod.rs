//! SDK workspace workflows backed by the Vapor-managed Cargo binary.

mod cargo;
mod deploy;
mod error;
mod identity;
mod report;

pub use error::WorkspaceCommandError;
pub use report::{WorkspaceCargoReport, WorkspaceDeployReport};

/// Commands that operate on the current authoring workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCommand {
    /// Run `cargo check` through the Vapor-managed Cargo binary.
    Check,
    /// Run `cargo fmt` through the Vapor-managed Cargo binary.
    Fmt,
    /// Run `cargo build --workspace` through the Vapor-managed Cargo binary.
    Build,
    /// Build and promote the first-party SDK CLI into the executable-root `bin` directory.
    Deploy,
}

/// Run `cargo check --workspace` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_check() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    cargo::VaporCargo::new()?.run(&["check", "--workspace"])
}

/// Run `cargo fmt` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_fmt() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    cargo::VaporCargo::new()?.run(&["fmt"])
}

/// Run `cargo build --workspace` through `$VAPOR_HOME/toolchain/active/bin/cargo`.
pub fn workspace_build() -> Result<WorkspaceCargoReport, WorkspaceCommandError> {
    cargo::VaporCargo::new()?.run(&["build", "--workspace"])
}

/// Build and promote `vapor_sdk_cli` into `$VAPOR_HOME/bin`.
pub fn workspace_deploy() -> Result<WorkspaceDeployReport, WorkspaceCommandError> {
    deploy::workspace_deploy()
}
