use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env, fs,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::SystemTime,
};

use anyhow::{Context, Result};
use ra_ap_base_db::{SourceDatabase, all_crates, salsa::Setter};
use ra_ap_cfg::{CfgAtom, CfgDiff};
use ra_ap_hir::ChangeWithProcMacros;
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_intern::Symbol;
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::{
    CargoConfig, CargoFeatures, ProjectManifest, ProjectWorkspace, RustLibSource,
};
use ra_ap_vfs::{AbsPathBuf, FileId, Vfs, VfsPath};
use sha2::{Digest, Sha256};

use super::{
    CheckOptions, FeatureSelection, plan::Plan, types::LoadedWorkspace, utils::tessera_crates,
    watchdog,
};

#[cfg(test)]
pub(crate) fn load_workspace_from(
    options: &CheckOptions<'_>,
    root: &Path,
) -> Result<LoadedWorkspace> {
    let plan = Plan::resolve_at(options, root)?;
    load(options, root, &plan, None)
}

pub(crate) fn load(
    options: &CheckOptions<'_>,
    root: &Path,
    plan: &Plan,
    previous: Option<LoadedWorkspace>,
) -> Result<LoadedWorkspace> {
    let root = AbsPathBuf::assert_utf8(env::current_dir()?.join(root));
    let manifest = ProjectManifest::discover_single(&root)?;
    let (messages, checked) = run_cargo_check(
        options,
        AsRef::<Path>::as_ref(&root),
        manifest.manifest_path().as_ref(),
    )?;
    let messages = deterministic_build_script_runs(&messages);
    let mut native_artifacts = Vec::new();
    let mut out_dirs = Vec::new();
    for message in cargo_metadata::Message::parse_stream(messages.as_slice()) {
        match message {
            Ok(cargo_metadata::Message::CompilerArtifact(artifact)) => {
                native_artifacts.push(artifact)
            }
            Ok(cargo_metadata::Message::BuildScriptExecuted(run)) => {
                out_dirs.push(run.out_dir.into_std_path_buf())
            }
            _ => {}
        }
    }
    let reload_key = reload_key(plan, &messages, &native_artifacts);
    if let Some(mut workspace) = previous
        && workspace.reload_key == reload_key
        && refresh(&mut workspace)
    {
        workspace.cacheable = checked && workspace.build_scripts_ok;
        workspace.native_artifacts = native_artifacts;
        return Ok(workspace);
    }

    let mut cargo_config = CargoConfig {
        wrap_rustc_in_build_scripts: false,
        all_targets: options.target_selection.requires_cargo_all_targets(),
        features: cargo_features(&options.features),
        sysroot: Some(RustLibSource::Discover),
        target: options.target.map(str::to_string),
        metadata_extra_args: vec!["--filter-platform".to_string(), super::plan::host_target()?],
        ..CargoConfig::default()
    };
    if let Some(package) = options.package {
        cargo_config.extra_args.push("--package".to_string());
        cargo_config.extra_args.push(package.to_string());
    }
    if options.release {
        cargo_config.extra_args.push("--release".to_string());
        cargo_config.cfg_overrides.global = CfgDiff::new(
            Vec::new(),
            vec![CfgAtom::Flag(Symbol::intern("debug_assertions"))],
        );
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
        // Every analysis worker expands attribute macros through this pool.
        proc_macro_processes: super::worker_count(options.jobs)?.clamp(1, 4),
    };

    let mut project = ProjectWorkspace::load(manifest, &cargo_config, &|_| {})?;
    watchdog::progress();
    // rust-analyzer would otherwise run Cargo again from the workspace root,
    // which selects different packages and features than this check.
    let mut recorded = tempfile::Builder::new()
        .prefix("tessera-cargo-messages-")
        .tempfile()
        .context("Failed to record Cargo check messages")?;
    recorded.write_all(&messages)?;
    recorded.flush()?;
    let recorded_path = recorded.path().display().to_string();
    cargo_config.run_build_script_command = Some(if cfg!(windows) {
        vec!["cmd".into(), "/C".into(), "type".into(), recorded_path]
    } else {
        vec!["cat".into(), recorded_path]
    });
    let build_scripts = project.run_build_scripts(&cargo_config, &|_| {})?;
    drop(recorded);
    let build_scripts_ok = build_scripts.error().is_none();
    project.set_build_scripts(build_scripts);
    let (mut db, vfs, proc_macro_client) =
        ra_ap_load_cargo::load_workspace(project.clone(), &cargo_config.extra_env, &load_config)
            .context("Failed to load rust-analyzer workspace for CallGuard")?;
    watchdog::progress();
    for krate in all_crates(&db).iter().copied().collect::<Vec<_>>() {
        let root = vfs.file_path(krate.data(&db).root_file_id).to_string();
        let units = native_artifacts
            .iter()
            .filter(|artifact| artifact.target.src_path.as_str() == root)
            .collect::<Vec<_>>();
        let test = units.iter().any(|artifact| artifact.profile.test);
        // `cargo metadata` unifies features across the whole workspace, while
        // the check builds only the selected packages. Analyze the features
        // Cargo actually compiled when that choice is unambiguous.
        let mut feature_sets = units
            .iter()
            .filter(|artifact| artifact.profile.test == test)
            .map(|artifact| {
                let mut features = artifact.features.clone();
                features.sort();
                features
            })
            .collect::<Vec<_>>();
        feature_sets.sort();
        feature_sets.dedup();
        let features = match feature_sets.as_slice() {
            [features] => Some(features),
            _ => None,
        };
        if !test && features.is_none() {
            continue;
        }
        let mut atoms = krate
            .cfg_options(&db)
            .into_iter()
            .filter(|atom| {
                features.is_none()
                    || !matches!(atom, CfgAtom::KeyValue { key, .. } if key.as_str() == "feature")
            })
            .cloned()
            .collect::<Vec<_>>();
        atoms.extend(
            features
                .into_iter()
                .flatten()
                .map(|feature| CfgAtom::KeyValue {
                    key: Symbol::intern("feature"),
                    value: Symbol::intern(feature),
                }),
        );
        if test {
            atoms.push(CfgAtom::Flag(Symbol::intern("test")));
        }
        let cfg = atoms.into_iter().collect::<ra_ap_cfg::CfgOptions>();
        if cfg != *krate.cfg_options(&db) {
            krate.set_cfg_options(&mut db).to(cfg);
        }
    }
    let (graph, macros) = project.to_crate_graph(
        &mut |path| {
            vfs.file_id(&VfsPath::from(path.to_path_buf()))
                .map(|(file, _)| file)
        },
        &cargo_config.extra_env,
    );
    let mut macro_artifacts = HashMap::<_, Vec<_>>::new();
    for (krate, artifact) in macros {
        if let Ok((_, path)) = artifact {
            macro_artifacts
                .entry(graph[krate].basic.root_file_id)
                .or_default()
                .push(PathBuf::from(path.as_str()));
        }
    }
    for paths in macro_artifacts.values_mut() {
        paths.sort();
        paths.dedup();
    }
    drop(proc_macro_client);
    let source_directories = local_sources(&db, &vfs, &out_dirs)
        .into_iter()
        .filter_map(|(_, _, path)| Some(path.parent()?.to_path_buf()))
        .collect::<BTreeSet<_>>();

    Ok(LoadedWorkspace {
        host: AnalysisHost::with_database(db),
        vfs,
        macro_artifacts,
        native_artifacts,
        cacheable: checked && build_scripts_ok,
        build_scripts_ok,
        reload_key,
        out_dirs,
        rust_files: rust_files(&source_directories),
        source_directories,
    })
}

/// Fingerprints every input of a load other than source text.
///
/// Manifests, build script results, compiled units, and proc-macro libraries
/// all change the crate graph or macro expansion, which source file updates
/// cannot express.
fn reload_key(plan: &Plan, messages: &[u8], artifacts: &[cargo_metadata::Artifact]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plan.metadata.as_bytes());
    let mut runs = messages
        .split(|byte| *byte == b'\n')
        .filter(|line| {
            matches!(
                serde_json::from_slice::<cargo_metadata::Message>(line),
                Ok(cargo_metadata::Message::BuildScriptExecuted(_))
            )
        })
        .collect::<Vec<_>>();
    runs.sort();
    for run in runs {
        hasher.update(run);
    }
    let mut units = artifacts
        .iter()
        .map(|artifact| {
            let mut features = artifact.features.clone();
            features.sort();
            let libraries = if artifact.target.is_proc_macro() {
                artifact
                    .filenames
                    .iter()
                    .map(|path| {
                        let modified = fs::metadata(path)
                            .and_then(|metadata| metadata.modified())
                            .ok()
                            .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok());
                        (path.to_string(), modified)
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (
                artifact.package_id.to_string(),
                artifact.target.src_path.to_string(),
                features,
                artifact.profile.test,
                libraries,
            )
        })
        .collect::<Vec<_>>();
    units.sort();
    hasher.update(serde_json::to_vec(&units).expect("reload key"));
    hex::encode(hasher.finalize())
}

/// Lists the loaded files of workspace packages and build script outputs.
fn local_sources<'a>(
    db: &RootDatabase,
    vfs: &'a Vfs,
    out_dirs: &[PathBuf],
) -> Vec<(FileId, &'a VfsPath, PathBuf)> {
    vfs.iter()
        .filter_map(|(file, path)| {
            let local = PathBuf::from(path.as_path()?.as_str());
            let root = db.file_source_root(file).source_root_id(db);
            (!db.source_root(root).source_root(db).is_library
                || out_dirs
                    .iter()
                    .any(|directory| local.starts_with(directory)))
            .then_some((file, path, local))
        })
        .collect()
}

/// Lists Rust files in `directories` and in their subdirectories that hold no
/// loaded file.
fn rust_files(directories: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    fn collect(directory: &Path, known: &BTreeSet<PathBuf>, files: &mut BTreeSet<PathBuf>) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !known.contains(&path) && name != "target" && !name.starts_with('.') {
                    collect(&path, known, files);
                }
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.insert(path);
            }
        }
    }
    let mut files = BTreeSet::new();
    for directory in directories {
        collect(directory, directories, &mut files);
    }
    files
}

/// Brings a loaded workspace up to date with source files on disk.
///
/// Returns `false` when a Rust file was added or removed, which changes module
/// discovery and requires a fresh load.
fn refresh(workspace: &mut LoadedWorkspace) -> bool {
    if rust_files(&workspace.source_directories) != workspace.rust_files {
        return false;
    }
    let db = workspace.host.raw_database();
    let mut changes = Vec::new();
    for (file, path, local) in local_sources(db, &workspace.vfs, &workspace.out_dirs) {
        let Ok(Ok(text)) = fs::read(&local).map(String::from_utf8) else {
            return false;
        };
        if db.file_text(file).text(db).as_bytes() != text.as_bytes() {
            changes.push((file, path.clone(), text));
        }
    }
    if changes.is_empty() {
        return true;
    }
    let mut change = ChangeWithProcMacros::default();
    for (file, path, text) in changes {
        workspace
            .vfs
            .set_file_contents(path, Some(text.clone().into_bytes()));
        change.change_file(file, Some(text));
    }
    let _ = workspace.vfs.take_changes();
    workspace.host.apply_change(change);
    true
}

/// Keeps one build script run per package.
///
/// A package built both as a build dependency and as a normal dependency runs
/// its build script twice. rust-analyzer keeps the last run it reads, and Cargo
/// reports the runs in scheduling order, so the run with the greatest output
/// directory is kept to make analysis inputs reproducible.
fn deterministic_build_script_runs(messages: &[u8]) -> Vec<u8> {
    let run = |line: &[u8]| match serde_json::from_slice::<cargo_metadata::Message>(line) {
        Ok(cargo_metadata::Message::BuildScriptExecuted(run)) => Some(run),
        _ => None,
    };
    let mut kept = HashMap::new();
    for line in messages.split(|byte| *byte == b'\n') {
        if let Some(run) = run(line) {
            let out_dir = kept
                .entry(run.package_id)
                .or_insert_with(|| run.out_dir.clone());
            if run.out_dir > *out_dir {
                *out_dir = run.out_dir;
            }
        }
    }
    let mut output = Vec::with_capacity(messages.len());
    for line in messages.split(|byte| *byte == b'\n') {
        if !line.is_empty() && run(line).is_none_or(|run| kept[&run.package_id] == run.out_dir) {
            output.extend_from_slice(line);
            output.push(b'\n');
        }
    }
    output
}

/// Runs the selected `cargo check` and returns its JSON messages and success.
///
/// Cargo's status output is discarded except for lock waits, which would
/// otherwise make the checker appear to hang.
fn run_cargo_check(
    options: &CheckOptions<'_>,
    root: &Path,
    manifest: &Path,
) -> Result<(Vec<u8>, bool)> {
    let mut command = build_script_command(options);
    // Package IDs embed the manifest path as given. rust-analyzer passes its
    // manifest explicitly, while a working directory may already be resolved
    // through symlinks, so both must name the same manifest.
    command.extend([
        "--manifest-path".to_string(),
        manifest.display().to_string(),
    ]);
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to collect Cargo check artifacts")?;
    let stderr = child
        .stderr
        .take()
        .context("Cargo check stderr is unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("Cargo check stdout is unavailable")?;
    let quiet = options.quiet;
    let status_lines = thread::spawn(move || {
        for line in io::BufReader::new(stderr).lines().map_while(Result::ok) {
            watchdog::progress();
            if !quiet && line.trim_start().starts_with("Blocking") {
                eprintln!("{line}");
            }
        }
    });
    let (messages, child) = watchdog::with_cargo(child, || {
        let mut messages = Vec::new();
        let mut stdout = io::BufReader::new(stdout);
        loop {
            match stdout.read_until(b'\n', &mut messages) {
                Ok(0) => break Ok(messages),
                Ok(_) => watchdog::progress(),
                Err(error) => break Err(error),
            }
        }
    });
    let messages = messages.context("Failed to read Cargo check messages")?;
    let mut child = child.context("Cargo check was terminated")?;
    let status = child.wait().context("Failed to wait for Cargo check")?;
    let _ = status_lines.join();
    Ok((messages, status.success()))
}

fn build_script_command(options: &CheckOptions<'_>) -> Vec<String> {
    let mut command = vec![
        "cargo".to_string(),
        "check".to_string(),
        "--message-format=json".to_string(),
        "--keep-going".to_string(),
    ];
    if let Some(package) = options.package {
        command.extend(["--package".to_string(), package.to_string()]);
    } else if options.workspace {
        command.push("--workspace".to_string());
    }
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
        if let Some(binary) = &selection.bin {
            command.extend(["--bin".to_string(), binary.clone()]);
        }
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

#[cfg(test)]
pub(crate) fn relevant_crates(
    db: &RootDatabase,
    vfs: &Vfs,
    target_roots: &[PathBuf],
) -> Result<HashSet<ra_ap_base_db::Crate>> {
    let target_root_files = target_roots
        .iter()
        .filter_map(|root| file_id_for_path(vfs, root))
        .collect::<HashSet<_>>();
    let allowed_crates = all_crates(db)
        .iter()
        .copied()
        .filter(|krate| target_root_files.contains(&krate.root_file_id(db).file_id(db)))
        .collect::<HashSet<_>>();
    Ok(relevant_dependencies(db, allowed_crates))
}

pub(crate) fn relevant_dependencies(
    db: &RootDatabase,
    allowed_crates: HashSet<ra_ap_base_db::Crate>,
) -> HashSet<ra_ap_base_db::Crate> {
    let dependencies = allowed_crates
        .iter()
        .flat_map(|krate| krate.transitive_deps(db))
        .collect::<HashSet<_>>();
    let sensitive_crates = tessera_crates(db)
        .into_iter()
        .flat_map(|krate| krate.transitive_rev_deps(db).into_vec())
        .collect::<HashSet<_>>();
    dependencies
        .intersection(&sensitive_crates)
        .chain(allowed_crates.iter())
        .copied()
        .collect()
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

pub(super) fn file_id_for_path(vfs: &Vfs, path: &Path) -> Option<FileId> {
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
            "[workspace]\nmembers = [\"member\", \"unselected\"]\ndefault-members = [\"member\"]\nresolver = \"3\"\n",
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

        for package in [Some("member"), None] {
            let command = build_script_command(&CheckOptions::new(package, None));
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
            let messages =
                String::from_utf8(output.stdout).expect("Cargo messages should be UTF-8");
            assert!(messages.contains("\"name\":\"member\""));
            assert!(!messages.contains("\"name\":\"unselected\""));
        }
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
        let command = build_script_command(&options);
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
    fn refresh_applies_edited_sources_and_rejects_new_modules() {
        let fixture = FixtureWorkspace::new();
        let options = CheckOptions::new(Some("member"), None);
        let mut workspace = load_workspace_from(&options, &fixture.root).expect("fixture loads");
        let library = fixture.root.join("member/src/lib.rs");
        let file = file_id_for_path(&workspace.vfs, &library).expect("library is loaded");

        fs::write(&library, "pub fn edited() {}\n").expect("edited library");
        assert!(refresh(&mut workspace));
        let db = workspace.host.raw_database();
        assert_eq!(
            db.file_text(file).text(db).as_bytes(),
            b"pub fn edited() {}\n"
        );

        fs::write(fixture.root.join("member/src/extra.rs"), "").expect("new module");
        assert!(!refresh(&mut workspace));
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
