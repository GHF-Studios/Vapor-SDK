use std::fs;

use super::error::WorkspaceCommandError;
use super::identity::{WorkspaceIdentity, discover_workspace_identity, require_current_repo_kind};
use super::report::{WorkspaceStatusReport, WorkspaceSyncReport};

const CUSTOM_CONTENT_KIND: &str = "custom-content";
const MANAGED_CARGO_WORKSPACE: &str = r#"[workspace]
resolver = "3"
members = ["crates/workspace_anchor"]

[workspace.package]
version = "0.5.0"
edition = "2024"
"#;
const ANCHOR_CARGO_TOML: &str = r#"[package]
name = "vapor_workspace_anchor"
version.workspace = true
edition.workspace = true
"#;
const ANCHOR_LIB_RS: &str =
    "#![forbid(unsafe_code)]\n\npub const WORKSPACE_ANCHOR: &str = \"vapor_workspace_anchor\";\n";

pub(super) fn workspace_status() -> Result<WorkspaceStatusReport, WorkspaceCommandError> {
    Ok(status_from_identity(discover_workspace_identity()?))
}

pub(super) fn workspace_sync() -> Result<WorkspaceSyncReport, WorkspaceCommandError> {
    let identity = require_current_repo_kind(CUSTOM_CONTENT_KIND)?;
    let mut changed_paths = Vec::new();
    let cargo_manifest = identity.workspace_root.join("Cargo.toml");
    let crates_dir = identity.workspace_root.join("crates");
    let anchor_dir = crates_dir.join("workspace_anchor");
    let anchor_src_dir = anchor_dir.join("src");
    let anchor_manifest = anchor_dir.join("Cargo.toml");
    let anchor_lib = anchor_src_dir.join("lib.rs");

    sync_file(&cargo_manifest, MANAGED_CARGO_WORKSPACE, &mut changed_paths)?;

    if !crates_dir.exists() {
        fs::create_dir_all(&crates_dir)?;
        changed_paths.push(crates_dir);
    }

    if !anchor_src_dir.exists() {
        fs::create_dir_all(&anchor_src_dir)?;
        changed_paths.push(anchor_src_dir.clone());
    }

    sync_file(&anchor_manifest, ANCHOR_CARGO_TOML, &mut changed_paths)?;
    sync_file(&anchor_lib, ANCHOR_LIB_RS, &mut changed_paths)?;

    Ok(WorkspaceSyncReport {
        status: status_from_identity(identity),
        changed_paths,
    })
}

fn sync_file(
    path: &std::path::Path,
    content: &str,
    changed_paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), WorkspaceCommandError> {
    if path.is_file() && fs::read_to_string(path)? == content {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    changed_paths.push(path.to_path_buf());
    Ok(())
}

fn status_from_identity(identity: WorkspaceIdentity) -> WorkspaceStatusReport {
    WorkspaceStatusReport {
        cargo_manifest_exists: identity.workspace_root.join("Cargo.toml").is_file(),
        crates_dir_exists: identity.workspace_root.join("crates").is_dir(),
        invocation_directory: identity.invocation_directory,
        workspace_root: identity.workspace_root,
        workspace_kind: identity.kind,
        workspace_id: identity.id,
    }
}
