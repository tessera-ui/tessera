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

Generated `*Builder` handles can be saved and passed to helpers until the
component commits.

Slots remain deferred content that their parent controls. See
[component calls](../../docs/COMPONENT_CALLS.md) for execution and lifetime details.

## License

This project is licensed under the same terms as the main Tessera framework. See [LICENSE-MIT](https://github.com/tessera-ui/tessera/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/tessera-ui/tessera/blob/main/LICENSE-APACHE) for details.
