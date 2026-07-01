//! Placeholder output for parsed SDK commands.

use vapor_sdk_core::{CommandSpec, GlobalOptions};

pub(crate) fn print_stub(globals: GlobalOptions, spec: CommandSpec) {
    println!(
        "Doing {}! Trust me, I am definitely doing it and not just a placeholder message.",
        spec.action
    );

    if globals.verbose {
        println!("summary: {}", spec.summary);
        println!("state_surface: {:?}", spec.surface);
        print_lines("preconditions", spec.preconditions);
        print_lines("future_effects", spec.future_effects);
        println!("yes: {}", globals.yes);
        println!("force: {}", globals.force);
        println!("strict: {}", globals.strict);
        println!("keep_unused_versions: {}", globals.keep_unused_versions);
    }
}

fn print_lines(label: &str, lines: &[&str]) {
    println!("{label}:");
    if lines.is_empty() {
        println!("  none");
    } else {
        for line in lines {
            println!("  {line}");
        }
    }
}
