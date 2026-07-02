use std::env;
use std::path::PathBuf;

use toml::Value;

use super::error::WorkspaceCommandError;

#[derive(Debug, Clone)]
pub(super) struct WorkspaceIdentity {
    pub(super) invocation_directory: PathBuf,
    pub(super) workspace_root: PathBuf,
    pub(super) kind: Option<String>,
    pub(super) id: Option<String>,
}

pub(super) fn discover_workspace_identity() -> Result<WorkspaceIdentity, WorkspaceCommandError> {
    let invocation_directory = env::current_dir()?;
    let mut candidate = invocation_directory.clone();

    loop {
        let manifest_path = candidate.join("Vapor.toml");
        if manifest_path.is_file() {
            let text = std::fs::read_to_string(&manifest_path)?;
            let manifest = text.parse::<Value>()?;
            let workspace = manifest.get("workspace");
            let kind = workspace
                .and_then(|workspace| workspace.get("kind"))
                .and_then(Value::as_str)
                .map(str::to_owned);
            let id = workspace
                .and_then(|workspace| workspace.get("id"))
                .and_then(Value::as_str)
                .map(str::to_owned);

            return Ok(WorkspaceIdentity {
                invocation_directory,
                workspace_root: candidate,
                kind,
                id,
            });
        }

        if !candidate.pop() {
            return Err(WorkspaceCommandError::MissingWorkspaceManifest(
                invocation_directory,
            ));
        }
    }
}

pub(super) fn require_current_repo_kind(
    expected: &str,
) -> Result<WorkspaceIdentity, WorkspaceCommandError> {
    let identity = discover_workspace_identity()?;

    if identity.kind.as_deref() == Some(expected) {
        Ok(identity)
    } else {
        Err(WorkspaceCommandError::WrongWorkspaceKind {
            expected: expected.to_owned(),
            actual: identity.kind,
        })
    }
}
