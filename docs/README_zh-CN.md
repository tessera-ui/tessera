<div align="center">

<img src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/logo.svg" alt="Tessera Logo" style="width:320px; height:auto;" />

# Tessera

[website](https://tessera-ui.github.io/) · [crates.io](https://crates.io/crates/tessera-ui) · [docs.rs](https://docs.rs/tessera-ui/)

[![English][readme-en-badge]][readme-en-url]
[![Stars][stars-badge]][stars-url]
[![CI][ci-badge]][ci-url]
[![tessera ui][tessera-ui-badge]][tessera-ui-url]
[![tessera ui macros][tessera-macros-badge]][tessera-macros-url]
[![tessera ui basic components][tessera-ui-basic-components-badge]][tessera-ui-basic-components-url]
[![tessera ui docs][tessera-ui-docs-badge]][tessera-ui-docs-url]
[![tessera ui macros docs][tessera-ui-macros-docs-badge]][tessera-macros-docs-url]
[![tessera ui basic components docs][tessera-ui-basic-components-docs-badge]][tessera-ui-basic-components-docs-url]

</div>

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/README.md
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

## 简介

Tessera 是一个为 Rust 设计的声明式、立即模式的 UI 框架。其核心采用函数式设计，旨在提供极致的性能、灵活性和可扩展性。

该项目目前处于早期开发阶段。欢迎通过[示例代码](https://github.com/tessera-ui/tessera/blob/main/example)探索最新进展。

## 核心特性

- **声明式组件模型**：使用 `#[tessera]` 宏，通过简单的函数来定义和组合组件，代码干净直观。
- **强大而灵活的布局系统**：基于约束（`Fixed`、`Wrap`、`Fill`）的布局引擎，结合 `row` 和 `column` 等组件（灵感来自 Jetpack Compose），可以轻松实现从简单到复杂的响应式布局。

<p align="center">
    <img alt="row alignment showcase" src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/alignment_showcase.png"/>
</p>
<p align="center" style="color: gray;"><em>`row` 的示例，可在 `example/alignment_showcase.rs` 中查看</em></p>

- **可插拔的着色器引擎**：在 Tessera 中，着色器是一等公民。Tessera 的核心没有内置“画刷”之类的绘图基元。相反，它提供了一个易于使用的 WGPU 渲染/计算管线插件系统，提供了更接近某些游戏引擎的体验。这是有意为之的，原因如下：

  - **WGPU 的出现**：WGPU 和 WGSL 的出现使着色器编程变得更简单、更高效，并且可以轻松适应主流的 GPU 后端。直接编写着色器不再是一个痛苦的过程。
  - **新拟物风格**：近年来，纯粹的扁平化设计已导致视觉疲劳，越来越多的应用程序开始采用新拟物设计风格。与千禧年旧的拟物化风格的主要区别在于其**超现实的完美感**，这需要许多难以统一实现的视觉效果，例如光照、阴影、反射、折射、辉光和透视。试图封装一个完美的“画刷”来实现这些效果是非常困难且不优雅的。
  - **灵活性**：通过自定义着色器，我们可以轻松实现高级效果，如自定义光照、阴影、粒子系统等，而无需依赖框架内置的绘图工具。
  - **GPU 计算**：WGPU 相对于其前辈的最大优势之一是计算着色器是一等公民。一个面向未来的框架应该充分利用这一点。通过使用自定义计算着色器，我们可以执行复杂的计算任务，例如图像处理和物理模拟，这些任务在 CPU 上执行通常效率低得令人无法接受。

<p align="center">
    <img alt="boxed component showcase with glass effect" src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/fluid_glass_showcase.png"/>
</p>
<p align="center" style="color: gray;"><em>使用自定义着色器代替内置画刷，可以轻松实现类似这样的高级玻璃效果。此示例可在 `example/fluid_glass_showcase.rs` 中找到。</em></p>

- **去中心化的组件设计**：得益于可插拔的渲染管线，`tessera` 本身不包含任何内置组件。虽然 `tessera_basic_components` 提供了一组常用组件，但您可以自由地混合搭配或创建自己的组件库。
- **显式的状态管理**：组件是无状态的。状态作为参数显式传入（由于高度并行的设计，通常以 `Arc<Lock<State>>` 的形式），交互逻辑在 `input_handler` 闭包中处理，使数据流清晰可控。
- **并行化设计**：该框架在其核心部分利用了并行处理。例如，组件树的尺寸测量使用 Rayon 进行并行计算，以提高复杂 UI 的性能。

## 预览

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

下面是[`example`](https://github.com/tessera-ui/tessera/tree/main/example/)的演示视频：

<video src="https://github.com/user-attachments/assets/74c93bd0-0b9b-474d-8237-ad451ca73eb8"></video>

## 相关文档

- [Tessera UI 官网](https://tessera-ui.github.io/)

  这是框架的官网兼主要文档网站，包含快速开始指南、API 文档和教程。

- [docs.rs `tessera_ui`](https://docs.rs/tessera-ui/)

  `tessera_ui` crate 的 API 文档。这是本框架的核心 crate。

- [docs.rs `tessera_ui_basic_components`](https://docs.rs/tessera-ui-basic-components/)

  `tessera_ui_basic_components` crate 的 API 文档。这是一个独立的 crate，提供了官方的基本组件集。

## 开始使用

请参考 [快速开始指南](https://tessera-ui.github.io/zhHans/guide/getting-started.html) 以使用 `Tessera` 创建您的第一个应用程序。

## 工作区结构

Tessera 采用多 crate 的工作区结构：

- **`tessera-ui`**：框架核心，包括组件树、渲染系统、运行时、基本类型（`Dp`、`Px`）和事件处理。
- **`tessera-ui-basic-components`**：提供一组即用型 UI 组件（如 `row`、`column`、`text`、`button`、`surface`）及其渲染管线。
- **`tessera-ui-macros`**：包含 `#[tessera]` 过程宏，简化组件定义。[文档](tessera-ui-macros/docs/README_zh-CN.md)
- **`example`**：示例项目，演示框架用法。

## 贡献

请阅读 [贡献指南](https://github.com/tessera-ui/tessera/blob/main/docs/CONTRIBUTING_zh-CN.md) 了解如何为项目做出贡献。

## 致谢

- [wgpu](https://github.com/gfx-rs/wgpu)，强大的图形 API 抽象层。
- [winit](https://github.com/rust-windowing/winit)，跨平台窗口和事件处理库。
- [glyphon](https://github.com/grovesNL/glyphon)，文本渲染解决方案。
- 原始 Logo 设计由 [@ktechhydle](https://github.com/ktechhydle) 完成。

## 许可证

Tessera 采用 [MIT 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE) 或 [Apache 2.0 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE)双重许可。

## Star History

<a href="https://www.star-history.com/#tessera-ui/tessera&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
 </picture>
</a>
