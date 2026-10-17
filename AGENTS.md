# Instructions

This document defines how You should assist in the Tessera project to ensure code and documentation are consistent with project architecture, style, and best practices.

---

## Language Policy

- **All code, comments, and documentation comments must be written in English**, including rustdoc, commit messages, and PR descriptions. Use other languages only when absolutely necessary for functional clarification.
- **This document itself is a development guideline and should be maintained in English only. No i18n is required.**

---

## 🧠 Project Overview & Structure

- **Project Type**: Rust UI Framework
- **Core Crates**:
  - **tessera-ui**: Framework core (component tree, rendering, runtime, basic types Dp/Px, event handling, etc.)
  - **tessera-foundation**: Shared foundational UI building blocks (alignment, shape definitions, and future common modifier APIs)
  - **tessera-components**: Basic UI components (row, column, text, button, surface, etc.) and their rendering pipelines
  - **tessera-macros**: The `#[tessera]` procedural macro for simplified component definition
  - **example**: Example project demonstrating framework usage

**Module Path Convention**: All modules must use the `src/module_name.rs` pattern. Do not use `src/module_name/mod.rs`.

---

## 🏗️ Core Development Model

### Component Model & #[tessera] Macro

- Components are stateless Rust functions annotated with `#[tessera]`. Persistent UI state is created with `remember` (returned as `State<T>`) and can be passed around as `State<T>`.
- Remembered resource destructors may update other still-live state. Slot cleanup must remove entries and release the slot-table borrow before dropping their values. Resource cleanup must check `State::is_alive` before accessing an owner that may already have been removed; liveness checks return false during runtime thread teardown.
- Inside the component function:
  - Compose child components to build the complete component tree
  - Use `Modifier` chains for node-local behavior such as layout modifiers, input, semantics, drawing, and focus
  - Use the internal layout primitive only inside framework/internal crates when a component must provide a custom layout policy
  - All child component closures must be executed to build the complete component tree

### Component API Pattern

- Truly zero-config components may use `#[tessera] pub fn component()` with no parameters.
- Configurable components must use a single public entrypoint declared with named owned parameters: `#[tessera] pub fn component(foo: Foo, bar: Bar, ...)`.
- Public component calls must use the generated builder syntax: `component().foo(value).bar(value);`.
- Do not introduce public wrapper variants (`*_with_controller`, `*_impl`, or extra public forwarding layers).
- Component parameters must use owned types that satisfy the macro requirements (`Clone`, `Default`, `Send`, `Sync`, and `'static` through the generated props model).
- `#[tessera]` parameter-level `#[default(...)]` is not supported. Do not introduce it in new or migrated code.
- For parameters with optional/default behavior, use `Option<T>` and resolve defaults explicitly in the component body (for example, `unwrap_or`, `unwrap_or_else`, or an explicit `if let Some(...)` fallback).
- Keep truly required parameters as non-`Option` constructor parameters.
- For optional external controllers, use `Option<State<...>>`; when `None`, create internal state with `remember`.
- Callback parameters should use `Callback` / `CallbackWith<...>`.
- Slot parameters should use `RenderSlot` / slot wrappers as needed by signature.
- The parameter type is the single source of truth for the generated builder surface; there is no per-prop helper attribute:
  - `Callback` / `CallbackWith<...>` parameters get a closure setter plus a `<name>_shared(impl Into<...>)` setter.
  - `RenderSlot` / `RenderSlotWith<...>` parameters get a closure setter plus a `<name>_shared(impl Into<...>)` setter.
  - every other parameter gets a plain value setter.
- `#[prop(...)]` accepts only these orthogonal options:
  - `into` — declare the setter (and the required constructor parameter) as `impl Into<T>`.
  - `skip_setter` — do not generate a builder setter for this field.
  - `skip_eq` — exclude this field from the generated props `PartialEq`.
- Do not add `#[prop(skip_setter)]` to `Option<T>` just to keep builder setters "clean". The macro already generates setters that accept `T` and store `Some(T)`.
- Prefer `#[prop(into)]` for public `Option<T>` fields whose inner type has a useful conversion surface.
- Reserve `#[prop(skip_setter)]` for true internal plumbing fields that must not appear in the public builder surface. Do not use it on public authoring parameters that should already be expressible through the macro-generated setters.
- Exception: when a public builder needs a deliberately custom semantic surface (for example, mutually exclusive modes such as `title(...)` vs `label(...)`), use hidden backing fields with `#[prop(skip_setter)]` and expose explicit hand-written semantic setters instead of leaking mechanical field setters.
- `Callback`/`RenderSlot` are immutable handles in practice; do not rely on closure hot-swap semantics.
- If callback behavior depends on changing values, capture `State<T>` (or other stable handles) and read latest values at call time.
- Do not add runtime callback/slot helper wrappers (`callback`, `callback_with`, `render_slot`, `render_slot_with`); construct handles directly via `Callback::new`, `CallbackWith::new`, `RenderSlot::new`, and `RenderSlotWith::new`.

### Tessera Color Checking Design

- Tessera color checking belongs in `cargo-tessera`, not in `tessera-macros` or per-crate `build.rs` scripts.
- `tessera-macros` must remain a local lowering layer only: props/builders, component wrappers, control-flow group guard instrumentation, and runtime call structure generation. It must not read or write color metadata, build global call graphs, or decide whether a call is color-valid.
- `tessera-build` remains a build-time utility crate for assets and related generated bindings. It must not be used as the global color-checking frontend.
- `cargo-tessera` should run color checking before invoking the real `cargo build`, `cargo check`, `cargo run`, `cargo tessera dev`, or platform-specific build command. It should use the resolved Cargo package graph for the selected package, target, features, and profile, not just workspace crates.
- Tessera color is an execution-context color, not a general UI-code marker. The only colored contexts are:
  - free functions annotated with `#[tessera]`, or a project-approved alias that lowers to a Tessera component free function such as `#[shard]`;
  - closures carried directly by `RenderSlot`, `RenderSlotWith`, or generated builder setters for parameters declared as `RenderSlot` / `Option<RenderSlot>` / `RenderSlotWith<...>` (including `impl Into<RenderSlot>` / `impl Into<RenderSlotWith<...>>`).
- All other executable contexts are uncolored by default: ordinary free functions, impl methods, trait methods/default methods, local functions, callbacks, event handlers, async tasks, iterator closures, `Option`/`Result` combinator closures, and arbitrary higher-order function closures.
- Color does not propagate through helpers. A helper called by a Tessera-colored component remains uncolored unless the helper itself is a Tessera-colored free function. A helper that calls a Tessera component or Tessera-only API is invalid; it is not implicitly promoted to Tessera.
- The core rule is one-way: Tessera-colored contexts may call uncolored functions, but uncolored contexts must not call Tessera-colored free functions or Tessera-only APIs.
- Tessera-only APIs such as `remember`, `remember_with_key`, `retain`, `retain_with_key`, `provide_context`, `use_context`, `receive_frame_nanos`, `key`, `RenderSlot::new`, `RenderSlotWith::new`, and internal current-instance/group/slot APIs require a Tessera-colored context. They do not color the ordinary function that calls them; calling them from an uncolored function or closure is invalid.
- Closures do not inherit Tessera color merely because they are written inside a Tessera-colored function. Only a closure passed directly to a recognized Tessera carrier, such as `RenderSlot::new` or a generated render-slot setter, is checked as a Tessera closure.
- `RenderSlot` and `RenderSlotWith` are Tessera functors: their carried closures are delayed Tessera fragments and may only be executed by the Tessera runtime at controlled build/replay boundaries.
- Do not introduce custom closure carriers that can execute Tessera code unless the carrier is explicitly added to the color-checker specification. Unknown higher-order functions must be treated as uncolored boundaries.
- A color-check diagnostic is a false positive only when the reported expression is actually inside a Tessera-colored free-function body or a recognized Tessera carrier closure. Reports for helper functions, ordinary closures, callbacks, iterator combinators, `unwrap_or_else`, `then`, or event handlers are not false positives merely because those constructs appear syntactically inside a component.
- The checker must use the `rust-analyzer` analysis crates as its semantic frontend and must run them as library crates from `cargo-tessera`; it must not depend on an LSP server process, `syn`, heuristic fallback parsing, `rustc_private`, or macro/build-script generated color metadata.
- The checker must fail closed when rust-analyzer cannot resolve semantic information that is needed to classify a Tessera-sensitive call. Diagnostics should report the specific invalid uncolored boundary and call target, for example a Tessera component or `remember` call inside an ordinary closure, rather than saying only that metadata is missing.

### Component Tree & Node Metadata

- The component tree is managed internally by `ComponentTree` and node metadata structures, supporting layout, rendering, and event dispatch.
- Accessibility updates flatten layout and composition nodes without semantics. Preserve explicit semantic nodes, their order, bounds, actions, and focus; synthesize only the root window node.
- Newly attached global-position observers must receive the first valid window rectangle even if another observer already watched the same unchanged node. Existing observers should run only when the rectangle changes.

---

## 📏 Layout & Measurement System

- Use `Constraint` and interval-based `AxisConstraint` to describe size constraints.
- Layout policies measure direct child layout nodes through handle-based APIs such as `MeasureScope`, `LayoutChild`, and `LayoutResult`.
- `LayoutChild::probe_intrinsic` probes natural content dimensions in isolated metadata and caches. It excludes minimum interactive-size padding while preserving visual sizing and content padding. Use normal `measure` to publish final layout and retain interaction bounds; intrinsic probes must not replace normal measurement.
- `place_node` is used to position child nodes.
- Default layout: If no layout policy is attached to the current node, `DefaultLayoutPolicy` stacks children at (0,0), and the container size is the minimal bounding rectangle.

---

## 🎨 Rendering & Pipeline System

- Rendering uses a pluggable architecture:
  - **DrawCommand**: Trait describing renderable objects
  - **DrawablePipeline**/**ComputablePipeline**: GPU rendering/compute logic
  - **PipelineRegistry**: Registers all pipelines at startup
- Components set draw/compute commands through `RenderInput`, `RenderMetadataMut`, and render policies
- The basic components crate provides common pipelines, which must be registered at entry (e.g., `register_pipelines`)

### Shape clipping

- UI draw pipelines must set `depth_stencil: tessera_ui::ui_stencil_state()`.
  The renderer supplies a `Stencil8` attachment and manages its reference value.
  Independent atlas or texture-generation passes may use their own attachments.
- Resolve animated size-relative corners through shared shape definitions so
  fills, outlines, shadows, and clipping use the same measured geometry.
- `Modifier::clip_shape` clips the node and its descendants without changing
  measurement. Shape masks preserve fractional placement; integer scissor bounds
  round outwards. Pointer hit testing must honor the same ancestor shapes.
- Shape masks are uploaded once per frame into a reusable buffer. Stencil targets
  are pooled by dimensions and sample count, and active masks are restored after
  scene-pass boundaries without constraining declared background sampling regions.
- Shape clipping supports up to 255 nested shape masks. `clip_to_bounds` clips do
  not consume stencil levels.

### Barrier System & Performance Optimization

- **SampleRegion**: An enum used by both `DrawCommand` and `ComputeCommand` to declare which part of the previously rendered scene a command samples. `DrawRegion` separately describes where the command draws. This distinction is crucial for effects like blur or glass morphism.
- `Global`: Samples the entire screen. This is expensive as it requires a full-screen texture copy.
- `PaddedLocal(PaddingRect)`: Defines a sampling region relative to the component's bounding box plus padding on each side. The draw region, clip stack, and render target bounds determine the scissor rectangle independently.
- `Absolute(PxRect)`: Samples a specific, absolute region of the screen.
- **Performance Optimization**: Sampled draw passes capture the bounding union of their declared sampling regions. `Global` and explicit scene reads without a declared sampling region retain full-screen capture. Sampling bounds must expand outward at fractional positions and remain independent of draw clipping. Scene compute passes still preserve the full target outside their dispatched regions. **Batching** groups subsequent barrier commands with non-overlapping draw regions into the same render pass, avoiding additional captures. A separate scissor rectangle limits actual drawing for each command.

### WebGPU Resource Usage Guidelines

- Don't: create temporary mapped buffers when updating data. Use `Queue::write_buffer` and `Queue::write_texture` instead. For large generated uploads, recycle staging buffers in a pool.
- Do: group resource bindings by change frequency, starting from the lowest. Put per-frame resources in bind group 0, per-pass resources in bind group 1, and per-material resources in bind group 2 to reduce state changes.
- Don't: create many buffers or textures per frame. Coalesce smaller resources into larger ones (buffer subranges, texture atlases, texture arrays).
- Don't: submit many times per frame. Multiple command buffers per submission are fine, but limit `submit()` calls to a few per frame (for example, 1-5).

---

## 🎯 Event & State Management

- Components are stateless; persistent state is stored via `remember` as `State<T>` and passed via parameters as needed.
- Event handling is attached through modifier-based interaction APIs and other node-local capabilities, using typed closures over the framework input contexts.
- The typed input contexts are `PointerInput`, `KeyboardInput`, and `ImeInput`.
- Treat input contexts as event/consumption surfaces, not generic renderer request bags. Cursor changes, semantics, IME publication, and window-side effects must use their dedicated modifier/controller/session APIs.
- Mouse, keyboard, and IME events are processed via event queues and must be consumed promptly.
- Use `FocusManager::request_move_focus` when traversal may need lazy content to be revealed, including Tab exit after popup focus restoration. This traversal is queued after layout and precedes following keys; explicit focus requests, capture, and window blur cancel it. `move_focus` remains an immediate operation on attached targets.
- Preserve each pointer sample's position when queuing it. Dispatch each sample's
  Initial, Main, and Final passes in order before processing the next sample.
  Captured paths use stable instance identities and survive recomposition;
  runtime reset clears captures. Independent held pointers retain their own
  positions during unrelated events and idle frames.
- Event handlers should be lightweight to avoid blocking the main loop.
- When opening or closing focusable content, use `FocusManager::defer_keyboard_until_layout` to preserve subsequent keys until the new layout and focus callbacks finish. Ordinary text-input updates must not introduce a layout barrier.

---

## 📐 Unit System

- **Dp (Density-independent Pixel)**: Used for public APIs, component sizes, margins, paddings, etc.; automatically adapts to DPI
- **Px (Physical Pixel)**: Used internally for rendering and measurement, for precise pixel-level operations
- The framework automatically converts between Dp and Px based on the screen scale factor. Prefer Dp for development.

---

## 🛠️ Contribution & Style Guidelines

- **Code Style**:
  - Enforce `rustfmt edition 2024` default rules
  - `use` imports must be grouped in four sections (standard library, third-party crates, crate root, submodules), separated by one blank line, sorted alphabetically within each group, and merged by root path
  - All code, comments, and documentation comments must be in English (except for rare functional clarifications)
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
- A test of an IME edit plan does not prove that the plan is applied, composition
  is cleared, a callback runs, or an input event is consumed. Those claims require
  invoking the corresponding production path and observing its effects.
- Test-only accessors may expose production state for observation or arrange a
  fixture. Do not retain obsolete algorithms behind `#[cfg(test)]` solely to keep
  their tests passing, or test transformations implemented only in such helpers.
- Do not supply the behavior the test claims to verify: a fixture that registers
  an animation driver cannot prove the component registers it, and assigning a
  nested animation target cannot prove its owner handles a requested transition.

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
- Do not mutate process environment variables in a multithreaded unit-test
  process. Use a child process with its own environment for environment-dependent
  behavior. Isolate and restore other shared runtime state using test guards.
- Run tests that change process-global density in isolated child processes so
  parallel layout tests cannot observe temporary scale factors.

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

- **example crate**: `example/Cargo.toml` uses `[lib]` pointing to `src/lib.rs` for shared entry logic (including Android) with the name `{package}_lib`, and `[[bin]]` pointing to `src/main.rs` for desktop. `src/lib.rs` must expose `#[tessera_ui::entry] pub fn run() -> EntryPoint`, and `src/main.rs` should only call `{package}_lib::run().run_desktop()` to keep a single startup path and avoid PDB output name collisions.

---

## FAQ

- **Module Path**: Always use `src/module_name.rs`, never `mod.rs`
- **Components must be stateless**; all state is passed via parameters
- **Events must be consumed promptly** to avoid missing or duplicate handling

---

## Component Documentation Guidelines

This project uses a strict, concise doc style for modules and component functions to
improve readability and enable testable examples.

### Module docs

- Module docs must follow this exact 5-line template:
  1. `//! <short description>`
  2. `//!`
  3. `//! ## Usage`
  4. `//!`
  5. `//! <one-line app-level usage scenario>` (e.g., alerts, confirmations, multi-step forms, or interactive controls).
- `//! ## Usage` must be on its own line and must be followed by a blank `//!` line (do not inline the usage sentence on the same line).

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
/// - `is_open` — whether the dialog is currently visible
/// - `on_dismiss` — callback invoked when the dialog should close
///
/// ## Examples
/// use tessera_components::dialog::DialogProviderState;
/// let s = DialogProviderState::new();
/// assert!(!s.is_open());
/// s.open();
/// assert!(s.is_open());
/// s.close();
/// assert!(!s.is_open());
```

### Verification and CI

- Add `cargo test --doc` to the CI pipeline to ensure rustdoc examples compile and run.
- Optionally add a lint script that checks: each `#[tessera] pub fn` has a doc-block
  containing `## Parameters` and `## Examples` and that modules have 2-line headers.

### Notes

- Aim for brevity and clarity. Examples should be minimal but assert meaningful behavior.
- Keep parameter descriptions concise. Reference helper/controller types when useful, but do not
  force public configuration through `Args` types.

---

If there are any changes to the architecture or conventions, this file and the main documentation must be updated accordingly to ensure consistency between documentation and code.
