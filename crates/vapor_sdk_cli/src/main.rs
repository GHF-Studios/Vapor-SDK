//! Command-line entrypoints for SDK workflows.
//!
//! This binary currently owns a real command surface and fake handlers. The
//! point of this stage is to make the SDK authoring vocabulary concrete enough
//! to discuss and test manually, without pretending the backend behavior exists.

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    dispatch::run(&cli);
}

mod cli {
    use clap::{Parser, Subcommand, ValueEnum};

    /// Parsed top-level SDK invocation.
    #[derive(Parser)]
    #[command(name = "vapor-sdk")]
    #[command(version, about = "Authoring workflows for Vapor content.")]
    pub(crate) struct Cli {
        /// Show operation planning, diagnostics, historical context, and live detail.
        #[arg(long, global = true)]
        verbose: bool,

        /// Accept non-destructive prompts such as template or dependency setup.
        #[arg(long, global = true)]
        yes: bool,

        /// Permit destructive or risk-bearing operations when that command supports it.
        #[arg(long, global = true)]
        force: bool,

        /// Reject authoring mutations that would leave authored content invalid.
        #[arg(long, global = true)]
        strict: bool,

        /// Keep old unused installed versions after update, lock, repair, or cleanup.
        #[arg(long, global = true)]
        keep_unused_versions: bool,

        #[command(subcommand)]
        command: Command,
    }

    impl Cli {
        pub(crate) fn command(&self) -> &Command {
            &self.command
        }

        pub(crate) fn globals(&self) -> GlobalOptions {
            GlobalOptions {
                verbose: self.verbose,
                yes: self.yes,
                force: self.force,
                strict: self.strict,
                keep_unused_versions: self.keep_unused_versions,
            }
        }
    }

    /// Global execution options after CLI parsing.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct GlobalOptions {
        pub(crate) verbose: bool,
        pub(crate) yes: bool,
        pub(crate) force: bool,
        pub(crate) strict: bool,
        pub(crate) keep_unused_versions: bool,
    }

    /// Root SDK workflows.
    #[derive(Subcommand)]
    pub(crate) enum Command {
        /// Print SDK version/build identity.
        Version,
        /// Summarize SDK health and authoring environment state.
        Status,
        /// Inspect, plan, or apply repairs to SDK-managed state.
        Repair {
            #[command(subcommand)]
            command: RepairCommand,
        },
        /// Inspect, install, or repair the pinned authoring toolchain.
        Toolchain {
            #[command(subcommand)]
            command: ToolchainCommand,
        },
        /// List or inspect SDK project templates.
        Template {
            #[command(subcommand)]
            command: TemplateCommand,
        },
        /// Author packagepacks, including packagepack-only locking.
        Packagepack {
            #[command(subcommand)]
            command: PackagepackCommand,
        },
        /// Author enginepacks.
        Enginepack {
            #[command(subcommand)]
            command: PackCommand,
        },
        /// Author gamepacks.
        Gamepack {
            #[command(subcommand)]
            command: PackCommand,
        },
        /// Author modpacks.
        Modpack {
            #[command(subcommand)]
            command: PackCommand,
        },
        /// Author engine content.
        Engine {
            #[command(subcommand)]
            command: LeafCommand,
        },
        /// Author game content.
        Game {
            #[command(subcommand)]
            command: LeafCommand,
        },
        /// Author engine mod content.
        #[command(name = "engine_mod")]
        EngineMod {
            #[command(subcommand)]
            command: LeafCommand,
        },
        /// Author game mod content.
        #[command(name = "game_mod")]
        GameMod {
            #[command(subcommand)]
            command: LeafCommand,
        },
        /// Author extension mod content.
        #[command(name = "extension_mod")]
        ExtensionMod {
            #[command(subcommand)]
            command: LeafCommand,
        },
    }

    /// Repair commands intentionally split planning from mutation.
    #[derive(Subcommand)]
    pub(crate) enum RepairCommand {
        /// Inspect repairable areas without proposing mutations.
        Status,
        /// Produce a repair plan without applying it.
        Plan { target: RepairTarget },
        /// Apply repairs for a target after the plan is accepted.
        Apply { target: RepairTarget },
    }

    /// Toolchain commands for the pinned SDK-managed Rust/Cargo toolchain.
    #[derive(Subcommand)]
    pub(crate) enum ToolchainCommand {
        /// Inspect pinned toolchain state.
        Status,
        /// Install the pinned toolchain chosen by the project owner.
        Install,
        /// Repair a damaged or incomplete pinned toolchain installation.
        Repair,
    }

    /// Template discovery commands.
    #[derive(Subcommand)]
    pub(crate) enum TemplateCommand {
        /// List available project/content templates.
        List,
        /// Inspect template metadata and intended use.
        Info,
    }

    /// Packagepack authoring commands.
    #[derive(Subcommand)]
    pub(crate) enum PackagepackCommand {
        /// List packagepacks from one content source.
        List { source: ContentSource },
        /// Show SDK-known status for one packagepack source project.
        Status { packagepack_id: String },
        /// Compute or display a deterministic packagepack fingerprint.
        Fingerprint { packagepack_id: String },
        /// Inspect packagepack source metadata and authored graph state.
        Inspect { packagepack_id: String },
        /// Validate packagepack source metadata and graph invariants.
        Validate { packagepack_id: String },
        /// Write a persistent packagepack lock artifact from the resolved graph.
        Lock { packagepack_id: String },
        /// Create a new packagepack project directory.
        New { packagepack_id: String },
        /// Initialize the current effectively empty directory as a packagepack project.
        Init { packagepack_id: String },
        /// Add child content to an authored packagepack.
        Add {
            packagepack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Remove child content from an authored packagepack.
        Remove {
            packagepack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Select active child content inside an authored packagepack.
        Select {
            packagepack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Keep child content present but inactive inside an authored packagepack.
        Unselect {
            packagepack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Build the packagepack source project.
        Build { packagepack_id: String },
        /// Package the packagepack for distribution.
        Package { packagepack_id: String },
        /// Publish the packagepack through configured release channels.
        Publish { packagepack_id: String },
    }

    /// Commands shared by authored non-root pack types.
    #[derive(Subcommand)]
    pub(crate) enum PackCommand {
        /// List packs from one content source.
        List { source: ContentSource },
        /// Show SDK-known status for one pack source project.
        Status { pack_id: String },
        /// Compute or display a deterministic pack fingerprint.
        Fingerprint { pack_id: String },
        /// Inspect pack source metadata and authored graph state.
        Inspect { pack_id: String },
        /// Validate pack source metadata and graph invariants.
        Validate { pack_id: String },
        /// Create a new pack project directory.
        New { pack_id: String },
        /// Initialize the current effectively empty directory as a pack project.
        Init { pack_id: String },
        /// Add child content to an authored pack.
        Add {
            pack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Remove child content from an authored pack.
        Remove {
            pack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Select active child content inside an authored pack.
        Select {
            pack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Keep child content present but inactive inside an authored pack.
        Unselect {
            pack_id: String,
            child_content_type: ContentType,
            child_content_id: String,
        },
        /// Build the pack source project.
        Build { pack_id: String },
        /// Package the pack for distribution.
        Package { pack_id: String },
        /// Publish the pack through configured release channels.
        Publish { pack_id: String },
    }

    /// Commands shared by authored leaf content types.
    #[derive(Subcommand)]
    pub(crate) enum LeafCommand {
        /// List content from one content source.
        List { source: ContentSource },
        /// Show SDK-known status for one content source project.
        Status { content_id: String },
        /// Compute or display a deterministic content fingerprint.
        Fingerprint { content_id: String },
        /// Inspect source metadata and authored state.
        Inspect { content_id: String },
        /// Validate source metadata and compatibility requirements.
        Validate { content_id: String },
        /// Create a new source project directory.
        New { content_id: String },
        /// Initialize the current effectively empty directory as a source project.
        Init { content_id: String },
        /// Build the source project.
        Build { content_id: String },
        /// Package content for distribution.
        Package { content_id: String },
        /// Publish content through configured release channels.
        Publish { content_id: String },
    }

    /// Places the SDK can list or discover content from.
    #[derive(Clone, Copy, Debug, ValueEnum)]
    pub(crate) enum ContentSource {
        /// Already discovered through configured sources.
        Discovered,
        /// Local filesystem or local authoring workspace.
        Local,
        /// Git-backed sources.
        Git,
        /// Steam Workshop-backed sources.
        Workshop,
        /// Every configured source.
        All,
    }

    /// Content kind used when a pack command targets a child.
    #[derive(Clone, Copy, Debug, ValueEnum)]
    pub(crate) enum ContentType {
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

    /// Repair target used by `repair plan` and `repair apply`.
    #[derive(Clone, Copy, Debug, ValueEnum)]
    pub(crate) enum RepairTarget {
        /// SDK-owned local state and configuration.
        #[value(name = "core_state")]
        CoreState,
        /// Pinned Rust/Cargo/toolchain state needed by Vapor workflows.
        Toolchain,
        /// Steam identity, ownership, Workshop, and Steam-facing cache state.
        Steam,
        /// Source indexes and discovered-content catalogs.
        #[value(name = "content_catalog")]
        ContentCatalog,
        /// Installed artifacts and local content records.
        #[value(name = "content_library")]
        ContentLibrary,
        /// The active composition, when SDK tooling needs to reason about one.
        #[value(name = "active_composition")]
        ActiveComposition,
        /// Every repair target in dependency order.
        All,
    }
}

mod dispatch {
    use super::cli::*;

    /// Broad state surface a future implementation may read or mutate.
    #[derive(Debug)]
    enum StateSurface {
        ReadOnly,
        RepairPlan,
        RepairApply,
        Toolchain,
        AuthoredSource,
        AuthoredComposition,
        BuildArtifact,
        Publication,
    }

    /// Local command contract used by the placeholder dispatcher.
    struct CommandSpec {
        action: String,
        summary: &'static str,
        surface: StateSurface,
        preconditions: &'static [&'static str],
        future_effects: &'static [&'static str],
    }

    pub(crate) fn run(cli: &Cli) {
        let globals = cli.globals();
        let spec = describe_command(cli.command());
        print_stub(globals, spec);
    }

    fn describe_command(command: &Command) -> CommandSpec {
        match command {
            Command::Version => spec(
                "sdk version",
                "Print SDK version and build identity.",
                StateSurface::ReadOnly,
                &[],
                &["display version metadata"],
            ),
            Command::Status => spec(
                "sdk status",
                "Summarize SDK health and authoring environment state.",
                StateSurface::ReadOnly,
                &[],
                &["display toolchain, template, and project state"],
            ),
            Command::Repair { command } => describe_repair(command),
            Command::Toolchain { command } => describe_toolchain(command),
            Command::Template { command } => describe_template(command),
            Command::Packagepack { command } => describe_packagepack(command),
            Command::Enginepack { command } => describe_pack("enginepack", command),
            Command::Gamepack { command } => describe_pack("gamepack", command),
            Command::Modpack { command } => describe_pack("modpack", command),
            Command::Engine { command } => describe_leaf("engine", command),
            Command::Game { command } => describe_leaf("game", command),
            Command::EngineMod { command } => describe_leaf("engine_mod", command),
            Command::GameMod { command } => describe_leaf("game_mod", command),
            Command::ExtensionMod { command } => describe_leaf("extension_mod", command),
        }
    }

    fn describe_repair(command: &RepairCommand) -> CommandSpec {
        match command {
            RepairCommand::Status => spec(
                "sdk repair status",
                "Inspect repairable SDK targets without proposing mutation.",
                StateSurface::ReadOnly,
                &[],
                &["display repair target health"],
            ),
            RepairCommand::Plan { .. } => spec(
                "sdk repair plan",
                "Prepare a repair plan without applying it.",
                StateSurface::RepairPlan,
                &["repair target is known"],
                &["compute proposed repair operations"],
            ),
            RepairCommand::Apply { .. } => spec(
                "sdk repair apply",
                "Apply repairs for an SDK target.",
                StateSurface::RepairApply,
                &["repair target is known", "planned mutations are acceptable"],
                &["repair selected SDK-managed state"],
            ),
        }
    }

    fn describe_toolchain(command: &ToolchainCommand) -> CommandSpec {
        match command {
            ToolchainCommand::Status => spec(
                "sdk toolchain status",
                "Inspect pinned Rust/Cargo/toolchain state.",
                StateSurface::ReadOnly,
                &[],
                &["display pinned toolchain state"],
            ),
            ToolchainCommand::Install => spec(
                "sdk toolchain install",
                "Install the pinned Rust/Cargo toolchain.",
                StateSurface::Toolchain,
                &["toolchain pin is known"],
                &["install pinned toolchain components"],
            ),
            ToolchainCommand::Repair => spec(
                "sdk toolchain repair",
                "Repair the pinned Rust/Cargo toolchain installation.",
                StateSurface::Toolchain,
                &["toolchain pin is known"],
                &["repair or reinstall damaged toolchain components"],
            ),
        }
    }

    fn describe_template(command: &TemplateCommand) -> CommandSpec {
        match command {
            TemplateCommand::List => read_spec("sdk template list", "List available authoring templates."),
            TemplateCommand::Info => read_spec("sdk template info", "Inspect template metadata and intended use."),
        }
    }

    fn describe_packagepack(command: &PackagepackCommand) -> CommandSpec {
        match command {
            PackagepackCommand::List { .. } => read_spec("sdk packagepack list", "List packagepacks from a content source."),
            PackagepackCommand::Status { .. } => read_spec("sdk packagepack status", "Show SDK-known packagepack source status."),
            PackagepackCommand::Fingerprint { .. } => read_spec("sdk packagepack fingerprint", "Compute or display a deterministic packagepack fingerprint."),
            PackagepackCommand::Inspect { .. } => read_spec("sdk packagepack inspect", "Inspect packagepack source metadata and authored graph state."),
            PackagepackCommand::Validate { .. } => read_spec("sdk packagepack validate", "Validate packagepack source metadata and graph invariants."),
            PackagepackCommand::Lock { .. } => composition_spec("sdk packagepack lock", "Write a persistent packagepack lock artifact from the resolved authored graph."),
            PackagepackCommand::New { .. } => source_spec("sdk packagepack new", "Create a new packagepack source project."),
            PackagepackCommand::Init { .. } => source_spec("sdk packagepack init", "Initialize the current empty directory as a packagepack source project."),
            PackagepackCommand::Add { .. } => composition_spec("sdk packagepack add", "Add child content to an authored packagepack."),
            PackagepackCommand::Remove { .. } => composition_spec("sdk packagepack remove", "Remove child content from an authored packagepack."),
            PackagepackCommand::Select { .. } => composition_spec("sdk packagepack select", "Select active child content inside an authored packagepack."),
            PackagepackCommand::Unselect { .. } => composition_spec("sdk packagepack unselect", "Keep child content present but inactive inside an authored packagepack."),
            PackagepackCommand::Build { .. } => build_spec("sdk packagepack build", "Build packagepack source artifacts."),
            PackagepackCommand::Package { .. } => build_spec("sdk packagepack package", "Package a packagepack for distribution."),
            PackagepackCommand::Publish { .. } => publish_spec("sdk packagepack publish", "Publish a packagepack through configured release channels."),
        }
    }

    fn describe_pack(content_type: &str, command: &PackCommand) -> CommandSpec {
        let action = match command {
            PackCommand::List { .. } => return read_spec(format!("sdk {content_type} list"), "List packs from a content source."),
            PackCommand::Status { .. } => return read_spec(format!("sdk {content_type} status"), "Show SDK-known pack source status."),
            PackCommand::Fingerprint { .. } => return read_spec(format!("sdk {content_type} fingerprint"), "Compute or display a deterministic pack fingerprint."),
            PackCommand::Inspect { .. } => return read_spec(format!("sdk {content_type} inspect"), "Inspect pack source metadata and authored graph state."),
            PackCommand::Validate { .. } => return read_spec(format!("sdk {content_type} validate"), "Validate pack source metadata and graph invariants."),
            PackCommand::New { .. } => return source_spec(format!("sdk {content_type} new"), "Create a new pack source project."),
            PackCommand::Init { .. } => return source_spec(format!("sdk {content_type} init"), "Initialize the current empty directory as a pack source project."),
            PackCommand::Build { .. } => return build_spec(format!("sdk {content_type} build"), "Build pack source artifacts."),
            PackCommand::Package { .. } => return build_spec(format!("sdk {content_type} package"), "Package a pack for distribution."),
            PackCommand::Publish { .. } => return publish_spec(format!("sdk {content_type} publish"), "Publish a pack through configured release channels."),
            PackCommand::Add { .. } => "add",
            PackCommand::Remove { .. } => "remove",
            PackCommand::Select { .. } => "select",
            PackCommand::Unselect { .. } => "unselect",
        };
        composition_spec(
            format!("sdk {content_type} {action}"),
            "Mutate child membership or active child selection inside an authored pack.",
        )
    }

    fn describe_leaf(content_type: &str, command: &LeafCommand) -> CommandSpec {
        match command {
            LeafCommand::List { .. } => read_spec(format!("sdk {content_type} list"), "List content from a content source."),
            LeafCommand::Status { .. } => read_spec(format!("sdk {content_type} status"), "Show SDK-known source status."),
            LeafCommand::Fingerprint { .. } => read_spec(format!("sdk {content_type} fingerprint"), "Compute or display a deterministic content fingerprint."),
            LeafCommand::Inspect { .. } => read_spec(format!("sdk {content_type} inspect"), "Inspect source metadata and authored state."),
            LeafCommand::Validate { .. } => read_spec(format!("sdk {content_type} validate"), "Validate source metadata and compatibility requirements."),
            LeafCommand::New { .. } => source_spec(format!("sdk {content_type} new"), "Create a new source project."),
            LeafCommand::Init { .. } => source_spec(format!("sdk {content_type} init"), "Initialize the current empty directory as a source project."),
            LeafCommand::Build { .. } => build_spec(format!("sdk {content_type} build"), "Build source artifacts."),
            LeafCommand::Package { .. } => build_spec(format!("sdk {content_type} package"), "Package content for distribution."),
            LeafCommand::Publish { .. } => publish_spec(format!("sdk {content_type} publish"), "Publish content through configured release channels."),
        }
    }

    fn read_spec(action: impl Into<String>, summary: &'static str) -> CommandSpec {
        spec(action, summary, StateSurface::ReadOnly, &[], &["display requested information"])
    }

    fn source_spec(action: impl Into<String>, summary: &'static str) -> CommandSpec {
        spec(
            action,
            summary,
            StateSurface::AuthoredSource,
            &["content identity is valid for this command"],
            &["create or update source files when implemented"],
        )
    }

    fn composition_spec(action: impl Into<String>, summary: &'static str) -> CommandSpec {
        spec(
            action,
            summary,
            StateSurface::AuthoredComposition,
            &["target pack source is available", "child type is allowed by the parent pack type"],
            &["update authored pack membership or active selection when implemented"],
        )
    }

    fn build_spec(action: impl Into<String>, summary: &'static str) -> CommandSpec {
        spec(
            action,
            summary,
            StateSurface::BuildArtifact,
            &["source project is available", "pinned toolchain is available"],
            &["produce or update build/package artifacts when implemented"],
        )
    }

    fn publish_spec(action: impl Into<String>, summary: &'static str) -> CommandSpec {
        spec(
            action,
            summary,
            StateSurface::Publication,
            &["source project has releasable artifacts", "release channel credentials are available"],
            &["publish content through configured release channels when implemented"],
        )
    }

    fn spec(
        action: impl Into<String>,
        summary: &'static str,
        surface: StateSurface,
        preconditions: &'static [&'static str],
        future_effects: &'static [&'static str],
    ) -> CommandSpec {
        CommandSpec {
            action: action.into(),
            summary,
            surface,
            preconditions,
            future_effects,
        }
    }

    fn print_stub(globals: GlobalOptions, spec: CommandSpec) {
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
}
