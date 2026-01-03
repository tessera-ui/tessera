# Useful Scripts

Here are some useful scripts, mainly for automating tasks and simplifying the development workflow.
All scripts are written in Rust and run using the `rust-script` tool.

## Install `rust-script`

If you haven't installed `rust-script` yet, you can do so with the following command:

```bash
cargo install rust-script
```

## Running Scripts

for example [`check-imports.rs`](check-imports.rs):

```bash
rust-script scripts/check-imports.rs --help
```

## Available Scripts

- [`check-imports.rs`](check-imports.rs): Checks and fixes `use` statements in Rust files and directories, respecting .gitignore.
- [`analyze-profiler.rs`](analyze-profiler.rs): Summarizes profiler JSONL output with cache hit rates and hot components.
- [`release-package.rs`](release-package.rs): Workspace-aware release tool for tessera. Bumps version, generates changelog, replaces path dependencies with version, and supports dry-run and no-garbage-commit publish to crates.io.
