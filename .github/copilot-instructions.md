# GitHub Copilot Instructions

These instructions define how GitHub Copilot should assist with this project. The goal is to ensure consistent, high-quality code generation aligned with our conventions, stack, and best practices.

## 🧠 Project Overview

- **Project Type**: UI Framework
- **Language**: Rust
- **Name**: Tessera - A modern UI framework for Rust

### 📁 Workspace Structure

The `tessera` project uses a multi-crate workspace structure with clear separation of responsibilities:

- **`tessera`**: Core framework functionality including component tree management, rendering, runtime, and basic types (`Dp`, `Px`, cursor/keyboard/scroll states)
- **`tessera_basic_components`**: Ready-to-use UI components (layout: `row`/`column`, content: `Text`/`TextEditor`, functional: `spacer`/`surface`)
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

### 🎨 Visual Rendering with BasicDrawable

Components can render visual elements by setting a `BasicDrawable` in their `measure` function:

```rust
measure(Box::new(move |input| {
    // ... layout logic ...

    // Define what to draw
    let drawable = BasicDrawable::Rect {
        color: [0.2, 0.5, 0.8, 1.0], // RGBA
        corner_radius: 8.0,
        shadow: Some(ShadowProps {
            color: [0.0, 0.0, 0.0, 0.3],
            offset: [2.0, 2.0],
            smoothness: 0.5,
        }),
    };

    // Apply drawable to component
    if let Some(mut metadata) = input.metadatas.get_mut(&input.current_node_id) {
        metadata.basic_drawable = Some(drawable);
    }

    Ok(ComputedData { width, height })
}));
```

**Available BasicDrawable Types:**

1. **`Rect`** - Filled rectangle with color, corner radius, optional shadow
2. **`OutlinedRect`** - Border-only rectangle with configurable border width
3. **`Text`** - Rendered text with font size, color, and text constraints

**Rendering Notes:**

- Set drawable in the same `measure` closure where you calculate size
- Layout containers often don't need drawables (invisible)
- Framework converts `BasicDrawable` to `DrawCommand`s using final position/size

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
