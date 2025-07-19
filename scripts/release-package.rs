#!/usr/bin/env rust-script
//!
//! ```cargo
//! [dependencies]
//! clap = { version = "4.0", features = ["derive"] }
//! anyhow = "1.0"
//! toml_edit = "0.22"
//! chrono = "0.4"
//! tabled = "0.15"
//! diffy = "0.3"
//! colored = "2.1"
//! ```
//! release-package.rs
//! A release tool for tessera, similar to cargo-release but project-specific.

use std::{fs, process::Command};

use anyhow::{Result, bail};
use clap::{Parser, ValueEnum};
use colored::*;
use tabled::Tabled;
use toml_edit::{DocumentMut, value};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about,
    long_about = None,
    bin_name = "rust-script scripts/release-package.rs"
)]
struct Cli {
    /// Package to release
    #[arg(short, long)]
    package: String,

    /// Version bump type: major, minor, or patch
    #[arg(value_enum)]
    bump: BumpType,

    /// Actually perform the release
    #[arg(long)]
    execute: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

const GITHUB_OWNER: &str = "shadow3aaa";
const GITHUB_REPO: &str = "tessera";

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Parse workspace, find package path
    let workspace = Workspace::load("Cargo.toml")?;
    let package_path = workspace.find_package(&cli.package)?;
    let cargo_toml_path = package_path.join("Cargo.toml");

    // 2. Read and parse version number
    let mut doc = read_toml(&cargo_toml_path)?;
    let old_version = doc["package"]["version"].as_str().unwrap_or("");
    let new_version = bump_version(old_version, cli.bump)?;

    // Beautify package info output with emoji (no color) and colored text
    println!(
        "{} {}",
        "📦",
        format!("Package: {}", cli.package).yellow().bold()
    );
    println!(
        "{} {}",
        "📄",
        format!("Path: {}", cargo_toml_path.display()).cyan()
    );
    println!(
        "{} {}",
        "🕓",
        format!("Old version: {}", old_version).blue()
    );
    println!(
        "{} {}",
        "🆕",
        format!("New version: {}", new_version).green()
    );

    // 3. Find latest tag and related commits
    let latest_tag = find_latest_tag(&cli.package)?;
    let rel_path = package_path.to_string_lossy();
    let commits = if let Some(tag) = &latest_tag {
        collect_commits_since_tag(tag, &rel_path)?
    } else {
        // No tag, collect all history
        collect_commits_since_tag("", &rel_path)?
    };
    let changelog = generate_changelog(&new_version, &commits, latest_tag.as_deref(), &cli.package);

    // Replace all path dependencies of this package with version dependencies
    let package_versions = workspace.collect_versions()?;
    let modified_files = replace_path_with_version_in_workspace(
        &workspace,
        &cli.package,
        &old_version,
        &package_versions,
    )?;

    // Generate and write (or prepend) ChangeLog to <package>/CHANGELOG.md
    let changelog_path = package_path.join("CHANGELOG.md");
    let changelog_path_str = changelog_path.to_str().unwrap();
    let old_changelog = std::fs::read_to_string(&changelog_path).unwrap_or_default();
    let new_changelog = format!("{}\n{}", changelog, old_changelog);
    let dry_run = !cli.execute;
    write_or_preview_file(
        dry_run,
        changelog_path_str,
        &new_changelog,
        Some(&old_changelog),
    )?;
    run_or_preview_cmd(dry_run, "git", &["add", changelog_path_str])?;

    // 1. bump version number, commit, tag
    doc["package"]["version"] = value(new_version.clone());
    write_or_preview_file(
        dry_run,
        cargo_toml_path.to_str().unwrap(),
        &doc.to_string(),
        Some(&fs::read_to_string(&cargo_toml_path)?),
    )?;
    run_or_preview_cmd(
        dry_run,
        "git",
        &["add", cargo_toml_path.to_str().unwrap()],
    )?;
    let release_commit_msg = format!("release({}): v{}", cli.package, new_version);
    run_or_preview_cmd(dry_run, "git", &["commit", "-m", &release_commit_msg])?;
    let tag = format!("{}-v{}", cli.package, new_version);
    run_or_preview_cmd(dry_run, "git", &["tag", &tag])?;
    // Push commit and tag to remote
    run_or_preview_cmd(dry_run, "git", &["push"])?;
    run_or_preview_cmd(dry_run, "git", &["push", "--tags"])?;

    // 2. path->version dependency changes and temporary commit
    for (file, old, new) in &modified_files {
        write_or_preview_file(dry_run, file, new, Some(old))?;
        run_or_preview_cmd(dry_run, "git", &["add", file])?;
    }
    let temp_commit_msg = "chore: replace path dependencies with version for publish";
    run_or_preview_cmd(dry_run, "git", &["commit", "-m", temp_commit_msg])?;

    // 3. publish
    run_or_preview_cmd(dry_run, "cargo", &["publish", "-p", &cli.package])?;

    // 4. reset to tag
    run_or_preview_cmd(dry_run, "git", &["reset", "--hard", &tag])?;

    if dry_run {
        // File diff preview
        for (file, old, new) in &modified_files {
            write_or_preview_file(dry_run, file, new, Some(old))?;
        }
        // Remove ChangeLog Preview Start/End output
    } else {
        // Write back all modified Cargo.toml
        for (file, _old, new) in &modified_files {
            std::fs::write(file, new)?;
        }
        // Update main package's Cargo.toml (version number already written back)
        doc["package"]["version"] = value(new_version.clone());
        fs::write(&cargo_toml_path, doc.to_string())?;
        println!("Updated version in {}", cargo_toml_path.display());
    }

    Ok(())
}

fn run_or_preview_cmd(dry_run: bool, program: &str, args: &[&str]) -> Result<()> {
    if dry_run {
        let mut out = format!("{} {}", "[dry-run]".dimmed(), program.green().bold());
        if !args.is_empty() {
            for (i, arg) in args.iter().enumerate() {
                let colored_arg = if i == 0 {
                    arg.yellow().to_string()
                } else if arg.ends_with(".toml") {
                    arg.cyan().to_string()
                } else if arg.starts_with("tessera-") && arg.contains("v") {
                    arg.yellow().to_string()
                } else if arg.starts_with("-") {
                    arg.blue().to_string()
                } else {
                    // Add quotes for commit message
                    if program == "git"
                        && args.get(0) == Some(&"commit")
                        && args.get(i.wrapping_sub(1)) == Some(&"-m")
                    {
                        format!("\"{}\"", arg).blue().to_string()
                    } else {
                        arg.normal().to_string()
                    }
                };
                out.push(' ');
                out.push_str(&colored_arg);
            }
        }
        println!("{}", out);
        Ok(())
    } else {
        let status = Command::new(program).args(args).status()?;
        if !status.success() {
            bail!("{} command failed: {} {}", program, program, args.join(" "));
        }
        Ok(())
    }
}

fn write_or_preview_file(
    dry_run: bool,
    path: &str,
    new_content: &str,
    old_content: Option<&str>,
) -> Result<()> {
    if dry_run {
        use diffy::{PatchFormatter, create_patch};
        use tabled::settings::Style;
        use tabled::{Table, Tabled};
        #[derive(Tabled)]
        struct DiffLine {
            line: String,
        }
        let old = old_content.unwrap_or("");
        let patch = create_patch(old, new_content);
        let diff_str = format!("{}", PatchFormatter::new().fmt_patch(&patch));
        let diff_lines: Vec<DiffLine> = diff_str
            .lines()
            .map(|l| DiffLine {
                line: l.to_string(),
            })
            .collect();
        let table = Table::new(diff_lines).with(Style::rounded()).to_string();
        println!("{}", path.cyan());
        println!("{}", table);
        Ok(())
    } else {
        std::fs::write(path, new_content)?;
        Ok(())
    }
}

fn read_toml(path: &std::path::Path) -> Result<DocumentMut> {
    let content = fs::read_to_string(path)?;
    let doc = content.parse::<DocumentMut>()?;
    Ok(doc)
}

struct Workspace {
    members: Vec<String>,
}

impl Workspace {
    fn load(root: &str) -> Result<Self> {
        let doc = read_toml(std::path::Path::new(root))?;
        let members = doc["workspace"]["members"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No [workspace] members found"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        Ok(Self { members })
    }
    fn find_package(&self, name: &str) -> Result<std::path::PathBuf> {
        for member in &self.members {
            let path = std::path::Path::new(member);
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                let doc = read_toml(&cargo_toml)?;
                if doc["package"]["name"].as_str() == Some(name) {
                    return Ok(path.to_path_buf());
                }
            }
        }
        bail!("Package '{}' not found in workspace members", name)
    }
    fn collect_versions(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut map = std::collections::HashMap::new();
        for member in &self.members {
            let path = std::path::Path::new(member);
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                let doc = read_toml(&cargo_toml)?;
                if let (Some(name), Some(version)) = (
                    doc["package"]["name"].as_str(),
                    doc["package"]["version"].as_str(),
                ) {
                    map.insert(name.to_string(), version.to_string());
                }
            }
        }
        Ok(map)
    }
}

fn replace_path_with_version_in_workspace(
    workspace: &Workspace,
    target_package: &str,
    target_version: &str,
    package_versions: &std::collections::HashMap<String, String>,
) -> Result<Vec<(String, String, String)>> {
    let mut modified = Vec::new();
    for member in &workspace.members {
        let path = std::path::Path::new(member);
        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let mut doc = read_toml(&cargo_toml)?;
        let mut changed = false;
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(table) = doc.get_mut(section).and_then(|t| t.as_table_like_mut()) {
                let keys: Vec<_> = table.iter().map(|(k, _)| k.to_string()).collect();
                for dep in keys {
                    if dep == target_package {
                        if let Some(item) = table.get_mut(&dep) {
                            if let Some(dep_table) = item.as_table_like_mut() {
                                if dep_table.remove("path").is_some() {
                                    dep_table.insert("version", value(target_version));
                                    changed = true;
                                }
                            }
                        }
                    } else if let Some(ver) = package_versions.get(&dep) {
                        if let Some(item) = table.get_mut(&dep) {
                            if let Some(dep_table) = item.as_table_like_mut() {
                                if dep_table.remove("path").is_some() {
                                    dep_table.insert("version", value(ver));
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if changed {
            let old = std::fs::read_to_string(&cargo_toml)?;
            let new = doc.to_string();
            modified.push((cargo_toml.display().to_string(), old, new));
        }
    }
    Ok(modified)
}

fn bump_version(old: &str, bump: BumpType) -> Result<String> {
    let mut parts: Vec<u64> = old.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    while parts.len() < 3 {
        parts.push(0);
    }
    match bump {
        BumpType::Major => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
        BumpType::Minor => {
            parts[1] += 1;
            parts[2] = 0;
        }
        BumpType::Patch => {
            parts[2] += 1;
        }
    }
    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

fn find_latest_tag(package: &str) -> Result<Option<String>> {
    let pat = format!("{}-v*", package);
    let output = Command::new("git").args(["tag", "--list", &pat]).output()?;
    if !output.status.success() {
        bail!("Failed to list git tags");
    }
    let tags = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let latest = tags.into_iter().max_by(|a, b| version_cmp(a, b));
    Ok(latest)
}

fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    // Compare version numbers in tags
    let va = a.rsplit_once("-v").map(|(_, v)| v).unwrap_or("");
    let vb = b.rsplit_once("-v").map(|(_, v)| v).unwrap_or("");
    let pa: Vec<u64> = va.split('.').filter_map(|s| s.parse().ok()).collect();
    let pb: Vec<u64> = vb.split('.').filter_map(|s| s.parse().ok()).collect();
    pa.cmp(&pb)
}

fn collect_commits_since_tag(tag: &str, package_path: &str) -> Result<Vec<String>> {
    let range = if tag.is_empty() {
        "HEAD".to_string()
    } else {
        format!("{}..HEAD", tag)
    };
    let output = Command::new("git")
        .args(["log", &range, "--oneline", "--", package_path])
        .output()?;
    if !output.status.success() {
        bail!("Failed to get git log");
    }
    let commits = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(commits)
}

fn generate_changelog(
    new_version: &str,
    commits: &[String],
    last_tag: Option<&str>,
    package: &str,
) -> String {
    use chrono::Local;
    let now = Local::now();
    let date = now.format("%Y-%m-%d %:z");
    let new_tag = format!("{}-v{}", package, new_version);
    let mut s = format!("## [v{}] - {}\n\n### Changes\n", new_version, date);
    for c in commits {
        if let Some((_, msg)) = c.split_once(' ') {
            s.push_str(&format!("- {}\n", msg));
        } else {
            s.push_str(&format!("- {}\n", c));
        }
    }
    if let Some(last) = last_tag {
        s.push_str(&format!(
            "\n[Compare with previous release](https://github.com/{}/{}/compare/{}...{})\n",
            GITHUB_OWNER, GITHUB_REPO, last, new_tag
        ));
    }
    s
}
