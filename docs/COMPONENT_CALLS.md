# Component calls

A component entry registers one use in the current build and returns a typed
configuration handle. Setters update that use while it is Pending. The runtime
finalizes its props at the start of execution and completes the build before
layout and drawing.

```rust
row().children(|| {
    let title = text("Title");
    let description = text("Description");

    description.size(Dp(14.0));
    title.size(Dp(24.0));
});
```

The resulting order is Title, Description. Handles do not affect order.
Chaining remains the usual concise form: `text("Title").color(RED);`.
Ordinary helpers can accept a handle and configure it. Declaration ownership,
control-flow group, key, and stable component identity come from the entry
call, including when configuration happens in another group.

## Checking call boundaries

Run `cargo tessera check -p my-app` to check Tessera execution contexts and then
run Cargo's compilation checks. Tessera components and runtime APIs must be
called inside a component or a recognized render-slot closure. Ordinary helpers,
callbacks, and iterator closures remain uncolored, including inside components.

The checker covers the selected targets and their Tessera-dependent libraries;
unresolved calls fail.

`tessera::redundant_render_slot` warns when a content argument constructs a slot
merely to forward it through a closure whose only operation is `render`.

`tessera::remembered_state_in_model` warns about remembered state stored in a
model; see [Models](STATE_AND_ASYNC.md#models).

`tessera::named_modifier_ext_import` warns when a private `use` binds a Tessera
modifier extension trait (`ModifierExt`, `FocusModifierExt`, `CursorModifierExt`)
to a name that the file and its child module files never mention. Method calls
need only the trait in scope, so import it as `use ...::ModifierExt as _;`.

Lint levels follow rustc semantics through `#[tessera_ui::lint(...)]` and
`-A/-W/-D/--forbid/--force-warn/--cap-lints`; `tessera::all` selects all Tessera
lints. Call-boundary errors cannot be lowered.

## Execution and lifetime

Root build callbacks use the same controlled execution model. The callbacks
accepted by `EntryPoint::new`, `HeadlessRenderer::new`, `testing::UiTest::content`,
and the `Renderer::run` variants are build fragments.

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

Pass content directly as a build closure. Create `RenderSlot` / `RenderSlotWith`
handles only for independent storage, sharing, or forwarding, and pass existing
handles through the matching `_shared` setter. The parent expands the slot
during build, choosing whether, where, and in which context it runs.
An unexpanded slot registers no uses from its closure.

```rust
row().children(|| {
    text("Title");
});
```

A component handle already refers to a registered use. It is not child content.
Calling `text("Title")` as an argument happens before the receiving function runs.

Pass actions directly as ordinary closures that update state or controllers;
subsequent UI construction reflects the result. Create `Callback` / `CallbackWith`
handles only for independent storage, sharing, or forwarding, and pass existing
handles through the matching `_shared` setter.

Props own their values. Stored callbacks and slots require owned `'static`
captures and execute in the owning UI session, so their captures need not be
`Send + Sync`. Keep changing values in `State<T>` and read them when the action
or content runs.

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
use ordinary Rust contexts.

The build closure passed to `key` runs as a controlled Tessera fragment.
Initializer factories remain ordinary Rust closures.
