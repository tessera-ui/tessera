use std::{
    collections::{HashMap, HashSet},
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use ra_ap_base_db::all_crates;
use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::crate_def_map;
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, CargoFeatures, RustLibSource};
use ra_ap_vfs::{FileId, Vfs, VfsPath};

use super::{
    CheckOptions, FeatureSelection,
    progress::Progress,
    types::{LoadedWorkspace, SelectedSources},
    utils::tessera_crates,
};

pub(crate) fn load_workspace(
    options: &CheckOptions<'_>,
    progress: &Progress,
) -> Result<LoadedWorkspace> {
    let root = env::current_dir().context("Failed to resolve current directory")?;
    load_workspace_from(options, progress, &root)
}

pub(crate) fn load_workspace_from(
    options: &CheckOptions<'_>,
    progress: &Progress,
    root: &Path,
) -> Result<LoadedWorkspace> {
    let mut cargo_config = CargoConfig {
        all_targets: options.target_selection.requires_cargo_all_targets(),
        features: cargo_features(&options.features),
        sysroot: Some(RustLibSource::Discover),
        target: options.target.map(str::to_string),
        set_test: options.target_selection.tests || options.target_selection.all_targets,
        ..CargoConfig::default()
    };
    if let Some(package) = options.package {
        cargo_config.extra_args.push("--package".to_string());
        cargo_config.extra_args.push(package.to_string());
        cargo_config.run_build_script_command = Some(build_script_command(options, package));
    }
    if options.release {
        cargo_config.extra_args.push("--release".to_string());
        cargo_config.cfg_overrides.global = CfgDiff::new(
            Vec::new(),
            vec![CfgAtom::Flag(Symbol::intern("debug_assertions"))],
        );
    }
    if options.workspace {
        cargo_config.extra_args.push("--workspace".to_string());
    }
    if let Some(jobs) = options.jobs {
        cargo_config
            .extra_args
            .extend(["--jobs".to_string(), jobs.to_string()]);
    }

    let load_config = LoadCargoConfig {
        // Generated configuration handles require macro expansion to resolve
        // saved values and parameters across ordinary configuration helpers.
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };

    let (db, vfs, proc_macro_client) =
        load_workspace_at(root, &cargo_config, &load_config, &|message| {
            progress.workspace(&message)
        })
        .context("Failed to load rust-analyzer workspace for CallGuard")?;
    drop(proc_macro_client);

    Ok(LoadedWorkspace {
        host: AnalysisHost::with_database(db),
        vfs,
    })
}

fn build_script_command(options: &CheckOptions<'_>, package: &str) -> Vec<String> {
    let mut command = vec![
        "cargo".to_string(),
        "check".to_string(),
        "--quiet".to_string(),
        "--message-format=json".to_string(),
        "--keep-going".to_string(),
        "--package".to_string(),
        package.to_string(),
    ];
    if options.release {
        command.push("--release".to_string());
    }
    if let Some(jobs) = options.jobs {
        command.extend(["--jobs".to_string(), jobs.to_string()]);
    }
    if let Some(target) = options.target {
        command.extend(["--target".to_string(), target.to_string()]);
    }
    if options.target_selection.all_targets {
        command.push("--all-targets".to_string());
    } else {
        let selection = &options.target_selection;
        for (selected, flag) in [
            (selection.lib, "--lib"),
            (selection.bins, "--bins"),
            (selection.examples, "--examples"),
            (selection.tests, "--tests"),
            (selection.benches, "--benches"),
        ] {
            if selected {
                command.push(flag.to_string());
            }
        }
    }
    match &options.features {
        FeatureSelection::All => command.push("--all-features".to_string()),
        FeatureSelection::Selected {
            features,
            no_default_features,
        } => {
            if *no_default_features {
                command.push("--no-default-features".to_string());
            }
            if !features.is_empty() {
                command.extend(["--features".to_string(), features.join(",")]);
            }
        }
    }
    command
}

pub(crate) fn local_files_for_roots(
    db: &RootDatabase,
    vfs: &Vfs,
    target_roots: &[PathBuf],
) -> Result<HashSet<FileId>> {
    let roots = target_roots.iter().map(|root| file_id_for_path(vfs, root).ok_or_else(|| anyhow!("rust-analyzer did not load target {}; refusing to proceed with unchecked calls", root.display()))).collect::<Result<HashSet<_>>>()?;
    let loaded_roots = all_crates(db)
        .iter()
        .map(|krate| krate.root_file_id(db).file_id(db))
        .collect::<HashSet<_>>();
    if !roots.is_subset(&loaded_roots) {
        return Err(anyhow!(
            "rust-analyzer did not resolve every selected target crate; refusing to proceed with unchecked calls"
        ));
    }
    let files = all_crates(db)
        .iter()
        .filter(|krate| roots.contains(&krate.root_file_id(db).file_id(db)))
        .flat_map(|krate| {
            crate_def_map(db, *krate)
                .modules()
                .filter_map(|(_, module)| module.definition_source_file_id().file_id())
                .map(|file| file.file_id(db))
        })
        .collect::<HashSet<_>>();
    if files.is_empty() {
        return Err(anyhow!(
            "rust-analyzer did not load the selected target modules; refusing to proceed with unchecked calls"
        ));
    }
    Ok(files)
}

pub(crate) fn sources_for_roots(
    db: &RootDatabase,
    vfs: &Vfs,
    target_roots: &[PathBuf],
) -> Result<SelectedSources> {
    let target_root_files = target_roots
        .iter()
        .filter_map(|root| file_id_for_path(vfs, root))
        .collect::<HashSet<_>>();
    let allowed_crates = all_crates(db)
        .iter()
        .copied()
        .filter(|krate| target_root_files.contains(&krate.root_file_id(db).file_id(db)))
        .collect::<HashSet<_>>();

    let local_files = local_files_for_roots(db, vfs, target_roots)?;

    let dependencies = allowed_crates
        .iter()
        .flat_map(|krate| krate.transitive_deps(db))
        .collect::<HashSet<_>>();
    let sensitive_crates = tessera_crates(db)
        .into_iter()
        .flat_map(|krate| krate.transitive_rev_deps(db).into_vec())
        .collect::<HashSet<_>>();
    let mut file_crates = HashMap::<_, HashSet<_>>::new();
    // Only crates in the selected dependency graph can supply call targets.
    // Include third-party Tessera crates, but avoid expanding unrelated crates
    // or traversing every source file in the VFS.
    for krate in dependencies
        .intersection(&sensitive_crates)
        .chain(allowed_crates.iter())
        .copied()
        .collect::<HashSet<_>>()
    {
        for (_, module) in crate_def_map(db, krate).modules() {
            if let Some(file_id) = module.definition_source_file_id().file_id() {
                file_crates
                    .entry(file_id.file_id(db))
                    .or_default()
                    .insert(krate);
            }
        }
    }

    Ok(SelectedSources {
        local_files,
        file_crates,
    })
}

fn cargo_features(selection: &FeatureSelection<'_>) -> CargoFeatures {
    match selection {
        FeatureSelection::All => CargoFeatures::All,
        FeatureSelection::Selected {
            features,
            no_default_features,
        } => CargoFeatures::Selected {
            features: (*features).to_vec(),
            no_default_features: *no_default_features,
        },
    }
}

fn file_id_for_path(vfs: &Vfs, path: &Path) -> Option<FileId> {
    let path = normalize_path(path);
    let vfs_path = VfsPath::new_real_path(path.display().to_string());
    vfs.file_id(&vfs_path).map(|(file_id, _)| file_id)
}

/// Normalizes a path so it can be compared against rust-analyzer VFS paths.
///
/// rust-analyzer deliberately never canonicalizes paths; it mirrors the paths
/// reported by Cargo. We therefore must not call `std::fs::canonicalize` here:
/// on Windows that prepends a verbatim prefix such as `\\?\C:` which never
/// matches the VFS representation and makes every path comparison fail. This
/// defensively strips an existing verbatim prefix while otherwise keeping the
/// path untouched.
fn normalize_path(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(rest.to_string());
    }
    path.to_path_buf()
}
#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
    };

    use super::super::TargetSelection;
    use super::*;

    #[test]
    fn build_script_check_ignores_unselected_workspace_members() {
        let fixture = FixtureWorkspace::new();
        let unselected = fixture.root.join("unselected");
        fs::create_dir_all(unselected.join("src")).expect("unselected source directory");
        fs::write(
            fixture.root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\", \"unselected\"]\nresolver = \"3\"\n",
        )
        .expect("workspace manifest");
        fs::write(
            unselected.join("Cargo.toml"),
            "[package]\nname = \"unselected\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("unselected manifest");
        fs::write(
            unselected.join("src/lib.rs"),
            "compile_error!(\"the unselected package must not be checked\");\n",
        )
        .expect("unselected source");

        let command = build_script_command(&CheckOptions::new(Some("member"), None), "member");
        let output = Command::new(&command[0])
            .args(&command[1..])
            .current_dir(&fixture.root)
            .output()
            .expect("selected package check should run");
        assert!(
            output.status.success(),
            "selected package check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let messages = String::from_utf8(output.stdout).expect("Cargo messages should be UTF-8");
        assert!(messages.contains("\"name\":\"member\""));
        assert!(!messages.contains("\"name\":\"unselected\""));
    }

    #[test]
    fn build_script_check_preserves_cross_target_feature_selection() {
        let features = vec!["member/extra".to_string(), "member/other".to_string()];
        let mut options = CheckOptions::new(Some("member"), Some("wasm32-unknown-unknown"));
        options.target_selection = TargetSelection::lib_only();
        options.features = FeatureSelection::Selected {
            features: &features,
            no_default_features: true,
        };
        let command = build_script_command(&options, "member");
        assert!(
            command
                .windows(2)
                .any(|args| args == ["--target", "wasm32-unknown-unknown"])
        );
        assert!(
            command
                .windows(2)
                .any(|args| args == ["--features", "member/extra,member/other"])
        );
        assert!(command.iter().any(|arg| arg == "--lib"));
        assert!(command.iter().any(|arg| arg == "--no-default-features"));
        assert!(
            !command
                .iter()
                .any(|arg| arg == "--workspace" || arg == "--all-targets")
        );
    }

    #[test]
    fn normalize_path_strips_verbatim_prefix() {
        assert_eq!(
            normalize_path(Path::new(r"\\?\C:\workspace\example")),
            PathBuf::from(r"C:\workspace\example")
        );
        assert_eq!(
            normalize_path(Path::new(r"\\?\UNC\server\share\example")),
            PathBuf::from(r"\\server\share\example")
        );
        assert_eq!(
            normalize_path(Path::new(r"C:\workspace\example")),
            PathBuf::from(r"C:\workspace\example")
        );
    }

    struct FixtureWorkspace {
        root: PathBuf,
        _directory: tempfile::TempDir,
    }

    impl FixtureWorkspace {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("isolated checker fixture");
            let root = directory.path().to_path_buf();
            let member_dir = root.join("member");
            fs::create_dir_all(member_dir.join("src"))
                .expect("fixture member src directory should be created");
            fs::write(
                root.join("Cargo.toml"),
                "[workspace]\nmembers = [\"member\"]\nresolver = \"3\"\n",
            )
            .expect("fixture workspace manifest should be written");
            fs::write(
                member_dir.join("Cargo.toml"),
                "[package]\nname = \"member\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
            )
            .expect("fixture package manifest should be written");
            fs::write(member_dir.join("src/lib.rs"), "")
                .expect("fixture package source should be written");

            Self {
                root,
                _directory: directory,
            }
        }
    }
}
