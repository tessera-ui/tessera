## [v1.0.0] - 2026-11-16

Requires Rust 1.98 or newer and targets Tessera 3.

### Breaking changes

- **Configuration lives in `tessera.toml`.** Each package may have a
  `tessera.toml` next to its `Cargo.toml` with `[app]` (name, identifier,
  icon), `[assets]`, `[i18n]`, and `[plugin.android]` tables. Keys are
  kebab-case and unknown keys are errors. `[package.metadata.tessera.android]`
  and `[package.metadata.tessera.ios]` are rejected: the application ID comes
  from `[app] identifier`, architecture and format are command-line options,
  and other platform settings are edited in the generated platform projects.

  ```toml
  # before (Cargo.toml)
  [package.metadata.tessera.android]
  package = "com.example.myapp"
  arch = "arm64"
  format = "apk"

  # after (tessera.toml); pass --arch / --format on the command line
  [app]
  identifier = "com.example.myapp"
  ```

- **Android builds no longer use xbuild.** `cargo tessera android init`
  generates a Gradle project (using `GameActivity`) in `gen/android` that
  belongs to the application: commit it and edit permissions, SDK levels,
  Gradle plugins, and dependencies there. Android commands only rewrite the
  files cargo-tessera maintains (`tessera.properties`, plugin module scripts,
  `buildSrc`, generated resources and assets). Run
  `cargo tessera android init --force` to regenerate the editable files after
  a template update.
- **iOS builds go through Xcode.** `cargo tessera apple init` generates an
  Xcode project in `gen/apple` with iOS and macOS targets, and
  `cargo tessera ios build` builds it with `xcodebuild` (`--device` for an
  IPA). Signing, capabilities, and entitlements are configured in Xcode; the
  command has no signing options.
- **Web hosts.** `gen/web/index.html` must contain the `<!-- tessera-app -->`
  block, which is the only part cargo-tessera rewrites. Run
  `cargo tessera web init` to generate a host.
- **Option changes.** `android dev` requires `--device <serial>` (from
  `adb devices`); `--arch` accepts `arm64`, `arm`, `x86`, or `x86_64`;
  `new --template` accepts `basic` or `blank`; `profiling analyze-android`
  takes the application ID with `--app-id` (defaulting to `[app] identifier`),
  and `--package` selects a Cargo package; `web dev` accepts
  `-F/--features`.
- **Build-time checks.** `build`, `dev`, `android`, and `web` commands check
  Tessera call boundaries before compiling and fail on violations.
- New projects use the Tessera 3 templates (a library crate with a
  `#[tessera_ui::entry]` function, a `tessera.toml`, a `build.rs` calling
  `tessera_build::build()`, and a web host).

### Added

- `cargo tessera check` checks Tessera execution contexts (components, render
  slots, and Tessera-only APIs) with rust-analyzer before running
  `cargo check`, with stable diagnostic codes, `--explain`, named lints, and
  rustc-style lint levels (`-A`/`-W`/`-D`/`--forbid`/`--cap-lints`, or
  `#[tessera_ui::lint(...)]`).
- `cargo tessera fix` applies machine-applicable Tessera suggestions and
  `cargo fix`.
- `cargo tessera web init`, `web build`, and `web dev` for WebAssembly builds
  with a generated browser host and local dev server.
- `cargo tessera apple init` and `cargo tessera ios build` for iOS simulator
  apps and device IPAs.
- `cargo tessera package` builds desktop installers and bundles through
  cargo-packager (`nsis`, `wix`, `app`, `dmg`, `deb`, `appimage`, `pacman`),
  using the name, identifier, version, and icon from `tessera.toml` and Cargo.
- Platform icons and application metadata are generated from one icon
  (the Tessera logo by default).
- `cargo tessera headless` runs scripted offscreen sessions over JSONL stdio:
  render frames to PNG, inject input, take snapshots, and resize.
- Profiling: `--profiling-output` for `dev`, `build`, and `android dev`;
  `cargo tessera dev --top` for a live component view; and
  `cargo tessera profiling analyze` / `analyze-android`.
- `--debug-dirty-overlay` highlights regions rebuilt by the runtime.
- `cargo tessera skill` installs the `tessera-good-taste` agent skill locally
  or globally (`--local`, `--global`, `--path`, `--force`), and scaffolds
  include an `AGENTS.md`.
- `cargo tessera plugin new` scaffolds Tessera plugins, including Android
  plugin modules.
- `cargo tessera dev` also watches app configuration, resources, and
  localization catalogs, and debounces rebuilds.

### Performance

- Call checking reuses the rust-analyzer analysis across rebuilds in `dev`
  and across passes of `fix`, resolves dependency crates concurrently, bounds
  retained syntax trees, and reuses the selected `cargo check` output, which
  lowers time and memory use.
- Check results are cached per crate across invocations.
- Android builds check each target once instead of running a separate check
  before Gradle.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.5.0...cargo-tessera-v1.0.0)

## [v0.5.0] - 2025-11-18 +08:00

### Changes

- feat: add Windows subsystem configuration to prevent additional console window

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.4.1...cargo-tessera-v0.5.0)

## [v0.4.1] - 2025-11-15 +08:00

### Changes

- fix(cargo-tessera): Add suffix to template's main.rs to avoid incorrect formatting

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.4.0...cargo-tessera-v0.4.1)

## [v0.4.0] - 2025-11-15 +08:00

### Changes

- feat(dev): add release mode option to development server command
- chore(deps): bump indicatif from 0.18.2 to 0.18.3
- feat(accessibility): integrate AccessKit for screen reader support

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.3.0...cargo-tessera-v0.4.0)

## [v0.3.0] - 2025-11-09 +08:00

### Changes

- feat(cargo-tessera): add workspace package selection support

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.2.1...cargo-tessera-v0.3.0)

## [v0.2.1] - 2025-11-08 +08:00

### Changes

- docs(cargo-tessera/dev): clarify rebuild behavior
- docs: sync android instructions across languages
- docs(cargo-tessera): add zh translation

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.2.0...cargo-tessera-v0.2.1)

## [v0.2.0] - 2025-11-08 +08:00

### Changes

- fix(templates): rename cargo files for packaging
- fix(cargo-tessera): include templates in package
- style: format codes
- fix(android): require device for run command
- style(cli): replace emoji with colored text
- feat(templates): add android-ready lib targets
- feat(android): add dedicated subcommand
- fix(build): streamline android xbuild errors
- feat(build): add android xbuild support
- fix(dev): exit loop when child terminates
- feat(new): modernize prompts with inquire
- feat(cli): restyle new summary with comfy-table
- feat(new): show project summary card
- refactor(main): clean up unused imports and enhance logging setup
- docs(templates): describe basic scaffold accurately
- fix(dev): limit watcher scope to project files
- feat(dev): improve watcher coverage and resilience
- feat(new): support nested template directories
- feat(cargo-tessera): add CLI tool for Tessera projects

