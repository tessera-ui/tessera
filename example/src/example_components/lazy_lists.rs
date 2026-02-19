use tessera_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgs, column},
    lazy_list::{LazyColumnArgs, LazyRowArgs, lazy_column, lazy_row},
    modifier::ModifierExt as _,
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, shard, tessera, use_context};
#[tessera]
#[shard]
pub fn lazy_lists_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                &LazyColumnArgs {
                    content_padding: Dp(24.0),
                    ..Default::default()
                }
                .content(move |scope| {
                    scope.item(move || {
                        column(
                            ColumnArgs::default()
                                .modifier(Modifier::new().fill_max_width()),
                            move |scope| {
                                scope.child(|| {
                                    text(&TextArgs::default()
                                            .text("Lazy Lists")
                                            .size(Dp(24.0)));
                                });
                                scope.child(|| {
                                    text(&TextArgs::default()
                                            .text("Virtualized column/row that only mounts what is visible in the viewport.")
                                            .color(
                                                use_context::<MaterialTheme>()
                                                    .expect("MaterialTheme must be provided")
                                                    .get()
                                                    .color_scheme
                                                    .on_surface_variant,
                                            ));
                                });
                                scope.child(|| {
                                    text(&TextArgs::default()
                                            .text("Virtual contacts (lazy_column)")
                                            .size(Dp(18.0)));
                                });
                                scope.child(vertical_list);
                                scope.child(|| {
                                    text(&TextArgs::default()
                                            .text("Horizontal gallery (lazy_row)")
                                            .size(Dp(18.0)));
                                });
                                scope.child(horizontal_gallery);
                            }
                        );
                    });
                }),
            );
        },
    ));
}
fn vertical_list() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
            .style(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .surface_variant
                    .into(),
            )
            .shape(Shape::rounded_rectangle(Dp(18.0))),
        move || {
            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width().height(Dp(360.0)))
                    .item_spacing(Dp(12.0))
                    .estimated_item_size(Dp(68.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .overscan(3)
                    .content(|scope| {
                        let indices: Vec<usize> = (0..500).collect();
                        scope.items_from_iter(indices, |_, idx| {
                            contact_card(*idx);
                        });
                    }),
            );
        },
    ));
}
fn horizontal_gallery() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
            .style(
                use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .surface_variant
                    .into(),
            )
            .shape(Shape::rounded_rectangle(Dp(18.0))),
        move || {
            lazy_row(
                &LazyRowArgs::default()
                    .modifier(Modifier::new().fill_max_width().height(Dp(180.0)))
                    .item_spacing(Dp(16.0))
                    .estimated_item_size(Dp(160.0))
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .overscan(4)
                    .content(|scope| {
                        scope.items(240, |index| {
                            gallery_card(index);
                        });
                    }),
            );
        },
    ));
}
fn contact_card(index: usize) {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(12.0)))
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .style(color_for_index(index).with_alpha(0.15).into()),
        move || {
            column(
                ColumnArgs::default().modifier(Modifier::new().fill_max_width()),
                |scope| {
                    scope.child({
                        let contact_number = index + 1;
                        move || {
                            text(
                                &TextArgs::default()
                                    .text(format!("Contact {}", contact_number))
                                    .size(Dp(16.0)),
                            );
                        }
                    });
                    scope.child({
                        let unread_count = (index * 3) % 7;
                        move || {
                            text(
                                &TextArgs::default()
                                    .text(format!("{unread_count} unread messages"))
                                    .color(
                                        use_context::<MaterialTheme>()
                                            .expect("MaterialTheme must be provided")
                                            .get()
                                            .color_scheme
                                            .on_surface_variant,
                                    ),
                            );
                        }
                    });
                },
            );
        },
    ));
}
fn gallery_card(index: usize) {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(
                Modifier::new()
                    .size(Dp(150.0), Dp(150.0))
                    .padding_all(Dp(12.0)),
            )
            .shape(Shape::rounded_rectangle(Dp(24.0)))
            .style(color_for_index(index).into()),
        move || {
            text(
                &TextArgs::default()
                    .text(format!("Card {}", index + 1))
                    .size(Dp(16.0))
                    .color(Color::WHITE),
            );
        },
    ));
}

fn color_for_index(index: usize) -> Color {
    let palette = [
        Color::new(0.35, 0.31, 0.82, 1.0),
        Color::new(0.11, 0.58, 0.95, 1.0),
        Color::new(0.0, 0.68, 0.55, 1.0),
        Color::new(0.98, 0.66, 0.0, 1.0),
        Color::new(0.9, 0.23, 0.4, 1.0),
    ];
    palette[index % palette.len()]
}
