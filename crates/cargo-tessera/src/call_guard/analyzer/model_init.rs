use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_hir::{ModuleDef, PathResolution};
use ra_ap_syntax::{
    AstNode, SyntaxNode,
    ast::{self, HasName},
};

use super::{CallAnalyzer, Diagnostic, ErrorCode, lints::MODEL_STORE_PATHS};

const STATE_PATHS: &[&str] = &["tessera_ui::runtime::State", "tessera_ui::State"];
const MODEL_INIT_PATHS: &[&str] = &[
    "tessera_ui::runtime::model::ModelInit",
    "tessera_ui::ModelInit",
];

impl CallAnalyzer<'_> {
    /// Rejects `State::new` outside a model initializer.
    pub(super) fn check_model_state(&mut self, call: &ast::CallExpr) {
        if !self
            .call_function(call)
            .is_some_and(|function| self.is_method_of(function, "new", STATE_PATHS))
        {
            return;
        }
        if self.in_model_initializer(call.syntax()) {
            return;
        }
        let source = self.sema.original_range(call.syntax());
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id) {
            return;
        }
        self.diagnostics.push(Diagnostic {
            code: ErrorCode::ModelState.into(),
            severity: Severity::Error,
            file_id,
            range: source.range,
            message: "`State::new` outside a model initializer".into(),
            label: "model state needs a model owner".into(),
            help: "use `remember` for component state, or create this state in \
                   `ModelInit::init` of a model type"
                .into(),
            notes: Vec::new(),
            suggestions: Vec::new(),
            boundary: None,
        });
    }

    fn in_model_initializer(&self, node: &SyntaxNode) -> bool {
        let Some(boundary) = node
            .ancestors()
            .skip(1)
            .find(|node| ast::ClosureExpr::can_cast(node.kind()) || ast::Fn::can_cast(node.kind()))
        else {
            return false;
        };
        match ast::ClosureExpr::cast(boundary.clone()) {
            Some(closure) => self.is_initializer_closure(&closure),
            None => ast::Fn::cast(boundary).is_some_and(|function| self.is_model_init(&function)),
        }
    }

    fn is_initializer_closure(&self, closure: &ast::ClosureExpr) -> bool {
        let Some(owner) = closure
            .syntax()
            .parent()
            .filter(|parent| ast::ArgList::can_cast(parent.kind()))
            .and_then(|args| args.parent())
        else {
            return false;
        };
        let Some(call) = ast::MethodCallExpr::cast(owner) else {
            return false;
        };
        self.method_call_variants(&call)
            .find_map(|call| self.sema.resolve_method_call(&call))
            .is_some_and(|function| self.is_method_of(function, "model", MODEL_STORE_PATHS))
    }

    /// Returns whether `function` is `init` in an `impl ModelInit for ...`.
    fn is_model_init(&self, function: &ast::Fn) -> bool {
        if function.name().is_none_or(|name| name.text() != "init") {
            return false;
        }
        let Some(item_impl) = function
            .syntax()
            .parent()
            .and_then(|items| items.parent())
            .and_then(ast::Impl::cast)
        else {
            return false;
        };
        let Some(ast::Type::PathType(trait_type)) = item_impl.trait_() else {
            return false;
        };
        matches!(
            trait_type.path().and_then(|path| self.sema.resolve_path(&path)),
            Some(PathResolution::Def(ModuleDef::Trait(model_init)))
                if self
                    .trait_canonical_path(model_init)
                    .is_some_and(|path| MODEL_INIT_PATHS.contains(&path.as_str()))
        )
    }
}
