# Material 3 Expressive alignment

This checklist tracks the complete [Material component catalog](https://m3.material.io/components),
inspected on 2026-10-17. It is an implementation and verification ledger, not a
claim of complete visual equivalence. Existing modules still require a review of
their current Expressive variants, interaction states, and responsive behavior.

## Catalog coverage

| Official component | Tessera implementation | Remaining verification or implementation |
| --- | --- | --- |
| Button groups | `button_groups` | Five-size child typography/icons, round/square presets, standard press width exchange with adaptive content padding, connected equal widths and 2dp gaps, external selection/clear, live selection modes, required selection, disabled items/groups, checkbox roles, horizontal focus traversal, item cleanup, and size-specific pressed/selected shape motion implemented; native 1x/4x GPU scenarios cover geometry, pixels, input, resize and style/focus retention; browser required selection, disabled input and keyboard traversal verified; mixed action/icon items and color variants, RTL, vertical groups, overflow, and normalized reference state snapshots remain |
| Buttons | `button` | Five sizes, round/square presets, controlled toggles, pressed shape motion, and elevation transitions implemented; native 1×/4× GPU geometry, typography, selection, semantics, and disabled input verified; WASM variants, pointer/Enter/Space, narrow wrapping, and resize/screenshot regression checked; complete normalized reference snapshots, dark themes, custom-shape states, constrained icon/label allocation, elevation snapshots, and motion-curve comparisons |
| Extended FABs | `extended_floating_action_button` (new) | Three sizes, icon sizing, label-only mode, disabled input, dynamic props, and reversible expansion tested; native GPU cycle restores identical pixels; WASM pointer/keyboard activation and three-size label restoration verified; complete color/hover/focus/disabled reference comparisons |
| FAB menu | `floating_action_button_menu` (new) | Native geometry, short viewports, input blocking, state retention, reversible stagger, GPU pixel recovery, and published AccessKit tree tested; WASM actions, arrow keys, reverse Tab, Escape, focus restoration, and three color sets verified; complete reference state snapshots, larger trigger anchoring, icon morph, and web baseline-menu presentation |
| Floating action buttons | `floating_action_button` | 80dp medium size with 28dp icon and 20dp corners added; elevation transitions implemented; verify hover/focus/pressed snapshots and remaining color variants |
| Icon buttons | `icon_button` | Five sizes, three widths, round/square presets, controlled toggles, vibrant/inherited colors, and 48dp targets implemented; native 1×/4× GPU geometry, fractional density, dynamic colors, input, selection semantics, and resize recovery verified; WASM four styles, three widths, pointer/keyboard activation, long press, disabled input, narrow wrapping, and lazy-list state retention verified; complete normalized reference state snapshots, dark themes, custom-shape states, and motion comparisons |
| Segmented buttons | `segmented_buttons` | Standard M3 control retained; Expressive recommends connected button groups. Native 1x/4x checks at four densities cover roles, group metadata, selection, default checkmarks, focus, disabled input, live typography, stable natural widths, exact bounded widths, and zero-width recovery; WASM pointer/arrow/Tab/Space/Enter selection and stable group widths verified; icon/label motion, custom icons, full state snapshots, and RTL remain |
| Split buttons | `split_buttons` | Five-size typography and icon contexts, sized outlines, trailing width priority, optical offsets, pressed/hover/focus shapes, controlled menu expansion, and arrow/elevation motion implemented; native 1×/4× GPU and fractional-density geometry, dynamic props, disabled semantics, and width-priority checks passed; full example menus, resizing, and lazy-list retention exercised on native and WASM; mixed-size/wrapped-label visual-height synchronization covered by 1×/4× native pixel readback at 1.5 density and WASM 390px/877px resize recovery; complete normalized state snapshots, dark themes, and motion-curve comparisons |
| Date pickers | `date_picker` | Single-date layout/input/year grid tested; native diagram callouts remain stable under integer/fractional viewport translation and update for real view changes; range state, virtualized months, inclusive endpoints, restricted dates, and joint manual input implemented; native 1×/4× GPU checks cover range bands, semantics, selection, restrictions, month requests, resize recovery, local focus restoration, unavailable-date focus pixels, and one-stop Tab exit; day/week/month/year navigation, week boundaries, short-month/leap-day clamping, rapid navigation/selection, and forward/reverse Tab checked in WASM at normal and narrow widths; implement docked variants and complete modal integration, year-grid/grid-semantics audits, resize scroll-anchor retention, and normalized reference states |
| Time pickers | `time_picker` | Native 1x/4x GPU checks cover individual clock button roles, labels, selected state, focus and activation, AM/PM radio semantics, roving traversal, exact values, pointer snapping, dynamic steps, 12/24-hour recovery, and manual entry; horizontal layout, narrow fallback, stable focus across resize, and 48dp period hit targets implemented; WASM horizontal 12/24-hour layouts, narrow fallback and resize recovery, arrow focus, Enter/Space activation, radio navigation, Tab exit, mode changes, input and confirmation checked; complete modal integration, automatic layout choice, outer-list resize anchoring, normalized state, dark-theme, and motion comparisons; browser screen-reader publication remains open |
| Loading indicator | `loading_indicator` (new) | Official cubic morph data and spring timing verified; 48 GPU comparisons across eight sequences and 1×/4× sampling meet a 2.5% normalized coverage-error limit; animation/paused frames reuse buffers; native progress semantics and WASM pause/resume plus 0/50/100% progress verified; complete reference-theme screenshots and custom-size/container comparisons |
| Progress indicators | `progress`, `wavy_progress` (new) | Straight and wavy linear/circular determinate/indeterminate variants present; wavy variants include independent stroke styles, adaptive gaps, reverse flow, frame-clock motion, and pause/resume; circle/star morph geometry checked against the independent Graphics Shapes library; complete normalized state, custom geometry, and straight-motion audits |
| Navigation bar | `navigation_bar` | Flexible top/start arrangement tested; complete state/responsive screenshots |
| Navigation drawer | `navigation_drawer` (new) | 336×56dp destinations, selection and disabled input tested; destination switching verified in WASM; complete modal overlay integration, transitions, and full state screenshots |
| Navigation rail | `navigation_rail` | Audit collapsed/expanded/modal modes and current tokens |
| Bottom sheets | `bottom_sheet` | Modal opening, exit focus restoration, forward/reverse traversal, empty-content fallback, added controls, background pointer isolation, unavailable origins and programmatic motion reversal covered at two densities and 1x/4x MSAA; named Dialog roles, modal flags and background semantic isolation verified; handle release/cancellation, velocity thresholds, nested drag settling and reversal verified; standard/modal geometry, partial detents, stacked-modal/platform semantics and reference states remain |
| Side sheets | `side_sheet` | Modal focus restoration, traversal, empty-content fallback and scrim isolation covered at two densities and 1x/4x MSAA; named Dialog roles, modal flags and background semantic isolation verified; standard traversal/background interaction, both-edge motion reversal, 400dp width cap, 24dp padding and narrow resize covered; nested drag settling, flings, resized thresholds and live edge changes verified; direct container dragging, detached geometry, stacked-modal/platform semantics and reference states remain |
| App bars | `app_bar` | Small, medium flexible, and large flexible layouts, optional subtitles, leading/global-center alignment, action avoidance, dynamic slot colors, and multiline growth implemented; native 1×/4× GPU geometry, typography, reading order, colors, input, wrapping, and resize recovery verified; nested-scroll collapse, snap, inertia, color changes, idle callback cleanup, and hidden-navigation semantics and keyboard traversal verified; WASM variants, subtitle switching, centered titles, pointer/Tab/Enter actions, and 390px sizing checked; search composition, custom title content, scaffold-wide reading order, normalized reference states, dark themes, and motion comparisons remain |
| Badges | `badge` | Dot/count sizes, offsets, rounded fractional-density geometry, anchor constraints, empty/multiple slot children, live theme/text contexts, explicit colors, semantic authoring, and pointer passthrough verified at four densities and 1x/4x GPU sampling; WASM count/dot/theme transitions checked; full normalized reference images, maximum-count width, anchor baselines, RTL, and bounded navigation placement remain |
| Cards | `card` | Slow-frame elevation, disabled/static semantics, accessible labels, foreground borders/feedback over clipped media, outlined hover/focus tokens, dark container/content roles, theme restoration, and resize recovery verified by native 1x/4x GPU regressions; WASM media, keyboard focus/activation, disabled/static traversal, and re-enabling checked; full normalized reference states, fractional edge fidelity, remaining minimum-target/custom-shape cases, and motion comparisons |
| Carousel | `carousel` (new) | Multi-browse, uncontained, mixed-aspect uncontained, hero, centered hero, and full-screen layouts implemented with virtualized full-size content, 28dp masks, snapping/free scrolling, controller navigation, reverse layout, keyboard and accessibility actions; Native checks include source geometry/padding vectors, 1.25×/1.5×/2× density, dynamic props, pointer cancellation, zero-width/resize recovery, a responsive lazy feed, and 1×/4× GPU readback; WASM layout switching, horizontal/vertical input, Home/End, activation, narrow-window recovery, idle content, and scrolling out of the lazy viewport are verified; complete reference state comparisons and motion audits |
| Checkbox | `checkbox` | Checked, unchecked, and indeterminate states share one builder and controller; native 1x/4x checks cover density, geometry, light/dark/error/disabled colors, custom overrides, validation semantics, activation, spatial springs, Oklab effects, reversal pixels, and idle-clock shutdown; normalized cross-platform reference comparisons remain |
| Chips | `chip` | Four variants preserve icon/label edge spacing; selectable semantics, live typography, interaction elevation, focus outlines, and input icon colors verified; input avatars preserve source colors with circular center cropping, disabled opacity, priority, and restored spacing at four densities and 1x/4x sampling; Expressive selection/press shapes and interrupted spring motion verified; icon motion, RTL, tonal variants, and normalized reference images remain |
| Dialogs | `dialog` | Provider opening, exit focus restoration, forward/reverse traversal, empty-content fallback, added controls, background pointer isolation, unavailable origins and programmatic motion reversal covered at two densities and 1x/4x MSAA; named Dialog roles, modal flags and background semantic isolation verified; basic/full-screen variants, action geometry, stacked-modal/platform semantics and reference states remain |
| Divider | `divider` | Parent-constrained thickness, unbounded-axis recovery, modifier-based insets, hairlines, and live outline-variant colors implemented; analytic fractional strokes, default-palette helper colors, half-pixel placement, opacity, stencil clipping, forced-size recovery, and stable upload allocation verified at 1x/4x sampling; native checks cover four densities; release WASM and browser rendering verified; complete normalized reference comparisons remain |
| Lists | `list_item` | 56/72/88dp minimum heights, measured supporting-text wrapping, short/tall automatic alignment and explicit overrides, 20dp icon context, live slot typography/colors, selected semantics and disabled focus/input corrected; native 1x/4x tests at four densities and WASM interaction/reflow verified; segmented positions and FastSpatial state shapes verified by native input/pixel regressions; perceptual color transitions and controlled drag palettes implemented, with rendering verification recorded below; main/overline/supporting render slots implemented with verification recorded below; controlled action/single/multiple selection roles and callbacks verified; selection indicators/group navigation, nested controls/targets, expansion/swipe/drag behavior, and normalized reference states remain |
| Menus | `menus` | Current/saved anchors, reflow, gap-aware side fallback, exact-fit resize, and outside dismissal GPU-tested; trigger focus restoration, explicit fallback, disabled trigger recovery, and forward/reverse Tab exit passed native GPU regression; WASM selection, Escape, Enter/Space reopening, Tab exit, narrow-window recovery, and lazy-list retention checked; queued close/navigation barrier covered by native core/order regressions and WASM rapid keyboard sequences; Expressive group surfaces, standard/vibrant colors, position-specific corners, selected menu-radio semantics, trailing-space reservation, and scrolling implemented; shared cascading overlays, branch/chain dismissal, source removal/disable cleanup, and virtualized Tab-exit navigation implemented; shadow overflow, rounded content clipping, scrolling shadow input bounds, container morphing, and delayed mouse-hover branch switching GPU-tested; normalized group/state snapshots, compact adaptive layout, and remaining focus audits |
| Radio button | `radio_button` | Native 1x/4x GPU checks cover group names/roles/orientation, disabled checked-state semantics, 48dp targets at 1.5 density, directional selection, all four arrows for both orientations, wrapped traversal skipping disabled options, one-stop Tab entry/exit, unselected entry without activation, disabled-focus recovery without value changes, caller traversal exclusions, re-enabling identity, pointer completion/cancellation, interaction colors, custom color retention, internally managed mutual exclusion, independent groups, removal/reinsertion, and controller-driven selection; browser arrows and pointer cancellation verified; horizontal/RTL visual groups, toolbar integration, normalized dark/state snapshots, and motion comparisons remain |
| Search | `search` | Focus, typing, submit/dismiss fixed; audit expanded/full-screen layouts and state screenshots |
| Sliders | `slider` | Single, centered, and range keyboard adjustment, fixed Tab order, peer constraints, disabled input, dynamic steps, and retained callbacks verified in native tests and WASM; published AccessKit handles retain identity across value/step changes and expose current peer limits; vertical orientation and direction reversal share layout, pointer coordinates, corner mapping, and keyboard behavior across all three variants; website color roles, independent tick colors, theme updates, marker gaps, and end-stop visibility implemented; optional floating value indicators, formatted screen-reader values, focus/drag visibility, and window-edge placement implemented; complete nested-viewport occlusion, fractional-anchor precision, range-label collision, focus-ring, and normalized reference state comparisons |
| Snackbar | `snackbar` | Polite status messages, named action/close buttons, customizable dismiss labels, appearance focus preservation, dismissal restoration, controlled timeout and consecutive actions verified at two densities and 1x/4x MSAA; complete queue/host lifecycle, normalized geometry/states, motion, platform accessibility timeout preferences, and browser semantics remain |
| Switch | `switch` | Native 1x/4x checks cover semantics, targets, controlled updates, colors, input, icon sizing, spatial size/offset transitions, reversal, cancelled press/release, controller snapping, pixel recovery, and idle clocks; WASM dynamic icons, cancelled dragging, and keyboard operation verified; native accessibility focus rings and optional interaction-state previews are covered at 1x/4x MSAA, including live-input precedence and disabled suppression; the 20-control article States figure passes native independent-pointer, disabled, focus-without-activation and responsive-target checks; normalized whole-figure pixel comparisons remain; normalized reference geometry, intrinsic empty-slot sizing, ripple-motion audits, and cross-platform motion comparisons remain |
| Tabs | `tabs` | Primary/secondary heights, colors, indicator geometry, selected semantics, roving traversal, dynamic props, controlled selection, focus recovery, and narrow scrolling covered by native 1x/4x GPU tests; manual/automatic keyboard activation exercised in WASM; complete final-build responsive browser checks, inline icons/badges, normalized state snapshots, primary indicator inset, dark themes, RTL, motion, and nested scroll-edge behavior |
| Text fields | `text_field` | Shared typography and outline fixes verified; Anatomy uses one theme-aware 1024×300 native artboard with outlined callout capsules, matching the official 2048×600 reference proportions; native checks cover both themes, clear actions, narrow scaling and retained values; focused clipboard input, accepted/rejected/stale cut completion, and edit/policy pipelines tested natively; WASM full-date paste, copy readback, cut, rapid replacement, cross-field Tab routing, keyboard save, and read-only policy verified; complete normalized diagram comparisons, delayed-write/lifecycle/cross-browser audits and error, supporting text, counters, icons, multiline comparisons remain |
| Toolbars | `toolbar` (new) | Floating horizontal/vertical and collapsed layouts tested; docked 64dp height and 4–32dp spacing tested at 132/140/320/800px widths; pointer/keyboard activation and docked actions verified in WASM; complete transitions, FAB combination, scroll behavior, and full state screenshots |
| Tooltips | `tooltip` (new) | Plain/rich page-level overlays, placement fallback, delayed hover, focus, touch hold, Escape/outside dismissal, and custom actions implemented; native GPU tests verify focus, descriptions, content/variant updates, pixel restoration, release cancellation, later clicks, and scroll cancellation; WASM rendering, word wrapping, pointer actions, repeated forward/reverse Tab, action activation, dismissal, and trigger re-entry verified; complete reference state snapshots, short-viewport/lifecycle/host coordination, caret and motion audits |

## Slider axes and retained semantics

`slider`, `centered_slider`, and `range_slider` accept `orientation` and
`reverse_direction`. Horizontal is the default; vertical values increase from top
to bottom. Reversal changes placement and pointer mapping, not selected values.
Up/Down and PageUp/PageDown follow screen direction; Home/End select numeric
endpoints. Range handles retain their logical Tab order and peer limits.

## Slider value indicators

`SliderValueIndicatorVisibility` supports Hidden (the default), OnInteraction,
and Always. Place `slider_value_indicator_provider` above scrolling and clipped
content. Nested providers share the outer provider's labels.

OnInteraction shows labels for focused or dragged handles, not hover alone.
Disabled handles hide labels. `value_label`, `start_value_label`, and
`end_value_label` set both visual and screen-reader values without invoking
`on_change`. Default labels are percentages rounded halfway upward, with the
percent unit announced by screen readers.

## Controlled slider values

All slider variants clamp controlled values to 0–1, resolve NaN to zero, and
snap discrete values to the nearest tick, rounding halfway positions upward.
The range end cannot fall below the normalized start. Prop updates neither
invoke `on_change` nor write back to caller state. Returning to continuous mode
restores the caller's original fractions.

## Date range selection and long-list scrolling

`date_range_picker` adds a continuous month calendar and a two-field input mode.
It shares the existing 48dp date targets and 40dp visual circles with the single
date picker. Inclusive endpoints use the primary color; the 40dp range band uses
secondary-container colors. Weekday labels stay above the scrolling months, and
the default 1900–2100 year range is virtualized. Month headings retain explicit
date semantics beneath their group node.

Selecting an earlier date restarts an unfinished range. Selecting after a
completed range starts a new one, and selecting the same date twice completes
a one-day range. Programmatic invalid ranges leave the existing selection
unchanged. Restrictions apply to endpoints. Manual input validates both drafts
together, including entering the end before the start. Explicit clearing also
clears invalid drafts that have never produced a selection. The example stores
the range and confirmation status at page scope.

The 360dp body, 68dp minimum header without host actions, title-large headline,
40dp range fill, and six 48dp date rows follow the current
[date picker specification](https://m3.material.io/components/date-pickers/specs)
and [reference range implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/DateRangePicker.kt).
Full-screen host actions, docked presentation, normalized reference images,
locale variants, and complete keyboard behavior remain separate audits.

`LazyListController::scroll_to` applies an explicit logical-position request
once, including after mounting. Both horizontal and vertical layout regressions
verify jumping and returning to the start. Shared indexed render slots keep month
contents inside an explicit Tessera carrier and read current year constraints.

The cross-month GPU probe also exposed an interpolation precision failure at
large virtualized offsets: an f32 position could stop short of its target and
leave the animation clock pending indefinitely. Interpolation now accumulates
in f64 while publishing the existing render position. Native regressions verify
both axes at large offsets with ordinary and slow smoothing, exact arrival, and
an idle clock. Direct input and inertia keep their existing position behavior.
The cross-month probe failed before this correction and passes afterward.

Native GPU checks use 1.5 density with 1× and 4× sampling, real pointer scrolling,
and IME text commits. They cover disabled endpoints, range-band colors and caps,
same-month and cross-month selection, mutable year limits, repeated month
requests, two-way input, invalid-draft clearing, and complete pixel recovery
after narrow and collapsed viewports. Existing single-date layout checks now
explicitly select 1× density. The complete component binary passes 316 tests;
all 107 component doctests, formatting/import checks, and the release WASM
build with Tessera call-color validation pass.

The first browser rendering exposed a narrower month grid caused by an
alongside scrollbar. Lazy lists now expose scrollbar visibility, and the range
calendar hides its bar so all seven 48dp columns align with the fixed weekday
row. GPU checks compare the outer weekday/date centers and target widths.
Month-heading insets also have explicit layout assertions. An initial heading
authored with padding directly on text drew at the unpadded origin; the heading
now uses a padded container. The shared drawing-bounds correction below also
covers padding authored directly on a drawing node.

Browser checks confirm joint manual input and Tab/Enter confirmation. A Saturday
Right-arrow sequence initially moved to the mode toggle; range-calendar left
and right navigation now request previous/next dates in traversal order. The
rebuilt WASM preview verifies Saturday-to-Sunday navigation, reverse navigation,
and five queued Right-arrow presses from October 29 followed by Enter selecting
November 3. A 390px viewport retains the range without horizontal date overflow;
switching to input mode and saving publishes the confirmed range. A bulk browser typing
request delivered only its first character, while individual key presses
delivered the complete date; the input-tool versus web-input boundary remains
unattributed, and bulk input is not yet verified by this observation.

The final WASM check also exercised Ctrl+V and a browser paste event with a full
date. Neither inserted text in the earlier build. The platform clipboard backend
returned `None` when initialized for WASM, and its read/write methods were
unimplemented on that target. This confirmed missing browser integration, separate from
the earlier typing-tool observation. Per-key entry of the end before the start,
joint error recovery, and Tab/Enter confirmation still pass after the drawing
correction. The browser clipboard was restored after the synthetic-date probe.

## Browser clipboard integration

Clipboard actions now enter a focused input channel and retain their order
with keyboard events, including navigation/layout deferral. Paste uses the
plain text carried by a browser clipboard event. The platform plugin allows
only clipboard keyboard defaults through Winit's canvas cancellation, restores
the previous default policy after dispatch, and unregisters its listeners on
suspend or shutdown. Other DOM input targets are left to the browser.

Copy and cut request an asynchronous browser clipboard write. Cut commits only
after success and only while the original text version and selection remain
unchanged. A failed write leaves the editor intact. Paste and accepted cuts use
the existing input-transform/on-change pipeline and clear composition through
the normal edit path. Read-only fields allow copying but reject edits; disabled
fields do not accept clipboard requests. Browser text is event-driven rather
than pretending a synchronous clipboard getter can read it.

The integration uses the [Clipboard Event API](https://www.w3.org/TR/clipboard-apis/)
for incoming user actions and browser clipboard writes for outgoing text.
The native GPU scenario exercises backward Unicode selection, accepted and
rejected writes, stale cut completion after a newer edit, joint transform and
change callbacks, focused routing, and read-only/disabled policy through real
text inputs. All 316 component tests, 122 core tests in the combined
headless/testing configuration, and 197 component/core doctests pass. The release
WASM build, call-color validation, formatting, and import checks pass.

Fresh-browser checks verify full-date Ctrl+V and paste events, replacing an
existing date with Ctrl+A/Ctrl+V, copy readback from the actual browser clipboard,
cut removal and write readback, and copying across fields with Tab. A rapid
cut/paste sequence restores the selected date even after seeding the clipboard
with a different date. The edited range saves through Tab/Space. A read-only
field copies its value and rejects both cut and paste without altering the
clipboard. There are no error-level or clipboard-warning logs. Clipboard contents
are restored after each probe. These observations do not force a delayed pending
write; delayed writes, suspended owners, browser permission differences, and
cross-browser lifecycle behavior remain audits.

## Modifier drawing bounds

The range-heading audit exposed a framework-wide drawing error: wrapping layout
modifiers reserved space and moved child nodes, but recording discarded those
layout layers. A text node with padding drew its glyphs at the outer origin.
Backgrounds and borders on either side of padding also saw the same outer size.

Measurement now retains the position and size of each wrapped content layer.
Drawing reads the bounds at its own position in the modifier chain, and emitted
scene commands receive that layer's local offset and default size. Explicit
command offsets remain relative to the active layer, while local render targets
retain their resource coordinates. Clip geometry uses the active layer origin.
Layout snapshots restore these bounds alongside child placements. A cached
placement update replays wrappers around the cached policy result so content
coordinates can change without remeasuring the policy or its child subtree.

The native 1.5-density GPU regression compares the entire image of a text node
with two padded backgrounds and an outer border against independently nested
containers. It failed before the correction at both 1× and 4× sampling and
passes afterward, including repeated cached frames, changed insets and ink, and
restoration. A separate regression checks placement-only movement, hiding
unplaced content, exact pixel restoration, and an unchanged policy measure count.
This probe also exposed that erased wrapping modifiers compared only full
equality, bypassing their measurement/placement equality methods. They now
honor those methods independently, while component props retain full equality
so a placement-only change still reaches the layout node. Fractional-density probes run in isolated
child processes; the isolated runner explicitly includes GPU tests so an
ignored child test cannot silently be skipped.

The wider regression caught scrolling clips authored after the content-offset
layout. Those clips now precede it and use the viewport bounds. Menus likewise
apply their custom paint-overflow clip before scrolling, retaining lateral
shadows while cropping long groups at the viewport edges. All 316 component
tests, 122 core tests in the combined headless/testing configuration, and 197 component/core doctests pass, including the opt-in
GPU scenarios. The final release WASM build and call-color check pass. A fresh
browser load verifies three menu levels, a third-level action followed by full
dismissal, clipped month scrolling, joint date input, and keyboard confirmation,
with no error-level logs. Browser navigation still produced warning-level frame
outliers, including 52–64ms encoding and one-off build/record spikes; these have
not been attributed and are not a performance-equivalence result. Multiple
clips and drawing before a clip still need a separate ordering audit.

## Split button size and disclosure verification

Split presets use 32/40/56/96/136dp visual heights and a 2dp inter-button gap.
Leading icons use 20/20/24/32/40dp; menu icons use 22/22/26/38/50dp. Resting
inner corners are 4/4/4/8/12dp, and pressed/hover/focus targets are
12/12/12/20/20dp. Menu icons use the site's explicit optical offsets of
-1/-1/-2/-3/-6dp at rest and center when the trailing segment becomes circular.
The disclosure arrow rotates 180 degrees. The critically damped effects spring
preserves velocity on reversal and stops requesting frames after settling.

Trailing segments retain width priority under tight constraints. Native GPU
checks cover 1×/4× sampling, 1.5 density, typography/icon contexts, all four
styles, dynamic sizes, disabled input, and stable disclosure semantics. Disabling
preserves `expanded` and `has_popup` while removing activation/focus actions.
Applying a color variant no longer silently enables a disabled segment.

The full native example opens menus from XS and XL triggers, selects both
commands, dismisses outside, resizes to a 390dp viewport, and retains its action
count after the virtualized item leaves and returns to the build tree. WASM
checks exercise all four styles, disabled primary/menu actions, keyboard
selection, Escape, Enter/Space reopening, arrow rotation, and restoration after
resizing. Split pairs now probe intrinsic content separately from interaction
padding and apply the larger visual height to both sides before normal
measurement. Native pixel readback covers a wrapped XS label growing from 32dp
to 40dp within a 48dp touch target, a wrapped XL label growing beyond 136dp,
mixed presets, disabled geometry, and restoration to identical initial pixels.
The first frame after each width/preset update must already align both sides.
Intrinsic probes use scratch metadata/caches and never overwrite the final
interaction layout. Layout snapshots track intrinsic dependencies: a child
changing from 40dp to 32dp must invalidate its parent even though both normal
measurements retain a 48dp touch target. A core regression checks cache/placement
isolation in both probe orders and verifies zero snapshot stores on idle frames.
WASM checks confirm equal 160dp visual heights for the wrapped XL pair at
390×780 and recovery to 136dp at 877×871. XS returns to its 32dp visual height.
The narrow menu remains anchored to the enlarged trailing segment; keyboard
selection increments the retained action count once. Temporary viewport
overrides are reset after verification.

State layers now use straight-alpha source-over compositing. Transparent fills
retain the overlay RGB instead of being darkened twice. Native GPU readback
compares transparent filled/outlined/combined surfaces with directly drawn
state-layer colors at 1×/4×, before and after caching. Surface-owned disabled
semantics update only their own fields and preserve caller disclosure state
and test tags.

Menu providers retain the main-content focus scope independently from the
popup scope. Opening and closing focus requests run after the rebuilt tree is
laid out so disabled or removed triggers cannot leave deferred requests that
steal focus on re-enable. Selection and Escape restore the invoking control;
explicit fallback targets remain supported. Tab and Shift+Tab close the menu
and traverse from the restored trigger in the rebuilt tree; arrow keys handle
menu-internal navigation. Native regression also checks that closing does not
steal focus that has already moved outside the popup. The latest WASM build
confirms ordinary selection/reopening, forward Tab to the following primary
action, reverse Tab to the preceding action, and XS/XL focus recovery across
viewport changes. Its action count remains five after scrolling away and back.
Menu visibility transitions now request a focus-owned keyboard barrier. The
remaining key/IME queues are retained until a subsequent layout and its focus
callbacks finish. Input resumes in order in bounded recomposition retries; any
remaining work schedules another frame. This barrier is explicit and does not
split ordinary typing batches. Native regressions protect the barrier phases,
new-transition preservation, event order, reset behavior, and existing menu
focus restoration. WASM rapid sequences completed five and then ten rounds of
Tab/open/reverse-Tab/activate with exact counts. Escape/reopen/choose-draft and
forward-Tab/activate reached the expected targets. The original pointer-open
then immediate reverse-Tab/Enter sequence now activates exactly once. Selecting
a draft and immediately reopening/activating also returns to the first menu
item; the retained count reaches 21 with the expected scheduled-send status.
Ordinary character/space typing and select-all/delete remain complete. These
checks cover the example menu; focus-triggered popups, nested-menu bursts, and
other modal providers still need their own interaction audits. The inspected build reports no browser
errors, and the viewport override is reset to 877×871 at 1.5 density.

## Menu hover intent

Mouse movement focuses enabled items. A 150ms dwell opens a submenu or closes
obsolete descendants when hovering a leaf. Leaving the source, moving keyboard
focus, disabling it, or closing the parent cancels pending hover intent.
Idle pointer dispatch does not steal keyboard focus or reopen dismissed branches;
touch motion does not create mouse hover. Only the topmost menu accepts hover.

## Shared rendering regressions

Carousel preparation adds `Modifier::clip_shape` for arbitrary subtree content,
including rounded corners with independent curvature and elliptical masks.
The renderer packs masks into one reusable upload buffer and pools stencil
attachments by size and sample count. Active masks are rebuilt after pass
boundaries; background sampling regions remain independent of the draw mask.
All built-in draw pipelines and the custom-shader example use
`tessera_ui::ui_stencil_state()`. Custom UI pipelines need the same configuration;
independent atlas-generation passes retain their own attachment configuration.

Native GPU regressions cover nested masks, raster images, text, elevation,
blur/contrast scene sampling, removal/reinstatement on cached layouts, clipped
pointer targets, and release cancellation at 1× and 4× sampling. The 4× case
asserts partial edge coverage at fractional positions. This exposed a shared
draw/scissor rounding bug: draw and sample bounds now round outwards, and negative
origins no longer extend the opposite edge. Single-sample stencil masks have
hard edges; smooth mask edges require multisampling. The example now uses 4×
multisampling on native and WebAssembly.

Carousel GPU checks also exposed shared-image uniform overwrites: repeated
draws of the same image previously used the final draw's position and opacity.
The image pipeline now separates reusable texture resources from a reusable,
aligned per-draw uniform arena. Native checks exercise arena growth with forty
copies of one image and verify that settled frames neither allocate new instance
buffers nor upload textures again. Images use sRGB source textures and linear
premultiplication so transparency and parent opacity apply exactly once.
Pixel readback checks cover both half-opacity and translucent source images.

Mixed-aspect uncontained carousel items preserve their 9:16–16:9 aspect ratios
while scrolling. Missing/invalid ratios use square items. The layout defaults
to 8dp spacing and 16dp leading padding; width follows the measured viewport
height, including dynamic ratio/height changes. Prefix geometry stores only
explicit ratios and virtualizes large collections. Visible width intersects the
viewport at both ends. Native geometry/GPU tests verify that intersection, and
WASM checks confirm that partially clipped captions are hidden while fully
visible captions remain readable during dragging. Pointer dragging keeps
keyboard navigation focused; navigation must realign the current item when
free scrolling leaves it between its snap positions, even if its index is
unchanged. Native regressions cover both boundaries and drag focus. The rebuilt
WASM example also returns a partially scrolled first item to its exact start
with Home immediately after dragging, restores its caption, and does not
activate the item.

Full-screen carousel pages use vertical layout and snapping, rectangular masks,
and 16dp page spacing. A portrait preview appears in the example; Up/Down keys
and vertical accessibility actions follow that orientation. Other carousel
layouts continue to pass vertical wheel input to their surrounding page.

The complete native example also passes a 1316×1307, 1.5-density carousel
scenario covering all six layouts, item activation, reverse/disabled controls,
window resizing, and vertical wheel propagation to the surrounding page.
Navigation and activation state also survive scrolling the carousel out of
the lazy viewport and back.
Native and WebAssembly example builds and their color checks pass after these
changes. Browser carousel state comparisons are tracked separately above.

Responsive browser checks exposed density changes retaining measurements and
build-time pixel values from the previous scale. Density changes now trigger
root recomposition while retaining remembered state. Native tests change density
without any component state write, check text measurement and raster positions,
then restore the original density and pixels. Headless snapshots report the
current density. Web surface sizes use CSS content dimensions and the same
density as pointer input, including when browser emulation reports a different
device-pixel content box. Browser checks at 390×780 CSS pixels and the default
877×871 viewport preserve a noninitial item and following text, including after
idle frames and a trip out of the lazy viewport.

The same scenario exposed stale estimated-size baselines in lazy-list and
regular-grid prefix sums. Changing an estimate, including through DPI conversion,
could make offsets negative or overlap already measured items. Both caches now
rebase measured deltas when their estimate changes; grid geometry also resets
when the column count changes. Regression checks cover mixed measured/unmeasured
entries, offset lookup, count/column changes, and live list estimate/density
updates. Grid state writes are skipped when measurements and extents are
unchanged. The staggered-grid cache already invalidates its lane geometry when
its estimate changes.

Carousel navigation also preserves its index through a temporary zero-width
measurement. Native GPU checks cover a noninitial item across density and
viewport changes, including a lazy-list parent and a responsive feed whose
following item must remain visible and contain rendered text pixels after a
1px-wide window recovers. List and grid caches retain usable measurements while
the cross axis is collapsed instead of recording extreme wrapped-text heights.
Separate list and fixed-grid regressions check their caches before, during,
and after collapse, including grid spacing that remains positive while all
cells have zero width.
The example keeps carousel navigation, layout, and selection state at page scope
so virtualized item removal does not discard it.
Carousel item widths are reactive getters rather than captured numeric snapshots,
so nested content can hide labels as an item becomes small. The responsive GPU
check verifies the label changes after resize; the example hides titles below
120dp of visible width. Native and WASM checks verify titles disappear as items
shrink and return when they grow.

Layout-only state changes previously stayed pending when component composition
was skipped. Layout now drains those updates directly, and the renderer includes
pending layout work when scheduling another frame. A regression changes a
measure-only width from 40px to 80px and 120px without a component rebuild; it
failed before the fix. This prevents cached geometry from remaining stale until
another input event. Carousel wheel snapping also uses the time supplied by its
frame callback rather than recursively reading the borrowed frame clock; native
GPU and browser wheel checks cover the transition from scrolling to snapping.

Offscreen culling also uses enclosing bounds so a fractional sliver at a viewport
edge is retained; a 4× GPU assertion checks that coverage. Clip state changes
preserve nonoverlapping sampled-draw batching while clips following atlas/local
work explicitly target the scene. Core tests cover both planning cases.

Native and WebAssembly color checks and example builds pass. The complete native
example passes the 1316×1307, 1.5-density headless smoke scenario. The rebuilt WASM
example renders the custom shader and Material page, completes a rich-tooltip
action/dismiss cycle, and reports no browser error logs. WASM carousel checks now also cover rounded masks, rectangular portrait pages,
vertical wheel snapping, Up/Down and Home/End navigation, drag cancellation,
horizontal navigation, responsive titles, and retained state. Screenshot
evidence is stored outside the repository in the local QA artifact directory.

The extended FAB expansion check exposed two cache issues: draw opacity and
clipping could survive between record passes, and a draw-only text rebuild could
lose its measured glyph buffer. Record state now resets independently of layout
reuse. Shaped text persists on the component instance and is keyed by text,
typography, physical font size, and measurement constraints.

Native GPU readback verifies that a small extended FAB returns to exactly the
same pixels after collapse and expansion. A second readback test verifies
constant opacity across repeated cached frames, recovery from zero opacity, and
removal of clipping. These tests require a native GPU adapter and are run explicitly:

```sh
cargo +stable test -p tessera-components --lib rendered_labels_restore -- --ignored --test-threads=1
cargo +stable test -p tessera-components --lib cached_layout_rebuilds_opacity -- --ignored --test-threads=1
cargo +stable test -p tessera-components --lib rendered_menu_restores_pixels -- --ignored --test-threads=1
```

The FAB menu check also inspects the published AccessKit tree through
`HeadlessRenderer::accessibility_snapshot()`. Layout wrappers must preserve the
menu group and action items, and the toggle must expose its expanded state and
popup kind. This complements pointer, focus, and screenshot checks.

Loading shapes are compared against an independent scanline reference sampled
from their cubic paths. The GPU test covers eight morphs, three progress values,
rotation, and both single-sample antialiasing and 4× MSAA. Coverage error is the
sum of absolute per-pixel alpha differences divided by reference covered area;
its 2.5% bound accounts for rasterization differences and is not a claim of
identical edge pixels. A second GPU test checks progress semantics, animation,
pause, pixel restoration, and stable instance-buffer allocations.

```sh
cargo +stable test -p tessera-components --lib gpu_loading -- --ignored --test-threads=1
```

The repeating animation uses a 650ms morph interval, a 4666ms global rotation,
and a stiffness-200, damping-0.6 spring with a 0.1 visibility threshold. Duration
(297ms) and sampled values were checked against AndroidX Animation Core 1.12.0.
The example's paused WASM screenshots were identical across separate captures;
restoring playback resumed shape changes. Asset regeneration with the pinned
geometry dependencies reproduced the committed binary byte for byte.

Centered slider tracks preserve both handle gaps at 50% and near the midpoint.
Single and range handle placement retains half-pixel centers for odd physical
widths; geometry tests cover odd/even widths and unequal range handles. Native
1×/4× GPU readback checks the empty gaps before and after shape caching.
WASM checks confirm the default midpoint, keyboard adjustment to 51%, dragging,
disabled input, and recovery between 877×871 at 1.5 density and 390×780 at
1.0 density. The window override is reset after verification.

Slider keyboard tests cover continuous/discrete increments, Home/End bounds,
PageUp/PageDown, modifier shortcuts, repeated events, and disabled input. Range
handles retain independent names, fixed reading/Tab order, and dynamic peer
limits. Retained callbacks preserve the latest opposite endpoint when values
change, and switching discrete ticks rebuilds the matching layout children.

The published accessibility tree now includes numeric bounds and uses stable
component instance identities rather than arena indices. A rebuilt control must
retain its identity, and an action addressed to it must reach its current node
even when its arena index has changed. A native GPU test verifies both range
handle identities and peer bounds before/after value and step changes:

```sh
cargo +stable test -p tessera-components --lib published_slider_semantics -- --ignored --test-threads=1
```

The example Material Components page contains interactive previews. Passing its
build, unit tests, or rustdoc tests does not complete this checklist.

Tooltip providers render above page content so a scroll viewport does not clip
their panels. Plain help uses a 24dp minimum height, 4dp corners, 8dp horizontal
padding, and inverse surface colors. Rich help uses 12dp corners, 16dp horizontal
padding, and surface-container colors, with optional title and action slots.
The custom action slot supports the two-button configuration.

Native GPU checks preserve the trigger's focus during hover, publish its help
as a description, and recover the original pixels after dismissal. They also
change supporting text and the plain/rich variant while open. Touch checks cover
the 500ms threshold, cancellation instead of click on release, later normal
activation, 1500ms transient dismissal, and movement cancellation. Stable
visibility stops requesting timer frames.

```sh
cargo +stable test -p tessera-components --lib tooltip::tests::gpu_ -- --ignored --test-threads=1
```

These checks exposed a headless/window mismatch: headless frames now flush
global-position callbacks after layout, matching window rendering. Platform
touch identifiers are mapped to independent nonzero pointer identifiers so
touch zero cannot alias the mouse. Display text wraps at word boundaries and
splits an oversized word only when necessary.

Published focus now comes from the current focus owner when available, with a
semantic ancestor for layout-only wrappers. Native tooltip tests enter the
action group, return to the trigger, and enter again while checking the
published control identity.

Keyboard events now retain the modifier state present when each event arrived.
Dispatch resolves the current focus chain for each event, so queued navigation
can change the target of subsequent activation. Unprocessed keys and IME commits
survive a lazy focus retry and root recomposition; a runtime reset clears them.
Core regressions cover modifier changes, routing order, retry ordering, and one
IME delivery to the final focused control. Headless callers can forward modifier
changes with `HeadlessRenderer::set_key_modifiers`.

Slider input reads a shared live value between key dispatches. Range endpoints
also share their current limits instead of capturing the other endpoint from a
previous build. Rapid WASM sequences verify three increments from 0% to 3%,
forward/reverse Tab, and range convergence to 5% at both ends.

Tooltip dismissal restores focus at the end of pointer or keyboard dispatch.
Deferred focus callbacks read current ownership instead of replaying obsolete
intermediate focus states that could reopen an already-dismissed panel. A native
GPU regression performs several focus transfers inside one input dispatch and
checks dismissal and restoration. Separate cases exercise the default action
and a two-button custom slot while the action also invalidates the parent
boundary; both recover the original pixels after interactions settle. The native
cases also nest two tooltip providers with an explicit external anchor controller.

The full WASM example exposed a focus-command retry bug: requesting an already
focused action queued an unchanged request. After dismissal restored the anchor,
the queued request focused the action again and reopened the panel. Focus commands
now defer only while their target is unattached; attached no-ops and rejected
requests cannot become future focus intents. Core regressions cover repeated
requests, clear/capture/free operations, scope restoration, attachment, and disabled
targets. WASM verifies the rapid `Tab`, `Shift+Tab`, `Tab`, `Tab`, `Enter` sequence,
then trigger re-entry and another action cycle; the panel stays closed after each
action. Closed popup input cannot reopen a dismissed tooltip, and touch suppression
clears on actual focus exit. The example uses an explicit `RenderSlot` for its page
content.

The complete example also passes a native headless check at the browser's actual
1316×1307 physical viewport and 1.5 scale factor. A cancelled press first focuses
the action without activating it; pressing it again exercises the already-focused
request. The rich panel count falls from one to zero and the action status becomes
`done`. Offline session startup now resolves binary metadata without loading
unused dependency packages.

Web tracing now writes complete records to the browser console, preserving severity
and omitting terminal colors. This made the controller transition trace observable;
the temporary probe statements have been removed.
These checks do not establish complete visual equivalence or cover every
keyboard sequence that creates a new overlay before the next build.

Sequential headless startup remains a lifecycle audit: constructing a second
renderer with the same entry on one thread exposed a stale controller handle
during its initial build. GPU scenarios currently use separate test entrypoints.


## Date range calendar traversal cost

The range calendar's scroll slowdown was CPU work. Every date previously built
an interactive transparent surface around its visual surface, and every grid
cell built a range-fill surface even without a selection. Those nodes increased
composition, layout traversal, draw recording, and command encoding across the
visible months. The shared day component now uses a centered layout with the
existing clickable modifier for its 48dp target, retaining the 40dp visual
surface. Range backgrounds use a lightweight layout background only for dates
inside a completed range. The single-date picker shares the day improvement.

Release headless comparisons use the complete example at 1316x1307 physical
pixels, 1.5 density, 4x sampling, and the same DX12 adapter as the carousel probe.
Both runs navigate through 22 page-scroll steps, settle for 31 frames, then
scroll the range calendar by 12 physical pixels per frame for 60 frames and
settle for another 90 frames. Values below are mean milliseconds over the 60
calendar-scroll frames. CPU frame totals include profiler metadata collection;
GPU timestamp readback waits are outside those totals. Draw recording is part
of the layout-and-record interval rather than an additional frame cost.

| Measurement | Before | Fixed |
| --- | ---: | ---: |
| CPU frame with profiling | 28.58 | 9.19 |
| Component construction | 2.74 | 1.08 |
| Layout, traversal, input, and draw recording | 13.76 | 4.76 |
| Draw recording | 4.97 | 1.84 |
| CPU command encoding | 3.50 | 0.64 |

The representative mid-scroll tree shrinks from 13,022 to 7,796 nodes. The
initial and final-scroll PNGs are byte-identical between runs. The baseline GPU
timestamp averages 0.22ms, confirming that GPU execution was not the dominant
cost. Raw traces and the summary are in the ignored `target/date-range-perf`
directory. These measurements compare this scene and do not guarantee a frame
rate across all hosts.

The existing 1x/4x GPU scenarios still cover range bands and caps, disabled
dates, cross-month selection, scrolling, input mode, year restrictions, and
resize recovery. They now also click outside the 40dp visual circle but inside
the 48dp target and require the correct selected date. All 342 component tests
including GPU scenarios and all 110 component rustdoc examples pass.

The release WASM build and Tessera call-color check pass. A separate 1280x720
browser session verifies forward/backward month scrolling and selecting an
October 18–22 range with the expected endpoint circles and band. The isolated
month-scroll interval has no slow-frame or error records. Initial navigation
through the date sections still records 42–53ms construction frames, down from
the earlier 94–133ms observations; cold mounting is not eliminated by this fix.

## Date range calendar cold construction and visual nodes

The first range-calendar mount previously used the lazy list's fallback of
eight estimated items before measuring its viewport. Each item was a complete
336dp month, so the cold frame constructed nine months for a 360dp calendar.
The calendar now supplies its known viewport cap, and lazy lists apply that
cap to item selection before measurement as well as to reported layout size.
The same cold frame now constructs two months. A production-component layout
test checks the initial construction bound on both list axes without timing
assertions.

Calendar dates now draw their explicit fill, outline, and interaction state
through the existing shape pipeline instead of constructing a general surface
and its content-color/elevation providers for every date. The shared single
date picker benefits as well. The 48dp target, 40dp visual, fractional text
centering, date semantics, and existing pointer handling are retained.

The release headless comparison uses the complete example at 1316x1307 physical
pixels, 1.5 density, and 4x sampling. Identical navigation reaches the range
calendar, settles for 35 frames, then scrolls by 12 physical pixels per frame
for 80 frames. CPU times include profiling overhead; GPU timestamp readback
waits are outside CPU frame totals. These are local measurements, with other
build activity on the host, rather than a cross-device frame-rate guarantee.

| Measurement | Before | Fixed |
| --- | ---: | ---: |
| First range mount: component construction | 20.80ms | 3.32ms |
| First range mount: CPU frame | 47.34ms | 10.02ms |
| Scroll: mean CPU frame | 7.59ms | 6.85ms |
| Scroll: mean component construction | 1.19ms | 0.92ms |
| Scroll: mean GPU timestamp | 0.18ms | 0.17ms |
| Settled scene nodes | 3096 | 2544 |

The initial and final-scroll PNGs are byte-identical between runs. Raw traces,
snapshots, and the compact comparison are in the ignored
`target/date-range-current` directory. Native 1x/4x GPU range scenarios also
check that hovering paints only the circular visual and leaving clears the
state layer, alongside selection, disabled dates, range bands, manual input,
and narrow/collapsed resize recovery. Nested lazy-list GPU scenarios select a
date, verify focus, scroll until its month is unmounted, and check that focus
does not move to an unrelated control or change the outer scroll position.
All eleven date-picker tests and all 285 non-GPU component tests pass.

The rebuilt WASM example also renders and selects a cross-month range. A
separate focus-lifecycle audit identified the outer-page jump observed while
scrolling a focused month out of view; the corrections are described below.
A browser frame also reported 30.3ms in draw-command work with no component
construction. The cold-construction measurements above do not establish that
all scrolling stalls are resolved across devices and browsers.

## Calendar keyboard navigation

Both calendar bodies now use the [date-picker keyboard conventions](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/examples/datepicker-dialog/):
one date participates in Tab navigation; arrows move by day or week; Home/End
use the configured week start; Page Up/Down move by month; shifted page keys
move by year. Short months and allowed-year endpoints clamp the target date.
Focus movement preserves selection until Enter or Space activates a date.
Unavailable dates remain discoverable during calendar navigation, with disabled
semantics and a visible focus layer, while activation stays disabled. This is
an intentional composite-control policy consistent with the
[focusability guidance](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/#focusability-of-disabled-controls).

A shared calendar focus handle is rebound to the current date stop. Requested
targets receive focus after their new bounds are published, including when a
different virtualized month must be built. Following keyboard input waits for
that layout/focus transition, preserving rapid navigation followed by selection.
Calendar arithmetic uses wide day counts so supported integer-year endpoints
do not overflow intermediate calculations.

Three algorithm tests check day/week movement, Sunday/Monday week boundaries,
month/year changes, leap days, short months, and year limits. Existing native
1×/4× range scenarios additionally verify one-stop Tab exit without selection
changes and request focus on an unavailable date through the production
controller, checking disabled semantics, visible focus pixels, and pixel
restoration after clearing focus. All 16 date-picker tests, all 288 non-GPU
component tests, and 111 component rustdoc examples pass. Actual key-event
routing is checked in the browser rather than inferred from arithmetic tests.

WASM checks at 907x871 CSS pixels include October 29–November 7 range selection
through week boundaries, month/year keys, and Enter; reverse Tab and Space then
select November 14–December 14 and reach Save. Single-date checks include
October 31→November 30 focus, rapid mixed navigation, February 29, 2024→February
28, 2025 focus without premature selection, Enter/Space activation, and both Tab
directions through Cancel/OK. At 390x844, range day/week/month navigation and
Space select November 14–December 22. The viewport override is reset afterward.
Screenshots are in `tessera-calendar-keyboard-qa` under the local visualization
directory.

A release headless comparison for the keyboard implementation, before the
semantic-container correction below, replays the same complete-example navigation at
1361x1307 physical pixels, 1.5 density, and 4× sampling, then scrolls the range
calendar by 12 physical pixels for 80 frames. The initial and final-scroll PNGs
are byte-identical, sampled node counts match, and cold construction still
mounts two months. Local measurements are:

| Measurement | Prior binary | Keyboard build | Repeat |
| --- | ---: | ---: | ---: |
| Mean CPU frame | 9.73ms | 10.99ms | 11.43ms |
| 95th-percentile CPU frame | 12.54ms | 13.61ms | 14.10ms |
| Mean component construction | 1.27ms | 1.33ms | 1.43ms |
| Mean GPU timestamp | 0.094ms | 0.117ms | 0.101ms |

The baseline executable predates shared headless focus cleanup as well as the
keyboard changes, so these timings do not isolate keyboard overhead. A matched
lifecycle baseline is needed before attributing the CPU increase. Raw traces,
frame marks, screenshots, and a compact summary are in the ignored
`target/date-keyboard-current` directory; orchestration ran in memory without
adding a temporary script file.

Resizing the complete browser example from its normal viewport to 390px
changes the visible section from the single-date picker to the range picker;
selection values survive and widening returns to the single-date section.
Outer lazy-list scroll-anchor retention remains open. Docked/modal variants,
year-grid keyboard behavior, full grid semantics, dark themes, and normalized
reference screenshots also remain on the catalog checklist.

### Calendar semantic container publication

The single-date picker, its date grid, and its year list attached semantic
containers with descendant publication disabled. This hid interactive content
from the published tree even though layout snapshots and pointer input worked.
The containers now publish their descendants as named groups. Before the fix,
the new native regression observed zero published July date buttons instead of
31. Native 1×/4× scenarios now verify all July dates, focused-date publication,
one-stop Tab exit, published year selection and its month update without date
selection, and the manual-entry text input. This is distinct from the remaining
full grid-role and year-grid keyboard audits.

The final WASM build passes the call-color check. Its browser confirmation
selects October 31, moves to November 30, advances to November 30, 2027, then
uses Enter and two Tab steps to confirm that date. No browser errors are
captured during this sequence; `calendar-keyboard-final.png` records the result.

The other conditional test-tag containers were inspected as well: FAB menu
groups already publish descendants, extended FAB content deliberately hides
decorative content, and the clock dial is a single focusable interaction surface.
Its role/value/action semantics need a separate time-picker audit.

### Range calendar cell construction and idle drawing

A browser revisit captured a 61.9ms entry frame (43.7ms construction, 12ms
layout/command processing, and 6.2ms rendering). Crossing a month also captured
a 72.6ms frame dominated by construction. These CPU-side warnings are distinct
from GPU timestamp execution; another frame spent 38.5ms in layout/command
processing without construction. Host compilation was active during this
inspection, so these observations identify phases rather than a stable frame
rate.

The range calendar now places its range band in the date's existing target
layout instead of wrapping every grid cell in another layout component. Empty
and unselected cells no longer construct that wrapper. Range endpoints retain
their half-cell bands and the shared date target retains its 48dp hit area and
40dp circular visual. Idle dates with no fill or outline also omit their
fully transparent circle command; text and active interaction layers remain.

The existing 1x/4x native range scenarios additionally check both inward band
edges and the absence of a band outside a same-day selection. Their retained
assertions cover hover clearing, disabled focus, selection, nested scrolling,
manual input, and pixel restoration after narrow and collapsed viewports.

Verification passes all 16 date-picker scenarios, including native 1x/4x GPU
coverage, and 288 ordinary component tests. Formatting, import grouping, the
Tessera call-color check, and the release WASM build pass. In the rebuilt
browser example, October 29-31 keeps its joined range band; selecting October
29, scrolling October out of view, then pressing Right and Enter completes
October 29-November 1 without moving the outer page. Browser proof is saved
as `range-band.png` and `range-scroll-focus.png` in the local visualization
directory, outside the repository.

After compilation finished, a paired release headless measurement ran both
executables sequentially with identical keyboard/focus lifecycle code, at
1361x1307 physical pixels, 1.5 density, and 4x sampling. Each run selects
October 29-31, settles, then scrolls 12 physical pixels per frame for 80 frames.
The baseline already contains the two-month construction cap and semantic
publication fixes; this comparison isolates the subsequent cell-layout and
transparent-draw changes rather than the earlier keyboard work.

| Measurement | Before cell optimization | After |
| --- | ---: | ---: |
| First range mount: scene nodes | 2296 | 2044 |
| First range mount: CPU frame | 17.10ms | 14.99ms |
| Scroll: mean CPU frame | 10.75ms | 8.88ms |
| Scroll: P95 CPU frame | 12.47ms | 10.86ms |
| Scroll: mean construction | 1.37ms | 1.14ms |
| Scroll: mean layout/command processing | 5.63ms | 4.74ms |
| Scroll: mean command recording (included above) | 2.15ms | 1.78ms |
| Scroll: mean GPU timestamp | 0.082ms | 0.095ms |

Initial and final-scroll PNG files are byte-identical. CPU frame time falls
about 17% in this local sample; the tiny GPU timings do not establish a GPU
speedup. Raw traces, frame marks, snapshots, and `band-summary.json` are in the
ignored `target/date-keyboard-current` directory. The temporary baseline
executable was removed after measurement. Browser startup and large outer-page
jumps can still produce cold frames; this result is not a guarantee that every
browser frame meets a fixed refresh-rate budget.

## Time picker values, selection, and keyboard operation

The dial now exposes individual 48dp buttons with hour/minute labels, values,
selection, focus, and activation actions, following the
[time-picker accessibility guidance](https://m3.material.io/components/time-pickers/accessibility).
Selection is independent of checked state. AM/PM exposes a radio group and
checked radio buttons. Native accessibility focus requests route to registered
focus targets without invoking activation callbacks.

Only one value in the dial participates in linear traversal. Arrow keys move
focus around its values without selecting them; Home/End choose the first/last
focus destination. Enter or Space selects the focused value without switching
to minutes. Pointer hour selection retains its automatic switch to minutes,
and its release cannot also activate a newly mounted minute button. AM/PM has
one Tab stop and arrow-key selection. Mode, field, and period changes defer
following keys until the new layout is ready.

Explicit value actions and manual input preserve exact values even with custom
steps; step parameters apply to pointer snapping. Retained handlers read live
state, and pointer snapping updates with changed props. Switching a focused
inner 24-hour value to 12-hour format restores a valid dial focus destination.
The example exposes format switching, the selected time, and a time-specific
confirmation status independently of date-picker actions.

Native 1x/4x GPU scenarios exercise published roles, labels, selected/checked
states, target sizes, stable node identities, focus pixels, forward/reverse
traversal, live step changes, pointer releases, accessibility actions, and
manual-entry transitions and actual IME commits to both hour and minute inputs.
Core regressions cover selection publication and
focus actions independently of activation. The release WASM build and call-color
check pass. Browser checks at 907x871 verify arrow focus without value changes,
Enter selection, one-stop dial exit, period navigation, rapid mode/Tab changes,
manual entry via paste, keyboard confirmation, and Space selection of the inner
ring's 23-hour value while retaining the minute value. At 390x844, the dial
and its actions fit without horizontal clipping. Viewport restoration retains
the selected time. Proof images are stored outside the repository under
`tessera-time-picker-qa`.

Verification passes 130 core tests with headless/testing/profiling enabled
(two GPU scenarios remain opt-in), 17 foundation tests, the full 356 component
scenarios including 68 opt-in GPU cases, and the affected runnable rustdoc
examples. The two time-picker GPU scenarios pass again after extending manual
entry coverage. Formatting and import grouping pass.

Strict Clippy remains blocked by existing warnings in the macro, glyphon, core,
and component code. These include enum size, argument counts, and existing
unwrap/closure lints; this batch does not establish a clean strict-lint baseline.

At this checkpoint, horizontal/adaptive layouts, full modal integration, normalized light/dark
reference states, and motion comparisons remain open. The current WASM
AccessKit adapter does not publish a browser semantic tree, so the native
accessibility checks are not evidence of Web screen-reader support.

## Horizontal time picker and period hit targets

`TimePickerLayout::Horizontal` places the dial beside the time controls.
Select it with `time_picker().layout(...)` or
`time_picker_dialog().picker_layout(...)`. Vertical remains the default. Below
524dp of available content width, horizontal preference falls back to vertical.
The same layout nodes, remembered input controllers, and focus requesters remain
mounted while their placement changes.

The [website measurement diagram](https://m3.material.io/components/time-pickers/specs)
specifies 96dp time fields, a 24dp separator, a 52dp gap before the 256dp clock,
and a 216x38dp horizontal period selector. The horizontal dialog omits the
vertical header gap, retaining its 24dp action spacing. The resulting body is
524x256dp; normal 24dp dialog padding makes the complete width 572dp. Horizontal
24-hour fields use 96dp rather than the portrait variant's 114dp width.

The period selector previously constrained its clickable surfaces to 36/40dp
in vertical mode. Its visual group now paints independently of the radio
targets. Each target is 48dp high: horizontal targets extend above and below
the 38dp visual row; vertical targets extend only beyond the outer edges and
meet at the shared boundary. The visual clipping does not clip their input or
semantic bounds. Shared interaction state keeps focus/hover feedback within
the visible group, and decorative labels are excluded from accessibility.

Layout regressions cover both formats, the exact 524dp boundary, and narrower
containers. Native 1x/4x GPU scenarios retain the focused period's published
identity through wide/narrow/wide resizing, check the 96dp horizontal 24-hour
field, and activate the extended hit areas with real pointer samples.
Caller-provided semantic roles and labels remain intact. A constrained-width
dialog regression verifies that the vertical header gap returns when the picker
itself cannot accommodate the horizontal body, even inside a wide parent.

The final six affected tests pass, including both opt-in GPU scenarios. The full
357-test component suite passed before the final constrained-width and semantic
preservation refinements; the affected tests were rerun after those refinements.
Both runnable time-picker rustdoc examples, the final release WASM build, and
call-color checking pass.

Browser checks on that final WASM build verify horizontal 12/24-hour layouts,
period selection with arrows, dial focus movement without a value change,
Enter activation, manual entry of 19:56, and keyboard confirmation. At 390x844,
the horizontal preference falls back to a complete vertical picker without
horizontal clipping; restoring 907x871 restores horizontal placement and retains
19:56 and the confirmation status. Browser error logs are empty. The outer lazy
list still shifts its scroll position during resizing, so preserving the visible
item's viewport anchor remains open. Proof images are stored outside the
repository in `tessera-time-picker-qa`: `wasm-horizontal.png`,
`wasm-horizontal-24-hour.png`, `wasm-horizontal-input-confirm.png`, and
`wasm-horizontal-narrow.png`.

Automatic layout selection, short-window modal behavior, complete normalized
reference states, dark themes, and motion comparisons remain open.

## Radio semantics, selection, and interaction colors

Radio groups publish a `RadioGroup` with optional accessible name and description,
preserving their explicitly labelled radio children. Disabled radios retain their
role, description, checked state, and 48dp target while exposing no focus or click
actions. Re-enabling retains the semantic node identity. Directional traversal
wraps by default and skips disabled choices; `wrap(false)` remains available.

The directional-selection observer also preserves interaction feedback, avoiding
an overwritten callback that previously moved focus without changing the selected
value. Arrow traversal selects immediately. Tab, explicit accessibility focus,
and automatic focus restoration preserve the selected value. A pointer press only
moves focus: selection waits for a completed click, and dragging away cancels it.
The native fixture tests these production input paths rather than duplicating
selection logic.

Current [Radio tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/RadioButtonTokens.kt)
use `on_surface_variant` for an idle unselected ring and `on_surface` for its
hover/focus/press states. An explicit unselected color override applies to both
idle and interacting states. Native 1x/4x GPU readback checks these roles, the
38% disabled `on_surface` ink, and selected `primary` ink at 1.5 density. A separate
unselected radio outside the group protects focus feedback without implicitly
selecting it, and ensures group wrapping cannot escape into a following control.

The example supplies three choices, a middle-option enable toggle, a named group,
and a selection-change counter. Full normalized reference snapshots, dark themes,
label alignment, horizontal/RTL visual behavior, toolbar-specific keyboard
integration, and motion fidelity remain open. The
[radio-group keyboard pattern](https://www.w3.org/WAI/ARIA/apg/patterns/radio/)
defines the default keyboard behavior and the separate toolbar behavior.

The integrated native binary passes all 378 component tests, including the
explicitly enabled GPU cases. All 111 documentation examples and the example
check pass. The final release WASM artifact also runs successfully in the browser.
Real Up/Down input selects B, C, A, then C with exactly four changes. Disabling B
and navigating from A skips to C; clicking disabled B leaves the value and count
unchanged. After re-enabling B, dragging its press outside leaves C selected;
completing a click selects B once, and Enter on that selected option does not add
another change. Browser error logs are empty. The external
`wasm-radio-navigation.png` records the final state. These results apply to the
shared-workspace snapshot used for these runs, not subsequent layout-core edits.

### Cross-axis radio navigation

Radio groups use declaration-order traversal for all four arrows: Right/Down move
forward and Left/Up move backward. The orientation parameter describes the group
to assistive technology without restricting which arrow keys work. Native 1x/4x
GPU scenarios cover both announced orientations, disabled-option skipping, edge
wrapping, preserved group identity, and a following focusable control that must
not receive a wrapped arrow move. Tab behavior is covered separately below.

The expanded native GPU scenarios pass at 1x/4x MSAA and all 111 documentation
examples pass. Release WASM and call-color checks pass. In the rebuilt browser
example, Right, Right, Right, Left select B, C, A, C with exactly four changes.
After disabling B and clicking A, Right then Left skip B in both directions and
finish on A with seven changes. Browser error logs are empty;
`wasm-radio-cross-axis.png` records this state outside the repository.

### Radio Tab entry, exit, and restoration

An opt-in focus policy gives each Radio group one global Tab stop while retaining
all enabled choices for directional traversal. Entry from either side prefers the
selected enabled choice, then the first traversable choice. Tab and Shift+Tab leave
the whole group from any focused choice. A focused choice becoming disabled
restores focus inside the surviving group without changing its checked value.
Explicit caller traversal exclusions remain intact.

Focus notifications carry their traversal direction through callback dispatch.
Radio selects on arrow traversal and completed activation; Tab entry into an
unselected group and explicit accessibility focus do not select a choice. The
direction is scoped to each callback and restored after nested callbacks.

The expanded native scenario covers forward/reverse entry and exit, no-selection
entry, disabled focused-choice recovery, caller-provided traversal exclusions,
and direction context cleanup, alongside the existing pointer and pixel checks.

The final integrated native binary passes all 378 component tests, including
opt-in GPU cases, and all 112 core unit tests pass. The 111 component and 90 core
documentation tests, example check, release WASM build, and call-color check pass.
In the browser, clearing selection and pressing Tab focuses A with `None` and
zero changes. Right selects B once; Tab moves directly to the following text
field. Shift+Tab returns to B, and another Shift+Tab reaches the preceding clear
button without changing selection. Reverse entry into an unselected group also
focuses A without selecting it. Space selects A, and subsequent Enter does not
repeat activation. Browser error logs are empty. External
`wasm-radio-empty-tab-entry.png`, `wasm-radio-tab-exit.png`, and
`wasm-radio-tab-activation.png` record these checks. These are integrated snapshot
results; subsequent visual and group-ownership work requires its own checks.

### Internally managed radio groups

Omitting `selected` on every choice now gives the group responsibility for mutual
exclusion, including choices using an external controller. Explicit `selected`
values remain application-controlled; a group should use one ownership mode
consistently. Selecting another choice clears the previous controller without
emitting a second activation callback. An initially selected controller and later
controller-driven selection participate in the same rule. Group state retains
only a non-owning handle and checks its liveness before clearing a removed choice.

The native regression first reproduced both One and Preset remaining checked
after clicking One. The updated 1x/4x GPU scenarios verify pointer activation,
accessibility activation, directional activation, repeat activation, independent
groups, selected-choice removal and reinsertion, external controller updates,
and exact callback counts through the real components. The example adds a
horizontal group without `selected` props and with an activation counter.

All 380 integrated native component tests pass, including opt-in GPU cases. The
111 component documentation examples, example check, release WASM build, and
call-color check pass. In the browser, clicking Local A, B, then C leaves only C
selected with three activations. Repeating C does not increment the count; three
Left presses select B, A, then C and produce six activations. The independent
application-controlled group remains on A with zero changes. Tab leaves the local
group for the following text field, and Shift+Tab returns to C without another
activation. Browser error logs are empty. External `wasm-radio-uncontrolled.png`
records the result; normalized reference visuals and toolbar integration remain
open.

## Standard segmented button corrections

The current [segmented button overview](https://m3.material.io/components/segmented-buttons/overview)
recommends connected button groups for Expressive interfaces. The existing
segmented component remains a standard M3 control, and its gallery heading and
API guidance now make that distinction explicit. Connected-group fidelity is
still a separate open catalog requirement.

Standard segments now expose radio roles in single-choice groups and checkbox
roles in multi-choice groups, with a toggled state even when disabled. Group
semantics preserve caller labels/descriptions, publish horizontal orientation,
and retain child controls. The single-choice row is one Tab stop preferring
the selected enabled item. Directional focus selects an enabled option;
Tab entry, explicit focus, and a pending pointer press do not activate it.
Multi-choice directional focus preserves all checked values.

Selection shows an 18dp checkmark by default, replacing the optional leading
icon. Authors may supply `selected_icon` or disable this replacement with
`show_selected_icon(false)`. Live label typography updates its inherited
context. Equal-width rows distribute a bounded pixel budget, including border
overlap and remainder pixels, instead of extending all natural-width items
beyond the container. These defaults follow the primary
[segmented button implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SegmentedButton.kt).
Both native integration scenarios pass at densities 1, 1.25, 1.5, and 2 with
1x/4x sampling. Assertions inspect the published semantic tree, real focus
traversal and activation, checkmark pixels, live 20dp/28dp label line heights,
and exact row coverage across narrow, zero, and restored widths. All 111
documentation examples pass. Integrated GPU and browser results follow below.

The first integrated native run passes all 410 tests. Browser selection then
exposed another gap outside that fixture: a naturally sized row grew when a
checkmark appeared. The content now always reserves the icon and gap width,
centering an icon-free label within that reservation. The native scenarios now
also assert that natural group width survives selection/deselection and that
icon-free labels remain centered at all four densities. Final verification of
this correction follows below; icon/label transition animation remains open.

After icon-space reservation, the updated native scenarios pass all four
densities at both sample counts. All 309 ordinary component tests pass (101
opt-in GPU cases are excluded from that ordinary run), and all 111 documentation
examples pass. Release WASM and call-color checking also pass. The earlier
410-case GPU-inclusive run predates this final geometry correction and must
not be treated as a second full run of the final source.

The final browser build was checked at a 907x871 CSS viewport after the wider
1730px interaction pass. Pointer selection and directional selection move the
single-choice checkmark without changing the group width. Tab leaves that row;
Space selects News, moving right preserves News, and Enter additionally selects
Mail without changing the multi-choice group width. No console errors were
observed. A full viewport image is stored externally as
`wasm-segmented-stable-selection.png`. Browser semantics still expose only a
canvas, so these interactions do not establish browser screen-reader support.
Formatting and whitespace checks pass, the temporary copied test executable
has been removed, and README files remain unchanged. Reference pixel states,
transition motion, connected-group updates, and the broader performance work
remain open.

The connected-group follow-up now passes the actual size preset to each child,
including typography and inherited icon size. Connected rows distribute an
equal-width budget with 2dp gaps and retain the 48dp interaction minimum without
stretching 32dp/40dp visual containers. Selected buttons keep their identity and
width. The live token tables use inner corners of 8/8/8/16/20dp for XS through
XL, pressed corners of 4/4/4/12/16dp, and full rounding when selected, with
shared size-relative shape motion. These states follow the primary
[button group implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt).
The [official specs](https://m3.material.io/components/button-groups/specs)
contain an unresolved XS discrepancy: the measurements prose says 4dp at rest,
while the connected XS token table says 8dp. The implementation follows the
token table; this discrepancy remains part of the reference-parity audit.

`ButtonGroupsState` supports external selection and clearing. Click callbacks
observe the committed selection, programmatic changes do not invoke callbacks,
and switching to single selection retains the first selected item in visual
order. Removed items discard their selection. The gallery exposes all five
sizes and an external clear action. All nine group tests pass, including the
two native scenarios at densities 1 and 1.5 with 1x/4x sampling. They assert
actual visual and interaction heights, icon/label metrics, circular inner
corners, real pointer press/release transitions for all five sizes, stable
selection bounds/identity, live selection-mode updates, external clearing,
removed-item cleanup, and narrow/zero/restored widths. All 309 ordinary
component tests pass; 105 opt-in GPU cases are excluded from that ordinary run.
All 111 documentation examples pass. These tests do not establish normalized
reference-pixel parity or browser screen-reader support.
Mixed action/icon items and color variants, RTL, vertical groups, standard-group
press expansion and overflow, and normalized reference goldens remain open.

The final release WASM build and call-color check pass. Browser checks at
907x871 CSS pixels cover all five sizes, selection at natural width, and the
external clear action. XS/S preserve smaller visual containers while M/L/XL
update height and typography; connected selection preserves adjacent widths.
No browser console errors were observed. The final XL selected view is stored
externally as `wasm-connected-group-xl-selected.png`. All eight chip tests also
pass, including native pixel/input scenarios after sharing shape motion.

The selection follow-up publishes checkbox roles for toggle items, matching
the current [toggle button implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ToggleButton.kt).
The horizontal group keeps caller labels/descriptions and child controls.
Disabled indices and group disabling retain checked values while withholding
click/focus actions. Arrow traversal skips disabled items without activating
them; Tab continues between individual items. Required selection restores the
first enabled item after an empty update and prevents deselecting the final
choice in either selection mode. If no item is enabled, an empty group remains
empty until an enabled choice becomes available. These updates do not invoke
click callbacks.

Both expanded native scenarios pass at 1x/4x sampling and densities 1/1.5.
They verify published group/checkbox semantics, actual disabled pointer and
accessibility input, directional and Tab-order focus traversal, live required
selection in both modes, all-disabled/whole-group recovery, and callback counts.
All 309 ordinary tests, four button GPU regressions, and 111 documentation
examples also pass. Release WASM and call-color checking pass. Browser checks
at 907x871 CSS pixels verify arrow traversal skipping a disabled middle item,
Space activation, Tab exit followed by Enter activation of the next FAB,
required single/multiple selection, external reset, whole-group disabled input,
and selection restoration after re-enabling. The console contains no errors;
the final view is stored externally as
`wasm-connected-group-required-disabled.png`. These checks do not establish
browser assistive-technology support. A large batched scroll moved further
between its first screenshot and the next action, so general lazy-scroll
settling and layout anchoring remain part of the broader scroll audit.

The square preset follow-up adds `shape_style` using `ButtonShape`. Connected
square groups retain size-specific square outer corners while the pressed
item's inner corners contract; selected items become fully round. Standard
square groups use the button's size-specific corners and round their selected
items, while standard round groups square their selected items. Both styles
use one item rendering path so style changes preserve item identity and focus.

All nine group tests pass after this update. The native scenarios at densities
1/1.5 and 1x/4x sampling check square outer-corner pixels across five sizes,
selection shape inversion in both styles, a real square-end press/release,
and identity/selection/focus retention during shape and style changes. All 309
ordinary component tests pass. The 111 documentation tests pass after adding
the new public option. Final release WASM and call-color checking pass. Browser
checks at 907x871 CSS pixels cover all five connected square sizes, square-to-
round selection, round/square inversion in a standard group, selection
retention across style changes, and Right/Space activation after switching
styles. The console contains no errors. The final view is stored externally
as `wasm-connected-square-xl-selected.png`. These checks do not establish
complete reference goldens or singleton-state coverage.


The standard group now exchanges width with adjacent items during a press,
using the default 0.15 expansion ratio and limiting compression to their content
padding. Total width stays constant and the child content remains centered at
its natural width. `expanded_ratio` accepts finite values clamped to 0..1;
zero disables this motion. Short releases complete a visible expansion before
returning, while programmatic selection remains quiet. Disabling, changing
styles, removing items, and cancelling a pointer press stop or restore motion.
This follows the current [Material button group implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ButtonGroup.kt).

The obsolete selected-item expansion tests were replaced with five tests of
width exchange and deterministic press motion, plus two native GPU scenarios.
The native scenarios cover 1x/4x sampling at densities 1/1.5, actual held and
same-frame short presses, accessibility activation, cancellation, ratio limits,
style changes, disabling, empty/singleton recovery, content centering, and
pixel restoration after repeated clicks. They also observe the production item
build path to ensure held/resting frames stop rebuilding. All nine group tests
pass. Removing the legacy expansion also removed its unused animation variant;
the shared spring behavior and standalone easing function retain their tests.

Final verification passes 306 ordinary component tests, 111 documentation
examples, 15 Expressive component tests including four button GPU regressions,
and both stationary no-op ripple GPU tests. Release WASM and call-color checking
pass. Browser checks at 907x871 CSS pixels verify standard group selection,
width recovery, the expansion toggle, and Right/Space activation. The final
view is saved externally as `wasm-standard-group-width-recovery.png`; no console
errors were observed. These checks do not establish complete reference pixel
parity, overflow behavior, or browser assistive-technology support.


## Adaptive list row foundations

The current [list specs](https://m3.material.io/components/lists/specs) recommend
Expressive styling. The [current layout implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt)
and [list tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ListTokens.kt)
provide the layout/typography reference. The row now measures leading and
trailing slots before its text, allocates remaining width to main content,
and derives the three-line minimum from actual supporting-text wrapping.
Minimum outer heights are 56/72/88dp; padding is no longer subtracted twice.
Short rows center their content; tall rows top-align it. Authors may explicitly
choose top, center or bottom alignment. Larger media/text can grow the row,
while parent constraints still bound it.

Leading/trailing icons inherit 20dp sizing. Slot text inherits title-medium and
label-small typography respectively, and retained slot contexts update when
the theme changes. Headline text also updates its explicit style. Selected
metadata supplements caller semantics rather than replacing them, and disabled
rows withhold focus/click actions while retaining their selected value.
Resting and selected shapes use the theme's extra-small and large corners.
Segmented outer corners, shape transitions, additional selection modes,
interactive-slot target allocation, and custom main-content slots remain open.

All three list scenarios pass. The layout scenario checks token heights,
explicit and width-driven supporting-text wrapping, default icon placement,
and all explicit vertical alignments. Both opt-in native GPU scenarios run at
1x/4x sampling and densities 1/1.25/1.5/2, checking live typography/color pixels,
selection and stable identity, actual disabled pointer input, accessibility
activation, zero-width recovery and window resize. The documentation example,
release WASM and call-color checks pass. Browser checks at 907x871 CSS pixels
cover pointer and Tab/Space selection, disabled-state retention, large text,
narrow wrapping, all alignment choices and normal-size restoration; the console
contains no errors. Screenshots are stored externally as
`wasm-list-large-wrapped-selected.png` and `wasm-list-normal-restored.png`.
These checks do not establish complete official reference image parity or a
browser screen-reader bridge.

The complete ordinary component suite passes 308 tests (111 GPU/platform tests
remain opt-in). Nightly formatting and diff whitespace checks pass.


## Segmented list positions and state shapes

`list_item` now accepts `position`, a `ListItemShapes` palette and a controlled
`dragged` shape state. First/last/single rows use the larger outer corners;
interior corners retain the smaller resting shape. Authors compose rows with
`ListItemDefaults::SEGMENTED_GAP` (2dp). Defaults follow the theme's 4/12/16dp
corner sizes for rest/hover/selected-or-pressed-or-focused states. A fixed
`shape` overrides both grouping and state shapes. The public default-shape
helper now uses the same circular corners as the Material theme.

The [current shape defaults and priority](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItemDefaults.kt)
and [list interaction motion](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt)
are the reference. Shape transitions use shared FastSpatial motion with retained
velocity; frame callbacks stop at rest. Disabled updates snap to the applicable
resting/selected shape. The controlled drag option only selects the dragged
shape; gesture handling, drag elevation/color feedback and reordering remain
open, alongside color transitions, complete selection modes and main slots.


All five list scenarios pass after this follow-up. The two added opt-in GPU
scenarios run at 1x/4x sampling and densities 1/1.5. They compare corners to
independently authored 4/12/16dp reference surfaces, check 2dp gaps, first/middle/
last/single/standalone positions, real stationary hover and held/released
pointer input with a no-op application callback, selected/focused/controlled
drag states, interrupted motion, position changes, disabling during a press,
custom pressed palettes and fixed-shape precedence. Production component-entry
counts stay unchanged once hover, hold and resting motion settle. Restored
edges use a strict pixel tolerance; comparisons during feedback allow only the
additional coverage of partially covered ripple/state-layer edge pixels.

The 308 ordinary component tests, list documentation example and both stationary
no-op ripple regressions pass. Nightly formatting and diff whitespace checks
pass. Browser checks cover pointer and Tab/Space selection, standard/segmented
switching with retained selection, and disabling/re-enabling. Full color
transitions, drag interaction/elevation, selection-role variants, custom main
slots, target allocation and normalized official reference comparisons remain
open.


Final release WASM and call-color checks pass. Browser verification at 907x871
CSS pixels confirms pointer/Tab/Space selection and a standard-to-segmented
round trip without losing selection; no console errors were observed. The final
view is stored externally as `wasm-segmented-list-state-shapes.png`. The WASM
build emitted three warnings from the concurrent core work: unused `Px`,
`ComponentTree::metadatas`, and `is_instance_key_pending_build_invalidation`.
Those core files were not changed in this component follow-up. Complete official
reference goldens and the remaining list behaviors above are still unverified.


### List color motion and controlled drag palettes

List rows now share the existing Oklab color interpolation with checkbox motion.
The conversion algorithm and its independent numeric regression moved into the
shared animation module without changing checkbox timing or snapping behavior.
List containers and the five content roles use the default effects transition.
Color and shape motion advance through one frame receiver, including when an
explicit fixed shape prevents any shape animation.

Current Material defaults prioritize disabled colors, then controlled dragging,
then selection. Dragged containers use tertiary-container and their content uses
on-tertiary-container; all six dragged colors can be overridden. These follow the
current [list defaults](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItemDefaults.kt)
and [reorder list tokens](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ReorderListTokens.kt).
The example exposes a first-row drag-color preview. This is an externally
controlled visual state, not a claim of drag gestures, reordering, or elevation.

The new native rendering scenario fixes the row to a rectangle so shape motion
cannot supply the color clock. It observes intermediate selection pixels,
interrupted/reversed transitions, theme and explicit palette changes, drag over
selection, disabled over both states, live leading/trailing slot ink, restored
pixels, and no component builds after settling. It runs with 1x/4x MSAA at 1.0
and 1.5 density. Existing geometry tests use a uniform drag palette to keep their
shape comparisons independent of the new color transitions. Existing live-theme
endpoint assertions now wait for the effects transition to finish.


All six native list rendering scenarios pass, including the new color-only
motion tests and the retained shape/input/live-theme scenarios. All 308 ordinary
component tests pass (116 opt-in scenarios ignored), and both checkbox motion
GPU regressions pass after moving their shared color interpolation. Targeted
nightly formatting and diff whitespace checks pass. These checks establish the
covered state transitions; complete selection roles, rich main-content slots,
nested controls/target allocation, drag gesture/elevation/reorder behavior, and
normalized official reference goldens remain open.


Release WASM, Tessera call-color checking, and the list runnable documentation
example pass. Browser checks at 907x871 CSS pixels cover selection, controlled
drag over selection, disabled feedback and suppressed activation, re-enabling,
and return from drag preview with selection retained. No browser console errors
were observed. The drag-color view is stored externally as
`wasm-list-drag-color-preview.png`. The build emitted one existing core warning
for `ComponentTree::metadatas`; this component work did not change that method.


### List main, overline, and supporting render slots

The public list entrypoint now accepts `content`, `overline_content`, and
`supporting_content` render slots alongside string conveniences. A supplied slot
replaces its corresponding string. Main content receives body-large typography,
overline content label-small, and supporting content body-medium, each with its
own animated role color. Their wrappers add no leading/trailing spacing. The
existing layout measures them in the same constrained main column, preserving
side-slot allocation, automatic short/tall alignment and measured supporting
content classification. The design follows the current [Material list content
API](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt).

Automatic accessibility text does not reuse a string hidden by a custom slot;
callers can supply explicit row labels/descriptions, and slot semantics remain
in the subtree. This does not add selection-role variants or claim support for
nested interactive control allocation.

The layout regression composes all three public slots with independent fixed
sizes and deliberately longer fallback strings. It checks exact main-column
placement, constrained widths, side-slot bounds, minimum heights, and expansion
for taller supporting content. The existing four-density native scenario now
switches all three slots on, changes role typography and theme colors, selects
the row, then removes the custom slots and checks restored geometry before
continuing its real input/disabled/resize checks. The example includes custom
main, overline, and supporting content behind a plain/custom content control.


Final verification passes: 310 ordinary component tests (116 opt-in tests
ignored), all six list GPU scenarios, the updated runnable documentation example,
release WASM and call-color checking. The actual input scenario clicks inside
newly added custom-content height after reflow, asserts activation, then clears
focus before comparing unfocused theme colors. A dedicated Cargo test-profile
binary suffix avoids Windows executable locks from unrelated running regressions;
no production cfg or test behavior is changed.

Browser checks at 907x871 CSS pixels and a 1.5 backing-store ratio cover all three
slots, large typography with a 280dp column, plain/custom content round trips with
selection retained, Tab/Space activation, and pointer activation inside newly
added height. No console errors were observed. The final view is stored externally
as `wasm-list-custom-content-large-narrow.png`. An initial feedback-position
anomaly following first reflow was observed; later controlled round trips and
activation checks passed, so that initial transient remains unproven rather than
being declared fixed. The initial view also needs a deterministic browser
reproduction. The build retains the existing unused `ComponentTree::metadatas`
warning.

The wrapping fixture initially tagged a layout wrapper because `text` replaced
caller semantics and lost test tags. The text-semantics follow-up below now
preserves those tags, so the fixture queries the text node directly. Complete list selection
roles, nested interactive targets, drag/elevation/reordering, and normalized
reference goldens also remain open.


### Text semantic preservation and scrolled list reflow

Text now supplements caller semantics instead of replacing the whole record.
The default role is Label only when no role is provided. Explicit text label and
description parameters override modifier values; otherwise modifier labels and
descriptions remain, with displayed content used only as a missing-label fallback.
Testing keys, heading levels, tooltips, selection, disabled/hidden flags, and
unrelated metadata/action handling remain intact. Two actual native renderer
regressions verify caller Heading and ListBoxOption roles, explicit label updates,
restoration, automatic content labels, stable node identities, hidden flags, and
restored pixels under 1x/4x MSAA. The list wrapping regression now selects the
actual text node by its caller test tag.

Two scrolled list regressions at 1.0/1.5 density and 1x/4x MSAA place a row after
a measured 480dp item, scroll it to an independently known 80dp window origin,
then enlarge typography, reduce width and add custom content. They sample visible
container pixels inside newly added height, activate that area, restore the row,
scroll away, reject clicks at its old position, and scroll back to activate it
again. These protect rendered/semantic/pointer geometry together through actual
list, lazy-list and input entrypoints.

Final verification passes: 310 ordinary component tests (120 opt-in cases
ignored), all eight list GPU scenarios, both new text semantic tests and four
existing text drawing/placement/paint tests, the text documentation example,
release WASM and call-color checking. Targeted nightly formatting and whitespace
checks pass. The release build retains the unrelated unused
`ComponentTree::metadatas` warning.

A fresh browser reload followed by Material navigation, scrolling, custom content,
large text and narrow width reproduced the earlier setup at 907x871 CSS pixels
with a 1.5 backing-store ratio. Clicking both the original first-row region and
its expanded lower region correctly toggled selection; no console errors were
observed. Evidence is external as `wasm-list-first-reflow-hit-regression.png`.
The earlier one-off feedback-position anomaly did not recur. These successful
checks do not identify its root cause or establish a production fix for that
transient, so the broad first-reflow audit remains open. Complete selection modes,
nested interactive target allocation, drag/elevation/reordering and normalized
official reference goldens also remain open.


### Controlled list selection modes

`ListItemSelectionMode` adds Action, Single and Multiple behavior to the existing
entrypoint. `on_selection_change` emits controlled requests in Single/Multiple
modes and takes precedence over the fallback `on_click`; Action uses `on_click`.
Single requests `true`, including repeated activation. Multiple requests the
inverse of the latest supplied `selected` value. The owner updates state and
maintains single-selection exclusivity. A stable callback reads updated activation
state, so mode switches and external selection updates do not depend on closure
replacement. Read-only rows expose state without adding click/focus actions.

Single rows publish RadioButton role, selection and native checked state;
Multiple rows publish CheckBox role and checked state. Mode changes clear the
inapplicable checked/selection metadata while keeping unrelated caller metadata
and node identities. Disabled rows retain their state without activation. These
roles follow the current [Material list selection overloads](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt).
Indicators remain slot content; this API does not add nested indicator controls,
radio-group arrow traversal, or long-press selection gestures.

Two native scenarios run with 1x/4x MSAA at 1.0/1.5 density through actual pointer,
accessibility-action and focus paths. They verify repeated single requests,
independent multiple requests, external state changes, rejected state updates,
action callback fallback/precedence, live mode changes, stable identities,
read-only and disabled actions, retained metadata and stopped idle component
builds. The fixture owner enforces exclusivity, matching the public controlled
contract. The initial callback constructor was corrected to use the framework's
native stable handle directly, avoiding nested slot initialization.

All 310 ordinary component tests (122 opt-in tests ignored), ten list GPU
scenarios, and the updated controlled-selection documentation example pass.
Release WASM and call-color checking pass. Targeted nightly formatting and diff
whitespace checks pass. The release build retains the unrelated unused
`ComponentTree::metadatas` warning.

Browser checks at 907x871 CSS pixels verify multiple selection, transition to an
action with an opened-item result, owner-maintained exclusivity after entering
Single, repeated single activation, Tab/Space/Enter input, independent multiple
keyboard toggles, disabled activation suppression and state restoration. No
console errors were observed. The final view is stored externally as
`wasm-list-controlled-selection-modes.png`. Native role assertions do not establish
browser screen-reader publication because the WASM accessibility bridge remains
open. Complete selection indicators/group navigation, nested interactive targets,
long-press/drag/elevation/reordering and normalized reference goldens remain open.


### Stationary long actions and interaction cleanup

Surface and list rows now accept an optional `on_long_click`. A pending gesture
receives frame timestamps, so holding a stationary pointer fires once without
requiring movement or application state changes. The framework default is 500ms;
this is not an assertion about platform gesture preferences. A completed long
action suppresses the following short click. Shift+F10, the context-menu key and
native ShowContextMenu actions use the same callback.

Owned tap consumption is distinguished from presses claimed by nested children.
Movement beyond slop, scrolling, cancellation, focus loss, disabling, handler
removal and unmounting cancel pending work. Input ownership cleanup also releases
externally supplied interaction/ripple state, preventing retained pressed pixels
after a target disappears. Pending clocks stop after completion or cancellation.

Validation passes for all 20 foundation tests, all 310 ordinary component tests,
12 list GPU scenarios, both stationary no-op ripple scenarios and the tooltip
long-press/scroll regression. Native long-action cases cover 1x/4x MSAA at
1.0/1.5 density, actual pointer and accessibility dispatch, nested child priority,
release suppression, state restoration and external-controller cleanup. Release
WASM and call-color checking pass. Browser holding an Action row for 650ms enters
Multiple mode with one selected item; releasing preserves that selection. A
subsequent Space activation deselects it. This does not establish web screen-reader
publication or custom accessibility action labels.

Custom long-action labels, platform gesture preferences, full selection indicators
and group traversal, the broader nested-target audit, drag/elevation/reordering,
the earlier transient first-reflow investigation and official pixel goldens remain
open. Full Material parity is not established.


### Snackbar notification semantics and named controls

Snackbar messages publish a nonfocusable Status node with a Polite live region.
Both inline and separate-line actions expose their visible label as the Button
name. Close buttons expose Dismiss by default; `dismiss_action_label` allows a
localized or application-specific name. This follows the noninterrupting feedback
and named close-affordance behavior of the
[Material snackbar](https://m3.material.io/components/snackbar/overview) and
[official implementation](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt).

Two opt-in native GPU scenarios cover 1x/4x MSAA and 1.0/1.5 density using actual
components and published accessibility trees. They first focus an existing task,
mount the snackbar, and verify that neither layout steals focus. They verify the
message's role/politeness, invoke both controls through accessibility actions,
check callback results, and update the close-button name. Both scenarios, all
310 ordinary component tests and both snackbar documentation examples pass.
These checks protect native semantics; the current no-op WASM accessibility
adapter still prevents a claim of browser screen-reader support. Complete timing,
queue lifecycle, host transitions, normalized geometry and state goldens, and
platform timeout preference handling remain open.


Release WASM and call-color checking pass. In the fresh browser build, the
notification opens and both Undo and the close icon dismiss it. However, removal
of a focused snackbar control moves focus to the earlier Filled name field. The
first dismissal also moved the outer page by about 64 CSS pixels; the close-icon
cycle reproduced the text-field focus fallback without another displacement.
This is an open host focus-restoration regression, not covered by the standalone
component mount-focus tests above. Evidence is stored externally as
`wasm-snackbar-notification.png` and `wasm-snackbar-dismiss-focus-regression.png`.
The browser still exposes the canvas image rather than the native semantic tree.


### Snackbar host focus restoration and consecutive actions

`FocusManager::save_focus` now returns an optional `FocusRestorationTarget`.
Restoring checks the current tree and declines detached, disabled or unavailable
targets, unfocused windows and another target's focus capture. It does not retain
a target or leave a request waiting for later remounting.

The snackbar host saves the origin when its queue becomes visible and owns a
focus scope. When the final notification disappears while that scope still has
focus, restoration runs after layout. A newly disabled or removed origin is
therefore checked against the resulting tree. Focus already moved outside the
notification is preserved. Queued notifications share the original restoration
target until the queue empties. Opening a notification does not move focus.

The built-in snackbar renderer is now a private component with remembered current
notification data. Its stable action and dismiss callbacks resolve that data at
activation, preventing an earlier notification ID from rejecting later queued
actions. No callback replacement semantics are assumed.

Two new native host scenarios run at 1x/4x MSAA and 1.0/1.5 density. Actual host
components, pointer close clicks and native accessibility actions verify appearance,
Undo and close restoration, external navigation preservation, two consecutive
actions, deterministic 4-second timeout restoration, same-frame origin disabling
or removal, and no delayed focus theft after re-enabling/remounting. All four
snackbar GPU scenarios pass, as do 114 core tests, 310 ordinary component tests,
seven focus documentation examples and both snackbar documentation examples.
These checks do not establish complete host-unmount, simultaneous overlay,
platform timeout preferences, native assistive-technology behavior or browser
semantics publication. Full geometry, motion and reference goldens remain open.


The fresh release WASM build and call-color check pass. Browser reproduction at
907x871 CSS pixels verifies Undo and close-icon dismissal without the former
text-field focus fallback or 64px displacement. Pressing Enter immediately after
Undo reopens the notification through its restored trigger. Enqueuing two
notifications leaves the second visible after the first Undo; the next Undo
closes it. No error-level browser logs were observed. Evidence is external as
`wasm-snackbar-focus-before-dismiss.png` and `wasm-snackbar-focus-restored.png`.
This resolves the recorded focus fallback for these normal action/close paths;
full host disposal, simultaneous overlays and browser accessibility publication
remain separate requirements.

### Snackbar measured text and action geometry

The snackbar now measures message text independently of its action controls.
Single-line containers are 48dp; two 20dp lines with 14dp vertical message insets
produce 68dp. Natural wrapping grows the same layout. Controls no longer add
message padding to their own 48dp targets. Separate-line actions follow the
padded message with a 4dp bottom inset (100dp for a one-line message and standard
controls). Start padding is 16dp; inline action-only controls end 8dp from the
container edge, and close targets reach the trailing edge. Explicit content
padding controls message insets and horizontal action bounds.

Reference sources are the [website measurements](https://m3.material.io/components/snackbar/specs)
and the corrected OneRowSnackbar/NewLineButtonSnackbar paths in the
[official implementation](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt).
The correction branch is still behind a default-off
[upstream styling flag](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ComposeMaterial3Flags.kt).
These changes target the corrected layout; they do not assert pixel equivalence
to the legacy default branch. The website measurement diagram was inspected in
the browser and saved externally as `snackbar-official-measurements.png`.

Fractional placement preserves half-pixel text insets and action centering.
Vertical insets are combined before conversion to integer container dimensions,
avoiding separate truncation at 1.25 density. An explicit default message padding
produces the same pixels as the implicit default in the plain-message case.

Two new native scenarios cover eight explicit message/action/close combinations,
automatic wrapping, separate-line actions, constrained long action text, zero
width and exact pixel restoration at densities 1.0, 1.25, 1.5 and 2.0 with 1x/4x
MSAA. All six snackbar GPU scenarios, 310 ordinary component tests and both
snackbar documentation examples pass. This protects current geometry and recovery;
it does not establish independent reference raster goldens, legacy baseline
fidelity, all explicit inset/height constraints, dark colors or full motion
equivalence.

The final release WASM build and call-color checking pass. Browser verification
at the normal 907px width and a temporary 390x844px viewport checks inline
rendering, both control positions, close and Enter reopening, and restoration to
the default viewport. The compact sample remains within its host at both widths;
the browser check does not exercise the separate-line or long-action variants.
No error-level browser logs were observed. Evidence is external as
`wasm-snackbar-48dp.png`, `wasm-snackbar-390px.png` and
`wasm-snackbar-resize-restored.png`. The viewport override has been reset.

### Dialog and bottom-sheet exit focus ownership

Both providers keep the modal focus scope in the provider rather than in the
temporary content wrapper. They save the invoking target when the overlay
appears, defer keyboard input across visibility changes, and restore that target
after layout when the exit animation has removed focused modal content. Disabled
or detached targets are declined without a request waiting for later remounting.
Restoration is conditional on the modal scope still owning focus.

Scrims retain their pointer dismissal and blocking behavior but cannot acquire
focus, participate in Tab traversal or show ripple feedback. This prevents a
background dismissal from losing modal focus ownership before restoration. Frame
callbacks inspect animation state without mutating the controller merely to read
it.

The expected interaction follows the [modal dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
and [Material dialog behavior](https://material-web.dev/components/dialog/).
Two native GPU scenarios cover Dialog and Bottom sheet at 1.0/1.5 density with
1x/4x MSAA. Actual pointer input opens each provider, native queued traversal
cycles forward/reverse within its actions, action dismissal restores the origin,
and clicking over a background button dismisses without activating that button.
Disabling/removing an origin during exit prevents restoration and later focus
theft. Both scenarios, 310 ordinary component tests and all three affected dialog
and sheet documentation examples pass.

These checks cover settled opening and exit. Full modal semantic isolation,
simultaneous overlays, host disposal and interrupted
motion remain open. The existing forward-shifted reversal timestamps in these
controllers also need dedicated motion checks; this focus correction does not
establish Expressive animation fidelity. Basic/full-screen dialog variants,
sheet detents/dragging, normalized state goldens and browser screen-reader
publication remain separate requirements.

The browser then reproduced a text-only bottom sheet that could not handle
Escape because focus remained on its background trigger. The wrappers now request
their scope with a directly focusable, traversal-skipped fallback. Real controls
remain the preferred initial targets. Empty scopes own Tab/Shift+Tab even without
candidates; a handled empty wrapping policy no longer falls back to global
traversal. Navigation anchors the actual focused handle rather than resolving a
focused scope into a different descendant. Scope restoration excludes the scope
itself from its saved-descendant candidate, so later controls can receive focus.

The same two native scenarios now include empty content, forward/reverse Tab
retention, adding controls while open, entry into the first added control and
subsequent reopening. Both 1x/4x scenarios pass at both densities. The final core
run passes 114 tests, ordinary component tests pass all 310, and six snackbar,
four radio-button and two menu-focus GPU regressions also pass. Full semantic
modal isolation and background assistive-technology exclusion remain unverified;
the empty-focus correction does not establish those separate requirements.

The final release WASM build and call-color check pass. Browser checks verify
normal Dialog Tab/Shift+Tab, Escape, background dismissal without incrementing
the button below, and Enter reopening through the restored trigger. The text-only
Bottom sheet now retains forward/reverse Tab, closes with Escape and reopens with
Enter through its own trigger. Both behaviors were checked again on the final
build with no error-level browser logs. Evidence is external as
`wasm-dialog-focus-final.png`, `wasm-empty-sheet-escape-before.png` and
`wasm-empty-sheet-focus-restored.png`. All three affected component documentation
examples and seven core focus documentation examples pass. These are settled
motion checks, not interruption, simultaneous-modal or screen-reader validation.

### Interruptible overlay motion and modal side-sheet focus

Dialog, Bottom sheet and Side sheet no longer shift animation start timestamps
into the future when a visibility target reverses. A shared visibility motion
preserves the sampled value and velocity when retargeted. Sheet placement uses
the default Expressive spatial spring (stiffness 380, damping ratio 0.8), while
opacity uses the default effects spring (1600, 1.0). Dialog opacity uses the
effects spring. Sheet scrims have an independent effects track; their fade does
not inherit spatial overshoot. Placement preserves fractional coordinates.
Dialog scrim opacity now uses 0.32 rather than 0.5.

These values follow the current [Expressive motion tokens](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt),
the [motion scheme](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/MotionScheme.kt)
and [scrim tokens](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ScrimTokens.kt).
This resolves the timestamp-reversal defect recorded above; it does not establish
all component-specific entry/exit timing or reduced-motion behavior.

Two scalar regressions cover reversal values/velocities, repeated targets,
same-frame reversal and final settling. Two placement regressions cover
fractional sheet offsets and both side-sheet edges. Native rendering scenarios
use explicit 10ms frames to reverse all three provider types repeatedly, compare
successive bounds/opacity, require nontrivial entrance motion, and verify exact
pixel recovery after reopening and final exit. Both 1x/4x scenarios pass at
1.0/1.5 density. All 314 ordinary component tests pass. These counts include the
four scalar/placement cases rather than treating them as extra test runs.

Modal Side sheet now retains its focus scope in the provider and uses the same
exit restoration as Dialog and Bottom sheet. It requests a traversal-skipped
fallback when content contains no controls. Scrims cannot take focus or render a
ripple. The existing native modal-focus scenarios now exercise Side sheet too:
initial focus, forward/reverse wrapping, action and scrim dismissal, background
input isolation, empty content, adding controls, and disabled/removed origins.
Both scenarios pass at both densities and both sample counts. The example page
now includes an end-edge modal side sheet for browser interaction checks.

The [current side-sheet specs](https://m3.material.io/components/side-sheets/specs)
also exposed geometry differences. The width cap is now 400dp and ordinary
content padding is 24dp, replacing the previous 250dp/16dp values. Standard
Side sheet traversal no longer wraps within its own scope; reverse traversal can
return to the preceding main-content action. The native focus scenario checks
this and verifies that the background action still runs while a standard sheet
stays open. Closing a focused standard sheet restores its origin; closing after
focus has moved to main content preserves that new focus. Rendering scenarios
additionally cover standard sheets on both edges,
exact content placement under the width cap, and resizing below the cap.
Sheet drag settling, detached margins, modal semantic isolation and
reference-image comparisons remain open. Programmatic visibility reversal does
not prove drag interruption behavior.

The [side-sheet accessibility guidance](https://m3.material.io/components/side-sheets/accessibility)
requires a persistent close affordance and a Dialog accessibility role. The new
example includes a labeled close icon beside its headline and a separate Done
action. Provider-level Dialog semantics and accessible naming still need
implementation and verification; keyboard focus tests alone do not prove those
requirements.

The final release WASM build and call-color check pass. Browser interaction
verifies the new end-edge sheet's close icon, Tab movement to Done, Escape exit
and Enter reopening through the restored trigger. Dialog and the text-only
Bottom sheet also exit with Escape and retain a usable trigger for reopening.
The normal-width sheet shows the 400dp container with 24dp content insets. No
error-level browser logs were observed. External evidence is saved as
`wasm-side-sheet-400dp.png`, `wasm-dialog-material-motion.png` and
`wasm-bottom-sheet-material-motion.png`. The requested browser viewport override
did not change the rendered dimensions in this run and was reset; narrow
placement is supported here by native resize tests, not a narrow-browser claim.
All three Side sheet documentation examples pass; the two Dialog examples and
the Bottom sheet example also pass with the visibility-motion implementation.

### Dialog roles, names and background semantic isolation

All four overlay providers now accept `accessibility_label` to describe their
purpose. Dialog, modal Bottom sheet and modal Side sheet publish a Dialog role
with the modal flag; standard Side sheet publishes a nonmodal Dialog. The
underlying accessibility metadata and semantics modifier forward the modal flag
to AccessKit. The example supplies explicit names for all three modal providers.

The dialog's semantic boundary and focus scope are attached to its painted
surface rather than its full-window positioning wrapper. Actions remain
descendants of that boundary. While a modal surface is visible, a stable wrapper
around main content clears its descendant semantics and hides its own node.
Scrims are hidden from assistive technology without losing pointer dismissal.
Standard Side sheet retains the main content's semantics and interaction.
This follows the [modal dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)
and [side-sheet accessibility guidance](https://m3.material.io/components/side-sheets/accessibility).

Accessibility action dispatch now rejects targets beneath hidden, cleared or
unplaced ancestors. This prevents a previously published background node ID from
activating or focusing a control after the modal appears. A core regression
checks successful action dispatch, each exclusion independently, and successful
dispatch again after revealing the same target.

The two native focus scenarios now assert exactly one Dialog boundary with the
expected name/modal flag, ownership of both action buttons, a 248dp painted
dialog boundary for the fixed-width fixture, exclusion of background controls,
and rejection of stale background click/focus requests. Updating the accessible
name preserves the dialog's node ID. Both 1x/4x scenarios pass at 1.0/1.5 density,
including the previous restoration, empty-content and standard-sheet cases.
Both overlay rendering/motion scenarios also pass. The current verification
passes 115 core tests, 314 ordinary component tests, 20 foundation tests, all 111
component documentation examples and all three foundation documentation examples.

Release WASM and call-color checking pass. Browser inspection still exposes the
application as a canvas image rather than a published web Dialog tree; native
AccessKit snapshot assertions do not establish browser screen-reader support.
Stacked modal ownership, host removal, platform assistive-technology behavior and
optional description relationships remain open.

Browser testing also exposed an input-order gap when opening a provider and
immediately pressing Tab before its first layout: traversal could change the
background focus before the provider saved its origin. The example now defers
keyboard input immediately in its visibility-change callbacks, following the
same layout barrier used by focusable popup transitions. This is an example
integration correction, not an automatic guarantee for arbitrary application
state writes.

The final release WASM build and call-color check pass with that correction.
Actual browser sequences open each of the three modals, immediately send Tab
and Escape, observe its exit, and use Enter to reopen the same provider. The
reopened Dialog, Side sheet and empty-content Bottom sheet were visually
confirmed, with no error-level browser logs. External rendering evidence is
saved as `wasm-dialog-semantic-boundary.png`,
`wasm-side-sheet-semantic-boundary.png` and
`wasm-bottom-sheet-semantic-boundary.png`. Those images verify rendering and
interaction; the accessibility tree contracts above are established by native
snapshots, not by the images or the browser canvas node.

### Sheet drag settling, thresholds and context reset

Bottom and Side sheets now retain drag displacement in density-independent
units and settle it with the default Expressive spatial spring. A release keeps
its current displacement instead of resetting it in one frame. Grabbing or
reopening a moving sheet samples the existing displacement; sheet visibility
and drag tracks remain continuous. Direct drag displacement is limited by the measured
sheet extent. Settling requests frames while an open sheet still moves, without
keeping an already hidden provider alive for an invisible residual track.

Bottom sheet dismissal uses the 56dp positional and 125dp/s velocity thresholds
from [Sheet defaults](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SheetDefaults.kt).
The handle uses the shared bounded velocity tracker, and cancellation returns it
to the open anchor without requesting dismissal. Its 48dp region now has a
`Drag handle` accessibility label. Side sheet uses half its measured width as
the distance threshold, following the [edge delegate's halfway rule](https://raw.githubusercontent.com/material-components/material-components-android/master/lib/java/com/google/android/material/sidesheet/RightSheetDelegate.java),
and a 500dp/s significant horizontal velocity threshold. Nested scroll release
velocity is forwarded in the correct direction for both edges. The Android
delegate also has platform-specific prediction and direction rules that these
checks do not establish as equivalent.

Nested scroll callbacks are owned by the persistent providers and passed into
the transient sheet wrappers. The current side is read from a stable state
handle so retained callbacks follow an edge change. Scrims now disable state
layers as well as ripple, preventing cursor hover from tinting the dimmed scene
after a sheet returns to its initial position.

The expanded headless scenarios also exposed a separate reset defect: remembered
callback slots were reset while context values containing their handles survived
into the next renderer. Runtime slot reset now clears context slots too, advances
their generations, rebuilds the free list and releases values after the table
borrow ends. The existing cleanup regression now covers full runtime reset in
addition to individual owner removal and recycling. All 115 core tests pass.

Two new native GPU cases exercise the Bottom sheet and modal/standard Side
sheets on both edges, at 1.0/1.5 density and 1x/4x MSAA. They obtain the nested
scroll connection from the actual providers and observe rendered geometry,
pixels and close callbacks for small releases, threshold crossing, outward and
inward flings, rejected dismissal, close/reopen interruption, narrow resizing
and changing sides. Bottom-handle coverage additionally sends real pointer
samples for cancellation, a held slow release and a fast short fling. Pointer
assertions wait for the layout after input rather than reading the preceding
frame's cached geometry. Final settled pixels must match the initial image.
Both cases, both prior overlay-motion cases and both modal-focus/semantic cases
pass. All 314 ordinary component tests and 111 component documentation examples
also pass.

These checks cover the providers' current expanded/hidden anchors. Partial
detents, direct whole-container Side sheet dragging, detached margins, platform
gesture prediction, normalized reference frames and browser assistive-technology
publication remain open.

The final release WASM build and call-color check pass. In the browser, a small
handle drag returns to the open position; a 64 CSS-pixel downward drag dismisses
the sheet, and Enter reopens it through the restored trigger. The settled sheet
and dimmed scene recover without a hover-tinted scrim, and no error-level browser
logs were observed. External interaction evidence is saved as
`wasm-sheet-drag-settled.png`, `wasm-sheet-drag-dismissed.png` and
`wasm-sheet-drag-reopened.png`. Browser input is supplementary interaction evidence;
the explicit-frame native cases establish the detailed settling and velocity
contracts.

The final checks also pass all 90 core documentation examples and 20 foundation
unit tests. Formatting and whitespace checks are clean.

## Interactive specification page migration

The Material Components route now starts a categorized directory based on the
official Components page inspected in the browser. Its categories and individual
component entries open focused articles instead of rendering the entire gallery.
The old stacked gallery route has been removed. Sliders, Buttons, Checkbox, Switch,
Radio button, Chips, Icon buttons, Cards, Lists, Divider and Time pickers have
interactive specification articles. Date pickers has its modal calendar, range,
input and element-state sections; the Docked date picker component and its
section are still missing. The other articles
host their own existing live examples while their specification sections are
migrated.
Unsupported resource and platform chips are omitted. Other article tabs are
also omitted until their actual content is implemented; abbreviated substitute
pages are not used to fill their positions.

The directory follows the six official component categories and exposes 36
individual entries without adding a component sidebar. A native headless GPU
regression clicks every card, verifies the matching article is published, and
returns through the real back button. All 36 round trips pass. Cards without
a purpose-built live illustration omit that illustration rather than using a
generic control as a substitute.

The Sliders page includes live standard, centered and range controls, five sizes,
vertical orientation, light and dark schemes, disabled controls, token groups
and section links. This is an initial migration, not a claim of complete article
or pixel parity: detailed sections in other component articles, full token
tables and precise diagram annotations still require work.

Browser inspection exposed text ignoring the minimum dimensions offered by
weighted rows. Text measurement now clamps its result to both offered axes.
A real three-column layout regression protects the allocated widths and
positions. All 315 ordinary component tests and four GPU text drawing tests
pass with this correction.

Buttons now separates default and toggle variants, five sizes, round and square
shapes, five color styles, anatomy, light/dark colors, live input states and
measurements. Checkbox and Switch provide their own tokens, settings examples,
colors, states and measurements; Switch also exposes all three icon configurations.
These pages follow the current Expressive sections inspected in the official
Specs. Deprecated button baseline tokens and unsupported resource/platform chips
are omitted.

All four detailed articles share responsive headers and section links. The
native GPU directory regression now also exercises button selection, token-table
updates, checkbox indeterminate-state clearing and switch icon configurations.
It checks first-section and Measurements links below the fixed back button,
article titles at 390-pixel width and the one-column compact directory. All 36
routes and these interaction/layout assertions pass. Browser inspection caught
first-section headings being obscured after navigation; anchors now resolve the
article viewport origin at click time and include section padding, independent
of observer callback order. The native fixture includes the app-bar offset.
Tables publish semantic rows and cells, and article groups retain their interactive descendants.

The release WASM build and call-color check pass after the anchor correction.
Browser verification confirms the first section heading remains visible, the
live toggle button changes selection and shape, the checkbox settings example
updates, and choosing Outlined updates the token table. External screenshots
are saved as `wasm-spec-buttons-expressive.png`,
`wasm-spec-checkbox-settings.png` and `wasm-spec-components-directory.png`.
Detailed sections for the remaining component articles and complete reference
annotation/pixel comparisons are still outstanding.

Radio button now follows the four official sections with live exclusive choices,
light/dark schemes, enabled/disabled controls and 20/40/48dp measurements. Icon
buttons follows the current Expressive sections with default/toggle variants,
five sizes, two shapes, three widths, four color styles, live state feedback,
shape morphing and dimension tables. Deprecated baseline token tables are omitted.
Chips follows the official token section and four purpose-specific sections,
each with anatomy, light/dark color examples, states and measurements. Its 8dp
corners match the current Chips specs rather than importing another component's
Expressive shapes. Optional imagery and unsupported resource chips are omitted.

The existing native GPU directory scenario now also verifies icon action counts,
icon-toggle selection, filter/input-chip selection, chip token-table updates and
mutually exclusive radio selection. It checks section links and 390-pixel titles
for all seven detailed articles, including Chips' final Suggestion chip section.
All 36 route round trips and these assertions pass.

The final release WASM build and call-color check pass for these three pages.
Browser interaction confirms exclusive radio selection, icon toggle color/shape
changes and filter-chip checkmarks. Shared subheadings inherit their surface's
content color so dark examples remain legible; the final browser preview verifies
this correction and reports no error-level console entries. External evidence is
saved as `wasm-spec-icon-buttons-expressive.png`, `wasm-spec-chips-selection.png`,
`wasm-spec-radio-exclusive.png` and `wasm-spec-chips-dark-theme.png`.
Remaining articles still require detailed specification migration; these checks
do not establish complete diagram or pixel parity for the whole catalog.

Cards now follows the official token section, three container-style sections and
measurements. Each style has real enabled/disabled cards, light/dark schemes and
observable activation counts. Content uses 16dp horizontal padding; adjacent
measurement cards use 8dp spacing and the component's 12dp corners.

Lists now follows the recommended Expressive sections, including standard and
segmented styles, single-action, multi-action, single-select and multi-select
examples, controlled expansion, slots, color, states and measurements. The
inspected common token module confirms 56/72/88dp minimum heights, 20dp Expressive
icons, 12dp between slots, 10dp vertical padding and a 2dp segmented gap. Optional
media is omitted. Platform-specific swipe examples and the unrecommended baseline
article are not presented as implemented content.

Divider follows its three official sections with live full-width, 16dp inset and
16dp middle-inset lines, light/dark colors, supporting-text spacing and a dimension
table. The native GPU directory case now verifies the actual line bounds and
thickness, list heights, independent primary/secondary list actions, exclusive and
multiple selection, expansion and card activation/disabled behavior. All 36 route
round trips and the ten detailed articles' section-link/narrow-title checks pass.

The final release WASM build and call-color check pass. Browser inspection verifies
card activation counts, selected segmented-list colors/corners and all three
horizontal divider insets. No error-level browser entries were observed. Evidence
is stored outside the repository as `wasm-spec-cards-activation.png`,
`wasm-spec-lists-selection.png` and `wasm-spec-divider-insets.png`. The remaining
articles and comprehensive token/diagram/pixel comparisons are still outstanding.

Date pickers now separates modal calendar selection, modal date input and element
states. Single-date/range and light/dark controls share live examples; range months
remain virtualized in a bounded viewport. The page also opens real dialogs and
confirms current draft values. Docked date picker is absent from the library and
has no substitute illustration or section. A reusable anchored Docked component
and its specifications remain required work.

Time pickers now follows the six official sections. Live examples switch between
dial/input, twelve/twenty-four-hour formats and vertical/horizontal layouts, expose
light/dark colors, and open a real confirmation dialog. Narrow articles use 8dp
content insets to preserve picker space; container padding belongs inside the time
picker surface rather than becoming an external margin.

The expanded native directory regression verifies scoped calendar/date-range
selection, named range inputs and values, unavailable-date rejection, current-value
confirmation, real dialog closure, time-dial selection and input transitions. It
also checks horizontal-clock fallback after narrowing and full calendar width in
a 390-pixel article. These checks and all 36 route round trips pass.

Inspection corrected modal date headlines to On surface. The range-input scenario
also exposed unnamed editors: TextField now inherits a nonempty visible label when
no explicit accessible name is supplied. Existing range GPU assertions now check
both editor names and reading order. Date-picker predicate initialization resolves
callbacks before remember's slot-table borrow; a real component/layout regression
covers internal-controller initialization with date and year predicates.

Final native checks pass all 316 ordinary component tests, all six opt-in date
calendar/range GPU cases at 1x/4x MSAA and all 111 component documentation examples.
The example's full directory/interaction scenario also passes with the calendar
and range year windows set to 2020–2030, preserving usable year navigation rather
than limiting the primary examples to one year.

Browser inspection exposed date fields expanding to the bounded modal's available
height after switching a range calendar to input mode. Single-date and range
fields now retain their 56dp height. The native directory scenario switches the
actual range dialog at a 900-by-1200 viewport, checks compact input bounds and
dialog height, and verifies the cancel action remains inside the dialog. All 316
ordinary component cases, six date GPU cases and the full directory scenario pass
after this correction. Section-link checks reopen each article before navigation
so focus restoration from an earlier dialog cannot leave their links offscreen.

The final release WASM build and call-color check pass. Browser inspection confirms
the range dialog's compact input fields, visible Cancel/OK actions and successful
closure. Time-picker checks cover horizontal 24-hour selection, confirmation,
keyboard mode and dialog closure. No error-level browser entries were observed.
Evidence remains outside the repository as `wasm-spec-date-range-input-modal.png`
and `wasm-spec-time-horizontal-confirmed.png`. The directory follows component
categories and individual Specs articles without adding a component sidebar.

The FAB and Extended FAB directory routes now use dedicated Expressive Specs
articles, following the current [FAB sections](https://m3.material.io/components/floating-action-button/specs)
and [Extended FAB sections](https://m3.material.io/components/extended-fab/specs).
They present three live sizes, anatomy, six color mappings in light/dark schemes,
interactive state feedback and measurement tables. FAB also retains its separate
Configurations section. Baseline-only variants and unavailable resource chips
are omitted. The Extended FAB anatomy example supports reversible label collapse.

The full native directory scenario passes after adding actual button activation,
three-size height checks, square FAB bounds, and collapsed/restored Extended FAB
width assertions. First/last section navigation and 390-pixel title bounds now
cover fourteen detailed or partially migrated articles. The other twenty-two
routes still require detailed article migration, and Docked date picker remains
missing; these page checks do not establish complete visual equivalence.

The release WASM build and call-color check pass for both new articles. Browser
checks verify three-size rendering, action counts, Extended FAB label collapse
and restoration, light/dark color rows and section navigation. No error-level
browser entries were observed. Screenshot evidence is stored outside the
repository as `wasm-spec-fab-variants.png` and
`wasm-spec-extended-fab-variants.png`.

Button groups, Split buttons and FAB menu now have dedicated Specs articles with
the current official chapter order. Their examples exercise single/multiple and
required selection, disabled groups, five group/split sizes, independent split
actions and anchored alternatives, and real floating creation menus. FAB menu
examples start closed so offscreen examples do not request focus and scroll the
article away from its heading. Split menu geometry is measured relative to the
article host, independently of the popup's size.

The official Button groups measurement captions list a 4dp XS inner corner,
whereas the current connected-Xsmall token set specifies an 8dp resting corner
and a 4dp pressed corner. The implementation and token table follow that explicit
token set. Existing native pixel assertions retain both resting and pressed
curvature checks. Split button Xsmall hovered/focused/pressed inner corners were
corrected from 12dp to the explicit 8dp token; Small retains 12dp.

Native checks pass four button-group GPU scenarios, five split-button GPU
scenarios and all 111 component documentation examples. The directory scenario
covers the new selection policies, disabled input, split option callbacks, popup
size limits, floating menu selection/closure and first/last section navigation.
Seventeen routes now have detailed or partially migrated articles; nineteen
still use the earlier focused demonstrations. Group color-style composition,
FAB menu's web-specific baseline presentation, Docked date picker and complete
normalized reference comparisons remain required work.

Final native checks pass 317 ordinary component cases and the complete directory
scenario, including 390-pixel connected-group and extra-large split bounds.
Split color samples now wrap at their natural widths on a theme-colored surface
so light/dark labels retain contrast without allocating a whole row per action.

The final release WASM build and call-color check pass. Browser checks confirm
group multi-selection, FAB menu opening/selection/closure, anchored split menu
callbacks, and wrapped light/dark split styles without error-level console
entries. Evidence remains outside the repository as
`wasm-spec-button-groups-selection.png`, `wasm-spec-fab-menu-expanded.png`,
`wasm-spec-split-button-menu.png` and `wasm-spec-split-button-colors.png`.

Badges, Loading indicator and Progress indicators now have independent Specs
articles with their current official chapter order. Badge examples use actual
navigation destinations, dot/numeric/capped labels, unread-count updates and a
bar/rail switch. Loading examples compare default and contained appearances with
pause controls. Progress examples switch flat/wavy and determinate/indeterminate
forms, configure thickness, update completion and pause wave motion. Light/dark
examples use theme surfaces; supplementary loading/wave illustrations are paused
to keep their states inspectable.

Chapter structure was checked against the official [Badges](https://m3.material.io/components/badges/specs),
[Loading indicator](https://m3.material.io/components/loading-indicator/specs) and
[Progress indicators](https://m3.material.io/components/progress-indicators/specs)
Specs pages.

The complete native directory regression passes with all twenty detailed or
partially migrated articles. New assertions cover 6dp/16dp badge geometry,
48dp loading bounds, progress values and indeterminate semantics, thick circular
bounds, unread updates, destination selection and rail replacement. Sixteen
directory routes still require detailed article migration. Existing partial-page
and component alignment work, including Docked date picker, remains outstanding.

Browser interaction exposed a shared article-anchor regression at short final
sections: an unconstrained jump temporarily scrolled beyond the content extent,
then the first focused control moved the page back into range. Section navigation
now clamps its target against the measured content bottom and visible height.
The directory regression reproduced the old 568-pixel heading displacement and
now verifies that the first unread-count action succeeds without moving the
heading. The full directory scenario passes after this correction.

All eight affected opt-in GPU cases pass (two loading, four wavy progress, two
badge), as do the final release WASM build and call-color check. Browser checks
verify 75% wavy completion, indeterminate progress, contained/paused loading,
retained badge navigation selection, rail replacement and badge removal at zero.
The corrected final-section jump accepts its first control click without the
previous page displacement. No error-level browser entries were observed.
Evidence remains outside the repository as `wasm-spec-progress-configured.png`,
`wasm-spec-loading-contained.png` and `wasm-spec-badges-rail.png`.


Component detail navigation now shares selection with the example app bar. The
upper-left navigation button crossfades between the drawer and back icons over
200ms; the separate content-area back row has been removed. The button retains
one 48dp interactive target and changes its accessible label with the action.
A native opt-in GPU regression verifies the shared position, article origin at
64dp, return to the directory, and continued bidirectional fading without pointer
events. This targeted regression passes. The complete directory scenario also passes
once the test lets initial adaptive navigation layout finish before checking
orientation changes.


## Segmented buttons, text fields and tooltips (2026-10-22)

Three more routes now use the corresponding official Specs chapter hierarchy:
Segmented buttons, Text fields and Tooltips. This brings the migrated or partially
migrated route count to 26; ten routes still use their legacy detail presentation:
App bars, Carousel, Toolbars, Navigation drawer, Dialogs, Snackbar, Menus, Search,
Bottom sheets and Side sheets. Full figure layout and normalized pixel parity
remain outstanding, including on migrated routes.

The shared preview panel now places multiple examples in a column. Previously,
its stacking container overlaid sibling examples and their hit targets. The real
single-select and multi-select segmented button scenario exposed this failure;
its retained regression also checks that the two examples have separate bounds.

Segmented buttons support visual minimum heights of 40, 36, 32 and 28dp while
retaining a 48dp minimum interactive target. Both existing native GPU cases cover
all four densities at scales 1, 1.25, 1.5 and 2 with MSAA 1 and 4; the assertions
check actual painted height as well as accessibility geometry.

Filled text field floating labels now begin 8dp below the container top. Icon
slots remain vertically centered independently of floating labels, prefix and
suffix content. Leading glyphs are inset 12dp with a 16dp gap before the text;
trailing actions retain their full 48dp hit target. The catalog regression checks
these measured positions, editable and read-only input, clear actions, secure
text, error toggles and multiline sizing. The anatomy examples use compact empty
and populated fields with a numbered legend instead of full-width stacked fields.
Tooltips use the real provider, anchors and actions, covering manual visibility,
long messages, plain/rich variants and action dismissal.

Verification completed during this update: 319 ordinary component tests pass,
with 136 GPU cases explicitly opt-in; both segmented GPU cases pass. Native
catalog scenarios and final WASM/browser results are recorded after the final
figure-layout check below.


Final verification: both native catalog scenarios pass after the last icon-style
correction (108.28s), including all 36 routes. Formatting and whitespace checks
pass. The final release WASM build and call-color check pass. Browser checks
confirm attributed official headers, independent segmented selections, working
chapter anchors, side-by-side text field anatomy, standard clear icons without
filled button backgrounds, and rich-tooltip action dismissal. A historical fetch
error from before the local preview server was restored remains in the tab log;
no later error entries were observed during the final checks. Proof images are
stored outside the repository as `wasm-text-field-official-header.png`,
`wasm-text-field-anatomy.png` and `wasm-rich-tooltip-action.png`.


## App bars and Snackbar articles (2026-10-23)

App bars now has independent Variants, Configurations, Tokens & specs, Anatomy,
Color and Measurements sections. Its interactive preview uses real app bar
scrolling, selectable small/medium-flexible/large-flexible layouts, subtitle and
alignment controls, standard/filled actions, and a bundled Tessera logo. Legacy
baseline bars remain superseded by flexible variants for the Expressive target.
Search uses the existing search component; the additional official search app
bar arrangements still need figure-level alignment.

The public top app bar accepts an optional `title_content` render slot. It
replaces the text title in both compact and expanded layers while retaining the
subtitle, navigation, actions and alignment. Existing GPU scenarios now verify
custom content dimensions, global centering, subtitle placement and absence of
the replaced text for all three variants, with MSAA 1 and 4.

Snackbar follows the four official Specs chapters, with its unheaded anatomy
figure before the first chapter. The shared article supports an optional intro
slot for that ordering. Its five configurations use real snackbar components,
including a separate line for a long action. Actions and close affordances can
dismiss and restore the individual examples. The interactive queue uses the real
snackbar host, immutable requests and action/dismiss results; changing the next
request settings does not replace the active notification.

The migrated or partially migrated route count is now 28. Eight legacy routes
remain: Carousel, Toolbars, Navigation drawer, Dialogs, Menus, Search, Bottom
sheets and Side sheets. Official figure layout and pixel-level acceptance remain
outstanding, including on migrated routes. Docked date picker and expanded modal
navigation rail are also still missing.

Verification: all 319 ordinary component tests pass (136 GPU cases remain
opt-in). Both extended app bar GPU scenarios pass. The complete native directory
scenario passes all 36 routes and the new interactions (127.45s); the independent
back-button/crossfade scenario also passes (3.30s). The runnable top app bar
rustdoc example passes. Formatting and whitespace checks pass. Final WASM and
browser checks follow below.


Final release WASM and call-color checks pass. Browser validation of the final
artifact confirms the Snackbar intro ordering, chapter jump, five configuration
figures and action dismissal through the real queue. The App bars preview
switches between variants, replaces the title with its bundled logo, preserves
centering/subtitle placement, and collapses during real nested scrolling. No
error entries were recorded after the final reload. Proof images remain outside
the repository: `wasm-snackbar-intro.png`, `wasm-snackbar-configurations.png`,
`wasm-snackbar-action.png`, `wasm-app-bar-logo.png` and
`wasm-app-bar-collapsed.png`.


Dialogs now has real Overview, Specs, Guidelines and Accessibility pages in the
shared article shell. Its navigation uses horizontal icon/label pills, a rounded
track, measured content widths and an independently selected page. Inactive
article content is unmounted. Other articles still need their additional page
data; their single Specs entry must not be counted as completed four-page
navigation. There are now 29 migrated or partially migrated routes and seven
legacy routes.

The basic dialog anatomy and measurement figures render actual dialog content,
actions and dividers. Measured component bounds drive numbered callouts, dotted
leaders, icon/action outlines, padding overlays and dimension labels. The current
Material theme is retained by request; legacy illustration colors are not a
matching target. Portrait assets, full-screen figure annotations, complete
Guidelines illustrations and normalized pixel acceptance remain outstanding.

The dialog provider supports a full-screen presentation with square corners,
zero outer padding and a 560dp maximum width. Full-screen dialog content provides
a fixed header, optional confirm action and a scrolling body. Basic dialog width
limits now include the provider's default 24dp padding. Native GPU scenarios
verify modal geometry, focus order, action omission, confirmation, dismissal and
focus restoration at two densities with MSAA 1 and 4. The new article scenario
verifies four-tab selection, inactive-content removal, actual annotation pixels
and opening a real dialog. All three native catalog scenarios pass (168.39s),
as do all 319 ordinary component tests and three runnable dialog rustdoc examples.

The article intro now stacks its slot children vertically. The dialog scenario
also checks that its launch controls remain separate and precede the anatomy
figure, guarding against an overlay container hiding those controls.

Final release WASM and call-color checks pass. Browser verification confirms
Overview content stacking, four-page selection, basic dialog dismissal, the
full-screen form, anatomy callouts and measurement overlays with the current
theme. No browser error entries were recorded. Proof images remain outside the
repository: `wasm-dialog-overview-tabs.png`, `wasm-dialog-anatomy.png` and
`wasm-dialog-measurements.png`. Pixel-level equivalence is still incomplete.


Checkbox, Radio button and Switch now have independent Overview, Specs,
Guidelines and Accessibility content. Their official chapter structure and
selection rules were inspected on 2026-10-23. Platform/resource availability is
omitted rather than inferred from another implementation. Four articles now
provide all four page choices; the other articles still require this migration.
The overall migrated or partially migrated route count remains 29 of 36.

The checkbox Guidelines page includes real parent/child selection: a mixed
parent, independent children, select-all and clear-all. Labels for the three
selection controls now operate their associated control and request its actual
focus target. Label targets retain a 48dp minimum size without adding a second
keyboard stop or duplicate accessible control. The example switch keeps its
component identity when its optional selected icon appears or disappears.

Checkbox, radio button and switch expose optional focus requesters bound to
their interactive targets. Existing native boolean-control scenarios verify
these handles against actual semantic nodes and assert that focus does not
activate or select controls. Both MSAA cases pass. All 319 ordinary component
tests and all 112 runnable component rustdoc examples pass. Four native catalog
scenarios pass; the extended selection article case also covers repeated label
toggles, radio exclusivity, actual focus, parent/child icon activation, tab
selection and removal of inactive chapters. Release WASM and call-color checks
pass.

The official Checkbox accessibility page currently contains a chip keyboard
table, including deletion actions. The checkbox article uses checkbox traversal
and activation behavior instead. Figure-by-figure visual matching, detailed
anatomy overlays and responsive reference comparisons are still incomplete.

Browser checks of the final WASM confirm label selection and subsequent Space
activation for checkboxes, mixed-parent select-all, radio-label selection followed
by arrow-key navigation within its group, and switch-label activation followed
by Space across icon removal/restoration. The Switch Accessibility page shows
its separate chapter list. No browser errors were recorded. Proof remains
outside the repository: `wasm-checkbox-guidelines-parent.png`,
`wasm-radio-overview-selection.png`, `wasm-switch-overview-keyboard.png` and
`wasm-switch-accessibility-tabs.png`.

The 2026-10-23 parallel article pass removes the remaining seven legacy demo
routes: Carousel, Toolbars, Navigation drawer, Menus, Search, Bottom sheets and
Side sheets. All 36 directory entries now open independent articles. Nineteen
articles provide distinct Overview, Specs, Guidelines and Accessibility pages;
17 still need alternate-page content. The unused legacy demonstration body has
been removed from the Material route module.

The eight action-family articles also provide separate pages. Native checks
cover actual actions, toggle icon pixels, tooltip behavior and the recommendation
from deprecated segmented buttons to button groups. Segmented accessibility
subheadings and filtered search items use stable per-item component boundaries.
Search results publish their real items and live counts, and the search field
uses its hint as its accessible name. Search supports a truly omitted divider
and independent result-surface colors. Floating toolbars expose optional shadow
elevation for paired FAB figures without changing the ordinary default.

Verification covers six native catalog scenarios, contextual-menu actions,
actual search input/filter/select/clear, responsive six-marker search anatomy,
all six GPU-rendered carousel viewports, and toolbar geometry/touch containment.
All 320 ordinary component tests and 112 runnable component rustdoc examples
pass. The call-color check passes. Screenshot proof remains outside the repository.

Pixel equivalence remains unfinished. Known gaps include standalone reference
photographs, proportional scaling of native controls inside illustrative device
frames, detailed guideline and motion figures, several paired color diagrams,
animated search margins and arrow-key traversal into search results. Native
geometry and interaction assertions do not establish visual equivalence. The
remaining navigation, tabs, app-bar and tooltip differences are recorded in
external next-pass audits; the current theme remains intentional.

The release WASM build and call-color validation pass. Browser checks confirm
independent native menu selection and search typing, filtering, selection and
restoration of all four suggestions. Batched IME commits preserve every character
in the native regression. Browser automation bulk text entry differs from normal
per-key input; it did not reproduce an ordinary keyboard-input failure. Search
navigation/clear buttons use transparent standard treatment, and phone content
is clipped inside its rounded bezel. Final screenshot checks follow these fixes.
Final release WASM and call-color checks pass after the Search visual corrections.
The browser confirms transparent back/clear icons and intact rounded phone
bezels around the six live anatomy callouts, with no new error entries. Proof:
`wasm-parallel-search-anatomy.png`, `wasm-parallel-menus-selection.png` and
`wasm-parallel-search-filter.png`, stored outside the repository. The next-pass
navigation bar, rail and tabs audit now also has live official DOM evidence for
all twelve pages, including actual heading levels. Lazy image placeholders
initially reported zero intrinsic dimensions; subsequent evidence uses the
rendered image source and declared aspect ratio, with full-size originals
inspected separately. Placeholder dimensions are not visual acceptance evidence.


## Parallel article expansion (2026-10-23)

All 36 articles now define separate Overview, Specs, Guidelines and Accessibility
pages. This pass completes the remaining 17 article definitions, while preserving
existing interactive Specs examples. Unsupported platform/resource information
remains omitted. The official site is the chapter and illustration reference;
older illustration palettes do not override the current default theme.

New figures compose native controls with geometry annotations. They include
navigation badges and large labels, combined app-bar anatomy, paired search
arrangements, tooltip grids, removable chips, card/list/divider anatomy, docked
calendars, clock/input time pickers, field anatomy, badges, snackbars, loading,
progress and sliders. This is implementation progress, not pixel acceptance.

The core changes include separate 48dp input-chip removal targets, rail badges
that follow the expanded label, navigation labels that grow with typography,
12dp rich-tooltip action spacing, and a public time-picker selection state.
The docked calendar provides independent month/year menus, adjacent-month dates,
selectable-date restrictions and the existing calendar keyboard navigation.
Its example keeps the date-field anchor fixed, validates on submission, commits
with OK and restores the last committed date on Cancel.

Verification so far: 323 ordinary component tests and 113 runnable component
rustdoc examples pass. Native GPU checks pass for chip removal, rail indicator
pixels and activation, docked-calendar geometry/navigation, rich-tooltip spacing,
and both time-picker MSAA cases. Example integration and release-WASM/browser
checks are tracked separately below.

Full visual equivalence remains unfinished. Native figures currently lack
subtree scaling to match the site's responsive illustration scaling. Several
contextual/device illustrations, paired color examples, configuration variants
and fine-grained date annotations remain incomplete. Chip/list group arrow-key
navigation, chip keyboard removal and snackbar Escape/shortcut handling also
remain gaps; example keyboard tables do not claim those behaviors. An official
date illustration contains inconsistent year/weekday data; the native calendar
keeps correct Gregorian arithmetic. Modal date figures still use the component's
actual current-day indication rather than falsifying a fixed historical today.


Example integration: the full 17-scenario run passed 15 cases, including every
one of the 36 directory routes and its four available tabs. The two failures
identified missing explicit accessible button names. After those fixes, the
docked field/commit/cancel case and the collection action case both pass. The
form scenario also passes again after matching the official Snackbar
Accessibility page's 24px/32px H3 headings, H4 strategy headings and omitted
contents list. Together these runs validate all 17 scenarios; the original full
run itself was not green. Formatting and whitespace checks pass. Release call
color validation passes; final WASM/browser verification follows the button and
heading corrections.


Browser inspection caught an additional layout regression that semantic-only
checks missed: the docked example's date field expanded to its figure host's
height, allowing the overlaid calendar to cover the value and trailing icon.
The field now has an explicit 56dp visual height. Its GPU regression observes
that actual outer geometry, checks painted date text, and verifies stable
anchoring, selection, confirmation and cancellation. The extended scenario
passes. The fixed figure is included in the final WASM build below.


The docked anatomy overlay now projects the actual selected date and displayed
month, rather than retaining the initial date's cell. Current-day and ordinary
day markers stay outside the calendar grid. The same GPU scenario selects a
date on a different row and compares its live cell center with the annotation,
then verifies confirmation and cancellation. This extended case passes.


A parallel padding audit also moved content insets inside the painted surfaces
of the time-picker figure and the card/chip/list/divider/slider color examples.
The contact-form figure now paints its background outside its content padding.
The added time-figure GPU regression checks the actual 328dp surface, 24dp text
inset and painted padding pixels; it passes. These changes do not alter the
source palette exception or establish full responsive figure equivalence.

The final combined native example run passes all 18 GPU scenarios, including
all 36 routes and their four tabs, the docked field's live annotation and
commit/cancel behavior, and the time figure's painted padding. The final release
WASM build and semantic call-color validation pass. Browser checks of that
artifact confirm date selection, cancellation, and the corrected time surface;
no browser errors were captured during that check.

Browser performance is not accepted: a fresh date-anatomy interaction after all
builds finished still reported approximately 67–68ms in the renderer's CPU
"Draw commands" phase and 88ms total frames. That phase includes layout and
other component-tree work, so these warnings do not establish GPU execution
cost or identify a specific bottleneck. Further investigation is required.

The subsequent bounded figure pass corrects slider measurement-span end caps,
6dp track gaps and the 4dp stop annotation, and makes the handle callout follow
its current value and interaction width. The badge Overview/Usage figure now
uses three horizontal native destinations (dot, 1, 999+) with labels and numbered
captions. The existing form-article GPU scenario now asserts three aligned
labels at 136dp spacing; the extended scenario passes.

Read-only performance inspection confirms that Date Specs mounts 18 picker
instances, including offscreen illustrations. Absolute-window geometry observers
also invalidate annotation state during whole-figure scrolling. These mechanisms
are potential contributors, not a measured attribution of the slow CPU phase;
SVG contents are cached, and no persistent idle redraw loop was observed.

The final bounded follow-up also passes release WASM and semantic call-color
validation. Browser proof confirms the three badge destinations and a slider
value change from 40 to 60 with its measurement leader following the handle.
No console errors were captured; slow-frame warnings remain. Responsive figure
scaling and performance therefore remain open acceptance items. Screenshots are
stored outside the repository in the dated specification-QA directory.

## Native configuration and annotation follow-up (2026-10-23)

Docked calendars now accept an optional shared `DockedDatePickerView` state for
calendar, month and year presentation. The menus use a 64dp navigation header,
full-width 48dp choices, selection checkmarks and a 28dp bottom inset. Their
360x428dp surface omits calendar arrows and action buttons; the calendar remains
360x460dp with actions. Its 16dp circular corners are checked with actual pixels
outside the antialiasing edge. The existing calendar GPU scenario also covers
external view changes, live menu transitions and unchanged date commitment.

The date article uses real independent views in the reference's two-row
configuration arrangement and adds the month-menu anatomy and measurements.
The slider sizes illustration now places XS/S/M above L/XL, with native controls,
measured bounds, numbered captions and dimension/corner overlays. The new slider
regression exposed a 2dp XS/S placement error caused by assuming a 48dp measured
height; their actual 44dp height is now used to preserve the intended centers.
Snackbar anatomy now includes four leaders, endpoint and action/icon outlines;
its native regression checks the callouts against actual controls and activation.

The first combined run passed 19 of 20 example scenarios; the new slider test
caught that placement discrepancy. Final verification after its correction is
recorded below. Component verification passes 323 ordinary tests, 113 runnable
rustdoc examples and the extended native docked-calendar scenario.

Remaining acceptance work includes whole-subtree responsive figure scaling,
initial modal year-selection figures, complete modal/input configuration
canvases, and item-level annotation observations after arbitrary menu scrolling.
The added source figures increase Date Specs from 18 to 22 mounted picker
instances; older slow-frame measurements are not a valid comparison for this
changed workload. Performance remains unaccepted.

The corrected combined example run passes all 20 GPU scenarios. The extended
date-anatomy regression also verifies that markers 2 and 3 are both published
and painted. Their construction now uses a single component call per iteration
instead of separate conditional paths.

## Text field regression investigation (2026-10-23)

Browser reproduction identified a narrow single-line placeholder wrapping and
increasing the field height, and an external Clear control modifying a read-only
example. Native regression then exposed a separate bounded-parent problem:
internal fill-height modifiers expanded fields to the available parent height.
The field now probes its natural body height and performs final measurement
within the caller's constraints. Single-line decorations remain on one line;
explicit heights and multiline wrapping remain supported.

Decoration clicks now use the measured editor origin rather than a padding
estimate. Obfuscated fields publish the password role without a plaintext
accessibility value; revealing and hiding the value retains the editor content.
Three native field scenarios and 323 ordinary component tests pass.

The example disables Clear while disabled or read-only, removes empty-value
clear icons, exposes validation messages as alerts, and reports counter overflow
without discarding input. Specs anatomy now has native numbered callouts and
internal outlines; leaders stop at caption edges. The combined native example
run passes all 23 scenarios, including the new text-field interactions and
caption pixel assertions. Release WASM and semantic call-color checking pass
for that revision.

Subsequent browser inspection of Outlined examples exposed two additional
issues. Their nested dark theme changed text colors while the example panel
retained a light background. Separately, the surface's foreground border was
drawn after the label's painted notch, crossing the floating label; its fallback
theme-colored rectangle also disagreed with arbitrary parent backgrounds.
Color panels now paint their nested theme's surface. Outlined fields paint the
container once and draw the native outline through precise shape clips, leaving
a real gap at the floating label without covering the parent background. The
new GPU regression compares fractional enabled/disabled outlines against an
unsplit native surface in both themes; all four native field scenarios pass.
All four example field scenarios also pass, including actual background pixels,
supporting-text contrast and numbered-caption interiors.

Intrinsic editor sizing now uses a separate bounded size cache keyed by text
revision and constraints. A miss shapes an isolated zero-scroll display buffer;
it does not replace the live editor's viewport, draw snapshot, cursor or IME
geometry. Final combined verification is recorded below.

Final native verification passes 324 ordinary component tests, all four
field GPU regressions, all 24 example GPU scenarios and 113 runnable rustdoc
examples. The actual intrinsic-probe regression also confirms width, content,
style and display-transform invalidation and that settled frames stop rebuilding.
Formatting and whitespace checks pass. A second read-only review found no
additional concrete correctness regressions. Final release WASM and semantic
call-color checking pass. Browser verification of that artifact confirms normal
and focused Outlined notches in both themes, readable dark panels, stable narrow
empty fields and disabled read-only clearing. No browser errors were captured.
Temporary focus-event instrumentation was removed and the clean preview reloaded.
These checks do not establish overall responsive pixel parity or an uncontended
scrolling-performance improvement.

An intermittent browser scroll clamp was observed after focusing a field. A
fresh reload then allowed two complete scroll-away checks while retaining focus;
typing while the field was offscreen and returning confirmed that focus remained
on the same editor. No production focus change was made without a stable
reproduction. The separate native manual-wheel scenario exercises actual pointer
input and later idle frames and passes. It also observes the actual field's
window bounds beyond the viewport and uses explicit frame timestamps. This
protects the normal manual-scroll contract while the intermittent browser
observation remains unclassified.

### Native field measurement and article navigation follow-up

Filled and Outlined measurement figures now render real 240dp fields with
native icons, supporting text and counters. Their overlays follow actual text
and subtree focus, including clearing a populated field and entering text into
an initially empty field. The shared initialization no longer reads another
remembered state while the slot table is borrowed. The existing constrained
editing scenario now exercises both field variants, including the focused
empty 56dp container and read-only clearing restrictions.

Articles now include Previous and Up next links between available chapters and
adjacent component overviews. Each link has an independent control identity;
the navigation group preserves its actionable descendants. Route context is
updated before mounting the destination article. Pill tabs use roving keyboard
focus with manual activation, matching the inspected official behavior.

Modal date pickers expose shared calendar/year view state, and year buttons
publish their accessible names. Modal range input uses its 120dp header while
the calendar keeps its compact header. The six affected native picker scenarios
and 324 ordinary component tests pass. These changes do not establish complete
responsive pixel parity across all articles.

The combined native example run passes all 27 scenarios, and all 113 runnable
component documentation examples pass. Browser follow-up confirms manual tab
activation after pointer focus. It also exposed an article scroll jump to an
earlier chapter tab before the Clear callback ran, including on blank-area
clicks. DOM event capture did not observe a focus/blur transition. Clear actions
now request their own editor's focus before removing the button; this protects
button removal but does not by itself resolve that earlier pointer observation.
The five field regressions pass with an added real-pointer sequence in both
variants that checks the focused editor, scroll offset and window rectangle.
The four native field GPU scenarios also pass, including fractional outline
pixels and transparent label notches.

### Parallel figure reconstruction and performance verification

Independent visual audits now cover all four Text fields, Date pickers, Sliders
and Menus chapters. Configuration, color, state and accessibility figures are
being reconstructed with native controls and annotations. Reference dimensions
and role legends remain separate from interaction tests; compilation alone does
not establish pixel parity. Responsive scaling, instructional scenes and several
chapter-specific diagrams remain incomplete.

The normalized date-annotation boundary skips its heavy inner SVG composition
during pure article translation. A complete native profiling capture still
reports approximately 19,585 geometry updates and 19,690 scene updates per
scrolling frame, with 15 cached measurements. Actual GPU timestamp samples are
approximately 0.2ms; these captures identify CPU maintenance as the main cost.
Concurrent compilation makes the CPU timings unsuitable as stable benchmarks.
Forced headless frames at rest reuse their caches and do not prove desktop or
browser idle scheduling.

The core suite passes 141 tests with two GPU tests ignored. Repeated conditional
branches inside loops previously shared component identities, causing remembered
state collisions and stale layout children. Local occurrence salts distinguish
these branches while explicit keys preserve reordered state. The regression
fails before the fix and passes afterward; the exact native date-selection and
chapter-navigation reproduction now completes through frame 452 without a panic
or missing layout metadata. Temporary structural diagnostics have been removed.

Native nested-scroll focus reveal now consumes placement retries, and finite
focus-handle cleanup keeps the next frame scheduled. Real nested-scroll and
manual-wheel regressions pass. The updated WASM passes semantic color checking
and loads in the browser. That browser build retains its position during steady
manual scrolling, but clicking a chapter tab immediately before scrolling exposes
a focus-reveal ordering issue. A native regression using the actual article tabs
reproduces the issue and passes after stale reveal requests are cancelled by newer
manual scrolling. Deferred focus ownership and later keyboard reveal remain
covered. A fresh WASM build passes semantic checking and verifies the same
immediate-click-and-wheel sequence in the browser: the article retains its
scrolled position at rest, and subsequent Right/Enter keys reveal and activate
the next chapter tab. Temporary focus tracing has been removed from the sources.
Overall pixel and performance parity remains unverified.

Native GPU regressions also cover requested tooltip widths surviving internal
constraints and transparent outline colors remaining independent of invisible
fill RGB, with and without a ripple. Live outside focus rings now use a separate
focus-visible signal; pointer presses hide the ring without clearing ownership,
and accessibility focus restores it. Core and native button/icon regressions
pass; the updated focus-visible behavior still requires browser verification.
Native date and year choices now share the same focus-outline geometry. Their
visibility is read during composition so focus changes invalidate the real draw
policy. A native GPU regression verifies that assistive-technology focus changes
pixels outside the visual body without selecting a date, and subsequent actions
still select a date or navigate to a year. The six ordinary date regressions and
shared focus-geometry regression also pass.

A fresh, fully drained native capture uses the normal optimized development
build at 1280 × 800, density 1, and four samples, without concurrent compilation.
Actual range selection updates the headline to August 12–19, and an inner
calendar wheel reveals September without moving the article. Across 18 wheel
frames, median CPU frame time is 17.23ms and GPU time is 0.099ms. Eight settled
frames perform no measurement stores, paint records, geometry, scene, or
accessibility updates. Selection transitions still reach 94–146ms on the CPU
and update tens of thousands of retained consumer nodes; this remains an open
performance issue. These measurements precede the uniform-coordinate rollout.

Before the uniform-coordinate rollout, all seven native Text fields GPU scenarios
passed, including retained translation, idle painting, reflow, and annotations.
Cached Boxed placement avoids needless measurement during translation. The two
native Sheets context scenarios also passed with real playback, track selection,
filter changes, and reopening. These
results do not establish complete article or browser parity. Native uniform
subtree scaling is being verified separately; its coordinate contract is
documented in [Local layout coordinates](../../../docs/LOCAL_COORDINATES.md).

Native fractional-scale GPU coverage now passes at both one and four samples.
An actual 240 × 56 outlined field occupies 150 × 35 window pixels at scale
0.625 and fractional origin (10.25, 20.5). Its icon, editor inset, accessible
bounds, transparent label notch, IME editing, focus, and ancestor shape clipping
are exercised through the real control. A native 256-pixel clock dial occupies
160 pixels; its 48-pixel digit target occupies 30 pixels and a physical click
changes the selected hour. Four additional native GPU scenarios preserve
fractional tooltip and submenu placement through resize, canonical touch slop
through ancestor translation, and scaled editor multi-click selection. A real
wheel-to-bottom regression reproduces a stale offset after responsive resizing:
the old valid offset of -628 must become -516 before any later input. Placement
now reconciles those extents, and the regression passes together with 328 ordinary
component tests. Complete scaled article and browser comparisons remain separate,
pending verification.

Retained replay no longer republishes every unchanged descendant to pointer,
focus, accessibility, and scene consumers. The production replay regression
failed before the fix: the same leaf update published 23 pointer nodes with a
small retained sibling and 203 with a larger sibling. It now passes alongside
actual retained reorder and semantic/focus ancestry scenarios. All 149 ordinary
core tests pass, as does the native headless protocol regression for fractional
input and scaled bounds. A subsequent normal optimized capture at the same
1280 × 800, density-1, four-sample configuration preserves range selection,
inner-calendar scrolling, article position, and zero-work settled frames. The
three selection CPU peaks fall from 94–146ms to 36–44ms; date-selection pointer
and accessibility publications fall from 27,540 to 763, and scene publications
from 27,609 to 832. Median wheel CPU time falls from 17.23ms to 14.92ms.

GPU results require a separate conclusion: a same-environment rerun of the
unchanged baseline binary also raises quiet GPU time from 0.097ms to 0.335ms and
wheel GPU time from 0.099ms to 4.06ms. The updated binary records 0.349ms and
6.11ms respectively. These captures do not isolate a rendering regression or
establish GPU parity. Existing identity-reconciliation walks and remaining
interaction latency still need investigation.


The first canonical article wave has 22 distinct passing native GPU scenarios,
including Date, Sliders, Dialogs, Menus, article navigation, and the complete
component-directory traversal. The tests exercise real physical input,
fractional annotation geometry, resizing, and independent article chapters.
A subsequent draw-only correction changes the article chapter tabs to circular
capsule ends; its three focused native scenarios also pass. The successful
WebAssembly build and initial browser comparison precede that capsule correction.
Neither result establishes pixel parity for all articles. The next table, sheet,
and date instructional diagrams are undergoing integration and separate checks.

The integrated articles now reserve a real 15dp scrollbar track from the first
frame. Native regressions exercise viewport sizing at multiple densities,
wheel and drag input through the track, accessibility actions, focus reveal,
and nested-scroll boundary propagation. Omitted scrollbar metrics use the
provided controller. All 119 component rustdoc examples pass.

The native Date instructional scenarios pass real editing, focus, selection,
and clearing checks together with annotation-notch and canvas-size pixel
assertions. Sheet color figures pass checks at one and four samples, including
fractional scaling, independent theme halves, and icon colors compared with
equivalent native icon controls. Shared native text tables cover intrinsic
column sizing, spans, typography changes, independent heading ink updates, and
accessible row/cell structure. Specification headers now use the reference's
400 weight and muted foreground, and the Switch measurement table has the
reference's three columns, 18 data rows, and merged element cells.
These checks establish the exercised contracts rather than complete visual
parity with every official figure.

The preceding WebAssembly milestone passed semantic checking and compilation.
A browser check verifies the real directory, the article back button, independent
chapter selection, and immediate scrolling after selecting Guidelines in Buttons.
This is a limited browser check rather than an all-page comparison. The complete
native directory traversal is being rerun after a targeted diagnostic established
that its Loading geometry assertion ran before the responsive header settled:
the first measured viewport reduces the hero height by 8 pixels, while subsequent
frames and actual control toggles preserve the settled bounds. The original
bounds-stability assertion remains in place. A subsequent native chapter-tab
regression passes hover, press, typography, focus, stable targets, and idle
painting checks at one and four samples. Native clock-driven selection transitions
now pass geometry, pixel, and idle-work checks; complete browser motion comparisons
remain open.
The fresh browser build for these subsequent changes is still being verified.

The browser also reports expensive article transitions dominated by CPU tree
building. Concurrent correctness tests make these observations unsuitable for a
controlled performance comparison, but they do not support a claim of smooth
navigation or complete performance parity.

That browser milestone exposed a Date accessibility integration defect: opening
an instructional calendar placed its bottom actions below the window. The
native correction hosts dialogs above the article viewport and restores 48px
action targets while preserving the 360 × 572 dialog size. The actual article
route regression passes Entry, Formatting, and Labels selection and cancellation,
visible actions, modal focus, background hiding, scrim coverage outside the
figure, and narrow-window resizing. The custom titlebar is outside that host.
Hardware keyboard ordering and the corrected browser scene still need checking.

The chapter-tab pixel assertions also exposed overly soft shared shape edges.
An independent native 2px border reproduced gray pixels where black fully covers
the pixel. Correcting the analytic coverage half-width passes that unchanged
test at one and four samples, including fractional placement and clipping, plus
the original strict chapter-tab border assertions. Scaled fields, the clock,
calendar focus, and scrollbar interaction regressions also pass.

The current source inventory contains 36 directory destinations, each declaring
Overview, Specs, Guidelines, and Accessibility chapters. This establishes the
declared navigation structure, not the fidelity of their contents. Several
action and selection articles still share generic instructional figures; these
must be replaced with the corresponding component-specific native layouts.
Every article still needs its own reference comparison and interaction review.

Odd-width app-bar centering currently follows the integer `Px` layout contract:
a 32-pixel logo in a 725-pixel bar starts at relative x=346. The native directory
assertion checks this exact placement. It does not establish the half-pixel
centering that a browser's fractional CSS layout can produce; that difference
remains part of the visual comparison.


Long specification chapters now compose visible sections on demand while retaining
visited examples within the chapter. Unplaced sections leave drawing, pointer,
and accessibility geometry; the active focused item remains available for reveal.
Chapter switches preserve the common header and tab identities. Native regressions
cover cold TOC navigation, later wheel input, real control state, focus traversal,
and cleanup. Fixed-size figure canvases preserve canonical dimensions before
outer scaling, preventing clipped outlines and inaccessible trailing buttons when
a section first appears. The affected figure and interaction run passes 53 native
scenarios, alongside 332 ordinary component tests and 119 runnable component docs.
The updated WASM build and color check pass. Browser checks verify the corrected
date-input canvas, calendar opening, and synchronized confirmed values.

A local 1280×800, density-1, four-sample headless probe records Date States wheel
CPU medians of 7.3ms and 7.7ms in light and dark scenes, compared with 349ms and
356ms in the earlier eager chapter. Maximum geometry updates fall from 28,506 to
1,649 nodes in those phases. This is a scoped development-profile measurement;
desktop load and GPU clocks are uncontrolled. Unknown section extents still use
estimates until measured, and retained offscreen animation consumers can remain
active. The historical all-directory interaction scenario still needs
viewport-aware queries: compact variant checks must navigate back from their
measurement sections. Date and time controls remain in their own lazy sections;
their integrated checks must first reveal the corresponding section.
Its assertions have not been removed or counted as a passing current catalog run.
Complete normalized references and all-page interaction verification remain open.

Outlined text-field anatomy now positions its live fields and nine callouts from
the canonical reference geometry. The floating-label and supporting-text boxes,
solid connectors, and endpoint circles are drawn as vectors around real editable
fields. Eight native text-field scenarios pass, including light and dark anatomy,
clear actions, retained values after resizing, and constrained editor interactions.
This scoped geometry check does not establish full-page pixel equivalence.

A narrow-editor rendering regression showed that text could retain its leading
glyphs after the controller scrolled to the caret. The text pipeline now applies
horizontal buffer scrolling to the glyph origin while retaining its clip bounds.
Scroll changes also update cached editor draw data and participate in draw-command
equality. The actual filled and outlined field pixel regression passes, alongside
333 ordinary component tests and eight native text-field showcase scenarios.
The updated WASM build and color check pass. Browser checks verify complete
per-key input values and visible glyph movement between Home and End without
changing the text. Whole-string browser automation typing is not treated as
evidence of a framework input failure; direct per-key input and paste are checked
separately.
