## [v0.1.0] - 2026-11-16

### Added

- First release. Reactive Fluent localization for Tessera applications:
  `tessera-build` embeds catalogs configured in `tessera.toml`, the generated
  `res::i18n()` package registers the language at the entry, and messages
  resolved through `res::I18N` update when the language changes.
