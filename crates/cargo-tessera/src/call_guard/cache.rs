use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, Write},
    path::Path,
};

use anyhow::{Context, Result, ensure};
use ra_ap_base_db::{Crate, SourceDatabase, all_crates};
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

pub(super) struct Cache<'a> {
    workspace: &'a LoadedWorkspace,
    directory: Option<std::path::PathBuf>,
    paths: HashMap<String, FileId>,
    keys: HashMap<Crate, String>,
    inputs: HashMap<Crate, Option<String>>,
    members: HashMap<Crate, HashSet<FileId>>,
    pub(super) indexed: usize,
}

fn fingerprint(value: impl Serialize) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).expect("cache input"))
    )
}

fn file_hash(path: &Path) -> Option<String> {
    let mut hash = Sha256::new();
    io::copy(&mut fs::File::open(path).ok()?, &mut hash).ok()?;
    Some(format!("{:x}", hash.finalize()))
}

fn cargo_inputs(
    workspace: &LoadedWorkspace,
) -> Option<HashMap<(String, Vec<String>, bool), String>> {
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

impl<'a> Cache<'a> {
    pub(super) fn new(
        workspace: &'a LoadedWorkspace,
        plan: &Plan,
        options: &CheckOptions<'_>,
    ) -> Self {
        let db = workspace.host.raw_database();
        let cargo = cargo_inputs(workspace);
        let tool = env::current_exe().ok().and_then(|path| file_hash(&path));
        let global = fingerprint((&tool, options.target, options.release));
        let paths = workspace
            .vfs
            .iter()
            .map(|(file, path)| (path.to_string(), file))
            .collect();
        let mut roots = HashMap::new();
        let mut keys = HashMap::new();
        let mut local = HashMap::new();
        let mut certified = HashSet::new();
        for &krate in all_crates(db).iter() {
            let data = krate.data(db);
            let key = fingerprint((
                &global,
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
            let root = db.file_source_root(data.root_file_id).source_root_id(db);
            let source = roots.entry(root).or_insert_with(|| {
                let source = db.source_root(root).source_root(db);
                let mut files = source
                    .iter()
                    .map(|file| {
                        (
                            workspace.vfs.file_path(file).to_string(),
                            fingerprint(db.file_text(file).text(db).as_ref()),
                        )
                    })
                    .collect::<Vec<_>>();
                files.sort();
                fingerprint(files)
            });
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
            let native = cargo.as_ref().and_then(|units| {
                units.get(&(
                    workspace.vfs.file_path(data.root_file_id).to_string(),
                    features,
                    test,
                ))
            });
            if native.is_some() || data.origin.is_lang() {
                certified.insert(krate);
            }
            local.insert(
                krate,
                artifact.map(|artifact| fingerprint((&key, &*source, artifact, native))),
            );
            keys.insert(krate, key);
        }
        let inputs = all_crates(db)
            .iter()
            .map(|&krate| {
                let mut dependencies = krate
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
                                    &keys[&edge.crate_id],
                                    edge.is_prelude(),
                                    edge.is_sysroot(),
                                )
                            })
                            .collect::<Vec<_>>();
                        edges.sort();
                        Some((&keys[&dependency], local[&dependency].as_ref()?, edges))
                    })
                    .collect::<Option<Vec<_>>>();
                if let Some(dependencies) = &mut dependencies {
                    dependencies.sort();
                }
                // The checked crate's Cargo certificate covers its actual dependency units.
                // rust-analyzer can also expose unbuilt optional or unified-feature variants.
                (
                    krate,
                    dependencies
                        .filter(|_| certified.contains(&krate))
                        .map(fingerprint),
                )
            })
            .collect();
        Self {
            workspace,
            directory: (workspace.cacheable && tool.is_some() && cargo.is_some())
                .then(|| plan.target_directory.join("tessera-check").join(&global)),
            paths,
            keys,
            inputs,
            members: HashMap::new(),
            indexed: 0,
        }
    }

    pub(super) fn index(&mut self, crates: &HashSet<Crate>, workers: usize) -> Result<CallIndex> {
        let db = self.workspace.host.raw_database();
        let mut index = CallIndex {
            tessera_crates: tessera_crates(db),
            ..CallIndex::default()
        };
        let mut crates = crates.iter().copied().collect::<Vec<_>>();
        crates.sort_by_key(|krate| &self.keys[krate]);
        for krate in crates {
            let key = Key {
                kind: "index",
                name: self.keys[&krate].clone(),
                input: self.inputs[&krate].clone(),
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
                            .all(|range| self.valid_range(file, *range))
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
                    let analyzer = CallAnalyzer::new(db, &self.workspace.vfs, sources)?;
                    let index = collect_index_parallel(&analyzer, workers)?;
                    self.write(&key, &index, index.inactive_code_ranges.keys().copied());
                    self.indexed += 1;
                    index
                }
            };
            self.members.insert(
                krate,
                partial.inactive_code_ranges.keys().copied().collect(),
            );
            index.merge(partial);
        }
        Ok(index)
    }

    pub(super) fn job(
        &self,
        job: &CrateJob,
        index: usize,
        shape: &str,
        options: &CheckOptions<'_>,
    ) -> Result<(AnalysisJob, Key)> {
        let db = self.workspace.host.raw_database();
        let roots = job
            .target_roots
            .iter()
            .map(|path| {
                workspace::file_id_for_path(&self.workspace.vfs, path)
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
            .map(|krate| &self.keys[krate])
            .collect::<Vec<_>>();
        keys.sort();
        let mut inputs = crates
            .iter()
            .map(|krate| self.inputs[krate].clone())
            .collect::<Option<Vec<_>>>();
        if let Some(inputs) = &mut inputs {
            inputs.sort();
        }
        let files = crates
            .iter()
            .map(|krate| {
                self.members
                    .get(krate)
                    .context("selected crate index is unavailable")
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .copied()
            .collect();
        Ok((
            AnalysisJob {
                index,
                name: job.name.clone(),
                display: job.display.clone(),
                files,
            },
            Key {
                kind: "diagnostics",
                name: fingerprint((&job.id, keys)),
                input: inputs
                    .map(|inputs| fingerprint((inputs, shape, format!("{:?}", options.lints)))),
            },
        ))
    }

    pub(super) fn diagnostics(&self, key: &Key) -> Option<Vec<Diagnostic>> {
        let (mut diagnostics, files) = self.read::<Vec<Diagnostic>>(key)?;
        for diagnostic in &mut diagnostics {
            diagnostic.file_id = *files.get(&diagnostic.file_id)?;
            if !self.valid_range(diagnostic.file_id, diagnostic.range)
                || diagnostic
                    .boundary
                    .as_ref()
                    .is_some_and(|(range, _)| !self.valid_range(diagnostic.file_id, *range))
            {
                return None;
            }
            for edit in diagnostic
                .suggestions
                .iter_mut()
                .flat_map(|suggestion| &mut suggestion.edits)
            {
                edit.file_id = *files.get(&edit.file_id)?;
                if !self.valid_range(edit.file_id, edit.range) {
                    return None;
                }
            }
        }
        Some(diagnostics)
    }

    pub(super) fn save_diagnostics(&self, key: &Key, diagnostics: &[Diagnostic]) {
        let files = diagnostics.iter().flat_map(|diagnostic| {
            std::iter::once(diagnostic.file_id).chain(
                diagnostic
                    .suggestions
                    .iter()
                    .flat_map(|suggestion| suggestion.edits.iter().map(|edit| edit.file_id)),
            )
        });
        self.write(key, &diagnostics, files);
    }

    fn valid_range(&self, file: FileId, range: TextRange) -> bool {
        let db = self.workspace.host.raw_database();
        let text = db.file_text(file);
        let text = text.text(db);
        text.is_char_boundary(u32::from(range.start()) as usize)
            && text.is_char_boundary(u32::from(range.end()) as usize)
    }

    fn read<T: DeserializeOwned>(&self, key: &Key) -> Option<(T, HashMap<FileId, FileId>)> {
        let path = self
            .directory
            .as_ref()?
            .join(format!("{}-{}.json", key.kind, key.name));
        let record: Record = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        if record.input != *key.input.as_ref()? || record.checksum != fingerprint(&record.payload) {
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
                .map(|file| (file.index(), self.workspace.vfs.file_path(file).to_string()))
                .collect::<Vec<_>>();
            files.sort();
            let payload = serde_json::to_string(&Payload { files, value })?;
            let record = Record {
                input: input.clone(),
                checksum: fingerprint(&payload),
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

pub(super) fn shape(index: &CallIndex) -> String {
    fingerprint((
        &index.tessera_function_names,
        &index.tessera_function_paths,
        &index.render_slot_setter_indexes_by_type_path,
        &index.render_slot_constructor_indexes_by_function_path,
    ))
}
