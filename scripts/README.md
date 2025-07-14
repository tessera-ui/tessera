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
