<div align="center">

# **Tessera (WIP)**

### Gui Is Not Special

</div>

## Introduction

Tessera is a declarative, immediate-mode UI framework for Rust. With a functional approach at its core, it aims to provide ultimate performance, flexibility, and extensibility.

The project is currently in its early stages of development. Feel free to explore the latest progress through the [example code](https://www.google.com/search?q=example).

## Core Features

- **Declarative Component Model**: Define and compose components using simple functions with the `#[tessera]` macro, resulting in clean and intuitive code.
- **Powerful and Flexible Layout System**: A constraint-based (`Fixed`, `Wrap`, `Fill`) layout engine, combined with built-in components like `Row` and `Column`, makes it easy to implement responsive layouts from simple to complex.
- **Pluggable Rendering Engine**: With the `DrawCommand` and `DrawablePipeline` traits, developers can fully customize rendering logic and easily extend the framework to support new visual effects.
- **Decentralized Component Design**: Thanks to the pluggable rendering pipeline, `tessera` itself does not include any built-in components. While `tessera_basic_components` provides a set of common components, you are free to mix and match or create your own component libraries.
- **Explicit State Management**: Components are stateless. State is passed in explicitly as parameters (usually in the form of `Arc<Lock<State>>` due to the highly parallel design), and interaction logic is handled within the `state_handler` closure, making data flow clear and controllable.
- **Parallelized By Design**: The framework utilizes parallel processing in its core. For example, the size measurement of the component tree uses Rayon for parallel computation to improve the performance of complex UIs.

## A Glance

Here is a simple counter application using `tessera_basic_components` that demonstrates the basic usage of `Tessera`.

```rust
use std::sync::{
    Arc,
    atomic::{self, AtomicU32},
};
use tessera::Renderer;
use tessera_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button::{ButtonArgsBuilder, button},
    row::{RowArgsBuilder, row_ui},
    surface::{RippleState, SurfaceArgs, surface},
    text::{TextArgsBuilder, text},
};
use tessera_macros::tessera;

/// Application State
struct AppState {
    click_count: AtomicU32,
    button_state: Arc<RippleState>,
}

impl AppState {
    fn new() -> Self {
        Self {
            click_count: AtomicU32::new(0),
            button_state: Arc::new(RippleState::new()),
        }
    }
}

/// Main application component
#[tessera]
fn counter_app(app_state: Arc<AppState>) {
    // Use surface as the root container
    surface(SurfaceArgs::default(), None, move || {
        // Use the row_ui! macro to create a horizontal layout
        row_ui![
            RowArgsBuilder::default()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .build()
                .unwrap(),
            // Button component
            move || {
                button(
                    ButtonArgsBuilder::default()
                        .on_click(Arc::new(move || {
                            // Update state
                            app_state
                                .click_count
                                .fetch_add(1, atomic::Ordering::Relaxed);
                        }))
                        .build()
                        .unwrap(),
                    app_state.button_state.clone(),
                    move || text("Click Me!"),
                )
            },
            // Text component to display the count
            move || {
                let count = app_state.click_count.load(atomic::Ordering::Relaxed);
                text(
                    format!("Count: {}", count),
                )
            }
        ];
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create application state
    let app_state = Arc::new(AppState::new());

    // Run the application
    Renderer::run(
        move || {
            counter_app(app_state.clone());
        },
        |app| {
            // Register rendering pipelines for basic components
            tessera_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
```

## Core Concepts

1.  **Component Model**
    `Tessera` components are regular Rust functions annotated with the `#[tessera]` macro. This macro integrates the component function into the framework's component tree. Inside the function body, you can call `measure` to customize layout logic, measure and place child component functions to build the UI hierarchy, and call `state_handler` to handle user interactions.

    `measure` and `state_handler` are automatically injected into the function context by the `tessera` macro and do not need to be imported.

2.  **Layout & Measurement**
    The UI layout is determined during the "measurement" phase. Each component can provide a `measure` closure, in which you can:

    - Measure the size of child components (with constraints).
    - Use `place_node` to determine the position of child components.
    - Return the final size of the current component (`ComputedData`).
      If no `measure` closure is provided, the framework defaults to stacking all child components at `(0, 0)` and setting the container size to the minimum size that envelops all children.

3.  **State Management**
    `Tessera` promotes an explicit state management pattern. Components are stateless; they receive shared state via parameters (usually `Arc<T>`). All state changes and event responses are handled within the `state_handler` closure, which makes the data flow unidirectional and predictable.

## Getting Started

`tessera` is currently in early development, and there is no stable way to create a project yet. The following uses the `example` crate as a showcase project that runs on Windows, Linux, macOS, and Android.

### Running the Example on Windows / Linux

```bash
# Enter the example directory
cd example
# Run
cargo run
```

### Running the Example on Android

1.  **Install xbuild**

    ```bash
    cargo install xbuild
    ```

2.  **Run the example**

    ```bash
    # Find your device ID
    x devices
    # Assuming device ID is adb:823c4f8b and architecture is arm64
    x run -p example --arch arm64 --device adb:823c4f8b
    ```

## Workspace Structure

Tessera uses a multi-crate workspace structure with a clear separation of responsibilities:

- **`tessera`**: The core functionality of the framework, including the component tree, renderer, runtime, basic types (`Dp`, `Px`), and event handling.
- **`tessera_basic_components`**: Provides a set of ready-to-use UI components (like `Row`, `Column`, `Text`, `Button`, `Surface`) and their rendering pipelines.
- **`tessera_macros`**: Contains the `#[tessera]` procedural macro, which greatly simplifies component definition.
- **`example`**: An example project demonstrating how to build applications with the `Tessera` framework.
