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
  - **tessera-ui-basic-components**: Basic UI components (row, column, text, button, surface, etc.) and their rendering pipelines
  - **tessera-ui-macros**: The `#[tessera]` procedural macro for simplified component definition
  - **example**: Example project demonstrating framework usage

**Module Path Convention**: All modules must use the `src/module_name.rs` pattern. Do not use `src/module_name/mod.rs`.

---

## 🏗️ Core Development Model

### Component Model & #[tessera] Macro

- Components are stateless Rust functions annotated with `#[tessera]`. Persistent UI state is created with `remember` (returned as `State<T>`) and can be passed around as `State<T>`.
- Inside the component function:
  - `measure`: Custom layout and measurement logic (optional)
  - `input_handler`: Event and interaction handling (optional)
  - All child component closures must be executed to build the complete component tree

**Automatic Injection**: `measure` and `input_handler` are injected by the macro and do not require manual import.

### Component Tree & Node Metadata

- The component tree is managed via structures like `ComponentNode` and `ComponentNodeMetaData`, supporting parallel measurement and rendering.

---

## 📏 Layout & Measurement System

- Use `Constraint` and `DimensionValue` to describe size constraints.
- `measure_nodes` supports parallel measurement of multiple child nodes.
- `place_node` is used to position child nodes.
- Default layout: If `measure` is not called, all child nodes are stacked at (0,0), and the container size is the minimal bounding rectangle.

---

## 🎨 Rendering & Pipeline System

- Rendering uses a pluggable architecture:
  - **DrawCommand**: Trait describing renderable objects
  - **DrawablePipeline**/**ComputablePipeline**: GPU rendering/compute logic
  - **PipelineRegistry**: Registers all pipelines at startup
- Components set draw/compute commands via `ComponentNodeMetaData`
- The basic components crate provides common pipelines, which must be registered at entry (e.g., `register_pipelines`)

### Barrier System & Performance Optimization

- **BarrierRequirement**: An enum used by both `DrawCommand` and `ComputeCommand` to declare if it needs to sample from the previously rendered scene. This is crucial for effects like blur or glass morphism.
- `Global`: Samples the entire screen. This is expensive as it requires a full-screen texture copy.
- `PaddedLocal(PaddingRect)`: Defines a region relative to the component's bounding box plus the padding described by `PaddingRect` on each side. This is used to determine the **scissor rectangle** for the draw pass, limiting GPU work to the relevant area.
- `Absolute(PxRect)`: Samples a specific, absolute region of the screen.
- **Performance Optimization**: When a command requires a barrier, the renderer performs a **full-screen texture copy** to make the background available for sampling. The key optimization is **batching**: subsequent commands that also require a barrier and have non-overlapping draw regions are processed in the same render pass, avoiding additional expensive texture copies. A scissor rectangle is applied to limit the actual drawing area for each command.

---

## 🎯 Event & State Management

- Components are stateless; persistent state is stored via `remember` as `State<T>` and passed via parameters as needed.
- Event handling is done via the `input_handler` closure, which receives `InputHandlerInput` containing:
  - `node_id`, `computed_data`, `cursor_position`
  - `cursor_events`, `keyboard_events`, `ime_events`
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

- **example crate**: `example/Cargo.toml` is intentionally configured with both `[lib]` and `[[bin]]` pointing to `src/lib.rs` for compatibility with both testing and running. The resulting compiler warning is expected—do not remove the `[[bin]]` section.

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
  4. `## Parameters` — list each parameter and its role; if a parameter is an `Args` or
     `State` structure, reference the type (e.g., `see [`DialogProviderArgs`]`) instead of
     listing its fields.
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
/// - `args` — see [`DialogProviderArgs`]
/// - `state` — a clonable [`DialogProviderState`] used to open/close
///
/// ## Examples
/// use tessera_ui_basic_components::dialog::DialogProviderState;
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
- Keep references to `Args` and `State` types to avoid duplicating configuration
  documentation across multiple components.

---

If there are any changes to the architecture or conventions, this file and the main documentation must be updated accordingly to ensure consistency between documentation and code.
