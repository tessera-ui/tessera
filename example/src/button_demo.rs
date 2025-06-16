use std::sync::{Arc, atomic};
use tessera::Dp;
use tessera::{DimensionValue, Px};
use tessera_basic_components::{
    button::{ButtonArgsBuilder, button},
    column::{ColumnItem, column},
    row::{RowItem, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

pub struct ButtonDemoData {
    pub button_click_count: atomic::AtomicU64,
}

impl ButtonDemoData {
    pub fn new() -> Self {
        Self {
            button_click_count: atomic::AtomicU64::new(0),
        }
    }
}

/// Button demo component with interactive buttons
#[tessera]
pub fn button_demo(data: Arc<ButtonDemoData>) {
    column([
        ColumnItem::wrap(Box::new(|| text("Button Demo:"))),
        ColumnItem::wrap(Box::new({
            let data_clone = data.clone();
            move || {
                row([
                    RowItem::wrap(Box::new({
                        let data_for_button = data_clone.clone();
                        move || {
                            button(
                                ButtonArgsBuilder::default()
                                    .text("Click Me!".to_string())
                                    .color([0.2, 0.7, 0.2, 1.0]) // Green
                                    .padding(Dp(16.0))
                                    .on_click(Arc::new({
                                        let data_for_click = data_for_button.clone();
                                        move || {
                                            data_for_click
                                                .button_click_count
                                                .fetch_add(1, atomic::Ordering::SeqCst);
                                            println!("Button clicked!");
                                        }
                                    }))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    })),
                    RowItem::wrap(Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .width(DimensionValue::Fixed(Px(10)))
                                .build()
                                .unwrap(),
                        )
                    })),
                    RowItem::wrap(Box::new({
                        let data_for_button2 = data_clone.clone();
                        move || {
                            button(
                                ButtonArgsBuilder::default()
                                    .text("Secondary".to_string())
                                    .color([0.6, 0.6, 0.6, 1.0]) // Gray
                                    .text_color([0, 0, 0]) // Black text
                                    .padding(Dp(12.0))
                                    .corner_radius(4.0)
                                    .on_click(Arc::new({
                                        let data_for_click2 = data_for_button2.clone();
                                        move || {
                                            println!("Secondary button clicked!");
                                            data_for_click2
                                                .button_click_count
                                                .fetch_add(1, atomic::Ordering::SeqCst);
                                        }
                                    }))
                                    .build()
                                    .unwrap(),
                            )
                        }
                    })),
                ])
            }
        })),
        ColumnItem::wrap(Box::new({
            let data_for_display = data.clone();
            move || {
                surface(
                    SurfaceArgsBuilder::default()
                        .corner_radius(8.0)
                        .color([0.9, 0.9, 0.7, 1.0]) // Light yellow fill, RGBA
                        .padding(Dp(8.0))
                        .build()
                        .unwrap(),
                    move || {
                        text(format!(
                            "Button clicks: {}",
                            data_for_display
                                .button_click_count
                                .load(atomic::Ordering::SeqCst)
                        ));
                    },
                )
            }
        })),
    ])
}
