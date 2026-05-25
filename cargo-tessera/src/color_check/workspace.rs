use std::env;

use anyhow::{Context, Result};
use ra_ap_ide::AnalysisHost;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace_at};
use ra_ap_project_model::{CargoConfig, CargoFeatures, RustLibSource};

use super::{CheckOptions, types::LoadedWorkspace};

pub(crate) fn load_workspace(options: &CheckOptions<'_>) -> Result<LoadedWorkspace> {
    let root = env::current_dir().context("Failed to resolve current directory")?;
    let mut cargo_config = CargoConfig {
        all_targets: true,
        features: CargoFeatures::All,
        sysroot: Some(RustLibSource::Discover),
        target: options
            .target
            .as_ref()
            .filter(|target| is_explicit_rust_target(target))
            .map(|t| t.to_string()),
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

pub(crate) fn is_explicit_rust_target(target: &str) -> bool {
    target.contains('-')
}
