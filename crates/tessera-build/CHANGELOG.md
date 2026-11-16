## [v1.0.0] - 2026-11-16

### Breaking changes

- **`tessera.toml` replaces `tessera-config.toml`** (and the
  `tessera-app.toml` used by pre-release tooling). Keys are kebab-case and
  unknown keys are errors.

  ```toml
  # before: tessera-config.toml
  permissions = ["INTERNET"]
  [assets]
  dir = "assets"
  [plugin.android]
  module = "my_plugin"
  package = "com.example.plugin"

  # after: tessera.toml
  [app]
  name = "My App"
  identifier = "com.example.myapp"
  icon = "icon.svg"
  [assets]
  dir = "assets"
  embed = ["fonts/*.ttf"]
  keep = ["licenses/**"]
  [i18n]
  fallback = "en"
  [plugin.android]
  module = "my_plugin"
  ```

  `permissions` and `[plugin.android] package` are removed; declare
  permissions in the generated Android project. `[assets]` takes `embed` and
  `keep` glob lists (`*` stays within a directory, `**` crosses directories)
  instead of tree-shaking settings.
- `generate_assets()` is replaced by `tessera_build::build()`, which also
  generates application defaults and localization catalogs.
- Generated asset constants are `tessera_ui::Asset` values instead of a
  generated `Asset` type implementing `tessera_ui::AssetExt`; load them
  asynchronously with `tessera_ui::Resource`. The platform backend now also
  delivers web assets separately from the Wasm binary.
- `TesseraConfig` drops `permissions` and gains `app` and `i18n`;
  `AssetsConfig` gains `embed` and `keep`; `PluginAndroidConfig::module` is a
  required `String` and `package` is removed.

### Added

- `[app]` configuration with the `application` module (`AppConfig`,
  `AppIcon`): the generated `res::application()` applies the name, icon, and
  identifier to the window, and platform icons are generated from one square
  SVG or PNG. Applications without an icon use the Tessera logo, so
  `AppConfig::load_icon` always returns an icon.
- `[i18n]` configuration that embeds Fluent catalogs for `tessera-i18n`;
  each locale is parsed on first lookup.
- `prepare_assets` and `resolve_resource_dirs` for tools that package assets
  and watch resource directories.

### Fixed

- Build scripts no longer rerun on every build when no configuration file
  exists.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-build-v0.1.0...tessera-build-v1.0.0)
