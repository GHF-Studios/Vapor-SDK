//! Root Vapor package and SteamPipe publishing workflows.
//!
//! Root publishing is intentionally about the Steam-distributed Vapor app root,
//! not about arbitrary authored content. The installed Rust toolchain is not
//! packaged here; Vapor should wrap/install Rust tooling deliberately instead
//! of shipping the active toolchain tree as app content.

mod error;
mod package;
mod publish;
mod types;

pub use error::RootCommandError;
pub use package::root_package;
pub use publish::root_publish;
pub use types::{
    ROOT_STEAM_APP_ID, ROOT_STEAM_DEPOT_ID, RootCommand, RootPackageReport, RootPackageRequest,
    RootPublishReport, RootPublishRequest,
};
