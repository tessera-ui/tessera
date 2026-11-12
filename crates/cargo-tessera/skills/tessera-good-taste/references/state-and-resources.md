# State and resources

- Own local state with `remember` and lift shared state to its common parent. Put screen or feature business state in a model: implement `ModelInit`, create the fields the UI reads as `State<T>` with `State::new` in `init`, keep other data in private ordinary fields changed by `&mut self` methods, and receive it as a `Model<T>` component parameter. Access it with `model.with` / `model.with_mut`; asynchronous operations take the `Model<Self>` handle. Without an argument the nearest `ModelStore` (one per navigation route) owns the model, otherwise the component does; pass a model explicitly to share it. Create model state only in model initializers.
- Do not store `remember` states in a model; they keep their slot lifetime.
- Pass data and controllers through props and report actions through callbacks. Reserve context for themes and inherited visual defaults such as content color, typography, and icon styling.
- Give reorderable content stable business keys. Capture cheap state handles directly in callbacks and read current values when handling actions.
- Read state near its output; use short `with` accesses for large values. Use component controllers for ongoing editing, scrolling, selection, and visibility commands.
- Tie composition-driven work to a key with `launch(key, future)` or `launch_once`. Start other work with `spawn(dispatcher, future)` and keep the returned `Task` where its lifetime belongs, such as a model field; dropping it cancels the work. Use `on(Dispatcher::Io, ...)` for blocking calls. States work the same on every thread, so set results directly; never block the UI thread on work that touches its states.
- Keep one source of mutable UI data and use framework tracking for derived output. Own fonts, caches, connections, and graphics resources through concrete components, services, or runtime sessions.
- Check state liveness when resource cleanup accesses an owner that may already have been removed.
- Drive custom animation from a UI task: `launch` a loop that awaits `next_frame()` and returns when the motion settles. Let existing components manage their internal motion.
