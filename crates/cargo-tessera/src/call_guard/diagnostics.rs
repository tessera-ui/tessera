use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::PathBuf,
};

use annotate_snippets::{
    AnnotationKind, Group, Level, Patch, Renderer, Snippet, renderer::DEFAULT_TERM_WIDTH,
};
use anyhow::{Context, Result, ensure};
use cargo_metadata::diagnostic::DiagnosticLevel;
use ra_ap_base_db::SourceDatabase;
use ra_ap_ide::{FileId, LineIndex};
use ra_ap_span::{TextRange, TextSize};
use serde_json::{Value, json};

use crate::output;

use super::types::{Applicability, CallAnalyzer, Diagnostic, MessageFormat};

fn byte_range(range: TextRange) -> std::ops::Range<usize> {
    u32::from(range.start()) as usize..u32::from(range.end()) as usize
}

fn child(level: &str, message: &str, spans: Vec<Value>) -> Value {
    json!({
        "message": message, "code": null, "level": level,
        "spans": spans, "children": [], "rendered": null,
    })
}

struct SourceFile {
    path: String,
    text: String,
    lines: LineIndex,
}

impl SourceFile {
    fn display_range(&self, range: TextRange) -> std::ops::Range<usize> {
        let offset = |offset: TextSize| {
            let offset = u32::from(offset) as usize;
            offset
                + self.text[..offset]
                    .bytes()
                    .filter(|byte| *byte == b'\t')
                    .count()
                    * 3
        };
        offset(range.start())..offset(range.end())
    }

    fn location(&self, offset: TextSize) -> (u32, usize) {
        let position = self.lines.line_col(offset);
        let start = self.lines.line(position.line).expect("source line").start();
        (
            position.line + 1,
            self.text[byte_range(TextRange::new(start, offset))]
                .chars()
                .count()
                + 1,
        )
    }

    fn line(&self, line: u32) -> &str {
        self.text[byte_range(self.lines.line(line - 1).expect("source line"))]
            .trim_end_matches(['\r', '\n'])
    }
}

pub(super) struct Reporter {
    files: HashMap<FileId, SourceFile>,
    root: PathBuf,
}

impl Reporter {
    pub(super) fn fix(&self, diagnostics: &[Diagnostic], quiet: bool) -> Result<bool> {
        let mut files = BTreeMap::<_, Vec<_>>::new();
        for diagnostic in diagnostics {
            let input = self.json(diagnostic)["message"].to_string();
            for suggestion in rustfix::get_suggestions_from_json(
                &input,
                &HashSet::<String>::new(),
                rustfix::Filter::MachineApplicableOnly,
            )? {
                let paths: BTreeSet<_> = suggestion
                    .solutions
                    .iter()
                    .flat_map(|solution| &solution.replacements)
                    .map(|replacement| replacement.snippet.file_name.clone())
                    .collect();
                for path in paths {
                    let mut suggestion = suggestion.clone();
                    for solution in &mut suggestion.solutions {
                        solution
                            .replacements
                            .retain(|replacement| replacement.snippet.file_name == path);
                    }
                    files.entry(path).or_default().push(suggestion);
                }
            }
        }
        let mut updates = Vec::new();
        for (path, suggestions) in files {
            let file = self
                .files
                .values()
                .find(|file| file.path == path)
                .context("suggestion source is unavailable")?;
            let fixed = rustfix::apply_suggestions(&file.text, &suggestions)
                .with_context(|| format!("cannot apply fixes to {path}"))?;
            if fixed != file.text {
                let path = self.root.join(path);
                ensure!(
                    fs::read_to_string(&path)? == file.text,
                    "source changed since analysis: {}",
                    path.display()
                );
                updates.push((path, fixed, suggestions.len()));
            }
        }
        for (path, fixed, count) in &updates {
            fs::write(path, fixed)?;
            if !quiet {
                output::status(
                    "Fixed",
                    format!(
                        "{} ({count} fix{})",
                        path.display(),
                        if *count == 1 { "" } else { "es" }
                    ),
                );
            }
        }
        Ok(!updates.is_empty())
    }

    pub(super) fn new(analyzer: &CallAnalyzer<'_>, diagnostics: &[Diagnostic]) -> Self {
        let mut files = HashMap::new();
        for file_id in diagnostics.iter().flat_map(|diagnostic| {
            std::iter::once(diagnostic.file_id).chain(
                diagnostic
                    .suggestions
                    .iter()
                    .flat_map(|suggestion| suggestion.edits.iter().map(|edit| edit.file_id)),
            )
        }) {
            files.entry(file_id).or_insert_with(|| {
                let path = analyzer.vfs.file_path(file_id);
                let path = path.as_path().map_or_else(
                    || path.to_string(),
                    |path| {
                        let path: &std::path::Path = path.as_ref();
                        path.strip_prefix(&analyzer.display_root)
                            .unwrap_or(path)
                            .display()
                            .to_string()
                    },
                );
                let text = analyzer.db.file_text(file_id).text(analyzer.db).to_string();
                SourceFile {
                    path,
                    lines: LineIndex::new(&text),
                    text,
                }
            });
        }
        Self {
            files,
            root: analyzer.display_root.clone(),
        }
    }

    fn report<'a>(&'a self, diagnostic: &'a Diagnostic, color: bool) -> Vec<Group<'a>> {
        let level = match diagnostic.severity {
            DiagnosticLevel::Warning => Level::WARNING,
            DiagnosticLevel::Help => Level::HELP,
            DiagnosticLevel::Note | DiagnosticLevel::FailureNote => Level::NOTE,
            _ => Level::ERROR,
        };
        let file = &self.files[&diagnostic.file_id];
        // Native source highlighting indexes the expanded text by character.
        let expand_tabs = color && file.text.contains('\t');
        let source = if expand_tabs {
            Cow::Owned(file.text.replace('\t', "    "))
        } else {
            Cow::Borrowed(file.text.as_str())
        };
        let display_range = |range| {
            if expand_tabs {
                file.display_range(range)
            } else {
                byte_range(range)
            }
        };
        let mut snippet = Snippet::source(source).path(&file.path).annotation(
            AnnotationKind::Primary
                .span(display_range(diagnostic.range))
                .highlight_source(true)
                .label(&diagnostic.label),
        );
        if let Some((range, message)) = &diagnostic.boundary {
            snippet = snippet.annotation(
                AnnotationKind::Context
                    .span(display_range(*range))
                    .highlight_source(true)
                    .label(message),
            );
        }
        let mut main = Group::with_title(
            level
                .primary_title(&diagnostic.message)
                .id(diagnostic.code.to_string()),
        );
        main = main.element(snippet);
        if !diagnostic.help.is_empty() {
            main = main.element(Level::HELP.message(&diagnostic.help));
        }
        main = main.elements(
            diagnostic
                .notes
                .iter()
                .map(|note| Level::NOTE.message(note)),
        );
        let mut report = vec![main];
        for suggestion in &diagnostic.suggestions {
            let mut files = BTreeMap::<_, Vec<_>>::new();
            for edit in &suggestion.edits {
                files
                    .entry(edit.file_id)
                    .or_default()
                    .push(Patch::new(byte_range(edit.range), &edit.replacement));
            }
            report.push(Level::HELP.secondary_title(&suggestion.message).elements(
                files.into_iter().map(|(file_id, patches)| {
                    let file = &self.files[&file_id];
                    Snippet::source(&file.text)
                        .path(&file.path)
                        .patches(patches)
                }),
            ));
        }
        report
    }

    pub(super) fn render(&self, diagnostic: &Diagnostic, short: bool, color: bool) -> String {
        let width = console::Term::stderr()
            .size_checked()
            .map_or(DEFAULT_TERM_WIDTH, |(_, columns)| usize::from(columns));
        let renderer = if color {
            Renderer::styled()
        } else {
            Renderer::plain()
        };
        let mut rendered = renderer
            .term_width(width)
            .short_message(short)
            .render(&self.report(diagnostic, color && !short));
        if color && !short {
            let file = &self.files[&diagnostic.file_id];
            let (line, column) = file.location(diagnostic.range.start());
            let tabs = file
                .line(line)
                .chars()
                .take(column - 1)
                .filter(|c| *c == '\t')
                .count();
            if tabs != 0 {
                rendered = rendered.replacen(
                    &format!("{}:{line}:{}", file.path, column + tabs * 3),
                    &format!("{}:{line}:{column}", file.path),
                    1,
                );
            }
        }
        rendered.push('\n');
        if !short {
            rendered.push('\n');
        }
        rendered
    }

    fn span(
        &self,
        file_id: FileId,
        range: TextRange,
        primary: bool,
        label: Option<&str>,
        replacement: Option<(&str, Applicability)>,
    ) -> Value {
        let file = &self.files[&file_id];
        let (line_start, column_start) = file.location(range.start());
        let (line_end, column_end) = file.location(range.end());
        let text: Vec<_> = (line_start..=line_end).map(|line| {
            let text = file.line(line);
            json!({
                "text": text,
                "highlight_start": if line == line_start { column_start } else { 1 },
                "highlight_end": if line == line_end { column_end } else { text.chars().count() + 1 },
            })
        }).collect();
        json!({
            "file_name": file.path,
            "byte_start": u32::from(range.start()), "byte_end": u32::from(range.end()),
            "line_start": line_start, "line_end": line_end,
            "column_start": column_start, "column_end": column_end,
            "is_primary": primary, "text": text, "label": label,
            "suggested_replacement": replacement.map(|(text, _)| text),
            "suggestion_applicability": replacement.map(|(_, applicability)| format!("{applicability:?}")),
            "expansion": null,
        })
    }

    pub(super) fn json(&self, diagnostic: &Diagnostic) -> Value {
        let mut spans = vec![self.span(
            diagnostic.file_id,
            diagnostic.range,
            true,
            Some(&diagnostic.label),
            None,
        )];
        if let Some((range, message)) = &diagnostic.boundary {
            spans.push(self.span(diagnostic.file_id, *range, false, Some(message), None));
        }
        let mut children = Vec::new();
        if !diagnostic.help.is_empty() {
            children.push(child("help", &diagnostic.help, Vec::new()));
        }
        children.extend(
            diagnostic
                .notes
                .iter()
                .map(|note| child("note", note, Vec::new())),
        );
        for suggestion in &diagnostic.suggestions {
            let spans = suggestion
                .edits
                .iter()
                .map(|edit| {
                    self.span(
                        edit.file_id,
                        edit.range,
                        true,
                        None,
                        Some((&edit.replacement, suggestion.applicability)),
                    )
                })
                .collect();
            children.push(child("help", &suggestion.message, spans));
        }
        json!({
            "reason": "compiler-message",
            "package_id": "cargo-tessera tessera-checkers",
            "target": {
                "kind": ["custom-build"], "crate_types": ["bin"], "name": "tessera-checkers",
                "src_path": self.root.join(&self.files[&diagnostic.file_id].path),
                "edition": "2024", "doctest": false, "test": false, "doc": false,
            },
            "message": {
                "message": diagnostic.message,
                "code": { "code": diagnostic.code.to_string(), "explanation": diagnostic.code.explanation() },
                "level": diagnostic.severity, "spans": spans, "children": children,
                "rendered": self.render(diagnostic, false, false),
            }
        })
    }

    pub(super) fn emit(&self, diagnostic: &Diagnostic, format: MessageFormat) {
        if format.is_json() {
            println!("{}", self.json(diagnostic));
        } else {
            eprint!(
                "{}",
                self.render(
                    diagnostic,
                    format == MessageFormat::Short,
                    console::colors_enabled_stderr()
                )
            );
        }
    }
}
