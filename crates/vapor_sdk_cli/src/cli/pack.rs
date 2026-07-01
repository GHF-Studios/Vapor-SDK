//! Clap commands for authored non-root packs.

use super::args::{ContentSource, ContentType, child};
use clap::Subcommand;
use vapor_sdk_core as core;

#[derive(Subcommand)]
pub(super) enum PackCommand {
    List {
        source: ContentSource,
    },
    Status {
        pack_id: String,
    },
    Fingerprint {
        pack_id: String,
    },
    Inspect {
        pack_id: String,
    },
    Validate {
        pack_id: String,
    },
    New {
        pack_id: String,
    },
    Init {
        pack_id: String,
    },
    Add {
        pack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Remove {
        pack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Select {
        pack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Unselect {
        pack_id: String,
        child_content_type: ContentType,
        child_content_id: String,
    },
    Build {
        pack_id: String,
    },
    Package {
        pack_id: String,
    },
    Publish {
        pack_id: String,
    },
}

impl PackCommand {
    pub(super) fn into_core(self) -> core::PackCommand {
        match self {
            Self::List { source } => read(core::ContentReadCommand::List {
                source: source.into(),
            }),
            Self::Status { pack_id } => read(core::ContentReadCommand::Status {
                content_id: pack_id,
            }),
            Self::Fingerprint { pack_id } => read(core::ContentReadCommand::Fingerprint {
                content_id: pack_id,
            }),
            Self::Inspect { pack_id } => read(core::ContentReadCommand::Inspect {
                content_id: pack_id,
            }),
            Self::Validate { pack_id } => read(core::ContentReadCommand::Validate {
                content_id: pack_id,
            }),
            Self::New { pack_id } => author(core::SourceAuthoringCommand::New {
                content_id: pack_id,
            }),
            Self::Init { pack_id } => author(core::SourceAuthoringCommand::Init {
                content_id: pack_id,
            }),
            Self::Build { pack_id } => author(core::SourceAuthoringCommand::Build {
                content_id: pack_id,
            }),
            Self::Package { pack_id } => author(core::SourceAuthoringCommand::Package {
                content_id: pack_id,
            }),
            Self::Publish { pack_id } => author(core::SourceAuthoringCommand::Publish {
                content_id: pack_id,
            }),
            Self::Add {
                pack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Add {
                pack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Remove {
                pack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Remove {
                pack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Select {
                pack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Select {
                pack_id,
                child: child(child_content_type, child_content_id),
            }),
            Self::Unselect {
                pack_id,
                child_content_type,
                child_content_id,
            } => compose(core::PackCompositionCommand::Unselect {
                pack_id,
                child: child(child_content_type, child_content_id),
            }),
        }
    }
}

fn read(command: core::ContentReadCommand) -> core::PackCommand {
    core::PackCommand::Read(command)
}

fn author(command: core::SourceAuthoringCommand) -> core::PackCommand {
    core::PackCommand::Author(command)
}

fn compose(command: core::PackCompositionCommand) -> core::PackCommand {
    core::PackCommand::Compose(command)
}
