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
- Component parameters must use owned types that satisfy the macro requirements (`Clone`, `Default`, `Send`, `Sync`, and `'static` through the generated props model). Use `Option<T>` when a value has no natural default.
- For optional external controllers, use `Option<State<...>>`; when `None`, create internal state with `remember`.
- Callback parameters should use `Callback` / `CallbackWith<...>`.
- Slot parameters should use `RenderSlot` / slot wrappers as needed by signature.
- Do not add `#[prop(skip_setter)]` to `Option<T>` just to keep builder setters "clean". The macro already generates setters that accept `T` and store `Some(T)`.
- Prefer `#[prop(into)]` for public `Option<T>` fields whose inner type has a useful conversion surface.
- Prefer `#[prop(render_slot)]` for public `RenderSlot` / `Option<RenderSlot>` parameters so the generated builder supports closure-style slot setters directly.
- Reserve `#[prop(skip_setter)]` for true internal plumbing fields that must not appear in the public builder surface. Do not use it on public authoring parameters that should already be expressible through the macro-generated setters.
- Exception: when a public builder needs a deliberately custom semantic surface (for example, mutually exclusive modes such as `title(...)` vs `label(...)`), use hidden backing fields with `#[prop(skip_setter)]` and expose explicit hand-written semantic setters instead of leaking mechanical field setters.
- `Callback`/`RenderSlot` are immutable handles in practice; do not rely on closure hot-swap semantics.
- If callback behavior depends on changing values, capture `State<T>` (or other stable handles) and read latest values at call time.
- Do not add runtime callback/slot helper wrappers (`callback`, `callback_with`, `render_slot`, `render_slot_with`); construct handles directly via `Callback::new`, `CallbackWith::new`, `RenderSlot::new`, and `RenderSlotWith::new`.

### Component Tree & Node Metadata

- The component tree is managed internally by `ComponentTree` and node metadata structures, supporting parallel measurement and rendering.

---

## 📏 Layout & Measurement System

- Use `Constraint` and interval-based `AxisConstraint` to describe size constraints.
- `measure_nodes` supports parallel measurement of multiple child nodes.
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

### Barrier System & Performance Optimization

- **BarrierRequirement**: An enum used by both `DrawCommand` and `ComputeCommand` to declare if it needs to sample from the previously rendered scene. This is crucial for effects like blur or glass morphism.
- `Global`: Samples the entire screen. This is expensive as it requires a full-screen texture copy.
- `PaddedLocal(PaddingRect)`: Defines a region relative to the component's bounding box plus the padding described by `PaddingRect` on each side. This is used to determine the **scissor rectangle** for the draw pass, limiting GPU work to the relevant area.
- `Absolute(PxRect)`: Samples a specific, absolute region of the screen.
- **Performance Optimization**: When a command requires a barrier, the renderer performs a **full-screen texture copy** to make the background available for sampling. The key optimization is **batching**: subsequent commands that also require a barrier and have non-overlapping draw regions are processed in the same render pass, avoiding additional expensive texture copies. A scissor rectangle is applied to limit the actual drawing area for each command.

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
- Event handlers should be lightweight to avoid blocking the main loop.

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
