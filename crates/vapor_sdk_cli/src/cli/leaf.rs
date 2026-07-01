//! Clap commands for authored leaf content.

use super::args::ContentSource;
use clap::Subcommand;
use vapor_sdk_core as core;

#[derive(Subcommand)]
pub(super) enum LeafCommand {
    List { source: ContentSource },
    Status { content_id: String },
    Fingerprint { content_id: String },
    Inspect { content_id: String },
    Validate { content_id: String },
    New { content_id: String },
    Init { content_id: String },
    Build { content_id: String },
    Package { content_id: String },
    Publish { content_id: String },
}

impl LeafCommand {
    pub(super) fn into_core(self) -> core::LeafCommand {
        match self {
            Self::List { source } => read(core::ContentReadCommand::List {
                source: source.into(),
            }),
            Self::Status { content_id } => read(core::ContentReadCommand::Status { content_id }),
            Self::Fingerprint { content_id } => {
                read(core::ContentReadCommand::Fingerprint { content_id })
            }
            Self::Inspect { content_id } => read(core::ContentReadCommand::Inspect { content_id }),
            Self::Validate { content_id } => {
                read(core::ContentReadCommand::Validate { content_id })
            }
            Self::New { content_id } => author(core::SourceAuthoringCommand::New { content_id }),
            Self::Init { content_id } => author(core::SourceAuthoringCommand::Init { content_id }),
            Self::Build { content_id } => {
                author(core::SourceAuthoringCommand::Build { content_id })
            }
            Self::Package { content_id } => {
                author(core::SourceAuthoringCommand::Package { content_id })
            }
            Self::Publish { content_id } => {
                author(core::SourceAuthoringCommand::Publish { content_id })
            }
        }
    }
}

fn read(command: core::ContentReadCommand) -> core::LeafCommand {
    core::LeafCommand::Read(command)
}

fn author(command: core::SourceAuthoringCommand) -> core::LeafCommand {
    core::LeafCommand::Author(command)
}
