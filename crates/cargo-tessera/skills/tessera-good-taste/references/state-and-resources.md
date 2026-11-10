# State and resources

- Own local state with `remember` and lift shared state to its common parent. Use `remember_model` for component-scoped business state and tasks; pass the model down or place it in a parent model scope when children must share its lifetime. Shard routes use route-entry model scopes.
- Pass data and controllers through props and report actions through callbacks. Reserve context for themes and inherited visual defaults such as content color, typography, and icon styling.
- Give reorderable content stable business keys. Capture cheap state handles directly in callbacks and read current values when handling actions.
- Read state near its output; use short `with` accesses for large values. Use component controllers for ongoing editing, scrolling, selection, and visibility commands.
- Start owned tasks from actions or explicit initialization. Give background work input snapshots, deliver results through the owning `UiDispatcher`, and accept them for the current request and live destination. Tie cancellation and resource cleanup to the owner.
- Keep one source of mutable UI data and use framework tracking for derived output. Own fonts, caches, connections, and graphics resources through concrete components, services, or runtime sessions.
- Check state liveness when resource cleanup accesses an owner that may already have been removed.
- Drive custom animation with the framework frame clock and stop completed frame subscriptions. Let existing components manage their internal motion.
