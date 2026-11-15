//! Stall bound for Tessera call checking.
//!
//! ## Usage
//!
//! Keep call checking responsive when semantic analysis stops making progress.

use std::{
    env,
    process::Child,
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// Environment variable that overrides the stall limit in seconds.
const LIMIT_ENV: &str = "TESSERA_COLOR_CHECK_TIMEOUT_SECS";

/// Default time call checking may run without reporting progress.
const DEFAULT_LIMIT: Duration = Duration::from_secs(300);

/// Exit code used when call checking stops making progress.
const TIMEOUT_EXIT_CODE: i32 = 2;

static EPOCH: OnceLock<Instant> = OnceLock::new();
static LAST_PROGRESS_MS: AtomicU64 = AtomicU64::new(0);
static CARGO: Mutex<Vec<Arc<Mutex<Child>>>> = Mutex::new(Vec::new());

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

/// Reads the configured stall limit.
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

fn elapsed_ms() -> u64 {
    EPOCH.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// Records that call checking made progress.
pub(super) fn progress() {
    LAST_PROGRESS_MS.store(elapsed_ms(), Ordering::Relaxed);
}

/// Lets a stall kill `child` while `action` reads its output.
///
/// Returns the child for waiting unless a stall already killed it.
pub(super) fn with_cargo<T>(child: Child, action: impl FnOnce() -> T) -> (T, Option<Child>) {
    let child = Arc::new(Mutex::new(child));
    lock(&CARGO).push(child.clone());
    let result = action();
    lock(&CARGO).retain(|registered| !Arc::ptr_eq(registered, &child));
    let child = Arc::into_inner(child).map(|child| {
        child
            .into_inner()
            .unwrap_or_else(|error| error.into_inner())
    });
    (result, child)
}

/// Keeps a bound alive for as long as call checking is running.
///
/// Dropping the handle marks the run as finished, which stops the watcher
/// without terminating the process.
pub(super) struct Watchdog {
    finished: Sender<()>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        let _ = self.finished.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// Arms the stall bound for the current call checking run.
///
/// The watcher terminates the process rather than returning an error because a
/// blocked rust-analyzer load cannot be cancelled cooperatively: the blocked
/// call owns the main thread and holds no cancellation handle.
pub(super) fn arm(limit: Option<Duration>) -> Option<Watchdog> {
    let limit = limit?;
    progress();
    let (finished, receiver) = mpsc::channel();
    let worker = thread::Builder::new()
        .name("tessera-call-guard-watchdog".to_string())
        .spawn(move || {
            loop {
                match receiver.recv_timeout(Duration::from_secs(1)) {
                    Err(RecvTimeoutError::Timeout) => {}
                    _ => return,
                }
                let idle = elapsed_ms().saturating_sub(LAST_PROGRESS_MS.load(Ordering::Relaxed));
                if idle < limit.as_millis() as u64 {
                    continue;
                }
                for child in lock(&CARGO).iter() {
                    let _ = lock(child).kill();
                }
                eprintln!(
                    "error: Tessera call checking made no progress for {}s and was aborted; \
                     set {} to raise the limit or 0 to disable it",
                    limit.as_secs(),
                    LIMIT_ENV
                );
                std::process::exit(TIMEOUT_EXIT_CODE);
            }
        })
        .ok()?;
    Some(Watchdog {
        finished,
        worker: Some(worker),
    })
}
