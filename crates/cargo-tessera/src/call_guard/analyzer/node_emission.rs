use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_syntax::{AstNode, SyntaxNode, ast};

use super::{CallAnalyzer, CallKind, CallTarget, Diagnostic, ErrorCode, TesseraRuntimeApi};

/// Nodes a build context may emit before reaching a later expression, or
/// `None` once every path to it has diverged.
type Emitted = Option<u32>;

#[derive(Clone, Copy)]
struct Scope {
    limit: u32,
    in_loop: bool,
}

impl CallAnalyzer<'_> {
    /// Rejects a build context that may emit more than `limit` nodes, emit one
    /// repeatedly, or declare content after its node outside `child`.
    pub(super) fn check_node_emissions(&mut self, body: &SyntaxNode, limit: u32) {
        let scope = Scope {
            limit,
            in_loop: false,
        };
        self.emissions(body, Some(0), scope);
    }

    fn emissions(&mut self, node: &SyntaxNode, before: Emitted, scope: Scope) -> Emitted {
        let before = before?;
        if self.is_inactive(node)
            || ast::ClosureExpr::can_cast(node.kind())
            || ast::Item::can_cast(node.kind())
        {
            return Some(before);
        }
        if let Some(block) = ast::BlockExpr::cast(node.clone())
            && (block.async_token().is_some() || block.const_token().is_some())
        {
            return Some(before);
        }
        if let Some(expr) = ast::IfExpr::cast(node.clone()) {
            let after = self.child_emissions(expr.condition(), Some(before), scope);
            let then = self.child_emissions(expr.then_branch(), after, scope);
            let otherwise = match expr.else_branch() {
                Some(ast::ElseBranch::Block(block)) => self.emissions(block.syntax(), after, scope),
                Some(ast::ElseBranch::IfExpr(branch)) => {
                    self.emissions(branch.syntax(), after, scope)
                }
                None => after,
            };
            return then.max(otherwise);
        }
        if let Some(expr) = ast::MatchExpr::cast(node.clone()) {
            let after = self.child_emissions(expr.expr(), Some(before), scope);
            return expr
                .match_arm_list()
                .into_iter()
                .flat_map(|arms| arms.arms())
                .map(|arm| self.child_emissions(arm.expr(), after, scope))
                .max()
                .flatten()
                .or(after.filter(|_| expr.match_arm_list().is_none()));
        }
        if ast::ForExpr::can_cast(node.kind())
            || ast::WhileExpr::can_cast(node.kind())
            || ast::LoopExpr::can_cast(node.kind())
        {
            let mut current = Some(before);
            for child in node.children() {
                let in_body = ast::BlockExpr::can_cast(child.kind());
                let child_scope = Scope {
                    in_loop: scope.in_loop || in_body,
                    ..scope
                };
                current = self.emissions(&child, current, child_scope).or(current);
            }
            return if ast::LoopExpr::can_cast(node.kind()) {
                current.filter(|_| self.loop_breaks(node))
            } else {
                current
            };
        }
        if ast::ReturnExpr::can_cast(node.kind())
            || ast::BreakExpr::can_cast(node.kind())
            || ast::ContinueExpr::can_cast(node.kind())
        {
            self.sequence(node, Some(before), scope);
            return None;
        }

        let after = self.sequence(node, Some(before), scope)?;
        if let Some(call) = ast::CallExpr::cast(node.clone()) {
            return self.call_emissions(node, self.resolve_call_target(&call), after, scope);
        }
        if let Some(call) = ast::MethodCallExpr::cast(node.clone()) {
            let target = self
                .method_call_variants(&call)
                .find_map(|call| self.sema.resolve_method_call(&call))
                .and_then(|function| self.call_target_for_function(function));
            return self.call_emissions(node, target, after, scope);
        }
        if let Some(expr) = ast::Expr::cast(node.clone())
            && ast::MacroExpr::can_cast(node.kind())
            && self
                .sema
                .type_of_expr(&expr)
                .is_some_and(|ty| ty.original.is_never())
        {
            return None;
        }
        Some(after)
    }

    fn child_emissions(
        &mut self,
        node: Option<impl AstNode>,
        before: Emitted,
        scope: Scope,
    ) -> Emitted {
        match node {
            Some(node) => self.emissions(node.syntax(), before, scope),
            None => before,
        }
    }

    fn sequence(&mut self, node: &SyntaxNode, before: Emitted, scope: Scope) -> Emitted {
        node.children().try_fold(before?, |emitted, child| {
            self.emissions(&child, Some(emitted), scope)
        })
    }

    fn call_emissions(
        &mut self,
        node: &SyntaxNode,
        target: Option<CallTarget>,
        emitted: u32,
        scope: Scope,
    ) -> Emitted {
        let Some(CallTarget::Resolved { kind, .. }) = target else {
            return Some(emitted);
        };
        match kind {
            CallKind::RuntimeApi(TesseraRuntimeApi::Layout | TesseraRuntimeApi::Group) => {
                if scope.limit == 0 {
                    self.push_emission_diagnostic(
                        node,
                        "a root build closure cannot emit a node",
                        "call a component that emits the node",
                    );
                } else if scope.in_loop {
                    self.push_emission_diagnostic(
                        node,
                        "a node cannot be emitted in a loop",
                        "emit each node from a component called in the loop",
                    );
                } else if emitted >= scope.limit {
                    self.push_emission_diagnostic(
                        node,
                        "this build context may already have emitted its node",
                        "emit each additional node from its own component or `child` content",
                    );
                }
                Some(emitted + 1)
            }
            CallKind::TesseraFunction
            | CallKind::RuntimeApi(
                TesseraRuntimeApi::RenderSlotRender | TesseraRuntimeApi::RenderSlotWithRender,
            ) if emitted > 0 => {
                self.push_emission_diagnostic(
                    node,
                    "content declared after `layout` or `group` would become its child",
                    "declare node content in `child`",
                );
                Some(emitted)
            }
            _ => Some(emitted),
        }
    }

    fn loop_breaks(&self, node: &SyntaxNode) -> bool {
        node.descendants()
            .filter_map(ast::BreakExpr::cast)
            .any(|expr| {
                expr.syntax()
                    .ancestors()
                    .skip(1)
                    .find(|ancestor| {
                        ast::LoopExpr::can_cast(ancestor.kind())
                            || ast::ForExpr::can_cast(ancestor.kind())
                            || ast::WhileExpr::can_cast(ancestor.kind())
                            || ast::ClosureExpr::can_cast(ancestor.kind())
                    })
                    .is_some_and(|ancestor| &ancestor == node)
            })
    }

    fn push_emission_diagnostic(&mut self, node: &SyntaxNode, message: &str, help: &'static str) {
        let source = self.sema.original_range(node);
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id) {
            return;
        }
        self.diagnostics.push(Diagnostic {
            file_id,
            range: source.range,
            code: ErrorCode::NodeEmission.into(),
            severity: Severity::Error,
            message: message.to_string(),
            label: "each component, slot, or build closure emits at most one node".to_string(),
            help: help.into(),
            boundary: None,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }
}
