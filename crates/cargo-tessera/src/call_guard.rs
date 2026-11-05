use std::{
    collections::HashSet,
    panic,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::diagnostic::DiagnosticLevel as Severity;

pub use error_codes::ErrorCode;
pub use levels::{Level, LintOptions, LintTarget};
pub use types::MessageFormat;

pub struct CheckOptions<'a> {
    pub package: Option<&'a str>,
    pub target: Option<&'a str>,
    pub target_selection: TargetSelection,
    pub features: FeatureSelection<'a>,
    pub message_format: MessageFormat,
    pub quiet: bool,
    pub verbose: bool,
    pub release: bool,
    pub workspace: bool,
    pub jobs: Option<&'a str>,
    pub lints: LintOptions,
    pub fix: bool,
}

impl<'a> CheckOptions<'a> {
    pub fn new(package: Option<&'a str>, target: Option<&'a str>) -> Self {
        Self {
            package,
            target,
            target_selection: TargetSelection::default(),
            features: FeatureSelection::default(),
            message_format: MessageFormat::default(),
            quiet: false,
            verbose: false,
            release: false,
            workspace: false,
            jobs: None,
            lints: LintOptions::default(),
            fix: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TargetSelection {
    pub bin: Option<String>,
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
    // Loading rust-analyzer may block indefinitely on cargo subprocesses or a
    // stuck proc-macro server. Call checking must always terminate, so the
    // whole run is bounded instead of relying on those subprocesses finishing.
    let _watchdog = watchdog::arm(watchdog::limit_from_env());
    while run_pass(&options)? {}
    Ok(())
}

fn run_pass(options: &CheckOptions<'_>) -> Result<bool> {
    let progress = progress::Progress::new(!options.quiet && !options.message_format.is_json());
    let workers = worker_count(options.jobs)?;

    let plan = plan::Plan::resolve(options)?;
    if plan.jobs.is_empty() {
        progress.finish(0, 0, 0);
        return Ok(false);
    }
    let workspace = workspace::load_workspace(options)?;
    let db = workspace.host.raw_database();

    // Incomplete semantic analysis must never become a reusable successful check.
    let mut cache = finish_analysis(|| {
        ra_ap_hir::attach_db(db, || Ok(cache::Cache::new(&workspace, &plan, options)))
    })?;
    let (index, jobs, keys, cached) = finish_analysis(|| {
        ra_ap_hir::attach_db(db, || {
            let roots = plan
                .jobs
                .iter()
                .flat_map(|job| job.target_roots.iter().cloned())
                .collect::<Vec<_>>();
            let crates = workspace::relevant_crates(db, &workspace.vfs, &roots)?;
            let index = cache.index(&crates, workers)?;
            if options.verbose && !options.quiet && !options.message_format.is_json() {
                crate::output::status(
                    "Index",
                    format!(
                        "Tessera: {} rebuilt, {} reused",
                        cache.indexed,
                        crates.len() - cache.indexed
                    ),
                );
            }
            let shape = cache::shape(&index);
            let mut jobs = Vec::new();
            let mut keys = Vec::new();
            let mut cached = Vec::new();
            for (position, job) in plan.jobs.iter().enumerate() {
                let (job, key) = cache.job(job, position, &shape, options)?;
                if let Some(diagnostics) = cache.diagnostics(&key) {
                    if options.verbose && !options.quiet && !options.message_format.is_json() {
                        crate::output::status("Fresh", format!("{} (Tessera)", job.display));
                    }
                    cached.push((position, Ok(diagnostics)));
                } else {
                    jobs.push(job);
                }
                keys.push(key);
            }
            Ok((index, jobs, keys, cached))
        })
    })?;
    progress.begin_crates(jobs.len());
    let results = analyze_parallel(
        db,
        &workspace.vfs,
        &index,
        &jobs,
        workers,
        &options.lints,
        &progress,
    )?;
    for (position, result) in &results {
        if let Ok(diagnostics) = result {
            cache.save_diagnostics(&keys[*position], diagnostics);
        }
    }
    let mut results = results;
    results.extend(cached);
    results.sort_by_key(|(index, _)| *index);
    let mut diagnostics = Vec::new();
    for (_, result) in results {
        diagnostics.extend(result?);
    }
    let mut seen = HashSet::new();
    diagnostics
        .retain(|diagnostic| seen.insert((diagnostic.file_id, diagnostic.range, diagnostic.code)));
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    if !diagnostics.is_empty() {
        let analyzer = types::CallAnalyzer::from_index(db, &workspace.vfs, HashSet::new(), index);
        let reporter = diagnostics::Reporter::new(&analyzer, &diagnostics);
        if options.fix
            && progress.suspend(|| {
                reporter.fix(
                    &diagnostics,
                    options.quiet || options.message_format.is_json(),
                )
            })?
        {
            return Ok(true);
        }
        progress.suspend(|| {
            for diagnostic in &diagnostics {
                reporter.emit(diagnostic, options.message_format);
            }
        });
    }
    progress.finish(plan.jobs.len(), errors, warnings);
    if errors == 0 {
        return Ok(false);
    }
    if options.message_format == MessageFormat::Human
        && let Some(first) = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .filter_map(|diagnostic| diagnostic.code.error_code())
            .min()
    {
        eprintln!("For more information, run `cargo tessera check --explain {first}`.");
    }

    bail!(
        "could not check Tessera calls due to {} previous error{}",
        errors,
        if errors == 1 { "" } else { "s" }
    )
}

pub(super) struct AnalysisJob {
    pub(super) index: usize,
    pub(super) name: String,
    pub(super) display: String,
    pub(super) files: std::collections::HashSet<ra_ap_ide::FileId>,
}

pub(super) fn analyze_parallel(
    db: &ra_ap_ide::RootDatabase,
    vfs: &ra_ap_vfs::Vfs,
    index: &types::CallIndex,
    jobs: &[AnalysisJob],
    workers: usize,
    lints: &LintOptions,
    progress: &progress::Progress,
) -> Result<Vec<(usize, Result<Vec<types::Diagnostic>>)>> {
    let next = AtomicUsize::new(0);
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..workers.max(1).min(jobs.len()) {
            let db = db.clone();
            let next = &next;
            handles.push(scope.spawn(move || {
                let mut results = Vec::new();
                loop {
                    let position = next.fetch_add(1, Ordering::Relaxed);
                    let Some(job) = jobs.get(position) else { break };
                    progress.crate_started(job.index, &job.name, &job.display);
                    let result = finish_analysis(|| {
                        ra_ap_hir::attach_db(&db, || {
                            let mut analyzer = types::CallAnalyzer::from_index(
                                &db,
                                vfs,
                                job.files.clone(),
                                index.clone(),
                            );
                            analyzer.check_files(lints)?;
                            Ok(analyzer.diagnostics)
                        })
                    });
                    progress.crate_finished(job.index);
                    results.push((job.index, result));
                }
                results
            }));
        }
        let mut results = Vec::new();
        for handle in handles {
            results.extend(handle.join().map_err(|_| {
                anyhow!("Tessera checker worker panicked; refusing to proceed with unchecked calls")
            })?);
        }
        Ok(results)
    })
}

pub(super) fn collect_index_parallel(
    analyzer: &types::CallAnalyzer<'_>,
    workers: usize,
) -> Result<types::CallIndex> {
    let mut files = analyzer.metadata_files.iter().copied().collect::<Vec<_>>();
    files.sort_unstable();
    let next = AtomicUsize::new(0);
    let mut index = analyzer.index.clone();
    let parts = thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..workers.max(1).min(files.len()) {
            let db = analyzer.db.clone();
            let vfs = analyzer.vfs;
            let local_files = analyzer.local_files.clone();
            let partial = index.clone();
            let next = &next;
            let files = &files;
            handles.push(scope.spawn(move || {
                finish_analysis(|| {
                    ra_ap_hir::attach_db(&db, || {
                        let mut analyzer =
                            types::CallAnalyzer::from_index(&db, vfs, local_files, partial);
                        loop {
                            let position = next.fetch_add(1, Ordering::Relaxed);
                            let Some(file) = files.get(position) else {
                                break;
                            };
                            analyzer.collect_file_metadata(*file)?;
                        }
                        Ok(analyzer.index)
                    })
                })
            }));
        }
        handles
            .into_iter()
            .map(|handle| {
                handle.join().map_err(|_| {
                    anyhow!(
                        "Tessera indexing worker panicked; refusing to proceed with unchecked calls"
                    )
                })?
            })
            .collect::<Result<Vec<_>>>()
    })?;
    for partial in parts {
        index.merge(partial);
    }
    Ok(index)
}

fn worker_count(jobs: Option<&str>) -> Result<usize> {
    let available = thread::available_parallelism().map_or(1, usize::from);
    let configured = jobs
        .map(str::to_owned)
        .or_else(|| std::env::var("CARGO_BUILD_JOBS").ok());
    let Some(configured) = configured.filter(|value| value != "default") else {
        return Ok(available);
    };
    let count = configured
        .parse::<isize>()
        .context("--jobs must be a nonzero integer or `default`")?;
    if count == 0 {
        bail!("--jobs must not be zero");
    }
    if count < 0 {
        Ok(available.saturating_add_signed(count).max(1))
    } else {
        Ok(count as usize)
    }
}

fn finish_analysis<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    panic::catch_unwind(panic::AssertUnwindSafe(action)).map_err(|_| {
        anyhow!("Tessera call checking could not complete because rust-analyzer panicked; refusing to proceed with unchecked calls")
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzer_panics_fail_closed() {
        let error = finish_analysis(|| -> Result<()> { panic!("semantic analysis failed") })
            .expect_err("incomplete analysis must fail");
        assert!(error.to_string().contains("refusing to proceed"));
    }
}

mod analyzer;
mod cache;
mod diagnostics;
mod error_codes;
mod levels;
mod plan;
mod progress;
mod types;
mod utils;
mod watchdog;
mod workspace;
