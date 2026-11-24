## [v1.0.1] - 2026-11-24

- Build router components against Tessera 4.

[v1.0.1]: https://github.com/tessera-ui/tessera/compare/tessera-shard-v1.0.0...tessera-shard-v1.0.1

## [v1.0.0] - 2026-11-16

### Breaking changes

- **Renamed from `tessera-ui-shard`.** Depend on `tessera-shard` and import from
  `tessera_shard`; `tessera_ui` no longer re-exports the crate or router.
- **Routers are scoped.** Use `shard_home()` to create a `RouterController` and
  `use_router()` from a page to navigate.
- **Route state uses models.** Pass a `Model<T>` to a shard; each route owns its
  model until that route is popped.

### Added

- `RouterController` supports `push`, `pop`, `replace`, `reset`, `clear`, and
  inspection methods; `shard_home().controller(..)` can share a controller.
- Android system back and predictive back, plus web navigation.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.2...tessera-shard-v1.0.0)

## [v0.4.2] - 2025-11-17 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.1...tessera-ui-shard-v0.4.2)

## [v0.4.1] - 2025-11-03 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.0...tessera-ui-shard-v0.4.1)

## [v0.4.0] - 2025-09-12 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.3.1...tessera-ui-shard-v0.4.0)

## [v0.3.1] - 2025-09-07 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.3.0...tessera-ui-shard-v0.3.1)

## [v0.3.0] - 2025-09-06 +08:00

### Changes

- Added a bottom navigation component and simplified the routing API.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.2.1...tessera-ui-shard-v0.3.0)

## [v0.2.1] - 2025-08-26 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.2.0...tessera-ui-shard-v0.2.1)

## [v0.2.0] - 2025-08-08 +08:00

### Changes

- Added declarative navigation and route-owned state, with cancellable asynchronous tasks.
