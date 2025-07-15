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
use syn::{File, Item, UseTree, Visibility, spanned::Spanned};

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
    is_pub: bool,
    attrs: Vec<syn::Attribute>,
}

/// Stores information about a flattened `use` import (for fixing).
#[derive(Debug, Clone, Eq, PartialEq)]
struct Import {
    attrs: String,
    category: ImportCategory,
    is_pub: bool,
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
            .then_with(|| self.is_pub.cmp(&other.is_pub))
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

    let files_to_process: Vec<PathBuf> = cli
        .paths
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
        .collect();

    let failed_items = Arc::new(Mutex::new(Vec::new()));
    let print_mutex = Arc::new(Mutex::new(()));

    files_to_process.into_par_iter().for_each(|file_path| {
        let result = if cli.fix {
            fix_file(&file_path)
                .map(|changed| {
                    if changed {
                        "Fixed".cyan().to_string()
                    } else {
                        "Correctly formatted".green().to_string()
                    }
                })
                .map_err(|e| format!("Failed to fix: {}", e))
        } else {
            check_file(&file_path)
                .map(|_| "Check passed".green().to_string())
                .map_err(|e| format!("Check failed: {}", e))
        };

        // Print result immediately after processing each file
        let _lock = print_mutex.lock().unwrap();
        match result {
            Ok(status) => {
                let icon = if status.contains("Fixed") {
                    "🔧"
                } else {
                    "✅"
                };
                println!("{} {} - {}", icon, status, file_path.display());
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                println!("❌ {} - {}: {}", "Error".red(), file_path.display(), e);
                io::stdout().flush().unwrap();
                failed_items.lock().unwrap().push(file_path);
            }
        }
    });

    let failed_items = Arc::try_unwrap(failed_items).unwrap().into_inner().unwrap();
    if !failed_items.is_empty() {
        bail!("Failed to process {} files.", failed_items.len());
    }

    Ok(())
}

fn fix_file(path: &Path) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    let ast = syn::parse_file(&content)?;

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
        return Ok(false);
    }

    let local_mods = find_local_modules(&ast);
    let mut collected_imports = collect_imports(&ast, &local_mods)?;
    collected_imports.sort();
    collected_imports.dedup();
    let new_imports_str = format_imports_from_collected(collected_imports)
        .trim()
        .to_string();

    let mut lines_to_remove = HashSet::new();
    for use_item in &use_items {
        for line in use_item.span().start().line..=use_item.span().end().line {
            lines_to_remove.insert(line);
        }
    }

    let first_use_line = use_items
        .iter()
        .map(|item| item.span().start().line)
        .min()
        .unwrap_or(0);

    let original_lines: Vec<&str> = content.lines().collect();
    let mut final_lines: Vec<String> = Vec::new();
    let mut new_imports_written = false;

    for (i, line) in original_lines.iter().enumerate() {
        let current_line_num = i + 1;

        if lines_to_remove.contains(&current_line_num) {
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

    let rustfmt_status = Command::new("rustfmt")
        .arg("--edition=2024")
        .arg(path)
        .status()?;
    if !rustfmt_status.success() {
        bail!("Failed to run `rustfmt` on the file: {:?}", path);
    }

    Ok(true)
}

fn format_imports_from_collected(imports: Vec<Import>) -> String {
    imports
        .into_iter()
        .group_by(|import| import.attrs.clone())
        .into_iter()
        .map(|(attrs, group)| {
            let imports: Vec<_> = group.collect();
            if !attrs.is_empty() {
                let items = imports
                    .iter()
                    .map(|import| {
                        let keyword = if import.is_pub { "pub use" } else { "use" };
                        format!("{} {};", keyword, import.path)
                    })
                    .join("\n");
                return format!("{}\n{}", attrs, items);
            }
            imports
                .into_iter()
                .group_by(|import| (import.category, import.is_pub))
                .into_iter()
                .sorted_by_key(|(key, _)| *key)
                .map(|((_category, is_pub), group)| {
                    let mut path_groups: BTreeMap<String, UseNode> = BTreeMap::new();
                    for import in group {
                        let path_parts: Vec<_> =
                            import.path.split("::").map(String::from).collect();
                        if let Some((root, rest)) = path_parts.split_first() {
                            path_groups.entry(root.clone()).or_default().insert(rest);
                        }
                    }

                    path_groups
                        .into_iter()
                        .map(|(root, node)| {
                            let keyword = if is_pub { "pub use" } else { "use" };
                            if node.is_terminal && node.children.is_empty() {
                                return format!("{} {};", keyword, root);
                            }
                            format!("{} {}::{{{}}};", keyword, root, node.format())
                        })
                        .join("\n")
                })
                .join("\n\n")
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

fn collect_imports(ast: &File, local_mods: &HashSet<String>) -> Result<Vec<Import>> {
    let mut imports = Vec::new();
    for item in &ast.items {
        if let Item::Use(use_item) = item {
            let is_pub = matches!(use_item.vis, Visibility::Public(_));
            let attrs = use_item
                .attrs
                .iter()
                .map(|attr| quote!(#attr).to_string())
                .join("\n");

            if use_item.attrs.is_empty() {
                collect_paths_from_tree(&use_item.tree, vec![], &mut |path_str, path_idents| {
                    let category = classify_path(&path_idents, local_mods);
                    imports.push(Import {
                        attrs: attrs.clone(),
                        is_pub,
                        category,
                        path: path_str,
                    });
                });
            } else {
                let category = classify_path(&get_path_idents(&use_item.tree), local_mods);
                imports.push(Import {
                    attrs,
                    is_pub,
                    category,
                    path: format_use_tree(&use_item.tree),
                });
            }
        }
    }
    Ok(imports)
}

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
                current_prefix.iter().map(|s| s.to_string()).join("::")
            } else {
                current_prefix.push(&name.ident);
                current_prefix.iter().map(|s| s.to_string()).join("::")
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
                current_prefix.iter().map(|s| s.to_string()).join("::"),
                rename.rename.to_string()
            );
            callback(path_str, current_prefix);
        }
        UseTree::Glob(_) => {
            let path_str = if prefix.is_empty() {
                "*".to_string()
            } else {
                format!("{}::*", prefix.iter().map(|s| s.to_string()).join("::"))
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

        if prev.attrs.is_empty()
            && curr.attrs.is_empty()
            && prev.category == curr.category
            && curr.span.start().line == prev.span.end().line + 1
        {
            if let Some(prev_root) = get_path_root(&prev.tree) {
                if let Some(curr_root) = get_path_root(&curr.tree) {
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
    }
    Ok(())
}

fn get_path_root(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        _ => None,
    }
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

fn collect_and_classify_uses(ast: &File, local_mods: &HashSet<String>) -> Result<Vec<UseItemInfo>> {
    let mut final_items = Vec::new();
    for item in &ast.items {
        if let Item::Use(use_item) = item {
            let is_pub = matches!(use_item.vis, Visibility::Public(_));
            let mut paths = Vec::new();
            collect_paths_from_tree(&use_item.tree, vec![], &mut |_, path_idents| {
                paths.push(path_idents)
            });

            if let Some(first_path) = paths.first() {
                let category = classify_path(first_path, local_mods);
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

                final_items.push(UseItemInfo {
                    category,
                    sort_key,
                    span: use_item.span(),
                    tree: use_item.tree.clone(),
                    is_pub,
                    attrs: use_item.attrs.clone(),
                });
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
        if curr.attrs.is_empty()
            && prev.attrs.is_empty()
            && curr.category == prev.category
            && curr.is_pub == prev.is_pub
            && curr.sort_key < prev.sort_key
        {
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

            if curr.category != prev.category || curr.is_pub != prev.is_pub {
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
