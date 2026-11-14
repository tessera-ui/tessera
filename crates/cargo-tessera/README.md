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
optional `tessera-good-taste` skill. Project creation does not install the skill.

### Install the agent skill

Run this command in a project when you want Tessera application guidance:

```bash
cargo tessera skill
```

The command shows a compact selector below your shell output, with **Local**
selected initially and the destination of each scope previewed. Use Up/Down or
Tab to select and Enter to confirm. Esc, q, or Ctrl-C cancels without installing.

- Local: `<project>/.agents/skills/tessera-good-taste/` (current directory by default).
- Global: `~/.agents/skills/tessera-good-taste/`, using your operating-system home directory.

For scripts or non-interactive terminals, explicitly select one scope:

```bash
cargo tessera skill --local
cargo tessera skill --local --path my-app
cargo tessera skill --global
```

`--local` and `--global` are mutually exclusive and bypass the selector entirely;
they do not require a TTY. Without either flag, both stdin and stdout must be
terminals; otherwise the command fails with a scope-selection hint. `--path` is
local-only: it conflicts with `--global`, and the selector shows Global as
unavailable when `--path` is supplied.

Both scopes leave `AGENTS.md`, other skills, and unrelated files unchanged.
Matching files are left untouched; use `--force` to replace locally modified
bundled skill files. Invoke the installed skill as `$tessera-good-taste` in an
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
`cargo tessera dev` watches `src/`, `Cargo.toml`, and (if present) `build.rs`, then rebuilds and restarts the app whenever changes are saved. Pass `--verbose` to see the underlying `cargo` commands.

Enable profiler output during desktop dev:

```bash
cargo tessera dev -p example --profiling-output profiles/dev.jsonl
```

Enable dirty replay overlay during desktop dev:

```bash
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

Enable profiler instrumentation in desktop build artifacts:

```bash
cargo tessera build -p example --profiling-output profiles/build.jsonl
```

Enable dirty replay overlay in desktop build artifacts:

```bash
cargo tessera build -p example --debug-dirty-overlay
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

Tessera supplies platform icon formats, Android adaptive and monochrome icons,
iOS app icons, and web favicon, touch icon, and manifest metadata. Opaque icons
use a white background. Android adaptive artwork is inset to fit launcher masks;
monochrome icons use the source alpha silhouette.

The standard `tessera_build::build()` generates `res::application()`. Use it as
renderer defaults (`..res::application()`) to apply the display name, desktop
window icon, and launcher identifier. New projects include this setup.

Versions come from Cargo. Android derives `versionCode` as
`major * 1_000_000 + minor * 1_000 + patch + 1` (major < 2100, minor/patch < 1000);
prerelease/build labels do not change this code. iOS uses the numeric version.

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

The JSONL session protocol supports synthetic input, frame rendering, snapshots,
and viewport resizing without opening a window. Profiler records include
component build, layout/draw calculation, render graph preparation
(`render_prepare_ns`), command encoding, and queue submission timings. These
measure CPU wall time, not GPU execution or desktop presentation.

Desktop `frame_total_ns` also includes publishing accessibility updates and
applying cursor/IME requests. Headless sessions omit these platform operations;
compare desktop traces when offscreen measurements do not reproduce a stutter.

Set `TESSERA_HEADLESS_GPU_TIMING=1` to add `render_gpu_ns` timestamp measurements
on supported native adapters. This mode waits for GPU readback after each frame;
the wait is excluded from CPU frame timings and changes CPU/GPU overlap. GPU
timestamps cover the main frame command buffer, including its render, compute,
and copy passes; separate submissions used to populate pipeline caches are
excluded. Use `WGPU_POWER_PREF=low` or `high` to compare integrated and discrete
adapters, and check the adapter name in the startup log.
Set `TESSERA_HEADLESS_SCALE_FACTOR=1.5` or `2` to reproduce high-DPI rendering;
the configured viewport dimensions remain physical pixels. Headless sessions
default to a scale factor of 1, and ready/snapshot responses include the factor.

Enable `RUST_LOG=tessera_ui::renderer::core::gpu_timing=debug` with GPU timing to
log GPU duration per render-graph pass and the final output blit. Draw stages
include any background capture needed by sampled effects; compute stages include
their input preservation and dispatches.

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

`cargo tessera android dev` requires `--device <device_id>` (list devices with `adb devices`).

Enable profiler data on Android builds by passing a sandbox path:

```bash
cargo tessera android dev \
  --device 8cd1353b \
  --profiling-output files/tessera-profiler.jsonl
```

Enable dirty replay overlay on Android:

```bash
cargo tessera android dev \
  --device 8cd1353b \
  --debug-dirty-overlay
```

## License

Licensed under either of [MIT](../../LICENSE) or [Apache-2.0](../../LICENSE) at your option.
