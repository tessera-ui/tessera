use std::{
    path::Path,
    process::{Child, Command},
    sync::mpsc::channel,
    time::Duration,
};

use anyhow::Result;
use colored::*;
use notify::{Event, EventKind, RecursiveMode, Watcher};

pub fn execute(verbose: bool) -> Result<()> {
    println!(
        "{}",
        "🚀 Starting development server with hot reload...".bright_cyan()
    );
    println!("{}", "Watching for file changes...".dimmed());

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            if matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                let _ = tx.send(());
            }
        }
    })?;

    let src_path = Path::new("src");
    watcher.watch(src_path, RecursiveMode::Recursive)?;

    for file in ["Cargo.toml", "build.rs"] {
        let path = Path::new(file);
        if path.exists() {
            watcher.watch(path, RecursiveMode::NonRecursive)?;
        }
    }

    let mut child: Option<Child> = None;
    let mut should_rebuild = true;

    loop {
        if should_rebuild {
            // Kill previous process
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait();
            }

            println!("\n{}", "🔨 Rebuilding...".bright_yellow());

            // Build first
            let build_status = Command::new("cargo")
                .args(if verbose {
                    vec!["build", "-v"]
                } else {
                    vec!["build"]
                })
                .status()?;

            if !build_status.success() {
                println!("{}", "❌ Build failed, waiting for changes...".red());
                should_rebuild = false;
            } else {
                println!("{}", "✅ Build successful, starting app...".green());

                // Run the app
                let mut run_cmd = Command::new("cargo");
                run_cmd.args(if verbose {
                    vec!["run", "-v"]
                } else {
                    vec!["run"]
                });

                match run_cmd.spawn() {
                    Ok(c) => {
                        child = Some(c);
                        should_rebuild = false;
                        println!("{}", "👀 Watching for changes... (Ctrl+C to stop)".cyan());
                    }
                    Err(e) => {
                        println!("{} Failed to start app: {}", "❌".red(), e);
                        should_rebuild = false;
                    }
                }
            }
        }

        // Wait for file changes
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                println!("\n{}", "📝 File changed, restarting...".bright_cyan());
                should_rebuild = true;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Some(mut running_child) = child.take() {
                    match running_child.try_wait() {
                        Ok(Some(status)) => {
                            if !status.success() {
                                println!(
                                    "\n{}",
                                    format!("❌ App crashed with exit code: {:?}", status.code())
                                        .red()
                                );
                            } else {
                                println!("\n{}", "✅ App exited normally.".green());
                            }
                            println!("{}", "Waiting for file changes to restart...".dimmed());
                        }
                        Ok(None) => {
                            child = Some(running_child);
                        }
                        Err(err) => {
                            println!("{} Failed to check app status: {}", "⚠️".yellow(), err);
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    // Cleanup
    if let Some(mut c) = child {
        let _ = c.kill();
        let _ = c.wait();
    }

    Ok(())
}
