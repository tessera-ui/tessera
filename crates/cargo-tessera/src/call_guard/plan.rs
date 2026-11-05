use std::{
    collections::{BTreeSet, HashMap, HashSet},
    env,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{
    CargoOpt, DependencyKind, Metadata, MetadataCommand, Node, Target, TargetKind,
};

use super::{CheckOptions, FeatureSelection, TargetSelection};

#[derive(Clone, Debug)]
pub(super) struct CrateJob {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) display: String,
    pub(super) target_roots: Vec<PathBuf>,
}

pub(super) struct Plan {
    pub(super) jobs: Vec<CrateJob>,
    pub(super) target_directory: PathBuf,
}

impl Plan {
    pub(super) fn resolve(options: &CheckOptions<'_>) -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to resolve current directory")?;
        Self::resolve_at(options, &current_dir)
    }

    pub(super) fn resolve_at(options: &CheckOptions<'_>, current_dir: &Path) -> Result<Self> {
        let target = match options.target {
            Some(target) => target.to_string(),
            None => host_target()?,
        };
        let mut command = MetadataCommand::new();
        command.current_dir(current_dir);
        match &options.features {
            FeatureSelection::All => {
                command.features(CargoOpt::AllFeatures);
            }
            FeatureSelection::Selected {
                features,
                no_default_features,
            } => {
                if *no_default_features {
                    command.features(CargoOpt::NoDefaultFeatures);
                }
                if !features.is_empty() {
                    let features = features
                        .iter()
                        .map(|feature| {
                            if let Some(package) = options.package
                                && !feature.contains('/')
                            {
                                format!("{package}/{feature}")
                            } else {
                                feature.clone()
                            }
                        })
                        .collect();
                    command.features(CargoOpt::SomeFeatures(features));
                }
            }
        }
        command.other_options(vec!["--filter-platform".to_string(), target.clone()]);
        let metadata = command
            .exec()
            .context("Failed to resolve Cargo dependencies for Tessera checking")?;
        Self::from_metadata(metadata, current_dir, options)
    }

    fn from_metadata(
        metadata: Metadata,
        current_dir: &Path,
        options: &CheckOptions<'_>,
    ) -> Result<Self> {
        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or_else(|| anyhow!("Cargo did not return a resolved dependency graph"))?;
        let packages = metadata
            .packages
            .iter()
            .map(|package| (package.id.to_string(), package))
            .collect::<HashMap<_, _>>();
        let nodes = resolve
            .nodes
            .iter()
            .map(|node| (node.id.to_string(), node))
            .collect::<HashMap<_, _>>();
        let selected = selected_packages(&metadata, current_dir, options)?;
        let include_dev = options.target_selection.requires_cargo_all_targets();
        let runtime_graph =
            dependency_closure(&nodes, selected.iter().cloned(), false, include_dev);
        let mut jobs = Vec::new();
        for id in &runtime_graph {
            let Some(package) = packages.get(id) else {
                bail!("Resolved Cargo package `{id}` is missing");
            };
            if package.name.as_ref() == "tessera-ui" {
                continue;
            }
            let Some(node) = nodes.get(id) else {
                bail!("Resolved Cargo node `{id}` is missing");
            };
            let explicitly_selected = selected.contains(id);
            let consumes_tessera = node.deps.iter().any(|dependency| {
                dependency
                    .dep_kinds
                    .iter()
                    .any(|kind| kind.kind == DependencyKind::Normal)
                    && packages
                        .get(&dependency.pkg.to_string())
                        .is_some_and(|dependency| is_tessera_runtime(dependency.name.as_ref()))
            });
            if !explicitly_selected && !consumes_tessera {
                continue;
            }
            let mut target_roots = package
                .targets
                .iter()
                .filter(|target| {
                    if !target.required_features.iter().all(|required| {
                        node.features
                            .iter()
                            .any(|feature| feature.as_str() == required)
                    }) {
                        return false;
                    }
                    if explicitly_selected {
                        target_is_selected(target, &options.target_selection)
                    } else {
                        target_is_library(target)
                    }
                })
                .map(|target| target.src_path.as_std_path().to_path_buf())
                .collect::<Vec<_>>();
            target_roots.sort();
            target_roots.dedup();
            if target_roots.is_empty() {
                continue;
            }
            jobs.push(CrateJob {
                id: id.clone(),
                name: package.name.to_string(),
                display: format!("{} v{}", package.name, package.version),
                target_roots,
            });
        }
        jobs.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            jobs,
            target_directory: metadata.target_directory.into_std_path_buf(),
        })
    }
}

fn selected_packages(
    metadata: &Metadata,
    current_dir: &Path,
    options: &CheckOptions<'_>,
) -> Result<BTreeSet<String>> {
    if let Some(name) = options.package {
        let matches = metadata
            .packages
            .iter()
            .filter(|package| package.name.as_ref() == name)
            .map(|package| package.id.to_string())
            .collect::<BTreeSet<_>>();
        if matches.is_empty() {
            bail!("Cargo package `{name}` was not found");
        }
        if matches.len() > 1 {
            bail!("Cargo package `{name}` is ambiguous");
        }
        return Ok(matches);
    }
    if options.workspace {
        return Ok(metadata
            .workspace_members
            .iter()
            .map(ToString::to_string)
            .collect());
    }
    let current = metadata
        .packages
        .iter()
        .filter(|package| {
            package
                .manifest_path
                .parent()
                .is_some_and(|directory| current_dir.starts_with(directory.as_std_path()))
        })
        .max_by_key(|package| package.manifest_path.as_str().len());
    if let Some(package) = current {
        return Ok(BTreeSet::from([package.id.to_string()]));
    }
    Ok(metadata
        .workspace_default_members
        .iter()
        .map(ToString::to_string)
        .collect())
}

fn dependency_closure(
    nodes: &HashMap<String, &Node>,
    roots: impl IntoIterator<Item = String>,
    include_build: bool,
    include_dev: bool,
) -> BTreeSet<String> {
    let roots = roots.into_iter().collect::<HashSet<_>>();
    let mut pending = roots.iter().cloned().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if let Some(node) = nodes.get(&id) {
            for dependency in &node.deps {
                if dependency.dep_kinds.iter().any(|kind| match kind.kind {
                    DependencyKind::Normal => true,
                    DependencyKind::Build => include_build,
                    DependencyKind::Development => include_dev && roots.contains(&id),
                    _ => false,
                }) {
                    pending.push(dependency.pkg.to_string());
                }
            }
        }
    }
    visited
}

fn is_tessera_runtime(name: &str) -> bool {
    matches!(
        name,
        "tessera-ui" | "tessera-foundation" | "tessera-components" | "tessera-shard"
    )
}

fn target_is_library(target: &Target) -> bool {
    !target.is_proc_macro()
        && (target.is_lib()
            || target.is_kind(TargetKind::RLib)
            || target.is_kind(TargetKind::CDyLib)
            || target.is_kind(TargetKind::DyLib)
            || target.is_kind(TargetKind::StaticLib))
}

fn target_is_selected(target: &Target, selection: &TargetSelection) -> bool {
    if target.is_proc_macro() || target.is_custom_build() {
        return false;
    }
    if selection.all_targets {
        return true;
    }
    let explicit = selection.bin.is_some()
        || selection.lib
        || selection.bins
        || selection.examples
        || selection.tests
        || selection.benches;
    if explicit {
        return (selection.lib || selection.bin.is_some()) && target_is_library(target)
            || selection
                .bin
                .as_ref()
                .is_some_and(|name| target.is_bin() && target.name == *name)
            || selection.bins && target.is_bin()
            || selection.examples && target.is_example()
            || selection.tests && target.is_test()
            || selection.benches && target.is_bench();
    }
    target_is_library(target) || target.is_bin()
}

pub(super) fn host_target() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .context("Failed to query the Rust host target")?;
    if !output.status.success() {
        bail!("rustc could not report the host target");
    }
    String::from_utf8(output.stdout)?
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .ok_or_else(|| anyhow!("rustc did not report the host target"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn plan_checks_runtime_consumers_without_unselected_build_or_optional_dependencies() {
        let fixture = Fixture::new();
        let plan = Plan::resolve_at(&CheckOptions::new(Some("app"), None), &fixture.root)
            .expect("fixture plan");
        let names = plan
            .jobs
            .iter()
            .map(|job| job.name.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(names, BTreeSet::from(["app", "widgets"]));
        let app = plan
            .jobs
            .iter()
            .find(|job| job.name == "app")
            .expect("app job");
        assert_eq!(
            app.target_roots.iter().cloned().collect::<BTreeSet<_>>(),
            BTreeSet::from([
                fixture.root.join("app/src/lib.rs"),
                fixture.root.join("app/src/main.rs")
            ])
        );
        let mut binary_options = CheckOptions::new(Some("app"), None);
        binary_options.target_selection.bin = Some("app".into());
        let binary_plan = Plan::resolve_at(&binary_options, &fixture.root).expect("binary plan");
        assert_eq!(
            binary_plan
                .jobs
                .iter()
                .find(|job| job.name == "app")
                .unwrap()
                .target_roots
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            app.target_roots.iter().cloned().collect::<BTreeSet<_>>()
        );
        let mut library_options = CheckOptions::new(Some("app"), None);
        library_options.target_selection = TargetSelection::lib_only();
        let library_plan = Plan::resolve_at(&library_options, &fixture.root).expect("library plan");
        assert_eq!(
            library_plan
                .jobs
                .iter()
                .find(|job| job.name == "app")
                .expect("app library job")
                .target_roots,
            vec![fixture.root.join("app/src/lib.rs")]
        );
        let runtime = Plan::resolve_at(&CheckOptions::new(Some("tessera-ui"), None), &fixture.root)
            .expect("runtime implementation is not checked as application code");
        assert!(runtime.jobs.is_empty());
        let mut workspace_options = CheckOptions::new(None, None);
        workspace_options.workspace = true;
        let workspace =
            Plan::resolve_at(&workspace_options, &fixture.root).expect("workspace plan");
        assert!(!workspace.jobs.iter().any(|job| job.name == "tessera-ui"));
        assert!(workspace.jobs.iter().any(|job| job.name == "widgets"));
        let defaults = Plan::resolve_at(&CheckOptions::new(None, None), &fixture.root)
            .expect("default-member plan");
        assert_eq!(
            defaults
                .jobs
                .iter()
                .map(|job| job.name.as_str())
                .collect::<BTreeSet<_>>(),
            names
        );
    }

    #[test]
    fn feature_selection_adds_optional_tessera_consumers() {
        let fixture = Fixture::new();
        let ordinary = Plan::resolve_at(&CheckOptions::new(Some("app"), None), &fixture.root)
            .expect("ordinary plan");
        let features = vec!["extra".to_string()];
        let mut options = CheckOptions::new(Some("app"), None);
        options.features = FeatureSelection::Selected {
            features: &features,
            no_default_features: true,
        };
        let extra = Plan::resolve_at(&options, &fixture.root).expect("feature plan");
        assert_eq!(
            extra
                .jobs
                .iter()
                .map(|job| job.name.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["app", "optional-widgets", "widgets"])
        );
        assert!(
            !ordinary
                .jobs
                .iter()
                .any(|job| job.name == "optional-widgets")
        );
    }

    #[test]
    fn target_selection_includes_only_dependencies_active_for_that_platform() {
        let fixture = Fixture::new();
        let host = Plan::resolve_at(&CheckOptions::new(Some("app"), None), &fixture.root)
            .expect("host plan");
        let bare_metal = Plan::resolve_at(
            &CheckOptions::new(Some("app"), Some("thumbv7em-none-eabihf")),
            &fixture.root,
        )
        .expect("bare-metal plan");
        assert!(!host.jobs.iter().any(|job| job.name == "target-widgets"));
        assert!(
            bare_metal
                .jobs
                .iter()
                .any(|job| job.name == "target-widgets")
        );
    }

    struct Fixture {
        root: PathBuf,
        _directory: tempfile::TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("isolated checker fixture");
            let root = directory.path().to_path_buf();
            fs::create_dir_all(&root).expect("fixture directory");
            fs::write(root.join("Cargo.toml"), "[workspace]\nresolver = \"3\"\nmembers = [\"app\", \"widgets\", \"optional-widgets\", \"build-widgets\", \"target-widgets\", \"unrelated\", \"tessera-ui\"]\ndefault-members = [\"app\"]\n").expect("workspace manifest");
            for name in [
                "app",
                "widgets",
                "optional-widgets",
                "build-widgets",
                "target-widgets",
                "unrelated",
                "tessera-ui",
            ] {
                fs::create_dir_all(root.join(name).join("src")).expect("package sources");
                let dependencies = match name {
                    "app" => {
                        "[dependencies]\nwidgets = { path = \"../widgets\" }\noptional-widgets = { path = \"../optional-widgets\", optional = true }\n[features]\nextra = [\"dep:optional-widgets\"]\n[build-dependencies]\nbuild-widgets = { path = \"../build-widgets\" }\n[target.'cfg(target_os = \"none\")'.dependencies]\ntarget-widgets = { path = \"../target-widgets\" }\n"
                    }
                    "tessera-ui" => "",
                    _ => "[dependencies]\ntessera-ui = { path = \"../tessera-ui\" }\n",
                };
                let library = if name == "app" {
                    "[lib]\nname = \"app_shared\"\n"
                } else {
                    ""
                };
                fs::write(root.join(name).join("Cargo.toml"), format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{library}{dependencies}")).expect("package manifest");
                fs::write(root.join(name).join("src/lib.rs"), "").expect("package source");
                if name == "app" {
                    fs::write(root.join(name).join("src/main.rs"), "fn main() {}\n")
                        .expect("binary source");
                }
            }
            Self {
                root,
                _directory: directory,
            }
        }
    }
}
