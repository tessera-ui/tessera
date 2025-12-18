use tessera_ui::{Color, Dp, Modifier, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    modifier::ModifierExt as _,
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, scrollable},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn layouts_showcase() {
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
                || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0)))
                            .build()
                            .unwrap(),
                        || {
                            column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                                scope.child(row_showcase);
                                scope.child(column_showcase);
                                scope.child(boxed_showcase);
                                scope.child(modifier_showcase);
                            })
                        },
                    );
                },
            )
        },
    );
}

#[tessera]
fn showcase_box(color: Color) {
    surface(
        SurfaceArgsBuilder::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0)))
            .build()
            .unwrap(),
        || {},
    );
}

#[tessera]
fn row_showcase() {
    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
        scope.child(|| text("Row Showcase".to_string()));
        scope.child(|| {
            text(
                TextArgsBuilder::default()
                    .text("Arranges children horizontally.")
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    )
                    .build()
                    .unwrap(),
                || {
                    row(
                        RowArgsBuilder::default()
                            .main_axis_alignment(MainAxisAlignment::Center)
                            .build()
                            .unwrap(),
                        |scope| {
                            scope.child(|| showcase_box(Color::new(0.8, 0.2, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.8, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.2, 0.8, 1.0)));
                        },
                    )
                },
            )
        });
    });
}

#[tessera]
fn column_showcase() {
    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
        scope.child(|| text("Column Showcase"));
        scope.child(|| {
            text(
                TextArgsBuilder::default()
                    .text("Arranges children vertically.")
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    )
                    .build()
                    .unwrap(),
                || {
                    column(
                        ColumnArgsBuilder::default()
                            .cross_axis_alignment(CrossAxisAlignment::Center)
                            .build()
                            .unwrap(),
                        |scope| {
                            scope.child(|| showcase_box(Color::new(0.8, 0.2, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.8, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.2, 0.8, 1.0)));
                        },
                    )
                },
            )
        });
    });
}

#[tessera]
fn boxed_showcase() {
    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
        scope.child(|| text("Boxed Showcase"));
        scope.child(|| {
            text(
                TextArgsBuilder::default()
                    .text("A container that can align its single child.")
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    )
                    .build()
                    .unwrap(),
                || {
                    boxed(
                        BoxedArgsBuilder::default()
                            .alignment(Alignment::Center)
                            .build()
                            .unwrap(),
                        |scope| {
                            scope.child(|| showcase_box(Color::new(0.8, 0.5, 0.2, 1.0)));
                        },
                    )
                },
            )
        });
    });
}

#[tessera]
fn modifier_showcase() {
    column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
        scope.child(|| text("Modifier Showcase"));
        scope.child(|| {
            text(
                TextArgsBuilder::default()
                    .text("Applies alpha, clipping, background, and border behavior to subtrees.")
                    .size(Dp(16.0))
                    .build()
                    .unwrap(),
            )
        });

        scope.child(|| {
            surface(
                SurfaceArgsBuilder::default()
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    )
                    .modifier(Modifier::new().fill_max_width())
                    .build()
                    .unwrap(),
                || {
                    column(
                        ColumnArgsBuilder::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(10.0)))
                            .build()
                            .unwrap(),
                        |column_scope| {
                            column_scope.child(|| {
                                row(
                                    RowArgsBuilder::default()
                                        .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(Modifier::new().fill_max_width())
                                        .build()
                                        .unwrap(),
                                    |row_scope| {
                                        row_scope.child(|| {
                                            surface(
                                                SurfaceArgsBuilder::default()
                                                    .style(Color::new(0.2, 0.6, 0.9, 1.0).into())
                                                    .modifier(
                                                        Modifier::new().size(Dp(80.0), Dp(40.0)),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                || {
                                                    boxed(
                                                        BoxedArgsBuilder::default()
                                                            .alignment(Alignment::Center)
                                                            .build()
                                                            .unwrap(),
                                                        |s| {
                                                            s.child(|| text("alpha=1.0"));
                                                        },
                                                    );
                                                },
                                            );
                                        });

                                        row_scope.child(|| {
                                            surface(
                                                SurfaceArgsBuilder::default()
                                                    .style(Color::new(0.2, 0.6, 0.9, 1.0).into())
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(80.0), Dp(40.0))
                                                            .alpha(0.35),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                || {
                                                    boxed(
                                                        BoxedArgsBuilder::default()
                                                            .alignment(Alignment::Center)
                                                            .build()
                                                            .unwrap(),
                                                        |s| {
                                                            s.child(|| text("alpha=0.35"));
                                                        },
                                                    );
                                                },
                                            );
                                        });
                                    },
                                );
                            });

                            column_scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

                            column_scope.child(|| {
                                row(
                                    RowArgsBuilder::default()
                                        .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(Modifier::new().fill_max_width())
                                        .build()
                                        .unwrap(),
                                    |row_scope| {
                                        let shape = Shape::rounded_rectangle(Dp(12.0));
                                        let fill = Color::new(0.2, 0.6, 0.9, 1.0);
                                        let border = Color::new(0.1, 0.4, 0.7, 1.0);

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgsBuilder::default()
                                                    .alignment(Alignment::Center)
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(92.0), Dp(40.0))
                                                            .background_with_shape(fill, shape),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                |s| {
                                                    s.child(|| text("background"));
                                                },
                                            );
                                        });

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgsBuilder::default()
                                                    .alignment(Alignment::Center)
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(92.0), Dp(40.0))
                                                            .border_with_shape(
                                                                Dp(2.0),
                                                                border,
                                                                shape,
                                                            ),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                |s| {
                                                    s.child(|| text("border"));
                                                },
                                            );
                                        });

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgsBuilder::default()
                                                    .alignment(Alignment::Center)
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(92.0), Dp(40.0))
                                                            .background_with_shape(fill, shape)
                                                            .border_with_shape(
                                                                Dp(2.0),
                                                                border,
                                                                shape,
                                                            ),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                |s| {
                                                    s.child(|| text("both"));
                                                },
                                            );
                                        });
                                    },
                                );
                            });

                            column_scope.child(|| spacer(Modifier::new().height(Dp(10.0))));

                            column_scope.child(|| {
                                boxed(
                                    BoxedArgsBuilder::default()
                                        .modifier(
                                            Modifier::new()
                                                .size(Dp(240.0), Dp(96.0))
                                                .clip_to_bounds(),
                                        )
                                        .alignment(Alignment::TopStart)
                                        .build()
                                        .unwrap(),
                                    |boxed_scope| {
                                        boxed_scope.child(|| {
                                            surface(
                                                SurfaceArgsBuilder::default()
                                                    .style(Color::new(0.8, 0.2, 0.3, 1.0).into())
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(320.0), Dp(160.0))
                                                            .offset(Dp(-40.0), Dp(-32.0))
                                                            .alpha(0.75),
                                                    )
                                                    .build()
                                                    .unwrap(),
                                                || {
                                                    boxed(
                                                        BoxedArgsBuilder::default()
                                                            .alignment(Alignment::Center)
                                                            .build()
                                                            .unwrap(),
                                                        |s| {
                                                            s.child(|| text("clipped"));
                                                        },
                                                    );
                                                },
                                            );
                                        });
                                    },
                                );
                            });
                        },
                    );
                },
            )
        });
    });
}
