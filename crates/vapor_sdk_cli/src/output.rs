//! Output for parsed SDK commands.

use vapor_sdk_core::{
    CommandSpec, GlobalOptions, SdkCommand, ToolchainCommand, ToolchainInstallState,
    WorkspaceCargoReport, WorkspaceCommand, WorkspaceDeployReport, WorkspaceStatusReport,
    WorkspaceSyncReport, toolchain_install, toolchain_status, workspace_build, workspace_check,
    workspace_deploy, workspace_fmt, workspace_status, workspace_sync,
};

pub(crate) fn print_command(
    globals: GlobalOptions,
    command: &SdkCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = vapor_sdk_core::describe_command(command);
    crate::safety::guard(globals, command, &spec)?;

    match command {
        SdkCommand::Workspace(WorkspaceCommand::Status) => {
            print_workspace_status(globals, spec, workspace_status()?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Sync) => {
            print_workspace_sync(globals, spec, workspace_sync()?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Check) => {
            print_workspace_cargo(globals, spec, workspace_check()?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Fmt) => {
            print_workspace_cargo(globals, spec, workspace_fmt()?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Build) => {
            print_workspace_cargo(globals, spec, workspace_build()?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Deploy) => {
            print_workspace_deploy(globals, spec, workspace_deploy()?)
        }
        SdkCommand::Toolchain(ToolchainCommand::Status) => print_toolchain_status(globals, spec),
        SdkCommand::Toolchain(ToolchainCommand::Install) => print_toolchain_install(globals, spec),
        _ => {
            print_stub(globals, spec);
            Ok(())
        }
    }
}

fn print_workspace_status(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: WorkspaceStatusReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_workspace_status_report(&report);

    if globals.verbose {
        print_workspace_spec(&spec);
    }

    Ok(())
}

fn print_workspace_sync(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: WorkspaceSyncReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_workspace_status_report(&report.status);
    println!("changed_paths:");
    if report.changed_paths.is_empty() {
        println!("  none");
    } else {
        for path in report.changed_paths {
            println!("  {}", path.display());
        }
    }

    if globals.verbose {
        print_workspace_spec(&spec);
    }

    Ok(())
}

fn print_workspace_cargo(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: WorkspaceCargoReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_workspace_cargo_report(&report);

    if globals.verbose {
        print_workspace_spec(&spec);
    }

    if report.status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with {}", spec.action, report.status).into())
    }
}

fn print_workspace_deploy(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: WorkspaceDeployReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_workspace_cargo_report(&report.build);
    println!("source_executable: {}", report.source_executable.display());
    println!(
        "deployed_executable: {}",
        report.deployed_executable.display()
    );
    println!("alias_executable: {}", report.alias_executable.display());

    if globals.verbose {
        print_workspace_spec(&spec);
    }

    if report.build.status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with {}", spec.action, report.build.status).into())
    }
}

fn print_workspace_cargo_report(report: &WorkspaceCargoReport) {
    println!(
        "invocation_directory: {}",
        report.invocation_directory.display()
    );
    println!("workspace_root: {}", report.workspace_root.display());
    println!(
        "workspace_kind: {}",
        report.workspace_kind.as_deref().unwrap_or("none")
    );
    println!(
        "workspace_id: {}",
        report.workspace_id.as_deref().unwrap_or("none")
    );
    println!("working_directory: {}", report.working_directory.display());
    println!("cargo: {}", report.cargo_path.display());
    println!("rustc: {}", report.rustc_path.display());
    println!("cargo_home: {}", report.cargo_home.display());
    println!("cargo_target_dir: {}", report.target_dir.display());
    println!("cargo_args: {}", report.cargo_args.join(" "));
    println!("status: {}", report.status);
}

fn print_workspace_status_report(report: &WorkspaceStatusReport) {
    println!(
        "invocation_directory: {}",
        report.invocation_directory.display()
    );
    println!("workspace_root: {}", report.workspace_root.display());
    println!(
        "workspace_kind: {}",
        report.workspace_kind.as_deref().unwrap_or("none")
    );
    println!(
        "workspace_id: {}",
        report.workspace_id.as_deref().unwrap_or("none")
    );
    println!("cargo_manifest_exists: {}", report.cargo_manifest_exists);
    println!("crates_dir_exists: {}", report.crates_dir_exists);
}

fn print_workspace_spec(spec: &CommandSpec) {
    println!("state_surface: {:?}", spec.surface);
    print_lines("preconditions", spec.preconditions);
    print_lines("future_effects", spec.future_effects);
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

fn print_toolchain_status(
    globals: GlobalOptions,
    spec: CommandSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = toolchain_status()?;

    println!("{}", spec.summary);
    println!("toolchain: {}", status.toolchain.identifier());
    println!("host: {}", status.host_triple);
    println!("host_supported: {}", status.host_supported);
    println!(
        "vapor_home: {} ({})",
        status.vapor_home.display(),
        status.vapor_home_source.as_str()
    );
    println!("rust_toolchain_home: {}", status.toolchain_home.display());
    println!("rust_toolchain_root: {}", status.toolchain_root.display());
    println!("bootstrap_root: {}", status.bootstrap_root.display());
    println!("output_root: {}", status.output_root.display());
    println!("cargo: {}", status.cargo_path.display());
    println!("rustc: {}", status.rustc_path.display());
    print_install_state(&status.install_state);

    if globals.verbose {
        println!("state_surface: {:?}", spec.surface);
        print_lines("preconditions", spec.preconditions);
        print_lines("future_effects", spec.future_effects);
        print_lines("supported_hosts", status.supported_host_triples());
        print_lines("supported_targets", status.supported_target_triples());
        println!("required_components:");
        for component in status.toolchain.required_components() {
            println!("  {}", component.as_str());
        }
        println!("channel: {}", status.toolchain.channel);
        println!("date: {}", status.toolchain.date);
        println!(
            "{}: override with a portable/dev Vapor root",
            vapor_sdk_core::VAPOR_HOME_ENV
        );
    }

    Ok(())
}

fn print_toolchain_install(
    globals: GlobalOptions,
    spec: CommandSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = toolchain_install()?;
    let plan = &report.plan;

    println!("{}", spec.summary);
    println!("toolchain: {}", plan.status.toolchain.identifier());
    println!("host: {}", plan.status.host_triple);
    println!("manifest: {}", plan.manifest_url);
    println!("manifest_date: {}", plan.manifest_date);
    println!("vapor_home: {}", plan.status.vapor_home.display());
    println!(
        "rust_toolchain_home: {}",
        plan.status.toolchain_home.display()
    );
    println!(
        "rust_toolchain_root: {}",
        plan.status.toolchain_root.display()
    );
    println!("bootstrap_root: {}", plan.status.bootstrap_root.display());
    println!("output_root: {}", plan.status.output_root.display());
    println!("download_root: {}", plan.download_root.display());
    println!("staging_root: {}", report.staging_root.display());
    println!("installed_root: {}", report.installed_root.display());
    println!("archive_count: {}", plan.archives.len());
    println!("archives:");

    for archive in &plan.archives {
        println!("  {} {}", archive.component.as_str(), archive.target);
        println!("    package: {}", archive.package);
        println!("    url: {}", archive.url);
        println!("    sha256: {}", archive.sha256);
        println!("    download_path: {}", archive.download_path.display());
    }

    if globals.verbose {
        println!("state_surface: {:?}", spec.surface);
        print_lines("preconditions", spec.preconditions);
        print_lines("future_effects", spec.future_effects);
        print_lines("supported_hosts", plan.status.supported_host_triples());
        print_lines("supported_targets", plan.status.supported_target_triples());
        println!("install_state: {}", plan.status.install_state.as_str());
    }

    Ok(())
}

fn print_install_state(state: &ToolchainInstallState) {
    println!("install_state: {}", state.as_str());
    if let ToolchainInstallState::Broken { reason } = state {
        println!("install_problem: {reason}");
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
