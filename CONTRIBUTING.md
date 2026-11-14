# Contributing to tessera

[![简体中文][contributing-zh-badge]][contributing-zh-url]

[contributing-zh-badge]: https://img.shields.io/badge/CONTRIBUTING-简体中文-blue.svg?style=for-the-badge&logo=contributing
[contributing-zh-url]: docs/CONTRIBUTING_zh-CN.md

Thank you for your interest in `tessera`! We welcome all forms of contributions, including but not limited to code, documentation, and issues.

## Development Tools

If you need to contribute code to `tessera`, in addition to the latest stable Rust, it is highly recommended to install the following tools:

- [`rust-script`](https://rust-script.org/#installation): We use it to run [some rust scripts](scripts). It is helpful for development.

## Code Contribution Guidelines

To ensure code quality and consistency, and to keep the repository clean, please follow these guidelines:

## Getting Started

You can set up your development environment in one of two ways:

1. **Manual Setup**: Install Rust and the system dependencies yourself.
2. **Nix**: Use the development shells provided by `flake.nix` on Linux and macOS.

### Option A - Manual Setup

Follow the instructions for your operating system below.

#### 1. Install Rust

First, install the Rust toolchain by following the official instructions at [rustup.rs](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Install System Dependencies

##### For Linux

For Linux users who already have a desktop environment, you most likely do not need this step. The following is for users without a desktop environment.

If you do not currently have a desktop environment, you need to choose and install development packages for either X11 or Wayland. You typically only need one set of packages.

- **For X11**:

  - _Debian/Ubuntu_: `sudo apt install libx11-dev libxrandr-dev libxcursor-dev`
  - _Arch/Manjaro_: `sudo pacman -S libx11 libxrandr libxcursor`
  - _Fedora_: `sudo dnf install libX11-devel libXrandr-devel libXcursor-devel`

- **For Wayland**:
  - _Debian/Ubuntu_: `sudo apt install libwayland-dev libxkbcommon-dev`
  - _Arch/Manjaro_: `sudo pacman -S wayland libxkbcommon`
  - _Fedora_: `sudo dnf install wayland-devel libxkbcommon-devel`

##### For macOS

No additional system dependencies are required if you have installed the Xcode Command Line Tools.

##### For Windows

No additional dependencies are required after installing Rust. The necessary tools are included with the standard Rust installation for Windows.

#### 3. Install Vulkan SDK (Optional)

The Vulkan SDK is **optional**. It is only required if you need to perform shader validation or debugging. For general UI development, you can skip this step.

If you need it, you can download it from [vulkan.lunarg.com](https://vulkan.lunarg.com/), run the installer, and follow its post-install instructions.

### Option B - Nix package manager

If you have [Nix](https://nixos.org/download.html) installed, you can get a development shell with all dependencies ready by running:

```bash
nix develop            # Desktop dev shell
nix develop .#android  # Android dev shell
```

### Language for Code

All code, including documentation within the code (like `rustdoc` comments) and comments, must be written in English, unless a feature specifically requires pointing out a word in another language.

### Code Checks

- Please ensure your code format complies with the project's specifications. The rules are as follows:

  - `rustfmt edition 2024` default rules.
  - Imports: std / third-party / `crate::` / `super::`/`self::` groups, one blank line apart, sorted, merged by root (enforced by `scripts/check-imports.rs`).

- **Formatting Tools**

  - Run this at the project root to fix imports and run `rustfmt`, including on script files that `cargo fmt` skips; add `--check` to check without modifying files:

    ```bash
    rust-script scripts/check-imports.rs .
    ```

  - Nix users: `fmt` inside `nix develop` runs the same command from any directory.

### Commit Guidelines

- Please ensure your commit messages are clear, concise, and describe the changes made.
- The commit format must follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification. We have no special requirements for the scope; you can omit it, but it is recommended to identify the scope or feature of the change.
- **Breaking Changes**: If your commit introduces a breaking change, you **MUST** include `BREAKING CHANGE:` in the commit body or footer. This is required because our release script relies on this flag to automatically trigger a **Major** version bump.
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

## Special Note on Android Builds

Android projects are initialized, run, and built with `cargo tessera android`.

- **Prerequisites**: Ensure the Android SDK/NDK and a JDK supported by Gradle 8.14 (17 or 21) are installed, and `adb` is available.
- **Initialization**: Run `cargo tessera android init` to generate the Gradle project.
- **Run and build**:
  - Run: `cargo tessera android dev --device <device_id>`
  - Build: `cargo tessera android build --format apk`
- **Nix Users**: The Android development shell (`nix develop .#android`) provides the Android SDK/NDK, JDK, and Rust Android targets.

## License

We assume that your contributions follow the project's dual-license terms. This project is dual-licensed under the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE). You can choose either license.

By submitting contributions to this project, you agree that your contributions will be released under the same license terms as the project, i.e., MIT OR Apache-2.0 dual license. This means:

- Your contributions will be available under both the MIT License and the Apache License 2.0.
- Users can choose to use your contributions under either of these two licenses.
- You confirm that you have the right to provide your contributions under these license terms.
