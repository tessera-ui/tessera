# Tessera 组件

[![English][readme-en-badge]][readme-en-url]

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge&logo=readme
[readme-en-url]: https://github.com/tessera-ui/tessera/blob/main/crates/tessera-components/README.md

`tessera-components` 提供 Tessera 的官方组件。crate 根模块包含布局容器、图片等与设计系统无关的基础组件。
每套设计系统各自负责组件、排版、滚动和交互反馈：`md3` 实现 Material Design 3，`experiment::hig` 包含实验性的 Apple Human Interface Guidelines 组件。

## 使用

在应用入口注册组件渲染管线：

```rust
EntryPoint::new(app).package(tessera_components::ComponentsPackage)
```

原生平台使用系统字体；Web 构建内置一个启动字体，并从 CDN 加载其他字体。自定义字体请注册 `Fonts` 包，Web 端也可注册 `FontStylesheet`。

各组件的选项、默认值和示例见 [API 文档](https://docs.rs/tessera-components)。
