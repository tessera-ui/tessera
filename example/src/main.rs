// Prevents additional console window on Windows in release, DO NOT REMOVE
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "android"))]
fn main() {
    if let Err(err) = example_lib::run().run_desktop() {
        eprintln!("App failed to run: {err}");
    }
}
