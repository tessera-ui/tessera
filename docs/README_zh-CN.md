<div align="center">

<img src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/logo.svg" alt="Tessera Logo" style="width:320px; height:auto;" />

# Tessera

[![English][readme-en-badge]][readme-en-url]
[![doc][doc-badge]][doc-url]
[![Stars][stars-badge]][stars-url]
[![CI][ci-badge]][ci-url]
![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue.svg?style=for-the-badge)

tessera 是一个专注于性能与可扩展性的跨平台 UI 库。

</div>

[doc-badge]: https://img.shields.io/github/actions/workflow/status/tessera-ui/tessera-ui.github.io/.github/workflows/docs.yml?style=for-the-badge&label=doc
[doc-url]: https://tessera-ui.github.io/
[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/README.md
[stars-badge]: https://img.shields.io/github/stars/tessera-ui/tessera?style=for-the-badge
[stars-url]: https://github.com/tessera-ui/tessera
[ci-badge]: https://img.shields.io/github/actions/workflow/status/tessera-ui/tessera/ci.yml?style=for-the-badge&label=CI
[ci-url]: https://github.com/tessera-ui/tessera/actions/workflows/ci.yml

## 特性

- 简单，易用的声明式、函数式编程模型
- 基于约束的布局系统
- 实现任意视觉效果(自定义着色器的原生支持)
- 独立的基本组件库(包括text_input，scrollable和更多)
- 并行布局支持
- 跨平台支持（TODO：移动端和平台特定功能）
- 现代化性能分析系统

tessera是一个实验性的框架，如果您在使用过程中遇到任何问题，请随时[提交 issue](https://github.com/tessera-ui/tessera/issues)。

## 概览

tessera 采用采用声明式编程范式，设计灵感来自于现代 UI 框架，如 react 和 compose。

我们从声明一个 UI 组件开始：

```rust
use tessera_ui::tessera;

#[tessera]
fn app() {
    // 组件逻辑
}
```

编写它的ui逻辑

```rust
use tessera_components::{
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::Modifier;

#[tessera]
fn app() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        || {
            column(ColumnArgs::default(), |scope| {
                scope.child(|| {
                    button(&ButtonArgs::with_child(ButtonArgs::filled(|| {}), || {
                        text(&TextArgs::from("+"));
                    }));
                });
                scope.child(|| text(&TextArgs::from("Count: 0")));
                scope.child(|| {
                    button(&ButtonArgs::with_child(ButtonArgs::filled(|| {}), || {
                        text(&TextArgs::from("-"));
                    }));
                });
            });
        },
    ));
}
```

下一步，为了实际实现counter我们需要使用remember功能记忆计数器的状态

```rust
use tessera_components::{
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Modifier, remember};

#[tessera]
fn app() {
    let count = remember(|| 0i32);
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(ColumnArgs::default(), move |scope| {
                scope.child(move || {
                    button(&ButtonArgs::with_child(
                        ButtonArgs::filled(move || count.with_mut(|c| *c += 1)),
                        || text(&TextArgs::from("+")),
                    ));
                });
                scope.child(move || {
                    let label = format!("Count: {}", count.get());
                    text(&TextArgs::from(label));
                });
                scope.child(move || {
                    button(&ButtonArgs::with_child(
                        ButtonArgs::filled(move || count.with_mut(|c| *c -= 1)),
                        || text(&TextArgs::from("-")),
                    ));
                });
            });
        },
    ));
}
```

这就是一个完整的计数器应用程序！进一步的细节请参考[快速开始指南](https://tessera-ui.github.io/zhHans/guide/getting-started.html)。

## 贡献

请阅读 [贡献指南](https://github.com/tessera-ui/tessera/blob/main/docs/CONTRIBUTING_zh-CN.md) 了解如何为项目做出贡献。

## 致谢

- [wgpu](https://github.com/gfx-rs/wgpu)，强大的图形 API 抽象层。
- [winit](https://github.com/rust-windowing/winit)，跨平台窗口和事件处理库。
- [glyphon](https://github.com/grovesNL/glyphon)，文本渲染解决方案。
- 原始 Logo 设计由 [@ktechhydle](https://github.com/ktechhydle) 完成。

## 许可证

Tessera 采用 [MIT 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE-MIT) 或 [Apache 2.0 许可证](https://github.com/tessera-ui/tessera/blob/main/LICENSE-APACHE)双重许可。

## Star History

<a href="https://www.star-history.com/#tessera-ui/tessera&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
 </picture>
</a>
