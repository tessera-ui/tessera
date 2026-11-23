## [v1.0.0] - 2026-11-16

### Breaking changes

- **Renamed from `tessera-ui-macros`.** Most apps use the macros through
  `tessera_ui` or `tessera_shard`; direct imports should use `tessera-macros`.
- **`#[tessera]` parameters define component props.** Required parameters are
  passed at entry; optional values, callbacks, and child slots use generated
  setters.
- Use `layout(policy)`, input modifiers, and plugin lifecycle hooks instead of
  injected measurement, input, or window-lifecycle helpers.
- `#[shard]` parameters become route destination fields. Use `Model<T>` for
  state that lives with a route.

### Added

- `#[tessera_ui::entry]` provides a shared entry function for desktop, iOS, web,
  and Android apps.
- `#[tessera_ui::lint(...)]` configures Tessera lints on items and modules.
- Generic Tessera components and route-owned model parameters.
- Remembered state remains associated with its component as branches and loops
  change.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.4...tessera-macros-v1.0.0)

## [v0.4.4] - 2025-11-15 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.3...tessera-ui-macros-v0.4.4)

## [v0.4.3] - 2025-11-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.2...tessera-ui-macros-v0.4.3)

## [v0.4.2] - 2025-11-03 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.1...tessera-ui-macros-v0.4.2)

## [v0.4.1] - 2025-09-13 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.0...tessera-ui-macros-v0.4.1)

## [v0.4.0] - 2025-09-12 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.3.0...tessera-ui-macros-v0.4.0)

## [v0.3.0] - 2025-09-06 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.4...tessera-ui-macros-v0.3.0)

## [v0.2.4] - 2025-08-26 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.3...tessera-ui-macros-v0.2.4)

## [v0.2.3] - 2025-08-25 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.2...tessera-ui-macros-v0.2.3)

## [v0.2.2] - 2025-08-15 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.1...tessera-ui-macros-v0.2.2)

## [v0.2.1] - 2025-08-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.0...tessera-ui-macros-v0.2.1)

## [v0.1.7] - 2025-08-03 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.6...tessera-ui-macros-v0.1.7)

## [v0.1.6] - 2025-08-02 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.5...tessera-ui-macros-v0.1.6)

## [v0.1.5] - 2025-07-31 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.4...tessera-ui-macros-v0.1.5)

## [v0.1.4] - 2025-07-29 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.3...tessera-ui-macros-v0.1.4)

## [v0.1.3] - 2025-07-28 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-macros-v0.1.2...tessera-ui-macros-v0.1.3)

## [v0.1.2] - 2025-07-25 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-macros-v0.1.1...tessera-ui-macros-v0.1.2)

## [v0.1.1] - 2025-07-24 +08:00

### Changes

- Added callbacks for window-close and minimize events.
