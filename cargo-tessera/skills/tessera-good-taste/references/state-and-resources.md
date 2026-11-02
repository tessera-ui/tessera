# State and resources

- Own local state with `remember`, lift shared state to its common owner, and use context for scoped environment dependencies. Choose retained and route state by their intended lifetime.
- Give reorderable content stable business keys. Capture cheap state handles directly in callbacks and read current values when handling actions.
- Read state near its output; use short `with` accesses for large values. Use component controllers for ongoing editing, scrolling, selection, and visibility commands.
- Start owned tasks from actions or explicit initialization. Tie cancellation and resource cleanup to their owner; accept results for the current request and live destination.
- Check state liveness when resource cleanup accesses an owner that may already have been removed.
- Drive custom animation with the framework frame clock and stop completed frame subscriptions. Let existing components manage their internal motion.
