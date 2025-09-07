<div align="center">

<img src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/logo.svg" alt="Tessera Logo" style="width:320px; height:auto;" />

<br />

<p align="center" style="color: gray;"><em>Gui rust in rust</em></p>

<br />

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

## 路线图

路线图现已按 crate 拆分：

### tessera-ui（v2.0.0 路线图）

- android 平台 ime 支持
- 优化 api
  - ~~更简单的 `measure_node(s)` 用法~~
  - ~~更简单的 `place_node` 用法~~
  - and more...
- ~~优化渲染性能~~
- ~~设计如何给出异步 api 到组件~~
- ~~优化触屏适配~~
- ~~router~~

### tessera-ui-basic-components（v2.0.0 路线图）

- 美化/优化这些组件
  - ~~checkbox~~
  - ~~dialog~~
  - ~~slider~~
  - ~~text_editor~~
  - ~~progress~~
  - ~~scrollable~~
- 增加这些组件
  - radio
  - ~~bottom sheet~~
  - ~~tab~~
  - ~~bottom nav bar~~
  - ~~side bar~~

## 与 `egui` 的对比

Tessera 和 `egui` 都是 Rust 生态中优秀的即时模式 UI 框架，但它们的设计哲学和目标用户不同。

| 特性         | Tessera                                                                                                  | `egui`                                                                                             |
| :----------- | :------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------- |
| **核心哲学** | 性能、灵活性和极致的可扩展性。旨在给予开发者对渲染管线的完全控制。                                       | 简洁、易用和快速开发。旨在成为在 Rust 中创建 GUI 的最简单方式。                                    |
| **渲染架构** | **可插拔的着色器引擎**。不提供内置的绘图基元。开发者通过编写自定义的 WGPU 着色器来获得完全控制。         | **内置的 Painter API**。提供一个高层级的 API（`painter.rect`, `painter.circle`），抽象了渲染细节。 |
| **视觉定制** | **极高**。非常适合复杂的、自定义的视觉风格，如超现实的玻璃拟态、自定义光照和粒子效果。                   | **有限**。定制受限于内置的绘图基元和样式选项。                                                     |
| **布局系统** | **基于约束**。一个强大而灵活的系统（`Fixed`, `Wrap`, `Fill`），灵感来自 Jetpack Compose 等现代 UI 框架。 | **简单和顺序化**。基于垂直和水平布局，对于标准 UI 非常直观。                                       |
| **状态管理** | **显式与无状态**。组件是无状态的函数，状态通过 `Arc<Mutex<T>>` 等方式显式传递。                          | **隐式/显式混合**。通过 ID 在内部管理一些 UI 状态（如窗口位置）。                                  |
| **学习曲线** | **中等**。需要理解 GPU 概念和 WGPU 管线才能完整发挥 tessera 的能力，但是直接使用组件库也足够直观         | **非常平缓**。数小时内即可学会。                                                                   |

总结:

- **选择 `egui`**：如果您的首要任务是快速轻松地构建功能性 UI，且不需要深度的视觉定制。
- **选择 Tessera**：如果您需要极致的性能和创作自由，以构建视觉上令人惊艳且高度定制化的用户体验，并且您愿意更接近 GPU 底层进行工作。

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
- **显式的状态管理**：组件是无状态的。状态作为参数显式传入（由于高度并行的设计，通常以 `Arc<Lock<State>>` 的形式），交互逻辑在 `state_handler` 闭包中处理，使数据流清晰可控。
- **并行化设计**：该框架在其核心部分利用了并行处理。例如，组件树的尺寸测量使用 Rayon 进行并行计算，以提高复杂 UI 的性能。

## 快速一览

下面是一个使用 `tessera_basic_components` 的简单计数器应用，展示了 `Tessera` 的基本用法。

```rust
/// 计数器应用主组件
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
<p align="center" style="color: gray;"><em>此示例可在 `example/counter.rs` 中找到</em></p>

## 核心概念

1. **组件模型**
   `Tessera` 组件是使用 `#[tessera]` 宏注解的普通 Rust 函数。该宏将组件函数集成到框架的组件树中。在函数体内，您可以调用 `measure` 来自定义布局逻辑，测量和放置子组件函数来构建 UI 层次结构，并调用 `state_handler` 来处理用户交互。

   `measure` 和 `state_handler` 由 `tessera` 宏自动注入到函数上下文中，无需导入。

2. **布局与测量**
   UI 布局在“测量”阶段确定。每个组件都可以提供一个 `measure` 闭包，在其中您可以：

   - 测量子组件的尺寸（带约束）。
   - 使用 `place_node` 来确定子组件的位置。
   - 返回当前组件的最终尺寸（`ComputedData`）。
     如果未提供 `measure` 闭包，框架默认将所有子组件堆叠在 `(0, 0)` 位置，并将容器尺寸设置为足以包裹所有子组件的最小尺寸。

3. **状态管理**
   `Tessera` 提倡显式的状态管理模式。组件是无状态的；它们通过参数接收共享状态（通常是 `Arc<T>`）。所有状态更改和事件响应都在 `state_handler` 闭包内处理，这使得数据流是单向且可预测的。

## 入门

`tessera` 目前处于早期开发阶段，尚无稳定的方法来创建项目。以下使用 `example` crate 作为一个展示项目，可在 Windows、Linux、macOS 和 Android 上运行。

### 在 Windows / Linux 上运行示例

请确保已安装 Rust：https://rustup.rs/

```bash
# 进入 example 目录
cd example
# 运行
cargo run
```

### 在 Android 上运行示例

1. **安装 xbuild**

   ```bash
   cargo install xbuild
   ```

2. **运行示例**

   ```bash
   # 查找您的设备 ID
   x devices
   # 假设设备 ID 为 adb:823c4f8b，架构为 arm64
   x run -p example --arch arm64 --device adb:823c4f8b
   ```

## 通过 Nix 开始

### 使用 Nix 在桌面环境运行示例

```bash
nix develop           # 进入桌面环境 nix shell
cargo run -p example  # 构建并运行示例
```

### 使用 Nix 在 Android 上运行示例

```bash
# 进入 Android shell（包括所有 android 工具和设置）
nix develop

# 查找您的设备 ID
x devices

# 假设设备 ID 为 adb:823c4f8b，架构为 arm64
x run -p example --arch arm64 --device adb:823c4f8b
```

## 工作区结构

Tessera 采用多 crate 的工作区结构：

- **`tessera-ui`**：框架核心，包括组件树、渲染系统、运行时、基本类型（`Dp`、`Px`）和事件处理。
- **`tessera-ui-basic-components`**：提供一组即用型 UI 组件（如 `row`、`column`、`text`、`button`、`surface`）及其渲染管线。
- **`tessera-ui-macros`**：包含 `#[tessera]` 过程宏，简化组件定义。[文档](tessera-ui-macros/docs/README_zh-CN.md)
- **`example`**：示例项目，演示框架用法。

## 贡献

请阅读 [贡献指南](docs/CONTRIBUTING_zh-CN.md) 了解如何为项目做出贡献。

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
