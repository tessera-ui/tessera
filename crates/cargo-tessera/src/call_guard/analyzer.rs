use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet},
};

use anyhow::{Result, anyhow};
use ra_ap_base_db::Crate;
use ra_ap_hir::{
    AsAssocItem, CallableKind, EditionedFileId, Function, ModuleDef, PathResolution, Semantics,
    Trait, Type, crate_def_map,
};
use ra_ap_hir_def::{
    DefWithBodyId,
    expr_store::{ExpressionStoreDiagnostics, body::Body},
    nameres::diagnostics::DefDiagnosticKind,
};
use ra_ap_ide::{FileId, RootDatabase};
use ra_ap_syntax::{
    AstNode, SyntaxNode,
    ast::{self, HasArgList, HasAttrs, HasGenericArgs, HasModuleItem, HasName},
};
use ra_ap_vfs::Vfs;

use super::{
    LintOptions,
    error_codes::{DiagnosticCode, ErrorCode},
    types::{
        CallAnalyzer, CallIndex, CallKind, CallTarget, ContextColor, Diagnostic, SelectedSources,
        TesseraRuntimeApi,
    },
    utils::{
        expr_path, is_internal_runtime_name, param_name, path_last_segment, path_text,
        runtime_api_label, tessera_crates,
    },
};

mod lint_levels;
mod lints;
mod model_init;
mod private_api;

const RENDER_SLOT_TYPE_PATH: &str = "tessera_ui::prop::RenderSlot";
const RENDER_SLOT_WITH_TYPE_PATH: &str = "tessera_ui::prop::RenderSlotWith";
const PUBLIC_RENDER_SLOT_TYPE_PATH: &str = "tessera_ui::RenderSlot";
const PUBLIC_RENDER_SLOT_WITH_TYPE_PATH: &str = "tessera_ui::RenderSlotWith";
const ENTRY_POINT_TYPE_PATH: &str = "tessera_ui::entry_point::EntryPoint";
const PUBLIC_ENTRY_POINT_TYPE_PATH: &str = "tessera_ui::EntryPoint";
const REMEMBER_PATH: &str = "tessera_ui::runtime::remember";
const PUBLIC_REMEMBER_PATH: &str = "tessera_ui::remember";
const REMEMBER_WITH_KEY_PATH: &str = "tessera_ui::runtime::remember_with_key";
const PUBLIC_REMEMBER_WITH_KEY_PATH: &str = "tessera_ui::remember_with_key";
const RETAIN_PATH: &str = "tessera_ui::runtime::retain";
const PUBLIC_RETAIN_PATH: &str = "tessera_ui::retain";
const RETAIN_WITH_KEY_PATH: &str = "tessera_ui::runtime::retain_with_key";
const PUBLIC_RETAIN_WITH_KEY_PATH: &str = "tessera_ui::retain_with_key";
const PROVIDE_CONTEXT_PATH: &str = "tessera_ui::context::provide_context";
const PUBLIC_PROVIDE_CONTEXT_PATH: &str = "tessera_ui::provide_context";
const USE_CONTEXT_PATH: &str = "tessera_ui::context::use_context";
const PUBLIC_USE_CONTEXT_PATH: &str = "tessera_ui::use_context";
const LAUNCH_PATH: &str = "tessera_ui::runtime::launch";
const PUBLIC_LAUNCH_PATH: &str = "tessera_ui::launch";
const LAUNCH_ONCE_PATH: &str = "tessera_ui::runtime::launch_once";
const PUBLIC_LAUNCH_ONCE_PATH: &str = "tessera_ui::launch_once";
const USE_ROUTER_PATH: &str = "tessera_shard::router::use_router";
const PUBLIC_USE_ROUTER_PATH: &str = "tessera_shard::use_router";
const KEY_PATH: &str = "tessera_ui::runtime::key";
const PUBLIC_KEY_PATH: &str = "tessera_ui::key";
const RUNTIME_MODULE_PATH_PREFIX: &str = "tessera_ui::runtime::";
const RENDER_SLOT_NEW_PATH: &str = "tessera_ui::prop::RenderSlot::new";
const RENDER_SLOT_WITH_NEW_PATH: &str = "tessera_ui::prop::RenderSlotWith::new";
const PUBLIC_RENDER_SLOT_NEW_PATH: &str = "tessera_ui::RenderSlot::new";
const PUBLIC_RENDER_SLOT_WITH_NEW_PATH: &str = "tessera_ui::RenderSlotWith::new";
const ENTRY_POINT_NEW_PATH: &str = "tessera_ui::entry_point::EntryPoint::new";
const PUBLIC_ENTRY_POINT_NEW_PATH: &str = "tessera_ui::EntryPoint::new";
const CORE_OPTION_TYPE_PATH: &str = "core::option::Option";
const STD_OPTION_TYPE_PATH: &str = "std::option::Option";
const CORE_INTO_TRAIT_PATH: &str = "core::convert::Into";
const STD_INTO_TRAIT_PATH: &str = "std::convert::Into";

impl<'db> CallAnalyzer<'db> {
    pub(crate) fn new(
        db: &'db RootDatabase,
        vfs: &'db Vfs,
        sources: SelectedSources,
    ) -> Result<CallAnalyzer<'db>> {
        let sema = Semantics::new(db);
        let tessera_crates = tessera_crates(db);
        let SelectedSources {
            local_files,
            file_crates,
        } = sources;
        let metadata_files = file_crates.keys().copied().collect::<HashSet<_>>();

        if local_files.is_empty() {
            return Err(anyhow!(
                "rust-analyzer did not load any selected Rust source files for CallGuard"
            ));
        }

        let relevant_crates = file_crates
            .values()
            .flat_map(|crates| crates.iter().copied())
            .collect::<HashSet<_>>();
        let mut inactive_by_crate = HashMap::new();
        for krate in relevant_crates {
            let mut ranges_by_file = HashMap::<FileId, Vec<_>>::new();
            for diagnostic in crate_def_map(db, krate).diagnostics() {
                if let DefDiagnosticKind::UnconfiguredCode { ast_id, .. } = &diagnostic.kind
                    && let Some(file_id) = ast_id.file_id.file_id()
                {
                    ranges_by_file
                        .entry(file_id.file_id(db))
                        .or_default()
                        .push(ast_id.to_range(db));
                }
            }
            inactive_by_crate.insert(krate, ranges_by_file);
        }
        let inactive_code_ranges = metadata_files
            .iter()
            .map(|file_id| {
                let contexts = file_crates[file_id]
                    .iter()
                    .map(|krate| {
                        inactive_by_crate
                            .get(krate)
                            .and_then(|files| files.get(file_id))
                            .cloned()
                            .unwrap_or_default()
                    })
                    .collect();
                (*file_id, contexts)
            })
            .collect();

        Ok(CallAnalyzer {
            display_root: std::env::current_dir().unwrap_or_default(),
            db,
            sema,
            index: CallIndex {
                tessera_crates,
                inactive_code_ranges,
                ..CallIndex::default()
            },
            vfs,
            local_files,
            metadata_files,
            diagnostics: Vec::new(),
            inactive_bodies: HashMap::new(),
            call_targets: RefCell::new(HashMap::new()),
            private_api_items: RefCell::new(HashMap::new()),
        })
    }

    pub(crate) fn check_files(&mut self, lints: &LintOptions) -> Result<()> {
        let mut files = self.local_files.iter().copied().collect::<Vec<_>>();
        files.sort_unstable();
        for file_id in files {
            self.analyze_file(file_id)?;
        }
        self.apply_lint_levels(lints);
        Ok(())
    }

    pub(crate) fn from_index(
        db: &'db RootDatabase,
        vfs: &'db Vfs,
        local_files: HashSet<FileId>,
        index: CallIndex,
    ) -> Self {
        Self {
            display_root: std::env::current_dir().unwrap_or_default(),
            db,
            sema: Semantics::new(db),
            vfs,
            local_files,
            metadata_files: HashSet::new(),
            index,
            diagnostics: Vec::new(),
            inactive_bodies: HashMap::new(),
            call_targets: RefCell::new(HashMap::new()),
            private_api_items: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn collect_file_metadata(&mut self, file_id: FileId) -> Result<()> {
        let tree = self
            .sema
            .parse(EditionedFileId::current_edition(self.db, file_id));

        for item in tree.items() {
            self.collect_tessera_function_names_from_item(&item);
        }

        for item in tree.items() {
            self.collect_render_slot_carriers_from_item(&item);
        }

        Ok(())
    }

    fn is_inactive(&self, syntax: &SyntaxNode) -> bool {
        let source = self.sema.original_range(syntax);
        let file_id = source.file_id.file_id(self.db);
        self.inactive_bodies.get(&file_id).is_some_and(|ranges| {
            ranges
                .iter()
                .any(|range| range.contains_range(source.range))
        }) || self
            .index
            .inactive_code_ranges
            .get(&file_id)
            .is_some_and(|contexts| {
                // A shared file remains active if any loaded crate context
                // enables it.
                !contexts.is_empty()
                    && contexts.iter().all(|ranges| {
                        ranges
                            .iter()
                            .any(|range| range.contains_range(source.range))
                    })
            })
    }

    fn analyze_file(&mut self, file_id: FileId) -> Result<()> {
        let tree = self
            .sema
            .parse(EditionedFileId::current_edition(self.db, file_id));

        for item in tree.items() {
            self.analyze_item(item);
        }
        self.check_private_api_uses(tree.syntax());
        let imports = self.named_modifier_ext_imports(tree.syntax());
        self.diagnostics.extend(imports);
        Ok(())
    }

    fn collect_tessera_function_names_from_item(&mut self, item: &ast::Item) {
        if self.is_inactive(item.syntax()) {
            return;
        }
        match item {
            ast::Item::Fn(function) if self.is_tessera_function(function) => {
                if let Some(name) = function.name() {
                    self.index
                        .tessera_function_names
                        .insert(name.text().to_string());
                }
                if let Some(path) = self.source_function_path(function) {
                    self.index.tessera_function_paths.insert(path);
                }
            }
            ast::Item::Module(module) => {
                if let Some(items) = module.item_list() {
                    for item in items.items() {
                        self.collect_tessera_function_names_from_item(&item);
                    }
                }
            }
            _ => {}
        }
    }

    fn analyze_item(&mut self, item: ast::Item) {
        if self.is_inactive(item.syntax()) {
            return;
        }
        match item {
            ast::Item::Fn(function) => self.analyze_function(function),
            ast::Item::Impl(item_impl) => {
                if let Some(items) = item_impl.assoc_item_list() {
                    for item in items.assoc_items() {
                        self.analyze_assoc_item(item);
                    }
                }
            }
            ast::Item::Trait(item_trait) => {
                if let Some(items) = item_trait.assoc_item_list() {
                    for item in items.assoc_items() {
                        self.analyze_assoc_item(item);
                    }
                }
            }
            ast::Item::Module(module) => {
                if let Some(items) = module.item_list() {
                    for item in items.items() {
                        self.analyze_item(item);
                    }
                }
            }
            _ => {}
        }
    }
    fn collect_render_slot_carriers_from_item(&mut self, item: &ast::Item) {
        if self.is_inactive(item.syntax()) {
            return;
        }
        match item {
            ast::Item::Fn(function) => {
                self.collect_generated_slot_setters_from_function(function);
            }
            ast::Item::Impl(item_impl) => {
                self.collect_explicit_slot_setters_from_impl(item_impl);
            }
            ast::Item::Module(module) => {
                if let Some(items) = module.item_list() {
                    for item in items.items() {
                        self.collect_render_slot_carriers_from_item(&item);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_generated_slot_setters_from_function(&mut self, function: &ast::Fn) {
        if !self.is_tessera_function(function) {
            return;
        }
        let type_path = self.generated_builder_type_path(function);
        let Some(param_list) = function.param_list() else {
            return;
        };

        if let Some(path) = self.source_function_path(function) {
            let mut index = 0;
            let mut slots = BTreeSet::new();
            for param in param_list.params() {
                let Some(ty) = param.ty() else { continue };
                let optional = if let ast::Type::PathType(ty) = &ty {
                    ty.path()
                        .and_then(|path| path.segment())
                        .is_some_and(|segment| {
                            segment
                                .name_ref()
                                .is_some_and(|name| name.text() == "Option")
                                && segment.generic_arg_list().is_some_and(|args| {
                                    let mut args = args.generic_args();
                                    matches!(args.next(), Some(ast::GenericArg::TypeArg(_)))
                                        && args.next().is_none()
                                })
                        })
                } else {
                    false
                };
                if optional {
                    continue;
                }
                if self.type_is_direct_render_slot_handle(ty) {
                    slots.insert(index);
                }
                index += 1;
            }
            if !slots.is_empty() {
                self.index
                    .render_slot_constructor_indexes_by_function_path
                    .insert(path, slots);
            }
        }

        for param in param_list.params() {
            if self.param_skips_setter(&param) {
                continue;
            }
            let Some(name) = param_name(&param) else {
                continue;
            };
            let Some(ty) = param.ty() else {
                continue;
            };
            if !self.type_is_optional_render_slot_handle(ty) {
                continue;
            }

            let mut indexes = BTreeSet::new();
            indexes.insert(0);
            let Some(type_path) = &type_path else {
                continue;
            };
            let setters = self
                .index
                .render_slot_setter_indexes_by_type_path
                .entry(type_path.clone())
                .or_default();
            setters.insert(name.clone(), indexes.clone());
            setters.insert(format!("{name}_shared"), indexes);
        }
    }

    fn collect_explicit_slot_setters_from_impl(&mut self, item_impl: &ast::Impl) {
        let Some(self_ty) = item_impl.self_ty() else {
            return;
        };
        let Some(type_path) = self.impl_self_type_key(&self_ty) else {
            return;
        };
        let Some(items) = item_impl.assoc_item_list() else {
            return;
        };

        for item in items.assoc_items() {
            let ast::AssocItem::Fn(function) = item else {
                continue;
            };
            if self.is_inactive(function.syntax()) {
                continue;
            }
            let Some(name) = function.name() else {
                continue;
            };
            let slot_indexes = self.render_slot_carrier_param_indexes(&function);
            if !slot_indexes.is_empty() {
                self.index
                    .render_slot_setter_indexes_by_type_path
                    .entry(type_path.clone())
                    .or_default()
                    .insert(name.text().to_string(), slot_indexes);
            }
        }
    }

    fn impl_self_type_key(&self, ty: &ast::Type) -> Option<String> {
        let ty = self.sema.resolve_type(ty)?;
        self.type_adt_canonical_path(&ty)
    }

    fn generated_builder_type_path(&self, function: &ast::Fn) -> Option<String> {
        let path = self.source_function_path(function)?;
        let (module, name) = path.rsplit_once("::")?;
        Some(format!(
            "{module}::{}",
            Self::generated_builder_type_name(name)
        ))
    }

    fn source_function_path(&self, function: &ast::Fn) -> Option<String> {
        if let Some(definition) = self.sema.to_fn_def(function) {
            return Some(self.function_path(definition));
        }
        let name = function.name()?;
        let module = self.sema.scope(function.syntax())?.module();
        let mut path = module.krate(self.db).display_name(self.db)?.to_string();
        for name in module
            .path_to_root(self.db)
            .into_iter()
            .rev()
            .filter_map(|module| module.name(self.db))
        {
            path.push_str("::");
            path.push_str(name.as_str());
        }
        path.push_str("::");
        path.push_str(name.text());
        Some(path)
    }

    fn generated_builder_type_name(function_name: &str) -> String {
        let mut output = String::new();
        for segment in function_name
            .split('_')
            .filter(|segment| !segment.is_empty())
        {
            let mut chars = segment.chars();
            if let Some(first) = chars.next() {
                output.extend(first.to_uppercase());
                output.push_str(chars.as_str());
            }
        }
        output.push_str("Builder");
        output
    }

    fn param_skips_setter(&self, param: &ast::Param) -> bool {
        param.attrs().any(|attr| {
            attr.path()
                .and_then(|path| path_last_segment(&path))
                .is_some_and(|name| name == "prop")
                && attr.syntax().text().to_string().contains("skip_setter")
        })
    }

    fn render_slot_carrier_param_indexes(&self, function: &ast::Fn) -> BTreeSet<usize> {
        let Some(param_list) = function.param_list() else {
            return BTreeSet::new();
        };
        param_list
            .params()
            .enumerate()
            .filter(|(_, param)| self.param_is_render_slot_carrier(param))
            .map(|(index, _)| index)
            .collect()
    }

    fn param_is_render_slot_carrier(&self, param: &ast::Param) -> bool {
        let Some(ty) = param.ty() else {
            return false;
        };
        self.type_is_render_slot_handle(ty.clone()) || self.type_is_impl_into_render_slot_handle(ty)
    }

    fn type_is_render_slot_handle(&self, ty: ast::Type) -> bool {
        self.type_is_direct_render_slot_handle(ty.clone())
            || self.type_is_optional_render_slot_handle(ty)
    }

    fn type_is_optional_render_slot_handle(&self, ty: ast::Type) -> bool {
        let mut resolved = false;
        let matches = std::iter::once(ty.clone())
            .chain(self.sema.descend_node_into_attributes(ty.clone()))
            .filter_map(|ty| {
                let ty = self.sema.resolve_type(&ty);
                resolved |= ty.is_some();
                ty
            })
            .any(|ty| self.semantic_type_is_option_of_render_slot_handle(&ty));
        matches || (!resolved && self.declared_type_is_render_slot(&ty, true))
    }

    fn type_is_impl_into_render_slot_handle(&self, ty: ast::Type) -> bool {
        match ty {
            ast::Type::ImplTraitType(ty) => ty.type_bound_list().is_some_and(|bounds| {
                bounds
                    .bounds()
                    .any(|bound| self.bound_is_into_render_slot_handle(&bound))
            }),
            ast::Type::ParenType(ty) => ty
                .ty()
                .is_some_and(|ty| self.type_is_impl_into_render_slot_handle(ty)),
            _ => false,
        }
    }

    fn bound_is_into_render_slot_handle(&self, bound: &ast::TypeBound) -> bool {
        let Some(ast::TypeBoundKind::PathType(_, path_ty)) = bound.kind() else {
            return false;
        };
        let Some(path) = path_ty.path() else {
            return false;
        };
        self.path_is_into_trait_with_render_slot_arg(&path)
    }

    fn path_is_into_trait_with_render_slot_arg(&self, path: &ast::Path) -> bool {
        let Some(PathResolution::Def(ModuleDef::Trait(trait_))) = self.sema.resolve_path(path)
        else {
            return false;
        };
        let Some(trait_path) = self.trait_canonical_path(trait_) else {
            return false;
        };
        if !matches!(
            trait_path.as_str(),
            CORE_INTO_TRAIT_PATH | STD_INTO_TRAIT_PATH
        ) {
            return false;
        }
        let Some(segment) = path.segment() else {
            return false;
        };
        segment.generic_arg_list().is_some_and(|args| {
            args.generic_args().any(|arg| {
                let ast::GenericArg::TypeArg(arg) = arg else {
                    return false;
                };
                arg.ty()
                    .is_some_and(|ty| self.type_is_direct_render_slot_handle(ty))
            })
        })
    }

    fn type_is_direct_render_slot_handle(&self, ty: ast::Type) -> bool {
        let mut resolved = false;
        let matches = std::iter::once(ty.clone())
            .chain(self.sema.descend_node_into_attributes(ty.clone()))
            .filter_map(|ty| {
                let ty = self.sema.resolve_type(&ty);
                resolved |= ty.is_some();
                ty
            })
            .any(|ty| self.semantic_type_is_direct_render_slot_handle(&ty));
        matches || (!resolved && self.declared_type_is_render_slot(&ty, false))
    }

    fn declared_type_is_render_slot(&self, ty: &ast::Type, optional: bool) -> bool {
        let ast::Type::PathType(ty) = ty else {
            return false;
        };
        let Some(path) = ty.path() else { return false };
        let resolution = self.sema.resolve_path(&path).or_else(|| {
            let owner = path.syntax().ancestors().find_map(ast::Fn::cast)?;
            self.sema.scope(owner.syntax())?.speculative_resolve(&path)
        });
        let Some(PathResolution::Def(definition)) = resolution else {
            return false;
        };
        if let ModuleDef::TypeAlias(alias) = definition {
            let ty = alias.ty(self.db);
            return if optional {
                self.semantic_type_is_option_of_render_slot_handle(&ty)
            } else {
                self.semantic_type_is_direct_render_slot_handle(&ty)
            };
        }
        let Some(type_path) = self.module_def_canonical_path(definition) else {
            return false;
        };
        if !optional {
            return matches!(
                type_path.as_str(),
                RENDER_SLOT_TYPE_PATH
                    | RENDER_SLOT_WITH_TYPE_PATH
                    | PUBLIC_RENDER_SLOT_TYPE_PATH
                    | PUBLIC_RENDER_SLOT_WITH_TYPE_PATH
            );
        }
        if !matches!(
            type_path.as_str(),
            CORE_OPTION_TYPE_PATH | STD_OPTION_TYPE_PATH
        ) {
            return false;
        }
        path.segment()
            .and_then(|segment| segment.generic_arg_list())
            .is_some_and(|args| {
                args.generic_args().any(|arg| {
                    let ast::GenericArg::TypeArg(arg) = arg else {
                        return false;
                    };
                    arg.ty()
                        .is_some_and(|ty| self.type_is_direct_render_slot_handle(ty))
                })
            })
    }

    fn analyze_assoc_item(&mut self, item: ast::AssocItem) {
        if let ast::AssocItem::Fn(function) = item {
            self.analyze_function(function);
        }
    }

    fn analyze_function(&mut self, function: ast::Fn) {
        if self.is_inactive(function.syntax()) {
            return;
        }
        let Some(body) = function.body() else {
            return;
        };
        let expanded = self
            .sema
            .expand_attr_macro(&ast::Item::Fn(function.clone()));
        let expanded_functions = expanded.iter().flat_map(|expansion| {
            expansion
                .value
                .value
                .descendants()
                .filter_map(ast::Fn::cast)
        });
        for candidate in std::iter::once(function.clone()).chain(expanded_functions) {
            let Some(definition) = self.sema.to_def(&candidate) else {
                continue;
            };
            let Ok(id) = DefWithBodyId::try_from(ra_ap_hir::DefWithBody::Function(definition))
            else {
                continue;
            };
            let (_, source_map) = Body::with_source_map(self.db, id);
            for diagnostic in source_map.diagnostics() {
                if let ExpressionStoreDiagnostics::InactiveCode { node, .. } = diagnostic {
                    let root = self.sema.parse_or_expand(node.file_id);
                    let source = self.sema.original_range(&node.value.to_node(&root));
                    self.inactive_bodies
                        .entry(source.file_id.file_id(self.db))
                        .or_default()
                        .push(source.range);
                }
            }
        }
        let allow_entry_point_new = self.is_entry_function(&function);
        let color =
            if self.is_tessera_function(&function) || self.is_runtime_api_function(&function) {
                ContextColor::Tessera
            } else {
                ContextColor::Plain
            };
        self.walk_node(body.syntax(), color, allow_entry_point_new);
    }

    fn analyze_closure(&mut self, closure: ast::ClosureExpr) {
        let Some(body) = closure.body() else {
            return;
        };
        let color = if self.is_tessera_carrier_closure(&closure) {
            ContextColor::Tessera
        } else {
            ContextColor::Plain
        };
        if color == ContextColor::Tessera
            && let Some(diagnostic) = self.redundant_render_slot(&closure)
        {
            self.diagnostics.push(diagnostic);
        }
        if let Some(call) = ast::CallExpr::cast(body.syntax().clone()) {
            self.analyze_call(call, color, false);
        } else if let Some(call) = ast::MethodCallExpr::cast(body.syntax().clone()) {
            self.analyze_method_call(call, color);
        }
        self.walk_node(body.syntax(), color, false);
    }

    fn walk_node(&mut self, node: &SyntaxNode, color: ContextColor, allow_entry_point_new: bool) {
        for child in node.children() {
            if self.is_inactive(&child) {
                continue;
            }
            if let Some(function) = ast::Fn::cast(child.clone()) {
                self.analyze_function(function);
                continue;
            }
            if let Some(closure) = ast::ClosureExpr::cast(child.clone()) {
                self.analyze_closure(closure);
                continue;
            }
            if let Some(call) = ast::CallExpr::cast(child.clone()) {
                self.analyze_call(call, color, allow_entry_point_new);
                self.walk_node(&child, color, allow_entry_point_new);
                continue;
            }
            if let Some(call) = ast::MethodCallExpr::cast(child.clone()) {
                self.analyze_method_call(call, color);
                self.walk_node(&child, color, allow_entry_point_new);
                continue;
            }
            self.walk_node(&child, color, allow_entry_point_new);
        }
    }

    fn analyze_call(
        &mut self,
        call: ast::CallExpr,
        color: ContextColor,
        allow_entry_point_new: bool,
    ) {
        self.check_model_state(&call);
        let Some(target) = self.resolve_call_target(&call) else {
            return;
        };

        match target {
            CallTarget::Resolved { path, kind } => {
                let is_allowed_entry_point_new = allow_entry_point_new
                    && matches!(kind, CallKind::RuntimeApi(TesseraRuntimeApi::EntryPointNew));
                if color == ContextColor::Plain && !is_allowed_entry_point_new {
                    self.push_forbidden_call(call.syntax(), &path, kind);
                }
            }
            CallTarget::Unresolved { path } => {
                let is_allowed_entry_point_new =
                    allow_entry_point_new && Self::is_entry_point_constructor_path(&path);
                if color == ContextColor::Plain && !is_allowed_entry_point_new {
                    self.push_unresolved_call(call.syntax(), &path);
                }
            }
        }
    }

    fn analyze_method_call(&mut self, call: ast::MethodCallExpr, color: ContextColor) {
        if color == ContextColor::Tessera
            && let Some(diagnostic) = self.redundant_shared_render_slot(&call)
        {
            self.diagnostics.push(diagnostic);
        }
        let function = self
            .method_call_variants(&call)
            .find_map(|call| self.sema.resolve_method_call(&call));
        if let Some(function) = function {
            let diagnostics = self.remembered_state_in_model(&call, function);
            self.diagnostics.extend(diagnostics);
            if color == ContextColor::Plain
                && let Some(CallTarget::Resolved { path, kind }) =
                    self.call_target_for_function(function)
            {
                self.push_forbidden_call(call.syntax(), &path, kind);
            }
        } else if color == ContextColor::Plain {
            let path = self.method_call_path(&call);
            if self.is_semantically_relevant_unresolved_method(&path) {
                self.push_unresolved_call(call.syntax(), &path);
            }
        }
    }

    fn resolve_call_target(&self, call: &ast::CallExpr) -> Option<CallTarget> {
        let expr = call.expr()?;
        let mut known = false;
        let mut unresolved = None;
        for expr in
            std::iter::once(expr.clone()).chain(self.sema.descend_node_into_attributes(expr))
        {
            let (resolved, target) = self.resolve_call_expr_target(&expr);
            if resolved {
                known = true;
                if target.is_some() {
                    return target;
                }
            } else if unresolved.is_none() {
                unresolved = target;
            }
        }
        if known { None } else { unresolved }
    }

    fn resolve_call_expr_target(&self, expr: &ast::Expr) -> (bool, Option<CallTarget>) {
        if ast::ClosureExpr::can_cast(expr.syntax().kind()) {
            return (true, None);
        }
        if let Some(callable) = self.sema.resolve_expr_as_callable(expr) {
            return (
                true,
                match callable.kind() {
                    CallableKind::Function(function) => self.call_target_for_function(function),
                    _ => None,
                },
            );
        }
        if let Some(path) = expr_path(expr) {
            if let Some(resolution) = self.sema.resolve_path(&path) {
                return (true, self.call_target_for_path_resolution(resolution));
            }
            let path = path_text(&path);
            if self.is_semantically_relevant_unresolved_path(&path) {
                return (false, Some(CallTarget::Unresolved { path }));
            }
        }
        (false, None)
    }

    fn call_target_for_path_resolution(&self, resolution: PathResolution) -> Option<CallTarget> {
        match resolution {
            PathResolution::Def(ModuleDef::Function(function)) => {
                self.call_target_for_function(function)
            }
            _ => None,
        }
    }

    fn call_target_for_function(&self, function: Function) -> Option<CallTarget> {
        if let Some(target) = self.call_targets.borrow().get(&function) {
            return target.clone();
        }
        let path = self.function_path(function);
        let kind = if self.is_tessera_function_def(function) {
            Some(CallKind::TesseraFunction)
        } else {
            self.runtime_api_for_function(function, &path)
                .map(CallKind::RuntimeApi)
        };
        let target = kind.map(|kind| CallTarget::Resolved { path, kind });
        self.call_targets
            .borrow_mut()
            .insert(function, target.clone());
        target
    }

    fn is_tessera_function(&self, function: &ast::Fn) -> bool {
        if self.is_free_function(function)
            && function
                .attrs()
                .any(|attr| self.is_tessera_attribute(&attr))
        {
            return true;
        }

        let Some(def) = self.sema.to_fn_def(function) else {
            return false;
        };
        self.is_tessera_function_def(def)
    }

    // A Tessera-only API runs only in Tessera contexts, so its own body is one.
    fn is_runtime_api_function(&self, function: &ast::Fn) -> bool {
        self.is_free_function(function)
            && self.sema.to_fn_def(function).is_some_and(|def| {
                Self::runtime_api_for_canonical_path(&self.function_path(def)).is_some()
            })
    }

    fn is_free_function(&self, function: &ast::Fn) -> bool {
        function.syntax().parent().is_some_and(|parent| {
            ast::SourceFile::can_cast(parent.kind()) || ast::ItemList::can_cast(parent.kind())
        })
    }

    fn is_entry_function(&self, function: &ast::Fn) -> bool {
        self.is_free_function(function)
            && function.attrs().any(|attr| self.is_entry_attribute(&attr))
    }

    fn is_tessera_function_def(&self, function: Function) -> bool {
        if function.as_assoc_item(self.db).is_some() {
            return false;
        }

        let path = self.function_path(function);
        if self.index.tessera_function_paths.contains(&path) {
            return true;
        }

        let Some(source) = self.sema.source(function) else {
            return false;
        };
        let original = self
            .sema
            .original_ast_node(source.value.clone())
            .unwrap_or(source.value);
        let file_id = self
            .sema
            .original_range(original.syntax())
            .file_id
            .file_id(self.db);
        if !self.local_files.contains(&file_id) {
            return false;
        }
        if original
            .name()
            .is_none_or(|name| name.text() != function.name(self.db).as_str())
        {
            return false;
        }

        original
            .attrs()
            .any(|attr| self.is_tessera_attribute(&attr))
    }

    fn is_tessera_attribute(&self, attr: &ast::Attr) -> bool {
        let Some(path) = attr.path() else {
            return false;
        };
        if let Some(PathResolution::Def(ModuleDef::Macro(mac))) = self.sema.resolve_path(&path) {
            return matches!(mac.name(self.db).as_str(), "tessera" | "shard")
                && self.is_tessera_crate(mac.module(self.db).krate(self.db).into());
        }

        false
    }

    fn is_entry_attribute(&self, attr: &ast::Attr) -> bool {
        let Some(path) = attr.path() else {
            return false;
        };
        if let Some(PathResolution::Def(ModuleDef::Macro(mac))) = self.sema.resolve_path(&path) {
            return mac.name(self.db).as_str() == "entry"
                && self.is_tessera_crate(mac.module(self.db).krate(self.db).into());
        }

        false
    }

    fn runtime_api_for_canonical_path(canonical_path: &str) -> Option<TesseraRuntimeApi> {
        match canonical_path {
            REMEMBER_PATH | PUBLIC_REMEMBER_PATH => Some(TesseraRuntimeApi::Remember),
            REMEMBER_WITH_KEY_PATH | PUBLIC_REMEMBER_WITH_KEY_PATH => {
                Some(TesseraRuntimeApi::RememberWithKey)
            }
            RETAIN_PATH | PUBLIC_RETAIN_PATH => Some(TesseraRuntimeApi::Retain),
            RETAIN_WITH_KEY_PATH | PUBLIC_RETAIN_WITH_KEY_PATH => {
                Some(TesseraRuntimeApi::RetainWithKey)
            }
            PROVIDE_CONTEXT_PATH | PUBLIC_PROVIDE_CONTEXT_PATH => {
                Some(TesseraRuntimeApi::ProvideContext)
            }
            USE_CONTEXT_PATH | PUBLIC_USE_CONTEXT_PATH => Some(TesseraRuntimeApi::UseContext),
            LAUNCH_PATH | PUBLIC_LAUNCH_PATH => Some(TesseraRuntimeApi::Launch),
            LAUNCH_ONCE_PATH | PUBLIC_LAUNCH_ONCE_PATH => Some(TesseraRuntimeApi::LaunchOnce),
            USE_ROUTER_PATH | PUBLIC_USE_ROUTER_PATH => Some(TesseraRuntimeApi::UseRouter),
            KEY_PATH | PUBLIC_KEY_PATH => Some(TesseraRuntimeApi::Key),
            path => {
                let name = path.rsplit("::").next().unwrap_or(path);
                if path.starts_with(RUNTIME_MODULE_PATH_PREFIX) && is_internal_runtime_name(name) {
                    Some(TesseraRuntimeApi::InternalRuntime)
                } else {
                    None
                }
            }
        }
    }

    fn runtime_api_for_function(
        &self,
        function: Function,
        canonical_path: &str,
    ) -> Option<TesseraRuntimeApi> {
        if self.is_runtime_api_definition(function) {
            return None;
        }

        if let Some(api) = Self::runtime_api_for_canonical_path(canonical_path) {
            return Some(api);
        }

        let name = function.name(self.db).as_str().to_string();
        match name.as_str() {
            "new" if self.is_entry_point_method(function) => Some(TesseraRuntimeApi::EntryPointNew),
            "new" if self.is_render_slot_method(function, RENDER_SLOT_TYPE_PATH) => {
                Some(TesseraRuntimeApi::RenderSlotNew)
            }
            "new" if self.is_render_slot_method(function, PUBLIC_RENDER_SLOT_TYPE_PATH) => {
                Some(TesseraRuntimeApi::RenderSlotNew)
            }
            "new" if self.is_render_slot_method(function, RENDER_SLOT_WITH_TYPE_PATH) => {
                Some(TesseraRuntimeApi::RenderSlotWithNew)
            }
            "new" if self.is_render_slot_method(function, PUBLIC_RENDER_SLOT_WITH_TYPE_PATH) => {
                Some(TesseraRuntimeApi::RenderSlotWithNew)
            }
            "render"
                if [RENDER_SLOT_TYPE_PATH, PUBLIC_RENDER_SLOT_TYPE_PATH]
                    .iter()
                    .any(|path| self.is_render_slot_method(function, path)) =>
            {
                Some(TesseraRuntimeApi::RenderSlotRender)
            }
            "render"
                if [
                    RENDER_SLOT_WITH_TYPE_PATH,
                    PUBLIC_RENDER_SLOT_WITH_TYPE_PATH,
                ]
                .iter()
                .any(|path| self.is_render_slot_method(function, path)) =>
            {
                Some(TesseraRuntimeApi::RenderSlotWithRender)
            }
            _ => None,
        }
    }

    fn is_runtime_api_definition(&self, function: Function) -> bool {
        let Some(source) = self.sema.source(function) else {
            return false;
        };
        let Some(body) = source.value.body() else {
            return false;
        };

        body.syntax()
            .text_range()
            .contains_range(source.value.syntax().text_range())
    }

    fn is_render_slot_method(&self, function: Function, expected_type_path: &str) -> bool {
        let Some(assoc_item) = function.as_assoc_item(self.db) else {
            return false;
        };
        assoc_item
            .implementing_ty(self.db)
            .and_then(|ty| self.type_adt_canonical_path(&ty))
            .is_some_and(|type_path| type_path == expected_type_path)
    }

    fn is_entry_point_method(&self, function: Function) -> bool {
        let Some(assoc_item) = function.as_assoc_item(self.db) else {
            return false;
        };
        assoc_item
            .implementing_ty(self.db)
            .and_then(|ty| self.type_adt_canonical_path(&ty))
            .is_some_and(|type_path| {
                matches!(
                    type_path.as_str(),
                    ENTRY_POINT_TYPE_PATH | PUBLIC_ENTRY_POINT_TYPE_PATH
                )
            })
    }

    fn is_tessera_crate(&self, krate: Crate) -> bool {
        self.index.tessera_crates.contains(&krate)
    }

    fn type_adt_canonical_path(&self, ty: &Type<'db>) -> Option<String> {
        let mut ty = ty.clone();
        while let Some((inner, _)) = ty.as_reference() {
            ty = inner;
        }

        let adt = ty.as_adt()?;
        self.module_def_canonical_path(ModuleDef::from(adt))
    }

    fn semantic_type_is_direct_render_slot_handle(&self, ty: &Type<'db>) -> bool {
        self.type_adt_canonical_path(ty).is_some_and(|type_path| {
            matches!(
                type_path.as_str(),
                RENDER_SLOT_TYPE_PATH
                    | RENDER_SLOT_WITH_TYPE_PATH
                    | PUBLIC_RENDER_SLOT_TYPE_PATH
                    | PUBLIC_RENDER_SLOT_WITH_TYPE_PATH
            )
        })
    }

    fn semantic_type_is_option_of_render_slot_handle(&self, ty: &Type<'db>) -> bool {
        let mut ty = ty.clone();
        while let Some((inner, _)) = ty.as_reference() {
            ty = inner;
        }

        let Some((adt, args)) = ty.as_adt_with_args() else {
            return false;
        };
        let Some(type_path) = self.module_def_canonical_path(ModuleDef::from(adt)) else {
            return false;
        };
        if !matches!(
            type_path.as_str(),
            CORE_OPTION_TYPE_PATH | STD_OPTION_TYPE_PATH
        ) {
            return false;
        }
        args.into_iter()
            .flatten()
            .any(|arg| self.semantic_type_is_direct_render_slot_handle(&arg))
    }

    fn trait_canonical_path(&self, trait_: Trait) -> Option<String> {
        self.module_def_canonical_path(ModuleDef::from(trait_))
    }

    fn is_tessera_carrier_closure(&self, closure: &ast::ClosureExpr) -> bool {
        self.is_direct_tessera_carrier_closure(closure)
    }

    fn is_direct_tessera_carrier_closure(&self, closure: &ast::ClosureExpr) -> bool {
        let closure_range = closure.syntax().text_range();
        let mut current = closure.syntax().parent();
        while let Some(node) = current {
            if ast::ClosureExpr::can_cast(node.kind()) {
                return false;
            }
            if let Some(block) = ast::BlockExpr::cast(node.clone()) {
                if block.async_token().is_some()
                    || block.gen_token().is_some()
                    || block.const_token().is_some()
                    || block
                        .stmt_list()
                        .and_then(|list| list.tail_expr())
                        .is_none_or(|tail| {
                            !tail.syntax().text_range().contains_range(closure_range)
                        })
                {
                    return false;
                }
            }
            if let Some(call) = ast::CallExpr::cast(node.clone()) {
                let Some(index) = call.arg_list().and_then(|arg_list| {
                    arg_list
                        .args()
                        .position(|arg| arg.syntax().text_range().contains_range(closure_range))
                }) else {
                    return false;
                };
                if index == 0 && self.call_is_box_constructor(&call) {
                    current = node.parent();
                    continue;
                }
                return self.call_takes_tessera_carrier_closure_at(&call, index);
            }
            if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
                let Some(index) = call.arg_list().and_then(|arg_list| {
                    arg_list
                        .args()
                        .position(|arg| arg.syntax().text_range().contains_range(closure_range))
                }) else {
                    return false;
                };
                return self.method_call_takes_render_slot_setter_at(&call, index)
                    || self.method_call_takes_render_slot_closure_at(&call, index)
                    || self.method_call_takes_pending_property_at(&call, index)
                    || self.method_call_is_build_entry_at(&call, index);
            }
            current = node.parent();
        }

        false
    }

    fn call_takes_tessera_carrier_closure_at(&self, call: &ast::CallExpr, index: usize) -> bool {
        if index == 0 && self.call_is_build_entry(&call) {
            return true;
        }
        match self.resolve_call_target(call) {
            Some(CallTarget::Resolved {
                kind:
                    CallKind::RuntimeApi(
                        TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew,
                    ),
                ..
            }) => index == 0,
            Some(CallTarget::Resolved {
                kind:
                    CallKind::RuntimeApi(TesseraRuntimeApi::ProvideContext | TesseraRuntimeApi::Key),
                ..
            }) => index == 1,
            Some(CallTarget::Resolved {
                path,
                kind: CallKind::TesseraFunction,
            }) => self
                .index
                .render_slot_constructor_indexes_by_function_path
                .get(&path)
                .is_some_and(|indexes| indexes.contains(&index)),
            _ => false,
        }
    }

    fn method_call_takes_pending_property_at(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        index == 1
            && self.method_call_variants(call).any(|call| {
                self.sema
                    .resolve_method_call(&call)
                    .is_some_and(|function| {
                        matches!(
                            function.name(self.db).as_str(),
                            "append" | "configure_deferred"
                        ) && self.is_render_slot_method(
                            function,
                            "tessera_ui::component_call::ComponentCall",
                        )
                    })
            })
    }

    fn call_resolves_to(&self, call: &ast::CallExpr, matches: impl Fn(Function) -> bool) -> bool {
        let Some(expr) = call.expr() else {
            return false;
        };
        std::iter::once(expr.clone())
            .chain(self.sema.descend_node_into_attributes(expr))
            .filter_map(|expr| {
                if let Some(callable) = self.sema.resolve_expr_as_callable(&expr)
                    && let CallableKind::Function(function) = callable.kind()
                {
                    return Some(function);
                }
                let path = expr_path(&expr)?;
                match self.sema.resolve_path(&path)? {
                    PathResolution::Def(ModuleDef::Function(function)) => Some(function),
                    _ => None,
                }
            })
            .any(matches)
    }

    fn call_is_box_constructor(&self, call: &ast::CallExpr) -> bool {
        self.call_resolves_to(call, |function| {
            function.name(self.db).as_str() == "new"
                && ["alloc::boxed::Box", "std::boxed::Box"]
                    .iter()
                    .any(|path| self.is_render_slot_method(function, path))
        })
    }

    fn call_is_build_entry(&self, call: &ast::CallExpr) -> bool {
        self.call_resolves_to(call, |function| {
            let name = function.name(self.db);
            (name.as_str() == "new"
                && (self.is_entry_point_method(function)
                    || self
                        .is_render_slot_method(function, "tessera_ui::headless::HeadlessRenderer")))
                || (matches!(
                    name.as_str(),
                    "run" | "run_with_config" | "run_web_with_config"
                ) && self.is_render_slot_method(function, "tessera_ui::renderer::Renderer"))
        })
    }

    fn method_call_is_build_entry_at(&self, call: &ast::MethodCallExpr, index: usize) -> bool {
        index == 0
            && self.method_call_variants(call).any(|call| {
                self.sema
                    .resolve_method_call(&call)
                    .is_some_and(|function| {
                        function.name(self.db).as_str() == "content"
                            && self.is_render_slot_method(function, "tessera_ui::testing::UiTest")
                    })
            })
    }

    fn method_call_takes_render_slot_closure_at(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        self.method_source_param_is_render_slot_carrier(call, index)
    }

    fn method_source_param_is_render_slot_carrier(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        self.method_call_variants(call).any(|call| {
            let Some(function) = self.sema.resolve_method_call(&call) else {
                return false;
            };
            let Some(source) = self.sema.source(function) else {
                return false;
            };
            let Some(params) = source.value.param_list() else {
                return false;
            };
            params
                .params()
                .nth(index)
                .is_some_and(|param| self.param_is_render_slot_carrier(&param))
        })
    }

    fn method_call_takes_render_slot_setter_at(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        let Some(name) = call.name_ref() else {
            return false;
        };
        let name = name.text();

        self.method_call_variants(call).any(|variant| {
            let receiver = variant
                .receiver()
                .and_then(|receiver| self.receiver_type_key(&receiver));
            let implementation = self.method_call_implementing_type_key(&variant);
            receiver
                .into_iter()
                .chain(implementation)
                .any(|path| self.setter_accepts_render_slot_at(&path, name, index))
        })
    }

    fn setter_accepts_render_slot_at(
        &self,
        receiver_type_path: &str,
        name: &str,
        index: usize,
    ) -> bool {
        self.index
            .render_slot_setter_indexes_by_type_path
            .get(receiver_type_path)
            .and_then(|setters| setters.get(name))
            .is_some_and(|indexes| indexes.contains(&index))
            || Self::builtin_render_slot_argument(receiver_type_path, name) == Some(index)
    }

    fn builtin_render_slot_argument(receiver: &str, method: &str) -> Option<usize> {
        match receiver {
            "tessera_components::app_bar::TopAppBarBuilder" if method == "action" => Some(0),
            "tessera_components::button_groups::ButtonGroupsBuilder" if method == "child" => {
                Some(0)
            }
            "tessera_components::tabs::TabBuilder" if method == "title" => Some(0),
            "tessera_components::lazy_list::LazyColumnBuilder"
            | "tessera_components::lazy_list::LazyRowBuilder"
            | "tessera_components::lazy_grid::LazyVerticalGridBuilder"
            | "tessera_components::lazy_grid::LazyHorizontalGridBuilder"
            | "tessera_components::lazy_staggered_grid::LazyVerticalStaggeredGridBuilder"
            | "tessera_components::lazy_staggered_grid::LazyHorizontalStaggeredGridBuilder" => {
                match method {
                    "item" | "sticky_header" => Some(0),
                    "item_with_key"
                    | "items"
                    | "items_from_iter"
                    | "items_from_iter_values"
                    | "sticky_header_with_key" => Some(1),
                    "items_with_key" | "items_from_iter_with_key" => Some(2),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn receiver_type_key(&self, receiver: &ast::Expr) -> Option<String> {
        std::iter::once(receiver.clone())
            .chain(self.sema.descend_node_into_attributes(receiver.clone()))
            .find_map(|receiver| {
                let ty = self.sema.type_of_expr(&receiver)?.adjusted();
                self.type_adt_canonical_path(&ty)
            })
    }

    fn method_call_implementing_type_key(&self, call: &ast::MethodCallExpr) -> Option<String> {
        self.method_call_variants(call).find_map(|call| {
            let function = self.sema.resolve_method_call(&call)?;
            let assoc_item = function.as_assoc_item(self.db)?;
            assoc_item
                .implementing_ty(self.db)
                .and_then(|ty| self.type_adt_canonical_path(&ty))
        })
    }

    fn method_call_variants<'a>(
        &'a self,
        call: &'a ast::MethodCallExpr,
    ) -> impl Iterator<Item = ast::MethodCallExpr> + 'a {
        std::iter::once(call.clone()).chain(
            call.name_ref()
                .into_iter()
                .flat_map(|name| self.sema.descend_node_into_attributes(name))
                .filter_map(|name| name.syntax().parent().and_then(ast::MethodCallExpr::cast)),
        )
    }

    fn is_semantically_relevant_unresolved_path(&self, path: &str) -> bool {
        let last = path.rsplit("::").next().unwrap_or(path);
        matches!(
            last,
            "remember"
                | "remember_with_key"
                | "retain"
                | "retain_with_key"
                | "provide_context"
                | "use_context"
                | "launch"
                | "launch_once"
                | "use_router"
                | "key"
        ) || self.index.tessera_function_names.contains(last)
            || Self::is_render_slot_constructor_path(path)
            || Self::is_entry_point_constructor_path(path)
            || is_internal_runtime_name(last)
    }

    fn is_semantically_relevant_unresolved_method(&self, path: &str) -> bool {
        Self::is_render_slot_constructor_path(path) || Self::is_entry_point_constructor_path(path)
    }

    fn is_render_slot_constructor_path(path: &str) -> bool {
        matches!(
            path,
            RENDER_SLOT_NEW_PATH
                | RENDER_SLOT_WITH_NEW_PATH
                | PUBLIC_RENDER_SLOT_NEW_PATH
                | PUBLIC_RENDER_SLOT_WITH_NEW_PATH
                | "RenderSlot::new"
                | "RenderSlotWith::new"
        )
    }

    fn is_entry_point_constructor_path(path: &str) -> bool {
        matches!(
            path,
            ENTRY_POINT_NEW_PATH | PUBLIC_ENTRY_POINT_NEW_PATH | "EntryPoint::new"
        )
    }

    fn function_path(&self, function: Function) -> String {
        let edition = function.module(self.db).krate(self.db).edition(self.db);
        self.module_def_canonical_path(ModuleDef::from(function))
            .unwrap_or_else(|| function.name(self.db).display(self.db, edition).to_string())
    }

    fn module_def_canonical_path(&self, def: ModuleDef) -> Option<String> {
        let module = def.module(self.db)?;
        let krate = module.krate(self.db);
        let edition = krate.edition(self.db);
        let crate_name = krate.display_name(self.db)?.to_string();
        let relative_path = def.canonical_path(self.db, edition)?;
        Some(format!("{crate_name}::{relative_path}"))
    }

    fn method_call_path(&self, call: &ast::MethodCallExpr) -> String {
        let receiver = call
            .receiver()
            .map(|expr| {
                expr.syntax()
                    .text()
                    .to_string()
                    .replace(char::is_whitespace, "")
            })
            .unwrap_or_else(|| "<unknown>".to_string());
        let method = call
            .name_ref()
            .map(|name| name.text().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        format!("{receiver}::{method}")
    }

    fn push_forbidden_call(&mut self, syntax: &SyntaxNode, target: &str, kind: CallKind) {
        let (code, message, label) = match kind {
            CallKind::TesseraFunction => (
                ErrorCode::ComponentCall,
                format!("cannot call Tessera component `{target}` here"),
                "component call requires a Tessera build context",
            ),
            CallKind::RuntimeApi(api) => (
                ErrorCode::RuntimeCall,
                format!(
                    "cannot call Tessera-only API `{target}` ({}) here",
                    runtime_api_label(api)
                ),
                "runtime API requires a Tessera build context",
            ),
        };
        self.push_diagnostic(syntax, code, message, label,
            "move this call into a component or recognized build closure; ordinary callbacks should update state instead");
    }

    fn push_unresolved_call(&mut self, syntax: &SyntaxNode, target: &str) {
        self.push_diagnostic(
            syntax,
            ErrorCode::UnresolvedCall,
            format!("cannot resolve Tessera-sensitive call `{target}`"),
            "the call target must be resolved before this context can be checked",
            "check the import, dependency, selected Cargo features, and target",
        );
    }

    fn push_diagnostic(
        &mut self,
        syntax: &SyntaxNode,
        code: ErrorCode,
        message: String,
        label: &str,
        help: &'static str,
    ) {
        let source = self.sema.original_range(syntax);
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id) {
            return;
        }
        let boundary = syntax
            .ancestors()
            .skip(1)
            .find_map(|node| {
                if let Some(closure) = ast::ClosureExpr::cast(node.clone()) {
                    Some((
                        closure.param_list()?.syntax().clone(),
                        "this ordinary closure is not a Tessera build context".to_string(),
                    ))
                } else if let Some(function) = ast::Fn::cast(node) {
                    Some((
                        function.name()?.syntax().clone(),
                        "this function is not a Tessera build context".to_string(),
                    ))
                } else {
                    None
                }
            })
            .and_then(|(node, message)| {
                let boundary = self.sema.original_range(&node);
                (boundary.file_id.file_id(self.db) == file_id).then_some((boundary.range, message))
            });
        self.diagnostics.push(Diagnostic {
            file_id,
            range: source.range,
            code: code.into(),
            severity: cargo_metadata::diagnostic::DiagnosticLevel::Error,
            message,
            label: label.to_string(),
            help: help.into(),
            boundary,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }

    #[cfg(test)]
    fn absolute_path(&self, file_id: FileId) -> String {
        self.vfs
            .file_path(file_id)
            .as_path()
            .map(|path| {
                let std_path: &std::path::Path = path.as_ref();
                std_path.display().to_string()
            })
            .unwrap_or_else(|| self.vfs.file_path(file_id).to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::super::{CheckOptions, types::CallAnalyzer, workspace};

    #[test]
    fn dependency_components_and_slots_are_indexed_without_unrelated_members() {
        with_analyzer(
            r#"
use fixture_widgets::foreign_component;

#[tessera_ui::tessera]
fn local_component() {}

#[tessera_ui::tessera]
fn host() {
    foreign_component().children(|| { local_component(); });
}

fn plain() { foreign_component(); }
"#,
            |analyzer| {
                assert_eq!(analyzer.diagnostics.len(), 1);
                assert!(
                    analyzer.diagnostics[0]
                        .message
                        .contains("foreign_component"),
                    "{}; indexed components: {:?}",
                    analyzer.diagnostics[0].message,
                    analyzer.index.tessera_function_paths,
                );
                let paths = analyzer
                    .metadata_files
                    .iter()
                    .map(|file| analyzer.absolute_path(*file))
                    .collect::<Vec<_>>();
                assert!(
                    paths
                        .iter()
                        .any(|path| Path::new(path).ends_with("widgets/src/lib.rs"))
                );
                assert!(
                    !paths
                        .iter()
                        .any(|path| Path::new(path).ends_with("unrelated/src/lib.rs"))
                );
                assert!(
                    !paths
                        .iter()
                        .any(|path| Path::new(path).ends_with("src/orphan.rs"))
                );
                let dependency_files = analyzer
                    .metadata_files
                    .iter()
                    .copied()
                    .filter(|file| {
                        Path::new(&analyzer.absolute_path(*file)).ends_with("widgets/src/lib.rs")
                    })
                    .collect();
                let jobs = [
                    super::super::AnalysisJob {
                        index: 0,
                        name: "fixture".to_string(),
                        display: "fixture v0.0.0".to_string(),
                        files: analyzer.local_files.clone(),
                    },
                    super::super::AnalysisJob {
                        index: 1,
                        name: "widgets".to_string(),
                        display: "widgets v0.0.0".to_string(),
                        files: dependency_files,
                    },
                ];
                for workers in [1, 2] {
                    let index = super::super::collect_index_parallel(analyzer, workers)
                        .expect("parallel semantic index");
                    let progress = super::super::progress::Progress::new(false);
                    let mut results = super::super::analyze_parallel(
                        analyzer.db,
                        analyzer.vfs,
                        &index,
                        &jobs,
                        workers,
                        &super::LintOptions::default(),
                        &progress,
                    )
                    .expect("crate workers");
                    results.sort_by_key(|(index, _)| *index);
                    let messages = results
                        .into_iter()
                        .map(|(_, result)| {
                            result
                                .expect("complete semantic analysis")
                                .into_iter()
                                .map(|diagnostic| diagnostic.message)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(messages[0].len(), 1);
                    assert!(messages[0][0].contains("foreign_component"));
                    assert_eq!(messages[1].len(), 1);
                    assert!(messages[1][0].contains("remember"));
                }
            },
        );
    }

    #[test]
    fn shared_setters_warn_only_for_new_slots_and_require_ownership_review() {
        with_analyzer(
            r#"
use tessera_ui::{RenderSlot as Slot, RenderSlotWith};

#[tessera_ui::tessera]
fn child() {}

#[tessera_ui::tessera]
fn host(slot: Option<tessera_ui::RenderSlot>, value: Option<RenderSlotWith<usize>>, unit: Option<RenderSlotWith<()>>) {}

struct Other;
impl Other {
    fn slot(&self, _: usize) {}
    fn slot_shared(&self, _: Slot) {}
}

#[tessera_ui::tessera]
fn root() {
    host().slot_shared(Slot::new(|| child()));
    host().value_shared(RenderSlotWith::new(|_: usize| child()));
    host().unit_shared(RenderSlotWith::new(|()| child()));
    let saved = host();
    if true {
        saved.slot_shared(Slot::new(|| child()));
    }
    host().slot_shared({
        let label = String::from("owned capture");
        Slot::new(move || { let _ = &label; child(); })
    }).slot_shared(Slot::new(|| child()));
    let shared = Slot::new(|| child());
    host().slot_shared(shared);
    Other.slot_shared(Slot::new(|| child()));
}
"#,
            |analyzer| {
                use super::super::types::Applicability;

                assert_eq!(analyzer.diagnostics.len(), 6);
                assert!(analyzer.diagnostics.iter().all(|diagnostic| diagnostic.code
                    == super::super::error_codes::DiagnosticCode::Lint(
                        super::super::error_codes::Lint::RedundantRenderSlot
                    )));
                let suggestions: Vec<_> = analyzer
                    .diagnostics
                    .iter()
                    .flat_map(|diagnostic| &diagnostic.suggestions)
                    .collect();
                assert_eq!(suggestions.len(), 4);
                assert!(suggestions.iter().all(|suggestion| suggestion.applicability
                    == Applicability::MaybeIncorrect
                    && suggestion.edits.len() == 3));
            },
        );
    }

    #[test]
    fn direct_slot_carriers_warn_and_fix_only_redundant_forwarders() {
        with_analyzer_options(
            r#"
use tessera_ui::{EntryPoint, RenderSlot, RenderSlotWith, remember};

#[tessera_ui::tessera]
fn child() {}

#[tessera_ui::tessera]
pub fn host(slot: Option<RenderSlot>) {
    if let Some(slot) = slot {
        slot.render();
    }
}

#[tessera_ui::tessera]
pub fn required_host(value: usize, optional: Option<bool>, slot: RenderSlotWith<()>) {
    slot.render(());
}

fn configure(handle: &HostBuilder) {
    handle.slot(|| child());
    handle.custom_slot();
}

impl HostBuilder {
    fn custom_slot(&self) -> Self {
        self.props.append("slot", |props| {
            props.slot = Some(RenderSlot::new(|| { child(); }));
        });
        self.props.configure_deferred("slot", |props| {
            props.slot = Some(RenderSlot::new(|| { child(); }));
        });
        Self { props: self.props }
    }
}

#[tessera_ui::tessera]
pub fn root() {
    let _slot = RenderSlot::new(|| {
        child();
        let _state = remember(|| 1usize);
    });
    let _slot_with = RenderSlotWith::<u32>::new(|_| child());
    tessera_ui::key(7u32, || { child(); let _state = remember(|| 0u32); });
    host().slot(|| child());
    let saved = host();
    if true {
        configure(&saved);
    }
    saved.slot(|| child());
    required_host(1, || child());
    use tessera_ui::RenderSlot as Slot;
    host().slot({
        // Preserve this explanation when reviewing the simplification.
        let content = Slot::new(|| child());
        move || { content.render(); }
    });
    required_host(1, {
        let content = RenderSlotWith::new(|_: ()| child());
        move |value| content.render(value)
    });
    let content = Slot::new(|| child());
    host().slot_shared(content);
    host().slot({
        let content = Slot::new(|| child());
        move || { content.render(); child(); }
    });
    required_host(1, {
        let content = RenderSlotWith::new(|_: ()| child());
        move |value| content.render({ let _ = value; () })
    });
}

use tessera_ui::lint as lint_level;

#[cfg_attr(all(), lint_level(allow(tessera::redundant_render_slot)))]
#[cfg_attr(any(), lint_level(deny(tessera::redundant_render_slot)))]
#[tessera_ui::tessera]
pub fn conditionally_allowed() {
    host().slot({
        let content = RenderSlot::new(|| child());
        move || content.render()
    });
}

#[lint_level(deny(tessera::all))]
mod scoped {
    use super::*;

    #[tessera_ui::tessera]
    #[lint_level(allow(tessera::redundant_render_slot))]
    pub fn inner() {
        host().slot({
            let content = RenderSlot::new(|| child());
            move || content.render()
        });
    }
}

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    EntryPoint::new(|| { root(); })
}

pub fn inspect_root() {
    let _ = tessera_ui::testing::UiTest::new().content({ let _input = 1; move || { child(); } });
    let _ = tessera_ui::headless::HeadlessRenderer::new(
        Box::new(|| { child(); }), Vec::new(), Default::default(),
    );
}

#[tessera_ui::tessera]
fn themed() {
    tessera_ui::provide_context(0u32, { let _input = 1; move || { child(); } });
}
"#,
            CheckOptions {
                workspace: true,
                ..CheckOptions::new(None, None)
            },
            |analyzer| {
                use super::super::{diagnostics::Reporter, error_codes::DiagnosticCode};
                use ra_ap_base_db::SourceDatabase;

                assert_eq!(analyzer.diagnostics.len(), 2);
                assert!(analyzer.diagnostics.iter().all(|diagnostic| {
                    diagnostic.code
                        == DiagnosticCode::Lint(
                            super::super::error_codes::Lint::RedundantRenderSlot,
                        )
                }));
                let reporter = Reporter::new(analyzer, &analyzer.diagnostics);
                let mut fixes = 0;
                for diagnostic in &analyzer.diagnostics {
                    let json = reporter.json(diagnostic);
                    assert_eq!(json["message"]["level"], "warning");
                    let input = json["message"].to_string();
                    let automatic = rustfix::get_suggestions_from_json(
                        &input,
                        &std::collections::HashSet::<String>::new(),
                        rustfix::Filter::MachineApplicableOnly,
                    )
                    .expect("rustfix reads Tessera suggestions");
                    let all = rustfix::get_suggestions_from_json(
                        &input,
                        &std::collections::HashSet::<String>::new(),
                        rustfix::Filter::Everything,
                    )
                    .expect("reviewable suggestions");
                    assert_eq!(all.len(), 1);
                    assert_eq!(all[0].solutions[0].replacements.len(), 2);
                    fixes += automatic.len();
                    assert_eq!(
                        json["message"]["code"]["explanation"],
                        serde_json::Value::Null
                    );
                    let span = &json["message"]["spans"][0];
                    let source = analyzer.db.file_text(diagnostic.file_id);
                    let highlighted = &source.text(analyzer.db)[span["byte_start"].as_u64().unwrap()
                        as usize
                        ..span["byte_end"].as_u64().unwrap() as usize];
                    assert!(
                        highlighted.starts_with("Slot::new(")
                            || highlighted.starts_with("RenderSlotWith::new(")
                    );
                    for short in [false, true] {
                        let output = reporter.render(diagnostic, short, false);
                        assert!(
                            output.contains("warning[tessera::redundant_render_slot]"),
                            "{output}"
                        );
                        if short {
                            assert_eq!(output.lines().count(), 1, "{output}");
                        } else {
                            assert_eq!(
                                output.matches("pass the build closure directly").count(),
                                1,
                                "{output}"
                            );
                            assert!(!output.contains("applicability:"), "{output}");
                            assert!(!output.contains("remove this"), "{output}");
                        }
                    }
                }
                assert_eq!(fixes, 1, "comment-removing suggestions require review");
                let snapshot = analyzer.db.file_text(analyzer.diagnostics[0].file_id);
                assert!(
                    reporter
                        .fix(&analyzer.diagnostics, true)
                        .expect("apply safe fixes")
                );
                let path = analyzer.absolute_path(analyzer.diagnostics[0].file_id);
                let fixed = fs::read_to_string(&path).expect("fixed source");
                assert_ne!(fixed.as_str(), snapshot.text(analyzer.db).as_ref());
                assert!(fixed.contains("required_host(1, |_: ()| child());"));
                assert!(
                    fixed.contains(
                        "// Preserve this explanation when reviewing the simplification."
                    )
                );
                assert!(
                    ra_ap_syntax::SourceFile::parse(&fixed, ra_ap_syntax::Edition::CURRENT)
                        .errors()
                        .is_empty()
                );
                let error = reporter.fix(&analyzer.diagnostics, true).unwrap_err();
                assert!(error.to_string().contains("source changed since analysis"));
                assert_eq!(fs::read_to_string(&path).unwrap(), fixed);
                for level in [super::super::Level::Allow, super::super::Level::Deny] {
                    analyzer.diagnostics.clear();
                    analyzer
                        .check_files(&super::super::LintOptions {
                            overrides: vec![(super::super::LintTarget::All, level)],
                            cap: None,
                        })
                        .expect("recheck with configured lint level");
                    if level == super::super::Level::Allow {
                        assert!(analyzer.diagnostics.is_empty());
                    } else {
                        assert_eq!(analyzer.diagnostics.len(), 2);
                        let reporter = Reporter::new(analyzer, &analyzer.diagnostics);
                        for diagnostic in &analyzer.diagnostics {
                            let json = reporter.json(diagnostic);
                            assert_eq!(json["message"]["level"], "error");
                            assert!(
                                json["message"]["rendered"]
                                    .as_str()
                                    .unwrap()
                                    .contains("error[tessera::redundant_render_slot]")
                            );
                        }
                    }
                }
            },
        );
    }

    #[test]
    fn ordinary_closures_and_indirect_wrappers_stay_plain() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{Callback, CallbackWith, EntryPoint, RenderSlot, RenderSlotWith, remember};

#[tessera_ui::tessera]
fn child() {}

#[tessera_ui::tessera]
pub fn host(slot: Option<RenderSlot>) {}

mod plain {
    pub struct RenderSlot;
    impl<F: Fn()> From<F> for RenderSlot {
        fn from(_: F) -> Self { Self }
    }

    pub fn accept(_: impl Into<RenderSlot>) {}

    pub struct UiTest;
    impl UiTest {
        pub fn content(self, _: impl Fn()) -> Self { self }
    }

    pub struct Builder;
    impl Builder {
        pub fn slot(&self, render: impl Fn()) {}
    }
    pub fn host() -> Builder { Builder }
}

fn wrapper(render: impl Fn() + Send + Sync + 'static) {
    let _slot = RenderSlot::new(render);
}

#[tessera_ui::tessera]
pub fn root() {
    let ordinary = || {
        child();
        let _state = remember(|| 1usize);
    };
    ordinary();
    Some(()).map(|_| child());
    wrapper(|| { child(); });
    plain::host().slot(|| { child(); });
    plain::accept(|| { child(); });
    plain::UiTest.content(|| { child(); });
    let _boxed = Box::new(|| { child(); });
    let _ = remember(|| { let _ = remember(|| 1usize); 0u32 });
}

fn render_from_helper(slot: RenderSlot, indexed: RenderSlotWith<u32>) {
    slot.render();
    indexed.render(1);
    RenderSlot::render(&slot);
    RenderSlotWith::render(&indexed, 2);
}

#[tessera_ui::tessera]
fn wrapped_render_calls() {
    let slot = RenderSlot::new(child);
    let indexed = RenderSlotWith::new(|_: u32| { child(); });
    let _ = Callback::new(move || slot.render());
    let _ = CallbackWith::new(move |value| indexed.render(value));
    let _ = move || slot.render();
    let _ = Some(1).map(move |value| indexed.render(value));
}

pub fn plain_entry() -> EntryPoint {
    EntryPoint::new(root)
}

#[test]
fn test_functions_are_plain() { child(); }
"#,
        );

        assert_contains(&diagnostics, "RenderSlot::new");
        assert_contains(&diagnostics, "EntryPoint::new");
        assert_eq!(count_containing(&diagnostics, "RenderSlot::render"), 4);
        assert_eq!(count_containing(&diagnostics, "RenderSlotWith::render"), 4);
        assert_contains(&diagnostics, "remember");
        assert!(
            count_containing(&diagnostics, "remember") >= 2,
            "initializer factories stay plain: {diagnostics:?}"
        );
        assert!(
            count_containing(&diagnostics, "Tessera component") >= 8,
            "expected ordinary closures to report Tessera component calls:\n{}",
            diagnostics.join("\n")
        );
    }

    #[test]
    fn builtin_render_setters_color_content_without_coloring_keys_or_actions() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_components::{app_bar::top_app_bar, button_groups::button_groups, lazy_grid::{GridCells, lazy_vertical_grid}, lazy_list::lazy_column, tabs::tab};

#[tessera_ui::tessera]
fn child() {}

#[tessera_ui::tessera]
pub fn root(value: usize) {
    let _slot = tessera_ui::RenderSlot::new(move || {
    let _ = value;
    top_app_bar().action(|| child());
    #[cfg(not(target_family = "wasm"))]
    {
        top_app_bar()
            .title(if true { "Title" } else { "Other" })
            .title_area_modifier(tessera_ui::Modifier::new())
            .navigation_icon(|| { child(); })
            .window_control_minimize();
    }
    #[cfg(target_family = "wasm")]
    {
        top_app_bar().navigation_icon(|| { child(); }).title("Web");
    }
    tab().title(|| child());
    button_groups().child(|| child(), |_| {});
    lazy_column().item(child).items(2, |_| child()).items_from_iter(vec![1, 2], |_, _| child());
    lazy_vertical_grid().columns(GridCells::fixed(2)).items_with_key(2, |index| index, |_| child());
    lazy_column().items_with_key(2, |index| { child(); index }, |_| {});
    button_groups().child(|| {}, |_| { child(); });
    });
}
"#,
        );
        assert_eq!(
            count_containing(&diagnostics, "Tessera component"),
            2,
            "{}",
            diagnostics.join("\n")
        );
    }

    #[test]
    fn launch_requires_a_tessera_context() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{launch, launch_once};

#[tessera_ui::tessera]
pub fn screen(id: u32) {
    launch(id, async {});
    launch_once(async {});
}

pub fn helper() {
    launch(1u32, async {});
    tessera_ui::launch_once(async {});
}
"#,
        );
        assert_eq!(
            count_containing(&diagnostics, "(launch)"),
            1,
            "{diagnostics:?}"
        );
        assert_eq!(
            count_containing(&diagnostics, "(launch_once)"),
            1,
            "{diagnostics:?}"
        );
    }

    #[test]
    fn user_functions_named_like_runtime_apis_remain_plain() {
        let diagnostics = diagnostics_for(
            r#"
fn remember() {}
fn remember_with_key() {}
fn retain() {}
fn retain_with_key() {}
fn provide_context() {}
fn use_context() {}
fn receive_frame_nanos() {}
fn key() {}
fn current_instance() {}
struct Ordinary;
impl Ordinary { fn render(&self) {} }

pub fn helper() {
    remember();
    remember_with_key();
    retain();
    retain_with_key();
    provide_context();
    use_context();
    receive_frame_nanos();
    key();
    current_instance();
    Ordinary.render();
}
"#,
        );

        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics for user-owned function names:\n{}",
            diagnostics.join("\n")
        );
    }

    #[test]
    fn private_macro_support_items_are_rejected_outside_official_expansions() {
        let source = r#"
use tessera_ui::__private::GroupGuard;
use tessera_ui::{Dp, __private::current_instance_key as instance_key};
use tessera_ui::__private as support;

mod shim {
    pub use tessera_ui::__private::current_instance_key as instance;
}

macro_rules! leak {
    () => { tessera_ui::__private::current_instance_key() };
}

#[tessera_ui::tessera]
fn generated(width: Dp) {
    let _ = width;
}

fn direct() -> u64 { tessera_ui::__private::current_instance_key() }
fn imported() -> u64 { instance_key() }
fn reexported() -> u64 { shim::instance() }
fn module_alias<T: support::Prop>() {}
fn expanded() -> u64 { leak!() }
fn guard(_: Option<GroupGuard>) {}
"#;
        with_analyzer(source, |analyzer| {
            use ra_ap_base_db::SourceDatabase;
            use ra_ap_syntax::{AstNode, ast::HasName};

            use super::super::error_codes::ErrorCode;

            let file_id = analyzer
                .local_files
                .iter()
                .copied()
                .find(|file| analyzer.absolute_path(*file).ends_with("lib.rs"))
                .expect("fixture library");
            let tree = analyzer
                .sema
                .parse(ra_ap_hir::EditionedFileId::current_edition(
                    analyzer.db,
                    file_id,
                ));
            let generated = tree
                .syntax()
                .descendants()
                .filter_map(ra_ap_syntax::ast::Fn::cast)
                .find(|function| {
                    function
                        .name()
                        .is_some_and(|name| name.text() == "generated")
                })
                .expect("generated component");
            let expansion = analyzer
                .sema
                .expand_attr_macro(&ra_ap_syntax::ast::Item::Fn(generated))
                .expect("#[tessera] expansion");
            assert!(
                expansion.value.value.to_string().contains("__private"),
                "the exempt expansion must exercise macro-support paths"
            );

            let text = analyzer.db.file_text(file_id).text(analyzer.db);
            let mut reported = analyzer
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == ErrorCode::PrivateApi.into())
                .map(|diagnostic| {
                    assert_eq!(diagnostic.file_id, file_id);
                    (
                        text[diagnostic.range].to_string(),
                        diagnostic.message.clone(),
                    )
                })
                .collect::<Vec<_>>();
            reported.sort();
            let item = |name: &str| {
                format!(
                    "cannot use private Tessera macro-support item `tessera_ui::__private{name}`"
                )
            };
            let mut expected = vec![
                ("GroupGuard".to_string(), item("::GroupGuard")),
                (
                    "__private::current_instance_key".to_string(),
                    item("::current_instance_key"),
                ),
                ("instance_key".to_string(), item("::current_instance_key")),
                ("leak!".to_string(), item("::current_instance_key")),
                ("shim::instance".to_string(), item("::current_instance_key")),
                ("support::Prop".to_string(), item("::Prop")),
                ("tessera_ui::__private".to_string(), item("")),
                (
                    "tessera_ui::__private::GroupGuard".to_string(),
                    item("::GroupGuard"),
                ),
                (
                    "tessera_ui::__private::current_instance_key".to_string(),
                    item("::current_instance_key"),
                ),
                (
                    "tessera_ui::__private::current_instance_key".to_string(),
                    item("::current_instance_key"),
                ),
            ];
            expected.sort();
            assert_eq!(reported, expected);
        });
    }

    #[test]
    fn selected_target_cfg_excludes_inactive_helpers_but_keeps_active_unresolved_calls() {
        let source = r#"
#[tessera_ui::tessera]
fn control() {}

#[cfg(target_family = "wasm")]
mod web {
    fn control() {}
    pub fn helper() { control(); }
}

#[cfg(not(target_family = "wasm"))]
mod native {
    fn control() {}
    pub fn helper() { control(); }
}

#[tessera_ui::tessera]
fn cfg_fragment(value: usize) {
    const DATE_COLUMNS: usize = 7;
    let mut days = [0usize; DATE_COLUMNS];
    for (index, day) in days.iter_mut().enumerate() {
        *day = index;
    }
    let _slot = tessera_ui::RenderSlot::new(move || {
    #[cfg(target_family = "wasm")]
    { web::helper(); }
    #[cfg(not(target_family = "wasm"))]
    { native::helper(); }
    #[cfg(any())]
    { missing::control(); }
    let _ = value;
    });
}

pub fn unresolved() { missing::control(); }
"#;
        for target in [None, Some("wasm32-unknown-unknown")] {
            with_analyzer_options(
                source,
                CheckOptions::new(Some("fixture"), target),
                |analyzer| {
                    let diagnostics = analyzer
                        .diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.message.as_str())
                        .collect::<Vec<_>>();
                    assert_eq!(diagnostics.len(), 1, "{target:?}: {diagnostics:?}");
                    assert!(diagnostics[0].contains("cannot resolve"));
                    assert!(diagnostics[0].contains("missing::control"));
                },
            );
        }
    }

    #[test]
    fn diagnostics_classify_calls_and_preserve_analyzed_source_spans() {
        use super::super::{diagnostics::Reporter, error_codes::ErrorCode};

        let source = "use tessera_ui::remember;\r\n#[tessera_ui::tessera]\r\nfn child() {}\r\nfn helper() {\r\n\tlet \u{754c} = remember(\r\n\t\t|| 1usize,\r\n\t);\r\n\tlet _callback = || child();\r\n\tmissing::remember(|| 0usize);\r\n}\r\n";
        let source = format!(
            "{source}{}",
            r#"
#[tessera_ui::lint(forbid(redundant_render_slot))]
mod guarded {
    #[tessera_ui::lint(deny(redundant_render_slot))]
    pub fn unchanged() {}
    #[tessera_ui::lint(allow(redundant_render_slot))]
    pub fn invalid() {}
}
"#
        );
        with_analyzer(&source, |analyzer| {
            assert_eq!(
                analyzer
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.code)
                    .collect::<Vec<_>>(),
                [
                    ErrorCode::RuntimeCall.into(),
                    ErrorCode::ComponentCall.into(),
                    ErrorCode::UnresolvedCall.into(),
                    ErrorCode::LintConfiguration.into()
                ]
            );
            let diagnostic = &analyzer.diagnostics[0];
            fs::write(analyzer.absolute_path(diagnostic.file_id), "").expect("edit after analysis");
            let reporter = Reporter::new(analyzer, &analyzer.diagnostics);
            let json = reporter.json(diagnostic);
            let span = &json["message"]["spans"][0];
            assert_eq!(json["message"]["code"]["code"], "T0002");
            assert_eq!(
                json["message"]["code"]["explanation"],
                ErrorCode::RuntimeCall.explanation()
            );
            assert_eq!(span["byte_start"], source.find("remember(\r\n").unwrap());
            assert_eq!(span["byte_end"], source.find("\t);").unwrap() + 2);
            assert_eq!(span["line_start"], 5);
            assert_eq!(span["line_end"], 7);
            assert_eq!(span["column_start"], 10);
            assert_eq!(span["column_end"], 3);
            assert_eq!(span["text"].as_array().unwrap().len(), 3);
            assert_eq!(span["text"][0]["text"], "\tlet \u{754c} = remember(");
            assert_eq!(json["message"]["spans"][1]["is_primary"], false);
            assert!(
                json["message"]["spans"][1]["label"]
                    .as_str()
                    .unwrap()
                    .contains("function")
            );
            let closure = reporter.json(&analyzer.diagnostics[1]);
            assert!(
                closure["message"]["spans"][1]["label"]
                    .as_str()
                    .unwrap()
                    .contains("ordinary closure")
            );
            let rendered = reporter.render(diagnostic, false, false);
            assert!(rendered.contains("error[T0002]"), "{rendered}");
            assert!(rendered.contains("let \u{754c} = remember("), "{rendered}");
            let styled = reporter.render(&analyzer.diagnostics[1], false, true);
            assert!(styled.contains("lib.rs:8:21"), "{styled:?}");
            assert!(
                styled.contains(&format!(
                    "{}child()",
                    annotate_snippets::renderer::DEFAULT_ERROR_STYLE
                )),
                "{styled:?}"
            );
            let short = reporter.render(diagnostic, true, false);
            assert!(short.contains("lib.rs:5:10: error[T0002]"), "{short}");
            assert_eq!(short.lines().count(), 1);
            analyzer.diagnostics.clear();
            analyzer
                .check_files(&super::super::LintOptions {
                    overrides: vec![(super::super::LintTarget::All, super::super::Level::Allow)],
                    cap: Some(super::super::Level::Allow),
                })
                .expect("hard errors remain checked");
            assert_eq!(analyzer.diagnostics.len(), 3);
            assert!(analyzer.diagnostics.iter().all(|diagnostic| {
                diagnostic.severity == cargo_metadata::diagnostic::DiagnosticLevel::Error
            }));
        });
    }

    #[test]
    fn persistent_cache_tracks_build_inputs_and_target_cfg() {
        use super::super::{
            TargetSelection,
            cache::{Cache, shape},
            plan::Plan,
        };

        let project = FixtureProject::new(
            r#"
#[schema_macro::select]
pub type Selected = ();
pub struct Ordinary;
impl Ordinary {
    pub fn new(_: impl Fn() + Send + Sync + 'static) -> Self { Self }
}
pub fn helper() { let _ = Selected::new(|| {}); }
"#,
        );
        for directory in ["mini-ui/src", "schema-macro/src", "host-only/src"] {
            fs::create_dir_all(project.root().join(directory)).unwrap();
        }
        fs::write(project.root().join("Cargo.toml"), "[package]\nname='fixture'\nversion='0.0.0'\nedition='2024'\n[workspace]\nmembers=['mini-ui','schema-macro']\nexclude=['host-only']\n[dependencies]\ntessera-ui={path='mini-ui'}\nschema-macro={path='schema-macro'}\n").unwrap();
        fs::write(
            project.root().join(".cargo/config.toml"),
            format!(
                "[build]\ntarget-dir=\"{}\"\n",
                toml_path(&project.root().join("target"))
            ),
        )
        .unwrap();
        fs::write(
            project.root().join("mini-ui/Cargo.toml"),
            "[package]\nname='tessera-ui'\nversion='0.0.0'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(project.root().join("mini-ui/src/lib.rs"), "pub mod prop { pub struct RenderSlot; impl RenderSlot { pub fn new(_: impl Fn() + Send + Sync + 'static) -> Self { Self } } }\npub use prop::RenderSlot;\n").unwrap();
        fs::write(project.root().join("schema-macro/Cargo.toml"), "[package]\nname='schema-macro'\nversion='0.0.0'\nedition='2024'\n[lib]\nproc-macro=true\n[target.'cfg(not(target_family = \"wasm\"))'.dependencies]\nhost-only={path='../host-only'}\n").unwrap();
        fs::write(
            project.root().join("host-only/Cargo.toml"),
            "[package]\nname='host-only'\nversion='0.0.0'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(
            project.root().join("host-only/src/lib.rs"),
            "pub const SLOT: &str = \"tessera_ui::RenderSlot\";\n",
        )
        .unwrap();
        fs::write(project.root().join("schema-macro/src/lib.rs"), r#"
use proc_macro::TokenStream;
#[proc_macro_attribute]
pub fn select(_: TokenStream, _: TokenStream) -> TokenStream {
    let path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/schema.txt");
    let target = if std::fs::read_to_string(path).unwrap() == "slot" { host_only::SLOT } else { "crate::Ordinary" };
    format!("pub type Selected = {target}; const _: &str = include_str!(\"schema.txt\");").parse().unwrap()
}
"#).unwrap();
        let schema = project.root().join("src/schema.txt");
        fs::write(&schema, "ordinary").unwrap();
        fs::create_dir(project.root().join("examples")).unwrap();
        fs::write(project.root().join("examples/plain.rs"), "#[cfg(test)]\nfn inactive() { let _ = tessera_ui::RenderSlot::new(|| {}); }\nfn main() {}\n").unwrap();
        let pass = |target| {
            let mut options = CheckOptions::new(Some("fixture"), target);
            options.target_selection = TargetSelection {
                all_targets: true,
                ..TargetSelection::default()
            };
            let loaded = workspace::load_workspace_from(&options, project.root()).unwrap();
            let db = loaded.host.raw_database();
            ra_ap_hir::attach_db(db, || {
                let plan = Plan::resolve_at(&options, project.root()).unwrap();
                let selected = plan.jobs.iter().find(|job| job.name == "fixture").unwrap();
                let crates =
                    workspace::relevant_crates(db, &loaded.vfs, &selected.target_roots).unwrap();
                let mut cache = Cache::new(&loaded, &plan, &options);
                let index = cache.index(&crates, 1).unwrap();
                let (job, key) = cache.job(selected, 0, &shape(&index), &options).unwrap();
                let cached = cache.diagnostics(&key);
                let reused = cached.is_some();
                let diagnostics = cached.unwrap_or_else(|| {
                    let mut analyzer = CallAnalyzer::from_index(db, &loaded.vfs, job.files, index);
                    analyzer.check_files(&options.lints).unwrap();
                    cache.save_diagnostics(&key, &analyzer.diagnostics);
                    analyzer.diagnostics
                });
                (
                    cache.indexed,
                    reused,
                    diagnostics
                        .into_iter()
                        .map(|diagnostic| diagnostic.code)
                        .collect::<Vec<_>>(),
                )
            })
        };
        let cold = pass(None);
        assert!(cold.0 > 0 && !cold.1 && cold.2.is_empty());
        assert_eq!(pass(None), (0, true, Vec::new()));
        fs::write(schema, "slot").unwrap();
        let changed = pass(None);
        assert!(changed.0 > 0 && !changed.1);
        assert_eq!(changed.2, [super::super::ErrorCode::RuntimeCall.into()]);
        assert_eq!(pass(None), (0, true, changed.2.clone()));
        let cross = pass(Some("wasm32-unknown-unknown"));
        assert_eq!(cross.2, changed.2);
        assert!(cross.0 > 0 && !cross.1);
        assert_eq!(pass(Some("wasm32-unknown-unknown")), (0, true, cross.2));
    }

    #[test]
    fn remembered_states_captured_by_a_model_initializer_are_reported() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{ModelStore, State, remember};

struct Search {
    query: State<String>,
    owned: State<u32>,
}

#[tessera_ui::tessera]
pub fn screen() {
    let query = remember(String::new);
    let unrelated = remember(|| 0u32);
    let _ = unrelated;
    let _model = ModelStore::new().model(move || Search {
        query,
        owned: State::new(0),
    });
}
"#,
        );
        assert_eq!(diagnostics, ["remembered state stored in a model"]);
    }

    #[test]
    fn model_state_is_created_only_by_model_initializers() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{Model, ModelInit, ModelStore, State};

struct Search {
    value: State<u32>,
}

impl ModelInit for Search {
    fn init() -> Self {
        Self { value: State::new(0) }
    }
}

struct NotAModel {
    value: State<u32>,
}

impl Default for NotAModel {
    fn default() -> Self {
        Self { value: State::new(1) }
    }
}

fn helper() -> State<u32> {
    State::new(2)
}

#[tessera_ui::tessera]
pub fn screen(model: Model<Search>) {
    let _ = model;
    let _stored = ModelStore::new().model(|| State::new(4u32));
    let _component = State::new(5u32);
}
"#,
        );
        assert_eq!(
            diagnostics,
            [
                "`State::new` outside a model initializer",
                "`State::new` outside a model initializer",
                "`State::new` outside a model initializer",
            ]
        );
    }

    #[test]
    fn named_modifier_ext_imports_are_reported_with_an_anonymous_import_fix() {
        with_analyzer(
            r#"
use tessera_ui::{Modifier, modifier::{CursorModifierExt as _, FocusModifierExt}};

pub fn focus(modifier: Modifier) -> Modifier {
    modifier.focus_group(None)
}
"#,
            |analyzer| {
                use ra_ap_base_db::SourceDatabase;

                let messages = analyzer
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(
                    messages,
                    ["modifier extension trait `FocusModifierExt` imported by name"]
                );
                let diagnostic = &analyzer.diagnostics[0];
                assert_eq!(
                    diagnostic.code,
                    super::super::error_codes::DiagnosticCode::Lint(
                        super::super::error_codes::Lint::NamedModifierExtImport,
                    )
                );
                let [suggestion] = diagnostic.suggestions.as_slice() else {
                    panic!("expected one suggestion");
                };
                assert_eq!(
                    suggestion.applicability,
                    super::super::types::Applicability::MachineApplicable
                );
                let mut text = analyzer
                    .db
                    .file_text(diagnostic.file_id)
                    .text(analyzer.db)
                    .to_string();
                for edit in suggestion.edits.iter().rev() {
                    text.replace_range(
                        usize::from(edit.range.start())..usize::from(edit.range.end()),
                        &edit.replacement,
                    );
                }
                assert!(
                    text.contains("modifier::{CursorModifierExt as _, FocusModifierExt as _}"),
                    "{text}"
                );
            },
        );
    }

    #[test]
    fn modifier_ext_imports_named_elsewhere_are_not_reported() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{Modifier, modifier::{CursorModifierExt, FocusModifierExt}};

pub fn focus(modifier: Modifier) -> Modifier {
    FocusModifierExt::focus_group(modifier, None)
}

pub fn bounded<T: CursorModifierExt>(modifier: T) -> T {
    modifier
}
"#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn modifier_ext_imports_named_by_child_module_files_are_not_reported() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::modifier::FocusModifierExt;

mod child;
//- src/child.rs
pub fn focus(modifier: tessera_ui::Modifier) -> tessera_ui::Modifier {
    super::FocusModifierExt::focus_group(modifier, None)
}
"#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn reexported_and_foreign_modifier_ext_imports_are_not_reported() {
        let diagnostics = diagnostics_for(
            r#"
pub use tessera_ui::modifier::FocusModifierExt;

mod plain;

mod user {
    use crate::plain::ModifierExt;
}
//- src/plain.rs
pub trait ModifierExt {}
"#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    fn diagnostics_for(source: &str) -> Vec<String> {
        with_analyzer(source, |analyzer| {
            analyzer
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect()
        })
    }

    fn with_analyzer<T>(source: &str, inspect: impl FnOnce(&mut CallAnalyzer<'_>) -> T) -> T {
        with_analyzer_options(source, CheckOptions::new(Some("fixture"), None), inspect)
    }

    fn with_analyzer_options<T>(
        source: &str,
        options: CheckOptions<'_>,
        inspect: impl FnOnce(&mut CallAnalyzer<'_>) -> T,
    ) -> T {
        let project = FixtureProject::new(source);
        let workspace = workspace::load_workspace_from(&options, project.root())
            .expect("fixture workspace should load");
        let db = workspace.host.raw_database();

        ra_ap_hir::attach_db(db, || {
            let plan = super::super::plan::Plan::resolve_at(&options, project.root())
                .expect("fixture plan");
            let job = plan
                .jobs
                .iter()
                .find(|job| job.name == "fixture")
                .expect("fixture job");
            let crates = workspace::relevant_crates(db, &workspace.vfs, &job.target_roots)
                .expect("fixture crates");
            let mut cache = super::super::cache::Cache::new(&workspace, &plan, &options);
            let index = cache.index(&crates, 1).expect("fixture index");
            let files = cache
                .job(job, 0, &super::super::cache::shape(&index), &options)
                .expect("fixture job files")
                .0
                .files;
            let metadata_files = index.inactive_code_ranges.keys().copied().collect();
            let mut analyzer = CallAnalyzer::from_index(db, &workspace.vfs, files, index);
            analyzer.metadata_files = metadata_files;
            analyzer.display_root = project.root().to_path_buf();
            analyzer
                .check_files(&super::LintOptions::default())
                .expect("fixture should analyze");
            inspect(&mut analyzer)
        })
    }

    fn assert_contains(diagnostics: &[String], needle: &str) {
        assert!(
            diagnostics.iter().any(|message| message.contains(needle)),
            "missing diagnostic containing `{needle}`:\n{}",
            diagnostics.join("\n")
        );
    }

    fn count_containing(diagnostics: &[String], needle: &str) -> usize {
        diagnostics
            .iter()
            .filter(|message| message.contains(needle))
            .count()
    }

    struct FixtureProject {
        root: PathBuf,
        _directory: tempfile::TempDir,
    }

    impl FixtureProject {
        fn new(source: &str) -> Self {
            let directory = tempfile::tempdir().expect("isolated checker fixture");
            let root = directory.path().to_path_buf();
            fs::create_dir_all(root.join("src")).expect("fixture src directory should be created");
            fs::create_dir_all(root.join(".cargo"))
                .expect("fixture config directory should be created");
            fs::write(
                root.join(".cargo/config.toml"),
                format!(
                    "[build]\ntarget-dir = \"{}\"\n",
                    toml_path(&workspace_root().join("target/color-check-fixtures")),
                ),
            )
            .expect("fixture target directory should be configured");

            let tessera_ui_path = workspace_root().join("tessera-ui");
            let ui_features =
                if source.contains("HeadlessRenderer") || source.contains("testing::UiTest") {
                    ", features = [\"testing\", \"headless\"]"
                } else {
                    ""
                };
            let tessera_components_path = workspace_root().join("crates/tessera-components");
            let widgets_dependency = if source.contains("use fixture_widgets::") {
                for name in ["widgets", "unrelated"] {
                    fs::create_dir_all(root.join(name).join("src")).expect("dependency sources");
                    fs::write(
                        root.join(name).join("Cargo.toml"),
                        format!("[package]\nname = \"fixture-{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\ntessera-ui = {{ path = \"{}\" }}\n", toml_path(&tessera_ui_path)),
                    ).expect("dependency manifest");
                }
                fs::write(root.join("widgets/src/lib.rs"), "#[tessera_ui::tessera]\npub fn foreign_component(children: Option<tessera_ui::RenderSlot>) {}\nfn invalid_helper() { tessera_ui::remember(|| 1); }\n").expect("dependency component");
                fs::write(
                    root.join("unrelated/src/lib.rs"),
                    "compile_error!(\"unselected workspace member\");\n",
                )
                .expect("unrelated source");
                fs::write(
                    root.join("src/orphan.rs"),
                    "fn orphan() { tessera_ui::remember(|| 1); }\n",
                )
                .expect("orphan source");
                "fixture-widgets = { path = \"widgets\" }\n\n[workspace]\nmembers = [\"widgets\", \"unrelated\"]\n"
            } else {
                ""
            };
            let components_dependency = if source.contains("use tessera_components::") {
                format!(
                    "tessera-components = {{ path = \"{}\" }}\n",
                    toml_path(&tessera_components_path)
                )
            } else {
                String::new()
            };
            fs::write(
                root.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\ntessera-ui = {{ path = \"{}\"{ui_features} }}\n{components_dependency}{widgets_dependency}",
                    toml_path(&tessera_ui_path)
                ),
            )
            .expect("fixture manifest should be written");
            let mut files = source.split("\n//- ");
            fs::write(root.join("src/lib.rs"), files.next().unwrap_or_default())
                .expect("fixture source should be written");
            for file in files {
                let (path, text) = file.split_once('\n').expect("fixture file header");
                fs::write(root.join(path), text).expect("fixture module should be written");
            }
            fs::write(root.join("src/main.rs"), "fn main() {}")
                .expect("fixture binary source should be written");

            Self {
                root,
                _directory: directory,
            }
        }

        fn root(&self) -> &Path {
            &self.root
        }
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("cargo-tessera should live under crates/")
            .to_path_buf()
    }

    fn toml_path(path: &Path) -> String {
        path.display().to_string().replace('\\', "\\\\")
    }
}
