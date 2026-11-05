use std::{path::Path, process::Command};

use anyhow::{Result, bail};

use crate::{CheckArgs, FixArgs, call_guard};

pub fn execute(
    args: &CheckArgs,
    lints: call_guard::LintOptions,
    fix: Option<&FixArgs>,
) -> Result<()> {
    let all_targets = args.all_targets
        || (fix.is_some()
            && !(args.bin.is_some()
                || args.lib
                || args.bins
                || args.examples
                || args.tests
                || args.benches));
    let command = |action| {
        let mut command = Command::new("cargo");
        command.arg(action);
        for (flag, value) in [
            ("--package", args.package.as_deref()),
            ("--target", args.target.as_deref()),
            ("--bin", args.bin.as_deref()),
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
    if let Some(fix) = fix {
        check_workspace(&std::env::current_dir()?, fix)?;
    }
    let check = |fix| {
        call_guard::run(call_guard::CheckOptions {
            package: args.package.as_deref(),
            target: args.target.as_deref(),
            target_selection: call_guard::TargetSelection {
                lib: args.lib,
                bins: args.bins,
                bin: args.bin.clone(),
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
            message_format: call_guard::MessageFormat::from_cargo_arg(
                args.message_format.as_deref(),
            ),
            quiet: args.quiet,
            verbose: args.verbose,
            release: args.release,
            workspace: args.workspace,
            jobs: args.jobs.as_deref(),
            lints: lints.clone(),
            fix,
        })
    };
    check(fix.is_some())?;
    if let Some(fix) = fix {
        let mut command = command("fix");
        // The preflight approved the original changes; Tessera may now have edited files.
        command.arg("--allow-dirty");
        for (flag, enabled) in [
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
        check(false)?;
    }
    run(&mut command("check"))
}

fn check_workspace(root: &Path, fix: &FixArgs) -> Result<()> {
    if fix.allow_no_vcs {
        return Ok(());
    }
    let status = Command::new("git")
        .args([
            "status",
            "--porcelain=v1",
            "-z",
            "--no-renames",
            "--untracked-files=normal",
        ])
        .current_dir(root)
        .output();
    let status = match status {
        Ok(status) if status.status.success() => status.stdout,
        _ => {
            if Command::new("hg")
                .arg("root")
                .current_dir(root)
                .output()
                .is_ok_and(|output| output.status.success())
            {
                return Ok(());
            }
            bail!("no VCS found; use --allow-no-vcs to permit fixes without version control");
        }
    };
    for entry in status
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let staged = entry.get(1) == Some(&b' ');
        if !(fix.allow_dirty || (staged && fix.allow_staged)) {
            bail!(
                "workspace has {} changes; commit them or pass --allow-{}",
                if staged { "staged" } else { "unstaged" },
                if staged { "staged" } else { "dirty" }
            );
        }
    }
    Ok(())
}

fn run(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn preflight_respects_vcs_permissions_before_fixes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let mut fix = FixArgs {
            allow_dirty: false,
            allow_staged: false,
            allow_no_vcs: false,
            broken_code: false,
            edition: false,
            edition_idioms: false,
        };
        assert!(check_workspace(root, &fix).is_err());
        fix.allow_no_vcs = true;
        check_workspace(root, &fix).unwrap();
        fix.allow_no_vcs = false;

        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(root)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        git(&["init", "--quiet"]);
        check_workspace(root, &fix).unwrap();
        fs::write(root.join("source.rs"), "original").unwrap();
        assert!(check_workspace(root, &fix).is_err());
        fix.allow_staged = true;
        assert!(check_workspace(root, &fix).is_err());
        git(&["add", "source.rs"]);
        check_workspace(root, &fix).unwrap();
        fix.allow_staged = false;
        assert!(check_workspace(root, &fix).is_err());
        fix.allow_dirty = true;
        check_workspace(root, &fix).unwrap();
        fix.allow_dirty = false;
        fix.allow_staged = true;
        fs::write(root.join("source.rs"), "modified").unwrap();
        assert!(check_workspace(root, &fix).is_err());
        fix.allow_dirty = true;
        check_workspace(root, &fix).unwrap();
        fix.allow_dirty = false;
        fix.allow_no_vcs = true;
        check_workspace(root, &fix).unwrap();
    }
}
