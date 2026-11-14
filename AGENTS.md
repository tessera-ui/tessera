# Instructions

This document defines how You should assist in the Tessera project to ensure code and documentation are consistent with project architecture, style, and best practices.

Before changing Tessera code, read and apply the
[Tessera skill](crates/cargo-tessera/skills/tessera-good-taste/SKILL.md). It defines the application
programming contract; the guidelines below also govern framework implementation.

---

## Language Policy

- **All code, comments, and documentation comments must be written in English**, including rustdoc, commit messages, and PR descriptions. Use other languages only when absolutely necessary for functional clarification.
- **This document itself is a development guideline and should be maintained in English only. No i18n is required.**

---

## 🧠 Project Overview & Structure

- **Project Type**: Rust UI Framework
- **Main Crates**:
  - **tessera-ui**: Framework core (component tree, rendering, runtime, basic types Dp/Px, event handling, resources, async tasks, etc.)
  - **tessera-foundation**: Shared foundational UI building blocks (alignment, shape definitions, gestures, scroll physics, and the common layout, input, semantics, and clipping modifiers)
  - **tessera-components**: Official components and their shared rendering pipelines. The crate root holds design-system-independent primitives (row, column, boxed, image, pager, etc.); `md3` holds Material Design 3 components; `experiment::hig` holds experimental Apple HIG components
  - **tessera-macros**: The `#[tessera]` procedural macro for simplified component definition
  - **tessera-platform**: Platform services and plugins
  - **tessera-shard**: Shard components and routing
  - **tessera-i18n**: Fluent localization
  - **tessera-build**: Build-time asset, localization, and application metadata generation
  - **cargo-tessera**: The CLI for scaffolding, development, checks, packaging, and headless rendering; `tessera-mobile` is its mobile build library
  - **example**: Example project demonstrating framework usage

**Module Path Convention**: All modules must use the `src/module_name.rs` pattern. Do not use `src/module_name/mod.rs`.

### Design System Layering

- `tessera-components` places a component at the crate root only when it reads no design-system theme and implies no design-system visual, interaction, motion, or scrolling defaults. When in doubt, place it in a design-system module; promoting an API to the root later is additive.
- Each design system owns its public text, icon, scrolling, lazy container, text editing, and interaction feedback components and their style types. Rendering pipelines and crate-private mechanisms, such as text shaping, editing, virtualization, and scroll control, may be shared.
- Non-test code in root modules and other design systems must not depend on `md3`; `scripts/check-imports.rs` rejects such imports. Experimental design systems live under `experiment`.

---

## 🏗️ Core Development Model

### Component Model & #[tessera] Macro

- Components are stateless Rust functions annotated with `#[tessera]`. Persistent UI state is created with `remember` (returned as `State<T>`) and can be passed around as `State<T>`.
- Release slot-table borrows before dropping remembered values; destructors must check `State::is_alive` before touching other state.
- Inside the component function:
  - Compose child components to build the complete component tree
  - Use `Modifier` chains for node-local behavior such as layout modifiers, input, semantics, drawing, and focus
  - Use `layout(policy)` with a `LayoutPolicy` to arrange children and `LayoutModifier` for wrapping layout modifiers.
  - A component call does not occupy a tree node: its identity, state, and replay snapshot live on a call record. Only the `layout(policy)` and `group()` primitives emit nodes, so a component that wraps one `layout` call costs one node. `#[tessera]` takes no arguments; Tessera crates alias themselves with `extern crate self as ...;`
  - Modifiers attach only to the node emitted by the executing primitive. Use `group()` to attach semantics, focus, or input to content laid out by an enclosing layout.
  - The `__private` module of `tessera_ui` serves official macro expansions only. `cargo tessera check` rejects other uses outside the defining crate (T0005); expose a public API instead.
  - Expand selected child content through `RenderSlot` / `RenderSlotWith` during build. A parent decides whether, when, and where its slots expand.

### Component API Pattern

- Truly zero-config components may use `#[tessera] pub fn component()` with no parameters.
- Configurable components must use a single public entrypoint declared with named owned parameters: `#[tessera] pub fn component(foo: Foo, bar: Bar, ...)`.
- A component entry registers one use and returns its configuration handle (the generated `*Builder`); configure it by chaining, saving it, or passing it to helpers. Modifying a committed or stale handle panics. See [component calls](docs/COMPONENT_CALLS.md).
- Keep child content in `RenderSlot` / `RenderSlotWith`.
- Pass required values at entry as non-`Option` parameters, such as `text("Title")`; configure optional props with named setters.
- Do not introduce public wrapper variants (`*_with_controller`, `*_impl`, or extra public forwarding layers).
- Component parameters must use owned types satisfying the generated props model's `Clone + 'static` requirements. Component props and remembered values are UI-session-local and do not need `Send + Sync`.
- For parameters with optional/default behavior, use `Option<T>` and resolve defaults explicitly in the component body (for example, `unwrap_or`, `unwrap_or_else`, or an explicit `if let Some(...)` fallback).
- For optional external controllers, use `Option<State<...>>`; when `None`, create internal state with `remember`.
- Callback parameters use `Callback` / `CallbackWith<...>`; slot parameters use `RenderSlot` / `RenderSlotWith<...>`.
- The parameter type is the single source of truth for the generated builder surface:
  - `Callback` / `CallbackWith<...>` parameters get a closure setter plus a `<name>_shared(impl Into<...>)` setter.
  - `RenderSlot` / `RenderSlotWith<...>` parameters get a closure setter plus a `<name>_shared(impl Into<...>)` setter.
  - every other parameter gets a plain value setter.
- `#[prop(...)]` accepts only these orthogonal options:
  - `into` — declare the setter (and the required constructor parameter) as `impl Into<T>`.
  - `skip_setter` — do not generate a builder setter for this field.
  - `skip_eq` — exclude this field from the generated props `PartialEq`.
- Prefer `#[prop(into)]` for public `Option<T>` fields whose inner type has a useful conversion surface.
- Reserve `#[prop(skip_setter)]` for internal plumbing fields; `Option<T>` setters already accept `T`.
- Exception: when a public builder needs a deliberately custom semantic surface (for example, mutually exclusive modes such as `title(...)` vs `label(...)`), use hidden backing fields with `#[prop(skip_setter)]` and expose explicit hand-written semantic setters instead of leaking mechanical field setters.
- `Callback`/`RenderSlot` are immutable handles; do not rely on closure hot-swap semantics.
- If callback behavior depends on changing values, capture `State<T>` (or other stable handles) and read latest values at call time.
- Construct handles with `Callback::new`, `CallbackWith::new`, `RenderSlot::new`, and `RenderSlotWith::new`; do not add helper wrappers.

### Tessera Color Checking Design

- Tessera color checking belongs in `cargo-tessera`, not in `tessera-macros` or per-crate `build.rs` scripts.
- `tessera-macros` must remain a local lowering layer only: props/builders, component wrappers, control-flow group guard instrumentation, and runtime call structure generation. It must not read or write color metadata, build global call graphs, or decide whether a call is color-valid.
- `tessera-build` remains a build-time utility crate for assets and related generated bindings. It must not be used as the global color-checking frontend.
- `cargo-tessera` should run color checking before invoking the real `cargo build`, `cargo check`, `cargo run`, `cargo tessera dev`, or platform-specific build command. It should use the resolved Cargo package graph for the selected package, target, features, and profile, not just workspace crates.
- Tessera color is an execution-context color, not a general UI-code marker. The only colored contexts are:
  - free functions annotated with `#[tessera]`, or a project-approved alias that lowers to a Tessera component free function such as `#[shard]`;
  - the bodies of recognized Tessera-only API free functions, such as `tessera_shard::use_router`;
  - closures carried directly by `RenderSlot`, `RenderSlotWith`, or generated setters and required constructor arguments for parameters declared as RenderSlot types (including `impl Into<RenderSlot>` / `impl Into<RenderSlotWith<...>>`);
  - root build closures passed to `EntryPoint::new`, `HeadlessRenderer::new`, `testing::UiTest::content`, or `Renderer::run` / `run_with_config` / `run_web_with_config`;
  - the build closures of `key` and `provide_context`, and deferred Pending-property materializers;
  - content closures in the approved built-in render setters listed in [component calls](docs/COMPONENT_CALLS.md#built-in-render-setters). Their key providers and action callbacks remain uncolored.
- All other executable contexts are uncolored by default: ordinary free functions, impl methods, trait methods/default methods, local functions, callbacks, event handlers, async tasks, iterator closures, `Option`/`Result` combinator closures, and arbitrary higher-order function closures.
- Color is one-way and does not propagate: colored contexts may call uncolored code, but uncolored code, including helpers called from components and closures written inside them, must not call Tessera-colored functions or Tessera-only APIs.
- `State::new` creates model-owned state and is valid only in model initializers: `ModelInit::init` implementations and closures passed directly to `ModelStore::model`. `cargo tessera check` rejects other calls (T0006); like color, this does not propagate through helpers.
- Tessera-only APIs such as `remember`, `remember_with_key`, `retain`, `retain_with_key`, `provide_context`, `use_context`, `launch`, `launch_once`, `key`, `RenderSlot::new`, `RenderSlotWith::new`, and internal current-instance/group/slot APIs require a Tessera-colored context.
- `RenderSlot` and `RenderSlotWith` are Tessera functors: their carried closures are delayed Tessera fragments and may only be executed by the Tessera runtime at controlled build/replay boundaries.
- Do not introduce custom closure carriers that can execute Tessera code unless the carrier is explicitly added to the color-checker specification.
- The checker must use the `rust-analyzer` analysis crates as its semantic frontend and must run them as library crates from `cargo-tessera`; it must not depend on an LSP server process, `syn`, heuristic fallback parsing, `rustc_private`, or macro/build-script generated color metadata.
- The checker must fail closed when rust-analyzer cannot resolve semantic information that is needed to classify a Tessera-sensitive call. Diagnostics name the invalid uncolored boundary and call target.

### Component Tree & Node Metadata

- The component tree is managed internally by `ComponentTree` and node metadata structures, supporting layout, rendering, and event dispatch.
- Accessibility updates flatten layout and group nodes without semantics. Preserve explicit semantic nodes, their order, bounds, actions, and focus; synthesize only the root window node.
- Publish item selection separately from checked/toggled state. Accessibility focus actions must not activate their target.

---

## 📏 Layout & Measurement System

- Use `Constraint` and interval-based `AxisConstraint` to describe size constraints.
- Layout policies measure direct child layout nodes through handle-based APIs such as `MeasureScope`, `LayoutChild`, and `LayoutResult`.
- `LayoutChild::probe_intrinsic` probes natural content dimensions without publishing layout; it must not replace normal `measure`.
- `place_node` is used to position child nodes.
- Measuring a layout child does not place it. An unplaced layout node hides its entire subtree from drawing, input geometry, semantics, and keyboard focus.
- `layout::Stack` places children at (0,0) and sizes the node to their bounding rectangle. `group()` nodes do not lay out their children; the enclosing layout measures them.
- Track composition, measurement, placement, transforms, paint, and pointer-state reads separately. Frame consumers (focus, pointer input, position observers, the render scene, and accessibility) must update from structural and geometry journals. See [incremental frame maintenance](docs/INCREMENTAL_FRAMES.md).
- Direct layout-child adjacency and layout metadata persist across frames and change only at build/replay structure changes. Stable frames and cache hits must not rebuild these lists or restore subtrees.
- Keep Compose, Measure, Place, and Paint dependencies separate. State reads during measurement invalidate measurement; placement-only calculations may use `MeasureScope::placement` or `LayoutModifierInput::placement` and must not determine sizes or constraints. Implement `LayoutPolicy::place` when a policy supports cached placement.
- Draw-only properties must be excluded from both measurement and placement equality. Placement transforms invalidate window geometry without measuring. Parent-data modifiers should implement `equivalent` when their values can be compared.
- Keep measured control dimensions in local units and share one inherited `LocalToWindow` transformation across rendering, clipping, pointer input, focus reveal, IME, and accessibility. Use `LayoutCoordinates` for local-size consumers; do not feed window rectangle dimensions back into local layout or drawing. See [local layout coordinates](docs/LOCAL_COORDINATES.md).

---

## 🎨 Rendering & Pipeline System

- Rendering uses a pluggable architecture:
  - **DrawCommand**: Trait describing renderable objects
  - **DrawablePipeline**/**ComputablePipeline**: GPU rendering/compute logic
  - **PipelineRegistry**: Registers all pipelines at startup
- Components set draw/compute commands through `RenderInput`, `RenderMetadataMut`, and draw modifiers (`DrawModifierNode`, `LayoutNode::draw`)
- Draw commands use the measured bounds at their position in the modifier chain.
- The components crate provides common pipelines; applications register them by adding `tessera_components::ComponentsPackage` to their `EntryPoint`

### Shape clipping

- UI draw pipelines must set `depth_stencil: tessera_ui::render::ui_stencil_state()`.
  The renderer supplies a `Stencil8` attachment and manages its reference value.
  Independent atlas or texture-generation passes may use their own attachments.
- Resolve animated size-relative corners through shared shape definitions so
  fills, outlines, shadows, and clipping use the same measured geometry.
- `Modifier::clip_shape` clips the node and its descendants without changing
  measurement. Pointer hit testing must honor the same ancestor shapes.
- Retained scene pixels require conservative draw-command opt-in and must not turn
  per-command alpha into group opacity; see
  [incremental frame maintenance](docs/INCREMENTAL_FRAMES.md).
- Use shared frame-buffer ranges for transient uniforms/storage and pooled scratch
  leases for filter intermediates. Bound idle caches by bytes and age.
- Shape clipping supports up to 255 nested shape masks. `clip_to_bounds` clips do
  not consume stencil levels.

### Barrier System & Performance Optimization

- **SampleRegion**: An enum used by both `DrawCommand` and `ComputeCommand` to declare which part of the previously rendered scene a command samples. `DrawRegion` separately describes where the command draws. This distinction is crucial for effects like blur or glass morphism.
- `Global`: Samples the entire screen. This is expensive as it requires a full-screen texture copy.
- `PaddedLocal(PaddingRect)`: Defines a sampling region relative to the component's bounding box plus padding on each side.
- `Absolute(PxRect)`: Samples a specific, absolute region of the screen.
- **Performance Optimization**: Sampled draw passes copy only the union of their declared sampling regions and receive its window rectangle as `scene_rect`; `Global` copies the full screen. Sampling bounds are independent of draw clipping. **Batching** groups subsequent barrier commands with non-overlapping draw regions into the same render pass.

### WebGPU Resource Usage Guidelines

- Don't: create temporary mapped buffers when updating data. Use `Queue::write_buffer` and `Queue::write_texture` instead. For large generated uploads, recycle staging buffers in a pool.
- Do: group resource bindings by change frequency, starting from the lowest. Put per-frame resources in bind group 0, per-pass resources in bind group 1, and per-material resources in bind group 2 to reduce state changes.
- Don't: create many buffers or textures per frame. Coalesce smaller resources into larger ones (buffer subranges, texture atlases, texture arrays).
- Don't: submit many times per frame. Multiple command buffers per submission are fine, but limit `submit()` calls to a few per frame (for example, 1-5).

---

## 🎯 Event & State Management

- `State<T>` has one API on every thread and is `Send + Sync` only when `T: Send`. The owning UI thread holds its session's access gate and accesses state without locking; other threads take the gate, which the owner yields only at frame boundaries (`dispatcher::flush`). Never make the UI thread wait on a thread that may access its states.
- Session-local storage (contexts, input and accessibility handlers, router destinations) must not require `Send + Sync`; use `Rc`/`RefCell` rather than `Arc`/locks there.
- Asynchronous work uses `Task` handles that cancel on drop. `spawn(Dispatcher, future)` takes `Send` futures; `spawn_local(future)` runs `!Send` futures on the current UI session from any code, and `launch`/`launch_once` tie them to a component call. UI-session futures are polled first at the next frame boundary.
- Event handling is attached through modifier-based interaction APIs and other node-local capabilities, using typed closures over the framework input contexts.
- The typed input contexts are `PointerInput`, `KeyboardInput`, `ImeInput`, and `ClipboardInput`. Clipboard requests share keyboard ordering and focus/layout deferral.
- Treat input contexts as event/consumption surfaces, not generic renderer request bags. Cursor changes, semantics, IME publication, and window-side effects must use their dedicated modifier/controller/session APIs.
- Mouse, keyboard, and IME events are processed via event queues and must be consumed promptly.
- Use `focus::request_move_focus` when traversal may need lazy content to be revealed; it runs after layout and before following keys. `move_focus` is immediate on attached targets.
- Focus cleanup must detach all removed handles before selecting a restoration target. Removing focus must not activate an unrelated control.
- Preserve each pointer sample's position when queuing it. Dispatch each sample's
  Initial, Main, and Final passes in order before processing the next sample.
  Captured paths use stable instance identities and survive recomposition.
- A child's tap must not activate or focus ancestor tap handlers; drag and scroll
  observers still receive the press.
- Event handlers should be lightweight to avoid blocking the main loop.
- Gesture slop starts at the recorded press position. Direct and inertial scroll
  deltas share nested-scroll dispatch; advance inertia from the framework frame clock.
- When opening or closing focusable content, use `focus::defer_keyboard_until_layout` to preserve subsequent keys until the new layout and focus callbacks finish.

---

## 📐 Unit System

- **Dp (Density-independent Pixel)**: Used for public APIs, component sizes, margins, paddings, etc.; automatically adapts to DPI
- **Px (Physical Pixel)**: Used internally for rendering and measurement, for precise pixel-level operations
- The framework automatically converts between Dp and Px based on the screen scale factor. Prefer Dp for development.

---

## 🛠️ Contribution & Style Guidelines

- **Code Style**:
  - Keep general cleanup and style refactors within Tessera-owned crates; preserve submodule sources for upstream synchronization.
  - Publish forks with distinct package names, upstream library names, and versioned path dependencies.
  - Declare dependencies of Tessera-owned libraries in the root `[workspace.dependencies]` and inherit them with `workspace = true`, adding features locally. `cargo-tessera` declares its dependencies directly with full `x.y.z` caret requirements so CLI fixes can raise dependency floors independently. Submodule crates keep standalone manifests.
  - Write library dependency requirements as short caret versions (`"1.16"`, `"0.8"`) so downstream users are not forced onto the newest patch release. Add a patch component only when a fix or API from that patch is required, and keep `0.0.x` crates fully specified. Do not use open or upper-bounded ranges; crates whose types are exchanged must resolve to one shared version, and exact `=` pins require a comment explaining the lockstep requirement. CI checks direct minimal versions.
  - Enforce `rustfmt edition 2024` default rules
  - `use` imports must be grouped in four sections (standard library, third-party crates, crate root, submodules), separated by one blank line, sorted alphabetically within each group, and merged by root path
  - Treat `State<T>` (returned by `remember`) and values returned from `use_context::<T>().get()` as cheap `Copy` handles: do **not** call `.clone()` just to pass them into closures.
  - Avoid `*_for_*` “capture helper” variable names (e.g. `state_for_handler`). Prefer capturing the original value directly; if a local alias is needed, keep the same name.
- **Commit Guidelines**:
  - Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification
  - **Breaking Changes**: If a commit introduces a breaking change of public api, you MUST include `BREAKING CHANGE:` in the commit body or footer. This is required to trigger a Major version bump.
  - **Unreleased API changes**: If an API or behavior has never been included in a published release (i.e., it exists only in the working tree or an unmerged branch), you do **not** need to preserve backward compatibility for it. Use git (tags/releases) to confirm whether an API was released; when in doubt, discuss with maintainers before merging.
  - Ensure all tests pass, documentation is updated, and formatting checks are clean before committing
- **Documentation**:
  - Main documentation must be in English. Other language versions, if any, should be placed in the docs/ directory and kept in sync with the English version.
  - Documentation format should pass markdown lint if possible
- **Comment Policy**:
  - Documentation comments (`//!`, `///`) are written for end users. Describe purpose, behavior, and usage. Do **not** include implementation details, internal architecture notes, or references to upstream/source code (e.g., “ported from Compose”, file paths, commit hashes).
  - Keep documentation DRY. State each fact once, where users look for it. Do **not** restate what names, types, and signatures already convey, and do not repeat item docs in module, field, or impl docs. Crate-private items, private fields, and test modules need no doc comments unless they record behavior that is not obvious from the code.
  - Non-documentation comments (`//`, `/* */`) are allowed only when necessary to explain *why* a piece of code is written a certain way (non-obvious tradeoffs, invariants, safety/performance constraints). Do **not** restate what the code does.
  - Any other commentary-style comments are not allowed.

---

## Unit Test Standards

### What a test must protect

- Every test must identify a concrete failure in Tessera-owned behavior. Its
  assertions must fail when that behavior breaks, even if the test fixture still
  works. Test names must describe what is actually exercised and observed.
- Prioritize layout constraints and placement, density and fractional geometry,
  state lifetime and recomposition, input consumption and focus, text editing and
  IME, accessibility, clipping and sampling, and generated component APIs.
- Test pure algorithms directly when their rules matter: interval arithmetic,
  animation convergence, text offset mapping, color conversion, checksums, and
  allocation limits are valid unit-test subjects, including private functions.
- Small tests and exact numeric assertions are valid when they express a real
  contract. Do not delete tests merely because they are short, use private APIs,
  or inspect internal state needed to verify a runtime invariant.

### Choose the right boundary

- Use the smallest production entrypoint that exercises the claimed behavior.
  Test component wiring, event routing, callbacks, and retained updates through
  actual components and the layout/headless runtime. Test an isolated decision
  function directly only when the claim is limited to that decision.
- Fixtures may construct inputs and observe outputs. They must not duplicate
  production event dispatch, editing, formatting, state transitions, or clipboard
  logic and then assert results produced by that duplicate implementation.
- Mock external boundaries when necessary, such as platform clipboard access;
  keep the Tessera logic under test real. A fake dependency is acceptable; a
  second implementation of the behavior being tested is not.
- Test-only accessors may expose production state for observation or arrange a
  fixture. Do not retain obsolete algorithms behind `#[cfg(test)]` solely to keep
  their tests passing, or test transformations implemented only in such helpers.
- Do not let the fixture supply the behavior the test claims to verify.

### Tests that must not be added or retained

- Tests that only verify Rust or dependency behavior, trivial field assignment,
  transparent getters/constructors, or derived traits without a Tessera contract.
- Duplicate tests that exercise the same behavior and cases already covered by
  a stronger test or runnable rustdoc example. Keep distinct boundary cases and
  supported platforms; overlapping coverage alone is not sufficient for removal.
- Tests that search repository source text for identifiers, cfg spellings,
  function names, or code fragments as a substitute for compilation or behavior.
  Generated text may be asserted when it is the tool's actual output contract;
  validate syntax or compilation when the claim concerns usable generated code.
- Tests that lock incidental private representation, container order, helper
  call counts, or intermediate values without a correctness or performance
  requirement. Cache invalidation, ordering, resource bounds, and avoiding work
  at rest remain valid requirements when the test exercises the relevant path.
- Tests that claim a runtime effect but only inspect a constant, an enum result,
  or state changed by the test itself. Do not encode unsupported behavior such as
  callback closure hot-swapping as a public contract.
- Negative tests with multiple unrelated invalid preconditions that accept any
  panic or error. Set up all unrelated preconditions correctly and identify the
  intended failure; an earlier guard must not make the test pass accidentally.
- Self-confirming expected values computed by repeating the same algorithm or
  calling the same implementation used to produce the actual result. For mapping
  claims, assert an independently known one-way result; a round trip alone can
  hide matching mistakes in both directions. Round-trip properties remain useful
  when reversibility itself is the contract.
- Timing printouts and assertion-free demos disguised as unit tests. Put measured
  performance in benchmarks and interactive demonstrations in examples.

### UI assertions and deterministic execution

- Assert externally meaningful layout, pixels, actions, semantics, and state
  transitions. Exact dimensions, theme-state precedence, and Material tokens are
  appropriate when they are intentional component contracts.
- Use explicit frame timestamps for animation and gesture thresholds. Avoid
  wall-clock speed assertions, arbitrary sleeps, and dependence on installed
  fonts, local devices, network services, or the developer's desktop.
- Keep GPU and platform integration tests separate in execution requirements;
  document required adapters or platforms and use explicit opt-in where needed.
  Do not remove rendering coverage simply because it cannot run on every host.
- Run tests that mutate process environment variables or global density in child
  processes; restore other shared runtime state with test guards.

### Review and verification

- Before adding a test, state the regression it catches and inspect existing
  coverage. Prefer extending an existing scenario over adding another equivalent
  test. Do not add a test solely to increase the test count or coverage percentage.
- Before deleting a test, read the assertion and production path; identify why it
  lacks value or which retained test covers the same cases. Remove unused fixtures
  and imports with it. Do not delete a failing test to conceal a production bug.
- Run affected crate tests and relevant feature/platform checks after cleanup.
  Keep `cargo test --workspace --doc` in CI; runnable API examples are part of the
  supported test suite. Report failures caused by unavailable tooling or existing
  work separately from regressions introduced by the cleanup.

---

## ⚙️ Special Notes

- **example crate**: `example/Cargo.toml` uses `[lib]` pointing to `src/lib.rs` for shared entry logic (including Android) with the name `{package}_lib`, and `[[bin]]` pointing to `src/main.rs` for desktop. `src/lib.rs` must expose `#[tessera_ui::entry] pub fn run() -> EntryPoint`, and `src/main.rs` should only call `{package}_lib::run().run_desktop()` (`run_ios()` on iOS) to keep a single startup path and avoid PDB output name collisions.

---

## Component Documentation Guidelines

This project uses a strict, concise doc style for modules and component functions to
improve readability and enable testable examples.

### Module docs

- Public component modules use this 5-line template:
  1. `//! <short description>`
  2. `//!`
  3. `//! ## Usage`
  4. `//!`
  5. `//! <one-line app-level usage scenario>` (e.g., alerts, confirmations, multi-step forms, or interactive controls).
- `//! ## Usage` must be on its own line and must be followed by a blank `//!` line (do not inline the usage sentence on the same line).
- Other public modules use a one-line summary when a usage section would only repeat it or the docs of the items they contain.
- Crate-private modules and test modules do not need module docs.

Example:

```rust
//! Modal dialog provider — show modal content above the main app UI.
//!
//! ## Usage
//!
//! Show alerts, confirmations and multi-step forms that block the main UI while active.
```

### Component function docs

- Use the following sections in order for each public component function:
  1. `# <component_name>` — function header (title)
  2. Single-line summary: What this component does **and** recommended use cases.
  3. `## Usage` — a short one-line usage scenario (non-placement, app-level).
  4. `## Parameters` — list each parameter and its role. For large parameter sets, keep each
     line concise and group related parameters by behavior when helpful.
  5. `## Examples` — a runnable rustdoc example (no `no_run`, no `ignore`) that demonstrates
     the key state logic (e.g., open/close a dialog or state toggle) and uses `assert!` to
     verify expected behavior.
- `/// ## Usage` must be on its own line and must be followed by a blank `///` line.

Example:

```rust
/// # dialog_provider
///
/// Provide a modal dialog for alerts and confirmations.
///
/// ## Usage
///
/// Show modals (alerts/confirmations/wizards) that block user interaction.
///
/// ## Parameters
///
/// - `controller` — optional external controller for visibility
/// - `on_close_request` — callback invoked when the dialog should close
///
/// ## Examples
/// use tessera_components::md3::dialog::DialogController;
/// let mut controller = DialogController::new(false);
/// assert!(!controller.is_open());
/// controller.open();
/// assert!(controller.is_open());
/// controller.close();
/// assert!(!controller.is_open());
```

### Notes

- Aim for brevity and clarity. Examples should be minimal but assert meaningful behavior.
- Keep parameter descriptions concise. Reference helper/controller types when useful.

---

If there are any changes to the architecture or conventions, this file and the main documentation must be updated accordingly to ensure consistency between documentation and code.
