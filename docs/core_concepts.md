# Core Concepts in Tessera

This document details the fundamental concepts and design principles of the `tessera` framework. Understanding these concepts is crucial for effective development.

---

### The Component Model

In `tessera`, the entire UI is composed of **components**, which are simply Rust functions marked with the `#[tessera]` attribute from the `tessera_macros` crate.

```rust
use tessera::prelude::*;
use tessera_macros::tessera;

#[tessera]
fn MyComponent() {
    // ... component logic ...
}
```

The `#[tessera]` macro transforms the function into a component that can be integrated into the framework's component tree.

---

### The Layout System

`tessera` employs a two-phase layout system to determine the size and position of every component on the screen. This process happens on every frame.

#### 1. The Measure Pass

The first phase is the "measure pass." The framework walks the component tree from top to bottom, and for each component, it calls its **`MeasureFn`**.

A `MeasureFn` is a function you provide to define how your component should be sized and how its children should be arranged. Its responsibilities are:
1.  **Measure Children**: It must recursively measure any child components it contains.
2.  **Determine Own Size**: Based on the sizes of its children and the constraints passed down from its parent, it must determine its own size.
3.  **Place Children**: It must call the `place_node` function for each child to set the child's position relative to the parent.

If a component is intended to display children, it **must** have a `MeasureFn`.

#### 2. The Draw Pass

After the entire tree has been measured and all relative positions have been set, the framework performs the "draw pass," traversing the tree again to compute the final absolute screen positions and generate the necessary drawing commands for the renderer.

---

### State Management: A Stateless Approach

A core principle of `tessera` is that **components are stateless**. They do not hold any internal state within their own struct or function scope.

All state changes are handled through a **`StateHandlerFn`**. This is an optional function you can provide for a component. It gets called on every frame after the measure pass and receives a single argument: `&StateHandlerInput`.

The `StateHandlerInput` struct contains all the context and events for the current frame, such as:
- `computed_data`: The component's size, as calculated in the measure pass.
- `cursor_position`: The current mouse/cursor position relative to the component.
- `cursor_events`: A list of clicks or other cursor events.
- `keyboard_events`: A list of key presses.

By responding to the data in `StateHandlerInput`, your component can react to user input and other events without ever needing to store state itself. State should be passed in as parameters to your component function.

---

### The Unit System: Dp vs. Px

`tessera` uses two different units for measurements: `Dp` and `Px`.

#### `Dp` (Density-independent Pixel)

- **Purpose**: This is the primary unit for the **public-facing API**. When you define the size, padding, or margins of your components, you should always use `Dp`.
- **Benefit**: `Dp` ensures that your UI scales correctly across displays with different resolutions and DPI (dots per inch) settings, providing a consistent user experience.

#### `Px` and `PxPosition` (Physical Pixel)

- **Purpose**: This unit is used by the **internal rendering engine**. It represents the actual physical pixels on the screen. `PxPosition` is used for all internal coordinate and position calculations.
- **Benefit**: Using physical pixels internally allows for precise, unambiguous layout calculations and rendering.

The framework automatically handles the conversion from `Dp` to `Px` during the measure and draw passes based on the screen's scale factor. As a developer, you should primarily work with `Dp`.