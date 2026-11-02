# Composition

- Compose UI in `#[tessera]` functions and RenderSlot closures. Use ordinary helpers for data and configuration.
- Call a component to register one use. Configure it through chaining or a saved handle, including across groups, while it is Pending. Registration fixes its owner, order, and stable identity; runtime execution finalizes props before layout.
- Pass deferred child UI through `RenderSlot` or `RenderSlotWith`. Let the parent choose its expansion position and context.
- Use owned props and the generated named setters. Choose `Callback` / `CallbackWith` for actions and RenderSlot types for content; give stored closures owned, `Send + Sync + 'static` captures.
- Keep component bodies repeatable through rebuild and replay. Express ongoing control through state and controllers.
