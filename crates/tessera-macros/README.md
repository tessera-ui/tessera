# Tessera Macros

The `tessera_macros` crate provides procedural macros for the [Tessera UI framework](https://github.com/tessera-ui/tessera). It includes `#[tessera]` plus entry/shard-related macros used by Tessera crates.

## Usage

### Basic Component Definition

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
}
```

### Component with Parameters

```rust
use tessera_macros::tessera;
use tessera_ui::Callback;

#[tessera]
fn button_component(label: Option<String>, on_click: Option<Callback>) {
    let _ = (label, on_click);
}
```

Call a component to register a use, then configure its returned handle:

```rust
button_component()
    .label("Confirm".into())
    .on_click(|| {});
```

## Configuration Handles

Generated `*Builder` handles can be saved and passed to helpers. Dropping a
handle has no effect on the declaration. Setters reject committed and stale
handles. Zero-parameter and parameterized components share this model.

Slots remain deferred content that their parent controls. See
[component calls](../../docs/COMPONENT_CALLS.md) for execution and lifetime details.

## Examples

### Simple Counter Component

```rust
use tessera_macros::tessera;
use tessera_ui::remember;
use tessera_components::{
    button::button,
    text::text,
};

#[tessera]
fn counter_component() {
    let count = remember(|| 0i32);

    button()
        .on_click(move || count.with_mut(|c| *c += 1))
        .child(move || {
            let label = format!("Count: {}", count.get());
            text(label);
        });
}
```

## Contributing

This crate is part of the larger Tessera project. For contribution guidelines, please refer to the main [Tessera repository](https://github.com/tessera-ui/tessera).

## License

This project is licensed under the same terms as the main Tessera framework. See the [LICENSE](https://github.com/tessera-ui/tessera/blob/main/LICENSE) file for details.
