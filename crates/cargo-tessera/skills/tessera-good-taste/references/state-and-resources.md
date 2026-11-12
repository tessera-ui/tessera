# State and resources

- Own local state with `remember` and lift shared state to its common parent. Put screen or feature business state in a model: implement `ModelInit` for a struct of `State<T>` fields created with `State::new` in `init`, and receive it as a `Model<T>` component parameter. Without an argument the nearest `ModelStore` (one per navigation route) owns the model, otherwise the component does; pass a model explicitly to share it. Create model state only in model initializers.
- Do not store `remember` states in a model; they keep their slot lifetime.
- Pass data and controllers through props and report actions through callbacks. Reserve context for themes and inherited visual defaults such as content color, typography, and icon styling.
- Give reorderable content stable business keys. Capture cheap state handles directly in callbacks and read current values when handling actions.
- Read state near its output; use short `with` accesses for large values. Use component controllers for ongoing editing, scrolling, selection, and visibility commands.
- Tie composition-driven work to a key with `launch(key, future)` or `launch_once`. Start other work with `spawn(dispatcher, future)` and keep the returned `Task` where its lifetime belongs, such as a model state; dropping it cancels the work. Use `on(Dispatcher::Io, ...)` for blocking calls. States work the same on every thread, so set results directly; never block the UI thread on work that touches its states.
- Keep one source of mutable UI data and use framework tracking for derived output. Own fonts, caches, connections, and graphics resources through concrete components, services, or runtime sessions.
- Check state liveness when resource cleanup accesses an owner that may already have been removed.
- Drive custom animation with the framework frame clock and stop completed frame subscriptions. Let existing components manage their internal motion.
