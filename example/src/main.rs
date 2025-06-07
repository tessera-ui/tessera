use std::sync::{
    Arc,
    atomic::{self, AtomicU64},
};

use tessera::{CursorEventContent, DimensionValue, Renderer};
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    let value = Arc::new(AtomicU64::new(0));
    Renderer::run(|| app(value.clone()))
}

// Header row component with two text items
#[tessera]
fn header_row() {
    row([
        RowItem::fill(Box::new(|| text("Hello, this is tessera")), Some(1.0), None),
        RowItem::fill(
            Box::new(|| text("Hello, this is another tessera")),
            Some(1.0),
            None,
        ),
    ])
}

// Vertical text column component
#[tessera]
fn text_column() {
    column([
        ColumnItem::fill(Box::new(|| text("This is a column")), Some(1.0), None),
        ColumnItem::fill(Box::new(|| text("Another item in column")), Some(1.0), None),
    ])
}

// Content section with header and text column
#[tessera]
fn content_section() {
    surface(
        SurfaceArgsBuilder::default().padding(20.0).build().unwrap(),
        || {
            column([
                ColumnItem::wrap(Box::new(header_row)),
                ColumnItem::wrap(Box::new(text_column)),
            ]);
        },
    )
}

// Value display component
#[tessera]
fn value_display(value: Arc<AtomicU64>) {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .build()
            .unwrap(),
        move || {
            text(value.load(atomic::Ordering::SeqCst).to_string());
        },
    )
}

// Main app component
#[tessera]
fn app(value: Arc<AtomicU64>) {
    {
        let value = value.clone();
        surface(
            SurfaceArgsBuilder::default()
                .color([1.0, 1.0, 1.0])
                .width(DimensionValue::Fill { max: None })
                .height(DimensionValue::Fill { max: None })
                .build()
                .unwrap(),
            move || {
                column([
                    ColumnItem::wrap(Box::new(content_section)),
                    ColumnItem::wrap(Box::new(|| {
                        spacer(
                            SpacerArgsBuilder::default()
                                .height(DimensionValue::Fixed(10))
                                .build()
                                .unwrap(),
                        )
                    })),
                    ColumnItem::wrap(Box::new(|| value_display(value))),
                ]);
            },
        );
    }

    {
        let value = value.clone();
        state_handler(Box::new(move |input| {
            // Handle state changes here, e.g., update UI based on cursor events
            // For this example, we do increment when left mouse button clicked
            let count = input
                .cursor_events
                .iter()
                .filter(|event| {
                    // filter out left release events
                    match &event.content {
                        CursorEventContent::Pressed(key) => match key {
                            tessera::PressKeyEventType::Left => {
                                println!("Left mouse button pressed");
                                true
                            }
                            _ => false,
                        },
                        _ => false,
                    }
                })
                .count();
            if count == 0 {
                return;
            }
            value.fetch_add(count as u64, atomic::Ordering::SeqCst);
        }));
    }
}
