# State, models, and asynchronous work

## Remembered state

Use `remember` for state owned by one component call. The state survives
recomposition while its slot remains present and is released when that
component leaves the composition. `remember_with_key` keeps identity stable when
keyed content is reordered.

Remembered values stay in the UI session and may contain `!Send` and `!Sync`
data. `State<T>` is a small, copyable handle; it does not own or extend the
value's lifetime.

## Model scopes

Use `remember_model()` for business state and tasks that should live as long as
the current component. Pass the model to children that share its state. When a
larger scope is needed, create the model in a common parent and pass it down.
Shard routes create one model per route entry: covered routes keep their models
while on the back stack, and popping a route closes its model.

```rust
#[tessera_ui::tessera]
fn search_panel() {
    let model = tessera_ui::remember_model();
    let query = model.state_with_key("query", String::new);
    query.with_mut(|value| value.push_str("tessera"));
}
```

`Model::new()` creates an explicit scope for a router entry or another
deliberately owned lifetime. Cloning a model shares its lifetime. Closing it, or
dropping its final clone, cancels its tasks and releases its keyed state.

## Updating and reading state asynchronously

`State::update` runs immediately when called in the owning session and queues
the mutation when called from another thread or session. The mutation closure
must be `Send + 'static`, but the stored value does not need to be `Send` or
`Sync`. Applying the mutation publishes normal state invalidation to its UI
readers. A `true` return means the session accepted the message; the state may
still expire before delivery.

```rust
let accepted = count.update(|value| *value += 1);
```

`State::read` asynchronously evaluates a closure on the owning UI session and
returns its result. The closure and result must be `Send`; the state value itself
does not. This read is untracked and does not subscribe the caller to dependency
invalidation. Its value is the snapshot observed when the queued read executes.

```rust
let length = count.read(|value| value.to_string().len()).await?;
```

Use `dispatch(Dispatcher::Io, work)` for blocking IO and
`dispatch(Dispatcher::Compute, work)` for synchronous background computation.
Call `State::update` from the work closure to deliver changes. Cancellation is
best-effort after synchronous work begins; queued state messages still validate
that their target is live.

## Keyed model tasks

Call `Model::launch` from the owning UI session. The call site's source location
identifies the effect slot. Recomposition at that site with an equal key keeps
the task; changing the key invalidates and cancels the previous task before
starting the replacement. Use `ModelTaskContext::update` to prevent a result
from an old key from applying after a replacement has started.

```rust
model.launch(Dispatcher::Io, request_id, move |task| async move {
    let result = load_request().await;
    task.update(result_state, move |state| *state = result);
});
```

Task cancellation is cooperative and cannot stop arbitrary synchronous work.
Always preserve the task context or another request generation check across
await points before delivering a result.
