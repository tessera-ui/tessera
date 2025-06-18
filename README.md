<div align="center">

# **Tessera(WIP)**

### gui is not special

</div>

## Introduction

Tessera is a functional immediately UI framework designed for Rust. Currently in early development. Take a look at the [`example` crate](example) to see the current progress.

## A Glance

```rust
#[tessera]
fn counter_app(app_data: Arc<AppData>) {
    row([
        // Click button
        || {
            button(
                ButtonArgsBuilder::default()
                    .color([0.2, 0.5, 0.8, 1.0]) // Blue button
                    .corner_radius(8.0)
                    .padding(Dp(12.0))
                    .on_click(Arc::new({
                        let app_data = app_data.clone();
                        move || {
                            app_data.click_count.fetch_add(1, atomic::Ordering::SeqCst);
                        }
                    }))
                    .build()
                    .unwrap(),
                app_data.button_state.clone(),
                || {
                    text(
                        TextArgsBuilder::default()
                            .text("Click Me!".to_string())
                            .color([1.0, 1.0, 1.0, 1.0]) // White text
                            .size(Dp(16.0))
                            .build()
                            .unwrap(),
                    )
                },
            )
        },
        // Counter display
        {
            let app_data = app_data.clone();
            move || {
                surface(
                    SurfaceArgsBuilder::default()
                        .corner_radius(8.0)
                        .color([0.9, 0.9, 0.9, 1.0]) // Light gray background
                        .padding(Dp(12.0))
                        .build()
                        .unwrap(),
                    None,
                    move || {
                        text(
                            TextArgsBuilder::default()
                                .text(format!("Count: {}", 
                                    app_data.click_count.load(atomic::Ordering::SeqCst)))
                                .color([0.2, 0.2, 0.2, 1.0]) // Dark text
                                .size(Dp(16.0))
                                .build()
                                .unwrap(),
                        )
                    },
                )
            }
        },
    ]);
}
```
