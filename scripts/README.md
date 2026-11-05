# Scripts

Run Rust tools with `rust-script`:

```sh
cargo install rust-script
rust-script scripts/check-imports.rs --help
```

- `check-imports.rs`: format Rust sources and imports.
- `analyze-profiler.rs`: summarize frame profiles.
- `release-package.rs`: preview release versions; use `--check` for packaged-source and scaffold acceptance, `--prepare` to write the plan and `--execute` to verify, publish and push it.
- `check-mobile.py`: verify Android and iOS first-frame rendering and resume.
- `test-apple-ime.py`: verify native IME on macOS and iOS Simulator.
