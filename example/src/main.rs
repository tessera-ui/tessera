use tessera::Renderer;
use tessera_basic_components::{
    column::{AsColumnChild, ColumnChild, column},
    row::{RowChild, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::text,
};
use tessera_macros::tessera;

fn main() -> Result<(), impl std::error::Error> {
    env_logger::init();
    Renderer::run(app)
}

#[tessera]
fn app() {
    surface(
        SurfaceArgsBuilder::default()
            .color([1.0, 1.0, 1.0])
            .build()
            .unwrap(),
        || {
            column([
                (|| {
                    surface(
                        SurfaceArgsBuilder::default().padding(20.0).build().unwrap(),
                        || {
                            column([
                                (&|| {
                                    row([
                                        RowChild::new(Some(1.0), &|| {
                                            text("Hello, this is tessera")
                                        }),
                                        RowChild::new(Some(1.0), &|| {
                                            text("Hello, this is another tessera")
                                        }),
                                    ])
                                })
                                    .into_column_child(),
                                (&|| {
                                    column([
                                        ColumnChild::new(Some(1.0), &|| text("This is a column")),
                                        ColumnChild::new(Some(1.0), &|| {
                                            text("Another item in column")
                                        }),
                                    ])
                                })
                                    .into_column_child(),
                            ]);
                        },
                    )
                })
                .into_column_child(),
                (&|| {
                    spacer(
                        SpacerArgsBuilder::default()
                            .height(10)
                            .build()
                            .unwrap(),
                    )
                })
                    .into_column_child(),
                (&|| {
                    surface(
                        SurfaceArgsBuilder::default()
                            .corner_radius(25.0)
                            .build()
                            .unwrap(),
                        || {
                            text("Hello, this is a surface with text");
                        },
                    )
                })
                    .into_column_child(),
            ]);
        },
    );
}
