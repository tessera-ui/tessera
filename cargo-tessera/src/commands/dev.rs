use std::{
    path::Path,
    process::{Child, Command},
    sync::mpsc::channel,
    time::Duration,
};

use anyhow::Result;
use colored::*;
use notify::{Event, RecursiveMode, Watcher};

pub fn execute(verbose: bool) -> Result<()> {
    println!(
        "{}",
        "🚀 Starting development server with hot reload...".bright_cyan()
    );
    println!("{}", "Watching for file changes...".dimmed());

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            // Only trigger on write/create events for Rust files
            if (event.kind.is_modify() || event.kind.is_create())
                && event.paths.iter().any(|p| {
                    p.extension()
                        .is_some_and(|ext| ext == "rs" || ext == "toml")
                })
            {
                let _ = tx.send(());
            }
        }
    })?;

    // Watch src directory, Cargo.toml and build.rs (if exists)
    watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
    watcher.watch(Path::new("Cargo.toml"), RecursiveMode::NonRecursive)?;
    if Path::new("build.rs").exists() {
        watcher.watch(Path::new("build.rs"), RecursiveMode::NonRecursive)?;
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
                // Check if child process is still running
                if let Some(ref mut c) = child
                    && let Ok(Some(status)) = c.try_wait()
                {
                    // Process exited - stop dev mode
                    if !status.success() {
                        println!(
                            "\n{}",
                            format!("❌ App crashed with exit code: {:?}", status.code()).red()
                        );
                    } else {
                        println!("\n{}", "✅ App exited normally.".green());
                    }
                    println!("{}", "Stopping dev server...".dimmed());
                    break;
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
