// Prevents additional console window on Windows in release, DO NOT REMOVE
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        tessera_example_calculator::desktop_main()
    }
    #[cfg(target_os = "android")]
    {
        // android platform wont actually compile this file
        // but we need to make rust-analyzer happy
        Ok(())
    }
}
