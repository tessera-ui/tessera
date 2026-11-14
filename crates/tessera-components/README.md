# Tessera components

[![简体中文][readme-cn-badge]][readme-cn-url]

[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge&logo=readme
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/crates/tessera-components/docs/README_CN.md

`tessera-components` provides Tessera's official components. The crate root
contains layout containers, images, and other primitives that are independent
of a design system. Each design system owns its components, typography,
scrolling, and interaction feedback: `md3` implements Material Design 3, and
`experiment::hig` contains experimental Apple Human Interface Guidelines
components.

## Usage

Register the component pipelines at the application entry:

```rust
EntryPoint::new(app).package(tessera_components::ComponentsPackage)
```

Native text uses system fonts; web builds bundle a startup font and load
additional fonts from a CDN. Register a `Fonts` package, or a `FontStylesheet`
on the web, for custom fonts.

Each component's options, defaults, and examples are documented in the
[API reference](https://docs.rs/tessera-components).
