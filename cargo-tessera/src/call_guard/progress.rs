use std::{collections::BTreeMap, sync::Mutex, time::Instant};

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::output;

pub(crate) struct Progress {
    bar: ProgressBar,
    active: Mutex<BTreeMap<usize, String>>,
    started: Instant,
    enabled: bool,
}

impl Progress {
    pub(super) fn new(enabled: bool) -> Self {
        if enabled {
            output::status("Running", "tessera checkers");
        }
        let bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::hidden());
        bar.set_style(
            ProgressStyle::with_template(
                "{prefix:>12.cyan.bold} [{bar:20}] {pos}/{len}: {wide_msg}",
            )
            .expect("valid checker progress template")
            .progress_chars("=> "),
        );
        Self {
            bar,
            active: Mutex::default(),
            started: Instant::now(),
            enabled,
        }
    }

    pub(super) fn begin_crates(&self, total: usize) {
        if self.enabled {
            self.bar.set_length(total as u64);
            self.bar.set_position(0);
            self.bar.set_prefix("Checking");
            self.bar
                .set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        }
    }

    pub(super) fn crate_started(&self, index: usize, name: &str, display: &str) {
        if self.enabled {
            self.bar.suspend(|| output::status("Checking", display));
            let mut active = self.active.lock().expect("checker progress poisoned");
            active.insert(index, name.to_owned());
            self.bar
                .set_message(active.values().cloned().collect::<Vec<_>>().join(", "));
        }
    }

    pub(super) fn crate_finished(&self, index: usize) {
        if self.enabled {
            let mut active = self.active.lock().expect("checker progress poisoned");
            active.remove(&index);
            self.bar.inc(1);
            self.bar
                .set_message(active.values().cloned().collect::<Vec<_>>().join(", "));
        }
    }

    pub(super) fn suspend<R>(&self, action: impl FnOnce() -> R) -> R {
        self.bar.suspend(action)
    }

    pub(super) fn finish(&self, crates: usize, errors: usize, warnings: usize) {
        self.bar.finish_and_clear();
        if self.enabled && errors == 0 {
            output::status(
                "Finished",
                format!(
                    "tessera checkers: passed ({crates} crates; warnings: {warnings}) in {:.2}s",
                    self.started.elapsed().as_secs_f64(),
                ),
            );
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;

    #[test]
    fn progress_tracks_parallel_crates_and_removes_completed_jobs() {
        let progress = Progress::new(true);
        progress.crate_started(0, "first", "first v1.0.0");
        progress.crate_started(1, "second", "second v2.0.0");
        assert!(progress.bar.message().contains("first"));
        assert!(progress.bar.message().contains("second"));
        progress.crate_finished(0);
        assert_eq!(progress.bar.message(), "second");
        assert_eq!(progress.bar.position(), 1);
        progress.crate_finished(1);
        assert!(progress.bar.message().is_empty());
        assert_eq!(progress.bar.position(), 2);
    }
}
