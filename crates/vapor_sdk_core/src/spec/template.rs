//! SDK template command specifications.

use super::{read_spec, CommandSpec};
use crate::template::TemplateCommand;

pub(super) fn describe(command: &TemplateCommand) -> CommandSpec {
    match command {
        TemplateCommand::List => read_spec("sdk template list", "List available authoring templates."),
        TemplateCommand::Info => {
            read_spec("sdk template info", "Inspect template metadata and intended use.")
        }
    }
}
