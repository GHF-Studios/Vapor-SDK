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

pub use commands::{
    ContentReadCommand, LeafCommand, PackagepackCommand, PackCommand, PackCompositionCommand,
    SdkCommand, SourceAuthoringCommand,
};
pub use content::{allowed_pack_children, ContentSource, ContentType};
pub use options::GlobalOptions;
pub use repair::{RepairCommand, RepairTarget};
pub use spec::{describe_command, CommandSpec, StateSurface};
pub use template::TemplateCommand;
pub use toolchain::ToolchainCommand;
pub use vapor_core::ChildContentRef;
