#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = example_lib::run().run_desktop() {
        eprintln!("App failed to run: {err}");
    }
}
