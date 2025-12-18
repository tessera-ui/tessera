use tessera_ui::{Color, Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn spacer_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .modifier(Modifier::new().fill_max_size())
            .build()
            .unwrap(),
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                move || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        || {
                            test_content();
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn test_content() {
    column(
        ColumnArgsBuilder::default()
            .modifier(Modifier::new().fill_max_width())
            .build()
            .unwrap(),
        |scope| {
            scope.child(|| {
                text(
                    TextArgsBuilder::default()
                        .text("Spacer Showcase")
                        .size(Dp(20.0))
                        .build()
                        .unwrap(),
                )
            });

            scope.child(|| text("Horizontal Spacer (in a Row):"));
            scope.child(|| {
                row(RowArgsBuilder::default().build().unwrap(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::BLUE));
                })
            });

            scope.child(|| text("Vertical Spacer (in a Column):"));
            scope.child(|| {
                column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| spacer(Modifier::new().height(Dp(20.0))));
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| spacer(Modifier::new().height(Dp(20.0))));
                    scope.child(|| colored_box(Color::BLUE));
                })
            });
        },
    )
}

#[tessera]
fn colored_box(color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0)))
            .build()
            .unwrap(),
        || {},
    );
}
