use std::time::Instant;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use crate::output;

pub(crate) struct Progress {
    bar: ProgressBar,
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
            started: Instant::now(),
            enabled,
        }
    }

    pub(super) fn begin_crates(&self, total: usize) {
        if self.enabled {
            self.bar.set_length(total as u64);
            self.bar.set_position(0);
            self.bar.set_prefix("Checking");
            self.bar.set_message("analyzing crates");
            self.bar
                .set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
        }
    }

    pub(super) fn crate_started(&self, display: &str) {
        if self.enabled {
            self.bar.suspend(|| output::status("Checking", display));
        }
    }

    pub(super) fn crate_finished(&self) {
        if self.enabled {
            self.bar.inc(1);
        }
    }

    pub(super) fn workspace(&self, message: &str) {
        let stage = if message == "discovering sysroot" {
            "preparing toolchain"
        } else if message == "querying project metadata" || message.starts_with("cargo metadata:") {
            "resolving dependencies"
        } else {
            "preparing build scripts and macros"
        };
        self.bar.set_message(stage);
    }

    pub(super) fn selecting(&self) {
        self.bar.set_message("preparing crate analysis");
    }

    pub(super) fn suspend<R>(&self, action: impl FnOnce() -> R) -> R {
        self.bar.suspend(action)
    }

    pub(super) fn finish(&self, crates: usize, diagnostics: usize) {
        self.bar.finish_and_clear();
        if self.enabled && diagnostics == 0 {
            output::status(
                "Finished",
                format!(
                    "tessera checkers: passed ({crates} crates) in {:.2}s",
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
