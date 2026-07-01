//! Reusable SDK command vocabulary and command specifications.
//!
//! This crate still does not implement filesystem, toolchain, packaging,
//! publishing, Steam, or template behavior. It defines the typed request model
//! and documented command contracts that SDK frontends can share.

#![forbid(unsafe_code)]

pub mod commands;
pub mod content;
pub mod options;
pub mod repair;
pub mod spec;
pub mod template;
pub mod toolchain;
pub mod workspace;

pub use commands::{
    ContentReadCommand, LeafCommand, PackCommand, PackCompositionCommand, PackagepackCommand,
    SdkCommand, SourceAuthoringCommand,
};
pub use content::{ContentSource, ContentType, allowed_pack_children};
pub use options::GlobalOptions;
pub use repair::{RepairCommand, RepairTarget};
pub use spec::{CommandSpec, StateSurface, describe_command};
pub use template::TemplateCommand;
pub use toolchain::{
    ACTIVE_TOOLCHAIN_DIR, BOOTSTRAP_DOWNLOADS_DIR, BOOTSTRAP_STAGING_DIR, DEPLOY_DIR, DistError,
    TOOLCHAIN_BOOTSTRAP_DIR, TOOLCHAIN_DIR, ToolchainArchivePlan, ToolchainCommand,
    ToolchainInstallError, ToolchainInstallPlan, ToolchainInstallReport, ToolchainInstallState,
    ToolchainPlanError, ToolchainStatus, ToolchainStatusError, VAPOR_HOME_ENV, VaporHomeSource,
    toolchain_install, toolchain_install_plan, toolchain_status,
};
pub use vapor_core::ChildContentRef;
pub use workspace::{WorkspaceCheckError, WorkspaceCheckReport, WorkspaceCommand, workspace_check};
