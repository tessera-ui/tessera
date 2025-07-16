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

- Components are stateless Rust functions annotated with `#[tessera]`. All state must be passed via parameters (e.g., `Arc<Mutex<T>>` or atomic types).
- Inside the component function:
  - `measure`: Custom layout and measurement logic (optional)
  - `state_handler`: Event and interaction handling (optional)
  - All child component closures must be executed to build the complete component tree

**Automatic Injection**: `measure` and `state_handler` are injected by the macro and do not require manual import.

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

---

## 🎯 Event & State Management

- Components are stateless; all state is passed via parameters (prefer `Arc<Mutex<T>>` or atomic types).
- Event handling is done via the `state_handler` closure, which receives `StateHandlerInput` containing:
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
- **Commit Guidelines**:
  - Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0) specification
  - Ensure all tests pass, documentation is updated, and formatting checks are clean before committing
- **Documentation**:
  - Main documentation must be in English. Other language versions, if any, should be placed in the docs/ directory and kept in sync with the English version.
  - Documentation format should pass markdown lint if possible

---

## ⚙️ Special Notes

- **example crate**: `example/Cargo.toml` is intentionally configured with both `[lib]` and `[[bin]]` pointing to `src/lib.rs` for compatibility with both testing and running. The resulting compiler warning is expected—do not remove the `[[bin]]` section.

---

## FAQ

- **Module Path**: Always use `src/module_name.rs`, never `mod.rs`
- **Components must be stateless**; all state is passed via parameters
- **Events must be consumed promptly** to avoid missing or duplicate handling

---

If there are any changes to the architecture or conventions, this file and the main documentation must be updated accordingly to ensure consistency between documentation and code.
