<div align="center">

<img src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/logo.svg" alt="Tessera Logo" style="width:320px; height:auto;" />

# Tessera

[website](https://tessera-ui.github.io/) · [crates.io](https://crates.io/crates/tessera-ui) · [docs.rs](https://docs.rs/tessera-ui/)

[![简体中文][readme-cn-badge]][readme-cn-url]
[![Stars][stars-badge]][stars-url]
[![CI][ci-badge]][ci-url]
[![tessera ui][tessera-ui-badge]][tessera-ui-url]
[![tessera ui macros][tessera-macros-badge]][tessera-macros-url]
[![tessera ui basic components][tessera-ui-basic-components-badge]][tessera-ui-basic-components-url]
[![tessera ui docs][tessera-ui-docs-badge]][tessera-ui-docs-url]
[![tessera ui macros docs][tessera-ui-macros-docs-badge]][tessera-macros-docs-url]
[![tessera ui basic components docs][tessera-ui-basic-components-docs-badge]][tessera-ui-basic-components-docs-url]

</div>

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/docs/README_zh-CN.md
[stars-badge]: https://img.shields.io/github/stars/tessera-ui/tessera?style=for-the-badge&logo=github
[stars-url]: https://github.com/tessera-ui/tessera
[ci-badge]: https://img.shields.io/github/actions/workflow/status/tessera-ui/tessera/ci.yml?style=for-the-badge&label=CI&logo=githubactions
[ci-url]: https://github.com/tessera-ui/tessera/actions/workflows/ci.yml
[tessera-ui-badge]: https://img.shields.io/badge/tessera%20ui-source-blue?style=for-the-badge&logo=rust
[tessera-ui-docs-badge]: https://img.shields.io/badge/docs%20(ci)-tessera%20ui-blue.svg?style=for-the-badge&logo=docsdotrs
[tessera-ui-docs-url]: https://tessera-ui.github.io/tessera/tessera_ui
[tessera-ui-macros-docs-badge]: https://img.shields.io/badge/docs%20(ci)-tessera%20ui%20macros-blue.svg?style=for-the-badge&logo=docsdotrs
[tessera-macros-docs-url]: https://tessera-ui.github.io/tessera/tessera_ui_macros
[tessera-ui-basic-components-docs-badge]: https://img.shields.io/badge/docs%20(ci)-tessera%20ui%20basic%20components-blue.svg?style=for-the-badge&logo=docsdotrs
[tessera-ui-basic-components-docs-url]: https://tessera-ui.github.io/tessera/tessera_ui_basic_components
[tessera-ui-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-ui
[tessera-ui-basic-components-badge]: https://img.shields.io/badge/tessera%20ui%20basic%20components-source-blue?style=for-the-badge&logo=rust
[tessera-ui-basic-components-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-ui-basic-components
[tessera-macros-badge]: https://img.shields.io/badge/tessera_macros-source-blue?style=for-the-badge&logo=rust
[tessera-macros-url]: https://github.com/tessera-ui/tessera/blob/main/tessera-ui-macros

## Introduction

Tessera is a declarative, immediate-mode UI framework for Rust. With a functional approach at its core, it aims to provide ultimate performance, flexibility, and extensibility.

## Roadmap

The roadmap is now organized by crate:

### tessera-ui (v2.0.0 roadmap)

- IME support for Android
- API optimization
  - ~~Easier way for `measure_node(s)`~~
  - ~~Easier way for `place_node`~~
  - and more...
- ~~Optimize rendering performance~~

- ~~Design how to provide async API to components~~
- ~~Optimize touch screen adaptation~~
- ~~router~~

### tessera-ui-basic-components (v2.0.0 roadmap)

- Beautify/optimize these components
  - ~~checkbox~~
  - ~~dialog~~
  - ~~slider~~
  - ~~text_editor~~
  - ~~progress~~
  - ~~scrollable~~
- Add these components
  - radio
  - ~~bottom sheet~~
  - ~~tab~~
  - ~~bottom nav bar~~
  - ~~side bar~~

## Core Features

- **Declarative Component Model**: Define and compose components using simple functions with the `#[tessera]` macro, resulting in clean and intuitive code.
- **Powerful and Flexible Layout System**: A constraint-based (`Fixed`, `Wrap`, `Fill`) layout engine, combined with components like `row` and `column` (inspired by Jetpack Compose), makes it easy to implement responsive layouts from simple to complex.

<p align="center">
    <img alt="row alignment showcase" src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/alignment_showcase.png"/>
</p>
<p align="center" style="color: gray;"><em>Example of `row`, viewable in `example/alignment_showcase.rs`</em></p>

- **Pluggable Shader Engine**: Shaders are first-class citizens in Tessera. The core of Tessera doesn't come with built-in drawing primitives like a "brush". Instead, it provides an easy-to-use WGPU rendering/compute pipeline plugin system, offering an experience closer to some game engines. This is intentional, for the following reasons:

  - **The Advent of WGPU**: The emergence of WGPU and WGSL has made shader programming simpler, more efficient, and easily adaptable to mainstream GPU backends. Writing shaders directly is no longer a painful process.
  - **Neumorphism**: In recent years, pure flat design has led to visual fatigue, and more applications are adopting neumorphic design styles. The main difference from the old skeuomorphism of the 2000s is its **hyper-realistic perfection**, which requires many visual effects that are difficult to implement uniformly, such as lighting, shadows, reflections, refractions, bloom, and perspective. Attempting to encapsulate a perfect "brush" to achieve these effects is very difficult and inelegant.
  - **Flexibility**: With custom shaders, we can easily implement advanced effects like custom lighting, shadows, particle systems, etc., without relying on the framework's built-in drawing tools.
  - **GPU Compute**: One of the biggest advantages of WGPU over its predecessors is that compute shaders are first-class citizens. A future-oriented framework should take full advantage of this. By using custom compute shaders, we can perform complex computational tasks such as image processing and physics simulations, which are often unacceptably inefficient to perform on the CPU.

<p align="center">
    <img alt="boxed component showcase with glass effect" src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/fluid_glass_showcase.png">
</p>
<p align="center" style="color: gray;"><em>Using custom shaders instead of a built-in brush allows us to easily achieve advanced glass effects like this. This example can be found in `example/fluid_glass_showcase.rs`.</em></p>

- **Decentralized Component Design**: Thanks to the pluggable rendering pipeline, `tessera` itself does not include any built-in components. While `tessera_basic_components` provides a set of common components, you are free to mix and match or create your own component libraries.
- **Explicit State Management**: Components are stateless. State is passed in explicitly as parameters (usually in the form of `Arc<Lock<State>>` due to the highly parallel design), and interaction logic is handled within the `input_handler` closure, making data flow clear and controllable.
- **Parallelized By Design**: The framework utilizes parallel processing in its core. For example, the size measurement of the component tree uses Rayon for parallel computation to improve the performance of complex UIs.

## A Glance

Here is a simple counter application using `tessera_basic_components` that demonstrates the basic usage of `Tessera`.

```rust
/// Main counter application component
#[tessera]
#[shard]
fn counter_app(#[state] app_state: AppState) {
    let button_state_clone = app_state.button_state.clone();
    let click_count = app_state.click_count.load(atomic::Ordering::Relaxed);
    let app_state_clone = app_state.clone();

    surface(
        SurfaceArgs {
            color: Color::WHITE,
            padding: Dp(25.0),
            ..Default::default()
        },
        None,
        move || {
            row(
                RowArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(move || {
                        button(
                            ButtonArgsBuilder::default()
                                .on_click(Arc::new(move || {
                                    app_state_clone
                                        .click_count
                                        .fetch_add(1, atomic::Ordering::Relaxed);
                                }))
                                .build()
                                .unwrap(),
                            button_state_clone,
                            move || text("click me!"),
                        )
                    });
                    scope.child(move || {
                        text(
                            TextArgsBuilder::default()
                                .text(format!("Count: {click_count}"))
                                .build()
                                .unwrap(),
                        )
                    });
                },
            );
        },
    );
}
```

<p align="center">
    <img alt="counter component example" src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/counter.png"/>
</p>
<p align="center" style="color: gray;"><em>This example can be found in `example/counter.rs`</em></p>

## Core Concepts

1. **Component Model**
   `Tessera` components are regular Rust functions annotated with the `#[tessera]` macro. This macro integrates the component function into the framework's component tree. Inside the function body, you can call `measure` to customize layout logic, measure and place child component functions to build the UI hierarchy, and call `input_handler` to handle user interactions.

   `measure` and `input_handler` are automatically injected into the function context by the `tessera` macro and do not need to be imported.

2. **Layout & Measurement**
   The UI layout is determined during the "measurement" phase. Each component can provide a `measure` closure, in which you can:

   - Measure the size of child components (with constraints).
   - Use `place_node` to determine the position of child components.
   - Return the final size of the current component (`ComputedData`).
     If no `measure` closure is provided, the framework defaults to stacking all child components at `(0, 0)` and setting the container size to the minimum size that envelops all children.

3. **State Management**
   `Tessera` promotes an explicit state management pattern. Components are stateless; they receive shared state via parameters (usually `Arc<T>`). All state changes and event responses are handled within the `input_handler` closure, which makes the data flow unidirectional and predictable.

## Getting Started

Currently we don't provide a scaffolding tool to create new projects (planned for the future). The following uses the `example` crate as a showcase project that runs on Windows, Linux, macOS, and Android.

### Running the Example on Windows / Linux

Make sure you have Rust installed: <https://rustup.rs/>

```bash
# Enter the example directory
cd example
# Run
cargo run
```

### Running the Example on Android

1. **Install xbuild**

   ```bash
   cargo install xbuild
   ```

2. **Run the example**

   ```bash
   # Find your device ID
   x devices
   # Assuming device ID is adb:823c4f8b and architecture is arm64
   x run -p example --arch arm64 --device adb:823c4f8b
   ```

## Getting started with Nix

### Running the Example on Desktop with Nix

```bash
nix develop           # to enter the desktop shell
cargo run -p example  # to build and run the example
```

### Running the Example on Android with Nix

```bash
# Enter the Android shell (includes all android tools and setup)
nix develop

# Find your device ID
x devices

# Assuming device ID is adb:823c4f8b and architecture is arm64
x run -p example --arch arm64 --device adb:823c4f8b
```

## Workspace Structure

Tessera adopts a multi-crate workspace structure:

- **`tessera-ui`**: Framework core, including the component tree, rendering system, runtime, basic types (`Dp`, `Px`), and event handling.
- **`tessera-ui-basic-components`**: Provides a set of ready-to-use UI components (such as `row`, `column`, `text`, `button`, `surface`) and their rendering pipelines.
- **`tessera-ui-macros`**: Contains the `#[tessera]` procedural macro for simplified component definition. [Documentation](tessera-ui-macros/README.md)
- **`example`**: Example project demonstrating framework usage.

## Contributing

Read the [Contributing Guide](https://github.com/tessera-ui/tessera/blob/main/CONTRIBUTING.md) for details on how to contribute to the project.

## Acknowledgements

- [wgpu](https://github.com/gfx-rs/wgpuhttps://github.com/gfx-rs/wgpu), for the powerful graphics API abstraction.
- [winit](https://github.com/rust-windowing/winit), for the cross-platform windowing and event handling.
- [glyphon](https://github.com/grovesNL/glyphon), for the text rendering solution.
- Original logo design by [@ktechhydle](https://github.com/ktechhydle)

## License

Tessera is licensed under either of the [MIT License](LICENSE) or the [Apache License 2.0](LICENSE).

## Star History

<a href="https://www.star-history.com/#tessera-ui/tessera&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
 </picture>
</a>
