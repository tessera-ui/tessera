# Layout policies

A layout node arranges its children with a `LayoutPolicy`; a `LayoutModifier`
measures the content it wraps in a modifier chain.

```rust
use tessera_ui::layout::{Stack, layout};

#[tessera_ui::tessera]
fn container() {
    layout(Stack).child(|| {});
}
```

`LayoutPolicy::measure` measures the children and returns the node's size and
placements. Implement `LayoutPolicy::place` to recompute placements from cached
child sizes when only placement inputs changed.

`layout` compares the policy with the one already stored on the node. An equal
policy is discarded; a changed policy replaces the stored value once, so
re-executing a component does not clone or allocate an unchanged policy.
`LayoutNode::draw` treats the node's drawing the same way.

Policies are configuration. If measurement intentionally mutates interior state
that must not affect later comparisons, wrap the policy so the node keeps a
separate measurement copy:

```rust
tessera_ui::layout::layout(tessera_ui::StatefulLayoutPolicy(tessera_ui::layout::Stack))
    .child(|| {});
```

The copy is created when the node is created or its policy changes.
