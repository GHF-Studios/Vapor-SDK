use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use vapor_core::ContentType;

use crate::GlobalOptions;

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
const SEEDED_RS_HEADER: &str = "// SEEDED BY VAPOR. THIS FILE IS USER-OWNED AFTER CREATION.\n";
const SEEDED_RS_FOOTER: &str = "// END VAPOR SEED.\n";

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    content: Option<Vec<ContentEntry>>,
}

#[derive(Debug, Deserialize)]
struct ContentEntry {
    kind: String,
    id: String,
}

pub(super) fn workspace_status(
    globals: &GlobalOptions,
) -> Result<WorkspaceStatusReport, WorkspaceCommandError> {
    Ok(status_from_identity(discover_workspace_identity(globals)?))
}

pub(super) fn workspace_sync(
    globals: &GlobalOptions,
) -> Result<WorkspaceSyncReport, WorkspaceCommandError> {
    let identity = require_current_repo_kind(globals, CUSTOM_CONTENT_KIND)?;
    let content = read_content_graph(&identity)?;
    let mut changed_paths = Vec::new();
    let cargo_manifest = identity.workspace_root.join("Cargo.toml");
    let crates_dir = identity.workspace_root.join("crates");

    sync_managed_file(
        &cargo_manifest,
        &workspace_cargo_toml(&content),
        GENERATED_HEADER,
        GENERATED_FOOTER,
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

        sync_managed_file(
            &crate_dir.join("Cargo.toml"),
            &crate_cargo_toml(entry),
            GENERATED_HEADER,
            GENERATED_FOOTER,
            &mut changed_paths,
        )?;
        sync_managed_file(
            &crate_dir.join("Vapor.toml"),
            &content_vapor_toml(entry),
            GENERATED_HEADER,
            GENERATED_FOOTER,
            &mut changed_paths,
        )?;
        sync_seed_file(
            &src_dir.join("lib.rs"),
            &content_lib_rs(entry),
            &legacy_content_lib_rs(entry),
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

    let mut crate_names = HashSet::new();
    for entry in &content {
        entry
            .kind
            .parse::<ContentType>()
            .map_err(|error| WorkspaceCommandError::InvalidContentGraph(format!("{}", error)))?;
        let crate_name = crate_name(&entry.id);
        if crate_name.is_empty() {
            return Err(WorkspaceCommandError::InvalidContentGraph(format!(
                "content id `{}` does not produce a valid crate name",
                entry.id
            )));
        }
        if !crate_names.insert(crate_name.clone()) {
            return Err(WorkspaceCommandError::InvalidContentGraph(format!(
                "multiple content ids produce the generated crate name `{crate_name}`"
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
        "{SEEDED_RS_HEADER}#![forbid(unsafe_code)]\n\npub const CONTENT_KIND: &str = \"{}\";\npub const CONTENT_ID: &str = \"{}\";\n{SEEDED_RS_FOOTER}",
        entry.kind, entry.id
    )
}

fn legacy_content_lib_rs(entry: &ContentEntry) -> String {
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

fn sync_managed_file(
    path: &Path,
    content: &str,
    header: &str,
    footer: &str,
    changed_paths: &mut Vec<PathBuf>,
) -> Result<(), WorkspaceCommandError> {
    if path.exists() && !path.is_file() {
        return Err(WorkspaceCommandError::GeneratedPathIsNotFile(
            path.to_path_buf(),
        ));
    }

    if path.is_file() {
        let current = fs::read_to_string(path)?;
        if current == content {
            return Ok(());
        }
        if !is_managed_file(&current, header, footer) {
            return Err(WorkspaceCommandError::UnmanagedGeneratedFile(
                path.to_path_buf(),
            ));
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    changed_paths.push(path.to_path_buf());
    Ok(())
}

fn sync_seed_file(
    path: &Path,
    content: &str,
    legacy_content: &str,
    changed_paths: &mut Vec<PathBuf>,
) -> Result<(), WorkspaceCommandError> {
    if path.exists() && !path.is_file() {
        return Err(WorkspaceCommandError::GeneratedPathIsNotFile(
            path.to_path_buf(),
        ));
    }

    if path.is_file() {
        let current = fs::read_to_string(path)?;
        if current == content {
            return Ok(());
        }
        if current != legacy_content {
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    changed_paths.push(path.to_path_buf());
    Ok(())
}

fn is_managed_file(content: &str, header: &str, footer: &str) -> bool {
    content.starts_with(header) && content.ends_with(footer)
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
