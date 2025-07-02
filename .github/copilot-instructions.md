# GitHub Copilot Instructions

These instructions define how GitHub Copilot should assist with this project. The goal is to ensure consistent, high-quality code generation aligned with our conventions, stack, and best practices.

## 🧠 Project Overview

- **Project Type**: UI Framework
- **Language**: Rust
- **Name**: Tessera - A modern UI framework for Rust

### 📁 Workspace Structure

The `tessera` project uses a multi-crate workspace structure with clear separation of responsibilities:

- **`tessera`**: Core framework functionality including component tree management, rendering, runtime, and basic types (`Dp`, `Px`, cursor/keyboard/scroll states)
- **`tessera_basic_components`**: Ready-to-use UI components (layout: `row`/`column`, content: `text`/`TextEditor`, functional: `spacer`/`surface`) and their associated rendering pipelines.
- **`tessera_macros`**: The `#[tessera]` procedural macro that simplifies component creation

## Tessera Framework Concepts

Understanding these core concepts is essential for effective development with the tessera framework.

### 🏗️ The Component Model

A tessera component is a function annotated with `#[tessera]` that defines UI behavior:

```rust
use tessera_macros::tessera;

#[tessera]
pub fn my_component(
    // Parameters like state, styling, children
    child: impl FnOnce(), // For components that accept children
) {
    // Optional: Define custom layout logic
    measure(Box::new(move |input| {
        // Calculate size, position children, set drawable
        // This runs during the measure pass
        Ok(ComputedData { width, height })
    }));

    // Required: Execute child components (if any)
    child(); // Builds the component tree

    // Optional: Handle user interactions and events
    state_handler(Box::new(move |input| {
        // Respond to clicks, keyboard, scroll events
        // This runs every frame after measure
    }));
}
```

**Key Principles:**

- **Flexible Order**: `measure()`, child execution, and `state_handler()` can be called in any order—the framework controls actual execution timing
- **Child Execution is Mandatory**: Components with children MUST call their child closures to build the component tree
- **Default Behavior**: If you don't call `measure()`, the framework provides sensible defaults (measure children with parent constraints, size as union of children, position children at origin)

### 📏 Layout and Measurement

The `measure` function defines how a component calculates its size and positions its children:

```rust
measure(Box::new(move |input| {
    // 1. Measure children with appropriate constraints
    let child_constraint = Constraint::new(width_behavior, height_behavior);
    let results = measure_nodes(child_nodes, input.tree, input.metadatas);

    // 2. Calculate this component's size based on children
    let total_width = children_widths.sum();
    let max_height = children_heights.max();

    // 3. Position children relative to this component
    let mut x_offset = 0;
    for (child_id, child_size) in children {
        place_node(child_id, PxPosition::new(x_offset, 0), input.metadatas);
        x_offset += child_size.width;
    }

    // 4. Return this component's final size
    Ok(ComputedData { width: total_width, height: max_height })
}));
```

**Default Measurement** (when `measure()` is not called):

- Measures all children with the parent's constraints
- Calculates size as the union (bounding box) of all children
- Positions children at (0, 0) relative to the component
- Works well for simple container components

### 🎨 Pluggable Rendering with DrawCommands and Pipelines

Tessera features a pluggable rendering system. Instead of a fixed set of drawable items, components can issue any `DrawCommand`. These commands are then processed by a corresponding `DrawablePipeline` that is registered with the renderer.

**Core Concepts:**

1.  **`DrawCommand`**: A trait that marks a struct or enum as a drawable item. Components create instances of `DrawCommand` implementors to describe what should be rendered.
2.  **`DrawablePipeline`**: A trait that defines the GPU logic for rendering a *specific* type of `DrawCommand`.
3.  **`PipelineRegistry`**: The central registry where `DrawablePipeline`s are registered at application startup. The renderer uses this registry to dispatch each `DrawCommand` to the correct pipeline.

This architecture allows developers to extend Tessera's rendering capabilities by creating their own commands and pipelines.

**Example: Drawing a Rectangle**

The `tessera_basic_components` crate provides `ShapeCommand` for drawing primitive shapes. Here's how a component would use it to draw a rectangle:

```rust
use tessera_basic_components::pipelines::shape::command::{ShapeCommand, ShadowProps};

measure(Box::new(move |input| {
    // ... layout logic to determine width and height ...

    // Define what to draw using a specific DrawCommand
    let command = ShapeCommand::Rect {
        color: [0.2, 0.5, 0.8, 1.0], // RGBA
        corner_radius: 8.0,
        shadow: Some(ShadowProps {
            color: [0.0, 0.0, 0.0, 0.3],
            offset: [2.0, 2.0],
            smoothness: 0.5,
        }),
    };

    // Apply the command to the component
    if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
        metadata.set_draw_command(Box::new(command));
    }

    Ok(ComputedData { width, height })
}));
```

**Rendering Notes:**

-   A component sets its `DrawCommand` within its `measure` closure.
-   The command is boxed (`Box<dyn DrawCommand>`) to be stored in the component's metadata.
-   Layout containers (like `column` or `row`) often don't have their own `DrawCommand`s, as they are invisible.
-   Before using components from `tessera_basic_components`, you must register their pipelines (e.g., using `pipelines::register_pipelines`) with the `PipelineRegistry`.

### 🎯 Event Handling and State

Tessera uses a **stateless component model**—components don't hold internal state. Instead, use `state_handler` to respond to events:

```rust
#[tessera]
pub fn interactive_button(on_click: Arc<dyn Fn()>, label: String) {
    // ... measure logic for button appearance ...

    state_handler(Box::new(move |input| {
        // Handle mouse clicks
        let clicks = input.cursor_events.iter()
            .filter(|event| matches!(&event.content,
                CursorEventContent::Pressed(PressKeyEventType::Left)))
            .count();

        if clicks > 0 {
            on_click(); // Invoke callback
        }

        // Handle keyboard events
        for key_event in input.keyboard_events.iter() {
            if key_event.state == ElementState::Pressed {
                match key_event.physical_key {
                    PhysicalKey::Code(KeyCode::Enter) => on_click(),
                    PhysicalKey::Code(KeyCode::Space) => on_click(),
                    _ => {}
                }
            }
        }
    }));
}
```

**StateHandlerInput provides:**

- `node_id`: This component's identifier
- `computed_data`: Component size from measure pass
- `cursor_position`: Mouse position relative to component
- `cursor_events`: Mutable list of clicks, scrolls for this frame
- `keyboard_events`: Mutable list of key presses for this frame

**Event Handling Patterns:**

- **Clicks**: Filter `cursor_events` for `Pressed`/`Released` events
- **Scroll**: Check for `Scroll` events with delta values
- **Keyboard**: Match on `physical_key` codes and `ElementState`
- **Animation**: Use `Instant::now()` for time-based updates
- **State Updates**: Use `Arc<Mutex<T>>` or atomics for shared state

**Important Notes:**

- State handlers run every frame after the measure pass
- Events are mutable—you can consume/modify them
- Keep handlers lightweight for performance
- `cursor_position` is relative to component's top-left corner

### 📐 Unit System: Dp vs Px

Tessera uses two measurement units for different purposes:

- **`Dp` (Density-independent Pixel)**: Use for public APIs, component sizes, padding, margins. Scales automatically across different screen DPIs for consistent visual appearance.

- **`Px` (Physical Pixel)**: Used internally by the rendering engine for precise pixel-level calculations. You'll encounter this in measure functions and low-level operations.

The framework automatically converts between `Dp` and `Px` based on screen scale factor. As a developer, primarily work with `Dp` for component dimensions.


## ⚙️ Project-Specific Conventions

This section documents specific configurations and patterns that might seem unusual but are intentional for the project's architecture.

### `example` Crate Build Configuration

-   **File**: `example/Cargo.toml`
-   **Configuration**: This crate is intentionally configured with both a `[lib]` and a `[[bin]]` target pointing to the same `src/lib.rs` file.
-   **Rationale**: This setup is necessary for our specific build and testing workflow, enabling `src/lib.rs` to function both as a library for other examples and as a runnable binary.
-   **Action**: **Do not "fix" this.** The resulting compiler warning is expected and should be ignored. Do not remove the `[[bin]]` section from `example/Cargo.toml`.
