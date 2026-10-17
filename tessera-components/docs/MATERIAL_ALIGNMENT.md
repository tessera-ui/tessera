# Material 3 Expressive alignment

This checklist tracks the complete [Material component catalog](https://m3.material.io/components),
inspected on 2026-10-17. It is an implementation and verification ledger, not a
claim of complete visual equivalence. Existing modules still require a review of
their current Expressive variants, interaction states, and responsive behavior.

## Comparison conditions

Use bundled Roboto, identical theme colors, locale, density, viewport, content,
and interaction state. Compare rendered bounds, typography, shape, colors,
disabled/hover/focus/pressed states, pointer and keyboard behavior, and accessibility.
Repeat on native and WebAssembly. Record screenshot evidence separately from
headless layout assertions. Animation and font rasterization need their own checks.

## Catalog coverage

| Official component | Tessera implementation | Remaining verification or implementation |
| --- | --- | --- |
| Button groups | `button_groups` | Audit connected/standard sizes, selection, shape motion |
| Buttons | `button` | Five sizes, round/square presets, controlled toggles, pressed shape motion, and elevation transitions implemented; native 1×/4× GPU geometry, typography, selection, semantics, and disabled input verified; WASM variants, pointer/Enter/Space, narrow wrapping, and resize/screenshot regression checked; complete normalized reference snapshots, dark themes, custom-shape states, constrained icon/label allocation, elevation snapshots, and motion-curve comparisons |
| Extended FABs | `extended_floating_action_button` (new) | Three sizes, icon sizing, label-only mode, disabled input, dynamic props, and reversible expansion tested; native GPU cycle restores identical pixels; WASM pointer/keyboard activation and three-size label restoration verified; complete color/hover/focus/disabled reference comparisons |
| FAB menu | `floating_action_button_menu` (new) | Native geometry, short viewports, input blocking, state retention, reversible stagger, GPU pixel recovery, and published AccessKit tree tested; WASM actions, arrow keys, reverse Tab, Escape, focus restoration, and three color sets verified; complete reference state snapshots, larger trigger anchoring, icon morph, and web baseline-menu presentation |
| Floating action buttons | `floating_action_button` | 80dp medium size with 28dp icon and 20dp corners added; elevation transitions implemented; verify hover/focus/pressed snapshots and remaining color variants |
| Icon buttons | `icon_button` | Five sizes, three widths, round/square presets, controlled toggles, vibrant/inherited colors, and 48dp targets implemented; native 1×/4× GPU geometry, fractional density, dynamic colors, input, selection semantics, and resize recovery verified; WASM four styles, three widths, pointer/keyboard activation, long press, disabled input, narrow wrapping, and lazy-list state retention verified; complete normalized reference state snapshots, dark themes, custom-shape states, and motion comparisons |
| Segmented buttons | `segmented_buttons` | Audit single/multiple selection and states |
| Split buttons | `split_buttons` | Five-size typography and icon contexts, sized outlines, trailing width priority, optical offsets, pressed/hover/focus shapes, controlled menu expansion, and arrow/elevation motion implemented; native 1×/4× GPU and fractional-density geometry, dynamic props, disabled semantics, and width-priority checks passed; full example menus, resizing, and lazy-list retention exercised on native and WASM; mixed-size/wrapped-label visual-height synchronization covered by 1×/4× native pixel readback at 1.5 density and WASM 390px/877px resize recovery; complete normalized state snapshots, dark themes, and motion-curve comparisons |
| Date pickers | `date_picker` | Single-date layout/input/year grid tested; implement range/docked variants; complete state screenshots |
| Time pickers | `time_picker` | Dial/input interaction tested; add/audit landscape layout; complete state screenshots |
| Loading indicator | `loading_indicator` (new) | Official cubic morph data and spring timing verified; 48 GPU comparisons across eight sequences and 1×/4× sampling meet a 2.5% normalized coverage-error limit; animation/paused frames reuse buffers; native progress semantics and WASM pause/resume plus 0/50/100% progress verified; complete reference-theme screenshots and custom-size/container comparisons |
| Progress indicators | `progress` | Audit current wave, gap, stop, determinate/indeterminate variants |
| Navigation bar | `navigation_bar` | Flexible top/start arrangement tested; complete state/responsive screenshots |
| Navigation drawer | `navigation_drawer` (new) | 336×56dp destinations, selection and disabled input tested; destination switching verified in WASM; complete modal overlay integration, transitions, and full state screenshots |
| Navigation rail | `navigation_rail` | Audit collapsed/expanded/modal modes and current tokens |
| Bottom sheets | `bottom_sheet` | Audit standard/modal geometry, dragging, dismissal, and focus |
| Side sheets | `side_sheet` | Audit modal/standard geometry, dragging, dismissal, and focus |
| App bars | `app_bar` | Compact top bar exists; add/audit medium/large layouts, centered titles, subtitles, and scroll behavior |
| Badges | `badge` | Audit dot/count geometry, offsets, and accessibility |
| Cards | `card` | Audit filled/elevated/outlined and interactive states |
| Carousel | `carousel` (new) | Multi-browse, uncontained, mixed-aspect uncontained, hero, centered hero, and full-screen layouts implemented with virtualized full-size content, 28dp masks, snapping/free scrolling, controller navigation, reverse layout, keyboard and accessibility actions; Native checks include source geometry/padding vectors, 1.25×/1.5×/2× density, dynamic props, pointer cancellation, zero-width/resize recovery, a responsive lazy feed, and 1×/4× GPU readback; WASM layout switching, horizontal/vertical input, Home/End, activation, narrow-window recovery, idle content, and scrolling out of the lazy viewport are verified; complete reference state comparisons and motion audits |
| Checkbox | `checkbox` | Audit selected/indeterminate/disabled states and transitions |
| Chips | `chip` | Audit assist/filter/input/suggestion variants and states |
| Dialogs | `dialog` | Audit basic/full-screen variants, actions, dismissal, and focus |
| Divider | `divider` | Audit horizontal/vertical thickness, insets, and colors |
| Lists | `list_item` | Audit line counts, leading/trailing content, selection, and Expressive grouping |
| Menus | `menus` | Anchoring and outside dismissal tested; trigger focus restoration, explicit fallback, disabled trigger recovery, and forward/reverse Tab exit passed native GPU regression; WASM selection, Escape, Enter/Space reopening, Tab exit, narrow-window recovery, and lazy-list retention checked; queued close/navigation barrier covered by native core/order regressions and WASM rapid keyboard sequences; Expressive group surfaces, standard/vibrant colors, position-specific corners, selected menu-radio semantics, trailing-space reservation, and scrolling implemented; shared cascading overlays, branch/chain dismissal, source removal/disable cleanup, and virtualized Tab-exit navigation implemented; shadow overflow, rounded content clipping, and scrolling shadow input bounds GPU-tested; normalized group/state snapshots, morphing, hover switching, compact adaptive layout, and remaining focus audits |
| Radio button | `radio_button` | Audit selection, disabled state, group keyboard behavior, and transitions |
| Search | `search` | Focus, typing, submit/dismiss fixed; audit expanded/full-screen layouts and state screenshots |
| Sliders | `slider` | Single, centered, and range keyboard adjustment, fixed Tab order, peer constraints, disabled input, dynamic steps, and retained callbacks verified in native tests and WASM; published AccessKit handles retain identity across value/step changes and expose current peer limits; implement vertical orientation and optional value indicators; complete five-size and reference state comparisons |
| Snackbar | `snackbar` | Audit action/dismiss variants, timing, focus, and live announcement |
| Switch | `switch` | Audit thumb/icon, checked/unchecked/disabled states, and transitions |
| Tabs | `tabs` | Audit primary/secondary, fixed/scrolling, indicator, and keyboard navigation |
| Text fields | `text_field` | Shared typography and outline fixes verified; audit error, supporting text, counters, icons, multiline |
| Toolbars | `toolbar` (new) | Floating horizontal/vertical and collapsed layouts tested; docked 64dp height and 4–32dp spacing tested at 132/140/320/800px widths; pointer/keyboard activation and docked actions verified in WASM; complete transitions, FAB combination, scroll behavior, and full state screenshots |
| Tooltips | `tooltip` (new) | Plain/rich page-level overlays, placement fallback, delayed hover, focus, touch hold, Escape/outside dismissal, and custom actions implemented; native GPU tests verify focus, descriptions, content/variant updates, pixel restoration, release cancellation, later clicks, and scroll cancellation; WASM rendering, word wrapping, pointer actions, repeated forward/reverse Tab, action activation, dismissal, and trigger re-entry verified; complete reference state snapshots, short-viewport/lifecycle/host coordination, caret and motion audits |

## Icon button size and state verification

The [current icon button specification](https://m3.material.io/components/icon-buttons/specs)
defines five sizes, two shapes, four styles, and three widths. Token values were
also checked against the AndroidX `IconButtonDefaults` and size token sources
on 2026-10-17.

| Size | Visual height | Icon | Narrow width | Uniform width | Wide width | Outline |
| --- | --- | --- | --- | --- | --- | --- |
| XS | 32dp | 20dp | 28dp | 32dp | 40dp | 1dp |
| S | 40dp | 24dp | 32dp | 40dp | 52dp | 1dp |
| M | 56dp | 24dp | 48dp | 56dp | 72dp | 1dp |
| L | 96dp | 32dp | 64dp | 96dp | 128dp | 2dp |
| XL | 136dp | 40dp | 104dp | 136dp | 184dp | 3dp |

The default variant is now filled. Transparent actions must select `.standard()`;
the app bar's window controls do so explicitly. Touch targets expand independently
of the visible container to at least 48dp when parent constraints permit. Native
readback measures all fifteen visual and icon sizes, checks clicks in transparent
touch padding, retains disabled geometry, and preserves toggle identities across
selection and ordinary-action mode changes. Separate runs cover 1×/4× MSAA and
1.5 density, followed by collapsed-width recovery.

Filled toggles use surface-container colors while unselected; selected tonal
toggles use secondary/on-secondary, and selected outlined toggles use inverse
surface colors with no border. The outline uses outline-variant; disabled outline
opacity is 38%, independently of the 10% disabled filled-container opacity.
Native pixel checks cover changing colors, inherited content updates, inferred
content for a custom container, and explicit content overrides.

Square radii are 12/12/16/28/28dp; pressed radii are 8/8/12/16/16dp. Selection
exchanges the round and square resting shapes, with press taking precedence.
Both action and toggle shape transitions use the critically damped default
effects spring. Fixed custom shapes stop internal corner motion.

The complete native example also checks three width settings, four variants,
two-second virtual holds, disabling/re-enabling, and 390dp viewport recovery.
The row must actually leave the lazy build before returning with its configuration
and action count intact. The example holds these values at page scope; local
state inside the virtualized demo was discarded when the item left the viewport.
WASM reproduces the scroll-out/return cycle with nondefault shape, width, color,
mode, and selected icons, then continues pointer and Enter activation.

WASM checks also cover Space, forward/reverse Tab, two-second pointer holds,
disabled activation, all four styles and three widths, 390×780 wrapping, and
recovery to 877×871 at 1.5 density. The viewport override is reset afterward.
These checks do not establish complete normalized reference-pixel or motion
equivalence, dark-theme coverage, or every custom-shape interaction.

```sh
cargo +stable test -p tessera-components --lib icon_button:: -- --include-ignored --test-threads=1
```

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

## Expressive vertical-menu implementation

Menus now use the recommended vertical container and group shapes instead of
baseline 4dp container corners. `menu_group` supports standalone, first, middle,
and last positions; middle and last groups insert a 2dp gap outside their surface
and semantics bounds. Groups expose an optional accessible name and inherit the
provider's standard or vibrant color style. An open provider propagates color
changes through retained group contexts instead of retaining its initial colors.

Items use a 44dp minimum visual height with 48dp minimum interaction targets,
body-large labels, body-medium supporting text, and label-small shortcuts.
Horizontal item margins and content insets total 16dp; optional icons are 20dp
with 12dp spacing. Absent icons reserve no empty column. Weighted labels are
measured after fixed icons and shortcuts, wrapping within the remaining width.
Optional `selected` props publish `MenuItemRadio` with both checked states;
submenu triggers expose expanded/menu-popup state and a vector chevron.
Items with no callback or submenu remain disabled. Menus scroll within their
maximum height rather than discarding overflowing actions.

Native 1× and 4× GPU regressions at 1.5 density verify group spacing,
16dp text insets without a phantom icon, non-overlapping shortcuts, wrapped
narrow labels, all standard/vibrant selection fills, checked radio semantics,
disabled input, fixed interaction bounds, pointer selection, and complete pixel
recovery after width/color changes. The component suite passes 299 tests including
GPU cases; 106 component rustdoc examples also pass. Grouped menus retain the
existing trigger/fallback/Tab restoration regression. The release WASM build
passes color checking, displays both surfaced groups, and completes queued
cross-group arrow/activation/reopen input and Escape dismissal without console
errors. The browser viewport remains at its native 877×871 size and 1.5 density.
Native example compilation also passes. The complete native example passes the
XS/XL menu actions, all split styles, disabled input, 390dp resize/restore, and
lazy-list state-retention probe after adopting group surfaces.

These changes establish the static vertical-menu surface and selection model.
They do not yet establish complete visual equivalence: active/inactive group
morphing, transition curves, grouped dividers/labels, dark-theme snapshots,
and the remaining cascade states still need dedicated reference comparisons.

## Cascading-menu placement and lifecycle

A real GPU regression exposed submenu items that had valid bounds outside the
parent but remained clipped for rendering and input: clicking `Copy link` left
the action count at zero. Cascades now share an overlay layer with their root
provider. Source item window bounds anchor each level, and stable request keys
keep independent focus scopes when several levels are open. A right-edge child
flips to the left when there is room. Leaf selection closes the full chain;
clicking an ancestor action passes through in one click. Outside dismissal,
source removal/disable, and programmatic root closure also clean up descendants.

Native 1×/4× GPU checks verify actual fill pixels beyond the parent clip and
pixel restoration on close, three-level actions, left-side fallback, 390px
resize recovery, repeated source removal and disable/re-enable, restoration to
valid remaining actions, and preservation of explicit external focus.

Two runtime contracts support teardown safely. State slots remove entries and
release their table borrow before dropping remembered resources; resource
cleanup can update other live owners, and liveness checks safely return false
at thread shutdown. A real component regression exercises conditional resource
recycling, child unmount, runtime reset, and thread teardown. Newly attached
position observers receive their initial rectangle even when the node already
has an unchanged observer. A separate component regression verifies attachment,
idle suppression, reattachment, and notification of both observers on resize.

Tab exit after restoring a popup trigger uses `FocusManager::request_move_focus`.
This queues navigation through the same beyond-bounds path as keyboard traversal
and retains retries across headless frames. Following keys wait for layout;
window blur, capture, and later explicit focus requests cancel queued traversal.
An actual lazy-list regression starts with the previous item unmaterialized,
closes the menu, then visits items 9, 8, and 7 and returns to item 8. It also
checks capture rejection and cancellation by a later explicit request.

The native suites pass 122 core tests and 303 component tests, including
GPU cases; 90 core and 106 component doctests also pass. The latest release WASM
build passes color checking. Browser checks select through three levels, return
with Left/Escape, exit with Tab, and retain an exact action count. A reverse-Tab
burst first exposes the previous lazy item and then processes Tab/Enter in order;
the earlier failed sequence now selects exactly once. A 390px resize keeps the
leaf usable and retains the count after restoring the native viewport. A single
click on an ancestor `Duplicate` action raises the final count to eight. No
browser console errors were observed. The freshly built native example also
passes the full split-button/menu/resize/state-retention headless probe after
these focus changes. Narrow cascades currently remain within the viewport; when two full
menus cannot fit side by side they overlap. Compact adaptive presentation,
hover-driven branch changes, active/inactive group morphing, full dark-theme
states, and transition timing remain part of the reference alignment audit.

## Menu shadow bounds

The grouped-menu browser preview exposed hard gray edges beside rounded group
corners. Both the panel and its scroll viewport clipped the elevated surfaces'
shadows to their rectangular bounds. A native GPU regression reproduced missing
shadow pixels immediately outside the panel at both 1× and 4× sampling.

Panel elevation now paints outside the content clip. Ordinary menus clip their
contents to the panel shape. Each group also clips its own contents to its shape,
while its parent viewport reserves paint overflow for the group shadows. When
content overflows vertically, the scroll viewport still crops the top and bottom
and keeps lateral shadow overflow. Shadow pixels do not expand the interactive
menu bounds.

The regression covers both grouped and ordinary menus at 1.5 density: near-edge
shadow coverage, distant fade to the page, cache-stable pixels, complete recovery
on close, long-menu scrolling, scroll-edge content clipping, and outside dismissal
from the shadow region without invoking a menu action. This does not establish
complete shadow-token or animation equivalence across all elevated components.

The rebuilt release WASM preview shows the external shadow falloff on three
simultaneous menu levels, with no console errors. Clicking the left shadow region
closes the chain without incrementing the action count. Reopening from the trigger
and selecting the third-level action increments the count once. One follow-up
Enter sequence after outside dismissal activated `Show snackbar` instead of
reopening the menu; that full-example focus-restoration boundary remains under
audit despite the passing isolated native focus fixtures.

Several subsequent immediate and settled close/reopen sequences, including a
fresh browser load, restored the trigger correctly. The earlier episode has not
been reproduced consistently, so these observations do not establish its cause
or a fix. Layout-snapshot metadata alone also did not expose actual focus for
the full native example; published accessibility focus or observable keyboard
activation is required for that check.

## Menu live anchors and placement fallback

The optional `menu_provider().anchor(...)` prop supplies the current rectangle
relative to the provider. It takes precedence over `MenuController::open_at`'s
saved rectangle and can follow global-position state while the menu stays open.
Omitting it retains the existing saved-anchor behavior.

Placement includes the requested gap when checking each side. If the requested
side does not fit, the opposite side is tried before clamping. Start/end alignment
also has an alternative, and an exactly fitting viewport clamps to zero instead
of leaving a positive out-of-bounds origin. Positive offsets on the expansion axis
point away from the anchor on either side.

Native 1×/4× GPU checks at 1.5 density cover a provider with a nonzero origin,
moving an open menu without losing its focus, removal of the old panel pixels,
top/bottom/left/right fallback, a gap that changes the side-fit decision, an
exact-fit resize, and returning to the saved rectangle when the prop is cleared.
The bottom-edge case failed before the placement fix at both sample counts.
The complete component suite passes 303 tests, and 106 component doctests pass.

The example derives current anchors for its normal and split-button menus. A
remembered source resource closes the active menu when that source is removed.
It also closes menus whose source bounds no longer intersect the page, since
virtualized overscan may retain an offscreen source even in a collapsed viewport.

The freshly built native example passes the complete source-visibility probe:
the menu remains aligned with its button and a 6px gap at 1.5 density before and
after a width change, both menu modes close when the usable page collapses,
neither reopens on restoration, and their subsequent actions each increment the
correct counter once. The first collapsed-page run failed before the explicit
visibility check was added.

The complete native split-button probe also passes after the anchor changes:
all five sizes, normal and extra-large menu actions, all four styles, disabled
geometry and input, narrow-window menu selection, and lazy-list state retention.

## Button size and state verification

Unweighted row children now receive the remaining width after preceding children,
including in rows with weighted children. Buttons reserve leading/trailing icon
space before allocating label width. A 260px extra-large button with two 40px
icons gives its `Go` label 20px; the label occupies two 40px lines and the button
grows to 176px rather than overlapping or squeezing the trailing icon. Native
checks preserve both icon sizes and keep the label inside the container.

Button presets use 32/40/56/96/136dp visual heights, 20/20/24/32/40dp icons,
8/8/8/12/16dp icon spacing, and 1/1/1/2/3dp outlines. Typography changes from
label-large to title-medium, headline-small, and headline-large with size.
Small buttons use 16dp horizontal insets; extra-small uses the current reference
helper's 12dp horizontal and 6dp vertical insets. Explicit content padding takes
precedence over uniform padding, and explicit outer constraints still apply.

Native GPU checks separately measure visual heights and 48dp minimum touch
targets. Disabling preserves the target and geometry while suppressing actions.
Elevation follows the visual surface rather than its enlarged touch target,
avoiding gray bands around small elevated buttons. Default elevated labels and
icons retain the primary role; custom container colors infer a matching content
role, and explicit content colors take precedence. Dynamic size changes update
both the inherited font metrics and icon sizes.

WASM checks cover all five color variants, pointer and Enter activation,
disabled clicks, and 390×780 wrapping. The final elevated comparison confirms
primary-colored content and the removal of the small-button shadow bands;
viewport overrides are reset and the inspected build reports no browser errors.
Native example probes cover all five sizes, variant switching, disabled geometry/input, and resize retention.
These checks do not replace normalized reference-pixel comparisons or every
interaction state in the catalog above.

Round/square presets resolve corners against the measured visual container.
Square radii are 12/12/16/28/28dp; pressed radii are 8/8/12/16/16dp. Ordinary
press motion uses a critically damped effects spring; toggle shape and outline
motion use the fast spatial spring. Selection reverses the resting shape.
Custom shapes remain fixed. Shape, outline, and shadow share resolved geometry;
resting motion stops requesting frames.

Controlled toggle selection uses `selected(...)` plus `on_click(...)`, retaining
the label, button role, published identity, and click target. AccessKit exposes
the toggled state while disabled and removes it when switching back to an action.
Text buttons have no toggle variant. The Basic Components page provides shape,
action/toggle, variant, and enabled controls.

Validation includes 294 component unit/GPU tests, 105 runnable component
doctests, 15 foundation tests and three foundation doctests. Full native example
probes exercise all five variants, round/square and action/toggle controls,
selection, disabling, resizing, and held/released XL shapes. WASM checks cover
Filled/Tonal/Outlined/Elevated toggles, Enter/Space reversal, disabled clicks, and 390×780
wrapping. Full normalized reference snapshots, dark themes, custom-shape
interaction snapshots, constrained two-icon label allocation, elevation state
snapshots, motion-curve samples, and stroke-color motion still require verification.

Ordinary clickables no longer create a long-press recognizer implicitly.
Holding a normal button for two seconds still activates it once on release;
only a supplied component-owned long-press recognizer can suppress that click.
This keeps tooltip touch recognition independent from ordinary button behavior.
The timestamped native GPU regression covers both 1× and 4× sampling; tooltip
long-press consumption and later normal activation are included in the suite.
The full native example also survives resizing to a 1px-wide viewport and back
without losing its action count. A two-second browser mouse hold activates once
on release, and the final WASM error log remains empty.

## Pointer dispatch verification

Pointer samples now retain their queue-time positions. Dispatch completes each
sample's Initial/Main/Final passes before advancing to the next event, preserving
capture and consumption order. Native GPU regressions at 1×/4× sampling release
inside then move outside in one frame, queue clicks on two different buttons,
and hold/release independent pointers. Both held states persist through idle
frames and recomposition; runtime reset discards capture. Tooltip long holds,
scroll cancellation, and normal later activation remain covered by the suite.
The complete native example verifies inside release followed by outside movement,
two queued clicks on different buttons, two simultaneous held contacts, and two
independent releases. The final suite passes 115 core tests, 278 component
unit/GPU tests, 90 core doctests, and 105 component doctests. WASM verifies double
clicks, pointer/keyboard activation, cancelled drags, later clicks, and a two-second
hold. Separate checks cover scrollbar capture, carousel drag without activation,
and tooltip action dismissal without activating the underlying carousel item.
The final browser error log is empty.

Input dispatch caches each frame's eligible nodes, resolved contexts, and ordered
pointer modifiers. The cache is rebuilt with current layout metadata every frame.
The local native headless experiment uses a 1316×1307 viewport at 1.5 scale,
eight trials per event count, and warmed pipelines. Values below are median
render-request wall times in milliseconds, including frame work and IPC; event
queueing time is excluded. They are not isolated GPU durations.

| Movement samples in one frame | Previous batching | Ordered dispatch without cache | Final cached dispatch |
| --- | --- | --- | --- |
| 0 | 2.164 | 2.040 | 1.944 |
| 1 | 1.933 | 2.006 | 1.935 |
| 8 | 2.260 | 2.519 | 2.363 |
| 32 | 3.573 | 4.478 | 3.816 |
| 128 | 9.238 | 12.744 | 9.814 |

The final implementation retains event order and independent pointer state;
large batches still have a small measured cost over the previous behavior.

Web entry points install a panic reporter that preserves the previous hook
and logs the Rust error and source location. This exposed a zero-width slider
clamp with a 3px lower bound and a 0px upper bound during the browser screenshot
resize path. Collapsed single/centered/range sliders now use the container
midpoint, and layout tests shrink all three variants through zero and narrow
widths before restoring their original bounds and values. The exact WASM
sequence of narrow resize, scroll, restoration, variant change, and clipped
screenshot now succeeds. Pointer, Enter, and Space activation still work after
that sequence, and the final browser error log is empty.

## Reference tokens

Use current primary implementation tokens when older site illustrations conflict:

- [Split button specifications](https://m3.material.io/components/split-button/specs)
- [Split button implementation and size tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SplitButton.kt)
- [Source-over alpha compositing](https://www.w3.org/TR/compositing-1/#simplealphacompositing)
- [Expressive vertical-menu specifications](https://m3.material.io/components/menus/specs)
- [Vertical-menu shape and dimension tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/SegmentedMenuTokens.kt)
- [Menu button keyboard pattern](https://www.w3.org/WAI/ARIA/apg/patterns/menu-button/)
- [Menu keyboard navigation and focus return](https://www.w3.org/WAI/ARIA/apg/patterns/menubar/)
- [Button specifications and state colors](https://m3.material.io/components/buttons/specs)
- [Toggle button colors, shapes, and motion](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ToggleButton.kt)
- [Toggle button accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/button/)
- [Expressive button metrics and content padding](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Button.kt)
- [Expressive button size tokens](https://github.com/androidx/androidx/tree/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens)
- [AndroidX typography tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/TypeScaleTokens.kt)
- [Floating toolbar tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FloatingToolbarTokens.kt)
- [Docked toolbar tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/DockedToolbarTokens.kt)
- [Navigation drawer tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/NavigationDrawerTokens.kt)
- [Medium FAB tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/FabMediumTokens.kt)
- [Extended FAB specifications](https://m3.material.io/components/extended-fab/specs)
- [FAB menu specifications](https://m3.material.io/components/fab-menu/specs)
- [FAB menu implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButtonMenu.kt)
- [Expressive motion tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt)
- [Loading indicator implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/LoadingIndicator.kt)
- [Loading indicator tokens](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/LoadingIndicatorTokens.kt)
- [Loading indicator specifications](https://m3.material.io/components/loading-indicator/specs)
- [Loading shape regeneration](../../tools/material-shapes/README.md)
- [Slider keyboard pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider/)
- [Multi-thumb slider pattern](https://www.w3.org/WAI/ARIA/apg/patterns/slider-multithumb/)
- [Material slider interaction](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Slider.kt)
- [Slider specifications](https://m3.material.io/components/sliders/specs)
- [Tooltip specifications](https://m3.material.io/components/tooltips/specs)
- [Tooltip implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Tooltip.kt)
- [Tooltip keyboard pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tooltip/)
- [Carousel specifications](https://m3.material.io/components/carousel/specs)
- [Carousel layout and scrolling guidelines](https://m3.material.io/components/carousel/guidelines)
- [Carousel implementation](https://github.com/androidx/androidx/blob/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/Carousel.kt)
- [Carousel geometry tests](https://github.com/androidx/androidx/tree/androidx-main/compose/material3/material3/src/androidHostTest/kotlin/androidx/compose/material3/carousel)

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


## Split height performance smoke check

The full native example was sampled in five 30-frame batches at 1316×1307,
1.5 density, with the split-button section visible. The updated binary's median
was 2.08ms per idle frame and 2.94ms per small alternating-scroll frame. The
scroll metric includes JSONL input/render IPC, CPU work, and GPU waits; it is
not a GPU timestamp or a controlled performance improvement claim. The earlier
binary measured 2.68ms and 4.53ms respectively under different host load. Raw
samples are retained outside the repository as `split-height-baseline-wall.json`
and `split-height-after-wall.json`. No substantial regression was observed in
this smoke check; broader performance conclusions need controlled runs.
