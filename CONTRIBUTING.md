# Contributing to tessera

[![简体中文][contributing-zh-badge]][contributing-zh-url]

[contributing-zh-badge]: https://img.shields.io/badge/CONTRIBUTING-简体中文-blue.svg?style=for-the-badge&logo=contributing
[contributing-zh-url]: docs/CONTRIBUTING_zh-CN.md

Thank you for your interest in `tessera`! We welcome all forms of contributions, including but not limited to code, documentation, and issues.

## Development Tools

If you need to contribute code to `tessera`, in addition to the latest stable Rust, it is highly recommended to install the following tools:

- [`xbuild`](https://github.com/rust-mobile/xbuild): We use it to build and test the Android version. In the future, it may also be used for iOS compatibility.
- [`rust-script`](https://rust-script.org/#installation): We use it to run [some rust scripts](./scripts). It is helpful for development.

## Code Contribution Guidelines

To ensure code quality and consistency, and to keep the repository clean, please follow these guidelines:

## Getting Started

### Option A - Nix package manager (one-liner)
```bash
nix develop            # desktop dev shell
nix develop .#android  # android dev shell
```

### Option B - Manual setup

Rust >= 1.77 (rustup toolchain install stable)

Vulkan SDK (includes loader + headers)
Download from https://vulkan.lunarg.com, run the installer, and
follow its post‑install instructions.

```bash
# X11
sudo apt install libxi-dev libxrandr-dev libxcursor-dev
# Wayland
sudo apt install libwayland-dev libxkbcommon-dev
```

### Language for Code

All code, including documentation within the code (like `rustdoc` comments) and comments, must be written in English, unless a feature specifically requires pointing out a word in another language.

### Code Checks

- Please ensure your code format complies with the project's specifications. The rules are as follows:

  - `rustfmt edition 2024` default rules.
  - `5 use import rules`:
    1. Imports are divided into four groups, arranged in the following strict order:
        - Group 1: Standard library (`std`, `core`, `alloc`)
        - Group 2: Third-party crates
        - Group 3: Current crate root (`crate::`)
        - Group 4: Submodules of the current crate (`super::`, `self::`)
    2. There must be exactly one blank line between different groups.
    3. Imports within the same group must be arranged consecutively without any blank lines.
    4. Imports within each group must be sorted alphabetically.
    5. Imports from the same root path should be merged into a single `use` statement.

- **Formatting Tools**

  - You can use the `cargo fmt` command to format your code. This will automatically apply the first formatting rule mentioned above.
  - However, it is recommended to always use the following command (at the project root):

    ```bash
    rust-script scripts/check-imports.rs . --fix
    ```
  - Nix users: just type `fmt` inside `nix develop` - it's a smart alias that runs the same command above from any directory

    This command checks and fixes import rules, and also calls `rustfmt` for formatting. It directly applies all the formatting rules mentioned above and does not ignore script files (whereas `cargo fmt` only formats what is managed by `Cargo.toml`).

### Commit Guidelines

- Please ensure your commit messages are clear, concise, and describe the changes made.
- The commit format must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification. We have no special requirements for the scope; you can omit it, but it is recommended to identify the scope or feature of the change.
- Before pushing, please ensure:
  - Your code passes all tests (`cargo test`).
  - You have updated relevant documentation (if applicable).
  - Your code complies with the [Code Check Guidelines](#code-checks).

## Documentation Contribution Guidelines

To ensure the clarity and consistency of the documentation and to enable project collaboration, please follow these guidelines:

### Language for Documentation

All documentation content, including READMEs, Wiki pages, and other documents, must have at least an English version, which is considered the primary version. Other i18n versions can be translated from the English version, but it must be ensured that the English version always exists and its content is the most up-to-date.

Note that because `cargo doc` generated documentation does not support i18n at all, and we cannot put all versions of the documentation in the code, please do not add any non-English documentation content in the code.

### Documentation Translation Guidelines

- Documentation translation must be based on the English version, ensuring that the translated content is consistent with the English version.
- The English version of the documentation does not need an i18n suffix. Other language versions need to have the corresponding language suffix added to the filename, such as `README_zh-CN.md`.
- It is recommended that translated documents are not placed directly in the same directory as the English version, but in a relative `docs` folder, unless absolutely necessary.

### Documentation Format

It is best to pass a markdown lint, but we do not strictly require it. Please ensure the documentation is clear, readable, and consistently formatted.

### Documentation Commits

- For direct commits/PRs to this repository, please refer to the [Commit Guidelines](#commit-guidelines).
- For commits to related repositories, such as the official website, Wiki, etc., please follow their respective contribution guidelines.

## License

We assume that your contributions follow the project's dual-license terms. This project is dual-licensed under the [MIT License](./LICENSE) or the [Apache License 2.0](./LICENSE). You can choose either license.

By submitting contributions to this project, you agree that your contributions will be released under the same license terms as the project, i.e., MIT OR Apache-2.0 dual license. This means:

- Your contributions will be available under both the MIT License and the Apache License 2.0.
- Users can choose to use your contributions under either of these two licenses.
- You confirm that you have the right to provide your contributions under these license terms.
