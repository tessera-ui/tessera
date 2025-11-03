## [v2.2.0] - 2025-11-03 +08:00

### Changes

- fix(ui): align shape cache alpha handling
- feat: Batch cached shape draws
- build(deps): update deps
- perf(blur): restructure pipeline around dual pass
- feat(compute): copy scene before blur dispatch
- feat(blur): add downscaled sampling option
- fix(blur): restore fallback copy outside target area
- feat(renderer/compute): batch compute dispatch support
- perf: Adjust worker thread scope to reduce blur overhead
- feat: Optimize mobile touch
- style: format code
- perf(shape): adjust cache
- perf(basic-components): cache large rects and use simple rect pipeline
- fix(fluid_glass): Fix abnormal black edge refraction
- perf: Replace full-screen SDF and GPU vertex generation in shape pipeline with local SDF and draw_indexed instanced rendering
- chore(deps): bump lru from 0.16.1 to 0.16.2
- feat(boxed): Support specifying independent alignment for child components
- feat(alignment): change alignment default value to TopStart and fix test cases
- fix(fluid_glass): Fix vertex data inconsistency issue introduced by f1b057c2153a7d8e218f6dc05cbfcc128b128fa8
- build(deps): update dependencies
- feat(dialog): add configurable padding parameter
- Merge pull request #63 from tessera-ui/dependabot/cargo/bytemuck-1.24.0
- chore(deps): bump parking_lot from 0.12.4 to 0.12.5
- chore(deps): bump bytemuck from 1.23.2 to 1.24.0
- fix(fluid_glass): wrong refraction
- chore(deps): bump glam from 0.30.6 to 0.30.7

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.1.0...tessera-ui-basic-components-v2.2.0)

## [v2.1.0] - 2025-09-22 +08:00

### Changes

- feat(renderer): add register_draw_pipeline/register_compute_pipeline and update call sites
- chore(deps): bump glam from 0.30.5 to 0.30.6
- refactor(tessera-ui-basic-components): simplify was_pressed_left invocation

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.0.0...tessera-ui-basic-components-v2.1.0)

## [v2.0.0] - 2025-09-17 +08:00

### Changes


[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.15.0...tessera-ui-basic-components-v2.0.0)

## [v1.15.0] - 2025-09-17 +08:00

### Changes

- feat(shape_def): change RoundedRectangle corner radii to Dp
- feat(side-bar): add blur effect to side bar content wrapper

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.14.0...tessera-ui-basic-components-v1.15.0)

## [v1.14.0] - 2025-09-15 +08:00

### Changes

- fix(text-editor): add default implementation for TextEditorArgs to fix doc test
- feat(text-editor): add on_change callback and safe action handling
- feat(checkbox): make CheckboxState fields private and add constructor
- docs(image): update example to use `no_run` for better clarity
- docs(column): update example to include SpacerArgs for clarity
- docs(text): add spacing for example section
- docs(slider): update example documentation

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.13.1...tessera-ui-basic-components-v1.14.0)

## [v1.13.1] - 2025-09-13 +08:00

### Changes

- refactor(input-handler): rename state_handler → input_handler across code and docs
- docs(surface): make example tested
- docs(tessera-ui-basic-components): add tabs component docs
- docs(side_bar): add API docs, examples and doc comments for side_bar_provider
- docs(glass_switch): update usage reference in documentation
- docs(tessera-ui-basic-components): add README files and fix macros docs formatting

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.13.0...tessera-ui-basic-components-v1.13.1)

## [v1.13.0] - 2025-09-12 +08:00

### Changes

- feat(basic-components,example)!: rework tabs to external state and add Tabs showcase
- feat(basic-components,example)!: rework switch to external state and add Switch showcase
- refactor(scrollbar): simplify width and height handling in render functions
- feat(surface): add setter for width and height in SurfaceArgs
- feat(spacer): add setter for width and height in SpacerArgs
- feat(basic-components, example)!: make width/height non-optional DimensionValue and update examples
- feat(basic-components, example)!: rework Glass Switch to external state; add showcase and navigation
- refactor(basic-components)!: internalize text edit core; tidy rustdoc
- chore(deps): bump lru from 0.16.0 to 0.16.1
- feat(renderer, basic-components, example)!: propagate clip-aware drawing and clamp Fluid Glass sampling
- feat(image, example)!: add Fluid Glass demo and switch Image data to Arc<ImageData> ([#49](https://github.com/tessera-ui/tessera/issues/49))
- feat(dialog): enhance shadow properties for dialog content
- fix(docs): align checkbox and checkmark doc tests with API changes
- feat(dialog, example)!: integrate Dialog provider and add demo (#49)
- refactor(checkbox)!: require explicit state; remove legacy args; add example (#49)
- feat(example): complete column/row/boxed showcase planned in #49
- fix(test): update examples to new Router API and clean up docs
- refactor(example): rework(ing) demos to better showcase components and updated APIs
- perf(text): add LRU cache to reuse TextData and reduce buffer rebuild cost
- fix(scrollable, surface): enhance constraint merging and child measurement logic
- feat(bottom_nav_bar): prevent unnecessary state updates on navigation item click
- feat(text): add None option for aligment in TextData
- build(deps): update glyphon-tessera-fork to version 0.9.4
- feat(button, bottom_nav_bar): integrate shadow properties for enhanced visual feedback
- feat(button): add shadow property to ButtonArgs with strip option
- refactor(surface): update shadow property to strip option in SurfaceArgs

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.12.0...tessera-ui-basic-components-v1.13.0)

## [v1.12.0] - 2025-09-07 +08:00

### Changes

- docs(basic-components): refine rustdoc, unify intra-doc links, add module docs, tidy examples
- feat(components): add SideBar component
- fix(padding-utils): correct min constraint adjustment for wrap dimensions and relocate tessera attribute
- style(bottom-nav-bar): format imports and docs(router) comment wrapping

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.11.0...tessera-ui-basic-components-v1.12.0)

## [v1.11.0] - 2025-09-06 +08:00

### Changes

- feat(animation): add easing-based selection transition to bottom nav bar
- feat(bottom-nav-bar): implement bottom navigation bar component and simplify routing API

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.2...tessera-ui-basic-components-v1.11.0)

## [v1.10.2] - 2025-09-05 +08:00

### Changes


[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.1...tessera-ui-basic-components-v1.10.2)

## [v1.10.1] - 2025-09-04 +08:00

### Changes

- fix(fluid_glass): relax FluidGlassArgs equality by ignoring on_click callback pointer

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.0...tessera-ui-basic-components-v1.10.1)

## [v1.10.0] - 2025-09-04 +08:00

### Changes

- perf(renderer): add dirty frame detection and dynamic command equality
- style(text_editor): reorder and deduplicate imports
- Fix Imports
- Update Doc Comments
- Update Text Editor & Core
- feat(tessera-ui-basic-components): add Tabs component and example
- perf(tessera-ui-basic-components): batch child measurements to use measure_children
- docs(tessera-ui-basic-components): update examples to scoped child API
- feat(tessera-ui-basic-components): introduce scoped child API and update examples
- refactor(logging): replace log/flexi_logger/android_logger with tracing
- chore(deps): bump encase from 0.11.1 to 0.11.2

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.9.1...tessera-ui-basic-components-v1.10.0)

## [v1.9.1] - 2025-08-26 +08:00

### Changes

- refactor(ui): simplify conditional checks in keyboard event handlers and state updates
- refactor(image, constraint): replace to_max_px with get_max for constraint handling
- fix(bottom-sheet): simplify child measurement logic and remove unnecessary constraints
- chore: add new line in changelog sections for better readability
- fix(text-editor/renderer): improve text selection rendering and clipping system

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.9.0...tessera-ui-basic-components-v1.9.1)

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
