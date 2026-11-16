# Asynchronous pointer input

This document proposes an asynchronous pointer-input API for Tessera. It is a
design proposal, not an implementation contract yet.

## Goals

The API should support gesture detectors with the same observable semantics as
Compose pointer input:

- await pointer events in a selected dispatch pass;
- distinguish consumed and unconsumed changes;
- consume changes during the pass in which they are delivered;
- preserve pointer identity and sampled positions;
- cancel a gesture when movement, consumption, bounds, or node lifetime cancel it;
- cancel the task when its modifier is detached or its key changes.

The API must work with Tessera's existing Initial, Main, and Final passes. It
must not be implemented as a task that receives a copied event after the frame:
that would make `consume()` too late to affect ancestor and descendant handlers.

## Compose correspondence

Compose's gesture detectors are built around `awaitEachGesture` and
`awaitPointerEventScope`:

```kotlin
awaitEachGesture {
    val down = awaitFirstDown()
    val up = waitForUpOrCancellation()
}
```

The corresponding Tessera concepts are:

| Compose | Tessera proposal |
| --- | --- |
| `PointerInputScope` | `PointerInputScope` owned by a modifier capability |
| `awaitPointerEvent(pass)` | `scope.await_pointer_event(pass).await` |
| `awaitFirstDown(...)` | `scope.await_first_down(pass, requirement).await` |
| `waitForUpOrCancellation()` | `scope.wait_for_up_or_cancellation(pass).await` |
| `consume()` | consumption token applied to the current dispatch sample |
| `awaitEachGesture` | `scope.await_each_gesture(...)` helper |

The names are intentionally provisional. The event and cancellation semantics
are the contract; the final Rust names should follow Tessera's public API
conventions.

## Proposed dispatch model

Each attached async pointer capability owns a persistent local task and an event
waiter. During pointer dispatch, the component tree should:

1. enter a node's selected pass;
2. publish a borrowed event view to that node's waiter;
3. poll the waiter task before continuing to the next node;
4. apply consumption made by the task to the live pointer changes;
5. continue normal propagation.

The event view cannot be an independent `Vec<PointerChange>` copy. It needs a
shared event transaction or consumption command set that is committed before
the next handler runs. This is required for parent-child gesture arbitration.

Tasks must be cancelled when the capability is detached. Recomposition with a
new key should cancel the old detector before starting the new one, matching
the lifetime behavior of `launch` and `launch_once`.

## Gesture helpers

`TapRecognizer`, `LongPressRecognizer`, and drag helpers can remain useful as
pure stateful algorithms and test units. Public asynchronous detectors should
compose the pointer scope instead of adding recognizer-specific event-entry
methods such as preview-only or tap-forwarding APIs.

For example, a long press should be expressible as:

```rust,ignore
scope.await_each_gesture(|gesture| async move {
    let down = gesture.await_first_down(PointerEventPass::Initial).await?;
    let result = gesture.await_long_press_or_cancellation(down).await?;
    result.consume_until_up().await;
    Some(result.position())
});
```

This example is illustrative. The final API must make invalid event-lifetime
borrows impossible and must define whether gesture helpers return owned event
snapshots or scoped handles.

## Acceptance tests

The implementation should include tests for:

1. first-down filtering by pass and consumption;
2. same-sample consumption affecting the next handler;
3. child tap ownership suppressing a parent long press;
4. long-press timeout followed by release consumption;
5. movement and scroll cancellation;
6. pointer identity when multiple pointers are active;
7. modifier detachment cancelling a waiting task;
8. sampled positions remaining stable when the cursor moves before dispatch;
9. Initial/Main/Final ordering for one pointer sample;
10. no frame work while no async pointer detector is waiting.

## References

- Compose `SuspendingPointerInputFilter`: the pointer-input coroutine and pass
  dispatch model.
- Compose `TapGestureDetector`: `awaitFirstDown`, gesture cancellation, and
  `awaitEachGesture` behavior.
- Compose `PointerEvent`: pointer changes, consumption, and event passes.
