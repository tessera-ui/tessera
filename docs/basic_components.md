# Using Basic Components

The `tessera_basic_components` crate provides a set of pre-built, foundational components to help you start building user interfaces quickly. These include layout components like `Row` and `Column`, and content components like `Text` and `TextEditor`.

While you can use these components out-of-the-box, they also serve as excellent examples of how to build your own components.

---

### Case Study: The `Row` Component

The `Row` component is a perfect example of how to implement a custom layout. Its primary job is to arrange its children horizontally, one after the other. This is achieved by providing a custom `MeasureFn`.

Let's look at a simplified version of how `Row` implements its measurement and layout logic.

#### The `MeasureFn` Signature

As described in `core_concepts.md`, the `MeasureFn` has a specific signature. The `Row` component defines a function with this signature to handle its layout.

```rust
// This is the function that will be executed during the measure pass.
fn row_measure_logic(
    node_id: NodeId,
    tree: &ComponentNodeTree,
    parent_constraint: &Constraint,
    children_node_ids: &[NodeId],
    metadatas: &ComponentNodeMetaDatas,
) -> Result<ComputedData, MeasurementError> {
    // ... layout logic goes here ...
}
```

#### The Layout Process

Inside its `MeasureFn`, the `Row` component performs the following steps:

**1. Measure All Children:**
First, it iterates through all of its child nodes and measures them. This is necessary to know the dimensions of each child before they can be arranged. It uses the `measure_nodes` utility function for efficient, parallel measurement.

**2. Calculate Total Size:**
It calculates the total width required by summing the widths of all children. The total height of the row is determined by the height of its tallest child.

```rust
// Simplified logic
let mut total_width: u32 = 0;
let mut max_height: u32 = 0;

for &child_id in children_node_ids {
    let child_size = measure_node(child_id, ...)?; // Measure each child
    total_width += child_size.width;
    max_height = max_height.max(child_size.height);
}
```

**3. Place Children:**
After measuring, it iterates through the children again. This time, it calls `place_node` for each child to set its final position. It maintains a `current_x_offset` to ensure each child is placed immediately to the right of the previous one.

```rust
// Simplified logic
let mut current_x_offset: u32 = 0;

for &child_id in children_node_ids {
    // Position each child at the current horizontal offset
    place_node(
        child_id,
        PxPosition::new(Px::new(current_x_offset as i32), Px::new(0)),
        metadatas,
    );
    
    // Increase the offset by the width of the child that was just placed
    let child_size = metadatas.get(&child_id).unwrap().computed_data.unwrap();
    current_x_offset += child_size.width;
}
```

**4. Return Final Size:**
Finally, the `MeasureFn` returns the calculated total size of the row.

```rust
// Simplified logic
return Ok(ComputedData {
    width: total_width,
    height: max_height,
});
```

This example demonstrates the core pattern for creating any layout component in `tessera`: you must provide a `MeasureFn` that first measures all children, then places them according to your desired layout logic.