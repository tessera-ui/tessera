# Composition

- Compose UI in `#[tessera]` functions and RenderSlot closures. Use ordinary helpers for data and configuration.
- Call a component to register one use. Configure it through chaining or a saved handle, including across groups, while it is Pending. Registration fixes its owner, order, and stable identity; runtime execution finalizes props before layout.
- Pass content directly as build closures. Create `RenderSlot` / `RenderSlotWith` handles only for independent storage, sharing, or forwarding; pass existing handles through the matching `_shared` setter. Parents expand slots during build.
- Pass required content at entry, such as `text("Title")`; configure optional props through named setters.
- Use owned props; stored UI callbacks and slot closures need owned, `'static` captures and may remain UI-thread-local.
- Keep component bodies repeatable through rebuild and replay. Express ongoing control through state and controllers.
