use std::collections::{HashMap, HashSet};

use ra_ap_base_db::Crate;
use ra_ap_hir::Semantics;
use ra_ap_ide::{AnalysisHost, FileId, RootDatabase};
use ra_ap_span::TextRange;
use ra_ap_vfs::Vfs;

pub(crate) struct Diagnostic {
    pub(crate) file_id: FileId,
    pub(crate) range: TextRange,
    pub(crate) message: String,
}

pub(crate) struct LoadedWorkspace {
    pub(crate) host: AnalysisHost,
    pub(crate) vfs: Vfs,
}

pub(crate) struct ColorAnalyzer<'db> {
    pub(crate) db: &'db RootDatabase,
    pub(crate) sema: Semantics<'db, RootDatabase>,
    pub(crate) tessera_crates: HashSet<Crate>,
    pub(crate) vfs: &'db Vfs,
    pub(crate) local_files: HashSet<FileId>,
    pub(crate) selected_package: Option<String>,
    pub(crate) tessera_function_names: HashSet<String>,
    pub(crate) explicit_slot_setter_indexes: HashMap<String, HashMap<String, HashSet<usize>>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextColor {
    Tessera,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TesseraRuntimeApi {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedMethod {
    RenderSlotNew,
    RenderSlotWithNew,
    Other,
}
