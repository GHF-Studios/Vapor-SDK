use std::fs;

use serde::Deserialize;
use vapor_core::ContentType;

use super::error::WorkspaceCommandError;
use super::identity::{WorkspaceIdentity, discover_workspace_identity, require_current_repo_kind};
use super::report::{WorkspaceStatusReport, WorkspaceSyncReport};

const CUSTOM_CONTENT_KIND: &str = "custom-content";
const GENERATED_HEADER: &str =
    "# MANAGED BY VAPOR. EDIT Vapor.toml, THEN RUN `vapor-sdk workspace sync`.\n";
const GENERATED_FOOTER: &str = "# END MANAGED BY VAPOR.\n";
const GENERATED_RS_HEADER: &str =
    "// MANAGED BY VAPOR. EDIT Vapor.toml, THEN RUN `vapor-sdk workspace sync`.\n";
const GENERATED_RS_FOOTER: &str = "// END MANAGED BY VAPOR.\n";

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    content: Option<Vec<ContentEntry>>,
}

#[derive(Debug, Deserialize)]
struct ContentEntry {
    kind: String,
    id: String,
}

pub(super) fn workspace_status() -> Result<WorkspaceStatusReport, WorkspaceCommandError> {
    Ok(status_from_identity(discover_workspace_identity()?))
}

pub(super) fn workspace_sync() -> Result<WorkspaceSyncReport, WorkspaceCommandError> {
    let identity = require_current_repo_kind(CUSTOM_CONTENT_KIND)?;
    let content = read_content_graph(&identity)?;
    let mut changed_paths = Vec::new();
    let cargo_manifest = identity.workspace_root.join("Cargo.toml");
    let crates_dir = identity.workspace_root.join("crates");

    sync_file(
        &cargo_manifest,
        &workspace_cargo_toml(&content),
        &mut changed_paths,
    )?;

    if !crates_dir.exists() {
        fs::create_dir_all(&crates_dir)?;
        changed_paths.push(crates_dir);
    }

    for entry in &content {
        let crate_dir = identity
            .workspace_root
            .join("crates")
            .join(crate_name(&entry.id));
        let src_dir = crate_dir.join("src");

        if !src_dir.exists() {
            fs::create_dir_all(&src_dir)?;
            changed_paths.push(src_dir.clone());
        }

        sync_file(
            &crate_dir.join("Cargo.toml"),
            &crate_cargo_toml(entry),
            &mut changed_paths,
        )?;
        sync_file(
            &crate_dir.join("Vapor.toml"),
            &content_vapor_toml(entry),
            &mut changed_paths,
        )?;
        sync_file(
            &src_dir.join("lib.rs"),
            &content_lib_rs(entry),
            &mut changed_paths,
        )?;
    }

    Ok(WorkspaceSyncReport {
        status: status_from_identity(identity),
        changed_paths,
    })
}

fn read_content_graph(
    identity: &WorkspaceIdentity,
) -> Result<Vec<ContentEntry>, WorkspaceCommandError> {
    let manifest = fs::read_to_string(identity.workspace_root.join("Vapor.toml"))?;
    let manifest: WorkspaceManifest = toml::from_str(&manifest)?;
    let content = manifest.content.unwrap_or_default();

    if content.is_empty() {
        return Err(WorkspaceCommandError::MissingContentGraph);
    }

    for entry in &content {
        entry
            .kind
            .parse::<ContentType>()
            .map_err(|error| WorkspaceCommandError::InvalidContentGraph(format!("{}", error)))?;
        if crate_name(&entry.id).is_empty() {
            return Err(WorkspaceCommandError::InvalidContentGraph(format!(
                "content id `{}` does not produce a valid crate name",
                entry.id
            )));
        }
    }

    Ok(content)
}

fn workspace_cargo_toml(content: &[ContentEntry]) -> String {
    let members = content
        .iter()
        .map(|entry| format!("    \"crates/{}\"", crate_name(&entry.id)))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        "{GENERATED_HEADER}[workspace]\nresolver = \"3\"\nmembers = [\n{members}\n]\n\n[workspace.package]\nversion = \"0.5.0\"\nedition = \"2024\"\n{GENERATED_FOOTER}"
    )
}

fn crate_cargo_toml(entry: &ContentEntry) -> String {
    format!(
        "{GENERATED_HEADER}[package]\nname = \"{}\"\nversion.workspace = true\nedition.workspace = true\n{GENERATED_FOOTER}",
        crate_name(&entry.id)
    )
}

fn content_vapor_toml(entry: &ContentEntry) -> String {
    format!(
        "{GENERATED_HEADER}[{}]\nid = \"{}\"\n{GENERATED_FOOTER}",
        entry.kind, entry.id
    )
}

fn content_lib_rs(entry: &ContentEntry) -> String {
    format!(
        "{GENERATED_RS_HEADER}#![forbid(unsafe_code)]\n\npub const CONTENT_KIND: &str = \"{}\";\npub const CONTENT_ID: &str = \"{}\";\n{GENERATED_RS_FOOTER}",
        entry.kind, entry.id
    )
}

fn crate_name(id: &str) -> String {
    id.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned()
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
