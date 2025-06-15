<div align="center">

# **Tessera(WIP)**

### gui is not special

</div>

## Introduction

Tessera is a functional immediately UI framework designed for Rust. Currently in early development. Take a look at the [`example` crate](example) to see the current progress.

## A Glance

```rust
#[tessera]
fn value_display(app_data: Arc<AppData>) {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .color([0.9, 0.8, 0.8, 1.0]) // Light red fill, RGBA
            .build()
            .unwrap(),
        move || {
            text(
                app_data
                    .click_count
                    .load(atomic::Ordering::SeqCst)
                    .to_string(),
            );
        },
    )
}
```
