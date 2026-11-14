# cargo-tessera

CLI tool for the Tessera UI framework - streamline project scaffolding, development, and building.

## Installation

Requires Rust 1.98 or newer.

```bash
cargo install cargo-tessera
```

## Usage

### Create a new project

```bash
cargo tessera new my-app
cd my-app
```

App and plugin scaffolds include an `AGENTS.md` explaining how to install the
optional `tessera-good-taste` skill.

### Install the agent skill

Run this command in a project when you want Tessera application guidance:

```bash
cargo tessera skill
```

An interactive selector chooses the scope (Local by default).

- Local: `<project>/.agents/skills/tessera-good-taste/` (current directory by default).
- Global: `~/.agents/skills/tessera-good-taste/`, using your operating-system home directory.

For scripts or non-interactive terminals, explicitly select one scope:

```bash
cargo tessera skill --local
cargo tessera skill --local --path my-app
cargo tessera skill --global
```

`--local`/`--global` skip the selector and need no TTY; `--path` applies only to
local installs.

Existing files are kept; `--force` replaces modified bundled skill files. Invoke the installed skill as `$tessera-good-taste` in an
agent that supports its installation location, or read its `SKILL.md` directly.

For agent-specific installation through the Skills CLI, select the skill from
this repository:

```bash
npx skills add tessera-ui/tessera --skill tessera-good-taste
```

### Start development server

```bash
cargo tessera dev
```
`cargo tessera dev` rebuilds and restarts the app when sources, manifests, app config, or resources change. Pass `--verbose` to see the underlying `cargo` commands.

Record profiler output or show the dirty replay overlay; both flags also work
with `build` and `android dev` (Android takes a sandbox path such as
`files/tessera-profiler.jsonl`):

```bash
cargo tessera dev -p example --profiling-output profiles/dev.jsonl
cargo tessera dev -p example --debug-dirty-overlay
```

### Build for release

```bash
cargo tessera build --release
```

Cross-compile for a specific target:

```bash
cargo tessera build --release --target x86_64-pc-windows-msvc
```

### Package a desktop application

```sh
cargo install cargo-packager --locked
cargo tessera package -p my-app --release
```

Choose formats with `--formats nsis,wix` (Windows), `app,dmg` (macOS), or
`deb,appimage,pacman` (Linux). Use `--bin` for packages with multiple executables.
Set shared application metadata in `tessera-app.toml`:

```toml
name = "My App"
identifier = "com.example.myapp"
icon = "icon.svg"
```

`name` defaults to the Cargo package name. `identifier` is a reverse-domain name
with ASCII alphanumeric segments starting with letters. Paths are relative to the
package root. Icons must be square:

| Source | Guidance |
| --- | --- |
| SVG | Recommended: scales to every output size. Convert text to paths and embed external resources. |
| PNG | Supported raster alternative; use at least 1024 × 1024 pixels. |

Tessera generates every platform icon format from this source.

The standard `tessera_build::build()` generates `res::application()`. Use it as
renderer defaults (`..res::application()`) to apply the display name, desktop
window icon, and launcher identifier. New projects include this setup.

Versions come from Cargo. Android `versionCode` is
`major * 1_000_000 + minor * 1_000 + patch + 1`.

Packages are written to `target/packages/<package>`. Binary and resource paths
are supplied automatically. Use `[package.metadata.packager]` for
[signing, installer settings, and additional resources](https://docs.crabnebula.dev/packager/configuration/).
Shared names, identifiers, versions, and configured icons take precedence.

### Fix diagnostics

```bash
cargo tessera fix -p my-app
```

Applies machine-applicable Tessera suggestions, runs `cargo fix`, and checks the
result. Review uncertain suggestions manually. Cargo fix options such as
`--allow-dirty`, `--allow-staged`, and `--allow-no-vcs` are supported.

### Analyze profiler output

```bash
cargo tessera profiling analyze profiles/dev.jsonl --top 30
```

Optional CSV export:

```bash
cargo tessera profiling analyze profiles/dev.jsonl --csv profiles/components.csv
```

Android pull + analyze in one step:

```bash
cargo tessera profiling analyze-android \
  --package com.example.my_app \
  --device 8cd1353b \
  --remote-path files/tessera-profiler.jsonl \
  --pull-to profiles/android.jsonl
```

### Headless profiling

```bash
cargo tessera headless -p example --profiling-output target/headless-profile.jsonl
```

The JSONL session renders frames, takes snapshots, resizes the viewport, and
injects input without a window. Profiles report CPU time for build, layout,
render preparation, encoding, and submission. `TESSERA_GPU_TIMING=1` adds GPU
timestamps and waits for readback after each frame.
`TESSERA_HEADLESS_SCALE_FACTOR` sets the scale factor (default 1).

### Build for iOS

On macOS with Xcode installed:

```sh
cargo tessera ios build -p my-app
cargo tessera ios build -p my-app --release --device \
  --signing-identity "Apple Development: ..." \
  --provisioning-profile app.mobileprovision
```

The default target is the host's iOS Simulator. Device builds produce an IPA;
installation requires a matching certificate, provisioning profile, and entitlements.
Application names, identifiers, and icons come from `tessera-app.toml`.
Optional `[package.metadata.tessera.ios]` paths are relative to the package:

```toml
[package.metadata.tessera.ios]
info-plist = "ios/Info.plist"
entitlements = "ios/app.entitlements"
resources = "ios/resources"
```

Set `IPHONEOS_DEPLOYMENT_TARGET` to override the default iOS 15 minimum.

### Build for Android

Make sure Android SDK/NDK are installed and `adb` is available in your PATH.

Android helpers live under the dedicated subcommand:

```bash
# Initialize the Android Gradle project (required once)
cargo tessera android init

# Build APK/AAB via Gradle
cargo tessera android build --release --format apk

# Run the app on a device/emulator (device id is required)
cargo tessera android dev --device 8cd1353b
```

## License

Licensed under either of [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE) at your option.
