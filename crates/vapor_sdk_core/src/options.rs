//! Global command options shared by every SDK surface.

/// Global execution knobs accepted by every SDK command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GlobalOptions {
    /// Print operation planning, diagnostics, historical context, and live detail.
    pub verbose: bool,
    /// Accept non-destructive interactive prompts.
    pub yes: bool,
    /// Accept destructive or risk-bearing operations when the command supports it.
    pub force: bool,
    /// Reject authoring mutations that would leave authored content invalid.
    pub strict: bool,
    /// Keep old unused installed versions after update, lock, repair, or cleanup.
    pub keep_unused_versions: bool,
}
