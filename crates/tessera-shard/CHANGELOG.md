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
