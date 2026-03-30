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
fn button_component(label: String, on_click: Callback) {
    let _ = (label, on_click);
    // Component implementation
}
```

Public components are called through the generated builder syntax:

```rust
button_component()
    .label("Confirm")
    .on_click(Callback::new(|| {}));
```

## How It Works

The `#[tessera]` macro performs the following transformations:

1. **Component Registration**: Adds the function to the component tree with its name
2. **Runtime Access**: Injects the internal runtime plumbing needed for build and replay
3. **Context Injection**: Establishes internal component context used by Tessera during build and replay
4. **Tree Management**: Handles pushing and popping nodes from the component tree
5. **Error Safety**: Wraps the original function body to prevent early returns from breaking the component tree

### Before Macro Application

```rust
#[tessera]
fn my_component() {
    // Component logic
}
```

### After Macro Application (Conceptual)

```rust
fn my_component() {
    // Establish build context, register a node, and attach replay metadata.
    // Execute the original body inside the managed component scope.
}
```

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
        .on_click(Callback::new(move || count.with_mut(|c| *c += 1)))
        .child(|| {
            let label = format!("Count: {}", count.get());
            text().content(label);
        });
}
```

## Contributing

This crate is part of the larger Tessera project. For contribution guidelines, please refer to the main [Tessera repository](https://github.com/tessera-ui/tessera).

## License

This project is licensed under the same terms as the main Tessera framework. See the [LICENSE](https://github.com/tessera-ui/tessera/blob/main/LICENSE) file for details.
