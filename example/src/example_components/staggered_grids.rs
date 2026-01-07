//! Staggered grid showcases for the example app.
//!
//! ## Usage
//!
//! Explore masonry-style staggered grids in the example catalog.
use tessera_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::{ColumnArgs, column},
    lazy_list::{LazyColumnArgs, lazy_column},
    lazy_staggered_grid::{
        LazyHorizontalStaggeredGridArgs, LazyVerticalStaggeredGridArgs, StaggeredGridCells,
        lazy_horizontal_staggered_grid, lazy_vertical_staggered_grid,
    },
    modifier::ModifierExt as _,
    scrollable::ScrollableArgs,
    shape_def::Shape,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, shard, tessera, use_context};

#[tessera]
#[shard]
pub fn staggered_grids_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                LazyColumnArgs {
                    content_padding: Dp(24.0),
                    ..Default::default()
                },
                move |scope| {
                    scope.item(move || {
                        column(
                            ColumnArgs::default()
                                .modifier(Modifier::new().fill_max_width()),
                            move |scope| {
                                scope.child(|| {
                                    text(
                                        TextArgs::default()
                                            .text("Staggered Grids")
                                            .size(Dp(24.0)),
                                    );
                                });
                                scope.child(|| {
                                    text(
                                        TextArgs::default()
                                            .text(
                                                "Masonry-style grids that distribute tiles by lane height.",
                                            )
                                            .color(
                                                use_context::<MaterialTheme>()
                                                    .expect("MaterialTheme must be provided")
                                                    .get()
                                                    .color_scheme
                                                    .on_surface_variant,
                                            ),
                                    );
                                });
                                scope.child(|| {
                                    text(
                                        TextArgs::default()
                                            .text("Vertical staggered grid")
                                            .size(Dp(18.0)),
                                    );
                                });
                                scope.child(vertical_staggered_grid);
                                scope.child(|| {
                                    text(
                                        TextArgs::default()
                                            .text("Horizontal staggered grid")
                                            .size(Dp(18.0)),
                                    );
                                });
                                scope.child(horizontal_staggered_grid);
                            },
                        );
                    });
                },
            );
        },
    );
}

#[tessera]
fn vertical_staggered_grid() {
    surface(
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
            lazy_vertical_staggered_grid(
                LazyVerticalStaggeredGridArgs::default()
                    .scrollable(
                        ScrollableArgs::default()
                            .modifier(Modifier::new().fill_max_width().height(Dp(360.0))),
                    )
                    .columns(StaggeredGridCells::adaptive(Dp(150.0)))
                    .main_axis_spacing(Dp(12.0))
                    .cross_axis_spacing(Dp(12.0))
                    .cross_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .item_alignment(CrossAxisAlignment::Stretch)
                    .estimated_item_size(Dp(140.0))
                    .overscan(2),
                |scope| {
                    scope.items(160, |index| {
                        let height = match index % 6 {
                            0 => Dp(220.0),
                            1 => Dp(180.0),
                            2 => Dp(160.0),
                            _ => Dp(120.0),
                        };
                        staggered_tile(index, None, Some(height));
                    });
                },
            );
        },
    );
}

#[tessera]
fn horizontal_staggered_grid() {
    surface(
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
            lazy_horizontal_staggered_grid(
                LazyHorizontalStaggeredGridArgs::default()
                    .scrollable(
                        ScrollableArgs::default()
                            .modifier(Modifier::new().fill_max_width().height(Dp(220.0))),
                    )
                    .rows(StaggeredGridCells::fixed(2))
                    .main_axis_spacing(Dp(16.0))
                    .cross_axis_spacing(Dp(12.0))
                    .cross_axis_alignment(MainAxisAlignment::SpaceAround)
                    .item_alignment(CrossAxisAlignment::Stretch)
                    .estimated_item_size(Dp(180.0))
                    .overscan(3),
                |scope| {
                    scope.items(140, |index| {
                        let width = match index % 5 {
                            0 => Dp(240.0),
                            1 => Dp(210.0),
                            2 => Dp(180.0),
                            _ => Dp(150.0),
                        };
                        staggered_tile(index, Some(width), None);
                    });
                },
            );
        },
    );
}

#[tessera]
fn staggered_tile(index: usize, width: Option<Dp>, height: Option<Dp>) {
    let mut modifier = Modifier::new();
    if let Some(width) = width {
        modifier = modifier.width(width);
    }
    if let Some(height) = height {
        modifier = modifier.height(height);
    }
    let modifier = modifier.padding_all(Dp(8.0));

    surface(
        SurfaceArgs::default()
            .modifier(modifier)
            .shape(Shape::rounded_rectangle(Dp(16.0)))
            .style(color_for_index(index).into()),
        move || {
            text(
                TextArgs::default()
                    .text(format!("Tile {}", index + 1))
                    .size(Dp(16.0))
                    .color(Color::WHITE),
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
