use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use ra_ap_base_db::{Crate, SourceDatabase};
use ra_ap_hir::{
    AsAssocItem, CallableKind, DisplayTarget, EditionedFileId, Function, HirDisplay, ModuleDef,
    PathResolution, Semantics,
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
        expr_path, is_internal_runtime_name, last_type_segment_name, param_name, path_last_segment,
        path_text, runtime_api_label, tessera_crates,
    },
};

impl<'db> ColorAnalyzer<'db> {
    pub(crate) fn new(
        db: &'db RootDatabase,
        vfs: &'db Vfs,
        selected_package: Option<String>,
    ) -> Result<ColorAnalyzer<'db>> {
        let sema = Semantics::new(db);
        let tessera_crates = tessera_crates(db);
        let local_files = vfs
            .iter()
            .filter_map(|(file_id, path)| {
                let path = path.as_path()?;
                let std_path: &std::path::Path = path.as_ref();
                if std_path.extension().is_some_and(|ext| ext == "rs")
                    && !db
                        .source_root(db.file_source_root(file_id).source_root_id(db))
                        .source_root(db)
                        .is_library
                {
                    Some(file_id)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

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
            selected_package,
            tessera_function_names: HashSet::new(),
            explicit_slot_setter_indexes: HashMap::new(),
            diagnostics: Vec::new(),
        })
    }

    pub(crate) fn analyze(&mut self) -> Result<()> {
        let files = self.local_files.iter().copied().collect::<Vec<_>>();
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
            ast::Item::Fn(function) => {
                if self.is_tessera_function(function)
                    && let Some(name) = function.name()
                {
                    self.tessera_function_names.insert(name.text().to_string());
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

    fn collect_explicit_slot_setters_from_impl(&mut self, item_impl: &ast::Impl) {
        let Some(self_ty) = item_impl.self_ty() else {
            return;
        };
        let Some(type_name) = self.type_last_segment_name(&self_ty) else {
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
                self.explicit_slot_setter_indexes
                    .entry(type_name.clone())
                    .or_default()
                    .insert(name.text().to_string(), slot_indexes);
            }
        }
    }
    fn type_last_segment_name(&self, ty: &ast::Type) -> Option<String> {
        let ast::Type::PathType(path_ty) = ty else {
            return None;
        };
        path_ty
            .path()
            .and_then(|path| path_last_segment(&path))
            .map(|name| name.to_string())
    }
    fn render_slot_carrier_param_indexes(&self, function: &ast::Fn) -> HashSet<usize> {
        let Some(param_list) = function.param_list() else {
            return HashSet::new();
        };
        param_list
            .params()
            .enumerate()
            .filter(|(_, param)| self.type_is_render_slot_handle(param.ty()))
            .map(|(index, _)| index)
            .collect()
    }

    fn type_is_render_slot_handle(&self, ty: Option<ast::Type>) -> bool {
        let Some(ty) = ty else {
            return false;
        };
        match ty {
            ast::Type::PathType(path_ty) => path_ty
                .path()
                .is_some_and(|path| self.path_is_render_slot_handle(&path)),
            ast::Type::ParenType(ty) => self.type_is_render_slot_handle(ty.ty()),
            _ => false,
        }
    }

    fn path_is_render_slot_handle(&self, path: &ast::Path) -> bool {
        let Some(segment) = path.segment() else {
            return false;
        };
        let Some(name) = segment.name_ref() else {
            return false;
        };
        match name.text().as_str() {
            "RenderSlot" | "RenderSlotWith" => true,
            "Option" => segment.generic_arg_list().is_some_and(|args| {
                args.generic_args().any(|arg| {
                    let ast::GenericArg::TypeArg(arg) = arg else {
                        return false;
                    };
                    self.type_is_render_slot_handle(arg.ty())
                })
            }),
            _ => false,
        }
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
        let color = if self.is_colored_function(&function) || allow_runtime_body {
            ContextColor::Tessera
        } else {
            ContextColor::Plain
        };
        self.walk_node(body.syntax(), color);
    }

    fn analyze_closure(&mut self, closure: ast::ClosureExpr, parent_color: ContextColor) {
        let Some(body) = closure.body() else {
            return;
        };
        let color = if self.is_render_slot_carrier_closure(&closure) {
            ContextColor::Tessera
        } else {
            parent_color
        };
        self.walk_node(body.syntax(), color);
    }

    fn walk_node(&mut self, node: &SyntaxNode, color: ContextColor) {
        for child in node.children() {
            if let Some(function) = ast::Fn::cast(child.clone()) {
                self.analyze_function(function, false);
                continue;
            }
            if let Some(closure) = ast::ClosureExpr::cast(child.clone()) {
                self.analyze_closure(closure, color);
                continue;
            }
            if let Some(call) = ast::CallExpr::cast(child.clone()) {
                self.analyze_call(call, color);
                self.walk_node(&child, color);
                continue;
            }
            if let Some(call) = ast::MethodCallExpr::cast(child.clone()) {
                self.analyze_method_call(call, color);
                self.walk_node(&child, color);
                continue;
            }
            self.walk_node(&child, color);
        }
    }

    fn analyze_call(&mut self, call: ast::CallExpr, color: ContextColor) {
        let Some(target) = self.resolve_call_target(&call) else {
            return;
        };
        let is_carrier = matches!(
            &target,
            CallTarget::Resolved {
                kind: CallKind::RuntimeApi(
                    TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
                ),
                ..
            }
        );

        match target {
            CallTarget::Resolved { path, kind } => {
                if color == ContextColor::Plain && !is_carrier {
                    self.push_forbidden_call(call.syntax(), &path, kind);
                }
            }
            CallTarget::Unresolved { path } => {
                if color == ContextColor::Plain {
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
                        "tessera_ui::renderer::RenderSlot::new",
                        CallKind::RuntimeApi(TesseraRuntimeApi::RenderSlotNew),
                    );
                }
            }
            Some(ResolvedMethod::RenderSlotWithNew) => {
                if color == ContextColor::Plain {
                    self.push_forbidden_call(
                        call.syntax(),
                        "tessera_ui::renderer::RenderSlotWith::new",
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

    fn is_tessera_function_def(&self, function: Function) -> bool {
        if function.as_assoc_item(self.db).is_some() {
            return false;
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
            "new" if self.is_render_slot_method(def, "RenderSlot") => true,
            "new" if self.is_render_slot_method(def, "RenderSlotWith") => true,
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
            "new" if self.is_render_slot_method(function, "RenderSlot") => {
                Some(TesseraRuntimeApi::RenderSlotNew)
            }
            "new" if self.is_render_slot_method(function, "RenderSlotWith") => {
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

    fn is_render_slot_method(&self, function: Function, type_name: &str) -> bool {
        let Some(assoc_item) = function.as_assoc_item(self.db) else {
            return false;
        };
        assoc_item.implementing_ty(self.db).is_some_and(|ty| {
            let display_target =
                DisplayTarget::from_crate(self.db, function.module(self.db).krate(self.db).into());
            ty.display(self.db, display_target)
                .to_string()
                .starts_with(type_name)
        })
    }

    fn is_tessera_crate(&self, krate: Crate) -> bool {
        self.tessera_crates.contains(&krate)
    }

    fn is_render_slot_carrier_closure(&self, closure: &ast::ClosureExpr) -> bool {
        self.is_direct_render_slot_carrier_closure(closure)
    }

    fn is_direct_render_slot_carrier_closure(&self, closure: &ast::ClosureExpr) -> bool {
        let Some((arg_list, index)) = self.closure_arg_list_and_index(closure) else {
            return false;
        };
        let Some(call_node) = arg_list.syntax().parent() else {
            return false;
        };

        if let Some(call) = ast::CallExpr::cast(call_node.clone()) {
            return self.call_is_render_slot_carrier(&call);
        }

        if let Some(call) = ast::MethodCallExpr::cast(call_node) {
            return self.method_call_takes_render_slot_setter_at(&call, index)
                || self.method_call_takes_render_slot_closure_at(&call, index);
        }

        false
    }
    fn closure_arg_list_and_index(
        &self,
        closure: &ast::ClosureExpr,
    ) -> Option<(ast::ArgList, usize)> {
        let parent = closure.syntax().parent()?;
        let arg_list = ast::ArgList::cast(parent)?;
        let index = arg_list
            .args()
            .position(|arg| arg.syntax() == closure.syntax())?;
        Some((arg_list, index))
    }

    fn call_is_render_slot_carrier(&self, call: &ast::CallExpr) -> bool {
        matches!(
            self.resolve_call_target(call),
            Some(CallTarget::Resolved {
                kind: CallKind::RuntimeApi(
                    TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
                ),
                ..
            })
        )
    }

    fn method_call_takes_render_slot_closure_at(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        let Some(callable) = self.sema.resolve_method_call_as_callable(call) else {
            return false;
        };
        if callable.params().get(index).is_some_and(|param| {
            self.type_is_render_slot_closure_param(param.ty())
                || self.type_is_into_render_slot_param(param.ty())
        }) {
            return true;
        }

        // For builder setters with \`impl Fn()\` parameters, resolve the method
        // definition and check whether the parameter flows into RenderSlot::new
        // or RenderSlotWith::new by analyzing the function body.
        self.method_body_passes_param_to_render_slot(call, index)
    }

    fn type_is_render_slot_closure_param(&self, ty: &ra_ap_hir::Type<'db>) -> bool {
        let Some(callable) = ty.as_callable(self.db) else {
            return false;
        };
        matches!(callable.kind(), CallableKind::FnImpl(_))
            && callable.n_params() == 0
            && self.type_is_unit_return(&callable.return_type())
            && self.type_display_mentions_render_slot(ty)
    }

    fn type_display_mentions_render_slot(&self, ty: &ra_ap_hir::Type<'db>) -> bool {
        let display_target = DisplayTarget::from_crate(self.db, self.default_display_crate());
        let ty = ty.display(self.db, display_target).to_string();
        ty.contains("RenderSlot") || ty.contains("RenderSlotWith")
    }

    fn type_is_unit_return(&self, ty: &ra_ap_hir::Type<'db>) -> bool {
        let display_target = DisplayTarget::from_crate(self.db, self.default_display_crate());
        matches!(
            ty.display(self.db, display_target).to_string().as_str(),
            "()"
        )
    }

    fn type_is_into_render_slot_param(&self, ty: &ra_ap_hir::Type<'db>) -> bool {
        let display_target = DisplayTarget::from_crate(self.db, self.default_display_crate());
        let ty_str = ty.display(self.db, display_target).to_string();
        ty_str.contains("Into<RenderSlot>")
            || ty_str.contains("Into<RenderSlotWith")
            || self.type_display_mentions_render_slot(ty)
    }

    fn method_call_takes_render_slot_setter_at(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        let Some(receiver) = call.receiver() else {
            return false;
        };
        let Some(receiver_type) = self.receiver_type_name(&receiver, call.syntax()) else {
            return false;
        };
        let Some(name) = call.name_ref() else {
            return false;
        };
        let name = name.text();
        self.explicit_slot_setter_indexes
            .get(receiver_type.as_str())
            .and_then(|setters| setters.get(name.as_str()))
            .is_some_and(|indexes| indexes.contains(&index))
    }

    fn method_body_passes_param_to_render_slot(
        &self,
        call: &ast::MethodCallExpr,
        index: usize,
    ) -> bool {
        let resolved = self.sema.resolve_method_call(call);
        let Some(function) = resolved.and_then(|f| self.sema.source(f)) else {
            return false;
        };

        let Some(param_list) = function.value.param_list() else {
            return false;
        };
        let params: Vec<_> = param_list.params().collect();
        // HIR callable params skip self, AST params() also skips self.
        // So the HIR index maps directly to params().
        let Some(target_param) = params.get(index) else {
            return false;
        };
        let Some(param_name) = param_name(target_param) else {
            return false;
        };

        let Some(body) = function.value.body() else {
            return false;
        };

        self.param_flows_to_render_slot_in_body(&param_name, &body)
    }

    fn param_flows_to_render_slot_in_body(&self, param_name: &str, body: &ast::BlockExpr) -> bool {
        for node in body.syntax().descendants() {
            if let Some(call) = ast::CallExpr::cast(node.clone()) {
                let target = self.resolve_call_target(&call);
                if let Some(CallTarget::Resolved {
                    kind: CallKind::RuntimeApi(api),
                    ..
                }) = target
                    && matches!(
                        api,
                        TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
                    )
                {}
                let Some(CallTarget::Resolved { kind, .. }) = target else {
                    continue;
                };
                let CallKind::RuntimeApi(api) = kind else {
                    continue;
                };
                if !matches!(
                    api,
                    TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
                ) {
                    continue;
                }
                let Some(args) = call.arg_list() else {
                    continue;
                };
                for arg in args.args() {
                    if let Some(path) = expr_path(&arg)
                        && path.segment().is_some_and(|seg| {
                            seg.name_ref().is_some_and(|n| n.text() == param_name)
                        })
                    {
                        return true;
                    }
                }
            }
            if let Some(mc) = ast::MethodCallExpr::cast(node) {
                let Some(name_ref) = mc.name_ref() else {
                    continue;
                };
                if name_ref.text() != "into" {
                    continue;
                }
                let Some(receiver) = mc.receiver() else {
                    continue;
                };
                if let Some(path) = expr_path(&receiver)
                    && path
                        .segment()
                        .is_some_and(|seg| seg.name_ref().is_some_and(|n| n.text() == param_name))
                {
                    return true;
                }
            }
        }
        false
    }

    fn receiver_type_name(&self, receiver: &ast::Expr, syntax: &SyntaxNode) -> Option<String> {
        let receiver_type = self.sema.type_of_expr(receiver).map(|ty| ty.adjusted())?;
        let display_target = DisplayTarget::from_crate(self.db, self.current_crate(syntax));
        Some(last_type_segment_name(
            receiver_type
                .display(self.db, display_target)
                .to_string()
                .as_str(),
        ))
    }

    fn current_crate(&self, syntax: &SyntaxNode) -> Crate {
        self.sema
            .scope(syntax)
            .map(|scope| scope.module().krate(self.db).into())
            .or_else(|| self.tessera_crates.iter().copied().next())
            .expect("rust-analyzer workspace should contain at least one Tessera crate")
    }
    fn default_display_crate(&self) -> Crate {
        self.tessera_crates
            .iter()
            .copied()
            .next()
            .expect("rust-analyzer workspace should contain at least one Tessera crate")
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
            || path.contains("RenderSlot::new")
            || path.contains("RenderSlotWith::new")
            || is_internal_runtime_name(last)
    }

    fn is_semantically_relevant_unresolved_method(&self, path: &str) -> bool {
        path.ends_with("RenderSlot::new") || path.ends_with("RenderSlotWith::new")
    }

    fn function_path(&self, function: Function) -> String {
        let edition = function.module(self.db).krate(self.db).edition(self.db);
        ModuleDef::from(function)
            .canonical_path(self.db, edition)
            .unwrap_or_else(|| function.name(self.db).display(self.db, edition).to_string())
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
