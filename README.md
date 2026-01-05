<div align="center">

<img src="https://raw.githubusercontent.com/tessera-ui/tessera/refs/heads/main/assets/logo.svg" alt="Tessera Logo" style="width:320px; height:auto;" />

# Tessera

[![简体中文][readme-cn-badge]][readme-cn-url]
[![doc][doc-badge]][doc-url]
[![Stars][stars-badge]][stars-url]
[![CI][ci-badge]][ci-url]
![License](https://img.shields.io/badge/License-MIT%2FApache%202.0-blue.svg?style=for-the-badge)

Tessera is a cross-platform UI library focused on performance and extensibility.

</div>

[doc-badge]: https://img.shields.io/github/actions/workflow/status/tessera-ui/tessera-ui.github.io/.github/workflows/docs.yml?style=for-the-badge&label=doc
[doc-url]: https://tessera-ui.github.io/
[readme-cn-badge]: https://img.shields.io/badge/README-简体中文-blue.svg?style=for-the-badge
[readme-cn-url]: https://github.com/tessera-ui/tessera/blob/main/docs/README_zh-CN.md
[stars-badge]: https://img.shields.io/github/stars/tessera-ui/tessera?style=for-the-badge
[stars-url]: https://github.com/tessera-ui/tessera
[ci-badge]: https://img.shields.io/github/actions/workflow/status/tessera-ui/tessera/ci.yml?style=for-the-badge&label=CI
[ci-url]: https://github.com/tessera-ui/tessera/actions/workflows/ci.yml

## Features

- Simple, easy-to-use declarative and functional programming model
- Constraint-based layout system
- Achieve any visual effect (native support for custom shaders)
- Standalone basic component library (including `text_editor`, `scrollable`, and more)
- Parallel layout support
- Cross-platform support(TODO for mobile and platform-specific features)
- Modern performance profiling system

Tessera is an experimental framework. If you encounter any issues, please feel free to [submit an issue](https://github.com/tessera-ui/tessera/issues).

## Overview

Tessera uses a declarative programming paradigm inspired by modern UI frameworks such as React and Compose.

We start by declaring a UI component:

```rust
use tessera::tessera;

#[tessera]
fn app() {
    // Component logic
}
```

Then we write its UI logic:

```rust
#[tessera]
fn app() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        || {
            column(ColumnArgs::default(), |scope| {
                scope.child(|| button(ButtonArgs::filled(|| {}), || text("+")));
                scope.child(|| text("count: 0"));
                scope.child(|| button(ButtonArgs::filled(|| {}), || text("-")));
            });
        },
    );
}
```

Next, to actually implement the counter we need to use `remember` to store the counter state:

```rust
#[tessera]
fn app() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        || {
            let count = remember(|| 0);
            column(ColumnArgs::default(), move |scope| {
                scope.child(move || {
                    button(
                        ButtonArgs::filled(move || count.with_mut(|c| *c += 1)),
                        || text("+"),
                    )
                });
                scope.child(move || text(format!("Count: {}", count.get())));
                scope.child(move || {
                    button(
                        ButtonArgs::filled(move || count.with_mut(|c| *c -= 1)),
                        || text("-"),
                    )
                });
            });
        },
    );
}
```

This is a complete counter application! For more details, please refer to the [Quick Start Guide](https://tessera-ui.github.io/guide/getting-started.html).

## Getting Started

Please refer to the [Quick Start Guide](https://tessera-ui.github.io/guide/getting-started.html) to create your first application with `Tessera`.

## Contributing

Please read the [Contributing Guide](https://github.com/tessera-ui/tessera/blob/main/CONTRIBUTING.md) to learn how to contribute to the project.

## Acknowledgements

- [wgpu](https://github.com/gfx-rs/wgpu), a powerful graphics API abstraction layer.
- [winit](https://github.com/rust-windowing/winit), a cross-platform windowing and event handling library.
- [glyphon](https://github.com/grovesNL/glyphon), a text rendering solution.
- Original logo design by [@ktechhydle](https://github.com/ktechhydle).

## License

Tessera is dual-licensed under the [MIT License](https://github.com/tessera-ui/tessera/blob/main/LICENSE-MIT) or the [Apache 2.0 License](https://github.com/tessera-ui/tessera/blob/main/LICENSE-APACHE).

## Star History

<a href="https://www.star-history.com/#tessera-ui/tessera&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=tessera-ui/tessera&type=Date" />
 </picture>
</a>
