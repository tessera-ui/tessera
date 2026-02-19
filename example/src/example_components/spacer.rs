use tessera_components::{
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    modifier::ModifierExt,
    row::{RowArgs, row},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Color, Dp, Modifier, retain, shard, tessera};
#[tessera]
#[shard]
pub fn spacer_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        test_content,
    ));
}
fn test_content() {
    let controller = retain(LazyListController::new);
    lazy_column(
        &LazyColumnArgs::default()
            .content_padding(Dp(16.0))
            .modifier(Modifier::new().fill_max_width())
            .controller(controller)
            .content(|scope| {
                scope.item(|| text(&TextArgs::default().text("Spacer Showcase").size(Dp(20.0))));
                scope.item(|| text(&TextArgs::from("Horizontal Spacer (in a Row):")));
                scope.item(|| {
                    row(RowArgs::default(), |scope| {
                        scope.child(|| colored_box(Color::RED));
                        scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(20.0)))));
                        scope.child(|| colored_box(Color::GREEN));
                        scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(20.0)))));
                        scope.child(|| colored_box(Color::BLUE));
                    })
                });

                scope.item(|| text(&TextArgs::from("Vertical Spacer (in a Column):")));
                scope.item(|| colored_box(Color::RED));
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));
                scope.item(|| colored_box(Color::GREEN));
                scope.item(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(20.0)))));
                scope.item(|| colored_box(Color::BLUE));
            }),
    )
}
fn colored_box(color: Color) {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0))),
        || {},
    ));
}
