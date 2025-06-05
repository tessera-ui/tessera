use std::{
    sync::{
        Arc,
        atomic::{self, AtomicU64},
    },
    time::Duration,
};

use tessera::{DimensionValue, Renderer, tokio_runtime};
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
    let random_value = Arc::new(AtomicU64::new(0));
    {
        let random_value = random_value.clone();
        tokio_runtime::get().spawn(async move {
            loop {
                // Simulate some async work to generate a random value
                let value = rand::random::<u64>();
                random_value.store(value, atomic::Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }
    Renderer::run(|| app(random_value.clone()))
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
        ColumnItem::fill(
            Box::new(|| text("Another item in column")),
            Some(1.0),
            None,
        ),
    ])
}

// Content section with header and text column
#[tessera]
fn content_section() {
    surface(
        SurfaceArgsBuilder::default().padding(20.0).build().unwrap(),
        || {
            column([
                ColumnItem::wrap(Box::new(|| header_row())),
                ColumnItem::wrap(Box::new(|| text_column())),
            ]);
        },
    )
}

// Random value display component
#[tessera]
fn random_value_display(random_value: Arc<AtomicU64>) {
    surface(
        SurfaceArgsBuilder::default()
            .corner_radius(25.0)
            .build()
            .unwrap(),
        move || {
            text(random_value.load(atomic::Ordering::SeqCst).to_string());
        },
    )
}

// Main app component
#[tessera]
fn app(random_value: Arc<AtomicU64>) {
    surface(
        SurfaceArgsBuilder::default()
            .color([1.0, 1.0, 1.0])
            .width(DimensionValue::Fill { max: None })
            .height(DimensionValue::Fill { max: None })
            .build()
            .unwrap(),
        move || {
            column([
                ColumnItem::wrap(Box::new(|| content_section())),
                ColumnItem::wrap(Box::new(|| {
                    spacer(
                        SpacerArgsBuilder::default()
                            .height(DimensionValue::Fixed(10))
                            .build()
                            .unwrap(),
                    )
                })),
                ColumnItem::wrap(Box::new({
                    let random_value = random_value.clone();
                    move || random_value_display(random_value.clone())
                })),
            ]);
        },
    );
}
