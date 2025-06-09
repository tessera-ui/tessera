use std::{
    sync::{
        Arc,
        atomic::{self, AtomicU64},
    },
    time::Instant,
};

use parking_lot::RwLock;
use tessera::{CursorEventContent, DimensionValue};
use tessera_basic_components::{
    column::{ColumnItem, column},
    row::{RowItem, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

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

#[tessera]
fn perf(last_frame: Arc<RwLock<Instant>>, fps: Arc<AtomicU64>) {
    text(format!("FPS: {}", fps.load(atomic::Ordering::SeqCst)));
    state_handler(Box::new(move |_| {
        let now = Instant::now();
        let mut last_frame = last_frame.write();

        fps.store(
            (1.0 / now.duration_since(*last_frame).as_secs_f32()) as u64,
            atomic::Ordering::SeqCst,
        );
        *last_frame = now;
    }));
}

// Main app component
#[tessera]
pub fn app(value: Arc<AtomicU64>, last_frame: Arc<RwLock<Instant>>, fps: Arc<AtomicU64>) {
    {
        let value = value.clone();
        let last_frame = last_frame.clone();
        let fps = fps.clone();
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
                    ColumnItem::wrap(Box::new(|| perf(last_frame, fps))),
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
