use std::cmp::Reverse;

use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_cfg::{CfgExpr, CfgOptions};
use ra_ap_hir::{EditionedFileId, ModuleDef, PathResolution};
use ra_ap_ide::FileId;
use ra_ap_span::TextRange;
use ra_ap_syntax::{
    AstNode, SyntaxElement, T,
    ast::{self, HasAttrs},
};

use super::super::{
    error_codes::Lint,
    levels::{Level, LintTarget},
};
use super::{CallAnalyzer, Diagnostic, DiagnosticCode, ErrorCode, LintOptions};

struct Setting {
    file: FileId,
    owner: TextRange,
    span: TextRange,
    target: LintTarget,
    level: Level,
}

impl Setting {
    fn contains(&self, file: FileId, range: TextRange) -> bool {
        self.file == file && self.owner.contains_range(range)
    }

    fn apply(&self, levels: &mut (Level, Level), options: &LintOptions) -> bool {
        if levels.0 == Level::ForceWarn || (levels.0 == Level::Warn && levels.1 == Level::ForceWarn)
        {
            return true;
        }
        let warnings = self.target == LintTarget::Warnings;
        if warnings && options.cap_level(levels.0) == Level::Forbid {
            return true;
        }
        let inherited_forbid =
            !warnings && levels.0 == Level::Warn && options.cap_level(levels.1) == Level::Forbid;
        let current = if warnings {
            &mut levels.1
        } else {
            &mut levels.0
        };
        if inherited_forbid || options.cap_level(*current) == Level::Forbid {
            return matches!(self.level, Level::Deny | Level::Forbid);
        }
        *current = self.level;
        true
    }
}

impl CallAnalyzer<'_> {
    pub(super) fn apply_lint_levels(&mut self, options: &LintOptions) {
        let mut settings = Vec::new();
        for file in self.local_files.clone() {
            let tree = self
                .sema
                .parse(EditionedFileId::current_edition(self.db, file));
            for node in tree.syntax().descendants().filter(|node| {
                ast::Item::can_cast(node.kind()) || ast::SourceFile::can_cast(node.kind())
            }) {
                if self.is_inactive(&node) {
                    continue;
                }
                let Some(owner) = ast::AnyHasAttrs::cast(node.clone()) else {
                    continue;
                };
                let Some(scope) = self.sema.scope(&node) else {
                    continue;
                };
                let krate = scope.module().krate(self.db);
                for attr in owner.attrs() {
                    if let Some(meta) = attr.meta() {
                        self.collect_lint_attribute(
                            meta,
                            krate.cfg(self.db),
                            file,
                            node.text_range(),
                            &mut settings,
                        );
                    }
                }
            }
        }
        settings.sort_by_key(|setting| (setting.file, Reverse(setting.owner.len())));
        for (index, setting) in settings.iter().enumerate() {
            let mut levels = options.command_line(Lint::RedundantRenderSlot);
            for (previous, entry) in settings[..=index]
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.contains(setting.file, setting.owner))
            {
                if !entry.apply(&mut levels, options) && previous == index {
                    self.lint_configuration_error(
                        setting.file,
                        setting.span,
                        "cannot lower a forbidden Tessera lint level".into(),
                    );
                }
            }
        }
        self.diagnostics.retain_mut(|diagnostic| {
            let DiagnosticCode::Lint(lint) = diagnostic.code else {
                return true;
            };
            let mut levels = options.command_line(lint);
            for setting in settings
                .iter()
                .filter(|setting| setting.contains(diagnostic.file_id, diagnostic.range))
            {
                setting.apply(&mut levels, options);
            }
            let level = options.level(levels);
            let Some(severity) = level.severity() else {
                return false;
            };
            diagnostic.severity = severity;
            diagnostic.notes.push(match level {
                Level::ForceWarn => "this lint is fixed at warning level by --force-warn".into(),
                Level::Forbid => {
                    "this lint is forbidden and cannot be allowed in an inner scope".into()
                }
                _ => format!(
                    "#[tessera_ui::lint(allow({}))]",
                    lint.name().trim_start_matches("tessera::")
                ),
            });
            true
        });
    }

    fn collect_lint_attribute(
        &mut self,
        meta: ast::Meta,
        cfg: &CfgOptions,
        file: FileId,
        owner: TextRange,
        settings: &mut Vec<Setting>,
    ) {
        if let ast::Meta::CfgAttrMeta(attribute) = meta {
            if attribute.cfg_predicate().is_some_and(|predicate| {
                cfg.check(&CfgExpr::parse_from_ast(predicate)) == Some(true)
            }) {
                for meta in attribute.metas() {
                    self.collect_lint_attribute(meta, cfg, file, owner, settings);
                }
            }
            return;
        }
        let Some(path) = meta.path() else { return };
        let Some(PathResolution::Def(ModuleDef::Macro(mac))) = self.sema.resolve_path(&path) else {
            return;
        };
        if mac.name(self.db).as_str() != "lint"
            || !self.is_tessera_crate(mac.module(self.db).krate(self.db).into())
        {
            return;
        }
        let result = (|| {
            let ast::Meta::TokenTreeMeta(attribute) = &meta else {
                return Err("expected lint level groups".into());
            };
            let tree = attribute.token_tree().ok_or("expected lint level groups")?;
            let mut parsed = Vec::new();
            for group in comma_groups(tree)? {
                if group.len() != 2 {
                    return Err("expected level(lint, ...)".into());
                }
                let level: Level = group[0].to_string().parse()?;
                if level == Level::ForceWarn {
                    return Err("source lint levels must be allow, warn, deny, or forbid".into());
                }
                let tree = group[1]
                    .clone()
                    .into_node()
                    .and_then(ast::TokenTree::cast)
                    .ok_or("expected parenthesized lint names")?;
                for target in comma_groups(tree)? {
                    let name: String = target.iter().map(ToString::to_string).collect();
                    let name = if name.contains("::") || name == "warnings" {
                        name
                    } else {
                        format!("tessera::{name}")
                    };
                    parsed.push(Setting {
                        file,
                        owner,
                        span: TextRange::new(
                            target.first().unwrap().text_range().start(),
                            target.last().unwrap().text_range().end(),
                        ),
                        target: name.parse()?,
                        level,
                    });
                }
            }
            Ok::<_, String>(parsed)
        })();
        match result {
            Ok(parsed) => settings.extend(parsed),
            Err(message) => {
                self.lint_configuration_error(file, meta.syntax().text_range(), message)
            }
        }
    }

    fn lint_configuration_error(&mut self, file_id: FileId, range: TextRange, message: String) {
        self.diagnostics.push(Diagnostic {
            code: ErrorCode::LintConfiguration.into(), severity: Severity::Error,
            file_id, range, message,
            label: "invalid Tessera lint configuration".into(),
            help: "use #[tessera_ui::lint(warn(tessera::redundant_render_slot))] with a supported level and lint name".into(),
            boundary: None,
            notes: Vec::new(),
            suggestions: Vec::new(),
        });
    }
}

fn comma_groups(tree: ast::TokenTree) -> Result<Vec<Vec<SyntaxElement>>, String> {
    if tree.l_paren_token().is_none() || tree.r_paren_token().is_none() {
        return Err("expected parentheses".into());
    }
    let mut groups = vec![Vec::new()];
    for token in tree
        .syntax()
        .children_with_tokens()
        .filter(|token| !token.kind().is_trivia())
        .skip(1)
        .take_while(|token| token.kind() != T![')'])
    {
        if token.kind() == T![,] {
            if groups.last().unwrap().is_empty() {
                return Err("empty lint configuration entry".into());
            }
            groups.push(Vec::new());
        } else {
            groups.last_mut().unwrap().push(token);
        }
    }
    if groups.last().unwrap().is_empty() {
        groups.pop();
    }
    Ok(groups)
}
