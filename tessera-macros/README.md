# Tessera Macros

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-macros/docs/README_zh-CN.md

The `tessera_macros` crate provides procedural macros for the [Tessera UI framework](https://github.com/tessera-ui/tessera). It includes `#[tessera]`, `#[derive(Prop)]`, and entry/shard-related macros used by Tessera components.

## Overview

The `#[tessera]` macro transforms regular Rust functions into Tessera UI components by automatically integrating them into the framework's component tree and injecting necessary runtime functionality.

## Features

- **Component Integration**: Automatically registers functions as components in the Tessera component tree
- **Runtime Injection**: Provides access to `layout` and `input_handler` functions within component functions
- **Clean Syntax**: Enables declarative component definition with minimal boilerplate
- **Tree Management**: Handles component tree node creation and cleanup automatically
- **Prop Derive**: Generates `Prop` implementation and ergonomic setters for component args structs

## Usage

### Basic Component Definition

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
    // Your component logic here
    // The macro automatically provides access to:
    // - layout: for custom layout logic
    // - input_handler: for handling user interactions
}
```

### Component with Parameters

```rust
use tessera_macros::tessera;
use tessera_ui::{Callback, Prop};

#[derive(Clone, PartialEq, Prop)]
struct ButtonArgs {
    label: String,
    on_click: Callback,
}

#[tessera]
fn button_component(args: &ButtonArgs) {
    let _ = args;
    // Component implementation
    // The macro handles component tree integration
}
```

## `#[derive(Prop)]` Usage

Use `#[derive(Prop)]` on named-field args structs. It implements `tessera_ui::Prop` and
generates fluent setters.

### Basic derive

```rust
use tessera_ui::{Callback, Prop, RenderSlot};

#[derive(Clone, PartialEq, Prop)]
pub struct CardArgs {
    pub title: String,
    pub subtitle: Option<String>,
    pub on_click: Option<Callback>,
    pub child: Option<RenderSlot>,
}

// Generated examples:
// CardArgs::default().title("Hello".to_string())
// CardArgs::default().subtitle("Optional text".to_string())   // Option<T> auto-wraps to Some(...)
// CardArgs::default().on_click(|| {})                         // closure helper for Callback
// CardArgs::default().on_click_shared(Callback::new(|| {}))   // shared handle helper
// CardArgs::default().child(|| {})                            // closure helper for RenderSlot
```

### Supported `#[prop(...)]` options

Struct-level:

- `crate_path = ...`
- `skip_setter`

Field-level:

- `into`
- `skip_setter`
- `skip_eq`

### Equality behavior

- `prop_eq` compares fields directly.
- Fields marked with `#[prop(skip_eq)]` are excluded from comparison.
- `Arc<T>`/`Rc<T>` fields (and `Option<Arc<T>>`/`Option<Rc<T>>`) are compared by pointer identity (`ptr_eq`).

### Using Layout and Input Handler

```rust
use tessera_macros::tessera;
use tessera_ui::{ComputedData, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px};

#[derive(Clone, PartialEq)]
struct FixedLayout;

impl LayoutSpec for FixedLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: Px(100),
            height: Px(50),
        })
    }
}

#[tessera]
fn custom_component() {
    // Define custom layout behavior
    layout(FixedLayout);

    // Handle user interactions
    input_handler(|_| {
        // Handle events like clicks, key presses, etc.
    });
}
```

## How It Works

The `#[tessera]` macro performs the following transformations:

1. **Component Registration**: Adds the function to the component tree with its name
2. **Runtime Access**: Injects code to access the Tessera runtime
3. **Function Injection**: Provides `layout` and `input_handler` functions in the component scope
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
    // Component tree registration
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.add_node(ComponentNode { ... });
    });
    
    // Inject layout and input_handler functions
    let layout = |spec: impl LayoutSpec| { /* ... */ };
    let input_handler = |fun: impl Fn(InputHandlerInput) + Send + Sync + 'static| { /* ... */ };
    
    // Execute original function body safely
    let result = {
        let closure = || {
            // Original component logic here
        };
        closure()
    };

    // Clean up component tree
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.pop_node();
    });
    
    result
}
```

## Examples

### Simple Counter Component

```rust
use tessera_macros::tessera;
use tessera_ui::remember;
use tessera_components::{
    button::{ButtonArgs, button},
    text::{TextArgs, text},
};

#[tessera]
fn counter_component() {
    let count = remember(|| 0i32);

    button(&ButtonArgs::with_child(
        ButtonArgs::filled(move || count.with_mut(|c| *c += 1)),
        move || {
            let label = format!("Count: {}", count.get());
            text(&TextArgs::from(label));
        },
    ));
}
```

### Custom Layout Component

```rust
use tessera_macros::tessera;
use tessera_components::text::{TextArgs, text};
use tessera_ui::{ComputedData, LayoutInput, LayoutOutput, LayoutSpec, MeasurementError, Px};

#[derive(Clone, PartialEq)]
struct FixedLayout;

impl LayoutSpec for FixedLayout {
    fn measure(
        &self,
        _input: &LayoutInput<'_>,
        _output: &mut LayoutOutput<'_>,
    ) -> Result<ComputedData, MeasurementError> {
        Ok(ComputedData {
            width: Px(120),
            height: Px(80),
        })
    }
}

#[tessera]
fn custom_layout() {
    layout(FixedLayout);

    // Child components
    text(&TextArgs::from("Hello, World!"));
}
```

## Contributing

This crate is part of the larger Tessera project. For contribution guidelines, please refer to the main [Tessera repository](https://github.com/tessera-ui/tessera).

## License

This project is licensed under the same terms as the main Tessera framework. See the [LICENSE](https://github.com/tessera-ui/tessera/blob/main/LICENSE) file for details.
