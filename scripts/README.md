# Maintenance scripts

Run Rust scripts from the repository root with `rust-script scripts/<name>.rs --help`. Run their tests with `rust-script --test scripts/<name>.rs`.

| Script | Task |
| --- | --- |
| `check-imports.rs` | Check or format Rust sources and imports. |
| `analyze-profiler.rs` | Summarize recorded frame profiles. |
| `check-package.rs` | Validate application packages. |
| `check-web-wasm.mjs` | Validate the web build with Node.js. |
| `check-mobile.rs` | Verify first-frame rendering and foreground resume on Android or iOS. |
| `test-apple-ime.rs` | Verify native text input on macOS or iOS Simulator. |
| `release-package.rs` | Follow the [release guide](../docs/RELEASE_RULE.md). |
