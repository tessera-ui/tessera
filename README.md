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
- Standalone basic component library (including `text_input`, `scrollable`, and more)
- Handle-based layout and measurement API
- Cross-platform support(TODO for mobile and platform-specific features)
- Modern performance profiling system

Tessera is an experimental framework. If you encounter any issues, please feel free to [submit an issue](https://github.com/tessera-ui/tessera/issues).

## Overview

- [Try the Web Demo](https://tessera-ui.github.io/tessera/example/)

Components compose stateless `#[tessera]` functions and keep persistent values in
`State<T>` handles created with `remember`. Node-local layout and drawing use
`LayoutModifierNode` and `DrawModifierNode`; layout policies measure and place
direct children through handles. `PxPosition` preserves fractional coordinates
until an integer pixel consumer rounds them. Rendering commands declare their
sampling region separately from their drawing region, so effects can sample a
wider background without expanding their visible bounds.

Remembered resources are dropped when their component leaves the tree. Cleanup
can update other live state; use `State::is_alive` when the owner may already
have been removed or the runtime is shutting down.

`LayoutChild::probe_intrinsic` measures natural content in isolated metadata and
caches, excluding minimum interactive-size padding. Layouts can use it to align
visual dimensions before normal measurement publishes the final placements and
interaction bounds. Intrinsic-dependent layouts remeasure after child-content
changes even when the outer interaction size stays fixed; idle frames retain
the normal layout cache.
Focus-changing popup transitions can defer queued keys through
`FocusManager::defer_keyboard_until_layout`; pending keys resume in order after
layout and focus callbacks, while ordinary typing remains batched.
Global-position callbacks receive the first valid rectangle on attachment,
including when they join an unchanged node with existing observers.
`FocusManager::request_move_focus` schedules traversal through lazy content after
layout, keeping following keys queued until the destination is ready. Explicit
focus requests, capture, and window blur cancel queued traversal.

Background captures for sampled draw passes cover the union of their declared
sampling regions, with full-screen capture for global sampling. This limits
texture bandwidth for small glass surfaces while preserving their padded source
pixels.

`RoundedCorner::relative` combines a fraction of the shortest measured edge
with a density-independent offset, then clamps the radius to fit. Animated
round/square shapes can therefore follow custom bounds without changing layout.

`Modifier::clip_shape` masks content to rounded rectangles or ellipses while
preserving its measured size and fractional placement. Nested masks also apply
to pointer hit testing. Custom UI draw pipelines set
`depth_stencil: tessera_ui::ui_stencil_state()` so their output follows the same
clipping as built-in text and images. The renderer owns the stencil reference;
independent texture-generation passes keep their own attachment configuration.

Accessibility exposes controls, text, and explicit semantic groups directly,
skipping layout and composition wrappers without semantics. This keeps platform
updates small when many controls move together during scrolling.

Pointer queues retain each sample's position and process complete input passes
in event order. Rapid clicks therefore use their original targets even when the
cursor has moved again before the next frame. Custom sources can attach a
position with `PointerChange::with_position`; `CursorState::update_pointer_position`
tracks independent pointer streams. Held touch targets retain their positions
through unrelated input, idle frames, and recomposition.

Tessera uses a declarative programming paradigm inspired by modern UI frameworks such as React and Compose.

We start by declaring a UI component:

```rust
use tessera_ui::tessera;

#[tessera]
fn app() {
    // Component logic
}
```

Then we write its UI logic:

```rust
use tessera_components::{
    button::button,
    column::column,
    surface::surface,
    text::text,
};
use tessera_ui::{Modifier, tessera};

#[tessera]
fn app() {
    surface()
        .modifier(Modifier::new().fill_max_size())
        .child(|| {
            column().children(|| {
                button().filled().on_click(|| {}).child(|| {
                    text().content("+");
                });
                text().content("Count: 0");
                button().filled().on_click(|| {}).child(|| {
                    text().content("-");
                });
            });
        });
}
```

Next, to actually implement the counter we need to use `remember` to store the counter state:

```rust
use tessera_components::{
    button::button,
    column::column,
    surface::surface,
    text::text,
};
use tessera_ui::{Modifier, remember, tessera};

#[tessera]
fn app() {
    let count = remember(|| 0i32);
    surface()
        .modifier(Modifier::new().fill_max_size())
        .child(move || {
            column().children(move || {
                button()
                    .filled()
                    .on_click(move || count.with_mut(|c| *c += 1))
                    .child(|| text().content("+"));
                let label = format!("Count: {}", count.get());
                text().content(label);
                button()
                    .filled()
                    .on_click(move || count.with_mut(|c| *c -= 1))
                    .child(|| text().content("-"));
            });
        });
}
```

This is a complete counter application! For more details, please refer to the [Quick Start Guide](https://tessera-ui.github.io/guide/getting-started.html).

## Contributing

Please read the [Contributing Guide](https://github.com/tessera-ui/tessera/blob/main/CONTRIBUTING.md) to learn how to contribute to the project.

Tests should protect observable framework behavior and meaningful algorithmic
invariants through production code. Use the layout/headless runtime for component
interaction and explicit frame times for animation. See the
[Unit Test Standards](AGENTS.md#unit-test-standards) for test boundaries, review
criteria, and deterministic execution. CI runs crate tests and runnable rustdoc
examples.

Keep expected results independent of the implementation, identify the intended
failure in negative tests, and avoid fixtures that supply the behavior under test.
Run tests that change process-global density in child processes to keep parallel
layout tests isolated.

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
