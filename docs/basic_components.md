# Using Basic Components

The `tessera_basic_components` crate provides a set of pre-built, foundational components to help you start building user interfaces quickly. These include layout components like `Row` and `Column`, and content components like `Text` and `TextEditor`.

While you can use these components out-of-the-box, they also serve as excellent examples of how to build your own components.

---

### Case Study: The `Row` Component

The `Row` component is a perfect example of how to implement a custom layout. Its primary job is to arrange its children horizontally. This is achieved by providing a custom layout logic to the `measure` function, which is made available by the `#[tessera]` macro.

#### The Component Function

A `tessera` component is a simple function annotated with `#[tessera]`. Inside this function, you define its behavior.

```rust
use tessera_macros::tessera;

#[tessera]
pub fn row(
    // ... parameters like children ...
) {
    // 1. Call `measure` to define layout logic
    measure(Box::new(
        move |node_id, tree, parent_constraint, children_node_ids, metadatas| {
            // ... layout logic goes here ...
        }
    ));

    // 2. Call the children closure to render them
    // (children)();
}
```

#### The Layout Process within `measure`

Inside the closure passed to the `measure` function, the `Row` component performs the following steps:

**1. Measure All Children:**
First, it iterates through all of its child nodes and measures them. This is necessary to know the dimensions of each child before they can be arranged. It uses the `measure_nodes` utility function for efficient, parallel measurement.

**2. Calculate Total Size:**
It calculates the total width required by summing the widths of all children. The total height of the row is determined by the height of its tallest child.

```rust
// Simplified logic inside the measure closure
let mut total_width: u32 = 0;
let mut max_height: u32 = 0;

for &child_id in children_node_ids {
    let child_size = tree.measure_node(child_id, ...)?; // Measure each child
    total_width += child_size.width;
    max_height = max_height.max(child_size.height);
}
```

**3. Place Children:**
After measuring, it iterates through the children again. This time, it calls `place_node` for each child to set its final position. It maintains a `current_x_offset` to ensure each child is placed immediately to the right of the previous one.

```rust
// Simplified logic inside the measure closure
let mut current_x_offset: u32 = 0;

for &child_id in children_node_ids {
    // Position each child at the current horizontal offset
    tree.place_node(
        child_id,
        PxPosition::new(Px::new(current_x_offset as i32), Px::new(0)),
    );
    
    // Increase the offset by the width of the child that was just placed
    let child_size = tree.get_meta(child_id).unwrap().computed_data.unwrap();
    current_x_offset += child_size.width;
}
```

**4. Return Final Size:**
Finally, the `measure` closure returns the calculated total size of the row as a `Result<ComputedData, MeasurementError>`.

```rust
// Simplified logic inside the measure closure
return Ok(ComputedData {
    width: total_width,
    height: max_height,
});
```

This example demonstrates the core pattern for creating any layout component in `tessera`: you define a `#[tessera]` function and, within it, call the `measure` function with a closure that contains your layout logic. Similarly, you can call `state_handler` to react to events.