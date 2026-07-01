//! Command-line entrypoints for SDK workflows.

use clap::Parser;

mod cli;
mod output;

fn main() {
    let (globals, command) = cli::Cli::parse().into_parts();
    if let Err(error) = output::print_command(globals, &command) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
