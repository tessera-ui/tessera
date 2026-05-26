use std::collections::HashSet;

use ra_ap_base_db::{Crate, all_crates};
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::ast::{self, HasName};

use super::types::TesseraRuntimeApi;

pub(crate) fn param_name(param: &ast::Param) -> Option<String> {
    let ast::Pat::IdentPat(pat) = param.pat()? else {
        return None;
    };
    pat.name().map(|name| name.text().to_string())
}

pub(crate) fn expr_path(expr: &ast::Expr) -> Option<ast::Path> {
    match expr {
        ast::Expr::PathExpr(path_expr) => path_expr.path(),
        _ => None,
    }
}

pub(crate) fn path_text(path: &ast::Path) -> String {
    path.segments()
        .filter_map(|segment| segment.name_ref().map(|name| name.text().to_string()))
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn path_last_segment(path: &ast::Path) -> Option<String> {
    path.segments()
        .last()
        .and_then(|segment| segment.name_ref())
        .map(|name| name.text().to_string())
}

pub(crate) fn is_internal_runtime_name(name: &str) -> bool {
    matches!(
        name,
        "current_instance"
            | "current_node"
            | "current_scope"
            | "enter_call"
            | "exit_call"
            | "enter_component"
            | "exit_component"
            | "enter_slot"
            | "exit_slot"
            | "start_group"
            | "end_group"
            | "set_current_instance"
    )
}

pub(crate) fn runtime_api_label(api: TesseraRuntimeApi) -> &'static str {
    match api {
        TesseraRuntimeApi::EntryPointNew => "EntryPoint::new",
        TesseraRuntimeApi::Remember => "remember",
        TesseraRuntimeApi::RememberWithKey => "remember_with_key",
        TesseraRuntimeApi::Retain => "retain",
        TesseraRuntimeApi::RetainWithKey => "retain_with_key",
        TesseraRuntimeApi::ProvideContext => "provide_context",
        TesseraRuntimeApi::UseContext => "use_context",
        TesseraRuntimeApi::ReceiveFrameNanos => "receive_frame_nanos",
        TesseraRuntimeApi::Key => "key",
        TesseraRuntimeApi::RenderSlotNew => "RenderSlot::new",
        TesseraRuntimeApi::RenderSlotWithNew => "RenderSlotWith::new",
        TesseraRuntimeApi::InternalRuntime => "internal runtime API",
    }
}

pub(crate) fn tessera_crates(db: &RootDatabase) -> HashSet<Crate> {
    all_crates(db)
        .iter()
        .copied()
        .filter(|krate| crate_is_tessera_crate(db, *krate))
        .collect()
}

pub(crate) fn crate_is_tessera_crate(db: &RootDatabase, krate: Crate) -> bool {
    if crate_has_tessera_name(db, krate) {
        return true;
    }

    krate
        .data(db)
        .dependencies
        .iter()
        .any(|dep| dep.name.symbol().as_str().starts_with("tessera_"))
}

pub(crate) fn crate_has_tessera_name(db: &RootDatabase, krate: Crate) -> bool {
    all_crates(db).iter().copied().any(|candidate| {
        candidate == krate
            && candidate
                .extra_data(db)
                .display_name
                .as_ref()
                .is_some_and(|name| {
                    let canonical = name.canonical_name().as_str();
                    canonical == "tessera-ui"
                        || canonical == "tessera-components"
                        || canonical == "tessera-shard"
                        || canonical.starts_with("tessera-")
                })
    })
}
