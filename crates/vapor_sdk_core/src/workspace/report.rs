use std::path::PathBuf;
use std::process::ExitStatus;

/// Result of running one Vapor-managed Cargo command.
#[derive(Debug, Clone)]
pub struct WorkspaceCargoReport {
    pub working_directory: PathBuf,
    pub cargo_path: PathBuf,
    pub rustc_path: PathBuf,
    pub cargo_home: PathBuf,
    pub target_dir: PathBuf,
    pub cargo_args: Vec<String>,
    pub status: ExitStatus,
}

/// Result of promoting a built SDK executable into the deploy root.
#[derive(Debug, Clone)]
pub struct WorkspaceDeployReport {
    pub build: WorkspaceCargoReport,
    pub source_executable: PathBuf,
    pub deployed_executable: PathBuf,
}
