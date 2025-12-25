use tessera_ui::{Color, Dp, Modifier, shard, tessera, use_context};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

#[tessera]
#[shard]
pub fn layouts_showcase() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                ScrollableArgs::default().modifier(Modifier::new().fill_max_width()),
                || {
                    surface(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                        || {
                            column(ColumnArgs::default(), |scope| {
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
        SurfaceArgs::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0))),
        || {},
    );
}

#[tessera]
fn row_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text("Row Showcase".to_string()));
        scope.child(|| {
            text(
                TextArgs::default()
                    .text("Arranges children horizontally.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    ),
                || {
                    row(
                        RowArgs::default().main_axis_alignment(MainAxisAlignment::Center),
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
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text("Column Showcase"));
        scope.child(|| {
            text(
                TextArgs::default()
                    .text("Arranges children vertically.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    ),
                || {
                    column(
                        ColumnArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
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
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text("Boxed Showcase"));
        scope.child(|| {
            text(
                TextArgs::default()
                    .text("A container that can align its single child.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    ),
                || {
                    boxed(BoxedArgs::default().alignment(Alignment::Center), |scope| {
                        scope.child(|| showcase_box(Color::new(0.8, 0.5, 0.2, 1.0)));
                    })
                },
            )
        });
    });
}

#[tessera]
fn modifier_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text("Modifier Showcase"));
        scope.child(|| {
            text(
                TextArgs::default()
                    .text("Applies alpha, clipping, background, and border behavior to subtrees.")
                    .size(Dp(16.0)),
            )
        });

        scope.child(|| {
            surface(
                SurfaceArgs::default()
                    .style(
                        use_context::<MaterialTheme>()
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    )
                    .modifier(Modifier::new().fill_max_width()),
                || {
                    column(
                        ColumnArgs::default()
                            .modifier(Modifier::new().fill_max_width().padding_all(Dp(10.0))),
                        |column_scope| {
                            column_scope.child(|| {
                                row(
                                    RowArgs::default()
                                        .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(Modifier::new().fill_max_width()),
                                    |row_scope| {
                                        row_scope.child(|| {
                                            surface(
                                                SurfaceArgs::default()
                                                    .style(Color::new(0.2, 0.6, 0.9, 1.0).into())
                                                    .modifier(
                                                        Modifier::new().size(Dp(80.0), Dp(40.0)),
                                                    ),
                                                || {
                                                    boxed(
                                                        BoxedArgs::default()
                                                            .alignment(Alignment::Center),
                                                        |s| {
                                                            s.child(|| text("alpha=1.0"));
                                                        },
                                                    );
                                                },
                                            );
                                        });

                                        row_scope.child(|| {
                                            surface(
                                                SurfaceArgs::default()
                                                    .style(Color::new(0.2, 0.6, 0.9, 1.0).into())
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(80.0), Dp(40.0))
                                                            .alpha(0.35),
                                                    ),
                                                || {
                                                    boxed(
                                                        BoxedArgs::default()
                                                            .alignment(Alignment::Center),
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
                                    RowArgs::default()
                                        .main_axis_alignment(MainAxisAlignment::SpaceAround)
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .modifier(Modifier::new().fill_max_width()),
                                    |row_scope| {
                                        let shape = Shape::rounded_rectangle(Dp(12.0));
                                        let fill = Color::new(0.2, 0.6, 0.9, 1.0);
                                        let border = Color::new(0.1, 0.4, 0.7, 1.0);

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgs::default()
                                                    .alignment(Alignment::Center)
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(92.0), Dp(40.0))
                                                            .background_with_shape(fill, shape),
                                                    ),
                                                |s| {
                                                    s.child(|| text("background"));
                                                },
                                            );
                                        });

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgs::default()
                                                    .alignment(Alignment::Center)
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(92.0), Dp(40.0))
                                                            .border_with_shape(
                                                                Dp(2.0),
                                                                border,
                                                                shape,
                                                            ),
                                                    ),
                                                |s| {
                                                    s.child(|| text("border"));
                                                },
                                            );
                                        });

                                        row_scope.child(move || {
                                            boxed(
                                                BoxedArgs::default()
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
                                                    ),
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
                                    BoxedArgs::default()
                                        .modifier(
                                            Modifier::new()
                                                .size(Dp(240.0), Dp(96.0))
                                                .clip_to_bounds(),
                                        )
                                        .alignment(Alignment::TopStart),
                                    |boxed_scope| {
                                        boxed_scope.child(|| {
                                            surface(
                                                SurfaceArgs::default()
                                                    .style(Color::new(0.8, 0.2, 0.3, 1.0).into())
                                                    .modifier(
                                                        Modifier::new()
                                                            .size(Dp(320.0), Dp(160.0))
                                                            .offset(Dp(-40.0), Dp(-32.0))
                                                            .alpha(0.75),
                                                    ),
                                                || {
                                                    boxed(
                                                        BoxedArgs::default()
                                                            .alignment(Alignment::Center),
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
