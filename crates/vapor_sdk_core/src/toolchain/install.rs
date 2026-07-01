//! Toolchain installation from a verified official Rust distribution plan.

use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

use super::BOOTSTRAP_STAGING_DIR;
use super::plan::{
    ToolchainArchivePlan, ToolchainInstallPlan, ToolchainPlanError, toolchain_install_plan,
};

/// Result of installing the canonical Vapor toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainInstallReport {
    pub plan: ToolchainInstallPlan,
    pub staging_root: PathBuf,
    pub installed_root: PathBuf,
}

/// Download, verify, stage, and promote the canonical Vapor toolchain.
pub fn toolchain_install() -> Result<ToolchainInstallReport, ToolchainInstallError> {
    if !cfg!(unix) {
        return Err(ToolchainInstallError::UnsupportedInstallerHost);
    }

    let plan = toolchain_install_plan()?;
    let installed_root = plan.status.toolchain_root.clone();

    if installed_root.exists() {
        return Err(ToolchainInstallError::AlreadyInstalled(installed_root));
    }

    fs::create_dir_all(&plan.download_root)?;
    let staging_root = plan
        .status
        .bootstrap_root
        .join(BOOTSTRAP_STAGING_DIR)
        .join(format!(
            "{}-{}-{}",
            plan.status.toolchain.identifier(),
            plan.status.host_triple,
            std::process::id()
        ));

    if staging_root.exists() {
        return Err(ToolchainInstallError::StagingAlreadyExists(staging_root));
    }

    let extract_root = staging_root.join("extracted");
    let staged_active_root = staging_root.join("active");
    fs::create_dir_all(&extract_root)?;
    fs::create_dir_all(&staged_active_root)?;

    for archive in &plan.archives {
        download_archive(archive)?;
        verify_archive(archive)?;
        let component_root = unpack_archive(archive, &extract_root)?;
        install_component(&component_root, &staged_active_root)?;
    }

    verify_staged_toolchain(&staged_active_root)?;
    fs::create_dir_all(plan.status.toolchain_home.clone())?;
    fs::rename(&staged_active_root, &installed_root)?;
    let _ = fs::remove_dir_all(&staging_root);

    Ok(ToolchainInstallReport {
        plan,
        staging_root,
        installed_root,
    })
}

fn download_archive(archive: &ToolchainArchivePlan) -> Result<(), ToolchainInstallError> {
    if archive.download_path.exists() && archive_hash_matches(archive)? {
        return Ok(());
    }

    if let Some(parent) = archive.download_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let partial_path = archive.download_path.with_extension("download");
    let mut response = ureq::get(&archive.url).call()?.into_reader();
    let mut partial = File::create(&partial_path)?;
    std::io::copy(&mut response, &mut partial)?;
    partial.flush()?;

    verify_path_hash(&partial_path, &archive.sha256)?;

    if archive.download_path.exists() {
        fs::remove_file(&archive.download_path)?;
    }
    fs::rename(partial_path, &archive.download_path)?;
    Ok(())
}

fn verify_archive(archive: &ToolchainArchivePlan) -> Result<(), ToolchainInstallError> {
    verify_path_hash(&archive.download_path, &archive.sha256)
}

fn archive_hash_matches(archive: &ToolchainArchivePlan) -> Result<bool, ToolchainInstallError> {
    Ok(sha256_file(&archive.download_path)? == archive.sha256)
}

fn verify_path_hash(path: &Path, expected: &str) -> Result<(), ToolchainInstallError> {
    let actual = sha256_file(path)?;
    if actual == expected {
        Ok(())
    } else {
        Err(ToolchainInstallError::HashMismatch {
            path: path.to_path_buf(),
            expected: expected.to_owned(),
            actual,
        })
    }
}

fn sha256_file(path: &Path) -> Result<String, ToolchainInstallError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn unpack_archive(
    archive: &ToolchainArchivePlan,
    extract_root: &Path,
) -> Result<PathBuf, ToolchainInstallError> {
    let archive_file = File::open(&archive.download_path)?;
    let decoder = xz2::read::XzDecoder::new(archive_file);
    let mut tar = tar::Archive::new(decoder);
    tar.unpack(extract_root)?;

    Ok(extract_root.join(archive_top_dir_name(archive)?))
}

fn archive_top_dir_name(archive: &ToolchainArchivePlan) -> Result<String, ToolchainInstallError> {
    let file_name = archive
        .download_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ToolchainInstallError::InvalidArchivePath(archive.download_path.clone()))?;
    Ok(file_name
        .strip_suffix(".tar.xz")
        .ok_or_else(|| ToolchainInstallError::InvalidArchivePath(archive.download_path.clone()))?
        .to_owned())
}

fn install_component(component_root: &Path, prefix: &Path) -> Result<(), ToolchainInstallError> {
    let install_script = component_root.join("install.sh");
    if !install_script.exists() {
        return Err(ToolchainInstallError::MissingInstallScript(install_script));
    }

    let status = Command::new("bash")
        .arg(&install_script)
        .arg(format!("--prefix={}", prefix.display()))
        .arg("--disable-ldconfig")
        .current_dir(component_root)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(ToolchainInstallError::InstallScriptFailed {
            script: install_script,
            status,
        })
    }
}

fn verify_staged_toolchain(staged_root: &Path) -> Result<(), ToolchainInstallError> {
    let cargo = staged_root.join("bin").join(executable_name("cargo"));
    let rustc = staged_root.join("bin").join(executable_name("rustc"));

    if !cargo.exists() {
        return Err(ToolchainInstallError::MissingInstalledBinary(cargo));
    }
    if !rustc.exists() {
        return Err(ToolchainInstallError::MissingInstalledBinary(rustc));
    }

    Ok(())
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", std::env::consts::EXE_SUFFIX)
}

/// Error returned while installing the canonical Vapor toolchain.
#[derive(Debug)]
pub enum ToolchainInstallError {
    Plan(ToolchainPlanError),
    Network(ureq::Error),
    Io(std::io::Error),
    UnsupportedInstallerHost,
    AlreadyInstalled(PathBuf),
    StagingAlreadyExists(PathBuf),
    HashMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    InvalidArchivePath(PathBuf),
    MissingInstallScript(PathBuf),
    InstallScriptFailed {
        script: PathBuf,
        status: std::process::ExitStatus,
    },
    MissingInstalledBinary(PathBuf),
}

impl From<ToolchainPlanError> for ToolchainInstallError {
    fn from(error: ToolchainPlanError) -> Self {
        Self::Plan(error)
    }
}

impl From<ureq::Error> for ToolchainInstallError {
    fn from(error: ureq::Error) -> Self {
        Self::Network(error)
    }
}

impl From<std::io::Error> for ToolchainInstallError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for ToolchainInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plan(error) => write!(formatter, "{error}"),
            Self::Network(error) => write!(formatter, "failed to download Rust archive: {error}"),
            Self::Io(error) => write!(formatter, "toolchain install I/O failed: {error}"),
            Self::UnsupportedInstallerHost => write!(
                formatter,
                "toolchain install currently requires a Unix host installer path"
            ),
            Self::AlreadyInstalled(path) => write!(
                formatter,
                "toolchain is already installed at `{}`",
                path.display()
            ),
            Self::StagingAlreadyExists(path) => write!(
                formatter,
                "toolchain staging directory already exists at `{}`",
                path.display()
            ),
            Self::HashMismatch {
                path,
                expected,
                actual,
            } => write!(
                formatter,
                "SHA-256 mismatch for `{}`: expected {expected}, got {actual}",
                path.display()
            ),
            Self::InvalidArchivePath(path) => write!(
                formatter,
                "Rust archive path is not a `.tar.xz` file: `{}`",
                path.display()
            ),
            Self::MissingInstallScript(path) => write!(
                formatter,
                "Rust archive is missing installer script `{}`",
                path.display()
            ),
            Self::InstallScriptFailed { script, status } => write!(
                formatter,
                "Rust component installer `{}` failed with {status}",
                script.display()
            ),
            Self::MissingInstalledBinary(path) => write!(
                formatter,
                "installed toolchain is missing `{}`",
                path.display()
            ),
        }
    }
}

impl Error for ToolchainInstallError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Plan(error) => Some(error),
            Self::Network(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::UnsupportedInstallerHost
            | Self::AlreadyInstalled(_)
            | Self::StagingAlreadyExists(_)
            | Self::HashMismatch { .. }
            | Self::InvalidArchivePath(_)
            | Self::MissingInstallScript(_)
            | Self::InstallScriptFailed { .. }
            | Self::MissingInstalledBinary(_) => None,
        }
    }
}
