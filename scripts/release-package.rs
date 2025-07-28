#!/usr/bin/env rust-script
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
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

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    process::Command,
};

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
    /// Specify a package for a major version bump.
    #[arg(long)]
    major: Option<String>,

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
const PUBLISHABLE_PACKAGES: &[&str] = &[
    "tessera-ui",
    "tessera-ui-basic-components",
    "tessera-ui-macros",
];

fn release_package(
    package_name: &str,
    bump: BumpType,
    execute: bool,
    workspace: &Workspace,
) -> Result<()> {
    let package_path = workspace.find_package(package_name)?;
    let cargo_toml_path = package_path.join("Cargo.toml");

    let mut doc = read_toml(&cargo_toml_path)?;
    let old_version = doc["package"]["version"].as_str().unwrap_or("").to_string();
    let new_version = bump_version(&old_version, bump)?;

    println!(
        "\n{} {}",
        "📦",
        format!("Releasing Package: {}", package_name)
            .yellow()
            .bold()
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

    let latest_tag = find_latest_tag(package_name)?;
    let rel_path = package_path.to_string_lossy();
    let commits = if let Some(tag) = &latest_tag {
        collect_commits_since_tag(tag, &rel_path)?
    } else {
        collect_commits_since_tag("", &rel_path)?
    };
    let changelog = generate_changelog(&new_version, &commits, latest_tag.as_deref(), package_name);

    let changelog_path = package_path.join("CHANGELOG.md");
    let changelog_path_str = changelog_path.to_str().unwrap();
    let old_changelog = std::fs::read_to_string(&changelog_path).unwrap_or_default();
    let new_changelog = format!("{}\n{}", changelog, old_changelog);
    let dry_run = !execute;
    write_or_preview_file(
        dry_run,
        changelog_path_str,
        &new_changelog,
        Some(&old_changelog),
    )?;

    doc["package"]["version"] = value(new_version.clone());
    write_or_preview_file(
        dry_run,
        cargo_toml_path.to_str().unwrap(),
        &doc.to_string(),
        Some(&fs::read_to_string(&cargo_toml_path)?),
    )?;
    run_or_preview_cmd(dry_run, "cargo", &["check", "--workspace"])?;

    let mut files_to_add = vec![
        changelog_path_str.to_string(),
        cargo_toml_path.to_str().unwrap().to_string(),
        "Cargo.lock".to_string(),
    ];

    let release_commit_msg = format!("release({}): v{}", package_name, new_version);
    let tag = format!("{}-v{}", package_name, new_version);

    let package_versions = workspace.collect_versions()?;
    let modified_files =
        replace_path_with_version_in_workspace(workspace, package_name, &package_versions)?;

    for (file, _, _) in &modified_files {
        files_to_add.push(file.clone());
    }

    let add_args: Vec<&str> = files_to_add.iter().map(|s| s.as_str()).collect();
    run_or_preview_cmd(
        dry_run,
        "git",
        &["add"]
            .iter()
            .cloned()
            .chain(add_args.into_iter())
            .collect::<Vec<_>>(),
    )?;
    run_or_preview_cmd(dry_run, "git", &["commit", "-m", &release_commit_msg])?;
    run_or_preview_cmd(dry_run, "git", &["tag", &tag])?;

    run_or_preview_cmd(dry_run, "git", &["push"])?;
    run_or_preview_cmd(dry_run, "git", &["push", "origin", &tag])?;

    for (file, old, new) in &modified_files {
        write_or_preview_file(dry_run, file, new, Some(old))?;
    }

    if !modified_files.is_empty() {
        let temp_commit_msg = "chore: replace path dependencies with version for publish";
        run_or_preview_cmd(dry_run, "git", &["add", "."])?;
        run_or_preview_cmd(dry_run, "git", &["commit", "-m", temp_commit_msg])?;
    }

    run_or_preview_cmd(dry_run, "cargo", &["publish", "-p", package_name])?;

    if !modified_files.is_empty() {
        run_or_preview_cmd(dry_run, "git", &["reset", "--hard", &tag])?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace = Workspace::load("Cargo.toml")?;
    let publishable_set: HashSet<&_> = PUBLISHABLE_PACKAGES.iter().cloned().collect();

    println!("Analyzing packages to determine version bumps...");

    let mut release_plan = HashMap::new();

    // 1. Determine initial bumps based on commit history for publishable packages
    for pkg_name in PUBLISHABLE_PACKAGES {
        let package = workspace.packages.get(*pkg_name).unwrap();
        if let Some(bump) = determine_bump_type(package, &workspace, cli.major.as_ref())? {
            release_plan.insert(package.name.clone(), bump);
        }
    }

    // 2. Propagate changes to dependents within the publishable set
    let mut final_release_plan = release_plan.clone();
    let all_packages_sorted = workspace.topological_sort()?;
    for package_name in &all_packages_sorted {
        if !publishable_set.contains(package_name.as_str()) {
            continue;
        }
        let package = workspace.packages.get(package_name).unwrap();
        for dep in &package.dependencies {
            if release_plan.contains_key(dep) {
                final_release_plan
                    .entry(package_name.clone())
                    .or_insert(BumpType::Patch);
            }
        }
    }

    if final_release_plan.is_empty() {
        println!("✅ No packages need to be released.");
        return Ok(());
    }

    println!("\n📝 Release Plan:");
    #[derive(Tabled)]
    struct PlanEntry {
        package: String,
        bump: String,
    }
    let mut plan_entries = Vec::new();
    let mut final_release_order = all_packages_sorted;
    final_release_order.retain(|pkg| final_release_plan.contains_key(pkg));

    for package_name in &final_release_order {
        plan_entries.push(PlanEntry {
            package: package_name.clone(),
            bump: format!("{:?}", final_release_plan.get(package_name).unwrap()),
        });
    }
    let mut table = tabled::Table::new(plan_entries);
    table.with(tabled::settings::Style::rounded());
    println!("{}", table);

    if !cli.execute {
        println!("\nThis is a dry run. To execute the release, run with --execute");
    }

    for package_name in &final_release_order {
        let bump = *final_release_plan.get(package_name).unwrap();
        release_package(package_name, bump, cli.execute, &workspace)?;
    }

    println!("\n✅ All packages released successfully!");

    Ok(())
}

fn determine_bump_type(
    package: &Package,
    _workspace: &Workspace,
    major_bump_pkg: Option<&String>,
) -> Result<Option<BumpType>> {
    if Some(&package.name) == major_bump_pkg {
        return Ok(Some(BumpType::Major));
    }

    let latest_tag = find_latest_tag(&package.name)?;
    let commits = collect_commits_since_tag(
        latest_tag.as_deref().unwrap_or(""),
        &package.path.to_string_lossy(),
    )?;

    if commits.is_empty() {
        return Ok(None);
    }

    let mut bump_type = BumpType::Patch;
    for commit in commits {
        if let Some(msg) = commit.split_once(' ').map(|(_, m)| m) {
            if msg.starts_with("feat") {
                bump_type = BumpType::Minor;
            }
            if msg.contains("BREAKING CHANGE") {
                bump_type = BumpType::Minor;
            }
        }
    }
    Ok(Some(bump_type))
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
        let table = tabled::Table::new(diff_lines)
            .with(Style::rounded())
            .to_string();
        println!("{}", path.cyan());
        println!("{}", table);
        Ok(())
    } else {
        std::fs::write(path, new_content)?;
        Ok(())
    }
}

fn read_toml(path: &Path) -> Result<DocumentMut> {
    let content = fs::read_to_string(path)?;
    let doc = content.parse::<DocumentMut>()?;
    Ok(doc)
}

#[derive(Debug, Clone)]
struct Package {
    name: String,
    path: std::path::PathBuf,
    version: String,
    dependencies: Vec<String>,
}

struct Workspace {
    members: Vec<String>,
    packages: HashMap<String, Package>,
}

impl Workspace {
    fn load(root: &str) -> Result<Self> {
        let doc = read_toml(Path::new(root))?;
        let members: Vec<String> = doc["workspace"]["members"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No [workspace] members found"))?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let mut packages = HashMap::new();
        let member_names: HashSet<_> = members
            .iter()
            .map(|m| {
                let path = Path::new(m);
                read_toml(&path.join("Cargo.toml"))
                    .ok()
                    .and_then(|d| d["package"]["name"].as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string())
            })
            .collect();

        for member_path in &members {
            let cargo_toml_path = Path::new(member_path).join("Cargo.toml");
            if !cargo_toml_path.exists() {
                continue;
            }

            let doc = read_toml(&cargo_toml_path)?;
            if !doc.contains_key("package") {
                continue;
            }
            let pkg_info = &doc["package"];
            let name = pkg_info["name"].as_str().unwrap().to_string();
            let version = pkg_info["version"].as_str().unwrap().to_string();

            let mut dependencies = Vec::new();
            if let Some(deps) = doc.get("dependencies").and_then(|d| d.as_table_like()) {
                for (dep_name, _) in deps.iter() {
                    if member_names.contains(dep_name) {
                        dependencies.push(dep_name.to_string());
                    }
                }
            }

            packages.insert(
                name.clone(),
                Package {
                    name,
                    version,
                    path: Path::new(member_path).to_path_buf(),
                    dependencies,
                },
            );
        }

        Ok(Self { members, packages })
    }

    fn find_package(&self, name: &str) -> Result<std::path::PathBuf> {
        self.packages
            .get(name)
            .map(|p| p.path.clone())
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in workspace members", name))
    }

    fn collect_versions(&self) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();
        for (name, package) in &self.packages {
            map.insert(name.clone(), package.version.clone());
        }
        Ok(map)
    }

    fn topological_sort(&self) -> Result<Vec<String>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        let mut package_names: Vec<_> = self.packages.keys().cloned().collect();
        package_names.sort();

        for package_name in &package_names {
            if !visited.contains(package_name) {
                self.topological_sort_util(
                    package_name,
                    &mut visited,
                    &mut recursion_stack,
                    &mut sorted,
                )?;
            }
        }

        Ok(sorted)
    }

    fn topological_sort_util(
        &self,
        package_name: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<()> {
        visited.insert(package_name.to_string());
        recursion_stack.insert(package_name.to_string());

        if let Some(package) = self.packages.get(package_name) {
            let mut deps = package.dependencies.clone();
            deps.sort();
            for dep_name in &deps {
                if !visited.contains(dep_name) {
                    self.topological_sort_util(dep_name, visited, recursion_stack, sorted)?;
                } else if recursion_stack.contains(dep_name) {
                    bail!("Circular dependency detected in workspace");
                }
            }
        }

        recursion_stack.remove(package_name);
        sorted.push(package_name.to_string());

        Ok(())
    }
}

fn replace_path_with_version_in_workspace(
    workspace: &Workspace,
    target_package: &str,
    package_versions: &HashMap<String, String>,
) -> Result<Vec<(String, String, String)>> {
    let mut modified = Vec::new();
    for member in &workspace.members {
        let path = Path::new(member);
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
                    if let Some(ver) = package_versions.get(&dep) {
                        if dep != target_package {
                            if let Some(item) = table.get_mut(&dep) {
                                if let Some(dep_table) = item.as_table_like_mut() {
                                    if dep_table.remove("path").is_some() {
                                        dep_table.insert("version", value(ver.clone()));
                                        changed = true;
                                    }
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
