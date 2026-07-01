//! SDK-managed toolchain command intent.

/// Toolchain commands for the pinned SDK-managed Rust/Cargo toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolchainCommand {
    /// Inspect pinned toolchain state.
    Status,
    /// Install the pinned toolchain chosen by the project owner.
    Install,
    /// Repair a damaged or incomplete pinned toolchain installation.
    Repair,
}
