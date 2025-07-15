# Tessera Macros

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/shadow3aaa/tessera/blob/main/tessera-ui-macros/docs/README_zh-CN.md

The `tessera_macros` crate provides procedural macros for the [Tessera UI framework](https://github.com/shadow3aaa/tessera). Currently, it contains the `#[tessera]` attribute macro, which is essential for creating components in the Tessera framework.

## Overview

The `#[tessera]` macro transforms regular Rust functions into Tessera UI components by automatically integrating them into the framework's component tree and injecting necessary runtime functionality.

## Features

- **Component Integration**: Automatically registers functions as components in the Tessera component tree
- **Runtime Injection**: Provides access to `measure` and `state_handler` functions within component functions
- **Clean Syntax**: Enables declarative component definition with minimal boilerplate
- **Tree Management**: Handles component tree node creation and cleanup automatically

## Usage

### Basic Component Definition

```rust
use tessera_macros::tessera;

#[tessera]
fn my_component() {
    // Your component logic here
    // The macro automatically provides access to:
    // - measure: for custom layout logic
    // - state_handler: for handling user interactions
}
```

### Component with Parameters

```rust
use tessera_macros::tessera;
use std::sync::Arc;

#[tessera]
fn button_component(label: String, on_click: Arc<dyn Fn()>) {
    // Component implementation
    // The macro handles component tree integration
}
```

### Using Measure and State Handler

```rust
use tessera_macros::tessera;
use tessera::{ComputedData, Constraints};

#[tessera]
fn custom_component() {
    // Define custom layout behavior
    measure(Box::new(|_| {
        // Custom measurement logic
        use tessera::{ComputedData, Px};
        Ok(ComputedData {
            width: Px(100),
            height: Px(50),
        })
    }));

    // Handle user interactions
    state_handler(Box::new(|_| {
        // Handle events like clicks, key presses, etc.
    }));
}
```

## How It Works

The `#[tessera]` macro performs the following transformations:

1. **Component Registration**: Adds the function to the component tree with its name
2. **Runtime Access**: Injects code to access the Tessera runtime
3. **Function Injection**: Provides `measure` and `state_handler` functions in the component scope
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
    TesseraRuntime::write().component_tree.add_node(ComponentNode { ... });
    
    // Inject measure and state_handler functions
    let measure = |fun: Box<MeasureFn>| { /* ... */ };
    let state_handler = |fun: Box<StateHandlerFn>| { /* ... */ };
    
    // Execute original function body safely
    let result = {
        let closure = || {
            // Original component logic here
        };
        closure()
    };
    
    // Clean up component tree
    TesseraRuntime::write().component_tree.pop_node();
    
    result
}
```

## Examples

### Simple Counter Component

```rust
use tessera_macros::tessera;
use std::sync::{Arc, atomic::{AtomicI32, Ordering}};

#[tessera]
fn counter_component(count: Arc<AtomicI32>) {
    let current_count = count.load(Ordering::Relaxed);
    
    // Use tessera_basic_components for UI
    button(
        ButtonArgs {
            on_click: Arc::new(move || {
                count.fetch_add(1, Ordering::Relaxed);
            }),
            ..Default::default()
        },
        button_state,
        move || text(format!("Count: {}", current_count)),
    );
}
```

### Custom Layout Component

```rust
use tessera_macros::tessera;
use tessera::{ComputedData, Constraints, Px};

#[tessera]
fn custom_layout() {
    measure(Box::new(|_| {
        // Custom measurement logic
        use tessera::{ComputedData, Px};
        
        Ok(ComputedData {
            width: Px(120),
            height: Px(80),
        })
    }));
    
    // Child components
    text("Hello, World!");
}
```

## Contributing

This crate is part of the larger Tessera project. For contribution guidelines, please refer to the main [Tessera repository](https://github.com/shadow3aaa/tessera).

## License

This project is licensed under the same terms as the main Tessera framework. See the [LICENSE](https://github.com/shadow3aaa/tessera/blob/main/LICENSE) file for details.
