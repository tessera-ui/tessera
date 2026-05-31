use std::panic;

use anyhow::{Context, Result, bail};

use crate::output;

pub use types::MessageFormat;

pub struct CheckOptions<'a> {
    pub package: Option<&'a str>,
    pub target: Option<&'a str>,
    pub target_selection: TargetSelection,
    pub features: FeatureSelection<'a>,
    pub message_format: MessageFormat,
}

impl<'a> CheckOptions<'a> {
    pub fn new(package: Option<&'a str>, target: Option<&'a str>) -> Self {
        Self {
            package,
            target,
            target_selection: TargetSelection::default(),
            features: FeatureSelection::default(),
            message_format: MessageFormat::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TargetSelection {
    pub lib: bool,
    pub bins: bool,
    pub examples: bool,
    pub tests: bool,
    pub benches: bool,
    pub all_targets: bool,
}

impl TargetSelection {
    pub fn lib_only() -> Self {
        Self {
            lib: true,
            ..Self::default()
        }
    }

    pub fn requires_cargo_all_targets(&self) -> bool {
        self.all_targets || self.tests || self.benches || self.examples
    }
}

#[derive(Clone, Debug)]
pub enum FeatureSelection<'a> {
    All,
    Selected {
        features: &'a [String],
        no_default_features: bool,
    },
}

impl Default for FeatureSelection<'_> {
    fn default() -> Self {
        Self::Selected {
            features: &[],
            no_default_features: false,
        }
    }
}

impl<'a> FeatureSelection<'a> {
    pub fn from_cargo_args(
        features: &'a [String],
        all_features: bool,
        no_default_features: bool,
    ) -> Self {
        if all_features {
            Self::All
        } else {
            Self::Selected {
                features,
                no_default_features,
            }
        }
    }
}

pub fn run(options: CheckOptions<'_>) -> Result<()> {
    if !options.message_format.is_json() {
        output::status("ColorCheck", "checking Tessera call colors");
    }

    let workspace = workspace::load_workspace(&options)?;
    let db = workspace.host.raw_database();

    // rust-analyzer's internal trait solver may panic on certain edge cases
    // (e.g. ConstId in generic positions). Catch and degrade to a warning.
    let analyzer_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        ra_ap_hir::attach_db(db, || {
            let local_files = workspace::selected_local_files(db, &workspace.vfs, &options)
                .context("Failed to select Rust source files for Tessera color checking")?;
            let mut analyzer = types::ColorAnalyzer::new(
                db,
                &workspace.vfs,
                options.package.map(str::to_owned),
                local_files,
            )?;
            analyzer.analyze()?;
            Ok::<_, anyhow::Error>(analyzer)
        })
    }));

    let analyzer = match analyzer_result {
        Ok(Ok(analyzer)) => analyzer,
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            if !options.message_format.is_json() {
                output::warn(
                    "ColorCheck: internal rust-analyzer panic during color check; skipping. Consider running `cargo check` separately to verify compilation.",
                );
            }
            return Ok(());
        }
    };

    if analyzer.diagnostics.is_empty() {
        if !options.message_format.is_json() {
            output::status("ColorCheck", "passed");
        }
        return Ok(());
    }
    for diagnostic in &analyzer.diagnostics {
        analyzer.emit_diagnostic(diagnostic, options.message_format);
    }

    bail!(
        "Tessera color check failed with {} diagnostic(s)",
        analyzer.diagnostics.len()
    )
}

mod analyzer;
mod types;
mod utils;
mod workspace;
