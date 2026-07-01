//! Toolchain install planning against the official Rust distribution manifest.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use vapor_core::ToolchainComponent;

use super::dist::{ChannelManifest, DistArchive, DistError};
use super::{toolchain_status, ToolchainStatus, ToolchainStatusError};

/// Zero-mutation plan for installing the canonical Vapor Rust/Cargo toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainInstallPlan {
    pub status: ToolchainStatus,
    pub manifest_url: String,
    pub manifest_date: String,
    pub dist_cache: PathBuf,
    pub archives: Vec<ToolchainArchivePlan>,
}

/// One official Rust archive required by a toolchain install plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainArchivePlan {
    pub component: ToolchainComponent,
    pub package: String,
    pub target: String,
    pub url: String,
    pub sha256: String,
    pub cache_path: PathBuf,
}

/// Build an install plan without downloading or unpacking any component archives.
pub fn toolchain_install_plan() -> Result<ToolchainInstallPlan, ToolchainPlanError> {
    let status = toolchain_status()?;

    if !status.host_supported {
        return Err(ToolchainPlanError::UnsupportedHost(status.host_triple.to_owned()));
    }

    let (manifest_url, manifest) = ChannelManifest::fetch(&status.toolchain)?;
    let dist_cache = status.vapor_home.join("dist-cache").join(status.toolchain.identifier());
    let mut archives = Vec::new();

    push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::Rustc, "rustc", status.host_triple)?;
    push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::Cargo, "cargo", status.host_triple)?;
    push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::Rustfmt, "rustfmt-preview", status.host_triple)?;
    push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::Clippy, "clippy-preview", status.host_triple)?;

    for target in status.supported_target_triples() {
        push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::RustStd, "rust-std", target)?;
    }

    push_host_archive(&mut archives, &dist_cache, &manifest, ToolchainComponent::RustSrc, "rust-src", "*")?;

    Ok(ToolchainInstallPlan { status, manifest_url, manifest_date: manifest.date, dist_cache, archives })
}

fn push_host_archive(
    archives: &mut Vec<ToolchainArchivePlan>,
    dist_cache: &PathBuf,
    manifest: &ChannelManifest,
    component: ToolchainComponent,
    package: &str,
    target: &str,
) -> Result<(), ToolchainPlanError> {
    archives.push(archive_plan(component, dist_cache, manifest.archive(package, target)?)?);
    Ok(())
}

fn archive_plan(
    component: ToolchainComponent,
    dist_cache: &PathBuf,
    archive: DistArchive,
) -> Result<ToolchainArchivePlan, ToolchainPlanError> {
    let file_name = archive
        .url
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ToolchainPlanError::InvalidArchiveUrl(archive.url.clone()))?;

    Ok(ToolchainArchivePlan {
        component,
        package: archive.package,
        target: archive.target,
        cache_path: dist_cache.join(file_name),
        url: archive.url,
        sha256: archive.hash,
    })
}

#[derive(Debug)]
pub enum ToolchainPlanError {
    Status(ToolchainStatusError),
    Dist(DistError),
    UnsupportedHost(String),
    InvalidArchiveUrl(String),
}

impl From<ToolchainStatusError> for ToolchainPlanError {
    fn from(error: ToolchainStatusError) -> Self {
        Self::Status(error)
    }
}

impl From<DistError> for ToolchainPlanError {
    fn from(error: DistError) -> Self {
        Self::Dist(error)
    }
}

impl fmt::Display for ToolchainPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status(error) => write!(formatter, "{error}"),
            Self::Dist(error) => write!(formatter, "{error}"),
            Self::UnsupportedHost(host) => write!(formatter, "host triple `{host}` is not supported by this Vapor toolchain pin"),
            Self::InvalidArchiveUrl(url) => write!(formatter, "official Rust archive URL has no filename: {url}"),
        }
    }
}

impl Error for ToolchainPlanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Status(error) => Some(error),
            Self::Dist(error) => Some(error),
            Self::UnsupportedHost(_) | Self::InvalidArchiveUrl(_) => None,
        }
    }
}
