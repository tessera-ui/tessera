# State, models, and asynchronous work

## Remembered state

Use `remember` for state owned by one component call. The state survives
recomposition while its slot remains present and is released when that
component leaves the composition. `remember_with_key` keeps identity stable when
keyed content is reordered.

`State<T>` is a small, copyable handle; it does not own or extend the value's
lifetime. Remembered values may contain `!Send` and `!Sync` data.

## Models

A model groups the states of a screen or feature. Implement `ModelInit` and
create its states with `State::new` in `init`:

```rust
use tessera_ui::{Model, ModelInit, State, Task, tessera};

struct SearchModel {
    query: State<String>,
    results: State<Vec<String>>,
    request: State<Option<Task>>,
}

impl ModelInit for SearchModel {
    fn init() -> Self {
        Self {
            query: State::new(String::new()),
            results: State::new(Vec::new()),
            request: State::new(None),
        }
    }
}

#[tessera]
fn search_panel(model: Model<SearchModel>) {
    let query = model.query.get();
}
```

A `Model<T>` component parameter is injected:

- Passing an existing model with the generated setter shares it; the creating
  owner keeps it alive.
- Otherwise `ModelInit::init` creates it. The nearest `ModelStore` provided
  through its `ModelStoreScope` context owns it and returns the same instance to
  every component asking for `Model<T>`. Without a store, the component owns its
  model until it leaves the composition.

Navigation provides one store per route entry, so models below a route live as
long as the route: covered routes keep their models and popping a route closes
them. Route destinations do not carry models.

Ownership follows how a state was created, not where it is stored. `State::new`
belongs to the model being initialized and is released when the model closes;
`cargo tessera check` rejects it outside `ModelInit::init` and closures passed
directly to `Model::new_with` or `ModelStore::model` (T0006). A `remember` state
stored in a model keeps its composition-slot lifetime; `cargo tessera check`
warns about this pattern.

## Threads

`State` has the same methods on every thread. A handle is `Send` and `Sync`
when its value is `Send`, so `!Send` values stay on the UI thread at compile
time.

The UI thread accesses its states directly. Another thread's access waits until
the owning session reaches a frame boundary and then runs with the session
active, so a frame never observes a write midway. Writes publish the usual
invalidation. Because the UI thread serves waiting threads only between frames,
it must not block on a thread that is accessing its states.

## Tasks

A `Task` is a running future that is cancelled when dropped; `Task::detach`
lets it finish on its own. Keep the task where its lifetime belongs: in a model
state, a local variable, or the composition through `launch`.

`spawn(dispatcher, future)` starts a `Send` future:

- `Dispatcher::Main` runs it on the UI session that spawned it, between frames;
- `Dispatcher::Io` runs it on a thread that may block;
- `Dispatcher::Default` runs it on a pool sized for computation.

`on(dispatcher, future).await` runs a future on another dispatcher and returns
its output. On the web, all dispatchers run on the browser's event loop.

In a component, `launch(key, future)` runs a future on the UI session while the
component call keeps the same key; a new key cancels it and starts the new
future, and leaving the composition cancels it. `launch_once(future)` uses a
constant key. These futures may capture UI-thread-local values.

```rust
#[tessera]
fn search_results(model: Model<SearchModel>) {
    let query = model.query.get();
    let results = model.results;
    launch(query.clone(), async move {
        let found = on(Dispatcher::Io, search(query)).await;
        results.set(found);
    });
}
```

Replacing a task stored in a model cancels the previous one, which fits
requests where only the latest result matters:

```rust
let results = model.results;
let task = spawn(Dispatcher::Io, async move {
    results.set(search(query).await);
});
model.request.set(Some(task));
```

Cancellation stops a future at its next suspension point; it cannot interrupt
synchronous work that has already started.
