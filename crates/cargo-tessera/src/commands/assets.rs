use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use aho_corasick::AhoCorasick;
use anyhow::{Context, Result, anyhow, ensure};
use cargo_metadata::MetadataCommand;
use globset::{Glob, GlobSet, GlobSetBuilder};
use tessera_build::{
    asset_files, asset_namespace, load_tessera_config_from_dir, normalize_asset_path,
    platform_asset_path, resolve_assets_dir,
};

#[derive(Debug)]
pub(super) struct ResolvedPackage {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) dir: PathBuf,
    pub(super) is_root: bool,
}

#[derive(Debug)]
pub(super) struct PlatformAssetCandidate {
    pub(super) platform_path: String,
    pub(super) source_path: PathBuf,
    pub(super) target_path: PathBuf,
    pub(super) always_copy: bool,
}

pub(super) fn sync_assets(
    root: &Path,
    candidates: &[PlatformAssetCandidate],
    used: Option<&HashSet<String>>,
) -> Result<(usize, u64)> {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    let mut count = 0;
    let mut bytes = 0;
    for asset in candidates.iter().filter(|asset| {
        asset.always_copy || used.is_none_or(|paths| paths.contains(&asset.platform_path))
    }) {
        let parent = asset
            .target_path
            .parent()
            .context("missing asset directory")?;
        fs::create_dir_all(parent)?;
        bytes += fs::copy(&asset.source_path, &asset.target_path)
            .with_context(|| format!("Failed to deliver asset {}", asset.source_path.display()))?;
        count += 1;
    }
    Ok((count, bytes))
}

pub(super) fn collect_platform_asset_candidates(
    packages: &[ResolvedPackage],
    tessera_assets_root: &Path,
) -> Result<Vec<PlatformAssetCandidate>> {
    let mut candidates = Vec::new();

    for package in packages {
        let Some(config) = load_tessera_config_from_dir(&package.dir)? else {
            continue;
        };
        let (Some(assets_dir), Some(assets)) = (
            resolve_assets_dir(&package.dir, Some(&config)),
            config.assets.as_ref(),
        ) else {
            continue;
        };
        if !assets_dir.is_dir() {
            return Err(anyhow!(
                "Configured assets directory not found for package `{}`: {}",
                package.name,
                assets_dir.display()
            ));
        }

        let tree_shaking_exclude_matcher = build_tree_shaking_exclude_matcher(
            assets.tree_shaking_exclude_patterns(),
            &package.name,
        )?;
        let namespace = asset_namespace(&package.name, &package.version);
        for source_path in asset_files(&assets_dir)? {
            let relative = source_path
                .strip_prefix(&assets_dir)
                .with_context(|| format!("Failed to relativize {}", source_path.display()))?;
            let relative_path = normalize_asset_path(relative);
            if assets.embed.contains(&relative_path) {
                continue;
            }
            let platform_path =
                platform_asset_path(&namespace, &relative_path, &fs::read(&source_path)?);
            let target_path =
                tessera_assets_root.join(platform_path.trim_start_matches("tessera/"));
            candidates.push(PlatformAssetCandidate {
                platform_path,
                source_path,
                target_path,
                always_copy: tree_shaking_exclude_matcher.is_match(&relative_path),
            });
        }
    }

    candidates.sort_by(|left, right| left.platform_path.cmp(&right.platform_path));
    Ok(candidates)
}

fn build_tree_shaking_exclude_matcher(patterns: &[String], package: &str) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let normalized = pattern.trim().replace('\\', "/");
        let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
        ensure!(
            !normalized.is_empty(),
            "Empty asset exclusion pattern in {package}"
        );
        builder.add(
            Glob::new(normalized)
                .with_context(|| format!("Invalid asset pattern {pattern} in {package}"))?,
        );
    }
    Ok(builder.build()?)
}

pub(super) fn collect_used_platform_asset_paths(
    binary_paths: &[PathBuf],
    candidates: &[PlatformAssetCandidate],
) -> Result<HashSet<String>> {
    let patterns = candidates
        .iter()
        .map(|candidate| candidate.platform_path.as_bytes())
        .collect::<Vec<_>>();
    let matcher = AhoCorasick::new(patterns)
        .context("Failed to build platform asset path matcher for binary scan")?;

    let mut used = HashSet::new();
    for binary_path in binary_paths {
        let bytes = fs::read(binary_path)
            .with_context(|| format!("Failed to read {}", binary_path.display()))?;
        for matched in matcher.find_overlapping_iter(&bytes) {
            if let Some(candidate) = candidates.get(matched.pattern().as_usize()) {
                used.insert(candidate.platform_path.clone());
            }
        }
    }
    Ok(used)
}

pub(super) fn collect_resolved_packages(
    root_dir: &Path,
    package_dir: Option<&PathBuf>,
) -> Result<Vec<ResolvedPackage>> {
    let manifest_path = root_dir.join("Cargo.toml");
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .context("Failed to run cargo metadata")?;

    let root_manifest = package_dir
        .map(|dir| dir.join("Cargo.toml"))
        .unwrap_or_else(|| manifest_path.clone())
        .canonicalize()
        .with_context(|| "Failed to resolve root Cargo.toml")?;

    let root_pkg = metadata
        .packages
        .iter()
        .find(|pkg| {
            pkg.manifest_path
                .as_std_path()
                .canonicalize()
                .is_ok_and(|path| path == root_manifest)
        })
        .ok_or_else(|| anyhow!("Failed to resolve the root package from cargo metadata"))?;

    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| anyhow!("cargo metadata missing dependency graph"))?;

    let mut nodes = HashMap::new();
    for node in &resolve.nodes {
        nodes.insert(node.id.clone(), node);
    }

    let mut visited = HashSet::new();
    let mut stack = vec![root_pkg.id.clone()];
    while let Some(id) = stack.pop() {
        if visited.insert(id.clone())
            && let Some(node) = nodes.get(&id)
        {
            for dep in &node.deps {
                stack.push(dep.pkg.clone());
            }
        }
    }

    let mut packages = Vec::new();
    for pkg in metadata
        .packages
        .iter()
        .filter(|pkg| visited.contains(&pkg.id))
    {
        let dir = pkg
            .manifest_path
            .clone()
            .into_std_path_buf()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("Failed to locate package root for {}", pkg.manifest_path))?;
        packages.push(ResolvedPackage {
            name: pkg.name.to_string(),
            version: pkg.version.to_string(),
            dir,
            is_root: pkg.id == root_pkg.id,
        });
    }
    packages.sort_by(|left, right| (&left.name, &left.version).cmp(&(&right.name, &right.version)));
    Ok(packages)
}
