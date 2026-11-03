use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use codespan_reporting::{
    diagnostic::{Diagnostic as Message, Label, LabelStyle},
    files::{Files, SimpleFiles},
    term::{self, Chars, Config, DisplayStyle, Styles, StylesWriter, termcolor::Buffer},
};
use ra_ap_base_db::SourceDatabase;
use ra_ap_ide::FileId;
use serde_json::{Value, json};

use super::types::{CallAnalyzer, Diagnostic, MessageFormat};

pub(super) struct Reporter {
    files: SimpleFiles<String, String>,
    ids: HashMap<FileId, usize>,
    root: PathBuf,
}

impl Reporter {
    pub(super) fn new(analyzer: &CallAnalyzer<'_>, diagnostics: &[Diagnostic]) -> Self {
        let mut files = SimpleFiles::new();
        let mut ids = HashMap::new();
        for diagnostic in diagnostics {
            ids.entry(diagnostic.file_id).or_insert_with(|| {
                let path = analyzer.vfs.file_path(diagnostic.file_id);
                let name = path.as_path().map_or_else(
                    || path.to_string(),
                    |path| {
                        let path: &std::path::Path = path.as_ref();
                        path.strip_prefix(&analyzer.display_root)
                            .unwrap_or(path)
                            .display()
                            .to_string()
                    },
                );
                files.add(
                    name,
                    analyzer
                        .db
                        .file_text(diagnostic.file_id)
                        .text(analyzer.db)
                        .to_string(),
                )
            });
        }
        Self {
            files,
            ids,
            root: analyzer.display_root.clone(),
        }
    }

    fn message(&self, diagnostic: &Diagnostic) -> Message<usize> {
        let id = self.ids[&diagnostic.file_id];
        let range = |range: ra_ap_span::TextRange| {
            u32::from(range.start()) as usize..u32::from(range.end()) as usize
        };
        let mut labels =
            vec![Label::primary(id, range(diagnostic.range)).with_message(&diagnostic.label)];
        if let Some((boundary, message)) = &diagnostic.boundary {
            labels.push(Label::secondary(id, range(*boundary)).with_message(message));
        }
        Message::error()
            .with_code(diagnostic.code.to_string())
            .with_message(&diagnostic.message)
            .with_labels(labels)
            .with_notes(vec![format!("help: {}", diagnostic.help)])
    }

    pub(super) fn render(
        &self,
        diagnostic: &Diagnostic,
        short: bool,
        color: bool,
    ) -> Result<String> {
        let config = Config {
            chars: Chars::ascii(),
            display_style: if short {
                DisplayStyle::Short
            } else {
                DisplayStyle::Rich
            },
            ..Config::default()
        };
        let mut buffer = if color {
            Buffer::ansi()
        } else {
            Buffer::no_color()
        };
        term::emit_to_write_style(
            &mut StylesWriter::new(&mut buffer, &Styles::default()),
            &config,
            &self.files,
            &self.message(diagnostic),
        )?;
        Ok(String::from_utf8(buffer.into_inner())?)
    }

    pub(super) fn json(&self, diagnostic: &Diagnostic) -> Result<Value> {
        let message = self.message(diagnostic);
        let spans = message.labels.iter().map(|label| {
            let start = self.files.location(label.file_id, label.range.start)?;
            let end = self.files.location(label.file_id, label.range.end)?;
            let source = self.files.source(label.file_id)?;
            let text = (start.line_number..=end.line_number).map(|line| {
                let range = self.files.line_range(label.file_id, line - 1)?;
                let text = source[range].trim_end_matches(['\r', '\n']);
                Ok(json!({
                    "text": text,
                    "highlight_start": if line == start.line_number { start.column_number } else { 1 },
                    "highlight_end": if line == end.line_number { end.column_number } else { text.chars().count() + 1 },
                }))
            }).collect::<Result<Vec<Value>>>()?;
            Ok(json!({
                "file_name": self.files.name(label.file_id)?,
                "byte_start": label.range.start,
                "byte_end": label.range.end,
                "line_start": start.line_number,
                "line_end": end.line_number,
                "column_start": start.column_number,
                "column_end": end.column_number,
                "is_primary": label.style == LabelStyle::Primary,
                "text": text,
                "label": label.message,
                "suggested_replacement": null,
                "suggestion_applicability": null,
                "expansion": null,
            }))
        }).collect::<Result<Vec<Value>>>()?;
        Ok(json!({
            "reason": "compiler-message",
            "package_id": "cargo-tessera tessera-checkers",
            "target": {
                "kind": ["custom-build"], "crate_types": ["bin"], "name": "tessera-checkers",
                "src_path": self.root.join(self.files.name(self.ids[&diagnostic.file_id])?),
                "edition": "2024", "doctest": false, "test": false, "doc": false,
            },
            "message": {
                "message": diagnostic.message,
                "code": { "code": diagnostic.code.to_string(), "explanation": diagnostic.code.explanation() },
                "level": "error",
                "spans": spans,
                "children": [{ "message": diagnostic.help, "code": null, "level": "help", "spans": [], "children": [], "rendered": null }],
                "rendered": self.render(diagnostic, false, false)?,
            }
        }))
    }

    pub(super) fn emit(&self, diagnostic: &Diagnostic, format: MessageFormat) -> Result<()> {
        if format.is_json() {
            println!("{}", self.json(diagnostic)?);
        } else {
            eprint!(
                "{}",
                self.render(
                    diagnostic,
                    format == MessageFormat::Short,
                    console::colors_enabled_stderr()
                )?
            );
        }
        Ok(())
    }
}
