//! Wall-clock bound for the Tessera color check.
//!
//! ## Usage
//!
//! Keep color checks responsive when semantic analysis stops making progress.

use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

/// Environment variable that overrides the wall-clock budget in seconds.
const LIMIT_ENV: &str = "TESSERA_COLOR_CHECK_TIMEOUT_SECS";

/// Default budget for a complete color check run.
const DEFAULT_LIMIT: Duration = Duration::from_secs(300);

/// Exit code used when the color check exceeds its wall-clock budget.
const TIMEOUT_EXIT_CODE: i32 = 2;

/// Reads the configured wall-clock budget.
///
/// A value of `0` disables the bound explicitly; unparsable values fall back to
/// the default so a malformed environment variable cannot remove the bound.
pub(super) fn limit_from_env() -> Option<Duration> {
    let Some(raw) = env::var_os(LIMIT_ENV) else {
        return Some(DEFAULT_LIMIT);
    };
    let Some(seconds) = raw.to_str().and_then(|raw| raw.trim().parse::<u64>().ok()) else {
        return Some(DEFAULT_LIMIT);
    };
    match seconds {
        0 => None,
        seconds => Some(Duration::from_secs(seconds)),
    }
}

/// Keeps a bound alive for as long as the color check is running.
///
/// Dropping the handle marks the run as finished, which stops the watcher
/// without terminating the process.
pub(super) struct Watchdog {
    finished: Arc<AtomicBool>,
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::SeqCst);
    }
}

/// Arms the wall-clock bound for the current color check run.
///
/// The watcher terminates the process rather than returning an error because a
/// blocked rust-analyzer load cannot be cancelled cooperatively: the blocked
/// call owns the main thread and holds no cancellation handle.
pub(super) fn arm(limit: Option<Duration>) -> Option<Watchdog> {
    let limit = limit?;
    let finished = Arc::new(AtomicBool::new(false));
    let watcher_flag = Arc::clone(&finished);
    thread::Builder::new()
        .name("tessera-color-check-watchdog".to_string())
        .spawn(move || {
            let started = Instant::now();
            while started.elapsed() < limit {
                if watcher_flag.load(Ordering::SeqCst) {
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
            if watcher_flag.load(Ordering::SeqCst) {
                return;
            }
            eprintln!(
                "error: Tessera color check exceeded its {}s wall-clock budget and was aborted; \
                 set {} to raise the limit or 0 to disable it",
                limit.as_secs(),
                LIMIT_ENV
            );
            std::process::exit(TIMEOUT_EXIT_CODE);
        })
        .ok()?;
    Some(Watchdog { finished })
}
