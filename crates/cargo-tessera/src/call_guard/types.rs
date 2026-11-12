use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    path::PathBuf,
};

use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_base_db::Crate;
use ra_ap_hir::{Function, ModuleDef, Semantics};
use ra_ap_ide::{AnalysisHost, FileId, RootDatabase};
use ra_ap_span::TextRange;
use ra_ap_vfs::Vfs;
use serde::{Deserialize, Serialize};

pub(crate) use rustfix::diagnostics::Applicability;

use super::error_codes::DiagnosticCode;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Edit {
    #[serde(with = "file_id")]
    pub(crate) file_id: FileId,
    pub(crate) range: TextRange,
    pub(crate) replacement: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Suggestion {
    pub(crate) message: String,
    #[serde(serialize_with = "applicability")]
    pub(crate) applicability: Applicability,
    pub(crate) edits: Vec<Edit>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Diagnostic {
    pub(crate) code: DiagnosticCode,
    pub(crate) severity: Severity,
    #[serde(with = "file_id")]
    pub(crate) file_id: FileId,
    pub(crate) range: TextRange,
    pub(crate) message: String,
    pub(crate) label: String,
    pub(crate) help: String,
    pub(crate) notes: Vec<String>,
    pub(crate) suggestions: Vec<Suggestion>,
    pub(crate) boundary: Option<(TextRange, String)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageFormat {
    #[default]
    Human,
    Short,
    Json,
}

impl MessageFormat {
    pub fn from_cargo_arg(value: Option<&str>) -> Self {
        let Some(value) = value else {
            return Self::Human;
        };

        if value.split(',').any(|part| part.starts_with("json")) {
            Self::Json
        } else if value == "short" {
            Self::Short
        } else {
            Self::Human
        }
    }

    pub fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}

pub(crate) struct LoadedWorkspace {
    pub(crate) host: AnalysisHost,
    pub(crate) vfs: Vfs,
    pub(crate) macro_artifacts: HashMap<FileId, Vec<PathBuf>>,
    pub(crate) native_artifacts: Vec<cargo_metadata::Artifact>,
    pub(crate) cacheable: bool,
}

#[derive(Clone)]
pub(crate) struct SelectedSources {
    pub(crate) local_files: HashSet<FileId>,
    pub(crate) file_crates: HashMap<FileId, HashSet<Crate>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub(crate) struct CallIndex {
    #[serde(skip)]
    pub(crate) tessera_crates: HashSet<Crate>,
    pub(crate) tessera_function_names: BTreeSet<String>,
    pub(crate) tessera_function_paths: BTreeSet<String>,
    pub(crate) render_slot_setter_indexes_by_type_path:
        BTreeMap<String, BTreeMap<String, BTreeSet<usize>>>,
    pub(crate) render_slot_constructor_indexes_by_function_path: BTreeMap<String, BTreeSet<usize>>,
    #[serde(with = "file_ranges")]
    pub(crate) inactive_code_ranges: HashMap<FileId, Vec<Vec<TextRange>>>,
}

impl CallIndex {
    pub(crate) fn merge(&mut self, partial: Self) {
        for (file, contexts) in partial.inactive_code_ranges {
            let target = self.inactive_code_ranges.entry(file).or_default();
            for context in contexts {
                if !target.contains(&context) {
                    target.push(context);
                }
            }
        }
        self.tessera_function_names
            .extend(partial.tessera_function_names);
        self.tessera_function_paths
            .extend(partial.tessera_function_paths);
        for (path, setters) in partial.render_slot_setter_indexes_by_type_path {
            let target = self
                .render_slot_setter_indexes_by_type_path
                .entry(path)
                .or_default();
            for (setter, arguments) in setters {
                target.entry(setter).or_default().extend(arguments);
            }
        }
        for (path, arguments) in partial.render_slot_constructor_indexes_by_function_path {
            self.render_slot_constructor_indexes_by_function_path
                .entry(path)
                .or_default()
                .extend(arguments);
        }
    }
}

pub(crate) struct CallAnalyzer<'db> {
    pub(crate) display_root: std::path::PathBuf,
    pub(crate) db: &'db RootDatabase,
    pub(crate) index: CallIndex,
    pub(crate) sema: Semantics<'db, RootDatabase>,
    pub(crate) vfs: &'db Vfs,
    pub(crate) local_files: HashSet<FileId>,
    pub(crate) metadata_files: HashSet<FileId>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) inactive_bodies: HashMap<FileId, Vec<TextRange>>,
    pub(crate) call_targets: RefCell<HashMap<Function, Option<CallTarget>>>,
    pub(crate) private_api_items: RefCell<HashMap<Crate, HashSet<ModuleDef>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextColor {
    Tessera,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TesseraRuntimeApi {
    EntryPointNew,
    Remember,
    RememberWithKey,
    Retain,
    RetainWithKey,
    ProvideContext,
    UseContext,
    Launch,
    LaunchOnce,
    UseRouter,
    Key,
    RenderSlotNew,
    RenderSlotWithNew,
    RenderSlotRender,
    RenderSlotWithRender,
    InternalRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallKind {
    TesseraFunction,
    RuntimeApi(TesseraRuntimeApi),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CallTarget {
    Resolved { path: String, kind: CallKind },
    Unresolved { path: String },
}

fn applicability<S: serde::Serializer>(
    value: &Applicability,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{value:?}"))
}

mod file_id {
    use super::*;

    pub(super) fn serialize<S: serde::Serializer>(
        file: &FileId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        file.index().serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<FileId, D::Error> {
        let raw = u32::deserialize(deserializer)?;
        if raw > i32::MAX as u32 {
            return Err(serde::de::Error::custom("invalid file ID"));
        }
        Ok(FileId::from_raw(raw))
    }
}

mod file_ranges {
    use super::*;

    type Ranges = HashMap<FileId, Vec<Vec<TextRange>>>;

    pub(super) fn serialize<S: serde::Serializer>(
        ranges: &Ranges,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        ranges
            .iter()
            .map(|(file, ranges)| (file.index(), ranges))
            .collect::<Vec<_>>()
            .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Ranges, D::Error> {
        let ranges = Vec::<(u32, Vec<Vec<TextRange>>)>::deserialize(deserializer)?;
        if ranges.iter().any(|(raw, _)| *raw > i32::MAX as u32) {
            return Err(serde::de::Error::custom("invalid file ID"));
        }
        Ok(ranges
            .into_iter()
            .map(|(file, ranges)| (FileId::from_raw(file), ranges))
            .collect())
    }
}
