## [v1.5.0] - 2025-07-25 +08:00

### Changes
- feat(tessera-ui): add Color::lerp and refine Px methods
- feat(px): correct Px::abs behavior and add positive()/negative()
- feat(px): add mul_f32 and div_f32 methods
- feat(cursor): impl PartialEq for cursor event types
- refactor(component_tree): remove unused node_id from StateHandlerInput
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

