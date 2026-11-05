use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use cargo_metadata::{MetadataCommand, Package};
use heck::ToLowerCamelCase;
use serde_json::{Value, json};
use tessera_build::AssetBackend;

use super::build;

pub(super) struct Application {
    pub package: Package,
    pub binary: String,
    pub config: Value,
    pub target_dir: PathBuf,
}

impl Application {
    pub fn new(package: Option<&str>, binary: Option<&str>) -> Result<Self> {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        let package = if let Some(name) = package {
            metadata
                .packages
                .iter()
                .find(|package| package.name.as_str() == name)
        } else {
            metadata.root_package()
        }
        .context("Select an application package with --package")?;
        let bins = package
            .targets
            .iter()
            .filter(|target| target.is_bin())
            .collect::<Vec<_>>();
        let name = binary
            .or(package.default_run.as_deref())
            .or_else(|| (bins.len() == 1).then(|| bins[0].name.as_str()))
            .context("Select an application executable with --bin")?;
        ensure!(
            bins.iter().any(|target| target.name == name),
            "Unknown executable `{name}`"
        );
        let mut config = json!({
            "productName": package.name.as_str(), "version": package.version.to_string(),
            "identifier": format!("com.tessera.{}", package.name.replace('_', "-")),
            "description": package.description, "authors": package.authors, "homepage": package.homepage,
        });
        if let Some(custom) = package.metadata.get("packager") {
            for (key, value) in custom
                .as_object()
                .context("packager metadata must be a table")?
            {
                config[key.to_lower_camel_case()] = value.clone();
            }
        }
        Ok(Self {
            package: package.clone(),
            binary: name.to_owned(),
            config,
            target_dir: metadata.target_directory.into_std_path_buf(),
        })
    }

    pub fn build(&self, release: bool, target: Option<&str>) -> Result<PathBuf> {
        build::execute(
            release,
            target,
            Some(self.package.name.as_str()),
            Some(&self.binary),
            None,
            false,
            Some(AssetBackend::Platform),
        )?
        .into_iter()
        .find(|path| {
            path.file_stem()
                .is_some_and(|stem| stem == self.binary.as_str())
        })
        .context("Cargo did not produce the selected executable")
    }

    pub fn output_dir(&self) -> PathBuf {
        self.target_dir
            .join("packages")
            .join(self.package.name.as_str())
    }
}
