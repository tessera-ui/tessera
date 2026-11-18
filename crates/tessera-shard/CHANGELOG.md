## [v1.0.0] - 2026-11-16

### Breaking changes

- **Renamed from `tessera-ui-shard`.** Depend on `tessera-shard` and import
  from `tessera_shard`; `tessera_ui` no longer re-exports the crate, the
  `shard` macro, or a router module.
- **Routers are scoped instead of global.** `Router::with_mut` and
  `router_root` are replaced by `shard_home`, which owns a
  `RouterController`; pages read it with `use_router()`.

  ```rust
  // before
  Router::with_mut(|router| router.push(DetailsDestination { id }));

  // after
  #[tessera]
  fn app() {
      shard_home().root(HomeDestination {});
  }

  #[shard]
  fn home() {
      let router = use_router();
      // in a callback:
      router.with_mut(|router| router.push(DetailsDestination { id: 1 }));
  }
  ```

- **Shard state is replaced by models.** `ShardState`,
  `ShardStateLifeCycle`, `ShardRegistry`, the `#[state]` parameter, and
  `task_handles` are removed. Take a `Model<T>` parameter in a `#[shard]`
  function; each route entry owns a `ModelStore`, so the model lives until
  its route is popped. Use `tessera_ui::Task` for cancellable work.
- `RouterDestination` only renders its page (`fn render(&self)`); it no longer
  executes components directly or exposes a destination ID.

### Added

- `RouterController` methods `push`, `pop`, `replace`, `reset`, `clear`,
  `last`, `len`, and `current_is`, and `shard_home().controller(..)` to share
  an external router state.
- Android system back pops one route, and predictive back gestures preview
  the previous route.
- Navigation works on the web.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.2...tessera-shard-v1.0.0)

## [v0.4.2] - 2025-11-17 +08:00

### Changes

- fix(tessera-ui-shard): Correct router documentation

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.1...tessera-ui-shard-v0.4.2)

## [v0.4.1] - 2025-11-03 +08:00

### Changes

- chore(deps): bump tokio from 1.47.1 to 1.48.0
- chore(deps): bump parking_lot from 0.12.4 to 0.12.5

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.4.0...tessera-ui-shard-v0.4.1)

## [v0.4.0] - 2025-09-12 +08:00

### Changes

- feat(shard, macros, router)!: destination-controlled shard state lifecycle with optional lifecycle argument
- refactor(example): rework(ing) demos to better showcase components and updated APIs

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.3.1...tessera-ui-shard-v0.4.0)

## [v0.3.1] - 2025-09-07 +08:00

### Changes

- style(bottom-nav-bar): format imports and docs(router) comment wrapping

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.3.0...tessera-ui-shard-v0.3.1)

## [v0.3.0] - 2025-09-06 +08:00

### Changes

- feat(bottom-nav-bar): implement bottom navigation bar component and simplify routing API

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.2.1...tessera-ui-shard-v0.3.0)

## [v0.2.1] - 2025-08-26 +08:00

### Changes

- chore: add new line in changelog sections for better readability

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-shard-v0.2.0...tessera-ui-shard-v0.2.1)

## [v0.2.0] - 2025-08-08 +08:00

### Changes

- fix(task_handle): make handle field public for external access
- fix(task_handles): ensure all tasks are canceled on TaskHandles drop
- feat(shard): add async task management with tokio and improve router API
- feat(shard): implement automatic memory management for ShardState
- feat(macros, shard): introduce declarative client-side routing
- feat(shard, macros): introduce shard state management and re-export macros
- feat(shard): implement core runtime for ShardRegistry
- feat(shard): initialize tessera-ui-shard crate and integrate into workspace
