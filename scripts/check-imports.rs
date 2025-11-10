#!/usr/bin/env rust-script
//!
//! This script checks and fixes `use` statements in Rust source files
//! to follow specific grouping and sorting rules. It respects .gitignore files.
//!
//! The rules are as follows:
//! 1. Grouped into four categories, in the following strict order:
//!    - Group 1: Standard library (`std`, `core`, `alloc`)
//!    - Group 2: Third-party crates
//!    - Group 3: The crate root (`crate::`)
//!    - Group 4: Submodules of the crate (`super::`, `self::`)
//! 2. There must be exactly one blank line between different groups.
//! 3. Imports within the same group must be contiguous, with no blank lines.
//! 4. Imports within each group must be sorted alphabetically.
//! 5. Imports from the same root path should be merged into a single `use` statement.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! syn = { version = "2.0", features = ["full", "extra-traits", "parsing"] }
//! anyhow = "1.0"
//! colored = "2.1"
//! proc-macro2 = { version = "1.0", features = ["proc-macro", "span-locations"] }
//! itertools = "0.10"
//! quote = "1.0"
//! clap = { version = "4.0", features = ["derive"] }
//! ignore = "0.4"
//! rayon = "1.10.0"
//! ```

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

use anyhow::{Result, bail};
use clap::Parser;
use colored::Colorize;
use ignore::WalkBuilder;
use itertools::Itertools;
use proc_macro2::Span;
use quote::quote;
use rayon::prelude::*;
use syn::{Expr, File, Item, Lit, Meta, UseTree, Visibility, spanned::Spanned};

/// Checks and fixes `use` statements in Rust files and directories, respecting .gitignore.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, bin_name = "rust-script scripts/check-imports.rs")]
struct Cli {
    /// Automatically fixes formatting issues.
    #[arg(short, long)]
    fix: bool,

    /// The list of files and/or directories to check or fix.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
}

/// Defines the four categories for imports.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
enum ImportCategory {
    Std,
    Extern,
    Crate,
    Local,
}

/// Stores information about a parsed `use` statement (for checking).
#[derive(Debug, Clone)]
struct UseItemInfo {
    category: ImportCategory,
    sort_key: String,
    span: Span,
    tree: UseTree,
    visibility: Visibility,
    attrs: Vec<syn::Attribute>,
}

/// Stores information about a flattened `use` import (for fixing).
#[derive(Debug, Clone, Eq, PartialEq)]
struct Import {
    attrs: String,
    category: ImportCategory,
    visibility: String,
    path: String,
}

impl PartialOrd for Import {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Import {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.attrs
            .cmp(&other.attrs)
            .then_with(|| self.category.cmp(&other.category))
            .then_with(|| self.visibility.cmp(&other.visibility))
            .then_with(|| self.path.cmp(&other.path))
    }
}

#[derive(Default, Debug)]
struct UseNode {
    children: BTreeMap<String, UseNode>,
    is_terminal: bool,
}

impl UseNode {
    fn insert(&mut self, path: &[String]) {
        if let Some((first, rest)) = path.split_first() {
            self.children.entry(first.clone()).or_default().insert(rest);
        } else {
            self.is_terminal = true;
        }
    }

    fn format(&self) -> String {
        let mut parts = Vec::new();
        if self.is_terminal {
            parts.push("self".to_string());
        }

        for (name, child) in &self.children {
            if child.is_terminal && child.children.is_empty() {
                parts.push(name.clone());
            } else {
                parts.push(format!("{}::{{{}}}", name, child.format()));
            }
        }
        parts.join(", ")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let files_to_process = collect_rs_files(&cli.paths);

    let failed_items = Arc::new(Mutex::new(Vec::new()));
    let print_mutex = Arc::new(Mutex::new(()));

    // Collect all file paths that need rustfmt
    let mut fmt_targets = Vec::new();

    for file_path in &files_to_process {
        match process_file(
            file_path,
            cli.fix,
            &mut fmt_targets,
            &failed_items,
            &print_mutex,
        ) {
            Ok(result) => {
                let _lock = print_mutex.lock().unwrap();
                let icon = if result.contains("Fixed") {
                    "🔧"
                } else {
                    "✅"
                };
                println!("{} {} - {}", icon, result, file_path.display());
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                drop(print_mutex.lock().unwrap());
                println!("❌ {} - {}: {}", "Error".red(), file_path.display(), e);
                io::stdout().flush().unwrap();
                failed_items.lock().unwrap().push(file_path.clone());
                continue;
            }
        }
    }

    let failed_items = Arc::try_unwrap(failed_items).unwrap().into_inner().unwrap();
    if !failed_items.is_empty() {
        bail!("Failed to process {} files.", failed_items.len());
    }

    // Batch format all files that need it
    run_rustfmt_if_needed(cli.fix, &fmt_targets)?;

    Ok(())
}

fn run_rustfmt_if_needed(cli_fix: bool, fmt_targets: &[PathBuf]) -> Result<()> {
    if cli_fix && !fmt_targets.is_empty() {
        let mut cmd = Command::new("rustfmt");
        cmd.arg("--edition=2024");
        for path in fmt_targets {
            cmd.arg(path);
        }
        let status = cmd.status()?;
        if !status.success() {
            bail!("Failed to run rustfmt on files: {:?}", fmt_targets);
        }
    }
    Ok(())
}

fn process_file(
    file_path: &Path,
    cli_fix: bool,
    fmt_targets: &mut Vec<PathBuf>,
    _failed_items: &Arc<Mutex<Vec<PathBuf>>>,
    _print_mutex: &Arc<Mutex<()>>,
) -> Result<String> {
    if cli_fix {
        match fix_file(file_path) {
            Ok(changed) => {
                fmt_targets.push(file_path.to_path_buf());
                if changed {
                    Ok("Fixed".cyan().to_string())
                } else {
                    Ok("Correctly formatted".green().to_string())
                }
            }
            Err(e) => Err(e),
        }
    } else {
        match check_file(file_path) {
            Ok(_) => Ok("Check passed".green().to_string()),
            Err(e) => Err(e),
        }
    }
}

fn collect_rs_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .par_iter()
        .flat_map(|path| {
            WalkBuilder::new(path)
                .build()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
                .map(|e| e.into_path())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Build a normalized imports string for `ast`, identify the original line numbers
/// occupied by `use` items, and return the insertion point (first_use_line).
/// Return value:
///  - String: the formatted imports block to insert
///  - Vec<usize>: original line numbers to remove
///  - usize: the first line where `use` items appeared (insertion anchor)
fn build_new_imports_and_lines(ast: &File) -> Result<(String, Vec<usize>, usize)> {
    let use_items: Vec<_> = ast
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Use(use_item) = item {
                Some(use_item)
            } else {
                None
            }
        })
        .collect();

    if use_items.is_empty() {
        return Ok(("".to_string(), Vec::new(), 0));
    }

    // Compute local modules once for classification helpers.
    let local_mods = find_local_modules(ast);
    let new_imports_str = collect_and_format_imports(ast, &local_mods)?;

    let lines_to_remove = collect_use_item_lines(&use_items);

    let first_use_line = use_items
        .iter()
        .map(|item| item.span().start().line)
        .min()
        .unwrap_or(0);

    Ok((
        new_imports_str,
        lines_to_remove.into_iter().collect(),
        first_use_line,
    ))
}

/// Collects imports from the AST, deduplicates and sorts them, then formats
/// the final imports block for insertion. This function centralizes the
/// collect -> sort -> dedup -> format pipeline.
fn collect_and_format_imports(ast: &syn::File, local_mods: &HashSet<String>) -> Result<String> {
    let mut collected_imports = collect_imports(ast, local_mods)?;
    collected_imports.sort();
    collected_imports.dedup();
    Ok(format_imports_from_collected(collected_imports)
        .trim()
        .to_string())
}

/// Replace the `use` statements in `path` with the normalized imports block.
/// This function:
///  - Parses the file to an AST
///  - Builds the new imports block and identifies original lines to remove
///  - Reconstructs file contents by skipping removed lines and inserting the
///    new imports block at the original first `use` line (or appending if needed)
///  - Writes the updated file back to disk
///
/// Returns Ok(true) if the file was modified, Ok(false) if there were no imports
/// to rewrite, or Err on IO / parse errors.
fn fix_file(path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    let ast = syn::parse_file(&content)?;

    let (new_imports_str, mut lines_to_remove, first_use_line) = build_new_imports_and_lines(&ast)?;

    if new_imports_str.is_empty() {
        return Ok(false);
    }

    let lines_to_remove_set: HashSet<usize> = lines_to_remove.drain(..).collect();
    let original_lines: Vec<&str> = content.lines().collect();
    let mut final_lines: Vec<String> = Vec::with_capacity(original_lines.len() + 4);
    let mut new_imports_written = false;

    for (i, line) in original_lines.iter().enumerate() {
        let current_line_num = i + 1;

        if lines_to_remove_set.contains(&current_line_num) {
            if !new_imports_written && current_line_num >= first_use_line {
                final_lines.push(new_imports_str.clone());
                new_imports_written = true;
            }
        } else {
            final_lines.push(line.to_string());
        }
    }

    // In case the file only contains `use` statements
    if !new_imports_written {
        final_lines.push(new_imports_str);
    }

    let final_content = final_lines.join("\n") + "\n";

    fs::write(path, &final_content)?;

    Ok(true)
}

/// Format a sequence of flattened `Import` entries into the canonical `use`
/// block string. This function groups imports by attributes (doc / cfg / etc),
/// then by (category, pub) to maintain the required group ordering, and finally
/// merges imports with the same root path into `{}` groups when possible.
/// Format a sequence of flattened `Import` entries into the canonical `use`
/// block string. This implementation delegates two responsibilities to focused
/// helpers:
///  - `format_with_attrs` handles imports that carry attributes (doc/cfg/etc)
///  - `format_without_attrs` handles the common case: grouping by (category, is_pub)
///    then merging root paths into `{}` groups via `merge_path_groups`.
fn format_imports_from_collected(imports: Vec<Import>) -> String {
    // Helper for imports that have attributes (keeps each as its own `use` line).
    fn format_with_attrs(attrs: &str, imports: &[Import]) -> String {
        imports
            .iter()
            .map(|import| {
                let keyword = if import.visibility.is_empty() {
                    "use".to_string()
                } else {
                    format!("{} use", import.visibility)
                };
                format!("{}\n{} {};", attrs, keyword, import.path)
            })
            .join("\n")
    }

    // Merge a group of imports that share the same root into `root::{...}` forms.
    fn merge_path_groups(group: impl IntoIterator<Item = Import>, visibility: &str) -> String {
        let mut path_groups: BTreeMap<String, UseNode> = BTreeMap::new();
        for import in group {
            let path_parts: Vec<_> = import.path.split("::").map(String::from).collect();
            if let Some((root, rest)) = path_parts.split_first() {
                path_groups.entry(root.clone()).or_default().insert(rest);
            }
        }

        path_groups
            .into_iter()
            .map(|(root, node)| {
                let keyword = if visibility.is_empty() {
                    "use".to_string()
                } else {
                    format!("{} use", visibility)
                };
                if node.is_terminal && node.children.is_empty() {
                    format!("{} {};", keyword, root)
                } else {
                    format!("{} {}::{{{}}};", keyword, root, node.format())
                }
            })
            .join("\n")
    }

    // Format the imports that don't have attributes by grouping and merging roots.
    fn format_without_attrs(imports: Vec<Import>) -> String {
        imports
            .into_iter()
            .group_by(|import| (import.category, import.visibility.clone()))
            .into_iter()
            .sorted_by_key(|(key, _)| key.clone())
            .map(|((_category, visibility), group)| merge_path_groups(group, &visibility))
            .join("\n\n")
    }

    imports
        .into_iter()
        .group_by(|import| import.attrs.clone())
        .into_iter()
        .map(|(attrs, group)| {
            let imports: Vec<_> = group.collect();
            if !attrs.is_empty() {
                return format_with_attrs(&attrs, &imports);
            }
            format_without_attrs(imports)
        })
        .join("\n\n")
}

fn get_path_idents(tree: &UseTree) -> Vec<&syn::Ident> {
    let mut idents = Vec::new();
    let mut current_tree = tree;
    while let UseTree::Path(path) = current_tree {
        idents.push(&path.ident);
        current_tree = &path.tree;
    }
    if let UseTree::Name(name) = current_tree {
        idents.push(&name.ident);
    }
    idents
}

fn format_use_tree(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(p) => format!("{}::{}", p.ident, format_use_tree(&p.tree)),
        UseTree::Name(n) => n.ident.to_string(),
        UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        UseTree::Glob(_) => "*".to_string(),
        UseTree::Group(g) => {
            let items = g.items.iter().map(format_use_tree).join(", ");
            format!("{{{}}}", items)
        }
    }
}

/// Join a prefix of idents into a `::` separated path string.
/// Extracted to remove duplicated `.iter().map(|s| s.to_string()).join("::")` uses.
fn prefix_to_string(prefix: &Vec<&syn::Ident>) -> String {
    prefix.iter().map(|s| s.to_string()).join("::")
}

/// Format a visibility modifier into a string representation.
/// - `Visibility::Public` -> "pub"
/// - `Visibility::Restricted` -> "pub(crate)", "pub(super)", etc.
/// - `Visibility::Inherited` -> "" (empty string for private)
fn format_visibility(vis: &Visibility) -> String {
    match vis {
        Visibility::Public(_) => "pub".to_string(),
        Visibility::Restricted(r) => quote!(#r).to_string(),
        Visibility::Inherited => String::new(),
    }
}

/// Format a slice of attributes into the string representation used by the
/// import formatting pipeline.
///
/// - `doc` attributes are converted into `///` or `//!` style comments preserving
///   inner/outer style.
/// - Other attributes are stringified via `quote!(#attr).to_string()`.
fn format_attrs(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .map(|attr| {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(nv) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            let comment_content = lit_str.value();
                            return if matches!(attr.style, syn::AttrStyle::Inner(_)) {
                                format!("//! {}", comment_content.trim())
                            } else {
                                format!("/// {}", comment_content.trim())
                            };
                        }
                    }
                }
            }
            quote!(#attr).to_string()
        })
        .join("\n")
}

fn collect_imports(ast: &File, local_mods: &HashSet<String>) -> Result<Vec<Import>> {
    let mut imports = Vec::new();
    for item in &ast.items {
        if let Item::Use(use_item) = item {
            let visibility = format_visibility(&use_item.vis);
            let attrs = format_attrs(&use_item.attrs);

            if use_item.attrs.is_empty() {
                collect_paths_from_tree(&use_item.tree, vec![], &mut |path_str, path_idents| {
                    let category = classify_path(&path_idents, local_mods);
                    imports.push(Import {
                        attrs: attrs.clone(),
                        visibility: visibility.clone(),
                        category,
                        path: path_str,
                    });
                });
            } else {
                let category = classify_path(&get_path_idents(&use_item.tree), local_mods);
                imports.push(Import {
                    attrs,
                    visibility,
                    category,
                    path: format_use_tree(&use_item.tree),
                });
            }
        }
    }
    Ok(imports)
}

/// Recursively traverse a `UseTree` producing flattened path strings and the
/// corresponding identifier prefixes. The callback receives (path_string, idents).
/// Implementation preserves the original behaviour but documents the shape of
/// recursion and explains why cloning of `prefix` is performed in some branches.
fn collect_paths_from_tree<'a>(
    tree: &'a UseTree,
    prefix: Vec<&'a syn::Ident>,
    callback: &mut dyn FnMut(String, Vec<&'a syn::Ident>),
) {
    match tree {
        UseTree::Path(path) => {
            let mut current_prefix = prefix.clone();
            current_prefix.push(&path.ident);
            collect_paths_from_tree(&path.tree, current_prefix, callback);
        }
        UseTree::Name(name) => {
            let mut current_prefix = prefix;
            let path_str = if name.ident == "self" {
                prefix_to_string(&current_prefix)
            } else {
                current_prefix.push(&name.ident);
                prefix_to_string(&current_prefix)
            };
            if !path_str.is_empty() {
                callback(path_str, current_prefix);
            }
        }
        UseTree::Rename(rename) => {
            let mut current_prefix = prefix.clone();
            current_prefix.push(&rename.ident);
            let path_str = format!(
                "{} as {}",
                prefix_to_string(&current_prefix),
                rename.rename.to_string()
            );
            callback(path_str, current_prefix);
        }
        UseTree::Glob(_) => {
            let path_str = if prefix.is_empty() {
                "*".to_string()
            } else {
                format!("{}::*", prefix_to_string(&prefix))
            };
            if !prefix.is_empty() {
                callback(path_str, prefix);
            }
        }
        UseTree::Group(group) => {
            for t in &group.items {
                collect_paths_from_tree(t, prefix.clone(), callback);
            }
        }
    }
}

fn check_file(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let ast = syn::parse_file(&content)?;
    let local_mods = find_local_modules(&ast);
    let use_items = collect_and_classify_uses(&ast, &local_mods)?;

    if use_items.is_empty() {
        return Ok(());
    }

    check_group_order(&use_items)?;
    check_intra_group_sorting(&use_items)?;
    check_blank_lines(&use_items)?;
    check_merge_status(&use_items)?;

    Ok(())
}

fn check_merge_status(items: &[UseItemInfo]) -> Result<()> {
    for i in 1..items.len() {
        let prev = &items[i - 1];
        let curr = &items[i];

        if can_merge(prev, curr) {
            if let (Some(prev_root), Some(curr_root)) =
                (get_path_root(&prev.tree), get_path_root(&curr.tree))
            {
                if prev_root == curr_root {
                    bail!(
                        "Line {}: Imports can be merged. `use {}...` should be merged with the previous line.",
                        curr.span.start().line,
                        curr_root
                    );
                }
            }
        }
    }
    Ok(())
}

fn get_path_root(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        _ => None,
    }
}

fn can_merge(prev: &UseItemInfo, curr: &UseItemInfo) -> bool {
    prev.attrs.is_empty()
        && curr.attrs.is_empty()
        && prev.category == curr.category
        && curr.span.start().line == prev.span.end().line + 1
}

fn intra_group_out_of_order(prev: &UseItemInfo, curr: &UseItemInfo) -> bool {
    curr.attrs.is_empty()
        && prev.attrs.is_empty()
        && curr.category == prev.category
        && format_visibility(&curr.visibility) == format_visibility(&prev.visibility)
        && curr.sort_key < prev.sort_key
}

fn find_local_modules(ast: &File) -> HashSet<String> {
    ast.items
        .iter()
        .filter_map(|item| {
            if let Item::Mod(mod_item) = item {
                Some(mod_item.ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Collect all identifier-path vectors from a `use` item's tree.
/// Returns a Vec of Vecs where each inner Vec is the sequence of idents for a path.
/// Example: for `use a::{b, c::d};` this returns `[["a","b"], ["a","c","d"]]`.
fn collect_use_paths(tree: &UseTree) -> Vec<Vec<&syn::Ident>> {
    let mut paths = Vec::new();
    collect_paths_from_tree(tree, vec![], &mut |_, path_idents| paths.push(path_idents));
    paths
}

fn collect_use_item_lines(use_items: &[&syn::ItemUse]) -> HashSet<usize> {
    // Collect all line numbers occupied by `use` items and their attributes.
    // This ensures that when we replace the imports block we remove the entire
    // original `use` statements including any preceding attributes (e.g. cfg/doc).
    let mut lines = HashSet::new();
    for item in use_items {
        // Include attribute lines attached to the use item.
        for attr in &item.attrs {
            let start = attr.span().start().line;
            let end = attr.span().end().line;
            for ln in start..=end {
                lines.insert(ln);
            }
        }
        // Include the lines spanned by the use item itself.
        let start = item.span().start().line;
        let end = item.span().end().line;
        for ln in start..=end {
            lines.insert(ln);
        }
    }
    lines
}

/// Build a single UseItemInfo from the original use item and the collected paths.
fn build_use_item_info(
    use_item: &syn::ItemUse,
    paths: Vec<Vec<&syn::Ident>>,
    local_mods: &HashSet<String>,
) -> Option<UseItemInfo> {
    if paths.is_empty() {
        return None;
    }
    let category = classify_path(&paths[0], local_mods);
    let sort_key = paths
        .iter()
        .map(|p| {
            p.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("::")
        })
        .min()
        .unwrap_or_default();

    Some(UseItemInfo {
        category,
        sort_key,
        span: use_item.span(),
        tree: use_item.tree.clone(),
        visibility: use_item.vis.clone(),
        attrs: use_item.attrs.clone(),
    })
}

fn collect_and_classify_uses(ast: &File, local_mods: &HashSet<String>) -> Result<Vec<UseItemInfo>> {
    let mut final_items = Vec::new();
    for item in &ast.items {
        if let Item::Use(use_item) = item {
            let paths = collect_use_paths(&use_item.tree);
            if let Some(info) = build_use_item_info(use_item, paths, local_mods) {
                final_items.push(info);
            }
        }
    }
    Ok(final_items)
}

fn classify_path<'a>(
    path_segments: &[&'a syn::Ident],
    local_mods: &HashSet<String>,
) -> ImportCategory {
    if let Some(first_seg) = path_segments.first() {
        let first_seg_str = first_seg.to_string();
        match first_seg_str.as_str() {
            "std" | "core" | "alloc" => ImportCategory::Std,
            "crate" => ImportCategory::Crate,
            "super" | "self" => ImportCategory::Local,
            _ if local_mods.contains(&first_seg_str) => ImportCategory::Local,
            _ if first_seg_str
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_lowercase()) =>
            {
                ImportCategory::Extern
            }
            _ => ImportCategory::Local,
        }
    } else {
        ImportCategory::Extern
    }
}

fn check_group_order(items: &[UseItemInfo]) -> Result<()> {
    for i in 1..items.len() {
        if items[i].attrs.is_empty() && items[i - 1].attrs.is_empty() {
            if items[i].category < items[i - 1].category {
                bail!(
                    "Line {}: Incorrect import group order. `{}` should not come after `{}`.",
                    items[i].span.start().line,
                    items[i].sort_key,
                    items[i - 1].sort_key,
                );
            }
        }
    }
    Ok(())
}

fn check_intra_group_sorting(items: &[UseItemInfo]) -> Result<()> {
    for i in 1..items.len() {
        let prev = &items[i - 1];
        let curr = &items[i];
        if intra_group_out_of_order(prev, curr) {
            bail!(
                "Line {}: Imports within a group are not sorted alphabetically. `{}` should not come after `{}`.",
                curr.span.start().line,
                curr.sort_key,
                prev.sort_key,
            );
        }
    }
    Ok(())
}

fn check_blank_lines(items: &[UseItemInfo]) -> Result<()> {
    for i in 1..items.len() {
        let prev = &items[i - 1];
        let curr = &items[i];

        if prev.attrs.is_empty() && curr.attrs.is_empty() {
            let prev_end_line = prev.span.end().line;
            let curr_start_line = curr.span.start().line;

            if curr.category != prev.category
                || format_visibility(&curr.visibility) != format_visibility(&prev.visibility)
            {
                if curr_start_line != prev_end_line + 2 {
                    bail!(
                        "Line {}: A blank line is required between groups. Incorrect spacing between `{}` and `{}`.",
                        curr.span.start().line,
                        prev.sort_key,
                        curr.sort_key
                    );
                }
            } else {
                if curr_start_line > prev_end_line + 1 {
                    bail!(
                        "Line {}: No blank lines are allowed within the same import group. Incorrect spacing between `{}` and `{}`.",
                        curr.span.start().line,
                        prev.sort_key,
                        curr.sort_key
                    );
                }
            }
        }
    }
    Ok(())
}
