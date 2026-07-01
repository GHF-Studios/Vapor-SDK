//! Command-line entrypoints for SDK workflows.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "vapor-sdk")]
#[command(version, about = "Authoring workflows for Vapor content.")]
struct Cli {
    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, global = true)]
    yes: bool,

    #[arg(long, global = true)]
    force: bool,

    #[arg(long, global = true)]
    strict: bool,

    #[arg(long, global = true)]
    keep_unused_versions: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Version,
    Status,
    Repair {
        #[command(subcommand)]
        command: RepairCommand,
    },
    Toolchain {
        #[command(subcommand)]
        command: ToolchainCommand,
    },
    Template {
        #[command(subcommand)]
        command: TemplateCommand,
    },
    Packagepack {
        #[command(subcommand)]
        command: PackagepackCommand,
    },
    Enginepack {
        #[command(subcommand)]
        command: PackCommand,
    },
    Gamepack {
        #[command(subcommand)]
        command: PackCommand,
    },
    Modpack {
        #[command(subcommand)]
        command: PackCommand,
    },
    Engine {
        #[command(subcommand)]
        command: LeafCommand,
    },
    Game {
        #[command(subcommand)]
        command: LeafCommand,
    },
    #[command(name = "engine_mod")]
    EngineMod {
        #[command(subcommand)]
        command: LeafCommand,
    },
    #[command(name = "game_mod")]
    GameMod {
        #[command(subcommand)]
        command: LeafCommand,
    },
    #[command(name = "extension_mod")]
    ExtensionMod {
        #[command(subcommand)]
        command: LeafCommand,
    },
}

#[derive(Subcommand)]
enum RepairCommand {
    Status,
    Plan { target: RepairTarget },
    Apply { target: RepairTarget },
}

#[derive(Subcommand)]
enum ToolchainCommand {
    Status,
    Install,
    Repair,
}

#[derive(Subcommand)]
enum TemplateCommand {
    List,
    Info,
}

#[derive(Subcommand)]
enum PackagepackCommand {
    List { source: ContentSource },
    Status { packagepack_id: String },
    Fingerprint { packagepack_id: String },
    Inspect { packagepack_id: String },
    Validate { packagepack_id: String },
    Lock { packagepack_id: String },
    New { packagepack_id: String },
    Init { packagepack_id: String },
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
    Build { packagepack_id: String },
    Package { packagepack_id: String },
    Publish { packagepack_id: String },
}

#[derive(Subcommand)]
enum PackCommand {
    List { source: ContentSource },
    Status { pack_id: String },
    Fingerprint { pack_id: String },
    Inspect { pack_id: String },
    Validate { pack_id: String },
    New { pack_id: String },
    Init { pack_id: String },
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
    Build { pack_id: String },
    Package { pack_id: String },
    Publish { pack_id: String },
}

#[derive(Subcommand)]
enum LeafCommand {
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

#[derive(Clone, Copy, ValueEnum)]
enum ContentSource {
    Discovered,
    Local,
    Git,
    Workshop,
    All,
}

#[derive(Clone, Copy, ValueEnum)]
enum ContentType {
    Packagepack,
    Enginepack,
    Gamepack,
    Modpack,
    Engine,
    Game,
    #[value(name = "engine_mod")]
    EngineMod,
    #[value(name = "game_mod")]
    GameMod,
    #[value(name = "extension_mod")]
    ExtensionMod,
}

#[derive(Clone, Copy, ValueEnum)]
enum RepairTarget {
    #[value(name = "core_state")]
    CoreState,
    Toolchain,
    Steam,
    #[value(name = "content_catalog")]
    ContentCatalog,
    #[value(name = "content_library")]
    ContentLibrary,
    #[value(name = "active_composition")]
    ActiveComposition,
    All,
}

fn main() {
    let cli = Cli::parse();
    dispatch(&cli);
}

fn dispatch(cli: &Cli) {
    match &cli.command {
        Command::Version => stub(cli, "sdk version"),
        Command::Status => stub(cli, "sdk status"),
        Command::Repair { command } => dispatch_repair(cli, command),
        Command::Toolchain { command } => dispatch_toolchain(cli, command),
        Command::Template { command } => dispatch_template(cli, command),
        Command::Packagepack { command } => dispatch_packagepack(cli, command),
        Command::Enginepack { command } => dispatch_pack(cli, "enginepack", command),
        Command::Gamepack { command } => dispatch_pack(cli, "gamepack", command),
        Command::Modpack { command } => dispatch_pack(cli, "modpack", command),
        Command::Engine { command } => dispatch_leaf(cli, "engine", command),
        Command::Game { command } => dispatch_leaf(cli, "game", command),
        Command::EngineMod { command } => dispatch_leaf(cli, "engine_mod", command),
        Command::GameMod { command } => dispatch_leaf(cli, "game_mod", command),
        Command::ExtensionMod { command } => dispatch_leaf(cli, "extension_mod", command),
    }
}

fn dispatch_repair(cli: &Cli, command: &RepairCommand) {
    match command {
        RepairCommand::Status => stub(cli, "sdk repair status"),
        RepairCommand::Plan { .. } => stub(cli, "sdk repair plan"),
        RepairCommand::Apply { .. } => stub(cli, "sdk repair apply"),
    }
}

fn dispatch_toolchain(cli: &Cli, command: &ToolchainCommand) {
    match command {
        ToolchainCommand::Status => stub(cli, "sdk toolchain status"),
        ToolchainCommand::Install => stub(cli, "sdk toolchain install"),
        ToolchainCommand::Repair => stub(cli, "sdk toolchain repair"),
    }
}

fn dispatch_template(cli: &Cli, command: &TemplateCommand) {
    match command {
        TemplateCommand::List => stub(cli, "sdk template list"),
        TemplateCommand::Info => stub(cli, "sdk template info"),
    }
}

fn dispatch_packagepack(cli: &Cli, command: &PackagepackCommand) {
    let action = match command {
        PackagepackCommand::List { .. } => "list",
        PackagepackCommand::Status { .. } => "status",
        PackagepackCommand::Fingerprint { .. } => "fingerprint",
        PackagepackCommand::Inspect { .. } => "inspect",
        PackagepackCommand::Validate { .. } => "validate",
        PackagepackCommand::Lock { .. } => "lock",
        PackagepackCommand::New { .. } => "new",
        PackagepackCommand::Init { .. } => "init",
        PackagepackCommand::Add { .. } => "add",
        PackagepackCommand::Remove { .. } => "remove",
        PackagepackCommand::Select { .. } => "select",
        PackagepackCommand::Unselect { .. } => "unselect",
        PackagepackCommand::Build { .. } => "build",
        PackagepackCommand::Package { .. } => "package",
        PackagepackCommand::Publish { .. } => "publish",
    };
    stub(cli, &format!("sdk packagepack {action}"));
}

fn dispatch_pack(cli: &Cli, content_type: &str, command: &PackCommand) {
    let action = match command {
        PackCommand::List { .. } => "list",
        PackCommand::Status { .. } => "status",
        PackCommand::Fingerprint { .. } => "fingerprint",
        PackCommand::Inspect { .. } => "inspect",
        PackCommand::Validate { .. } => "validate",
        PackCommand::New { .. } => "new",
        PackCommand::Init { .. } => "init",
        PackCommand::Add { .. } => "add",
        PackCommand::Remove { .. } => "remove",
        PackCommand::Select { .. } => "select",
        PackCommand::Unselect { .. } => "unselect",
        PackCommand::Build { .. } => "build",
        PackCommand::Package { .. } => "package",
        PackCommand::Publish { .. } => "publish",
    };
    stub(cli, &format!("sdk {content_type} {action}"));
}

fn dispatch_leaf(cli: &Cli, content_type: &str, command: &LeafCommand) {
    let action = match command {
        LeafCommand::List { .. } => "list",
        LeafCommand::Status { .. } => "status",
        LeafCommand::Fingerprint { .. } => "fingerprint",
        LeafCommand::Inspect { .. } => "inspect",
        LeafCommand::Validate { .. } => "validate",
        LeafCommand::New { .. } => "new",
        LeafCommand::Init { .. } => "init",
        LeafCommand::Build { .. } => "build",
        LeafCommand::Package { .. } => "package",
        LeafCommand::Publish { .. } => "publish",
    };
    stub(cli, &format!("sdk {content_type} {action}"));
}

fn stub(cli: &Cli, action: &str) {
    println!("Doing {action}! Trust me, I am definitely doing it and not just a placeholder message.");

    if cli.verbose {
        println!("verbose: true");
        println!("yes: {}", cli.yes);
        println!("force: {}", cli.force);
        println!("strict: {}", cli.strict);
        println!("keep_unused_versions: {}", cli.keep_unused_versions);
    }
}
