# Material 3 Expressive alignment

This checklist tracks the [Material component catalog](https://m3.material.io/components).
It does not establish complete visual equivalence. Existing modules still require
review of Expressive variants, interaction states, and responsive behavior.

## Catalog coverage

| Official component | Tessera implementation | Remaining verification or implementation |
| --- | --- | --- |
| Button groups | `button_groups` | mixed action/icon items and color variants, RTL, vertical groups, overflow, and normalized reference state snapshots remain |
| Buttons | `button` | complete normalized reference snapshots, dark themes, custom-shape states, constrained icon/label allocation, elevation snapshots, and motion-curve comparisons |
| Extended FABs | `extended_floating_action_button` (new) | complete color/hover/focus/disabled reference comparisons |
| FAB menu | `floating_action_button_menu` (new) | complete reference state snapshots, larger trigger anchoring, icon morph, and web baseline-menu presentation |
| Floating action buttons | `floating_action_button` | verify hover/focus/pressed snapshots and remaining color variants |
| Icon buttons | `icon_button` | complete normalized reference state snapshots, dark themes, custom-shape states, and motion comparisons |
| Segmented buttons | `segmented_buttons` | icon/label motion, custom icons, full state snapshots, and RTL remain |
| Split buttons | `split_buttons` | complete normalized state snapshots, dark themes, and motion-curve comparisons |
| Date pickers | `date_picker` | implement docked variants and complete modal integration, year-grid/grid-semantics audits, resize scroll-anchor retention, and normalized reference states |
| Time pickers | `time_picker` | complete modal integration, automatic layout choice, outer-list resize anchoring, normalized state, dark-theme, and motion comparisons; browser screen-reader publication remains open |
| Loading indicator | `loading_indicator` (new) | complete reference-theme screenshots and custom-size/container comparisons |
| Progress indicators | `progress`, `wavy_progress` (new) | complete normalized state, custom geometry, and straight-motion audits |
| Navigation bar | `navigation_bar` | complete state/responsive screenshots |
| Navigation drawer | `navigation_drawer` (new) | complete modal overlay integration, transitions, and full state screenshots |
| Navigation rail | `navigation_rail` | Audit collapsed/expanded/modal modes and current tokens |
| Bottom sheets | `bottom_sheet` | standard/modal geometry, partial detents, stacked-modal/platform semantics and reference states remain |
| Side sheets | `side_sheet` | direct container dragging, detached geometry, stacked-modal/platform semantics and reference states remain |
| App bars | `app_bar` | search composition, custom title content, scaffold-wide reading order, normalized reference states, dark themes, and motion comparisons remain |
| Badges | `badge` | full normalized reference images, maximum-count width, anchor baselines, RTL, and bounded navigation placement remain |
| Cards | `card` | full normalized reference states, fractional edge fidelity, remaining minimum-target/custom-shape cases, and motion comparisons |
| Carousel | `carousel` (new) | complete reference state comparisons and motion audits |
| Checkbox | `checkbox` | normalized cross-platform reference comparisons remain |
| Chips | `chip` | icon motion, RTL, tonal variants, and normalized reference images remain |
| Dialogs | `dialog` | basic/full-screen variants, action geometry, stacked-modal/platform semantics and reference states remain |
| Divider | `divider` | complete normalized reference comparisons remain |
| Lists | `list_item` | selection indicators/group navigation, nested controls/targets, expansion/swipe/drag behavior, and normalized reference states remain |
| Menus | `menus` | normalized group/state snapshots, compact adaptive layout, and remaining focus audits |
| Radio button | `radio_button` | horizontal/RTL visual groups, toolbar integration, normalized dark/state snapshots, and motion comparisons remain |
| Search | `search` | audit expanded/full-screen layouts and state screenshots |
| Sliders | `slider` | complete nested-viewport occlusion, fractional-anchor precision, range-label collision, focus-ring, and normalized reference state comparisons |
| Snackbar | `snackbar` | complete queue/host lifecycle, normalized geometry/states, motion, platform accessibility timeout preferences, and browser semantics remain |
| Switch | `switch` | normalized whole-figure pixel comparisons remain; normalized reference geometry, intrinsic empty-slot sizing, ripple-motion audits, and cross-platform motion comparisons remain |
| Tabs | `tabs` | complete final-build responsive browser checks, inline icons/badges, normalized state snapshots, primary indicator inset, dark themes, RTL, motion, and nested scroll-edge behavior |
| Text fields | `text_field` | complete normalized diagram comparisons, delayed-write/lifecycle/cross-browser audits and error, supporting text, counters, icons, multiline comparisons remain |
| Toolbars | `toolbar` (new) | complete transitions, FAB combination, scroll behavior, and full state screenshots |
| Tooltips | `tooltip` (new) | complete reference state snapshots, short-viewport/lifecycle/host coordination, caret and motion audits |
