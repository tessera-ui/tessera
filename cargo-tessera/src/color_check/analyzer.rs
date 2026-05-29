use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use ra_ap_base_db::{Crate, SourceDatabase};
use ra_ap_hir::{
    AsAssocItem, CallableKind, EditionedFileId, Function, ModuleDef, PathResolution, Semantics,
    Trait, Type,
};
use ra_ap_ide::{FileId, RootDatabase};
use ra_ap_syntax::{
    AstNode, SyntaxNode, TextSize,
    ast::{self, HasArgList, HasAttrs, HasGenericArgs, HasModuleItem, HasName},
};
use ra_ap_vfs::Vfs;

use super::{
    types::{
        CallKind, CallTarget, ColorAnalyzer, ContextColor, Diagnostic, ResolvedMethod,
        TesseraRuntimeApi,
    },
    utils::{
        expr_path, is_internal_runtime_name, param_name, path_last_segment, path_text,
        runtime_api_label, tessera_crates,
    },
};

const RENDER_SLOT_TYPE_PATH: &str = "tessera_ui::prop::RenderSlot";
const RENDER_SLOT_WITH_TYPE_PATH: &str = "tessera_ui::prop::RenderSlotWith";
const PUBLIC_RENDER_SLOT_TYPE_PATH: &str = "tessera_ui::RenderSlot";
const PUBLIC_RENDER_SLOT_WITH_TYPE_PATH: &str = "tessera_ui::RenderSlotWith";
const ENTRY_POINT_TYPE_PATH: &str = "tessera_ui::entry_point::EntryPoint";
const PUBLIC_ENTRY_POINT_TYPE_PATH: &str = "tessera_ui::EntryPoint";
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

impl<'db> ColorAnalyzer<'db> {
    pub(crate) fn new(
        db: &'db RootDatabase,
        vfs: &'db Vfs,
        selected_package: Option<String>,
        local_files: HashSet<FileId>,
    ) -> Result<ColorAnalyzer<'db>> {
        let sema = Semantics::new(db);
        let tessera_crates = tessera_crates(db);
        let mut metadata_files = HashSet::new();

        for (file_id, path) in vfs.iter() {
            let Some(path) = path.as_path() else {
                continue;
            };
            let std_path: &std::path::Path = path.as_ref();
            if !std_path.extension().is_some_and(|ext| ext == "rs") {
                continue;
            }

            let source_root_id = db.file_source_root(file_id).source_root_id(db);
            let source_root = db.source_root(source_root_id);
            let source_root = source_root.source_root(db);
            let relevant_crates = ra_ap_base_db::relevant_crates(db, file_id);
            let is_tessera_workspace_source = relevant_crates
                .iter()
                .any(|krate| tessera_crates.contains(krate));
            if !source_root.is_library || is_tessera_workspace_source {
                metadata_files.insert(file_id);
            }
        }

        if local_files.is_empty() {
            return Err(anyhow!(
                "rust-analyzer did not load any local Rust source files for Tessera color checking"
            ));
        }

        Ok(ColorAnalyzer {
            db,
            sema,
            tessera_crates,
            vfs,
            local_files,
            metadata_files,
            selected_package,
            tessera_function_names: HashSet::new(),
            tessera_function_paths: HashSet::new(),
            render_slot_setter_indexes_by_component_name: HashMap::new(),
            render_slot_setter_indexes_by_type_path: HashMap::new(),
            diagnostics: Vec::new(),
        })
    }

    pub(crate) fn analyze(&mut self) -> Result<()> {
        let files = self.metadata_files.iter().copied().collect::<Vec<_>>();
        for file_id in files {
            self.collect_file_metadata(file_id)?;
        }

        let files = self.local_files.iter().copied().collect::<Vec<_>>();
        for file_id in files {
            self.analyze_file(file_id)?;
        }
        Ok(())
    }

    fn collect_file_metadata(&mut self, file_id: FileId) -> Result<()> {
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

    fn analyze_file(&mut self, file_id: FileId) -> Result<()> {
        let tree = self
            .sema
            .parse(EditionedFileId::current_edition(self.db, file_id));

        for item in tree.items() {
            self.analyze_item(item);
        }
        Ok(())
    }

    fn collect_tessera_function_names_from_item(&mut self, item: &ast::Item) {
        match item {
            ast::Item::Fn(function) if self.is_tessera_function(function) => {
                if let Some(name) = function.name() {
                    self.tessera_function_names.insert(name.text().to_string());
                }
                if let Some(def) = self.sema.to_fn_def(function) {
                    let path = self.function_path(def);
                    self.tessera_function_paths.insert(path);
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
        match item {
            ast::Item::Fn(function) => self.analyze_function(function, false),
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
        let Some(function_name) = function.name().map(|name| name.text().to_string()) else {
            return;
        };
        let type_path = self.generated_builder_type_path(function);
        let Some(param_list) = function.param_list() else {
            return;
        };

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

            let mut indexes = HashSet::new();
            indexes.insert(0);
            let component_setters = self
                .render_slot_setter_indexes_by_component_name
                .entry(function_name.clone())
                .or_default();
            component_setters.insert(name.clone(), indexes.clone());
            component_setters.insert(format!("{name}_shared"), indexes.clone());

            let Some(type_path) = &type_path else {
                continue;
            };
            let setters = self
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
            let Some(name) = function.name() else {
                continue;
            };
            let slot_indexes = self.render_slot_carrier_param_indexes(&function);
            if !slot_indexes.is_empty() {
                self.render_slot_setter_indexes_by_type_path
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
        let function = self.sema.to_fn_def(function)?;
        self.generated_builder_type_path_for_function(function)
    }

    fn generated_builder_type_path_for_function(&self, function: Function) -> Option<String> {
        let function_path = self.function_path(function);
        let (module_path, function_name) = function_path.rsplit_once("::")?;
        Some(format!(
            "{module_path}::{}",
            Self::generated_builder_type_name(function_name)
        ))
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

    fn render_slot_carrier_param_indexes(&self, function: &ast::Fn) -> HashSet<usize> {
        let Some(param_list) = function.param_list() else {
            return HashSet::new();
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
        self.sema
            .resolve_type(&ty)
            .is_some_and(|ty| self.semantic_type_is_render_slot_handle(&ty))
            || Self::type_syntax_is_render_slot_handle(&ty)
    }

    fn type_is_optional_render_slot_handle(&self, ty: ast::Type) -> bool {
        self.sema
            .resolve_type(&ty)
            .is_some_and(|ty| self.semantic_type_is_option_of_render_slot_handle(&ty))
            || Self::type_syntax_is_option_of_render_slot_handle(&ty)
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
        self.sema
            .resolve_type(&ty)
            .is_some_and(|ty| self.semantic_type_is_direct_render_slot_handle(&ty))
            || Self::type_syntax_is_render_slot_handle(&ty)
    }

    fn type_syntax_is_render_slot_handle(ty: &ast::Type) -> bool {
        let text = ty
            .syntax()
            .text()
            .to_string()
            .replace(char::is_whitespace, "");
        Self::type_text_is_render_slot_handle(&text)
    }

    fn type_syntax_is_option_of_render_slot_handle(ty: &ast::Type) -> bool {
        let text = ty
            .syntax()
            .text()
            .to_string()
            .replace(char::is_whitespace, "");
        let Some(inner) = Self::option_type_inner_text(&text) else {
            return false;
        };
        Self::type_text_is_render_slot_handle(inner)
    }

    fn option_type_inner_text(text: &str) -> Option<&str> {
        let (base, inner) = text.split_once('<')?;
        if base.rsplit("::").next()? != "Option" || !inner.ends_with('>') {
            return None;
        }
        Some(&inner[..inner.len() - 1])
    }

    fn type_text_is_render_slot_handle(text: &str) -> bool {
        let base = text.split_once('<').map_or(text, |(base, _)| base);
        matches!(
            base.rsplit("::").next(),
            Some("RenderSlot" | "RenderSlotWith")
        )
    }

    fn analyze_assoc_item(&mut self, item: ast::AssocItem) {
        if let ast::AssocItem::Fn(function) = item {
            self.analyze_function(function, false);
        }
    }

    fn analyze_function(&mut self, function: ast::Fn, allow_runtime_body: bool) {
        let Some(body) = function.body() else {
            return;
        };
        let allow_entry_point_new = self.is_entry_function(&function) || allow_runtime_body;
        let color = if self.is_colored_function(&function) || allow_runtime_body {
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
        if let Some(call) = ast::CallExpr::cast(body.syntax().clone()) {
            self.analyze_call(call, color, false);
        } else if let Some(call) = ast::MethodCallExpr::cast(body.syntax().clone()) {
            self.analyze_method_call(call, color);
        }
        self.walk_node(body.syntax(), color, false);
    }

    fn walk_node(&mut self, node: &SyntaxNode, color: ContextColor, allow_entry_point_new: bool) {
        for child in node.children() {
            if let Some(function) = ast::Fn::cast(child.clone()) {
                self.analyze_function(function, false);
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
        match self.resolve_method_call(&call) {
            Some(ResolvedMethod::RenderSlotNew) => {
                if color == ContextColor::Plain {
                    self.push_forbidden_call(
                        call.syntax(),
                        RENDER_SLOT_NEW_PATH,
                        CallKind::RuntimeApi(TesseraRuntimeApi::RenderSlotNew),
                    );
                }
            }
            Some(ResolvedMethod::RenderSlotWithNew) => {
                if color == ContextColor::Plain {
                    self.push_forbidden_call(
                        call.syntax(),
                        RENDER_SLOT_WITH_NEW_PATH,
                        CallKind::RuntimeApi(TesseraRuntimeApi::RenderSlotWithNew),
                    );
                }
            }
            Some(ResolvedMethod::Other) => {}
            None => {
                if color == ContextColor::Plain {
                    let path = self.method_call_path(&call);
                    if self.is_semantically_relevant_unresolved_method(&path) {
                        self.push_unresolved_call(call.syntax(), &path);
                    }
                }
            }
        }
    }

    fn resolve_call_target(&self, call: &ast::CallExpr) -> Option<CallTarget> {
        let expr = call.expr()?;
        if ast::ClosureExpr::can_cast(expr.syntax().kind()) {
            return None;
        }

        if let Some(callable) = self.sema.resolve_expr_as_callable(&expr) {
            let CallableKind::Function(function) = callable.kind() else {
                return None;
            };
            let path = self.function_path(function);
            if self.is_tessera_function_def(function) {
                return Some(CallTarget::Resolved {
                    path,
                    kind: CallKind::TesseraFunction,
                });
            }
            if let Some(api) = self.runtime_api_for_function(function, &path) {
                return Some(CallTarget::Resolved {
                    path,
                    kind: CallKind::RuntimeApi(api),
                });
            }
            return None;
        }

        if let Some(path) = expr_path(&expr) {
            if let Some(resolution) = self.sema.resolve_path(&path) {
                if let Some(target) = self.call_target_for_path_resolution(resolution) {
                    return Some(target);
                }
                let path = path_text(&path);
                if Self::is_render_slot_constructor_path(&path)
                    || Self::is_entry_point_constructor_path(&path)
                {
                    return Some(CallTarget::Unresolved { path });
                }
                return None;
            }
            let path = path_text(&path);
            if self.is_semantically_relevant_unresolved_path(&path) {
                return Some(CallTarget::Unresolved { path });
            }
        }

        None
    }

    fn call_target_for_path_resolution(&self, resolution: PathResolution) -> Option<CallTarget> {
        match resolution {
            PathResolution::Def(ModuleDef::Function(function)) => {
                let path = self.function_path(function);
                if self.is_tessera_function_def(function) {
                    Some(CallTarget::Resolved {
                        path,
                        kind: CallKind::TesseraFunction,
                    })
                } else {
                    self.runtime_api_for_function(function, &path)
                        .map(|api| CallTarget::Resolved {
                            path,
                            kind: CallKind::RuntimeApi(api),
                        })
                }
            }
            _ => None,
        }
    }

    fn resolve_method_call(&self, call: &ast::MethodCallExpr) -> Option<ResolvedMethod> {
        let function = self.sema.resolve_method_call(call)?;
        let path = self.function_path(function);
        match self.runtime_api_for_function(function, &path) {
            Some(TesseraRuntimeApi::RenderSlotNew) => Some(ResolvedMethod::RenderSlotNew),
            Some(TesseraRuntimeApi::RenderSlotWithNew) => Some(ResolvedMethod::RenderSlotWithNew),
            _ => Some(ResolvedMethod::Other),
        }
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

    fn is_free_function(&self, function: &ast::Fn) -> bool {
        function.syntax().parent().is_some_and(|parent| {
            ast::SourceFile::can_cast(parent.kind()) || ast::ItemList::can_cast(parent.kind())
        })
    }

    fn is_colored_function(&self, function: &ast::Fn) -> bool {
        self.is_tessera_function(function)
            || self.is_runtime_api_function(function)
            || self.is_test_function(function)
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
        if self.tessera_function_paths.contains(&path) {
            return true;
        }
        if self.package_matches(function)
            && self
                .tessera_function_names
                .contains(function.name(self.db).as_str())
        {
            return true;
        }

        let Some(source) = self.sema.source(function) else {
            return false;
        };
        let Some(file_id) = source
            .file_id
            .file_id()
            .map(|file_id| file_id.file_id(self.db))
        else {
            return false;
        };
        if !self.local_files.contains(&file_id) {
            return false;
        }
        if !self.package_matches(function) {
            return false;
        }

        source
            .value
            .attrs()
            .any(|attr| self.is_tessera_attribute(&attr))
    }

    fn is_runtime_api_function(&self, function: &ast::Fn) -> bool {
        let Some(def) = self.sema.to_fn_def(function) else {
            return false;
        };

        if !self.is_tessera_crate(def.module(self.db).krate(self.db).into()) {
            return false;
        }

        let path = self.function_path(def);
        let name = def.name(self.db).as_str().to_string();
        match name.as_str() {
            "remember" if path.ends_with("::remember") => true,
            "remember_with_key" if path.ends_with("::remember_with_key") => true,
            "retain" if path.ends_with("::retain") => true,
            "retain_with_key" if path.ends_with("::retain_with_key") => true,
            "provide_context" if path.ends_with("::provide_context") => true,
            "use_context" if path.ends_with("::use_context") => true,
            "receive_frame_nanos" if path.ends_with("::receive_frame_nanos") => true,
            "key" if path.ends_with("::key") => true,
            "new" if self.is_render_slot_method(def, RENDER_SLOT_TYPE_PATH) => true,
            "new" if self.is_render_slot_method(def, RENDER_SLOT_WITH_TYPE_PATH) => true,
            name => is_internal_runtime_name(name),
        }
    }

    fn is_test_function(&self, function: &ast::Fn) -> bool {
        function.attrs().any(|attr| {
            attr.path()
                .and_then(|path| path_last_segment(&path))
                .is_some_and(|name| name == "test")
        })
    }

    fn package_matches(&self, function: Function) -> bool {
        let Some(selected_package) = self.selected_package.as_deref() else {
            return true;
        };
        function
            .module(self.db)
            .krate(self.db)
            .display_name(self.db)
            .is_some_and(|name| name.canonical_name().as_str() == selected_package)
    }

    fn is_tessera_attribute(&self, attr: &ast::Attr) -> bool {
        let Some(path) = attr.path() else {
            return false;
        };
        if path_last_segment(&path).is_some_and(|name| matches!(name.as_str(), "tessera" | "shard"))
        {
            return true;
        }

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
        if path_last_segment(&path).is_some_and(|name| name == "entry") {
            return true;
        }

        if let Some(PathResolution::Def(ModuleDef::Macro(mac))) = self.sema.resolve_path(&path) {
            return mac.name(self.db).as_str() == "entry"
                && self.is_tessera_crate(mac.module(self.db).krate(self.db).into());
        }

        false
    }

    fn runtime_api_for_function(
        &self,
        function: Function,
        canonical_path: &str,
    ) -> Option<TesseraRuntimeApi> {
        if self.is_runtime_api_definition(function) {
            return None;
        }

        if !self.is_tessera_crate(function.module(self.db).krate(self.db).into()) {
            return None;
        }

        let name = function.name(self.db).as_str().to_string();
        match name.as_str() {
            "remember" if canonical_path.ends_with("::remember") => {
                Some(TesseraRuntimeApi::Remember)
            }
            "remember_with_key" if canonical_path.ends_with("::remember_with_key") => {
                Some(TesseraRuntimeApi::RememberWithKey)
            }
            "retain" if canonical_path.ends_with("::retain") => Some(TesseraRuntimeApi::Retain),
            "retain_with_key" if canonical_path.ends_with("::retain_with_key") => {
                Some(TesseraRuntimeApi::RetainWithKey)
            }
            "provide_context" if canonical_path.ends_with("::provide_context") => {
                Some(TesseraRuntimeApi::ProvideContext)
            }
            "use_context" if canonical_path.ends_with("::use_context") => {
                Some(TesseraRuntimeApi::UseContext)
            }
            "receive_frame_nanos" if canonical_path.ends_with("::receive_frame_nanos") => {
                Some(TesseraRuntimeApi::ReceiveFrameNanos)
            }
            "key" if canonical_path.ends_with("::key") => Some(TesseraRuntimeApi::Key),
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
            name if is_internal_runtime_name(name) => Some(TesseraRuntimeApi::InternalRuntime),
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
        self.tessera_crates.contains(&krate)
    }

    fn type_adt_canonical_path(&self, ty: &Type<'db>) -> Option<String> {
        let mut ty = ty.clone();
        while let Some((inner, _)) = ty.as_reference() {
            ty = inner;
        }

        let adt = ty.as_adt()?;
        self.module_def_canonical_path(ModuleDef::from(adt))
    }

    fn semantic_type_is_render_slot_handle(&self, ty: &Type<'db>) -> bool {
        self.semantic_type_is_direct_render_slot_handle(ty)
            || self.semantic_type_is_option_of_render_slot_handle(ty)
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
        let module = adt.module(self.db);
        let Some(type_path) =
            ModuleDef::from(adt).canonical_path(self.db, module.krate(self.db).edition(self.db))
        else {
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
            if ast::ClosureExpr::can_cast(node.kind()) || ast::BlockExpr::can_cast(node.kind()) {
                return false;
            }
            if let Some(call) = ast::CallExpr::cast(node.clone()) {
                let Some(index) = call.arg_list().and_then(|arg_list| {
                    arg_list
                        .args()
                        .position(|arg| arg.syntax().text_range().contains_range(closure_range))
                }) else {
                    return false;
                };
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
                    || self.method_call_takes_render_slot_closure_at(&call, index);
            }
            current = node.parent();
        }

        false
    }

    fn call_takes_tessera_carrier_closure_at(&self, call: &ast::CallExpr, index: usize) -> bool {
        if index != 0 {
            return false;
        }

        matches!(
            self.resolve_call_target(call),
            Some(CallTarget::Resolved {
                kind: CallKind::RuntimeApi(
                    TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
                ),
                ..
            })
        ) || self.call_syntax_is_render_slot_constructor(call)
    }

    fn call_syntax_is_render_slot_constructor(&self, call: &ast::CallExpr) -> bool {
        call.expr()
            .and_then(|expr| expr_path(&expr))
            .is_some_and(|path| Self::is_render_slot_constructor_path(&path_text(&path)))
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
        let Some(function) = self.sema.resolve_method_call(call) else {
            return false;
        };
        let Some(source) = self.sema.source(function) else {
            return false;
        };
        let Some(param_list) = source.value.param_list() else {
            return false;
        };
        param_list
            .params()
            .nth(index)
            .is_some_and(|param| self.param_is_render_slot_carrier(&param))
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

        if call
            .receiver()
            .and_then(|receiver| self.receiver_generated_builder_root_function(&receiver))
            .is_some_and(|function| {
                self.component_generated_setter_accepts_render_slot(function, name.as_str(), index)
            })
        {
            return true;
        }

        if call
            .receiver()
            .and_then(|receiver| self.receiver_generated_builder_root_name(&receiver))
            .is_some_and(|function_name| {
                self.component_name_setter_accepts_render_slot(&function_name, name.as_str(), index)
            })
        {
            return true;
        }

        let Some(receiver_type_path) = call
            .receiver()
            .and_then(|receiver| self.receiver_type_key(&receiver))
            .or_else(|| self.method_call_implementing_type_key(call))
        else {
            return false;
        };

        self.setter_accepts_render_slot_at(&receiver_type_path, name.as_str(), index)
    }

    fn setter_accepts_render_slot_at(
        &self,
        receiver_type_path: &str,
        name: &str,
        index: usize,
    ) -> bool {
        self.render_slot_setter_indexes_by_type_path
            .get(receiver_type_path)
            .and_then(|setters| setters.get(name))
            .is_some_and(|indexes| indexes.contains(&index))
    }

    fn component_name_setter_accepts_render_slot(
        &self,
        function_name: &str,
        name: &str,
        index: usize,
    ) -> bool {
        self.render_slot_setter_indexes_by_component_name
            .get(function_name)
            .and_then(|setters| setters.get(name))
            .is_some_and(|indexes| indexes.contains(&index))
    }

    fn receiver_generated_builder_root_function(&self, receiver: &ast::Expr) -> Option<Function> {
        if let Some(call) = ast::CallExpr::cast(receiver.syntax().clone()) {
            return self.call_generated_builder_root_function(&call);
        }

        if let Some(call) = ast::MethodCallExpr::cast(receiver.syntax().clone()) {
            return call
                .receiver()
                .and_then(|receiver| self.receiver_generated_builder_root_function(&receiver));
        }

        None
    }

    fn call_generated_builder_root_function(&self, call: &ast::CallExpr) -> Option<Function> {
        let expr = call.expr()?;
        let callable = self.sema.resolve_expr_as_callable(&expr)?;
        let CallableKind::Function(function) = callable.kind() else {
            return None;
        };
        Some(function)
    }

    fn receiver_generated_builder_root_name(&self, receiver: &ast::Expr) -> Option<String> {
        if let Some(call) = ast::CallExpr::cast(receiver.syntax().clone()) {
            return self.call_generated_builder_root_name(&call);
        }

        if let Some(call) = ast::MethodCallExpr::cast(receiver.syntax().clone()) {
            return call
                .receiver()
                .and_then(|receiver| self.receiver_generated_builder_root_name(&receiver));
        }

        None
    }

    fn call_generated_builder_root_name(&self, call: &ast::CallExpr) -> Option<String> {
        let expr = call.expr()?;
        let path = expr_path(&expr)?;
        path.segments()
            .last()
            .and_then(|segment| segment.name_ref())
            .map(|name| name.text().to_string())
    }

    fn component_generated_setter_accepts_render_slot(
        &self,
        function: Function,
        method_name: &str,
        index: usize,
    ) -> bool {
        if index != 0 {
            return false;
        }
        let Some(source) = self.sema.source(function) else {
            return false;
        };
        if !self.is_tessera_function(&source.value) {
            return false;
        }
        let Some(param_list) = source.value.param_list() else {
            return false;
        };

        param_list.params().any(|param| {
            if self.param_skips_setter(&param) {
                return false;
            }
            let Some(name) = param_name(&param) else {
                return false;
            };
            if method_name != name && method_name != format!("{name}_shared") {
                return false;
            }
            let Some(ty) = param.ty() else {
                return false;
            };
            self.type_is_optional_render_slot_handle(ty)
        })
    }

    fn receiver_type_key(&self, receiver: &ast::Expr) -> Option<String> {
        let receiver_type = self.sema.type_of_expr(receiver).map(|ty| ty.adjusted())?;
        self.type_adt_canonical_path(&receiver_type)
    }

    fn method_call_implementing_type_key(&self, call: &ast::MethodCallExpr) -> Option<String> {
        let function = self.sema.resolve_method_call(call)?;
        let assoc_item = function.as_assoc_item(self.db)?;
        assoc_item
            .implementing_ty(self.db)
            .and_then(|ty| self.type_adt_canonical_path(&ty))
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
                | "receive_frame_nanos"
                | "key"
        ) || self.tessera_function_names.contains(last)
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
        let message = match kind {
            CallKind::TesseraFunction => {
                format!("uncolored context calls Tessera component `{target}`")
            }
            CallKind::RuntimeApi(api) => {
                format!(
                    "uncolored context calls Tessera-only API `{target}` ({})",
                    runtime_api_label(api)
                )
            }
        };
        self.push_diagnostic(syntax, message);
    }

    fn push_unresolved_call(&mut self, syntax: &SyntaxNode, target: &str) {
        self.push_diagnostic(
            syntax,
            format!(
                "uncolored context contains semantically unresolved Tessera-sensitive call `{target}`"
            ),
        );
    }

    fn push_diagnostic(&mut self, syntax: &SyntaxNode, message: String) {
        let file_range = self.sema.original_range(syntax);
        let file_id = file_range.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id) {
            return;
        }
        self.diagnostics.push(Diagnostic {
            file_id,
            range: file_range.range,
            message,
        });
    }

    pub(crate) fn emit_diagnostic(&self, diagnostic: &Diagnostic) {
        let path = self.display_path(diagnostic.file_id);
        let location = self.location(diagnostic.file_id, diagnostic.range.start());
        eprintln!("  {path}:{location}: {}", diagnostic.message);
    }

    fn display_path(&self, file_id: FileId) -> String {
        self.vfs
            .file_path(file_id)
            .as_path()
            .map(|path| {
                let std_path: &std::path::Path = path.as_ref();
                let path = PathBuf::from(std_path);
                env::current_dir()
                    .ok()
                    .and_then(|cwd| path.strip_prefix(cwd).ok().map(PathBuf::from))
                    .unwrap_or(path)
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| self.vfs.file_path(file_id).to_string())
    }

    fn location(&self, file_id: FileId, offset: TextSize) -> String {
        let Some(path) = self.vfs.file_path(file_id).as_path() else {
            return "?:?".to_string();
        };
        let std_path: &std::path::Path = path.as_ref();
        let Ok(text) = std::fs::read_to_string(std_path) else {
            return "?:?".to_string();
        };
        let mut line = 1usize;
        let mut column = 1usize;
        let target = u32::from(offset) as usize;
        for (index, byte) in text.as_bytes().iter().enumerate() {
            if index >= target {
                break;
            }
            if *byte == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        format!("{line}:{column}")
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        sync::{
            Mutex, OnceLock,
            atomic::{AtomicU64, Ordering},
        },
    };

    use super::super::{CheckOptions, types::ColorAnalyzer, workspace};

    static CURRENT_DIR_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn direct_slot_carriers_remain_colored() {
        let diagnostics = diagnostics_for(
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
pub fn root() {
    let _slot = RenderSlot::new(|| {
        child();
        let _state = remember(|| 1usize);
    });
    let _slot_with = RenderSlotWith::<u32>::new(|_| child());
    host().slot(|| child());
}

#[tessera_ui::entry]
pub fn run() -> EntryPoint {
    EntryPoint::new(root)
}
"#,
        );

        assert!(
            diagnostics.is_empty(),
            "unexpected diagnostics:\n{}",
            diagnostics.join("\n")
        );
    }

    #[test]
    fn ordinary_closures_and_indirect_wrappers_stay_plain() {
        let diagnostics = diagnostics_for(
            r#"
use tessera_ui::{EntryPoint, RenderSlot, remember};

#[tessera_ui::tessera]
fn child() {}

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
    wrapper(|| child());
}

pub fn plain_entry() -> EntryPoint {
    EntryPoint::new(root)
}
"#,
        );

        assert_contains(&diagnostics, "RenderSlot::new");
        assert_contains(&diagnostics, "EntryPoint::new");
        assert_contains(&diagnostics, "remember");
        assert!(
            count_containing(&diagnostics, "Tessera component") >= 3,
            "expected ordinary closures to report Tessera component calls:\n{}",
            diagnostics.join("\n")
        );
    }

    fn diagnostics_for(source: &str) -> Vec<String> {
        let _lock = CURRENT_DIR_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("current directory lock poisoned");
        let project = FixtureProject::new(source);
        let _current_dir = CurrentDirGuard::push(project.root());
        let options = CheckOptions::new(Some("fixture"), None);
        let workspace = workspace::load_workspace(&options).expect("fixture workspace should load");
        let db = workspace.host.raw_database();

        ra_ap_hir::attach_db(db, || {
            let local_files = workspace::selected_local_files(db, &workspace.vfs, &options)
                .expect("fixture files should be selected");
            let mut analyzer =
                ColorAnalyzer::new(db, &workspace.vfs, Some("fixture".to_string()), local_files)
                    .expect("fixture analyzer should initialize");
            analyzer.analyze().expect("fixture should analyze");
            analyzer
                .diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.message)
                .collect()
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
    }

    impl FixtureProject {
        fn new(source: &str) -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "tessera-color-check-fixture-{}-{id}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("src")).expect("fixture src directory should be created");

            let tessera_ui_path = workspace_root().join("tessera-ui");
            fs::write(
                root.join("Cargo.toml"),
                format!(
                    "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\ntessera-ui = {{ path = \"{}\" }}\n",
                    toml_path(&tessera_ui_path)
                ),
            )
            .expect("fixture manifest should be written");
            fs::write(root.join("src/lib.rs"), source).expect("fixture source should be written");

            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for FixtureProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn push(path: &Path) -> Self {
            let previous = env::current_dir().expect("current directory should resolve");
            env::set_current_dir(path).expect("current directory should switch to fixture");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.previous);
        }
    }

    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-tessera should live in the workspace root")
            .to_path_buf()
    }

    fn toml_path(path: &Path) -> String {
        path.display().to_string().replace('\\', "\\\\")
    }
}
