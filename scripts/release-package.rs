#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! anyhow = "1"
//! cargo_metadata = "0.23"
//! clap = { version = "4", features = ["derive"] }
//! semver = "1"
//! serde_json = "1"
//! flate2 = "1"
//! tar = "0.4"
//! tempfile = "3"
//! toml_edit = "0.25"
//! ```

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, ensure};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::Parser;
use semver::Version;
use serde_json::{Value, json};
use toml_edit::{DocumentMut, Item, Table, value};

#[derive(Parser)]
struct Cli {
    /// Verify packaged sources, install the CLI, and exercise generated applications.
    #[arg(long, conflicts_with_all = ["prepare", "execute", "major"])]
    check: bool,
    /// Force a major bump for a package; repeat for multiple packages.
    #[arg(long)]
    major: Vec<String>,
    /// Write the reviewed versions, dependency requirements and changelogs.
    #[arg(long, conflicts_with = "execute")]
    prepare: bool,
    /// Prepare, verify, publish, then push the release commit and tags.
    #[arg(long)]
    execute: bool,
}

#[derive(Clone)]
struct Release {
    version: Version,
    changes: Vec<String>,
}

fn output(command: &mut Command) -> Result<String> {
    let result = command
        .output()
        .with_context(|| format!("Failed to run {command:?}"))?;
    ensure!(
        result.status.success(),
        "{command:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(String::from_utf8(result.stdout)?)
}

fn run(command: &mut Command) -> Result<()> {
    ensure!(
        command
            .status()
            .with_context(|| format!("Failed to run {command:?}"))?
            .success(),
        "Command failed: {command:?}"
    );
    Ok(())
}

fn latest(package: &str, root: &std::path::Path) -> Result<Option<(String, Version)>> {
    let prefix = format!("{package}-v");
    Ok(output(Command::new("git").current_dir(root).args([
        "tag",
        "--list",
        &format!("{prefix}*"),
    ]))?
    .lines()
    .filter_map(|tag| {
        Some((
            tag.to_owned(),
            Version::parse(tag.strip_prefix(&prefix)?).ok()?,
        ))
    })
    .max_by(|a, b| a.1.cmp(&b.1)))
}

fn next(mut version: Version, major: bool, minor: bool) -> Version {
    if version.pre.is_empty() {
        if major {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        } else if minor {
            version.minor += 1;
            version.patch = 0;
        } else {
            version.patch += 1;
        }
    }
    version.pre = semver::Prerelease::EMPTY;
    version.build = semver::BuildMetadata::EMPTY;
    version
}

fn owned_packages(metadata: &Metadata) -> Vec<&cargo_metadata::Package> {
    metadata
        .workspace_packages()
        .into_iter()
        .filter(|package| {
            !package
                .manifest_path
                .parent()
                .unwrap()
                .ancestors()
                .take_while(|path| *path != metadata.workspace_root)
                .any(|path| path.join(".git").exists())
        })
        .collect()
}

fn plan(metadata: &Metadata, major: &[String]) -> Result<BTreeMap<String, Release>> {
    let packages = owned_packages(metadata)
        .into_iter()
        .filter(|package| {
            package
                .publish
                .as_ref()
                .is_none_or(|registries| !registries.is_empty())
        })
        .map(|package| (package.name.to_string(), package))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        major.iter().all(|name| packages.contains_key(name)),
        "Unknown or unpublished --major package"
    );
    let mut releases = BTreeMap::new();
    let mut previous = BTreeMap::new();
    for (name, package) in &packages {
        let tag = latest(name, metadata.workspace_root.as_std_path())?;
        let range = tag
            .as_ref()
            .map_or_else(|| "HEAD".to_owned(), |(tag, _)| format!("{tag}..HEAD"));
        let directory = package
            .manifest_path
            .parent()
            .context("Missing package directory")?;
        let log = output(
            Command::new("git")
                .current_dir(&metadata.workspace_root)
                .args([
                    "log",
                    "--format=%s%n%b%x00",
                    &range,
                    "--",
                    directory.as_str(),
                ]),
        )?;
        let commits = log
            .split('\0')
            .map(str::trim)
            .filter(|message| !message.is_empty())
            .collect::<Vec<_>>();
        let initial = tag.is_none();
        let forced = major.contains(name);
        let breaking = commits.iter().any(|message| {
            message.contains("BREAKING CHANGE:")
                || message
                    .lines()
                    .next()
                    .is_some_and(|subject| subject.contains("!:"))
        });
        let feature = commits.iter().any(|message| message.starts_with("feat"));
        previous.insert(
            name.clone(),
            tag.as_ref().map(|(_, version)| version.clone()),
        );
        if tag.is_none() || !commits.is_empty() || forced {
            let version = if let Some((_, version)) = tag {
                next(version, forced || breaking, feature).max(package.version.clone())
            } else if forced {
                next(package.version.clone(), true, false)
            } else {
                package.version.clone()
            };
            releases.insert(
                name.clone(),
                Release {
                    version,
                    changes: if initial {
                        vec!["Initial release.".into()]
                    } else {
                        commits
                            .iter()
                            .filter_map(|message| message.lines().next())
                            .filter(|subject| !subject.starts_with("release:"))
                            .map(str::to_owned)
                            .collect()
                    },
                },
            );
        }
    }
    loop {
        let mut changed = false;
        for (name, package) in &packages {
            if releases.contains_key(name) {
                continue;
            }
            if package
                .dependencies
                .iter()
                .any(|dep| dep.path.is_some() && releases.contains_key(&dep.name))
            {
                let version = previous[name].as_ref().map_or_else(
                    || package.version.clone(),
                    |version| next(version.clone(), false, false).max(package.version.clone()),
                );
                releases.insert(
                    name.clone(),
                    Release {
                        version,
                        changes: vec!["Update workspace dependencies.".into()],
                    },
                );
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Ok(releases)
}

fn visit_dependencies(
    item: &mut Item,
    action: &mut impl FnMut(&str, &mut Item) -> Result<()>,
) -> Result<()> {
    let Some(table) = item.as_table_like_mut() else {
        return Ok(());
    };
    for (name, item) in table.iter_mut() {
        match name.get() {
            "dependencies" | "dev-dependencies" | "build-dependencies" => {
                if let Some(dependencies) = item.as_table_like_mut() {
                    for (alias, dependency) in dependencies.iter_mut() {
                        action(&alias, dependency)?;
                    }
                }
            }
            "workspace" => visit_dependencies(item, action)?,
            "target" => {
                if let Some(targets) = item.as_table_like_mut() {
                    for (_, target) in targets.iter_mut() {
                        visit_dependencies(target, action)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn update_dependencies(item: &mut Item, versions: &BTreeMap<String, Version>) -> Result<()> {
    visit_dependencies(item, &mut |alias, item| {
        if let Some(dependency) = item.as_table_like_mut() {
            if dependency.get("path").is_some() {
                let name = dependency
                    .get("package")
                    .and_then(Item::as_str)
                    .unwrap_or(alias);
                if let Some(version) = versions.get(name) {
                    dependency.insert("version", value(version.to_string()));
                }
            }
        }
        Ok(())
    })
}

fn prepare(metadata: &Metadata, releases: &BTreeMap<String, Release>) -> Result<Vec<String>> {
    let mut versions = metadata
        .packages
        .iter()
        .filter(|package| package.source.is_none())
        .map(|package| (package.name.to_string(), package.version.clone()))
        .collect::<BTreeMap<_, _>>();
    versions.extend(
        releases
            .iter()
            .map(|(name, release)| (name.clone(), release.version.clone())),
    );
    let mut files = Vec::new();
    for package in owned_packages(metadata) {
        let path = package.manifest_path.as_std_path();
        let original = fs::read_to_string(path)?;
        let mut doc = original.parse::<DocumentMut>()?;
        if let Some(release) = releases.get(package.name.as_str()) {
            doc["package"]["version"] = value(release.version.to_string());
            let changelog = path.with_file_name("CHANGELOG.md");
            let old = fs::read_to_string(&changelog).unwrap_or_default();
            let header = format!("## [v{}]", release.version);
            if !old.starts_with(&header) {
                let changes = release
                    .changes
                    .iter()
                    .map(|change| format!("- {change}\n"))
                    .collect::<String>();
                fs::write(&changelog, format!("{header}\n\n{changes}\n{old}"))?;
                files.push(changelog.to_string_lossy().into_owned());
            }
        }
        update_dependencies(doc.as_item_mut(), &versions)?;
        if original != doc.to_string() {
            fs::write(path, doc.to_string())?;
            files.push(path.to_string_lossy().into_owned());
        }
    }
    let root_path = metadata.workspace_root.join("Cargo.toml");
    let original = fs::read_to_string(&root_path)?;
    let mut root = original.parse::<DocumentMut>()?;
    update_dependencies(root.as_item_mut(), &versions)?;
    if original != root.to_string() {
        fs::write(&root_path, root.to_string())?;
        files.push(root_path.to_string());
    }
    Ok(files)
}

fn publish(releases: &BTreeMap<String, Release>, dry_run: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command.arg("publish");
    if dry_run {
        command.args(["--dry-run", "--allow-dirty"]);
    }
    for name in releases.keys() {
        command.args(["--package", name]);
    }
    run(&mut command)
}

fn rendered_label(value: &Value) -> bool {
    value["accessibility"]["label"] == "Hello Tessera!"
        && value["bounds"]["width"]
            .as_f64()
            .is_some_and(|width| width > 0.0)
        || value
            .as_array()
            .is_some_and(|values| values.iter().any(rendered_label))
        || value
            .as_object()
            .is_some_and(|values| values.values().any(rendered_label))
}

fn render_check(command: &mut Command, requests: &[Value]) -> Result<()> {
    let mut child = command
        .env("TESSERA_HEADLESS", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut input = child.stdin.take().context("Missing worker stdin")?;
    for request in requests {
        writeln!(input, "{request}")?;
    }
    drop(input);
    let result = child.wait_with_output()?;
    let errors = String::from_utf8_lossy(&result.stderr);
    ensure!(
        result.status.success() && !errors.contains("resource loading failed"),
        "Worker failed: {errors}"
    );
    let records = String::from_utf8(result.stdout)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ensure!(
        records
            .iter()
            .all(|record| record.get("ok") != Some(&Value::Bool(false)))
            && records.iter().any(rendered_label),
        "Generated app did not publish its rendered text: {records:?}"
    );
    Ok(())
}

fn verify(metadata: &Metadata) -> Result<()> {
    let artifacts = metadata.target_directory.join("release-candidates");
    fs::create_dir_all(&artifacts)?;
    let packages = metadata
        .packages
        .iter()
        .filter(|package| {
            package.source.is_none()
                && package
                    .publish
                    .as_ref()
                    .is_none_or(|registries| !registries.is_empty())
        })
        .collect::<Vec<_>>();
    let mut jobs = vec![Vec::new()];
    for package in &packages {
        if metadata.workspace_members.contains(&package.id) {
            jobs[0].extend(["--package", package.name.as_str()]);
        } else {
            jobs.push(vec!["--manifest-path", package.manifest_path.as_str()]);
        }
    }
    for job in jobs {
        run(Command::new("cargo")
            .args([
                "package",
                "--no-verify",
                "--exclude-lockfile",
                "--allow-dirty",
                "--target-dir",
            ])
            .arg(&artifacts)
            .args(job))?;
    }
    let staging = tempfile::tempdir()?;
    let sources = packages
        .iter()
        .map(|package| {
            (
                package.name.to_string(),
                staging
                    .path()
                    .join(format!("{}-{}", package.name, package.version)),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for package in &packages {
        let archive = artifacts
            .join("package")
            .join(format!("{}-{}.crate", package.name, package.version));
        ensure!(
            fs::metadata(&archive)?.len() <= 10_000_000,
            "Package exceeds the default upload limit: {archive}"
        );
        tar::Archive::new(flate2::read::GzDecoder::new(fs::File::open(archive)?))
            .unpack(staging.path())?;
    }
    let wire = |document: &mut DocumentMut| {
        visit_dependencies(document.as_item_mut(), &mut |alias, item| {
            let name = item
                .get("package")
                .and_then(Item::as_str)
                .unwrap_or(alias)
                .to_owned();
            if let Some(path) = sources.get(&name) {
                let requirement = item
                    .as_str()
                    .or_else(|| item.get("version").and_then(Item::as_str))
                    .context("Missing packaged dependency version")?
                    .to_owned();
                let version = &packages
                    .iter()
                    .find(|package| package.name.as_str() == name)
                    .context("Missing candidate")?
                    .version;
                ensure!(
                    semver::VersionReq::parse(&requirement)?.matches(version),
                    "{name} {version} does not match {requirement}"
                );
                if item.is_str() {
                    *item = Item::Table(Table::new());
                    item["version"] = value(requirement);
                }
                item.as_table_like_mut()
                    .context("Invalid dependency")?
                    .insert("path", value(path.to_string_lossy().as_ref()));
            }
            Ok(())
        })
    };
    for path in sources.values() {
        let manifest = path.join("Cargo.toml");
        let mut document = fs::read_to_string(&manifest)?.parse::<DocumentMut>()?;
        wire(&mut document)?;
        fs::write(manifest, document.to_string())?;
    }
    let mut workspace = DocumentMut::new();
    workspace["workspace"]["resolver"] = value("3");
    workspace["workspace"]["members"] = value(toml_edit::Array::new());
    workspace["workspace"]["exclude"] = value(
        sources
            .values()
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<toml_edit::Array>(),
    );
    workspace["profile"]["dev"]["debug"] = value(0);
    let manifest = staging.path().join("Cargo.toml");
    fs::write(&manifest, workspace.to_string())?;
    let cli_root = staging.path().join("installed");
    run(Command::new("cargo")
        .args(["install", "--debug", "--path"])
        .arg(&sources["cargo-tessera"])
        .arg("--root")
        .arg(&cli_root)
        .env("CARGO_TARGET_DIR", metadata.target_directory.as_str())
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_PROFILE_DEV_OPT_LEVEL", "1"))?;
    let cli = cli_root
        .join("bin")
        .join(format!("cargo-tessera{}", std::env::consts::EXE_SUFFIX));
    for (kind, templates) in [
        ("app", ["basic", "blank"]),
        ("plugin", ["basic", "android"]),
    ] {
        for template in templates {
            let project = staging.path().join(format!("{kind}-{template}"));
            let mut command = Command::new(&cli);
            command.arg("tessera");
            if kind == "plugin" {
                command.arg("plugin");
            }
            run(command
                .arg("new")
                .arg(&project)
                .args(["--template", template])
                .current_dir(staging.path()))?;
            let path = project.join("Cargo.toml");
            let mut document = fs::read_to_string(&path)?.parse::<DocumentMut>()?;
            wire(&mut document)?;
            if kind == "app" {
                document["dependencies"]["tessera-ui"]["features"] =
                    value(["headless"].into_iter().collect::<toml_edit::Array>());
            }
            fs::write(path, document.to_string())?;
            if kind == "app" && template == "basic" {
                fs::write(project.join("assets/acceptance.txt"), "Hello Tessera!")?;
                let source = project.join("src/app.rs");
                fs::write(&source, fs::read_to_string(&source)?.replace(
                    "text(\"Hello Tessera!\");",
                    r#"let resource = tessera_ui::remember(|| crate::res::ACCEPTANCE_TXT.load()).get();
                    text(match resource.get() {
                        tessera_ui::LoadState::Ready(bytes) => String::from_utf8(bytes.to_vec()).expect("UTF-8 asset"),
                        _ => "Loading asset".to_owned(),
                    });"#,
                ))?;
            }
            workspace["workspace"]["members"]
                .as_array_mut()
                .context("Missing workspace members")?
                .push(format!("{kind}-{template}"));
        }
    }
    fs::write(&manifest, workspace.to_string())?;
    run(Command::new(&cli)
        .args(["tessera", "check", "--workspace", "--all-targets"])
        .current_dir(staging.path())
        .env("CARGO_TARGET_DIR", metadata.target_directory.as_str())
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0"))?;
    render_check(
        Command::new(&cli)
            .args(["tessera", "headless", "-p", "app-basic"])
            .current_dir(staging.path())
            .env("CARGO_TARGET_DIR", metadata.target_directory.as_str())
            .env("CARGO_INCREMENTAL", "0")
            .env("CARGO_PROFILE_DEV_DEBUG", "0"),
        &[
            json!({"cmd":"spawn","name":"check"}),
            json!({"cmd":"render","session":"check","frames":60}),
            json!({"cmd":"snapshot","session":"check"}),
            json!({"cmd":"shutdown"}),
        ],
    )?;
    let format = if cfg!(target_os = "windows") {
        "nsis"
    } else if cfg!(target_os = "macos") {
        "app"
    } else {
        "deb"
    };
    run(Command::new(&cli)
        .args(["tessera", "package", "-p", "app-basic", "--formats", format])
        .current_dir(staging.path())
        .env("CARGO_TARGET_DIR", metadata.target_directory.as_str())
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_PROFILE_DEV_DEBUG", "0"))?;
    let output = metadata.target_directory.join("packages/app-basic");
    let installed = staging.path().join("application");
    fs::create_dir_all(&installed)?;
    if cfg!(target_os = "linux") {
        let archive = fs::read_dir(&output)?
            .filter_map(std::result::Result::ok)
            .find(|entry| entry.path().extension().is_some_and(|ext| ext == "deb"))
            .context("Missing Debian package")?
            .path();
        run(Command::new("dpkg-deb")
            .arg("-x")
            .arg(archive)
            .arg(&installed))?;
    } else if cfg!(target_os = "windows") {
        let archive = fs::read_dir(&output)?
            .filter_map(std::result::Result::ok)
            .find(|entry| entry.path().extension().is_some_and(|ext| ext == "exe"))
            .context("Missing Windows installer")?
            .path();
        run(Command::new("7z")
            .args(["x", "-y"])
            .arg(format!("-o{}", installed.display()))
            .arg(archive))?;
    }
    let executable = if cfg!(target_os = "linux") {
        installed.join("usr/bin/app_basic")
    } else if cfg!(target_os = "macos") {
        output
            .join("app-basic.app/Contents/MacOS/app_basic")
            .into_std_path_buf()
    } else {
        installed.join("app_basic.exe")
    };
    render_check(
        Command::new(executable).current_dir(staging.path()),
        &[
            json!({"cmd":"render","frames":60}),
            json!({"cmd":"snapshot"}),
            json!({"cmd":"shutdown"}),
        ],
    )?;
    println!("Verified packaged CLI, four scaffolds, headless rendering and installed resources.");
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let metadata = MetadataCommand::new().exec()?;
    std::env::set_current_dir(&metadata.workspace_root)?;
    if cli.check {
        return verify(&metadata);
    }
    let releases = plan(&metadata, &cli.major)?;
    for (name, release) in &releases {
        println!("{name}: {}", release.version);
    }
    if releases.is_empty() {
        return Ok(());
    }
    if !cli.prepare && !cli.execute {
        return Ok(());
    }
    if cli.execute {
        ensure!(
            output(Command::new("git").args(["status", "--porcelain"]))?
                .trim()
                .is_empty(),
            "Commit or stash tracked changes before publishing"
        );
    }
    let mut files = prepare(&metadata, &releases)?;
    run(Command::new("cargo").args([
        "run",
        "--package",
        "cargo-tessera",
        "--",
        "tessera",
        "check",
        "--workspace",
        "--all-targets",
    ]))?;
    if !cli.execute {
        return Ok(());
    }
    publish(&releases, true)?;
    files.push(metadata.workspace_root.join("Cargo.lock").to_string());
    run(Command::new("git").arg("add").arg("--").args(&files))?;
    let changed = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .status()?;
    if !changed.success() {
        run(Command::new("git").args(["commit", "-m", "release: prepare workspace versions"]))?;
    }
    publish(&releases, false)?;
    let mut push = Command::new("git");
    push.args(["push", "--atomic", "origin", "HEAD"]);
    for (name, release) in &releases {
        let tag = format!("{name}-v{}", release.version);
        run(Command::new("git").args(["tag", &tag]))?;
        push.arg(tag);
    }
    run(&mut push)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_plans_follow_build_and_target_dependencies_and_preserve_paths() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers=['leaf','middle','app','new','vendor']\nresolver='3'\n",
        )
        .unwrap();
        for (name, version, dependencies) in [
            ("leaf", "1.3.0-rc.1", ""),
            (
                "middle",
                "1.0.0",
                "[build-dependencies]\nrenamed={package='leaf',path='../leaf',version='1.3.0-rc.1'}\n",
            ),
            (
                "app",
                "1.0.0",
                "[target.'cfg(unix)'.dependencies]\nmiddle={path='../middle',version='1'}\n",
            ),
            ("new", "0.4.0", ""),
            ("vendor", "0.2.0", ""),
        ] {
            let package = root.join(name);
            fs::create_dir_all(package.join("src")).unwrap();
            fs::write(package.join("src/lib.rs"), "pub fn value() {}\n").unwrap();
            fs::write(
                package.join("Cargo.toml"),
                format!(
                    "[package]\nname='{name}'\nversion='{version}'\nedition='2024'\n{dependencies}"
                ),
            )
            .unwrap();
        }
        let git = |args: &[&str]| output(Command::new("git").current_dir(root).args(args)).unwrap();
        git(&["init", "-q"]);
        git(&["config", "user.name", "Release test"]);
        git(&["config", "user.email", "release@example.invalid"]);
        git(&["add", "."]);
        git(&["commit", "-qm", "feat!: initialize"]);
        for tag in [
            "leaf-v1.2.3",
            "leaf-v1.3.0-rc.1",
            "middle-v1.0.0",
            "app-v1.0.0",
        ] {
            git(&["tag", tag]);
        }
        fs::write(root.join("leaf/src/lib.rs"), "pub fn changed() {}\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-qm", "fix: update leaf"]);
        git(&["-C", "vendor", "init", "-q"]);
        let metadata = MetadataCommand::new()
            .manifest_path(root.join("Cargo.toml"))
            .exec()
            .unwrap();
        let releases = plan(&metadata, &[]).unwrap();
        for (name, version) in [
            ("leaf", "1.3.0"),
            ("middle", "1.0.1"),
            ("app", "1.0.1"),
            ("new", "0.4.0"),
        ] {
            assert_eq!(releases[name].version, Version::parse(version).unwrap());
        }
        assert!(!releases.contains_key("vendor"));
        prepare(&metadata, &releases).unwrap();
        assert!(!root.join("vendor/CHANGELOG.md").exists());
        let read = |name| {
            fs::read_to_string(root.join(name))
                .unwrap()
                .parse::<DocumentMut>()
                .unwrap()
        };
        let middle = read("middle/Cargo.toml");
        assert_eq!(
            middle["build-dependencies"]["renamed"]["version"].as_str(),
            Some("1.3.0")
        );
        assert_eq!(
            middle["build-dependencies"]["renamed"]["path"].as_str(),
            Some("../leaf")
        );
        let app = read("app/Cargo.toml");
        assert_eq!(
            app["target"]["cfg(unix)"]["dependencies"]["middle"]["version"].as_str(),
            Some("1.0.1")
        );
    }
}
