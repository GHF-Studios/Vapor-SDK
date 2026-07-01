//! Official Rust distribution manifest access for toolchain planning.

use std::error::Error;
use std::fmt;

use vapor_core::CanonicalToolchain;

const DEFAULT_DIST_SERVER: &str = "https://static.rust-lang.org/dist";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DistArchive {
    pub package: String,
    pub target: String,
    pub url: String,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub(super) struct ChannelManifest {
    pub date: String,
    value: toml::Value,
}

impl ChannelManifest {
    pub(super) fn fetch(toolchain: &CanonicalToolchain) -> Result<(String, Self), DistError> {
        let url = channel_manifest_url(toolchain);
        let source = ureq::get(&url).call()?.into_string()?;
        Ok((url, Self::parse(&source)?))
    }

    fn parse(source: &str) -> Result<Self, DistError> {
        let value = source.parse::<toml::Value>()?;
        let date = string_at(&value, &["date"])?.to_owned();
        Ok(Self { date, value })
    }

    pub(super) fn archive(&self, package: &str, target: &str) -> Result<DistArchive, DistError> {
        let package_value = value_at(&self.value, &["pkg", package])?;
        let target_value = value_at(package_value, &["target", target])?;
        let available = target_value
            .get("available")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false);

        if !available {
            return Err(DistError::UnavailableTarget { package: package.to_owned(), target: target.to_owned() });
        }

        Ok(DistArchive {
            package: package.to_owned(),
            target: target.to_owned(),
            url: string_at(target_value, &["xz_url"])?.to_owned(),
            hash: string_at(target_value, &["xz_hash"])?.to_owned(),
        })
    }
}

fn channel_manifest_url(toolchain: &CanonicalToolchain) -> String {
    format!(
        "{}/{}/channel-rust-{}.toml",
        DEFAULT_DIST_SERVER, toolchain.date, toolchain.channel
    )
}

fn value_at<'a>(value: &'a toml::Value, path: &[&str]) -> Result<&'a toml::Value, DistError> {
    let mut current = value;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| DistError::MissingField(path.join(".")))?;
    }
    Ok(current)
}

fn string_at<'a>(value: &'a toml::Value, path: &[&str]) -> Result<&'a str, DistError> {
    value_at(value, path)?
        .as_str()
        .ok_or_else(|| DistError::InvalidField(path.join(".")))
}

#[derive(Debug)]
pub enum DistError {
    Network(ureq::Error),
    Read(std::io::Error),
    Parse(toml::de::Error),
    MissingField(String),
    InvalidField(String),
    UnavailableTarget { package: String, target: String },
}

impl From<ureq::Error> for DistError {
    fn from(error: ureq::Error) -> Self {
        Self::Network(error)
    }
}

impl From<std::io::Error> for DistError {
    fn from(error: std::io::Error) -> Self {
        Self::Read(error)
    }
}

impl From<toml::de::Error> for DistError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error)
    }
}

impl fmt::Display for DistError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => write!(formatter, "failed to fetch Rust dist manifest: {error}"),
            Self::Read(error) => write!(formatter, "failed to read Rust dist manifest response: {error}"),
            Self::Parse(error) => write!(formatter, "failed to parse Rust dist manifest: {error}"),
            Self::MissingField(path) => write!(formatter, "Rust dist manifest is missing `{path}`"),
            Self::InvalidField(path) => write!(formatter, "Rust dist manifest field `{path}` has an unexpected type"),
            Self::UnavailableTarget { package, target } => {
                write!(formatter, "Rust dist package `{package}` is unavailable for target `{target}`")
            }
        }
    }
}

impl Error for DistError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Network(error) => Some(error),
            Self::Read(error) => Some(error),
            Self::Parse(error) => Some(error),
            Self::MissingField(_) | Self::InvalidField(_) | Self::UnavailableTarget { .. } => None,
        }
    }
}
