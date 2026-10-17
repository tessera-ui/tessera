# Tessera basic components

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-components/docs/README_CN.md

`tessera-components` provides a set of basic UI components for building common user interfaces. This includes buttons, text, layout containers, and more.

## Material defaults

`button().size(ButtonSize::Medium)` selects an Expressive size preset. Five
presets provide 32/40/56/96/136dp container heights, matching typography,
content insets, icon spacing, and outline thickness. `leading_icon(...)` and
`trailing_icon(...)` size their icons automatically. Use `content_padding(...)`
for per-edge insets or `padding(...)` for a uniform override. Explicit modifiers
continue to bound the container; disabling a clickable button preserves its
minimum touch target and geometry. The Basic Components page demonstrates the
five sizes and visual variants.

Use `.round()` or `.square()` for size-aware resting corners and pressed shape
motion. `.selected(value)` creates a controlled toggle button; update its state
in `on_click`. Filled, elevated, tonal, and outlined variants support toggling;
text buttons remain ordinary actions. Selection reverses the resting shape and
uses the corresponding theme colors. `.shape(...)` fixes a custom shape instead.
Shared `interaction_state(...)` supports previews of hover, focus, and press.

Constrained buttons reserve space for both icons before measuring the grouped
label slot. Labels wrap within the remaining width and can increase the visual
height above the preset minimum. Wrap multiple label elements in a row or column
when they need a layout other than stacking.

`icon_button()` now defaults to the Expressive filled, small, round, uniform
configuration. Use `.standard()` for a transparent action. `IconButtonSize`
provides five heights (32/40/56/96/136dp) and 20/24/24/32/40dp icons;
`IconButtonWidth::{Narrow, Uniform, Wide}` controls horizontal proportions.
Interactive targets remain at least 48dp when constraints permit, including
when disabled. `.selected(value)` creates a controlled toggle for all four
variants; update the value in `on_click`. Round and square presets exchange
resting shapes on selection and share a pressed shape without spring bounce.
Explicit `.shape(...)` suppresses automatic morphing. Standard and outlined
variants use recommended vibrant colors by default; `color_style(IconButtonColorStyle::Inherited)`
uses the surrounding content color for unselected icons and outlines.

`split_button_layout` keeps the trailing action visible when width is limited.
`SplitButtonSize` supplies five minimum heights, matching label typography,
leading/trailing icon sizes, and outline thickness. Content slots arrange their
children in a row. Use `.expanded(value)` on `split_trailing_button` with a menu
controller; this publishes disclosure and popup semantics, rounds the selected
trailing shape, and animates its menu icon. Resting menu icons use the documented
optical offsets and become geometrically centered when expanded. Explicit icon
rotation overrides inherited control motion. Shared `interaction_state(...)`
previews hover, focus, and press; `.shape(...)` fixes a custom outline. Visual
variant setters preserve an explicitly disabled state. The Material Components
page demonstrates all five sizes with working menus and four styles.

`menu_provider` uses expressive vertical-menu shapes and standard surface colors. Set `.grouped(true)` and compose `menu_group`
children with `MenuPosition` for separately surfaced sections and 2dp gaps.
`MenuColorStyle::Vibrant` supplies tertiary-based emphasis and is inherited by
groups, items, and nested menus. Item labels use the remaining width after icons
and shortcuts; optional `selected` values publish menu-radio semantics. Long
menus scroll within their height limit. `menu_item().submenu_content(...)`
anchors cascading menus beside their parent item on a shared overlay layer.
Leaf actions close the complete chain; the opposite arrow or Escape returns one
level, and Tab
leaves the complete chain before traversing the page. Disabled or removed parent
items close their descendant menus without leaving stale focus requests. Menu
shadows extend outside the panel while content is clipped to its rounded shape.
Scrolling grouped menus clip overflowing content along the scroll axis and retain
lateral shadows.

Pass `.anchor(current_anchor)` to follow layout changes while a menu is open;
coordinates are relative to the provider. Without this prop, `open_at` retains a
fixed anchor snapshot. Placement tries the opposite side, including the requested
gap, before clamping to the available viewport. The example tracks its normal
and split-button triggers and closes their menus when the active source leaves
the viewport or is removed.

See the [Material alignment checklist](docs/MATERIAL_ALIGNMENT.md) for component
coverage and remaining verification. `floating_toolbar` provides horizontal and
vertical action groups with standard/vibrant colors and optional expanded
actions. `docked_toolbar` provides a full-width 64dp action surface with adaptive
4–32dp action spacing. `navigation_drawer` and `navigation_drawer_item` provide a navigation
surface, selected destinations, leading icons, and trailing badges. Both are
demonstrated on the example Material Components page.

`floating_action_button` includes an 80dp `Medium` size with a recommended 28dp
icon. `extended_floating_action_button("Action")` provides 56/80/96dp Expressive
sizes, automatic icon sizing, size-specific typography, and animated label
expansion. Labels remain available to accessibility when collapsed. A button
without an icon stays expanded. `MaterialShapes::large_increased` supplies the
20dp medium FAB corners; custom shape literals should include
`..Default::default()` to inherit additional shape roles.

`floating_action_button_menu` hosts persistent page content and a bottom-aligned
creation menu. Supply `FloatingActionButtonMenuItem` actions and optionally share
`FloatingActionButtonMenuState`. Actions expand in a staggered sequence; outside
clicks, Escape, or an action close the menu. Primary, secondary, and tertiary
colors are available. The list scrolls in short viewports while the toggle stays
fixed. `SemanticsArgs::expanded` and `has_popup` expose disclosure controls;
custom semantics literals should include `..Default::default()`.

`loading_indicator` provides the Expressive seven-shape animation and a
determinate circle-to-burst morph through `.progress(value)`. Use `.contained(true)`
for a themed background, and `.paused(true)` to pause infinite animation and stop
frame requests. Its default size is 48dp. Shape geometry is shared, and animation
updates reuse GPU instance buffers. Shapes, colors, size, and progress semantics
work on native and WebAssembly targets.

`slider`, `centered_slider`, and both `range_slider` handles support arrow keys,
Home/End, and PageUp/PageDown. Continuous keyboard adjustments use 1% steps;
discrete adjustments follow the configured ticks. Range handles keep a fixed
Tab order and cannot cross. Use `.start_accessibility_label(...)` and
`.end_accessibility_label(...)` for distinct localized handle names; the group
retains `.accessibility_label(...)`.

`tooltip_provider("Help text")` hosts a plain tooltip above page content; put
its trigger inside `tooltip_anchor().child(...)`. Share `TooltipController`
explicitly when nesting providers. Hover waits 500ms, keyboard focus shows the
description, and a 500ms touch hold prevents trigger activation on release. Escape and
outside presses dismiss without blocking unrelated controls. Use
`TooltipVariant::Rich`, `.headline(...)`, and `.action_label(...).on_action(...)`
for a rich explanation. `.actions(...)` replaces that action with custom content,
including two text buttons. Custom actions can dismiss the shared controller.
Place the provider outside scrolling/clipping containers so the popup can avoid
window edges. Pending timers request frames; a stable visible tooltip does not
run an animation clock.

`Modifier::clip_shape(Shape::rounded_rectangle(Dp(16.0)))` clips a node and its
content without changing their measured size. Ellipses and per-corner shapes are
also supported. Nested masks apply to text, raster/vector images, custom draw
pipelines, and pointer hit testing. Use multisampling for smooth mask edges.
Custom UI pipelines set `depth_stencil: tessera_ui::ui_stencil_state()` and leave
the stencil reference to the renderer. Independent atlas-generation passes keep
their own attachment configuration.

`carousel(item_count)` provides multi-browse, uncontained, mixed-aspect
uncontained, hero, centered hero, and full-screen layouts. Its `.content(...)`
slot receives each item's index and the reactive `full_width()` and
`masked_width()` getters. Read these getters where content adapts to its current
mask, including inside nested content slots. Content retains its full size while
the 28dp rounded mask changes; only visible items are built. Use
`.item_aspect_ratios(...)` for portrait, landscape, and square media in the
mixed-aspect layout, with 8dp spacing and 16dp leading padding by default. Share
`CarouselController` for programmatic navigation. Horizontal dragging and wheel
input, arrow keys, Home/End, and accessibility scroll actions are supported.
Horizontal layouts leave vertical wheel input to the surrounding page.
Uncontained layouts scroll freely by default; other layouts snap. Set spacing
and end padding explicitly for your composition. Full-screen pages fill a
portrait viewport with rectangular masks and 16dp vertical spacing. They use
vertical dragging, wheel input, Up/Down keys, and vertical accessibility scroll
actions; free scrolling is replaced by snapping. The example uses 8dp spacing
for other layouts and 4× multisampling. Keep external controllers and selection
state outside virtualized items when that state must survive scrolling
offscreen.

`text`, `text_input`, and `text_field` accept typography through `.style(...)`.
Text uses bundled Roboto Regular, Medium, and Bold fonts on desktop, Android, and
WebAssembly. `MaterialTypography` supplies font weight and letter spacing as well
as size and line height. Use `theme.typography.emphasized()` to obtain the
emphasized type scale. Custom `TextStyle` literals should inherit a theme role or
include `..Default::default()` to initialize the additional font properties.
Display text wraps between words and falls back to character wrapping when a
word is wider than the available line.

The navigation bar uses the 64dp flexible layout. Its default icon position changes
from above the label to before the label when the bar reaches 600dp. Set
`NavigationBarIconPosition::Top` or `Start` to choose a fixed configuration.

The date picker has a 360dp calendar, 48dp date targets at its standard width,
a scrollable year grid, and an editable `MM/DD/YYYY` input mode. It preserves
seven date columns and three year columns when constrained to a narrower width.
When embedding `date_picker_dialog` in
`dialog_provider`, set the provider's padding to `Dp(0.0)` because the picker
includes its own header and action padding. A calendar dialog with both weekday
labels and actions occupies 360×568dp.

The time picker supports a 256dp clock dial, hour and minute selection, the inner
24-hour ring, AM/PM selection, and keyboard input. `time_picker_dialog` supplies
the title, mode switch, and action layout; use the dialog provider's standard
24dp padding around it.

For visual comparisons, use the same color scheme, font, density, locale, and
component state. Typography and layout tokens alone do not establish pixel
equivalence across different renderers or reference themes.

Reference specifications: [typography](https://m3.material.io/styles/typography/type-scale-tokens),
[date pickers](https://m3.material.io/components/date-pickers/specs),
[time pickers](https://m3.material.io/components/time-pickers/specs), and
[navigation bars](https://m3.material.io/components/navigation-bar/specs).
