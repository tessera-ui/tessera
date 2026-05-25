use anyhow::{Result, bail};

use crate::output;

pub struct CheckOptions<'a> {
    pub package: Option<&'a str>,
    pub target: Option<&'a str>,
}

pub fn run(options: CheckOptions<'_>) -> Result<()> {
    output::status("ColorCheck", "checking Tessera call colors");

    let workspace = workspace::load_workspace(&options)?;
    let db = workspace.host.raw_database();
    let analyzer = ra_ap_hir::attach_db(db, || {
        let mut analyzer =
            types::ColorAnalyzer::new(db, &workspace.vfs, options.package.map(str::to_owned))?;
        analyzer.analyze()?;
        Ok::<_, anyhow::Error>(analyzer)
    })?;

    if analyzer.diagnostics.is_empty() {
        output::status("ColorCheck", "passed");
        return Ok(());
    }
    for diagnostic in &analyzer.diagnostics {
        analyzer.emit_diagnostic(diagnostic);
    }

    bail!("Tessera color check failed")
}

mod analyzer;
mod types;
mod utils;
mod workspace;
