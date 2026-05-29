use std::{collections::HashSet, env, path::Path};

use anyhow::{Context, Result, anyhow};
use cargo_metadata::{MetadataCommand, Target, TargetKind};
use ra_ap_base_db::{Crate, all_crates};
use ra_ap_ide::AnalysisHost;
use ra_ap_ide::RootDatabase;
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

    let mut local_files = HashSet::new();
    for (file_id, path) in vfs.iter() {
        let Some(path) = path.as_path() else {
            continue;
        };
        let std_path: &Path = path.as_ref();
        if !std_path.extension().is_some_and(|ext| ext == "rs") {
            continue;
        }
        if !std_path.starts_with(&manifest_dir) {
            continue;
        }
        let relevant_crates = ra_ap_base_db::relevant_crates(db, file_id);
        if relevant_crates
            .iter()
            .any(|krate| allowed_crates.contains(krate))
        {
            local_files.insert(file_id);
        }
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
    let current_dir = current_dir
        .canonicalize()
        .context("Failed to canonicalize current directory")?;
    let mut current_package = None;
    for package in &metadata.packages {
        let manifest_dir = package
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow!("Package manifest has no parent directory"))?;
        if current_dir.starts_with(manifest_dir) {
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
    manifest_dir
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize package directory `{manifest_dir}`"))
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
    let package = metadata
        .root_package()
        .ok_or_else(|| anyhow!("Unable to determine Cargo package for color checking"))?;
    let mut roots = HashSet::new();
    for target in &package.targets {
        if target_is_selected(target, target_selection) {
            roots.insert(target.src_path.clone().into_std_path_buf());
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
    let path = path.canonicalize().ok()?;
    let vfs_path = VfsPath::new_real_path(path.display().to_string());
    vfs.file_id(&vfs_path).map(|(file_id, _)| file_id)
}

fn crate_matches(
    db: &RootDatabase,
    krate: Crate,
    selected_package: Option<&str>,
    target_root_files: &HashSet<FileId>,
) -> bool {
    let Some(display_name) = krate.extra_data(db).display_name.as_ref() else {
        return false;
    };
    if selected_package.is_some_and(|package| display_name.canonical_name().as_str() != package) {
        return false;
    }
    if target_root_files.is_empty() {
        return true;
    }
    target_root_files.contains(&krate.root_file_id(db).file_id(db))
}
