use std::{
    collections::{HashMap, HashSet},
    panic,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::diagnostic::DiagnosticLevel as Severity;
use ra_ap_base_db::SourceDatabase;

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
    Session::default().run(options)
}

/// Call checking state reused across checks of one selection.
///
/// A session keeps rust-analyzer's analysis between checks, so a later check
/// only reanalyzes what changed when manifests, build scripts, and the set of
/// source files stay the same.
#[derive(Default)]
pub struct Session {
    workspace: Option<types::LoadedWorkspace>,
}

impl Session {
    /// Checks Tessera calls for `options`, applying fixes when requested.
    pub fn run(&mut self, options: CheckOptions<'_>) -> Result<()> {
        // Loading rust-analyzer may block indefinitely on cargo subprocesses or a
        // stuck proc-macro server. Call checking must always terminate, so a run
        // that stops making progress is aborted instead of relying on those
        // subprocesses finishing.
        let _watchdog = watchdog::arm(watchdog::limit_from_env());
        let result = (|| {
            while run_pass(&options, &mut self.workspace)? {}
            Ok(())
        })();
        // A retained analysis keeps only what later checks are likely to reuse.
        if let Some(workspace) = &mut self.workspace {
            workspace.host.trigger_garbage_collection();
        }
        result
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Releasing a large analysis takes seconds and does not affect results.
        if let Some(workspace) = self.workspace.take() {
            thread::spawn(move || drop(workspace));
        }
    }
}

fn run_pass(
    options: &CheckOptions<'_>,
    reusable: &mut Option<types::LoadedWorkspace>,
) -> Result<bool> {
    let progress = progress::Progress::new(!options.quiet && !options.message_format.is_json());
    let workers = worker_count(options.jobs)?;
    let verbose = options.verbose && !options.quiet && !options.message_format.is_json();

    let plan = plan::Plan::resolve(options)?;
    if plan.jobs.is_empty() {
        progress.finish(0, 0, 0);
        return Ok(false);
    }
    let root = std::env::current_dir().context("Failed to resolve current directory")?;
    let mut workspace = workspace::load(options, &root, &plan, reusable.take())?;
    // An analysis that failed part way is not reused.
    let diagnostics = check(options, &plan, &mut workspace, workers, verbose, &progress)?;
    let workspace = reusable.insert(workspace);
    let db = workspace.host.raw_database();

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;
    if !diagnostics.is_empty() {
        let analyzer = types::CallAnalyzer::from_index(
            db,
            &workspace.vfs,
            HashSet::new(),
            types::CallIndex::default(),
        );
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

/// Returns the deduplicated diagnostics of every planned job.
fn check(
    options: &CheckOptions<'_>,
    plan: &plan::Plan,
    workspace: &mut types::LoadedWorkspace,
    workers: usize,
    verbose: bool,
    progress: &progress::Progress,
) -> Result<Vec<types::Diagnostic>> {
    // Incomplete semantic analysis must never become a reusable successful check.
    let (mut cache, pending, keys, mut results) = {
        let db = workspace.host.raw_database();
        let workspace = &*workspace;
        finish_analysis(|| {
            ra_ap_hir::attach_db(db, || {
                let cache = cache::Cache::new(workspace, plan, options);
                let mut pending = Vec::new();
                let mut keys = Vec::new();
                let mut cached = Vec::new();
                for (position, job) in plan.jobs.iter().enumerate() {
                    let (job, key) = cache.job(workspace, job, position, options)?;
                    if let Some(diagnostics) = cache.diagnostics(workspace, &key) {
                        if verbose {
                            crate::output::status("Fresh", format!("{} (Tessera)", job.display));
                        }
                        cached.push((position, Ok(diagnostics)));
                    } else {
                        pending.push(job);
                    }
                    keys.push(key);
                }
                Ok((cache, pending, keys, cached))
            })
        })?
    };
    if !pending.is_empty() {
        let selected = pending
            .iter()
            .flat_map(|job| job.crates.iter().copied())
            .collect::<HashSet<_>>();
        let crates = workspace::relevant_dependencies(workspace.host.raw_database(), selected);
        warmup::prime(
            workspace.host.raw_database(),
            &crates.iter().copied().collect::<Vec<_>>(),
            workers,
        );
        // Resolving the dependency graph parses every dependency source file.
        // A new revision lets rust-analyzer evict those syntax trees, which
        // analysis of the selected crates no longer needs. This also collects
        // process-wide type interners, so no other analysis may be running.
        workspace.host.trigger_garbage_collection();
        watchdog::progress();
        let db = workspace.host.raw_database();
        let workspace = &*workspace;
        let jobs = finish_analysis(|| {
            ra_ap_hir::attach_db(db, || {
                cache.index(workspace, &crates, workers)?;
                if verbose {
                    crate::output::status(
                        "Index",
                        format!(
                            "Tessera: {} rebuilt, {} reused",
                            cache.indexed,
                            crates.len() - cache.indexed
                        ),
                    );
                }
                pending
                    .iter()
                    .map(|job| cache.analysis_job(workspace, job))
                    .collect::<Result<Vec<_>>>()
            })
        })?;
        progress.begin_crates(jobs.len());
        let analyzed =
            analyze_parallel(db, &workspace.vfs, &jobs, workers, &options.lints, progress)?;
        for (position, result) in &analyzed {
            if let Ok(diagnostics) = result {
                cache.save_diagnostics(workspace, &keys[*position], diagnostics);
            }
        }
        results.extend(analyzed);
    }
    results.sort_by_key(|(index, _)| *index);
    let mut diagnostics = Vec::new();
    for (_, result) in results {
        diagnostics.extend(result?);
    }
    let mut seen = HashSet::new();
    diagnostics
        .retain(|diagnostic| seen.insert((diagnostic.file_id, diagnostic.range, diagnostic.code)));
    Ok(diagnostics)
}

pub(super) struct AnalysisJob {
    pub(super) index: usize,
    pub(super) name: String,
    pub(super) display: String,
    pub(super) files: HashSet<ra_ap_ide::FileId>,
    pub(super) call_index: types::CallIndex,
}

pub(super) fn analyze_parallel(
    db: &ra_ap_ide::RootDatabase,
    vfs: &ra_ap_vfs::Vfs,
    jobs: &[AnalysisJob],
    workers: usize,
    lints: &LintOptions,
    progress: &progress::Progress,
) -> Result<Vec<(usize, Result<Vec<types::Diagnostic>>)>> {
    let mut units = Vec::new();
    for (slot, job) in jobs.iter().enumerate() {
        let mut files = job.files.iter().copied().collect::<Vec<_>>();
        files.sort_unstable();
        units.extend(files.into_iter().enumerate().map(|(order, file)| {
            let size = db.file_text(file).text(db).len();
            (size, slot, order, file)
        }));
    }
    // Large files first keeps a long file from finishing the run alone.
    units.sort_by_key(|&(size, slot, order, _)| (std::cmp::Reverse(size), slot, order));
    let remaining = jobs
        .iter()
        .map(|job| AtomicUsize::new(job.files.len()))
        .collect::<Vec<_>>();
    let started = jobs
        .iter()
        .map(|_| AtomicBool::new(false))
        .collect::<Vec<_>>();
    for (slot, job) in jobs.iter().enumerate() {
        if job.files.is_empty() {
            progress.crate_started(job.index, &job.name, &job.display);
            progress.crate_finished(job.index);
            started[slot].store(true, Ordering::Relaxed);
        }
    }
    let next = AtomicUsize::new(0);
    let outputs = thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 0..workers.max(1).min(units.len()) {
            let db = db.clone();
            let (next, units, remaining, started) = (&next, &units, &remaining, &started);
            handles.push(scope.spawn(move || {
                let mut analyzers = HashMap::new();
                let mut outputs = Vec::new();
                loop {
                    let position = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(_, slot, order, file)) = units.get(position) else {
                        break;
                    };
                    let job = &jobs[slot];
                    if !started[slot].swap(true, Ordering::Relaxed) {
                        progress.crate_started(job.index, &job.name, &job.display);
                    }
                    let result = finish_analysis(|| {
                        ra_ap_hir::attach_db(&db, || {
                            analyzers
                                .entry(slot)
                                .or_insert_with(|| {
                                    types::CallAnalyzer::from_index(
                                        &db,
                                        vfs,
                                        job.files.clone(),
                                        job.call_index.clone(),
                                    )
                                })
                                .check_file(file, lints)
                        })
                    });
                    if result.is_err() {
                        analyzers.remove(&slot);
                    }
                    watchdog::progress();
                    if remaining[slot].fetch_sub(1, Ordering::AcqRel) == 1 {
                        progress.crate_finished(job.index);
                    }
                    outputs.push((slot, order, result));
                }
                outputs
            }));
        }
        let mut outputs = Vec::new();
        for handle in handles {
            outputs.extend(handle.join().map_err(|_| {
                anyhow!("Tessera checker worker panicked; refusing to proceed with unchecked calls")
            })?);
        }
        Ok::<_, anyhow::Error>(outputs)
    })?;
    let mut by_job = jobs
        .iter()
        .map(|_| Vec::new())
        .collect::<Vec<Vec<(usize, Result<Vec<types::Diagnostic>>)>>>();
    for (slot, order, result) in outputs {
        by_job[slot].push((order, result));
    }
    Ok(by_job
        .into_iter()
        .zip(jobs)
        .map(|(mut files, job)| {
            files.sort_by_key(|(order, _)| *order);
            let diagnostics = files
                .into_iter()
                .try_fold(Vec::new(), |mut all, (_, result)| {
                    all.extend(result?);
                    Ok::<_, anyhow::Error>(all)
                });
            (job.index, diagnostics)
        })
        .collect())
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
                            watchdog::progress();
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

pub(super) fn worker_count(jobs: Option<&str>) -> Result<usize> {
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
mod warmup;
mod watchdog;
mod workspace;
