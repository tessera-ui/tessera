mod app;
mod example_components;

use std::{thread, time::Duration};

use parking_lot::deadlock;
use tessera_ui::Renderer;
use tracing::error;

use crate::app::app;

#[cfg(target_os = "android")]
use tessera_ui::winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: AndroidApp) {
    // Initialize tracing subscriber for Android (EnvFilter still honored)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    spawn_deadlock_detector();

    Renderer::run(
        app,
        |app| {
            tessera_components::pipelines::register_pipelines(app);
        },
        android_app.clone(),
    )
    .unwrap_or_else(|err| error!("App failed to run: {}", err));
}

#[allow(dead_code)]
#[cfg(target_os = "android")]
fn main() {}

#[cfg(not(target_os = "android"))]
pub fn desktop_main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("error,tessera_ui=info"))
        .unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    spawn_deadlock_detector();

    Renderer::run(app, |app| {
        tessera_components::pipelines::register_pipelines(app);
    })
    .unwrap_or_else(|e| error!("App failed to run: {e}"));
}

fn spawn_deadlock_detector() {
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(3));
            let deadlocks = deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            for (idx, threads) in deadlocks.iter().enumerate() {
                error!(
                    "Deadlock #{idx} detected involving {} threads",
                    threads.len()
                );
                for thread_info in threads {
                    error!(
                        "Thread {:?}\n{:?}",
                        thread_info.thread_id(),
                        thread_info.backtrace()
                    );
                }
            }
        }
    });
}
