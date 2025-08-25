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
