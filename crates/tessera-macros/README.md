# Tessera Macros

The `tessera_macros` crate provides procedural macros for the [Tessera UI framework](https://github.com/tessera-ui/tessera). It includes `#[tessera]` plus entry/shard-related macros used by Tessera crates.

## Usage

In an application depending on `tessera-ui` and `tessera-components`, define a
component and compose it under a theme:

```rust
use tessera_components::md3::{text::text, theme::material_theme};
use tessera_macros::tessera;

#[tessera]
fn greeting(name: String) {
    text(format!("Hello, {name}"));
}

#[tessera]
fn app() {
    material_theme().child(|| greeting("Ada".into()));
}
```

Required values are passed at the component call. Optional properties use named
setters on the returned configuration handle.

## Child content

Save generated builder handles or pass them to helpers to configure components
from reusable functions.

Slots let a parent control where and when child content is shown.

## License

This project is licensed under the same terms as the main Tessera framework. See [LICENSE-MIT](https://github.com/tessera-ui/tessera/blob/main/LICENSE-MIT) and [LICENSE-APACHE](https://github.com/tessera-ui/tessera/blob/main/LICENSE-APACHE) for details.
