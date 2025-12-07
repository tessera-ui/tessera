use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgsBuilder, column},
    lazy_list::{LazyColumnArgs, LazyColumnArgsBuilder, LazyRowArgsBuilder, lazy_column, lazy_row},
    material_color::global_material_scheme,
    scrollable::ScrollableArgsBuilder,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn lazy_lists_showcase() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            lazy_column(
                LazyColumnArgs {
                    content_padding: Dp(24.0),
                    ..Default::default()
                },
                move |scope| {
                    scope.item(move || {
                        column(
                            ColumnArgsBuilder::default()
                                .width(DimensionValue::FILLED)
                                .build()
                                .unwrap(),
                            move |scope| {
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text("Lazy Lists")
                                            .size(Dp(24.0))
                                            .build()
                                            .unwrap(),
                                    );
                                });
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text(
                                                "Virtualized column/row that only mounts what is visible in the \
                                                viewport.",
                                            )
                                            .color(global_material_scheme().on_surface_variant)
                                            .build()
                                            .unwrap(),
                                    );
                                });
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text("Virtual contacts (lazy_column)")
                                            .size(Dp(18.0))
                                            .build()
                                            .unwrap(),
                                    );
                                });
                                scope.child(vertical_list);
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text("Horizontal gallery (lazy_row)")
                                            .size(Dp(18.0))
                                            .build()
                                            .unwrap(),
                                    );
                                });
                                scope.child(horizontal_gallery);
                            },
                        );
                    });
                },
            );
        },
    );
}

#[tessera]
fn vertical_list() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .style(global_material_scheme().surface_variant.into())
            .padding(Dp(12.0))
            .shape(Shape::rounded_rectangle(Dp(18.0)))
            .build()
            .unwrap(),
        move || {
            lazy_column(
                LazyColumnArgsBuilder::default()
                    .scrollable(
                        ScrollableArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::Fixed(Dp(360.0).into()))
                            .build()
                            .unwrap(),
                    )
                    .item_spacing(Dp(12.0))
                    .estimated_item_size(Dp(68.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .overscan(3)
                    .build()
                    .unwrap(),
                |scope| {
                    let indices: Vec<usize> = (0..500).collect();
                    scope.items_from_iter(indices, |_, idx| {
                        contact_card(*idx);
                    });
                },
            );
        },
    );
}

#[tessera]
fn horizontal_gallery() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .style(global_material_scheme().surface_variant.into())
            .padding(Dp(12.0))
            .shape(Shape::rounded_rectangle(Dp(18.0)))
            .build()
            .unwrap(),
        move || {
            lazy_row(
                LazyRowArgsBuilder::default()
                    .scrollable(
                        ScrollableArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::Fixed(Dp(180.0).into()))
                            .build()
                            .unwrap(),
                    )
                    .item_spacing(Dp(16.0))
                    .estimated_item_size(Dp(160.0))
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .overscan(4)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.items(240, |index| {
                        gallery_card(index);
                    });
                },
            );
        },
    );
}

#[tessera]
fn contact_card(index: usize) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .padding(Dp(12.0))
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .style(color_for_index(index).with_alpha(0.15).into())
            .build()
            .unwrap(),
        move || {
            column(
                ColumnArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child({
                        let contact_number = index + 1;
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(format!("Contact {}", contact_number))
                                    .size(Dp(16.0))
                                    .build()
                                    .unwrap(),
                            );
                        }
                    });
                    scope.child({
                        let unread_count = (index * 3) % 7;
                        move || {
                            text(
                                TextArgsBuilder::default()
                                    .text(format!("{unread_count} unread messages"))
                                    .color(global_material_scheme().on_surface_variant)
                                    .build()
                                    .unwrap(),
                            );
                        }
                    });
                },
            );
        },
    );
}

#[tessera]
fn gallery_card(index: usize) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(Dp(150.0).into()))
            .height(DimensionValue::Fixed(Dp(150.0).into()))
            .padding(Dp(12.0))
            .shape(Shape::rounded_rectangle(Dp(24.0)))
            .style(color_for_index(index).into())
            .build()
            .unwrap(),
        move || {
            text(
                TextArgsBuilder::default()
                    .text(format!("Card {}", index + 1))
                    .size(Dp(16.0))
                    .color(Color::WHITE)
                    .build()
                    .unwrap(),
            );
        },
    );
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
