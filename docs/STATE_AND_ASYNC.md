# State, models, and asynchronous work

## Remembered state

Use `remember` for state owned by one component call. The state survives
recomposition while its slot remains present and is released when that
component leaves the composition. `remember_with_key` keeps identity stable when
keyed content is reordered.

`State<T>` is a small, copyable handle; it does not own or extend the value's
lifetime. Remembered values may contain `!Send` and `!Sync` data.

## Models

A model holds the data and operations of a screen or feature. Implement
`ModelInit`; fields the UI reads are `State` handles created with `State::new`
in `init`, and other fields are ordinary data changed by the model's methods:

```rust
use tessera_ui::{Dispatcher, Model, ModelInit, State, Task, launch_once, on, spawn, tessera};

struct SearchModel {
    results: State<Vec<String>>,
    cache: HashMap<String, Vec<String>>,
    request: Option<Task>,
}

impl ModelInit for SearchModel {
    fn init() -> Self {
        Self {
            results: State::new(Vec::new()),
            cache: HashMap::new(),
            request: None,
        }
    }
}

impl SearchModel {
    // Replacing the stored task cancels an outdated request.
    fn search(&mut self, query: String) {
        if let Some(hit) = self.cache.get(&query) {
            self.results.set(hit.clone());
            return;
        }
        let results = self.results;
        self.request = Some(spawn(Dispatcher::Io, async move {
            results.set(fetch(query).await);
        }));
    }

    // Asynchronous operations take the handle and access the value between
    // awaits.
    async fn refresh(model: Model<Self>) {
        let found = on(Dispatcher::Io, fetch_all()).await;
        model.with_mut(|model| model.results.set(found));
    }
}

#[tessera]
fn search_panel(model: Model<SearchModel>) {
    let results = model.with(|model| model.results.get());
    launch_once(SearchModel::refresh(model));
    // In a callback: model.with_mut(|model| model.search(query));
}
```

When only the work before the first await changes ordinary fields, a method can
take `&mut self` and return a `'static` future that captures the `State`
handles it updates afterwards:

```rust
fn reload(&mut self) -> impl Future<Output = ()> + 'static {
    self.cache.clear();
    let results = self.results;
    async move { results.set(fetch_all().await) }
}

// launch_once(model.with_mut(SearchModel::reload));
```

`Model<T>` is a copyable handle. `with` and `with_mut` are not tracked; only
reads of `State` fields invalidate components. Keep ordinary fields private and
change them through methods. Like `State`, the handle works on every thread and
is `Send` and `Sync` when `T: Send`, so background tasks can call model methods
directly. Calling `with_mut` on a model inside its own `with` or `with_mut`
panics.

A `Model<T>` component parameter is injected:

- Passing an existing model with the generated setter shares it; its owner
  keeps it alive.
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
directly to `ModelStore::model` (T0006). A `remember` state
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
field, a local variable, or the composition through `launch`.

`spawn(dispatcher, future)` starts a `Send` future:

- `Dispatcher::Ui` runs it on the UI session that spawned it, between frames;
- `Dispatcher::Io` runs it on a thread that may block;
- `Dispatcher::Default` runs it on a pool sized for computation.

`on(dispatcher, future).await` runs a future on another dispatcher and returns
its output. On native targets every dispatcher, including `launch`, provides a
Tokio 1 runtime context, so depend on `tokio = "1"` to use Tokio timers, files,
or sockets, or libraries built on them. On the web, all dispatchers run on the
browser's event loop.

In a component, `launch(key, future)` runs a future on the UI session while the
component call keeps the same key; a new key cancels it and starts the new
future, and leaving the composition cancels it. `launch_once(future)` uses a
constant key. These futures may capture UI-thread-local values.

```rust
#[tessera]
fn search_results(query: String, results: State<Vec<String>>) {
    launch(query.clone(), async move {
        results.set(on(Dispatcher::Io, search(query)).await);
    });
}
```

UI tasks drive animation by awaiting `next_frame()`, which resumes them once
per frame with its timestamp, after queued results and before the build:

```rust
if motion.with(Motion::is_animating) {
    launch_once(async move {
        loop {
            let now = next_frame().await;
            if !motion.with_mut(|motion| motion.advance(now)) {
                break;
            }
        }
    });
}
```

Cancellation stops a future at its next suspension point; it cannot interrupt
synchronous work that has already started.
