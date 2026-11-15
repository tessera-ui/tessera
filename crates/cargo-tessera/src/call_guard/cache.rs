use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use ra_ap_base_db::{Crate, SourceDatabase, SourceRootId, all_crates};
use ra_ap_cfg::CfgAtom;
use ra_ap_hir::crate_def_map;
use ra_ap_hir_expand::proc_macro::ProcMacros;
use ra_ap_ide::FileId;
use ra_ap_span::TextRange;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

use super::{
    AnalysisJob, CheckOptions, collect_index_parallel,
    plan::{CrateJob, Plan},
    types::{CallAnalyzer, CallIndex, Diagnostic, LoadedWorkspace, SelectedSources},
    utils::tessera_crates,
    workspace,
};

#[derive(Serialize, Deserialize)]
struct Record {
    input: String,
    checksum: String,
    payload: String,
}

#[derive(Serialize, Deserialize)]
struct Payload<T> {
    files: Vec<(u32, String)>,
    value: T,
}

pub(super) struct Key {
    kind: &'static str,
    name: String,
    input: Option<String>,
}

pub(super) struct PendingJob {
    pub(super) position: usize,
    pub(super) name: String,
    pub(super) display: String,
    pub(super) crates: Vec<Crate>,
}

#[derive(Clone)]
struct Local {
    fingerprint: Option<String>,
    certified: bool,
}

// Keys are computed on demand so that crates outside the checked dependency
// graphs are never hashed.
pub(super) struct Cache {
    directory: Option<PathBuf>,
    global: String,
    cargo: Option<CargoUnits>,
    paths: HashMap<String, FileId>,
    keys: RefCell<HashMap<Crate, String>>,
    sources: RefCell<HashMap<SourceRootId, String>>,
    locals: RefCell<HashMap<Crate, Local>>,
    inputs: RefCell<HashMap<Crate, Option<String>>>,
    partials: HashMap<Crate, CallIndex>,
    pub(super) indexed: usize,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn fingerprint(value: impl Serialize) -> String {
    digest(&serde_json::to_vec(&value).expect("cache input"))
}

fn file_hash(path: &Path) -> Option<String> {
    Some(digest(&fs::read(path).ok()?))
}

/// Fingerprints of compiled units keyed by source root, features, and test mode.
type CargoUnits = HashMap<(String, Vec<String>, bool), String>;

fn cargo_inputs(workspace: &LoadedWorkspace) -> Option<CargoUnits> {
    let mut units = HashMap::<_, Vec<_>>::new();
    for artifact in &workspace.native_artifacts {
        let mut features = artifact.features.clone();
        features.sort();
        let files = artifact
            .filenames
            .iter()
            .map(|path| {
                let metadata = fs::metadata(path).ok()?;
                // Test checks can emit empty metadata; recompilation must still invalidate it.
                let stamp = if metadata.len() == 0 {
                    Some(metadata.modified().ok()?)
                } else {
                    None
                };
                Some((path.to_string(), file_hash(path.as_std_path())?, stamp))
            })
            .collect::<Option<Vec<_>>>()?;
        if !files.is_empty() {
            units
                .entry((
                    artifact.target.src_path.to_string(),
                    features,
                    artifact.profile.test,
                ))
                .or_default()
                .extend(files);
        }
    }
    Some(
        units
            .into_iter()
            .map(|(unit, mut files)| {
                files.sort();
                files.dedup();
                (unit, fingerprint(files))
            })
            .collect(),
    )
}

impl Cache {
    pub(super) fn new(
        workspace: &LoadedWorkspace,
        plan: &Plan,
        options: &CheckOptions<'_>,
    ) -> Self {
        let cargo = cargo_inputs(workspace);
        let tool = env::current_exe().ok().and_then(|path| file_hash(&path));
        let global = fingerprint((&tool, options.target, options.release));
        Self {
            directory: (workspace.cacheable && tool.is_some() && cargo.is_some())
                .then(|| plan.target_directory.join("tessera-check").join(&global)),
            global,
            cargo,
            paths: workspace
                .vfs
                .iter()
                .map(|(file, path)| (path.to_string(), file))
                .collect(),
            keys: RefCell::default(),
            sources: RefCell::default(),
            locals: RefCell::default(),
            inputs: RefCell::default(),
            partials: HashMap::new(),
            indexed: 0,
        }
    }

    fn key(&self, workspace: &LoadedWorkspace, krate: Crate) -> String {
        if let Some(key) = self.keys.borrow().get(&krate) {
            return key.clone();
        }
        let db = workspace.host.raw_database();
        let data = krate.data(db);
        let key = fingerprint((
            &self.global,
            workspace.vfs.file_path(data.root_file_id).to_string(),
            format!(
                "{:?}",
                (
                    data.edition,
                    &data.origin,
                    &data.crate_attrs,
                    &data.proc_macro_cwd,
                    krate.cfg_options(db),
                    krate.env(db),
                    krate.workspace_data(db),
                    krate.extra_data(db)
                )
            ),
        ));
        self.keys.borrow_mut().insert(krate, key.clone());
        key
    }

    fn source(&self, workspace: &LoadedWorkspace, krate: Crate) -> String {
        let db = workspace.host.raw_database();
        let root = db
            .file_source_root(krate.data(db).root_file_id)
            .source_root_id(db);
        if let Some(source) = self.sources.borrow().get(&root) {
            return source.clone();
        }
        let mut files = db
            .source_root(root)
            .source_root(db)
            .iter()
            .map(|file| {
                (
                    workspace.vfs.file_path(file).to_string(),
                    digest(db.file_text(file).text(db).as_bytes()),
                )
            })
            .collect::<Vec<_>>();
        files.sort();
        let source = fingerprint(files);
        self.sources.borrow_mut().insert(root, source.clone());
        source
    }

    fn local(&self, workspace: &LoadedWorkspace, krate: Crate) -> Local {
        if let Some(local) = self.locals.borrow().get(&krate) {
            return local.clone();
        }
        let db = workspace.host.raw_database();
        let data = krate.data(db);
        let artifact = if data.is_proc_macro {
            let loaded = ProcMacros::get_for_crate(db, krate);
            if loaded
                .as_ref()
                .is_some_and(|macros| macros.get_error().is_none())
            {
                workspace
                    .macro_artifacts
                    .get(&data.root_file_id)
                    .and_then(|paths| {
                        paths
                            .iter()
                            .map(|path| file_hash(path))
                            .collect::<Option<Vec<_>>>()
                    })
                    .map(fingerprint)
            } else {
                // Optional macro dependencies can be unbuilt; their load state is an input.
                Some(fingerprint((
                    "unavailable",
                    loaded.map(|macros| format!("{:?}", macros.get_error())),
                )))
            }
        } else {
            Some(String::new())
        };
        let cfg = krate.cfg_options(db);
        let mut features = cfg
            .into_iter()
            .filter_map(|atom| match atom {
                CfgAtom::KeyValue { key, value } if key.as_str() == "feature" => {
                    Some(value.to_string())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        features.sort();
        let test = cfg
            .into_iter()
            .any(|atom| matches!(atom, CfgAtom::Flag(name) if name.as_str() == "test"));
        let native = self.cargo.as_ref().and_then(|units| {
            units.get(&(
                workspace.vfs.file_path(data.root_file_id).to_string(),
                features,
                test,
            ))
        });
        let key = self.key(workspace, krate);
        let source = self.source(workspace, krate);
        let local = Local {
            fingerprint: artifact.map(|artifact| fingerprint((&key, &source, artifact, native))),
            certified: native.is_some() || data.origin.is_lang(),
        };
        self.locals.borrow_mut().insert(krate, local.clone());
        local
    }

    fn input(&self, workspace: &LoadedWorkspace, krate: Crate) -> Option<String> {
        if let Some(input) = self.inputs.borrow().get(&krate) {
            return input.clone();
        }
        let db = workspace.host.raw_database();
        // The checked crate's Cargo certificate covers its actual dependency units.
        // rust-analyzer can also expose unbuilt optional or unified-feature variants.
        let input = self
            .local(workspace, krate)
            .certified
            .then(|| {
                krate
                    .transitive_deps(db)
                    .into_iter()
                    .map(|dependency| {
                        let mut edges = dependency
                            .data(db)
                            .dependencies
                            .iter()
                            .map(|edge| {
                                (
                                    edge.name.to_string(),
                                    self.key(workspace, edge.crate_id),
                                    edge.is_prelude(),
                                    edge.is_sysroot(),
                                )
                            })
                            .collect::<Vec<_>>();
                        edges.sort();
                        Some((
                            self.key(workspace, dependency),
                            self.local(workspace, dependency).fingerprint?,
                            edges,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()
            })
            .flatten()
            .map(|mut dependencies| {
                dependencies.sort();
                fingerprint(dependencies)
            });
        self.inputs.borrow_mut().insert(krate, input.clone());
        input
    }

    pub(super) fn index(
        &mut self,
        workspace: &LoadedWorkspace,
        crates: &HashSet<Crate>,
        workers: usize,
    ) -> Result<()> {
        let db = workspace.host.raw_database();
        let mut crates = crates
            .iter()
            .copied()
            .filter(|krate| !self.partials.contains_key(krate))
            .map(|krate| (self.key(workspace, krate), krate))
            .collect::<Vec<_>>();
        crates.sort();
        for (name, krate) in crates {
            let key = Key {
                kind: "index",
                name,
                input: self.input(workspace, krate),
            };
            let cached = self.read::<CallIndex>(&key).and_then(|(mut index, files)| {
                index.inactive_code_ranges = index
                    .inactive_code_ranges
                    .into_iter()
                    .map(|(file, contexts)| {
                        let file = *files.get(&file)?;
                        contexts
                            .iter()
                            .flatten()
                            .all(|range| valid_range(workspace, file, *range))
                            .then_some((file, contexts))
                    })
                    .collect::<Option<_>>()?;
                index
                    .inactive_code_ranges
                    .contains_key(&krate.data(db).root_file_id)
                    .then_some(index)
            });
            let partial = match cached {
                Some(index) => index,
                None => {
                    let files = crate_def_map(db, krate)
                        .modules()
                        .filter_map(|(_, module)| {
                            module
                                .definition_source_file_id()
                                .file_id()
                                .map(|file| file.file_id(db))
                        })
                        .collect::<HashSet<_>>();
                    let sources = SelectedSources {
                        local_files: files.clone(),
                        file_crates: files
                            .into_iter()
                            .map(|file| (file, HashSet::from([krate])))
                            .collect(),
                    };
                    let analyzer = CallAnalyzer::new(db, &workspace.vfs, sources)?;
                    let index = collect_index_parallel(&analyzer, workers)?;
                    self.write(
                        workspace,
                        &key,
                        &index,
                        index.inactive_code_ranges.keys().copied(),
                    );
                    self.indexed += 1;
                    index
                }
            };
            self.partials.insert(krate, partial);
        }
        Ok(())
    }

    pub(super) fn job(
        &self,
        workspace: &LoadedWorkspace,
        job: &CrateJob,
        position: usize,
        options: &CheckOptions<'_>,
    ) -> Result<(PendingJob, Key)> {
        let db = workspace.host.raw_database();
        let roots = job
            .target_roots
            .iter()
            .map(|path| {
                workspace::file_id_for_path(&workspace.vfs, path)
                    .with_context(|| format!("rust-analyzer did not load {}", path.display()))
            })
            .collect::<Result<HashSet<_>>>()?;
        let crates = all_crates(db)
            .iter()
            .copied()
            .filter(|krate| roots.contains(&krate.data(db).root_file_id))
            .collect::<Vec<_>>();
        ensure!(
            roots.iter().all(|root| crates
                .iter()
                .any(|krate| krate.data(db).root_file_id == *root)),
            "rust-analyzer did not resolve every selected target; refusing to proceed with unchecked calls"
        );
        let mut keys = crates
            .iter()
            .map(|krate| self.key(workspace, *krate))
            .collect::<Vec<_>>();
        keys.sort();
        // A job is analyzed against the indexes of its own dependency graph, so
        // the inputs of its crates determine its diagnostics.
        let mut inputs = crates
            .iter()
            .map(|krate| self.input(workspace, *krate))
            .collect::<Option<Vec<_>>>();
        if let Some(inputs) = &mut inputs {
            inputs.sort();
        }
        Ok((
            PendingJob {
                position,
                name: job.name.clone(),
                display: job.display.clone(),
                crates,
            },
            Key {
                kind: "diagnostics",
                name: fingerprint((&job.id, keys)),
                input: inputs.map(|inputs| fingerprint((inputs, format!("{:?}", options.lints)))),
            },
        ))
    }

    pub(super) fn analysis_job(
        &self,
        workspace: &LoadedWorkspace,
        job: &PendingJob,
    ) -> Result<AnalysisJob> {
        let db = workspace.host.raw_database();
        let mut files = HashSet::new();
        for krate in &job.crates {
            let partial = self
                .partials
                .get(krate)
                .context("selected crate index is unavailable")?;
            files.extend(partial.inactive_code_ranges.keys().copied());
        }
        let mut index = CallIndex {
            tessera_crates: tessera_crates(db),
            ..CallIndex::default()
        };
        let closure = job
            .crates
            .iter()
            .flat_map(|krate| krate.transitive_deps(db))
            .collect::<HashSet<_>>();
        for krate in closure {
            if let Some(partial) = self.partials.get(&krate) {
                index.merge(partial.clone());
            }
        }
        Ok(AnalysisJob {
            index: job.position,
            name: job.name.clone(),
            display: job.display.clone(),
            files,
            call_index: index,
        })
    }

    pub(super) fn diagnostics(
        &self,
        workspace: &LoadedWorkspace,
        key: &Key,
    ) -> Option<Vec<Diagnostic>> {
        let (mut diagnostics, files) = self.read::<Vec<Diagnostic>>(key)?;
        for diagnostic in &mut diagnostics {
            diagnostic.file_id = *files.get(&diagnostic.file_id)?;
            if !valid_range(workspace, diagnostic.file_id, diagnostic.range)
                || diagnostic
                    .boundary
                    .as_ref()
                    .is_some_and(|(range, _)| !valid_range(workspace, diagnostic.file_id, *range))
            {
                return None;
            }
            for edit in diagnostic
                .suggestions
                .iter_mut()
                .flat_map(|suggestion| &mut suggestion.edits)
            {
                edit.file_id = *files.get(&edit.file_id)?;
                if !valid_range(workspace, edit.file_id, edit.range) {
                    return None;
                }
            }
        }
        Some(diagnostics)
    }

    pub(super) fn save_diagnostics(
        &self,
        workspace: &LoadedWorkspace,
        key: &Key,
        diagnostics: &[Diagnostic],
    ) {
        let files = diagnostics.iter().flat_map(|diagnostic| {
            std::iter::once(diagnostic.file_id).chain(
                diagnostic
                    .suggestions
                    .iter()
                    .flat_map(|suggestion| suggestion.edits.iter().map(|edit| edit.file_id)),
            )
        });
        self.write(workspace, key, &diagnostics, files);
    }

    fn read<T: DeserializeOwned>(&self, key: &Key) -> Option<(T, HashMap<FileId, FileId>)> {
        let path = self
            .directory
            .as_ref()?
            .join(format!("{}-{}.json", key.kind, key.name));
        let record: Record = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        if record.input != *key.input.as_ref()?
            || record.checksum != digest(record.payload.as_bytes())
        {
            return None;
        }
        let payload: Payload<T> = serde_json::from_str(&record.payload).ok()?;
        let files = payload
            .files
            .into_iter()
            .map(|(raw, path)| {
                (raw <= i32::MAX as u32).then_some(())?;
                Some((FileId::from_raw(raw), *self.paths.get(&path)?))
            })
            .collect::<Option<_>>()?;
        Some((payload.value, files))
    }

    fn write<T: Serialize + ?Sized>(
        &self,
        workspace: &LoadedWorkspace,
        key: &Key,
        value: &T,
        files: impl Iterator<Item = FileId>,
    ) {
        let Some((directory, input)) = self.directory.as_ref().zip(key.input.as_ref()) else {
            return;
        };
        let _ = (|| -> Result<()> {
            let mut files = files
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|file| (file.index(), workspace.vfs.file_path(file).to_string()))
                .collect::<Vec<_>>();
            files.sort();
            let payload = serde_json::to_string(&Payload { files, value })?;
            let record = Record {
                input: input.clone(),
                checksum: digest(payload.as_bytes()),
                payload,
            };
            fs::create_dir_all(directory)?;
            let mut temporary = tempfile::NamedTempFile::new_in(directory)?;
            temporary.write_all(&serde_json::to_vec(&record)?)?;
            temporary.persist(directory.join(format!("{}-{}.json", key.kind, key.name)))?;
            Ok(())
        })();
    }
}

fn valid_range(workspace: &LoadedWorkspace, file: FileId, range: TextRange) -> bool {
    let db = workspace.host.raw_database();
    let text = db.file_text(file);
    let text = text.text(db);
    text.is_char_boundary(u32::from(range.start()) as usize)
        && text.is_char_boundary(u32::from(range.end()) as usize)
}
