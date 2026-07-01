//! Clap commands for authored packagepacks.

use super::args::{ContentSource, ContentType, child};
use clap::Subcommand;
use vapor_sdk_core as core;

#[derive(Subcommand)]
pub(super) enum PackagepackCommand {
    List {
        source: ContentSource,
    },
    Status {
        packagepack_id: String,
    },
    Fingerprint {
        packagepack_id: String,
    },
    Inspect {
        packagepack_id: String,
    },
    Validate {
        packagepack_id: String,
    },
    Lock {
        packagepack_id: String,
    },
    New {
        packagepack_id: String,
    },
    Init {
        packagepack_id: String,
    },
    Add {
        packagepack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Remove {
        packagepack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Select {
        packagepack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Unselect {
        packagepack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Build {
        packagepack_id: String,
    },
    Package {
        packagepack_id: String,
    },
    Publish {
        packagepack_id: String,
    },
}

impl PackagepackCommand {
    pub(super) fn into_core(self) -> core::PackagepackCommand {
        match self {
            Self::List { source } => read(core::ContentReadCommand::List {
                source: source.into(),
            }),
            Self::Status { packagepack_id } => read(core::ContentReadCommand::Status {
                content_id: packagepack_id,
            }),
            Self::Fingerprint { packagepack_id } => read(core::ContentReadCommand::Fingerprint {
                content_id: packagepack_id,
            }),
            Self::Inspect { packagepack_id } => read(core::ContentReadCommand::Inspect {
                content_id: packagepack_id,
            }),
            Self::Validate { packagepack_id } => read(core::ContentReadCommand::Validate {
                content_id: packagepack_id,
            }),
            Self::Lock { packagepack_id } => core::PackagepackCommand::Lock { packagepack_id },
            Self::New { packagepack_id } => author(core::SourceAuthoringCommand::New {
                content_id: packagepack_id,
            }),
            Self::Init { packagepack_id } => author(core::SourceAuthoringCommand::Init {
                content_id: packagepack_id,
            }),
            Self::Build { packagepack_id } => author(core::SourceAuthoringCommand::Build {
                content_id: packagepack_id,
            }),
            Self::Package { packagepack_id } => author(core::SourceAuthoringCommand::Package {
                content_id: packagepack_id,
            }),
            Self::Publish { packagepack_id } => author(core::SourceAuthoringCommand::Publish {
                content_id: packagepack_id,
            }),
            Self::Add {
                packagepack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Add {
                pack_id: packagepack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Remove {
                packagepack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Remove {
                pack_id: packagepack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Select {
                packagepack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Select {
                pack_id: packagepack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Unselect {
                packagepack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Unselect {
                pack_id: packagepack_id,
                child: child(child_content_type, child_content_id),
            }),
        }
    }
}

fn read(command: core::ContentReadCommand) -> core::PackagepackCommand {
    core::PackagepackCommand::Read(command)
}

fn author(command: core::SourceAuthoringCommand) -> core::PackagepackCommand {
    core::PackagepackCommand::Author(command)
}

fn compose(command: core::PackCompositionCommand) -> core::PackagepackCommand {
    core::PackagepackCommand::Compose(command)
}
