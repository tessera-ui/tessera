use std::process::Command;

use anyhow::Result;

use crate::{CheckArgs, FixArgs, call_guard};

pub fn execute(
    args: &CheckArgs,
    lints: call_guard::LintOptions,
    fix: Option<&FixArgs>,
) -> Result<()> {
    let all_targets = args.all_targets
        || (fix.is_some()
            && !(args.lib || args.bins || args.examples || args.tests || args.benches));
    let command = |action| {
        let mut command = Command::new("cargo");
        command.arg(action);
        for (flag, value) in [
            ("--package", args.package.as_deref()),
            ("--target", args.target.as_deref()),
            ("--jobs", args.jobs.as_deref()),
            ("--message-format", args.message_format.as_deref()),
        ] {
            if let Some(value) = value {
                command.arg(flag).arg(value);
            }
        }
        for (flag, enabled) in [
            ("--release", args.release),
            ("--lib", args.lib),
            ("--bins", args.bins),
            ("--examples", args.examples),
            ("--tests", args.tests),
            ("--benches", args.benches),
            ("--all-targets", all_targets),
            ("--all-features", args.all_features),
            ("--no-default-features", args.no_default_features),
            ("--workspace", args.workspace),
            ("--verbose", args.verbose),
            ("--quiet", args.quiet),
        ] {
            if enabled {
                command.arg(flag);
            }
        }
        if !args.features.is_empty() {
            command.arg("--features").arg(args.features.join(","));
        }
        command
    };
    let cargo_fix = if let Some(fix) = fix {
        let mut command = command("fix");
        for (flag, enabled) in [
            ("--allow-dirty", fix.allow_dirty),
            ("--allow-staged", fix.allow_staged),
            ("--allow-no-vcs", fix.allow_no_vcs),
            ("--broken-code", fix.broken_code),
            ("--edition", fix.edition),
            ("--edition-idioms", fix.edition_idioms),
        ] {
            if enabled {
                command.arg(flag);
            }
        }
        run(&mut command)?;
        // The initial Cargo pass checked VCS state before either fixer wrote files.
        if !fix.allow_dirty {
            command.arg("--allow-dirty");
        }
        Some(command)
    } else {
        None
    };
    call_guard::run(call_guard::CheckOptions {
        package: args.package.as_deref(),
        target: args.target.as_deref(),
        target_selection: call_guard::TargetSelection {
            lib: args.lib,
            bins: args.bins,
            examples: args.examples,
            tests: args.tests,
            benches: args.benches,
            all_targets,
        },
        features: call_guard::FeatureSelection::from_cargo_args(
            &args.features,
            args.all_features,
            args.no_default_features,
        ),
        message_format: call_guard::MessageFormat::from_cargo_arg(args.message_format.as_deref()),
        quiet: args.quiet,
        verbose: args.verbose,
        release: args.release,
        workspace: args.workspace,
        jobs: args.jobs.as_deref(),
        lints,
        fix: fix.is_some(),
    })?;
    if let Some(mut command) = cargo_fix {
        run(&mut command)
    } else {
        run(&mut command("check"))
    }
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
