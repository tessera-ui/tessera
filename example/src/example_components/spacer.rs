use tessera_ui::{Color, Dp, Modifier, shard, tessera};
use tessera_ui_basic_components::{
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};

#[tessera]
#[shard]
pub fn spacer_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                move || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
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
        ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
        |scope| {
            scope.child(|| text(TextArgs::default().text("Spacer Showcase").size(Dp(20.0))));

            scope.child(|| text("Horizontal Spacer (in a Row):"));
            scope.child(|| {
                row(RowArgs::default(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::BLUE));
                })
            });

            scope.child(|| text("Vertical Spacer (in a Column):"));
            scope.child(|| {
                column(ColumnArgs::default(), |scope| {
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
        SurfaceArgs::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0))),
        || {},
    );
}
