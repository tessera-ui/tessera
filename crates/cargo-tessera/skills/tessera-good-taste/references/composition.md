# Composition

- Compose UI in `#[tessera]` functions and RenderSlot closures. Use ordinary helpers for data and configuration.
- Call a component to get its configuration handle. Configure it through chaining or a saved handle before rendering.
- Pass content directly as closures. Create `RenderSlot` / `RenderSlotWith` handles only for independent storage, sharing, or forwarding; pass existing handles through the matching `_shared` setter. The parent controls where child content appears.
- Pass required content at entry, such as `text("Title")`; configure optional props through named setters.
- Use owned props; stored UI callbacks and slot closures need owned, `'static` captures and may remain UI-thread-local.
- Keep components declarative. Express ongoing control through state and controllers.
