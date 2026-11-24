## [v4.0.0] - 2026-11-24

- Embed native controls with `platform_view` on desktop, mobile, and web. Desktop hosts route input and compose native views in scene order; AppKit now shows mounted views correctly.
- Use `popup` for anchored layers with dismissal, focus handling, and Android predictive back. Pointer input no longer steals focus from a descendant that handled the press.
- **Breaking:** `render::Command` now includes a native platform-view boundary. Update exhaustive matches to handle the new variant; it is not a draw or compute command.

[v4.0.0]: https://github.com/tessera-ui/tessera/compare/tessera-ui-v3.0.0...tessera-ui-v4.0.0

## [v3.0.0] - 2026-11-16

Tessera 3 introduces typed component props, reactive state, and modifiers for
layout, input, and drawing. The breaking changes below explain how to migrate
application and component code from 2.x.

### Breaking changes

- **Components take named props and return builders.** `#[tessera]` functions
  declare owned parameters instead of an args struct and `impl FnOnce()`
  children. Required parameters become entry arguments; `Option<T>` parameters
  become setters, with defaults resolved in the body (`#[default(...)]` and
  `derive(Prop)` are gone). Child content is a `RenderSlot` /
  `RenderSlotWith<T>` and actions are `Callback` / `CallbackWith<T, R>`; each
  gets a closure setter and a `*_shared` setter. Configure a call through its
  returned builder; setters panic after that call has been built.
  `#[tessera]` no longer accepts arguments.

  ```rust
  // 2.x
  #[tessera]
  fn card(args: impl Into<CardArgs>, child: impl FnOnce()) { /* ... */ }
  card(CardArgs { padding: Dp(8.0), ..Default::default() }, || { /* ... */ });

  // 3.0
  #[tessera]
  pub fn card(#[prop(into)] title: String, padding: Option<Dp>, child: Option<RenderSlot>) {
      let padding = padding.unwrap_or(Dp(8.0));
      // ...
      if let Some(child) = child {
          child.render();
      }
  }
  card("Title").padding(Dp(8.0)).child(|| { /* ... */ });
  ```

- **Layout uses policies and modifiers.** `measure_node`, `place_node`,
  injected `measure` / `input_handler` closures, `DimensionValue`, and
  `width` / `height` args fields are replaced by
  `tessera_ui::layout::layout(policy)`, which emits a layout node configured
  with `.modifier(..)`, `.draw(..)`, and `.child(..)`. Implement
  `LayoutPolicy::measure` with `MeasureScope` / `LayoutChild` and return a
  `LayoutResult`; constraints are interval-based `AxisConstraint`s. Size,
  padding, and alignment come from `Modifier` chains (see
  `tessera_foundation::modifier::ModifierExt`). `group()` attaches semantics,
  focus, or input to content laid out by an enclosing layout. A component that
  emits nodes must use one of these primitives; modifiers attach only to the
  node it emits. `PxPosition` coordinates are now `f32`.
- **State is remembered per call.** Use `remember` / `remember_with_key`
  (returning copyable `State<T>` handles) instead of passing `Arc` state
  through args; `retain` keeps state while its call is absent. Screen-level
  data lives in models: implement `ModelInit`, take a `Model<T>` parameter, and
  access it with `with` / `with_mut`. `State::new` is only valid in model
  initializers.
- **Context.** `provide_context(value, || ..)` takes a value (`T: PartialEq`).
  `use_context::<T>()` returns `Option<Context<T>>` and consumers see updates
  when the provided value changes.
- **Async and time.** Use `Task` handles (cancelled on drop) with
  `spawn(Dispatcher::{Ui, Io, Default}, future)`, `on(dispatcher, future)`,
  `spawn_local`, and composition-scoped `launch(key, future)` /
  `launch_once(future)`. Animations await `with_frame_time(|now| ..)` in a
  launched task; builds and input handlers do not read a clock.
- **Entry points.** Replace `Renderer::run(app, |registry| ..register_pipelines..)`
  and per-platform setup with one shared entry function. The attribute
  generates the Android entry point; `main` calls `run().run_desktop()`
  (`run_ios()` on iOS) and web builds call `run().run_web()`:

  ```rust
  #[tessera_ui::entry]
  pub fn run() -> EntryPoint {
      EntryPoint::new(app).package(tessera_components::ComponentsPackage)
  }
  ```

  Plugins are registered as instances on the entry, are session-local (no
  `Send + Sync`), and are accessed through `with_plugin`.
- **Input and focus.** Event handling attaches through modifiers with typed
  `PointerInput`, `KeyboardInput`, `ImeInput`, and `ClipboardInput` contexts.
  `Focus` is replaced by `FocusRequester`, focus groups and scopes, and the
  free functions in `tessera_ui::focus`; `FocusModifierExt` and
  `CursorModifierExt` are imported from `tessera_ui::modifier`.
- **Custom rendering uses `tessera_ui::render`.** Draw and compute commands
  describe geometry and sampling regions. Register them through a
  `RenderModule` or `TesseraPackage`; UI draw pipelines use
  `tessera_ui::render::ui_stencil_state()` for clipping.
- **Assets.** Generated assets are `tessera_ui::Asset` values loaded
  asynchronously with `Resource::load(source, decode)`; `Source` is `Asset` or
  `Memory` bytes, and decoders receive `tessera_ui::asset::Bytes`.
- **Moved out of `tessera-ui`.** The router is in `tessera-shard`; the
  `Clipboard` service is `tessera_platform::Clipboard`; the `shard` macro and
  `tessera_ui_shard` are no longer re-exported.
- **Dependencies.** wgpu 30, AccessKit 0.25 types in the accessibility API,
  and Tessera's maintained `winit` fork (re-exported as `tessera_ui::winit`).
  Android uses `GameActivity` instead of `NativeActivity`.
- `TesseraConfig` gained fields such as `window`, `web`, and
  `max_non_sampler_bindings`; build it with `..Default::default()` or from the
  generated `res::application()`.

### Added

- `key` for stable identity in dynamic lists, and `retain` /
  `retain_with_key` for state that survives navigation.
- Shape clipping for rounded and elliptical masks (exposed as the
  `clip_shape` modifier in `tessera-foundation`); hit testing honors the same
  shapes.
- Accessibility semantics with roles, actions, and modal, selected, invalid,
  disclosure, and orientation states, published through AccessKit.
- Web (WebAssembly) support with `EntryPoint::run_web`, iOS support with
  UIKit integration, and Android system and predictive back
  (`tessera_ui::system_back`).
- Window insets for safe areas and keyboard occlusion
  (`tessera_ui::window_insets`), and `WindowConfig` options for
  undecorated windows, integrated title bars, application IDs, and icons.
- IME sessions synchronized across native and web platforms, and clipboard
  actions routed to focused editors.
- `testing` feature with `testing::UiTest` for layout-only or GPU-rendered
  frames with virtual time, input injection, accessibility queries, and pixel
  readback.
- `headless` feature with `headless::HeadlessRenderer` and
  `EntryPoint::into_headless` for offscreen rendering and scripted sessions.
- `profiling` feature with per-component build, layout, and render timings
  and opt-in GPU timestamps; `debug-dirty-overlay` feature to visualize
  rebuilt regions.
- Wheel scrolling normalized across platforms, nested scroll dispatch, and
  touch inertia.
- Modifier-owned asynchronous pointer input with `pointer_input(key, detector)`
  and `pointer_input_once(detector)`. A key change or detachment cancels the
  task; unchanged keys retain the running closure, so capture state handles for
  live values. `PointerInputScope` supports sampled events, same-dispatch
  consumption across Initial/Main/Final passes, and frame-clock long presses.
  Use the shared detectors in `tessera_foundation::pointer_gestures` for common
  gestures.

### Fixed

- Improved app startup across Linux systems and graphics adapters.
- Nested clip rectangles intersect correctly.
- Child taps do not activate ancestor click handlers, while drag and scroll
  observers still receive the press. Ordinary focus changes preserve held
  pointer presses.
- Passive input barriers stop propagation without canceling descendant gestures,
  including taps followed by a final position update on the web.

### Performance

- Less rendering work for idle interfaces, small updates, and effects that
  sample a limited area.
- Lower GPU memory use and less memory overhead when loading packaged assets.
- Smoother window resizing on Windows.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.5.0...tessera-ui-v3.0.0)

## [v2.5.0] - 2025-11-18 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.4.1...tessera-ui-v2.5.0)

## [v2.4.1] - 2025-11-17 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.4.0...tessera-ui-v2.4.1)

## [v2.4.0] - 2025-11-15 +08:00

### Changes

- Screen readers can navigate Tessera interfaces through AccessKit.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.3.0...tessera-ui-v2.4.0)

## [v2.3.0] - 2025-11-08 +08:00

### Changes

- Inertial scrolling now has a speed limit.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.2.0...tessera-ui-v2.3.0)

## [v2.2.0] - 2025-11-03 +08:00

### Changes

- `Dp` supports multiplication and division by floating-point values.
- Improved touch responsiveness and blur rendering.
- Fixed Android apps freezing after returning from the background.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.1.0...tessera-ui-v2.2.0)

## [v2.1.0] - 2025-09-22 +08:00

### Changes

- Draw and compute pipelines can be registered directly with the renderer.
- Rendering recovers when a window surface is lost or outdated.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v2.0.0...tessera-ui-v2.1.0)

## [v2.0.0] - 2025-09-17 +08:00

### Changes

- Rounded rectangle corner radii use `Dp`.
- Clipping applies consistently to drawing and input.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.2...tessera-ui-v2.0.0)

## [v1.13.2] - 2025-09-15 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.1...tessera-ui-v1.13.2)

## [v1.13.1] - 2025-09-13 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.13.0...tessera-ui-v1.13.1)

## [v1.13.0] - 2025-09-12 +08:00

### Changes

- Added `TEAL` and `ORANGE` color constants.
- Clipping now applies to custom drawing and fluid-glass effects.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.12.1...tessera-ui-v1.13.0)

## [v1.12.1] - 2025-09-07 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.12.0...tessera-ui-v1.12.1)

## [v1.12.0] - 2025-09-06 +08:00

### Changes

- Added a bottom navigation component and simplified the routing API.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.2...tessera-ui-v1.12.0)

## [v1.11.2] - 2025-09-05 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.1...tessera-ui-v1.11.2)

## [v1.11.1] - 2025-09-04 +08:00

### Changes

- Improved frame synchronization and window resizing behavior.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.11.0...tessera-ui-v1.11.1)

## [v1.11.0] - 2025-09-04 +08:00

### Changes

- Added a shared entry-point API for starting Tessera apps.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.2...tessera-ui-v1.11.0)

## [v1.10.2] - 2025-08-26 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.1...tessera-ui-v1.10.2)

## [v1.10.1] - 2025-08-25 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.10.0...tessera-ui-v1.10.1)

## [v1.10.0] - 2025-08-15 +08:00

### Changes

- Components gained clipping to prevent content from drawing outside their bounds.
- The calculator example gained switchable visual styles.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.9.1...tessera-ui-v1.10.0)

## [v1.9.1] - 2025-08-08 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.9.0...tessera-ui-v1.9.1)

## [v1.9.0] - 2025-08-08 +08:00

### Changes

- Added declarative routing with route-scoped state.
- `TesseraConfig` supports a custom window title.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.8.1...tessera-ui-v1.9.0)

## [v1.8.1] - 2025-08-03 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.8.0...tessera-ui-v1.8.1)

## [v1.8.0] - 2025-08-02 +08:00

### Changes

- Glass component borders gained a beveled highlight.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.7.0...tessera-ui-v1.8.0)

## [v1.7.0] - 2025-07-31 +08:00

### Changes

- Unified cursor position APIs and added event-blocking modifiers.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.6.0...tessera-ui-v1.7.0)

## [v1.6.0] - 2025-07-29 +08:00

### Changes

- Scrollbars support overlay and alongside layouts, with an always-visible default.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v1.5.2...tessera-ui-v1.6.0)

## [v1.5.2] - 2025-07-28 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.5.1...tessera-ui-v1.5.2)

## [v1.5.1] - 2025-07-25 +08:00

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.5.0...tessera-ui-v1.5.1)

## [v1.5.0] - 2025-07-25 +08:00

### Changes

- Added color interpolation, `Dp::ZERO`, and arithmetic operations for `Dp` and `Px`.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.4.0...tessera-ui-v1.5.0)

## [v1.4.0] - 2025-07-25 +08:00

### Changes

- Added clipboard clearing and Android clipboard support.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.3.0...tessera-ui-v1.4.0)

## [v1.3.0] - 2025-07-24 +08:00

### Changes

- Added convenient `DimensionValue` constants.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.2.0...tessera-ui-v1.3.0)

## [v1.2.0] - 2025-07-24 +08:00

### Changes

- Added a cross-platform clipboard API.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.1.0...tessera-ui-v1.2.0)

## [v1.1.0] - 2025-07-23 +08:00

### Changes

- Added helpers for implementing custom measurement behavior.

[Compare with previous release](https://github.com/shadow3aaa/tessera/compare/tessera-ui-v1.0.0...tessera-ui-v1.1.0)

## [v1.0.0] - 2025-07-21 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.5.0...tessera-ui-v1.0.0)

## [v0.5.0] - 2025-07-21 +08:00

### Changes

- Keyboard modifier state is available for shortcut handling.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.4.0...tessera-ui-v0.5.0)

## [v0.4.0] - 2025-07-20 +08:00

### Changes

- Added a callback for window close requests.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.3.0...tessera-ui-v0.4.0)

## [v0.3.0] - 2025-07-20 +08:00

### Changes

- Added window minimize notifications and close callbacks.
- Window managers handle resize cursors at window edges.

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.2.1...tessera-ui-v0.3.0)

## [v0.2.1] - 2025-07-19 +08:00

[Compare with previous release](https://github.com/tessera-ui/tessera/compare/tessera-ui-v0.2.0...tessera-ui-v0.2.1)
