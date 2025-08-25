## [v1.9.0] - 2025-08-25 +08:00

### Changes

- feat(text-editor): replace line-based scrolling with smooth pixel-based scrolling
- refactor: optimize loop logic and condition checks
- fix(tessera-ui-basic-components): enforce Fill constraint handling and fix row/column/dialog layout logic
- fix(switch): restore correct on_toggle behavior and stabilize state handling
- fix(pipelines): correct x-coordinate sign in pixel_to_ndc function
- docs(pipelines): remove example from pixel_to_ndc doc comment
- refactor(renderer, components): extract helpers and simplify rendering/compute flow
- fix(fluid_glass): change cursor event from pressed to released
- feat(fluid_glass): support independent corner radii
- feat(bottom_sheet): introduce glass style for scrim
- refactor(dialog): unify glass dialog and introduce dialog styles

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.8.0...tessera-ui-basic-components-v1.9.0)

## [v1.8.0] - 2025-08-15 +08:00

### Changes

- refactor(fluid_glass): optimize border anti-aliasing and highlight logic
- docs(shape): update examples to reflect independent corner radii API
- fix(renderer): correct clip command handling logic
- feat(components): introduce BottomSheet and independent corner radii
- feat(clip): implement component clipping to prevent content overflow
- fix(pipelines/text): fix doctest type errors in TextData example for Color and TextConstraint
- refactor(renderer,pipelines,logo): unify import order, optimize pipeline interfaces for batched command processing, simplify logo component structure
- perf(renderer): refactor rendering pipeline interfaces for batched command processing and optimize dependency graph rules
- perf(pipelines/shape): optimize shape rendering pipeline with instance-based uniforms and batched draw, update WGSL for multi-instance support
- perf(text-pipeline): optimize GlyphonTextRender with batched command collection and improved renderer reuse
- perf(renderer): batch draw commands and optimize fluid_glass pipeline
- perf(pipelines/fluid_glass): optimize FluidGlassPipeline with dynamic uniform buffer offset, improve multi-component rendering performance.
- chore(deps): bump bytemuck from 1.23.1 to 1.23.2

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.7.1...tessera-ui-basic-components-v1.8.0)

## [v1.7.1] - 2025-08-08 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.7.0...tessera-ui-basic-components-v1.7.1)

## [v1.7.0] - 2025-08-08 +08:00

### Changes

- perf(renderer): replace manual padding with zero padding for FluidGlassCommand barrier
- fix(checkbox): correct typo in documentation comment for CheckmarkState
- perf(renderer): implement instruction reordering and scoped compute
- perf(renderer): implement scissor and batching for barrier commands
- feat(macros, shard): introduce declarative client-side routing
- feat(shard, macros): introduce shard state management and re-export macros
- feat(fluid_glass): update example to use image background with Builder API, set default noise_amount to 0.0

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.6.1...tessera-ui-basic-components-v1.7.0)

## [v1.6.1] - 2025-08-03 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.6.0...tessera-ui-basic-components-v1.6.1)

## [v1.6.0] - 2025-08-03 +08:00

### Changes

- fix(tessera-ui-basic-components): include descender in text layout height calculation
- feat(tessera-ui-basic-components): use Dp for borders, add max_blur_radius & overlay blend highlight, update examples
- feat(glass_progress): add glassmorphism-style progress bar component and showcase example
- refactor(fluid_glass): enhance shape rendering and border highlight effects

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.5.0...tessera-ui-basic-components-v1.6.0)

## [v1.5.0] - 2025-08-02 +08:00

### Changes

- refactor(progress): optimize progress bar visual design by removing shape parameter and implementing height-based auto rounded corners
- feat(fluid_glass): add contrast property and set default tint to transparent
- refactor(glass_slider): use fluid_glass for progress indicator
- feat(glass-components): enhance border rendering with 3D bevel highlight
- refactor(glass_dialog_showcase,glass_dialog): optimize glass dialog and button visual parameters, remove blur_radius field for simpler configuration

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.4.0...tessera-ui-basic-components-v1.5.0)

## [v1.4.0] - 2025-07-31 +08:00

### Changes

- style: make scripts\check-imports.rs happy
- chore(example): remove unused glass_dialog_showcase example entry from Cargo.toml
- feat(glass-dialog): add modal glass dialog component and showcase example
- feat(animation): add cubic ease-in-out animation module and unify easing logic in dialog, glass_switch, and switch components
- build(deps): bump glyphon-tessera-fork from 0.9.0 to 0.9.1

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.3.1...tessera-ui-basic-components-v1.4.0)

## [v1.3.1] - 2025-07-31 +08:00

### Changes

- fix(fluid_glass): align ripple default behavior with glass_button
- refactor(example): remove custom Surface interactive demo and related state
- fix(dialog): improve doc example to demonstrate usage of color with alpha in button and text

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.3.0...tessera-ui-basic-components-v1.3.1)

## [v1.3.0] - 2025-07-31 +08:00

### Changes

- feat(dialog): add content_alpha parameter to dialog_content for animated opacity
- fix(surface, dialog): add block_input to surface and dialog to block all input events
- docs(dialog): improve doc example for dialog_provider
- feat(dialog): refactor DialogProvider state management and API, add animation support
- feat(basic-components,component_tree): unify cursor position API and add event blocking methods
- docs(dialog): rewrite doc example for dialog_provider with updated usage and API

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.2.1...tessera-ui-basic-components-v1.3.0)

## [v1.2.1] - 2025-07-29 +08:00

### Changes

- style(checkbox): adjust checkmark size and padding for better visual centering

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.2.0...tessera-ui-basic-components-v1.2.1)

## [v1.2.0] - 2025-07-29 +08:00

### Changes

- feat(scrollable): support Overlay/Alongside scrollbar layouts, always-visible by default, improve API
- feat(scrollable): add ScrollBarBehavior with AlwaysVisible, AutoHide, and Hidden modes
- chore(deps): bump glam from 0.30.4 to 0.30.5

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.1.3...tessera-ui-basic-components-v1.2.0)

## [v1.1.3] - 2025-07-28 +08:00

### Changes

- fix(tessera-ui-basic-components): correct layout calculation for fill dimension

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.2...tessera-ui-basic-components-v1.1.3)

## [v1.1.2] - 2025-07-28 +08:00

### Changes

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.1...tessera-ui-basic-components-v1.1.2)

## [v1.1.1] - 2025-07-25 +08:00

### Changes

- chore: update Cargo.toml to include homepage.workspace for all packages

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.0...tessera-ui-basic-components-v1.1.1)

## [v1.1.0] - 2025-07-25 +08:00

### Changes

- docs(scrollable): correct state initialization in doc example
- feat(scrollable): introduce reusable scrollbar and enhance scrollable component

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.2...tessera-ui-basic-components-v1.1.0)

## [v1.0.2] - 2025-07-24 +08:00

### Changes

- style(docs): normalize doc comments to standard format
- feat(clipboard): introduce core clipboard abstraction

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.1...tessera-ui-basic-components-v1.0.2)

## [v1.0.1] - 2025-07-24 +08:00

### Changes

- chore: transfer repo to https://github.com/tessera-ui/tessera
- refactor(core): provide ergonomic helpers on `MeasureInput`

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.0...tessera-ui-basic-components-v1.0.1)

## [v1.0.0] - 2025-07-21 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.4.1...tessera-ui-basic-components-v1.0.0)

## [v0.4.1] - 2025-07-21 +08:00

### Changes

- fix(layout): Correct `Fill` dimension behavior in Row and Column
- docs(ui-basic-components): add comprehensive rustdoc and examples

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.4.0...tessera-ui-basic-components-v0.4.1)

## [v0.4.0] - 2025-07-21 +08:00

### Changes

- feat(text-editor): clip selection highlight to visible area
- feat(text-editor): implement clipboard and shortcut support
- feat(text_editor): change cursor to text icon on hover

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.3.0...tessera-ui-basic-components-v0.4.0)

## [v0.3.0] - 2025-07-20 +08:00

### Changes

- feat(slider): change cursor to pointer on hover
- feat(slider): redesign to be thumb-less and add disabled state
- feat(glass_slider): redesign component for a modern, thumb-less look
- feat(fluid_glass): implement world-coordinate lighting for borders

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.2.0...tessera-ui-basic-components-v0.3.0)

## [v0.2.0] - 2025-07-19 +08:00

### Changes

- feat(glass_switch): add border support and enhance visuals
- feat(shape): add configurable G2-like corner continuity

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.1.0...tessera-ui-basic-components-v0.2.0)

## [v0.1.0] - 2025-07-19 +08:00

### Changes

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.2.0...tessera-ui-basic-components-v0.1.0)
