use std::env;

use toml::Value;

use super::error::WorkspaceCommandError;

pub(super) fn require_current_repo_kind(expected: &str) -> Result<(), WorkspaceCommandError> {
    let manifest_path = env::current_dir()?.join("Vapor.toml");

    if !manifest_path.is_file() {
        return Err(WorkspaceCommandError::MissingWorkspaceManifest(
            manifest_path,
        ));
    }

    let text = std::fs::read_to_string(manifest_path)?;
    let manifest = text.parse::<Value>()?;
    let actual = manifest
        .get("vapor")
        .and_then(|vapor| vapor.get("kind"))
        .and_then(Value::as_str)
        .map(str::to_owned);

    if actual.as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(WorkspaceCommandError::WrongWorkspaceKind {
            expected: expected.to_owned(),
            actual,
        })
    }
}
