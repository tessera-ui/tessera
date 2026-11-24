## [v0.1.1] - 2026-11-24

- Build localization catalogs against Tessera 4.

[v0.1.1]: https://github.com/tessera-ui/tessera/compare/tessera-i18n-v0.1.0...tessera-i18n-v0.1.1

## [v0.1.0] - 2026-11-16

### Added

- First release. Reactive Fluent localization for Tessera applications:
  `tessera-build` embeds catalogs configured in `tessera.toml`, the generated
  `res::i18n()` package registers the language at the entry, and messages
  resolved through `res::I18N` update when the language changes.
