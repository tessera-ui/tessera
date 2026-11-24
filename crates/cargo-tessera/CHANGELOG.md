## [v1.1.0] - 2026-11-24

- Generate mobile projects with native platform-view support. The component checker recognizes `popup` as a node-emitting API alongside `layout` and `group`.
- Keep piped stdin open until the dev server's cancelled child process exits.

[v1.1.0]: https://github.com/tessera-ui/tessera/compare/cargo-tessera-v1.0.0...cargo-tessera-v1.1.0

## [v1.0.0] - 2026-11-16

Requires Rust 1.98 or newer and targets Tessera 3.

### Breaking changes

- **Configuration uses `tessera.toml`.** Set application metadata in `[app]`,
  resources in `[assets]`, localization in `[i18n]`, and plugin options in
  `[plugin.android]`. Platform-specific permissions and signing stay in the
  generated platform projects.
- **Android projects are editable app files.** `cargo tessera android init`
  generates a Gradle project in `gen/android`; commit it and edit it like other
  Android project files.
- **Apple apps use Xcode projects.** `cargo tessera apple init` creates the
  project; `cargo tessera ios build` builds simulator apps or device IPAs.
- **Web apps use a generated host.** Run `cargo tessera web init` to create it.
- **Build commands check Tessera call usage.** Use `cargo tessera check` to see
  diagnostics before building.

### Added

- `cargo tessera check` and `fix` for Tessera diagnostics and compiler fixes.
- Web and Apple build commands, desktop installers, and Android project tools.
- `cargo tessera headless` for scripted offscreen rendering and input.
- Profiling output and a visual update overlay for development.
- `cargo tessera skill` and plugin scaffolding.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.5.0...cargo-tessera-v1.0.0)

## [v0.5.0] - 2025-11-18 +08:00

### Changes

- Windows applications no longer open an extra console window.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.4.1...cargo-tessera-v0.5.0)

## [v0.4.1] - 2025-11-15 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.4.0...cargo-tessera-v0.4.1)

## [v0.4.0] - 2025-11-15 +08:00

### Changes

- The development command supports release builds.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.3.0...cargo-tessera-v0.4.0)

## [v0.3.0] - 2025-11-09 +08:00

### Changes

- Commands can select a package in a Cargo workspace.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.2.1...cargo-tessera-v0.3.0)

## [v0.2.1] - 2025-11-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/cargo-tessera-v0.2.0...cargo-tessera-v0.2.1)

## [v0.2.0] - 2025-11-08 +08:00

### Changes

- Added Android project creation, build, and device-run commands.
- Project scaffolds support nested templates and Android library targets.
- The development server watches project files and exits when the app stops.
