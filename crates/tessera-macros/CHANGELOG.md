## [v1.0.0] - 2026-11-16

### Breaking changes

- **Renamed from `tessera-ui-macros`.** Applications normally use the macros
  through `tessera_ui::{tessera, entry, lint}` and `tessera_shard::shard`;
  depend on `tessera-macros` only if you imported it directly.
- **`#[tessera]` generates a props type and builder.** The function's
  parameters become props: required parameters are entry arguments,
  `Option<T>` parameters become setters, `Callback` / `RenderSlot` parameters
  get closure and `*_shared` setters, and the entry returns the builder.
  `#[prop(into)]`, `#[prop(skip_setter)]`, and `#[prop(skip_eq)]` adjust the
  generated surface. Parameters must be owned `Clone + 'static` values.
- The injected `measure`, `input_handler`, `on_minimize`, and `on_close`
  helpers are removed; use `tessera_ui::layout::layout` with a `LayoutPolicy`,
  input modifiers, and plugin lifecycle hooks. `#[default(...)]` parameter
  attributes are removed; resolve `Option` defaults in the body.
- `#[tessera]` and `#[shard]` no longer accept attribute arguments.
- **`#[shard]` no longer injects `#[state]` parameters.** Other parameters
  become fields of the generated `*Destination`; take a `Model<T>` parameter
  for state that lives as long as the route, and read the router with
  `tessera_shard::use_router()`.

### Added

- `#[tessera_ui::entry]` on a shared `pub fn run() -> EntryPoint` generates
  the Android entry point; desktop, iOS, and web entries call `run()`.
- `#[tessera_ui::lint(...)]` sets Tessera lint levels for an item or module,
  applied by `cargo tessera check`.
- Generic `#[tessera]` components.
- `Model<T>` parameters are injected from the nearest `ModelStore`, or owned
  by the component when none is provided.
- Control-flow instrumentation keeps remembered state stable across `if`,
  `match`, and loops.

### Performance

- Generated components register plain required props directly and keep
  per-component generic code small, reducing compile time and code size.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.4...tessera-macros-v1.0.0)

## [v0.4.4] - 2025-11-15 +08:00

### Changes

- chore(deps): bump syn from 2.0.108 to 2.0.110

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.3...tessera-ui-macros-v0.4.4)

## [v0.4.3] - 2025-11-08 +08:00

### Changes

- chore(deps): bump quote from 1.0.41 to 1.0.42

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.2...tessera-ui-macros-v0.4.3)

## [v0.4.2] - 2025-11-03 +08:00

### Changes

- chore(deps): bump syn from 2.0.106 to 2.0.108
- chore(deps): bump quote from 1.0.40 to 1.0.41

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.1...tessera-ui-macros-v0.4.2)

## [v0.4.1] - 2025-09-13 +08:00

### Changes

- refactor(input-handler): rename state_handler → input_handler across code and docs
- docs(tessera-ui-basic-components): add README files and fix macros docs formatting

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.4.0...tessera-ui-macros-v0.4.1)

## [v0.4.0] - 2025-09-12 +08:00

### Changes

- feat(shard, macros, router)!: destination-controlled shard state lifecycle with optional lifecycle argument
- fix(docs): correct import path for tessera macro usage example

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.3.0...tessera-ui-macros-v0.4.0)

## [v0.3.0] - 2025-09-06 +08:00

### Changes

- feat(bottom-nav-bar): implement bottom navigation bar component and simplify routing API

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.4...tessera-ui-macros-v0.3.0)

## [v0.2.4] - 2025-08-26 +08:00

### Changes

- chore: add new line in changelog sections for better readability

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.3...tessera-ui-macros-v0.2.4)

## [v0.2.3] - 2025-08-25 +08:00

### Changes

- fix(switch): restore correct on_toggle behavior and stabilize state handling
- refactor(renderer, components): extract helpers and simplify rendering/compute flow
- chore(deps): bump syn from 2.0.105 to 2.0.106

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.2...tessera-ui-macros-v0.2.3)

## [v0.2.2] - 2025-08-15 +08:00

### Changes

- chore(deps): bump syn from 2.0.104 to 2.0.105
- fix(docs): update code block in shard function example to ignore syntax highlighting

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.1...tessera-ui-macros-v0.2.2)

## [v0.2.1] - 2025-08-08 +08:00

### Changes

- fix(tessera-ui-macros): update version to 0.2.0 and adjust syn dependency features

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.2.0...tessera-ui-macros-v0.2.1)

## [v0.1.7] - 2025-08-03 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.6...tessera-ui-macros-v0.1.7)

## [v0.1.6] - 2025-08-02 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.5...tessera-ui-macros-v0.1.6)

## [v0.1.5] - 2025-07-31 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.4...tessera-ui-macros-v0.1.5)

## [v0.1.4] - 2025-07-29 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-macros-v0.1.3...tessera-ui-macros-v0.1.4)

## [v0.1.3] - 2025-07-28 +08:00

### Changes

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-macros-v0.1.2...tessera-ui-macros-v0.1.3)

## [v0.1.2] - 2025-07-25 +08:00

### Changes

- chore: update Cargo.toml to include homepage.workspace for all packages

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-macros-v0.1.1...tessera-ui-macros-v0.1.2)

## [v0.1.1] - 2025-07-24 +08:00

### Changes

- chore: transfer repo to https://github.com/tessera-ui/tessera
- feat(tessera-ui): add on_close callback for window close events
- feat(runtime): add minimize state handling and callback system
- chore(workspace): switch tessera-ui dependencies from crates.io to local path for all workspace crates
- fix(dependencies): update tessera-ui dependency to version 0.1.3
- fix(version): update version of tessera-ui-basic-components and tessera-ui-macros to 0.0.0
- docs(readme): update relative links in READMEs to absolute URLs
- docs(readme): use absolute paths for images and links in READMEs
- chore(rename): rename `tessera` to `tessera-ui`
