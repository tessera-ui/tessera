# Tessera Macros

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-macros/docs/README_zh-CN.md

The `tessera_macros` crate provides procedural macros for the [Tessera UI framework](https://github.com/tessera-ui/tessera). It includes `#[tessera]` plus entry/shard-related macros used by Tessera crates.

## Overview

The `#[tessera]` macro transforms plain Rust functions into Tessera UI components by integrating them into the framework's component tree and establishing the required build/replay context.

## Features

- **Component Integration**: Automatically registers functions as components in the Tessera component tree
- **Runtime Context**: Establishes the component/node context required by the framework
- **Clean Syntax**: Enables declarative component definition with minimal boilerplate
- **Tree Management**: Handles component tree node creation and cleanup automatically

## Usage

### Basic Component Definition

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
    // Your component logic here
}
```

### Component with Parameters

```rust
use tessera_macros::tessera;
use tessera_ui::Callback;

#[tessera]
fn button_component(label: Option<String>, on_click: Option<Callback>) {
    let _ = (label, on_click);
    // Component implementation
}
```

Call a component to register a use, then configure its returned handle:

```rust
button_component()
    .label("Confirm")
    .on_click(|| {});
```

## How It Works

The macro generates an entry, owned props, a typed configuration handle, and a
replay runner. The entry reserves declaration identity and registers a Pending
use. Setters configure that record. The runtime finalizes props and executes the
runner in the captured context, then expands newly declared descendants before
finishing the component scope.

Generated `*Builder` handles can be saved and passed to helpers. Dropping a
handle has no effect on the declaration. Setters reject committed and stale
handles. Zero-parameter and parameterized components share this model.

RenderSlot setters materialize their closures in a stable property scope tied
to the declaration, independently of configuration groups and setter order.
Slots remain deferred content that their parent controls. See
[component calls](../docs/COMPONENT_CALLS.md) for execution and lifetime details.

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
            text().content(label);
        });
}
```

## Contributing

This crate is part of the larger Tessera project. For contribution guidelines, please refer to the main [Tessera repository](https://github.com/tessera-ui/tessera).

## License

This project is licensed under the same terms as the main Tessera framework. See the [LICENSE](https://github.com/tessera-ui/tessera/blob/main/LICENSE) file for details.
