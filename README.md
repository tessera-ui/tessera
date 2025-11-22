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

## Preview

```rust
/// Create a small colored box
#[tessera]
fn small_box(text_content: &str, color: Color) {
    surface(
        SurfaceArgs {
            style: color.into(),
            shape: Shape::RoundedRectangle {
                top_left: RoundedCorner::manual(Dp(25.0), 3.0),
                top_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_right: RoundedCorner::manual(Dp(25.0), 3.0),
                bottom_left: RoundedCorner::manual(Dp(25.0), 3.0),
            },
            padding: Dp(8.0),
            width: DimensionValue::Fixed(Px(40)),
            height: DimensionValue::Fixed(Px(40)),
            ..Default::default()
        },
        None,
        move || {
            text(
                TextArgsBuilder::default()
                    .text(text_content.to_string())
                    .color(Color::WHITE)
                    .size(Dp(12.0))
                    .build()
                    .unwrap(),
            )
        },
    );
}
```

Here is a showcase video of the [`example`](https://github.com/tessera-ui/tessera/tree/main/example):

<video src="https://github.com/user-attachments/assets/74c93bd0-0b9b-474d-8237-ad451ca73eb8"></video>

## Related Documentation

- [Tessera UI Website](https://tessera-ui.github.io/)

  This is the official website and main documentation for the framework, containing the quick start guide, API documentation, and tutorials.

- [docs.rs `tessera_ui`](https://docs.rs/tessera-ui/)

  The API documentation for the `tessera_ui` crate. This is the core crate of the framework.

- [docs.rs `tessera_ui_basic_components`](https://docs.rs/tessera-ui-basic-components/)

  The API documentation for the `tessera_ui_basic_components` crate. This is a separate crate that provides the official set of basic components.

## Getting Started

Please refer to the [Quick Start Guide](https://tessera-ui.github.io/guide/getting-started.html) to create your first application with `Tessera`.

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
