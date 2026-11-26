## [v4.1.0] - 2026-11-26

- Fix selection highlighting that painted paragraphs outside the selected range.
- Add opt-in `show_scrollbar` to MD3 `text_input` and `text_field`. The scrollbar appears only when text overflows: horizontally for single-line input and vertically for multiline input.
- Let bounded multiline fields grow and shrink with their contents, scroll across paragraph boundaries, and place the caret correctly when clicking after scrolling.

[v4.1.0]: https://github.com/tessera-ui/tessera/compare/tessera-components-v4.0.0...tessera-components-v4.1.0

## [v4.0.0] - 2026-11-24

- **Breaking:** MD3 overlays use core popup layers. Replace overlay providers with `menu_anchor`, `tooltip_box`, `dialog`, `bottom_sheet`, `modal_side_sheet`, or `standard_side_sheet`. Dialog bodies no longer take `DialogScope`; modal navigation rails take visibility at entry, slider indicators need no host, and FAB menus no longer wrap page content.
- Text-field context menus draw and receive input above siblings. Text fields forward modifiers to their row and wrap to their preferred width.

[v4.0.0]: https://github.com/tessera-ui/tessera/compare/tessera-components-v3.0.0...tessera-components-v4.0.0

## [v3.0.0] - 2026-11-16

### Breaking changes

- **Package and module paths changed.** Depend on `tessera-components` and
  import Material components from `tessera_components::md3`; the crate root
  contains design-system-independent primitives.
- **Components use named props and builder handles.** Required values are
  passed at entry; optional values use named setters. Controls take their value
  at entry and report changes through callbacks.
- **The Material theme is explicit.** Provide `md3::theme::MaterialTheme` with
  `material_theme()`; visual defaults no longer come from a global color scheme.
- **Modifiers have separate owners.** Generic layout, semantics, and interaction
  modifiers come from `tessera-foundation`; Material scrolling and elevation
  remain in `md3`.
- **Components were renamed or removed.** Text editing uses `md3::text_input` or
  `md3::text_field`; navigation and glass components use their new `md3` or
  experimental HIG paths.
- **Fonts and images load through the supported resource APIs.** Text uses
  platform fonts by default; images load asynchronously from generated assets
  or in-memory bytes.

### Added

- Material Design 3 components for navigation, forms, overlays, progress,
  selection, and text entry.
- Flow layouts, pagers, lazy grids, and keyed lazy-list items.
- Custom typography, localized component labels, keyboard focus, and
  accessibility semantics.
- Experimental liquid-glass materials under `experiment::hig`.

### Fixed

- Text-field context menus respond consistently to right clicks and long
  presses, dismiss when focus leaves, and support fixed-height multiline fields.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.7.0...tessera-components-v3.0.0)

## [v2.7.0] - 2025-11-18 +08:00

### Changes

- Lazy lists now handle viewport resizing correctly.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.6.0...tessera-ui-basic-components-v2.7.0)

## [v2.6.0] - 2025-11-17 +08:00

### Changes

- Added lazy lists.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.5.1...tessera-ui-basic-components-v2.6.0)

## [v2.5.1] - 2025-11-15 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.5.0...tessera-ui-basic-components-v2.5.1)

## [v2.5.0] - 2025-11-15 +08:00

### Changes

- Added accessibility support for component controls.
- Vector images and refined fluid-glass effects are available.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.4.0...tessera-ui-basic-components-v2.5.0)

## [v2.4.0] - 2025-11-09 +08:00

### Changes

- Added icon and vector-image components.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.3.0...tessera-ui-basic-components-v2.4.0)

## [v2.3.0] - 2025-11-08 +08:00

### Changes

- Fluid glass supports multiply and screen blend modes.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.2.0...tessera-ui-basic-components-v2.3.0)

## [v2.2.0] - 2025-11-03 +08:00

### Changes

- Added independent child alignment and configurable dialog padding.
- Improved touch responsiveness and fluid-glass rendering.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.1.0...tessera-ui-basic-components-v2.2.0)

## [v2.1.0] - 2025-09-22 +08:00

### Changes

- Draw and compute pipelines can be registered directly with the renderer.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v2.0.0...tessera-ui-basic-components-v2.1.0)

## [v2.0.0] - 2025-09-17 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.15.0...tessera-ui-basic-components-v2.0.0)

## [v1.15.0] - 2025-09-17 +08:00

### Changes

- Rounded rectangle corner radii use `Dp`; the side bar supports blur.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.14.0...tessera-ui-basic-components-v1.15.0)

## [v1.14.0] - 2025-09-15 +08:00

### Changes

- Text editors report content changes through a callback.
- Checkboxes have a public constructor for their state.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.13.1...tessera-ui-basic-components-v1.14.0)

## [v1.13.1] - 2025-09-13 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.13.0...tessera-ui-basic-components-v1.13.1)

## [v1.13.0] - 2025-09-12 +08:00

### Changes

- Tabs and switches take their state from the application.
- Clipping applies to custom drawing and fluid-glass effects.
- Dialogs support configurable visual feedback and explicit control state.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.12.0...tessera-ui-basic-components-v1.13.0)

## [v1.12.0] - 2025-09-07 +08:00

### Changes

- Added the side bar component.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.11.0...tessera-ui-basic-components-v1.12.0)

## [v1.11.0] - 2025-09-06 +08:00

### Changes

- Added an animated bottom navigation component and simplified routing.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.2...tessera-ui-basic-components-v1.11.0)

## [v1.10.2] - 2025-09-05 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.1...tessera-ui-basic-components-v1.10.2)

## [v1.10.1] - 2025-09-04 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.10.0...tessera-ui-basic-components-v1.10.1)

## [v1.10.0] - 2025-09-04 +08:00

### Changes

- Added the tabs component.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.9.1...tessera-ui-basic-components-v1.10.0)

## [v1.9.1] - 2025-08-26 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.9.0...tessera-ui-basic-components-v1.9.1)

## [v1.9.0] - 2025-08-25 +08:00

### Changes

- Text editors scroll smoothly by pixel position.
- Bottom sheets support glass styling; fluid-glass shapes support independent corners.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.8.0...tessera-ui-basic-components-v1.9.0)

## [v1.8.0] - 2025-08-15 +08:00

### Changes

- Added bottom sheets, independent corner radii, and component clipping.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.7.1...tessera-ui-basic-components-v1.8.0)

## [v1.7.1] - 2025-08-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.7.0...tessera-ui-basic-components-v1.7.1)

## [v1.7.0] - 2025-08-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.6.1...tessera-ui-basic-components-v1.7.0)

## [v1.6.1] - 2025-08-03 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.6.0...tessera-ui-basic-components-v1.6.1)

## [v1.6.0] - 2025-08-03 +08:00

### Changes

- Added a glass progress component and configurable glass borders.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.5.0...tessera-ui-basic-components-v1.6.0)

## [v1.5.0] - 2025-08-02 +08:00

### Changes

- Refined progress indicators and glass component borders.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.4.0...tessera-ui-basic-components-v1.5.0)

## [v1.4.0] - 2025-07-31 +08:00

### Changes

- Added modal glass dialogs and easing-based transitions.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.3.1...tessera-ui-basic-components-v1.4.0)

## [v1.3.1] - 2025-07-31 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.3.0...tessera-ui-basic-components-v1.3.1)

## [v1.3.0] - 2025-07-31 +08:00

### Changes

- Dialogs and surfaces can block input; dialog visibility supports animation.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.2.1...tessera-ui-basic-components-v1.3.0)

## [v1.2.1] - 2025-07-29 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.2.0...tessera-ui-basic-components-v1.2.1)

## [v1.2.0] - 2025-07-29 +08:00

### Changes

- Scrollbars support overlay and alongside layouts, and can be hidden or auto-hidden.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v1.1.3...tessera-ui-basic-components-v1.2.0)

## [v1.1.3] - 2025-07-28 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.2...tessera-ui-basic-components-v1.1.3)

## [v1.1.2] - 2025-07-28 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.1...tessera-ui-basic-components-v1.1.2)

## [v1.1.1] - 2025-07-25 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.1.0...tessera-ui-basic-components-v1.1.1)

## [v1.1.0] - 2025-07-25 +08:00

### Changes

- Added reusable scrollbars.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.2...tessera-ui-basic-components-v1.1.0)

## [v1.0.2] - 2025-07-24 +08:00

### Changes

- Added a clipboard API for components.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.1...tessera-ui-basic-components-v1.0.2)

## [v1.0.1] - 2025-07-24 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-basic-components-v1.0.0...tessera-ui-basic-components-v1.0.1)

## [v1.0.0] - 2025-07-21 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.4.1...tessera-ui-basic-components-v1.0.0)

## [v0.4.1] - 2025-07-21 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.4.0...tessera-ui-basic-components-v0.4.1)

## [v0.4.0] - 2025-07-21 +08:00

### Changes

- Text editing supports clipboard shortcuts and visible-area selection.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.3.0...tessera-ui-basic-components-v0.4.0)

## [v0.3.0] - 2025-07-20 +08:00

### Changes

- Redesigned sliders with a disabled state and updated glass effects.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.2.0...tessera-ui-basic-components-v0.3.0)

## [v0.2.0] - 2025-07-19 +08:00

### Changes

- Glass switches support borders and shapes support configurable corner continuity.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.1.0...tessera-ui-basic-components-v0.2.0)

## [v0.1.0] - 2025-07-19 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-basic-components-v0.2.0...tessera-ui-basic-components-v0.1.0)
