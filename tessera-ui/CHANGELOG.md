## [v2.4.1] - 2025-11-17 +08:00

### Changes


[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.4.0...tessera-ui-v2.4.1)

## [v2.4.0] - 2025-11-15 +08:00

### Changes

- perf(core): use smallvec for hot buffers
- fix: remove wrong test
- fix(compute): fix scheduling bug in compute pipeline
- perf(renderer): add separate sampling/collision padding for barrier batching
- refactor: clean up incomplete documentation comments
- fix(accessibility): keep AccessKit tree in sync
- feat(accessibility): integrate AccessKit for screen reader support

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.3.0...tessera-ui-v2.4.0)

## [v2.3.0] - 2025-11-08 +08:00

### Changes

- feat: add speed limit for inertial scrolling

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.2.0...tessera-ui-v2.3.0)

## [v2.2.0] - 2025-11-03 +08:00

### Changes

- feat: Refine touch inertia sampling
- build(deps): update deps
- perf(blur): restructure pipeline around dual pass
- feat(dp): support mul/div float values
- feat(compute): copy scene before blur dispatch
- feat(renderer/compute): batch compute dispatch support
- feat: Optimize mobile touch
- fix(android): App freezes when returning from background
- perf(renderer): disable MSAA by default
- perf(renderer/reorder): bias scheduling toward contiguous batches
- test(reorder): enhance test cases
- fix(fluid_glass): Fix abnormal black edge refraction
- perf(renderer/reorder): Prioritize processing instruction batches with larger quantities
- perf: Replace full-screen SDF and GPU vertex generation in shape pipeline with local SDF and draw_indexed instanced rendering
- chore(deps): bump libc from 0.2.176 to 0.2.177
- build(deps): update dependencies
- refactor(renderer): Refactor ping-pong rendering
- fix(renderer): allow merging into batch when no previous command type is present
- refactor(renderer): remove insignificant PassTarget wrapper
- perf(renderer): avoid full texture copy during ping-pong rendering by using blit shader to sample specified range to target texture, reducing overhead
- style: make formater happy
- fix(ime): avoid redundant ime state toggles
- perf(renderer): avoid useless texture copy
- Merge pull request #63 from tessera-ui/dependabot/cargo/bytemuck-1.24.0
- chore(deps): bump parking_lot from 0.12.4 to 0.12.5
- chore(deps): bump bytemuck from 1.23.2 to 1.24.0
- chore(deps): bump petgraph from 0.8.2 to 0.8.3
- chore(deps): bump windows from 0.62.0 to 0.62.2
- chore(deps): bump libc from 0.2.175 to 0.2.176

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.1.0...tessera-ui-v2.2.0)

## [v2.1.0] - 2025-09-22 +08:00

### Changes

- fix(renderer): correctly handle lost/outdated surface errors
- feat(renderer): add register_draw_pipeline/register_compute_pipeline and update call sites
- docs(tessera-ui): update DrawablePipeline.draw docs to describe batch rendering and scene_texture_view
- docs(component_tree): update doc examples in constraint.rs to use explicit DimensionValue:: variants
- refactor(constraint): use `Self::` for `Wrap` variant in default implementation and change `get_max` to a const function
- refactor(clipboard): add #[must_use] attribute to Clipboard::new() method
- refactor(app): simplify return logic in apply_clip_to_batch_rect function
- refactor(cursor): rename scroll variables for clarity in inertia handling
- refactor(color): replace `Color::` with `Self::` for color constants and add #[must_use] to relevant methods
- refactor(component_tree): replace [`DimensionValue::`](tessera-ui/src/component_tree/constraint.rs:1) with [`Self::`](tessera-ui/src/component_tree/constraint.rs:1) and update doc examples
- refactor(command): simplify enum variant usage and remove examples

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.0.0...tessera-ui-v2.1.0)

## [v2.0.0] - 2025-09-17 +08:00

### Changes

- feat(shape_def): change RoundedRectangle corner radii to Dp
- feat(tessera-ui): add event clipping and clip propagation for input handling and draw commands

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.2...tessera-ui-v2.0.0)

## [v1.13.2] - 2025-09-15 +08:00

### Changes

- fix(renderer): improve error handling during rendering process
- fix(renderer): ensure full redraw on surface resize

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.1...tessera-ui-v1.13.2)

## [v1.13.1] - 2025-09-13 +08:00

### Changes

- refactor(input-handler): rename state_handler → input_handler across code and docs

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.0...tessera-ui-v1.13.1)

## [v1.13.0] - 2025-09-12 +08:00

### Changes

- feat(color): add TEAL and ORANGE color constants with descriptions
- feat(renderer, basic-components, example)!: propagate clip-aware drawing and clamp Fluid Glass sampling
- fix(test): update examples to new Router API and clean up docs
- refactor(example): rework(ing) demos to better showcase components and updated APIs

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.12.1...tessera-ui-v1.13.0)

## [v1.12.1] - 2025-09-07 +08:00

### Changes

- docs(basic-components): refine rustdoc, unify intra-doc links, add module docs, tidy examples

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.12.0...tessera-ui-v1.12.1)

## [v1.12.0] - 2025-09-06 +08:00

### Changes

- refactor(color): make methods to const for better optimization
- fix(renderer): suppress unwrap on dummy render to avoid panic
- feat(bottom-nav-bar): implement bottom navigation bar component and simplify routing API
- chore(deps): bump windows from 0.61.3 to 0.62.0
- refactor(renderer): remove debug println from render loop

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.2...tessera-ui-v1.12.0)

## [v1.11.2] - 2025-09-05 +08:00

### Changes

- perf(renderer): add dummy render on clean frames to drive immediate-mode updates with lower cost
- Revert "perf(renderer): throttle frames based on dynamic monitor refresh rate"
- perf(renderer): throttle frames based on dynamic monitor refresh rate
- refactor(renderer): set alpha mode to Auto for improved compatibility
- Revert "refactor(renderer): simplify surface configuration using AutoVsync and automatic composite alpha"
- Revert "fix(renderer): change present mode from AutoVsync to Fifo for improved frame synchronization"

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.1...tessera-ui-v1.11.2)

## [v1.11.1] - 2025-09-04 +08:00

### Changes

- fix(renderer): change present mode from AutoVsync to Fifo for improved frame synchronization
- fix(renderer): take position and size into account for dirty frame detection

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.0...tessera-ui-v1.11.1)

## [v1.11.0] - 2025-09-04 +08:00

### Changes

- refactor(renderer): simplify surface configuration using AutoVsync and automatic composite alpha
- docs(command): add example struct for implementing DrawCommand trait
- refactor(renderer.compute): replace Uuid-based resource IDs with sequential usize indices
- perf(renderer): add dirty frame detection and dynamic command equality
- perf(renderer): Optimize pipeline dispatch to O(1)
- chore(deps): bump uuid from 1.18.0 to 1.18.1
- refactor(renderer): use imported tracing::instrument attribute
- refactor(logging): replace log/flexi_logger/android_logger with tracing
- feat(renderer): add entry point wrapper to ensure proper tessera macro usage

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.2...tessera-ui-v1.11.0)

## [v1.10.2] - 2025-08-26 +08:00

### Changes

- refactor(lib, renderer): clean up example code and improve formatting in documentation
- refactor(image, constraint): replace to_max_px with get_max for constraint handling
- refactor(tessera-ui): reorganize DimensionValue implementation structure
- fix(cursor): improve cursor position cleanup to prevent premature clearing
- chore(deps): bump arboard from 3.6.0 to 3.6.1
- chore: add new line in changelog sections for better readability
- fix(text-editor/renderer): improve text selection rendering and clipping system

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.1...tessera-ui-v1.10.2)

## [v1.10.1] - 2025-08-25 +08:00

### Changes

- refactor: optimize loop logic and condition checks
- refactor(renderer, components): extract helpers and simplify rendering/compute flow
- fix(render): change current_batch_draw_rect to Option type for better handling of draw rectangles

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.0...tessera-ui-v1.10.1)

## [v1.10.0] - 2025-08-15 +08:00

### Changes

- fix(renderer): correct clip command handling logic
- Merge pull request #33 from tessera-ui/dependabot/cargo/rayon-1.11.0
- feat(clip): implement component clipping to prevent content overflow
- chore(deps): bump rayon from 1.10.0 to 1.11.0
- refactor(renderer/app): streamline main render loop with pass-based architecture
- feat(renderer/reorder): optimize instruction batching with batch_potential heuristic and improve stable sorting
- test(tests): update instruction reordering tests to account for non-deterministic order
- feat(renderer): optimize barrier batch draw logic and add PxRect::ZERO constant
- feat(renderer/app): implement conditional clear pass for initial rendering
- feat(renderer/reorder): optimize PriorityNode stable sorting and batch grouping, extend tests for instruction reordering
- feat(tests): add unit tests for instruction reordering logic
- Merge pull request #30 from tessera-ui/dependabot/cargo/uuid-1.18.0
- chore(deps): bump uuid from 1.17.0 to 1.18.0
- chore(deps): bump libc from 0.2.174 to 0.2.175
- feat(example-calculator): add CalStyle enum and CLI option for style switching, refactor keyboard and background to support glass and material styles, introduce Color::GRAY constant, update dependencies for clap support
- perf(renderer/reorder): improve PriorityNode sorting by adding type_id for finer-grained priority control
- refactor(renderer,pipelines,logo): unify import order, optimize pipeline interfaces for batched command processing, simplify logo component structure
- refactor(renderer): remove debug print statements from command processing for cleaner output
- perf(renderer): refactor rendering pipeline interfaces for batched command processing and optimize dependency graph rules
- perf(renderer): batch draw commands and optimize fluid_glass pipeline
- feat(renderer): add scene_texture_view parameter to render pass methods for improved pipeline flexibility and future glass morphism support
- chore(deps): bump bytemuck from 1.23.1 to 1.23.2

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.9.1...tessera-ui-v1.10.0)

## [v1.9.1] - 2025-08-08 +08:00

### Changes

- fix(tessera-ui): use positive() instead of abs() for rect clamping

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.9.0...tessera-ui-v1.9.1)

## [v1.9.0] - 2025-08-08 +08:00

### Changes

- refactor(renderer, deps): replace tokio with pollster and remove custom runtime
- perf(renderer): implement instruction reordering and scoped compute
- perf(renderer): implement scissor and batching for barrier commands
- feat(macros, shard): introduce declarative client-side routing
- feat(shard, macros): introduce shard state management and re-export macros
- feat(renderer): add window_title to TesseraConfig and support custom window title
- docs(runtime): include winit import in example and adjust formatting
- refactor(runtime): replace TesseraRuntime::read()/write() calls with with()/with_mut() closures and deprecate old methods

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.8.1...tessera-ui-v1.9.0)

## [v1.8.1] - 2025-08-03 +08:00

### Changes

- refactor(runtime)!: privatize window_size and add window_size() method

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.8.0...tessera-ui-v1.8.1)

## [v1.8.0] - 2025-08-02 +08:00

### Changes

- feat(glass-components): enhance border rendering with 3D bevel highlight
- chore(deps): bump tokio from 1.47.0 to 1.47.1

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.7.0...tessera-ui-v1.8.0)

## [v1.7.0] - 2025-07-31 +08:00

### Changes

- feat(basic-components,component_tree): unify cursor position API and add event blocking methods

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.6.0...tessera-ui-v1.7.0)

## [v1.6.0] - 2025-07-29 +08:00

### Changes

- docs(renderer): correct comment for TesseraConfig's default
- feat(scrollable): support Overlay/Alongside scrollbar layouts, always-visible by default, improve API
- build(deps): remove unused dependencies
- Merge pull request #20 from tessera-ui/dependabot/cargo/tokio-1.47.0
- chore(deps): bump tokio from 1.46.1 to 1.47.0
- chore(deps): bump winit from 0.30.11 to 0.30.12

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.5.2...tessera-ui-v1.6.0)

## [v1.5.2] - 2025-07-28 +08:00

### Changes

- fix(tessera-ui): ensure abs_position is calculated for all nodes

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.5.1...tessera-ui-v1.5.2)

## [v1.5.1] - 2025-07-25 +08:00

### Changes

- chore: update Cargo.toml to include homepage.workspace for all packages

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.5.0...tessera-ui-v1.5.1)

## [v1.5.0] - 2025-07-25 +08:00

### Changes

- feat(tessera-ui): add Color::lerp and refine Px methods
- feat(px): correct Px::abs behavior and add positive()/negative()
- feat(px): add mul_f32 and div_f32 methods
- feat(cursor): impl PartialEq for cursor event types
- refactor(component_tree): remove unused node_id from InputHandlerInput
- feat(dp): add ZERO constant
- feat(constraint): impl From<Px> and From<Dp> for DimensionValue
- feat(px): add Mul and Div implementations

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.4.0...tessera-ui-v1.5.0)

## [v1.4.0] - 2025-07-25 +08:00

### Changes

- feat(clipboard): add clear method to clipboard
- feat(clipboard): add actual support of clipboard for android.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.3.0...tessera-ui-v1.4.0)

## [v1.3.0] - 2025-07-24 +08:00

### Changes

- fix(clipboard): Add no_run attribute to clipboard documentation code examples
- fix(node): ensure metadata is reset and initialized for each node during measurement
- fix(node): ensure metadata exists for nodes during measurement
- feat(tessera-ui): add convenient constants to DimensionValue

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.2.0...tessera-ui-v1.3.0)

## [v1.2.0] - 2025-07-24 +08:00

### Changes

- docs(tessera-ui): add comprehensive documentation for clipboard module
- feat(clipboard): introduce core clipboard abstraction
- Fix non-posix pthread_setname_np call for apple platform

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.1.0...tessera-ui-v1.2.0)

## [v1.1.0] - 2025-07-23 +08:00

### Changes

- chore: transfer repo to https://github.com/tessera-ui/tessera
- refactor(core): provide ergonomic helpers on `MeasureInput`

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.0.0...tessera-ui-v1.1.0)

## [v1.0.0] - 2025-07-21 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.5.0...tessera-ui-v1.0.0)

## [v0.5.0] - 2025-07-21 +08:00

### Changes

- feat(tessera-ui): expose keyboard modifier state for shortcuts

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.4.0...tessera-ui-v0.5.0)

## [v0.4.0] - 2025-07-20 +08:00

### Changes

- feat(tessera-ui): add on_close callback for window close events

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.3.0...tessera-ui-v0.4.0)

## [v0.3.0] - 2025-07-20 +08:00

### Changes

- perf(tessera-ui): implement viewport culling and disable MSAA by default
- fix(renderer): allow window manager to handle resize cursors at edges
- feat(runtime): add minimize state handling and callback system

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.2.1...tessera-ui-v0.3.0)

## [v0.2.1] - 2025-07-19 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.2.0...tessera-ui-v0.2.1)
