# Component calls

A component entry registers one use in the current build and returns a typed
configuration handle. Setters update that use while it is Pending. The runtime
finalizes its props at the start of execution and completes the build before
layout and drawing.

```rust
row().children(|| {
    let title = text();
    let description = text();

    description.content("Description");
    title.content("Title");
});
```

The resulting order is Title, Description. Saving, moving, passing, dropping,
or forgetting a handle leaves that order and the registered uses intact.
Chaining remains the usual concise form: `text().content("Title").color(RED);`.
Ordinary helpers can accept a handle and configure it. Declaration ownership,
control-flow group, key, and stable component identity come from the entry
call, including when configuration happens in another group.

## Execution and lifetime

Each component body is a build fragment. After the body returns, the runtime
expands the uses declared by that fragment in declaration order. Each use
finishes its own fragment and descendants before the following sibling runs.
New declarations extend the active fragment, so nested and dynamically created
content completes in the same build.

`RenderSlot::render` and `RenderSlotWith::render` are controlled build boundaries.
They expand the slot at that position, committing any preceding Pending calls
in the current fragment first. A handle can therefore become committed within
the same body or frame. Every setter checks the individual call's status and
panics after execution has started, after an aborted fragment, or after runtime
reset. A later build creates new handles, even when stable component identity
and remembered state are reused.

Use state and controllers for ongoing interaction across frames. Configuration
handles describe a single build's declaration.

## Deferred child content

RenderSlot types hold build logic. A parent chooses whether to expand that
logic, its position in the tree, and the context available to the content.
An unexpanded slot registers no uses from its closure.

```rust
row().children(|| {
    text().content("Title");
});
```

A component handle already refers to a registered use. It is not child content
and has no implicit conversion to a slot, reattachment, or invocation operation.
Calling `text()` as an argument happens before the receiving function runs.

Props own their values. Stored callbacks and slots require owned captures that
satisfy `Send + Sync + 'static`; keep changing values in `State<T>` and read them
when the action or content runs. Slot closures can end with a component call:
its returned handle is discarded after the declaration has been registered.

## Built-in render setters

Build functions may return a component handle; the return value is discarded
once its use has been registered. This includes function pointers such as
`lazy_column().item(my_item)`, where `my_item` is a zero-parameter component.
Required RenderSlot constructor arguments follow the same rule.

These built-in content arguments carry Tessera build code:

| Owner | Content methods |
| --- | --- |
| Top app bar | `action` |
| Button groups | The content argument of `child` |
| Tab | `title` |
| Lazy lists and grids | `item`, `item_with_key`, `items`, `items_with_key`, `items_from_iter`, `items_from_iter_with_key`, `items_from_iter_values` |
| Lazy lists | `sticky_header`, `sticky_header_with_key` |

Shared variants accept RenderSlot handles. Key providers and action callbacks
use ordinary Rust contexts. The CLI resolves the actual receiver type before
recognizing a built-in content method.

## Runtime and replay

Entries reserve the existing stable logic identity from the declaration's
parent, group, key, and occurrence. A separate unique call token validates
configuration lifetime. Execution restores the captured tree parent, context,
group path, and key before invoking the existing component runner. Pending
storage is released before running configuration code, component bodies, or
destructors that can reenter the runtime.

Plain setters update the stored owned props directly. Closure setters defer
functor creation until execution, using a stable namespace for each declaration
and property. This keeps callback and slot identities independent of setter
order and configuration groups. Their resources belong to the declaring
component, so replaying a configured child cannot sweep caller-owned functors.
Root-entry props belong to the root build scope.

Final props feed the existing equality, subtree reuse, runner, and replay
snapshot machinery. State and slot dependencies retain their original owners;
forward invalidations finish before layout, while writes to boundaries already
executed in that build retain the existing deferred invalidation behavior.
Node completion maintains the existing layout adjacency and structural journals.
The scheduler does not introduce an additional layout or drawing phase.

Generated `*Builder` types now contain configuration tokens rather than pending
execution ownership. Component execution is explicit in the runtime. Guards
still restore build context and clean up resources on normal return or unwind.
