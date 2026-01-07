use tessera_components::{
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column_with_controller},
    modifier::ModifierExt,
    row::{RowArgs, row},
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Color, Dp, Modifier, retain, shard, tessera};

#[tessera]
#[shard]
pub fn spacer_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    );
}

#[tessera]
fn test_content() {
    let controller = retain(LazyListController::new);
    lazy_column_with_controller(
        LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width()),
        controller,
        |scope| {
            scope.item(|| text(TextArgs::default().text("Spacer Showcase").size(Dp(20.0))));
            scope.item(|| text("Horizontal Spacer (in a Row):"));
            scope.item(|| {
                row(RowArgs::default(), |scope| {
                    scope.child(|| colored_box(Color::RED));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::GREEN));
                    scope.child(|| spacer(Modifier::new().width(Dp(20.0))));
                    scope.child(|| colored_box(Color::BLUE));
                })
            });

            scope.item(|| text("Vertical Spacer (in a Column):"));
            scope.item(|| colored_box(Color::RED));
            scope.item(|| spacer(Modifier::new().height(Dp(20.0))));
            scope.item(|| colored_box(Color::GREEN));
            scope.item(|| spacer(Modifier::new().height(Dp(20.0))));
            scope.item(|| colored_box(Color::BLUE));
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
