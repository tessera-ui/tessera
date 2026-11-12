use ra_ap_base_db::SourceDatabase;
use ra_ap_hir::{Local, PathResolution};
use ra_ap_syntax::{
    AstNode, SyntaxNode,
    ast::{self, HasArgList},
};

use super::{
    super::{
        error_codes::{DiagnosticCode, Lint},
        types::{Applicability, Edit, Suggestion},
    },
    CallAnalyzer, CallKind, CallTarget, Diagnostic, TesseraRuntimeApi,
};

pub(super) const MODEL_PATHS: &[&str] = &["tessera_ui::runtime::model::Model", "tessera_ui::Model"];

impl<'db> CallAnalyzer<'db> {
    pub(super) fn redundant_shared_render_slot(
        &self,
        call: &ast::MethodCallExpr,
    ) -> Option<Diagnostic> {
        let name = call.name_ref()?;
        let setter = name.text().strip_suffix("_shared")?;
        let owner = self.method_call_implementing_type_key(call)?;
        if !self.setter_accepts_render_slot_at(&owner, setter, 0) {
            return None;
        }
        let mut args = call.arg_list()?.args();
        let mut value = args.next()?;
        if args.next().is_some() {
            return None;
        }
        while let ast::Expr::BlockExpr(block) = value {
            if block.async_token().is_some()
                || block.const_token().is_some()
                || block.gen_token().is_some()
            {
                return None;
            }
            value = block.stmt_list()?.tail_expr()?;
        }
        let ast::Expr::CallExpr(constructor) = value else {
            return None;
        };
        let CallTarget::Resolved {
            kind: CallKind::RuntimeApi(kind),
            ..
        } = self.resolve_call_target(&constructor)?
        else {
            return None;
        };
        if !matches!(
            kind,
            TesseraRuntimeApi::RenderSlotNew | TesseraRuntimeApi::RenderSlotWithNew
        ) {
            return None;
        }
        let source = self.sema.original_range(constructor.syntax());
        let file_id = source.file_id.file_id(self.db);
        let mut suggestions = Vec::new();
        if kind == TesseraRuntimeApi::RenderSlotNew
            && let Some(content) = constructor.arg_list()?.args().next()
            && let Some(mut suggestion) =
                self.direct_slot_suggestion(constructor.syntax(), content.syntax())
        {
            let method = self.sema.original_range(name.syntax());
            if method.file_id == source.file_id {
                suggestion.applicability = Applicability::MaybeIncorrect;
                suggestion.edits.push(Edit {
                    file_id,
                    range: method.range,
                    replacement: setter.into(),
                });
                suggestions.push(suggestion);
            }
        }
        Some(Diagnostic {
            code: DiagnosticCode::Lint(Lint::RedundantRenderSlot),
            severity: cargo_metadata::diagnostic::DiagnosticLevel::Warning,
            file_id,
            range: source.range,
            message: "redundant render-slot construction for a closure setter".into(),
            label: format!("`{setter}` already accepts build content directly"),
            help: format!("pass the build content to `{setter}`; use `{}` for an existing shared slot", name.text()),
            notes: vec!["the closure setter creates its slot in the component declaration's scope; review ownership when moving this construction".into()],
            suggestions,
            boundary: None,
        })
    }

    pub(super) fn redundant_render_slot(&self, closure: &ast::ClosureExpr) -> Option<Diagnostic> {
        let statements = closure.syntax().parent().and_then(ast::StmtList::cast)?;
        if statements.tail_expr()?.syntax() != closure.syntax() {
            return None;
        }
        let mut declarations = statements.statements();
        let ast::Stmt::LetStmt(declaration) = declarations.next()? else {
            return None;
        };
        if declarations.next().is_some() || self.is_inactive(declaration.syntax()) {
            return None;
        }
        let local = self.pattern_local(declaration.pat()?)?;
        let ast::Expr::CallExpr(constructor) = declaration.initializer()? else {
            return None;
        };
        let CallTarget::Resolved {
            kind: CallKind::RuntimeApi(kind),
            ..
        } = self.resolve_call_target(&constructor)?
        else {
            return None;
        };
        let expected = match kind {
            TesseraRuntimeApi::RenderSlotNew => TesseraRuntimeApi::RenderSlotRender,
            TesseraRuntimeApi::RenderSlotWithNew => TesseraRuntimeApi::RenderSlotWithRender,
            _ => return None,
        };
        let ast::Expr::ClosureExpr(content) = constructor.arg_list()?.args().next()? else {
            return None;
        };
        let params = closure.param_list()?.params().collect::<Vec<_>>();
        if content.param_list()?.params().count() != params.len() {
            return None;
        }
        let body = sole_expression(closure.body()?)?;
        let ast::Expr::MethodCallExpr(forward) = body else {
            return None;
        };
        let function = self
            .method_call_variants(&forward)
            .find_map(|call| self.sema.resolve_method_call(&call))?;
        if self.runtime_api_for_function(function, &self.function_path(function)) != Some(expected)
            || self.expression_local(forward.receiver()?) != Some(local)
        {
            return None;
        }
        let args = forward.arg_list()?.args().collect::<Vec<_>>();
        match expected {
            TesseraRuntimeApi::RenderSlotRender if params.is_empty() && args.is_empty() => {}
            TesseraRuntimeApi::RenderSlotWithRender if params.len() == 1 && args.len() == 1 => {
                if self.pattern_local(params[0].pat()?)?
                    != self.expression_local(args[0].clone())?
                {
                    return None;
                }
            }
            _ => return None,
        }
        let source = self.sema.original_range(constructor.syntax());
        let boundary = self.sema.original_range(forward.syntax());
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id) || boundary.file_id != source.file_id {
            return None;
        }
        let block = statements.syntax().parent()?;
        let suggestions: Vec<_> = self
            .direct_slot_suggestion(&block, content.syntax())
            .into_iter()
            .collect();
        Some(Diagnostic {
            code: DiagnosticCode::Lint(Lint::RedundantRenderSlot),
            severity: cargo_metadata::diagnostic::DiagnosticLevel::Warning,
            file_id,
            range: source.range,
            message: "redundant render-slot wrapper".into(),
            label: "this slot is only forwarded into another build closure".into(),
            help: if suggestions.is_empty() {
                "pass the build closure directly"
            } else {
                ""
            }
            .into(),
            notes: Vec::new(),
            suggestions,
            boundary: Some((
                boundary.range,
                "this closure only expands the local slot".into(),
            )),
        })
    }

    /// Reports `remember` or `retain` states captured by a `Model::new_with`
    /// initializer: storing them in the model does not transfer ownership.
    pub(super) fn remembered_state_in_model(&self, call: &ast::CallExpr) -> Vec<Diagnostic> {
        if !self
            .call_function(call)
            .is_some_and(|function| self.is_method_of(function, "new_with", MODEL_PATHS))
        {
            return Vec::new();
        }
        let Some(ast::Expr::ClosureExpr(init)) =
            call.arg_list().and_then(|args| args.args().next())
        else {
            return Vec::new();
        };
        let Some(body) = call
            .syntax()
            .ancestors()
            .find_map(ast::Fn::cast)
            .and_then(|f| f.body())
        else {
            return Vec::new();
        };
        let remembered: Vec<(Local<'db>, ast::LetStmt)> = body
            .syntax()
            .descendants()
            .filter_map(ast::LetStmt::cast)
            .filter_map(|statement| {
                let ast::Expr::CallExpr(initializer) = statement.initializer()? else {
                    return None;
                };
                let CallTarget::Resolved {
                    kind: CallKind::RuntimeApi(api),
                    ..
                } = self.resolve_call_target(&initializer)?
                else {
                    return None;
                };
                matches!(
                    api,
                    TesseraRuntimeApi::Remember
                        | TesseraRuntimeApi::RememberWithKey
                        | TesseraRuntimeApi::Retain
                        | TesseraRuntimeApi::RetainWithKey
                )
                .then_some(())?;
                Some((self.pattern_local(statement.pat()?)?, statement))
            })
            .collect();
        if remembered.is_empty() {
            return Vec::new();
        }
        init.syntax()
            .descendants()
            .filter_map(ast::PathExpr::cast)
            .filter_map(|path| {
                let local = self.expression_local(ast::Expr::PathExpr(path.clone()))?;
                let (_, statement) = remembered.iter().find(|(candidate, _)| *candidate == local)?;
                let source = self.sema.original_range(path.syntax());
                let file_id = source.file_id.file_id(self.db);
                if !self.local_files.contains(&file_id) {
                    return None;
                }
                let declaration = self.sema.original_range(statement.syntax());
                Some(Diagnostic {
                    code: DiagnosticCode::Lint(Lint::RememberedStateInModel),
                    severity: cargo_metadata::diagnostic::DiagnosticLevel::Warning,
                    file_id,
                    range: source.range,
                    message: "remembered state stored in a model".into(),
                    label: "this state keeps its composition-slot lifetime".into(),
                    help: "create model-owned state with `State::new` inside the model initializer"
                        .into(),
                    notes: vec![
                        "the model will hold a handle that expires when this component's slot is released"
                            .into(),
                    ],
                    suggestions: Vec::new(),
                    boundary: (declaration.file_id == source.file_id).then(|| {
                        (declaration.range, "state created by the component slot here".into())
                    }),
                })
            })
            .collect()
    }

    pub(super) fn call_function(&self, call: &ast::CallExpr) -> Option<ra_ap_hir::Function> {
        let callee = call.expr()?;
        std::iter::once(callee.clone())
            .chain(self.sema.descend_node_into_attributes(callee))
            .find_map(
                |callee| match self.sema.resolve_expr_as_callable(&callee)?.kind() {
                    ra_ap_hir::CallableKind::Function(function) => Some(function),
                    _ => None,
                },
            )
    }

    pub(super) fn is_method_of(
        &self,
        function: ra_ap_hir::Function,
        name: &str,
        type_paths: &[&str],
    ) -> bool {
        use ra_ap_hir::AsAssocItem;
        function.name(self.db).as_str() == name
            && function
                .as_assoc_item(self.db)
                .and_then(|item| item.implementing_ty(self.db))
                .and_then(|ty| self.type_adt_canonical_path(&ty))
                .is_some_and(|path| type_paths.contains(&path.as_str()))
    }

    fn direct_slot_suggestion(
        &self,
        wrapper: &SyntaxNode,
        content: &SyntaxNode,
    ) -> Option<Suggestion> {
        let source = self.sema.original_range(wrapper);
        let inner = self.sema.original_range(content);
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id)
            || source.file_id != inner.file_id
            || !source.range.contains_range(inner.range)
        {
            return None;
        }
        let snapshot = self.db.file_text(file_id);
        let text = snapshot.text(self.db);
        if text[u32::from(inner.range.start()) as usize..u32::from(inner.range.end()) as usize]
            != content.text().to_string()
        {
            return None;
        }
        let loses_comments = wrapper.descendants_with_tokens().any(|token| {
            token.kind() == ra_ap_syntax::SyntaxKind::COMMENT
                && !content.text_range().contains_range(token.text_range())
        });
        Some(Suggestion {
            message: "pass the build closure directly".into(),
            applicability: if loses_comments {
                Applicability::MaybeIncorrect
            } else {
                Applicability::MachineApplicable
            },
            edits: [
                ra_ap_span::TextRange::new(source.range.start(), inner.range.start()),
                ra_ap_span::TextRange::new(inner.range.end(), source.range.end()),
            ]
            .into_iter()
            .map(|range| Edit {
                file_id,
                range,
                replacement: String::new(),
            })
            .collect(),
        })
    }

    fn pattern_local(&self, pattern: ast::Pat) -> Option<Local<'db>> {
        let ast::Pat::IdentPat(pattern) = pattern else {
            return None;
        };
        std::iter::once(pattern.clone())
            .chain(self.sema.descend_node_into_attributes(pattern))
            .find_map(|pattern| self.sema.to_def(&pattern))
    }

    fn expression_local(&self, expression: ast::Expr) -> Option<Local<'db>> {
        let ast::Expr::PathExpr(expression) = expression else {
            return None;
        };
        let path = expression.path()?;
        std::iter::once(path.clone())
            .chain(self.sema.descend_node_into_attributes(path))
            .find_map(|path| match self.sema.resolve_path(&path) {
                Some(PathResolution::Local(local)) => Some(local),
                _ => None,
            })
    }
}

fn sole_expression(expression: ast::Expr) -> Option<ast::Expr> {
    let ast::Expr::BlockExpr(block) = expression else {
        return Some(expression);
    };
    if block.async_token().is_some()
        || block.const_token().is_some()
        || block.gen_token().is_some()
        || block.unsafe_token().is_some()
    {
        return None;
    }
    let list = block.stmt_list()?;
    let mut statements = list.statements();
    match (statements.next(), statements.next(), list.tail_expr()) {
        (None, None, Some(tail)) => Some(tail),
        (Some(ast::Stmt::ExprStmt(statement)), None, None) => statement.expr(),
        _ => None,
    }
}
