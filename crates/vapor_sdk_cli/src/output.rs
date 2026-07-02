//! Output for parsed SDK commands.

use vapor_sdk_core::{
    CommandSpec, GlobalOptions, RootCommand, RootPackageReport, RootPublishReport, SdkCommand,
    SteamCommand, SteamLoginReport, SteamStatusReport, ToolchainCommand, ToolchainInstallState,
    WorkspaceCargoReport, WorkspaceCommand, WorkspaceDeployReport, WorkspaceStatusReport,
    WorkspaceSyncReport, root_package, root_publish, steam_login, steam_status, toolchain_install,
    toolchain_status, workspace_build, workspace_check, workspace_deploy, workspace_fmt,
    workspace_status, workspace_sync,
};

pub(crate) fn print_command(
    globals: GlobalOptions,
    command: &SdkCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = vapor_sdk_core::describe_command(command);
    crate::safety::guard(&globals, command, &spec)?;

    match command {
        SdkCommand::Workspace(WorkspaceCommand::Status) => {
            print_workspace_status(globals.clone(), spec, workspace_status(&globals)?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Sync) => {
            print_workspace_sync(globals.clone(), spec, workspace_sync(&globals)?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Check) => {
            print_workspace_cargo(globals.clone(), spec, workspace_check(&globals)?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Fmt) => {
            print_workspace_cargo(globals.clone(), spec, workspace_fmt(&globals)?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Build) => {
            print_workspace_cargo(globals.clone(), spec, workspace_build(&globals)?)
        }
        SdkCommand::Workspace(WorkspaceCommand::Deploy) => {
            print_workspace_deploy(globals.clone(), spec, workspace_deploy(&globals)?)
        }
        SdkCommand::Root(RootCommand::Package(request)) => {
            print_root_package(globals.clone(), spec, root_package(request)?)
        }
        SdkCommand::Root(RootCommand::Publish(request)) => {
            print_root_publish(globals.clone(), spec, root_publish(request)?)
        }
        SdkCommand::Steam(SteamCommand::Status(request)) => {
            print_steam_status(globals.clone(), spec, steam_status(request)?)
        }
        SdkCommand::Steam(SteamCommand::Login(request)) => {
            print_steam_login(globals.clone(), spec, steam_login(request)?)
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
    print_workspace_status_summary(&report);

    if globals.verbose {
        print_workspace_status_report(&report);
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
    print_workspace_status_summary(&report.status);
    println!("changed_paths: {}", report.changed_paths.len());

    if globals.verbose {
        print_workspace_status_report(&report.status);
        println!("changed_path_details:");
        if report.changed_paths.is_empty() {
            println!("  none");
        } else {
            for path in &report.changed_paths {
                println!("  {}", path.display());
            }
        }
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
    print_workspace_cargo_summary(&report);

    if globals.verbose {
        print_workspace_cargo_report(&report);
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
    print_workspace_cargo_summary(&report.build);
    println!("alias: {}", report.alias_executable.display());
    println!("binary: {}", report.deployed_executable.display());
    println!("env: {}", report.activation_script.display());

    if globals.verbose {
        print_workspace_cargo_report(&report.build);
        println!("source_executable: {}", report.source_executable.display());
        println!(
            "deployed_executable: {}",
            report.deployed_executable.display()
        );
        println!("alias_executable: {}", report.alias_executable.display());
        println!("activation_script: {}", report.activation_script.display());
        print_workspace_spec(&spec);
    }

    if report.build.status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with {}", spec.action, report.build.status).into())
    }
}

fn print_root_package(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: RootPackageReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_root_package_summary(&report);

    if globals.verbose {
        print_root_package_report(&report);
        print_workspace_spec(&spec);
    }

    Ok(())
}

fn print_root_publish(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: RootPublishReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_root_package_summary(&report.package);
    println!("steamcmd: {}", report.steamcmd.display());
    println!("account: {}", report.account);

    match report.status {
        Some(status) => {
            println!("status: {}", status_label(status));
            if !status.success() {
                return Err(format!("{} failed with {}", spec.action, status).into());
            }
        }
        None => println!("status: planned"),
    }

    if globals.verbose {
        print_root_package_report(&report.package);
        println!("steam_args: {}", report.steam_args.join(" "));
        println!(
            "set_live: {}",
            report.set_live.as_deref().unwrap_or("manual")
        );
        print_workspace_spec(&spec);
    }

    Ok(())
}

fn print_root_package_report(report: &RootPackageReport) {
    println!("planned: {}", report.planned);
    println!("app_id: {}", report.app_id);
    println!("depot_id: {}", report.depot_id);
    println!("description: {}", report.description);
    println!(
        "set_live: {}",
        report.set_live.as_deref().unwrap_or("manual")
    );
    println!("vapor_home: {}", report.vapor_home.display());
    println!("package_root: {}", report.package_root.display());
    println!("content_root: {}", report.content_root.display());
    println!("scripts_root: {}", report.scripts_root.display());
    println!("steam_build_root: {}", report.steam_build_root.display());
    println!("app_build_script: {}", report.app_build_script.display());
    println!("copied_files: {}", report.copied_files);
    println!("included_roots:");
    for root in &report.included_roots {
        println!("  {}", root.display());
    }
}

fn print_root_package_summary(report: &RootPackageReport) {
    println!("app: {} depot: {}", report.app_id, report.depot_id);
    println!("platform: {}", report.host_triple);
    println!("package: {}", report.package_root.display());
    println!("files: {}", report.copied_files);
    println!(
        "status: {}",
        if report.planned { "planned" } else { "ready" }
    );
}

fn print_steam_status(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: SteamStatusReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_steam_status_summary(&report);

    if globals.verbose {
        print_steam_status_report(&report);
        print_workspace_spec(&spec);
    }

    Ok(())
}

fn print_steam_login(
    globals: GlobalOptions,
    spec: CommandSpec,
    report: SteamLoginReport,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", spec.summary);
    print_steam_status_summary(&report.status);
    println!("account: {}", report.account);
    println!("status: {}", status_label(report.exit_status));

    if globals.verbose {
        print_steam_status_report(&report.status);
        println!("steam_args: {}", report.steam_args.join(" "));
        print_workspace_spec(&spec);
    }

    if report.exit_status.success() {
        Ok(())
    } else {
        Err(format!("{} failed with {}", spec.action, report.exit_status).into())
    }
}

fn print_steam_status_report(report: &SteamStatusReport) {
    println!("app_id: {}", report.app_id);
    println!("vapor_home: {}", report.vapor_home.display());
    println!("steam_home: {}", report.steam_home.display());
    println!(
        "steamcmd_state_root: {}",
        report.steamcmd_state_root.display()
    );
    println!(
        "requested_steamcmd: {}",
        report.requested_steamcmd.display()
    );
    println!(
        "resolved_steamcmd: {}",
        report
            .resolved_steamcmd
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "none".to_owned())
    );
    println!("steamcmd_source: {}", report.steamcmd_source.as_str());
    println!("auth_model: {}", report.auth_model);
}

fn print_steam_status_summary(report: &SteamStatusReport) {
    println!("app: {}", report.app_id);
    println!(
        "steamcmd: {}",
        report
            .resolved_steamcmd
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unavailable".to_owned())
    );
    println!("source: {}", report.steamcmd_source.as_str());
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

fn print_workspace_cargo_summary(report: &WorkspaceCargoReport) {
    println!(
        "workspace: {} ({})",
        report.workspace_id.as_deref().unwrap_or("none"),
        report.workspace_kind.as_deref().unwrap_or("none")
    );
    println!("command: cargo {}", report.cargo_args.join(" "));
    println!("status: {}", status_label(report.status));
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

fn print_workspace_status_summary(report: &WorkspaceStatusReport) {
    println!(
        "workspace: {} ({})",
        report.workspace_id.as_deref().unwrap_or("none"),
        report.workspace_kind.as_deref().unwrap_or("none")
    );
    println!("root: {}", report.workspace_root.display());
    println!(
        "cargo_manifest: {}",
        present_label(report.cargo_manifest_exists)
    );
    println!("crates_dir: {}", present_label(report.crates_dir_exists));
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

fn status_label(status: std::process::ExitStatus) -> String {
    if status.success() {
        "ok".to_owned()
    } else {
        status.to_string()
    }
}

fn present_label(present: bool) -> &'static str {
    if present { "yes" } else { "no" }
}

fn print_toolchain_status(
    globals: GlobalOptions,
    spec: CommandSpec,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = toolchain_status()?;

    println!("{}", spec.summary);
    println!("toolchain: {}", status.toolchain.identifier());
    println!("host: {}", status.host_triple);
    print_install_state(&status.install_state);
    println!(
        "home: {} ({})",
        status.vapor_home.display(),
        status.vapor_home_source.as_str()
    );

    if globals.verbose {
        println!("host_supported: {}", status.host_supported);
        println!("rust_toolchain_home: {}", status.toolchain_home.display());
        println!("rust_toolchain_root: {}", status.toolchain_root.display());
        println!("bootstrap_root: {}", status.bootstrap_root.display());
        println!("output_root: {}", status.output_root.display());
        println!("cargo: {}", status.cargo_path.display());
        println!("rustc: {}", status.rustc_path.display());
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
    println!("installed_root: {}", report.installed_root.display());
    println!("archives: {}", plan.archives.len());

    if globals.verbose {
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
        println!("archive_details:");
        for archive in &plan.archives {
            println!("  {} {}", archive.component.as_str(), archive.target);
            println!("    package: {}", archive.package);
            println!("    url: {}", archive.url);
            println!("    sha256: {}", archive.sha256);
            println!("    download_path: {}", archive.download_path.display());
        }
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
