#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_family = "wasm"))]
fn main() {
    if let Err(err) = example_lib::run().run_desktop() {
        eprintln!("App failed to run: {err}");
    }
}

#[cfg(target_family = "wasm")]
fn main() {}
