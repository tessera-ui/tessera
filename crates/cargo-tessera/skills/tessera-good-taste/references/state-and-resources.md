# State and resources

- Own local state with `remember` and lift shared state to its common parent. Put screen or feature business state in a model: implement `ModelInit`, create the fields the UI reads as `State<T>` with `State::new` in `init`, keep other data in private ordinary fields changed by `&mut self` methods, and receive it as a `Model<T>` component parameter. Access it with `model.with` / `model.with_mut`; asynchronous operations take the `Model<Self>` handle. Without an argument the nearest `ModelStore` (one per navigation route) owns the model, otherwise the component does; pass a model explicitly to share it. Create model state only in model initializers.
- Do not store `remember` states in a model; they are owned by their component.
- Pass data and controllers through props and report actions through callbacks. Reserve context for themes and inherited visual defaults such as content color, typography, and icon styling.
- Give reorderable content stable business keys. Capture cheap state handles directly in callbacks and read current values when handling actions.
- Read state near its output; use short `with` accesses for large values. Use component controllers for ongoing editing, scrolling, selection, and visibility commands.
- Tie composition-driven work to a key with `launch(key, future)` or `launch_once`. Start other work with `spawn(dispatcher, future)` and keep the returned `Task` where its lifetime belongs, such as a model field; dropping it cancels the work. Use `on(Dispatcher::Io, ...)` for blocking calls. States work the same on every thread, so set results directly; never block the UI thread on work that touches its states.
- Keep mutable UI data in `State` or a model, and derive the UI from that data. Own external resources through components or services.
- Drive custom animation with `launch` and `with_frame_time`, keyed by its target. Use event timestamps in input handlers and let components manage their own motion.
