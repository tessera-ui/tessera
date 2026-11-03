use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use ra_ap_base_db::Crate;
use ra_ap_hir::{Function, Semantics};
use ra_ap_ide::{AnalysisHost, FileId, RootDatabase};
use ra_ap_span::TextRange;
use ra_ap_vfs::Vfs;

use super::error_codes::ErrorCode;

pub(crate) struct Diagnostic {
    pub(crate) code: ErrorCode,
    pub(crate) file_id: FileId,
    pub(crate) range: TextRange,
    pub(crate) message: String,
    pub(crate) label: String,
    pub(crate) help: &'static str,
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
}

#[derive(Clone)]
pub(crate) struct SelectedSources {
    pub(crate) local_files: HashSet<FileId>,
    pub(crate) file_crates: HashMap<FileId, HashSet<Crate>>,
}

#[derive(Clone, Default)]
pub(crate) struct CallIndex {
    pub(crate) tessera_crates: HashSet<Crate>,
    pub(crate) tessera_function_names: HashSet<String>,
    pub(crate) tessera_function_paths: HashSet<String>,
    pub(crate) render_slot_setter_indexes_by_type_path:
        HashMap<String, HashMap<String, HashSet<usize>>>,
    pub(crate) render_slot_constructor_indexes_by_function_path: HashMap<String, HashSet<usize>>,
    pub(crate) inactive_code_ranges: HashMap<FileId, Vec<Vec<TextRange>>>,
}

impl CallIndex {
    pub(crate) fn merge(&mut self, partial: Self) {
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
    ReceiveFrameNanos,
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
