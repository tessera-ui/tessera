use tessera::Renderer;
use tessera_basic_components::{
    column::{AsColumnChild, ColumnChild, column},
    row::{RowChild, row},
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
    let args = SurfaceArgsBuilder::default().build().unwrap();
    surface(args, || {
        column([
            (&|| {
                row([
                    RowChild::new(Some(1.0), &|| text("Hello, this is tessera")),
                    RowChild::new(Some(1.0), &|| text("Hello, this is another tessera")),
                ])
            })
                .as_column_child(),
            (&|| {
                column([
                    ColumnChild::new(Some(1.0), &|| text("This is a column")),
                    ColumnChild::new(Some(1.0), &|| text("Another item in column")),
                ])
            })
                .as_column_child(),
        ]);
    });
}
