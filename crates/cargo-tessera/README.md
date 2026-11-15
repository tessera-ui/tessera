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
Packages are written to `target/packages/<package>`. Use
`[package.metadata.packager]` in `Cargo.toml` for
[signing, installer settings, and additional resources](https://docs.crabnebula.dev/packager/configuration/);
the application name, identifier, version, and icon come from `tessera.toml`
and Cargo.

### Configuration

Each package may have a `tessera.toml` next to its `Cargo.toml`:

```toml
[app]                       # read only from the package being built
name = "My App"             # default: the Cargo package name
identifier = "com.example.myapp"
icon = "icon.svg"

[assets]                    # enables assets for this package
dir = "assets"              # default: "assets"
embed = ["fonts/*.ttf"]     # always embedded in the binary
keep = ["licenses/**"]      # delivered even without a reference in the binary

[i18n]
dir = "locales"             # default: "locales"
fallback = "en"

[plugin.android]            # plugin packages only
module = "my_plugin"        # Gradle library in the package's android directory
```

Unknown keys are errors. `identifier` is a reverse-domain name with ASCII
alphanumeric segments starting with letters. Patterns are globs relative to the
asset directory; `*` does not cross directories, `**` does. Without `icon`, the Tessera logo is used. Icons must be
square: SVG is recommended (convert text to paths and embed external
resources); PNG should be at least 1024 × 1024 pixels. Tessera generates every
platform icon format from it, and `res::application()` applies the name, icon,
and identifier to the desktop window.

Versions come from Cargo. Android `versionCode` is
`major * 1_000_000 + minor * 1_000 + patch + 1`.

Platform settings such as permissions, SDK versions, entitlements, and signing
are written in each platform's own files: the generated Gradle and Xcode
projects, or `[package.metadata.packager]` for desktop packages.

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

Android pull + analyze in one step (the application ID defaults to
`[app] identifier`; pass `--app-id` to override it):

```bash
cargo tessera profiling analyze-android \
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

### Build for iOS and macOS with Xcode

On macOS with Xcode and [XcodeGen](https://github.com/yonaskolb/XcodeGen)
(`brew install xcodegen`):

```sh
# Generate the Xcode project in gen/apple (once)
cargo tessera apple init -p my-app

# Build a simulator app, or a device IPA
cargo tessera ios build -p my-app
cargo tessera ios build -p my-app --release --device
```

`gen/apple` belongs to your project: commit it and open the `.xcodeproj` in
Xcode. It contains an iOS and a macOS target. Set the signing team,
capabilities, and entitlements in Xcode, and add usage descriptions and other
keys to `iOS/Info.plist` or `macOS/Info.plist`. Device IPAs use
`ExportOptions.plist`. Both targets run `cargo tessera apple rust-build` as a
build phase, so building in Xcode also builds the Rust executable and its
assets.

cargo-tessera rewrites `tessera.xcconfig` (bundle identifier, display name,
version) and `Tessera.xcassets` (app icon) on every Apple command; both are
excluded by the generated `.gitignore` and referenced by the project. Run any
Apple command once after a fresh clone. `cargo tessera apple init --force`
regenerates the editable files, replacing local changes.

Distribute macOS apps outside the App Store with `cargo tessera package`
(`--formats app,dmg`). For the Mac App Store, archive the macOS target in
Xcode (Product → Archive).

### Build for Android

Install the Android SDK and NDK, a JDK supported by Gradle 8.14 (17 or 21), and
make `adb` available in your PATH.

```bash
# Generate the Gradle project in gen/android (once)
cargo tessera android init

# Build an APK or AAB
cargo tessera android build --release --format apk

# Install and run on a device, redeploying when sources change
cargo tessera android dev --device 8cd1353b
```

`gen/android` belongs to your project: commit it and edit it like any Android
project. Declare permissions in `app/src/main/AndroidManifest.xml`, and set
`minSdk`, Gradle plugins, and dependencies in `app/build.gradle.kts`.
cargo-tessera does not modify these files after `init`.

On every Android command, cargo-tessera rewrites the files it maintains, which
the generated `.gitignore` excludes:

| File | Contents | Referenced from |
| --- | --- | --- |
| `tessera.properties` | Application ID, version, library name, Rust project path | `app/build.gradle.kts` |
| `tessera.settings.gradle` | Plugin modules from Cargo dependencies | `settings.gradle` |
| `tessera.build.gradle.kts` | Plugin module dependencies | `app/build.gradle.kts` |
| `buildSrc/`, `tessera-plugins/` | Rust build integration and plugin sources | Gradle |
| `app/src/main/tessera-res/` | Application name and icons | `AndroidManifest.xml` |
| `app/src/main/assets/tessera/` | Platform assets | APK assets |

A fresh clone needs one cargo-tessera Android command before Android Studio
can sync. `cargo tessera android init --force` regenerates the editable files,
replacing local changes, for example after a template update.

## License

Licensed under either of [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE) at your option.
