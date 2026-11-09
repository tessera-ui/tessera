use std::collections::{HashSet, VecDeque};

use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_base_db::Crate;
use ra_ap_hir::{Macro, Module, ModuleDef, PathResolution, ScopeDef};
use ra_ap_syntax::{
    AstNode, SyntaxNode,
    ast::{self, HasAttrs},
};

use super::{CallAnalyzer, Diagnostic, ErrorCode};

const PRIVATE_MODULE_NAME: &str = "__private";
const PRIVATE_MODULE_CRATES: [&str; 2] = ["tessera_ui", "tessera_shard"];

impl CallAnalyzer<'_> {
    /// Rejects names that reach a Tessera crate's macro-support module.
    ///
    /// Source text is user-written even inside official attribute macro input,
    /// so it is always checked. Expansions are checked unless the macro is
    /// defined by a Tessera crate; an expansion whose macro cannot be resolved
    /// is checked as user code.
    pub(super) fn check_private_api_uses(&mut self, node: &SyntaxNode) {
        if self.is_inactive(node) {
            return;
        }
        if let Some(item) = ast::Item::cast(node.clone()) {
            self.check_item_macro_expansions(&item);
        }
        if let Some(call) = ast::MacroCall::cast(node.clone())
            && let Some(expansion) = self.sema.expand_macro_call(&call)
            && !self.is_official_macro(self.sema.resolve_macro_call(&call))
        {
            self.check_private_api_uses(&expansion.value);
        }
        if let Some(path) = ast::Path::cast(node.clone()) {
            self.check_private_path(&path);
            return;
        }
        if let Some(tree) = ast::UseTree::cast(node.clone()) {
            // Imports below a rejected prefix name the same private module.
            if let Some(path) = tree.path()
                && self.check_private_path(&path)
            {
                return;
            }
            if let Some(list) = tree.use_tree_list() {
                self.check_private_api_uses(list.syntax());
            }
            return;
        }
        self.check_private_api_children(node);
    }

    fn check_private_api_children(&mut self, node: &SyntaxNode) {
        for child in node.children() {
            self.check_private_api_uses(&child);
        }
    }

    fn check_item_macro_expansions(&mut self, item: &ast::Item) {
        if let Some(expansion) = self.sema.expand_attr_macro(item)
            && !self.is_official_macro(self.sema.resolve_attr_macro_call(item))
        {
            self.check_private_api_uses(&expansion.value.value);
        }
        if !matches!(
            item,
            ast::Item::Struct(_) | ast::Item::Enum(_) | ast::Item::Union(_)
        ) {
            return;
        }
        for meta in item.attrs().filter_map(|attr| attr.meta()) {
            let Some(expansions) = self.sema.expand_derive_macro(&meta) else {
                continue;
            };
            let macros = self.sema.resolve_derive_macro(&meta).unwrap_or_default();
            for (index, expansion) in expansions.into_iter().enumerate() {
                if let Some(expansion) = expansion
                    && !self.is_official_macro(macros.get(index).copied().flatten())
                {
                    self.check_private_api_uses(&expansion.value);
                }
            }
        }
    }

    fn is_official_macro(&self, mac: Option<Macro>) -> bool {
        mac.is_some_and(|mac| self.is_tessera_crate(mac.module(self.db).krate(self.db).into()))
    }

    /// Reports `path` once if any of its prefixes reaches a macro-support item,
    /// then checks paths nested in its generic arguments.
    fn check_private_path(&mut self, path: &ast::Path) -> bool {
        let prefixes = std::iter::successors(Some(path.clone()), ast::Path::qualifier)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let use_site = self.sema.scope(path.syntax()).map(|scope| scope.module());
        let mut reported = false;
        if let Some(use_site) = use_site {
            for (index, prefix) in prefixes.iter().enumerate() {
                let Some(item) = self
                    .resolve_path_any(prefix)
                    .and_then(|resolution| self.private_api_item(resolution, use_site))
                else {
                    continue;
                };
                let rest = prefixes[index + 1..]
                    .iter()
                    .filter_map(|prefix| prefix.segment()?.name_ref())
                    .map(|name| format!("::{}", name.text()))
                    .collect::<String>();
                self.push_private_api_use(path.syntax(), format!("{item}{rest}"));
                reported = true;
                break;
            }
        }
        for segment in prefixes.iter().filter_map(ast::Path::segment) {
            self.check_private_api_children(segment.syntax());
        }
        reported
    }

    fn resolve_path_any(&self, path: &ast::Path) -> Option<PathResolution<'_>> {
        std::iter::once(path.clone())
            .chain(self.sema.descend_node_into_attributes(path.clone()))
            .find_map(|path| self.sema.resolve_path(&path))
    }

    /// Names a resolution that is only reachable through another crate's
    /// macro-support module.
    fn private_api_item(&self, resolution: PathResolution<'_>, use_site: Module) -> Option<String> {
        let PathResolution::Def(definition) = resolution else {
            return None;
        };
        let module = match definition {
            ModuleDef::Module(module) => module,
            definition => definition.module(self.db)?,
        };
        let krate = module.krate(self.db);
        let crate_name = krate.display_name(self.db)?.to_string();
        if !PRIVATE_MODULE_CRATES.contains(&crate_name.as_str()) || use_site.krate(self.db) == krate
        {
            return None;
        }
        if let ModuleDef::Module(module) = definition
            && self.is_private_module(module)
        {
            return Some(format!("{crate_name}::{PRIVATE_MODULE_NAME}"));
        }
        if !self.is_private_only_item(krate.into(), definition, use_site) {
            return None;
        }
        let name = definition.name(self.db)?;
        Some(format!(
            "{crate_name}::{PRIVATE_MODULE_NAME}::{}",
            name.as_str()
        ))
    }

    fn is_private_module(&self, module: Module) -> bool {
        module
            .name(self.db)
            .is_some_and(|name| name.as_str() == PRIVATE_MODULE_NAME)
            && module
                .parent(self.db)
                .is_some_and(|parent| parent.is_crate_root(self.db))
    }

    /// `observer` lies outside `krate`, so it sees exactly the public API.
    fn is_private_only_item(&self, krate: Crate, definition: ModuleDef, observer: Module) -> bool {
        if let Some(items) = self.private_api_items.borrow().get(&krate) {
            return items.contains(&definition);
        }
        let root = ra_ap_hir::Crate::from(krate).root_module(self.db);
        let observer = Some(observer);
        let mut public = HashSet::new();
        let mut private = HashSet::new();
        let mut visited = HashSet::from([root]);
        let mut queue = VecDeque::from([root]);
        while let Some(module) = queue.pop_front() {
            for (_, scope) in module.scope(self.db, observer) {
                let ScopeDef::ModuleDef(item) = scope else {
                    continue;
                };
                let local_module = match item {
                    ModuleDef::Module(child) if Crate::from(child.krate(self.db)) == krate => {
                        Some(child)
                    }
                    _ => None,
                };
                if let Some(child) = local_module
                    && self.is_private_module(child)
                {
                    private.extend(child.scope(self.db, observer).into_iter().filter_map(
                        |(_, scope)| match scope {
                            ScopeDef::ModuleDef(item) => Some(item),
                            _ => None,
                        },
                    ));
                    continue;
                }
                public.insert(item);
                if let Some(child) = local_module
                    && visited.insert(child)
                {
                    queue.push_back(child);
                }
            }
        }
        let items = private.difference(&public).copied().collect::<HashSet<_>>();
        let found = items.contains(&definition);
        self.private_api_items.borrow_mut().insert(krate, items);
        found
    }

    fn push_private_api_use(&mut self, syntax: &SyntaxNode, item: String) {
        let source = self.sema.original_range(syntax);
        let file_id = source.file_id.file_id(self.db);
        if !self.local_files.contains(&file_id)
            || self.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == ErrorCode::PrivateApi.into()
                    && diagnostic.file_id == file_id
                    && diagnostic.range == source.range
            })
        {
            return;
        }
        self.diagnostics.push(Diagnostic {
            code: ErrorCode::PrivateApi.into(),
            severity: Severity::Error,
            file_id,
            range: source.range,
            message: format!("cannot use private Tessera macro-support item `{item}`"),
            label: "reserved for code generated by official Tessera macros".into(),
            help: "use the public Tessera API instead; macro-support items have no stability guarantee".into(),
            boundary: None,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }
}
