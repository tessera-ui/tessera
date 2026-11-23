# Contributing to Tessera

[简体中文](docs/CONTRIBUTING_zh-CN.md)

## Set up and verify changes

1. Install [Rust](https://rustup.rs/) and run `git submodule update --init --recursive`, then `git -C crates/tessera-cosmic-text lfs pull` (requires [Git LFS](https://git-lfs.com/)). Keep upstream forks out of general cleanups.
2. On Linux or macOS, use `nix develop`, or install the platform dependencies manually. Linux needs the X11 (`libx11`, `libxrandr`, `libxcursor`) or Wayland (`wayland`, `libxkbcommon`) development packages; macOS needs the Xcode Command Line Tools.
3. Install [rust-script](https://rust-script.org/#installation), then run the checks from the repository root:

   ```sh
   rust-script scripts/check-imports.rs . --check
   cargo test --workspace
   ```

For Android setup, see the [CLI guide](crates/cargo-tessera/README.md#build-for-android). For packaging and platform checks, see the [maintenance scripts](scripts/README.md).

## Submit changes

- Open a pull request from a short-lived branch against `main`. CI must pass before squash merge.
- Use a [Conventional Commits](https://www.conventionalcommits.org/) title. For breaking changes, add `!` to the title and `BREAKING CHANGE:` to the description, with migration guidance.
- Follow [AGENTS.md](AGENTS.md). Update English documentation first and keep translations in sync.
- Contributions are licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at the recipient's option. Only submit work you have the right to license on those terms.

Maintainers: see the [release guide](docs/RELEASE_RULE.md).
