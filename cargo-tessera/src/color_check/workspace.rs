use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use cargo_metadata::{MetadataCommand, Target, TargetKind};
use ra_ap_base_db::{Crate, all_crates};
use ra_ap_hir::crate_def_map;
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, CargoFeatures, RustLibSource};
use ra_ap_vfs::{FileId, Vfs, VfsPath};

use super::{CheckOptions, FeatureSelection, TargetSelection, types::LoadedWorkspace};

pub(crate) fn load_workspace(options: &CheckOptions<'_>) -> Result<LoadedWorkspace> {
    let root = env::current_dir().context("Failed to resolve current directory")?;
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
        // The default command also checks every workspace member, whose host
        // dependencies may be absent from the selected target's metadata.
        if options.target.is_some() {
            cargo_config.run_build_script_command = Some(build_script_command(options, package));
        }
    }

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: false,
        num_worker_threads: 1,
        proc_macro_processes: 1,
    };

    let (db, vfs, proc_macro_client) =
        load_workspace_at(&root, &cargo_config, &load_config, &|_| {})
            .context("Failed to load rust-analyzer workspace for Tessera color checking")?;
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

pub(crate) fn selected_local_files(
    db: &RootDatabase,
    vfs: &Vfs,
    options: &CheckOptions<'_>,
) -> Result<HashSet<FileId>> {
    let package_name = selected_package_name(options)?;
    let manifest_dir = selected_package_manifest_dir(package_name.as_deref())?;
    let target_roots = selected_target_roots(&manifest_dir, &options.target_selection)?;
    let target_root_files = target_roots
        .iter()
        .filter_map(|root| file_id_for_path(vfs, root))
        .collect::<HashSet<_>>();
    let allowed_crates = all_crates(db)
        .iter()
        .copied()
        .filter(|krate| crate_matches(db, *krate, package_name.as_deref(), &target_root_files))
        .collect::<HashSet<_>>();

    // Only analyze source files that belong to the selected crates' module
    // trees. Files that live in the VFS but are not declared as modules (for
    // example a stray `src/pages/foo.rs` without a matching `mod` declaration)
    // are not part of the build, so rust-analyzer cannot resolve anything in
    // them and they would otherwise produce spurious diagnostics.
    let local_files = allowed_crates
        .iter()
        .flat_map(|krate| {
            crate_def_map(db, *krate)
                .modules()
                .map(|(_, module)| module.definition_source_file_id())
        })
        .filter_map(|file_id| file_id.file_id())
        .map(|file_id| file_id.file_id(db))
        .collect::<HashSet<_>>();

    if local_files.is_empty() {
        return Err(anyhow!(
            "rust-analyzer did not load any local Rust source files for Tessera color checking"
        ));
    }

    Ok(local_files)
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

fn selected_package_name(options: &CheckOptions<'_>) -> Result<Option<String>> {
    if let Some(package) = options.package {
        return Ok(Some(package.to_string()));
    }

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to read Cargo metadata for Tessera color checking")?;
    let current_dir = env::current_dir().context("Failed to resolve current directory")?;
    let current_dir = normalize_path(&current_dir);
    let mut current_package = None;
    for package in &metadata.packages {
        let manifest_dir = package
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow!("Package manifest has no parent directory"))?;
        if current_dir.starts_with(normalize_path(manifest_dir.as_std_path())) {
            current_package = Some(package.name.to_string());
            break;
        }
    }

    Ok(current_package)
}

fn selected_package_manifest_dir(package: Option<&str>) -> Result<std::path::PathBuf> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .context("Failed to read Cargo metadata for Tessera color checking")?;
    let package = if let Some(package) = package {
        metadata
            .packages
            .iter()
            .find(|candidate| candidate.name.as_ref() == package)
            .ok_or_else(|| anyhow!("Cargo package `{package}` was not found"))?
    } else {
        metadata
            .root_package()
            .ok_or_else(|| anyhow!("Unable to determine Cargo package for color checking"))?
    };
    let manifest_dir = package
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("Package manifest has no parent directory"))?;
    Ok(normalize_path(manifest_dir.as_std_path()))
}

fn selected_target_roots(
    manifest_dir: &Path,
    target_selection: &TargetSelection,
) -> Result<HashSet<std::path::PathBuf>> {
    let manifest_path = manifest_dir.join("Cargo.toml");
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .no_deps()
        .exec()
        .with_context(|| {
            format!(
                "Failed to read Cargo metadata from {}",
                manifest_path.display()
            )
        })?;
    let normalized_manifest_path = normalize_path(&manifest_path);
    let package = metadata
        .packages
        .iter()
        .find(|candidate| {
            let candidate_manifest_path: &Path = candidate.manifest_path.as_ref();
            normalize_path(candidate_manifest_path) == normalized_manifest_path
        })
        .ok_or_else(|| {
            anyhow!(
                "Unable to find Cargo package manifest at {} for color checking",
                manifest_path.display()
            )
        })?;
    let mut roots = HashSet::new();
    for target in &package.targets {
        if target_is_selected(target, target_selection) {
            roots.insert(normalize_path(target.src_path.as_std_path()));
        }
    }

    Ok(roots)
}

fn target_is_selected(target: &Target, selection: &TargetSelection) -> bool {
    if selection.all_targets {
        return true;
    }

    let explicit = selection.lib
        || selection.bins
        || selection.examples
        || selection.tests
        || selection.benches;
    if explicit {
        return selection.lib && target_is_lib(target)
            || selection.bins && target.is_bin()
            || selection.examples && target.is_example()
            || selection.tests && target.is_test()
            || selection.benches && target.is_bench();
    }

    target_is_lib(target) || target.is_bin()
}

fn target_is_lib(target: &Target) -> bool {
    target.is_lib()
        || target.is_kind(TargetKind::RLib)
        || target.is_kind(TargetKind::CDyLib)
        || target.is_kind(TargetKind::DyLib)
        || target.is_kind(TargetKind::StaticLib)
        || target.is_proc_macro()
}

fn file_id_for_path(vfs: &Vfs, path: &Path) -> Option<FileId> {
    let path = normalize_path(path);
    let vfs_path = VfsPath::new_real_path(path.display().to_string());
    vfs.file_id(&vfs_path).map(|(file_id, _)| file_id)
}

fn crate_matches(
    db: &RootDatabase,
    krate: Crate,
    selected_package: Option<&str>,
    target_root_files: &HashSet<FileId>,
) -> bool {
    if !target_root_files.is_empty() {
        return target_root_files.contains(&krate.root_file_id(db).file_id(db));
    }

    let Some(display_name) = krate.extra_data(db).display_name.as_ref() else {
        return false;
    };
    if let Some(package) = selected_package {
        return display_name.canonical_name().as_str() == package;
    }

    true
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
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

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
    fn selected_target_roots_finds_workspace_member_manifest() {
        let fixture = FixtureWorkspace::new();
        let roots = selected_target_roots(fixture.member_dir(), &TargetSelection::default())
            .expect("workspace member target roots should be selected");
        let expected = fixture
            .member_dir()
            .join("src/lib.rs")
            .canonicalize()
            .expect("fixture source should canonicalize");
        let contains_expected = roots.iter().any(|root| {
            root.canonicalize()
                .map(|root| root == expected)
                .unwrap_or(false)
        });

        assert!(contains_expected, "selected roots: {roots:?}");
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
        member_dir: PathBuf,
    }

    impl FixtureWorkspace {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "tessera-color-check-workspace-fixture-{}-{id}",
                std::process::id()
            ));
            let member_dir = root.join("member");
            let _ = fs::remove_dir_all(&root);
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

            Self { root, member_dir }
        }

        fn member_dir(&self) -> &Path {
            &self.member_dir
        }
    }

    impl Drop for FixtureWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
