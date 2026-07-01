//! Command-line entrypoints for SDK workflows.

use clap::Parser;

mod cli;
mod output;

fn main() {
    let (globals, command) = cli::Cli::parse().into_parts();
    output::print_stub(globals, vapor_sdk_core::describe_command(&command));
}
