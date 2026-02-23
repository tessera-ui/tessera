use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgs, boxed},
    column::{ColumnArgs, column},
    flow_column::{FlowColumnArgs, flow_column},
    flow_row::{FlowRowArgs, flow_row},
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    scrollable::{ScrollableArgs, scrollable},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};
use tessera_ui::{Color, Dp, Modifier, shard, use_context};
#[shard]
pub fn layouts_showcase() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            scrollable(
                &ScrollableArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .child(|| {
                        surface(&SurfaceArgs::with_child(
                            SurfaceArgs::default()
                                .modifier(Modifier::new().fill_max_width().padding_all(Dp(25.0))),
                            || {
                                column(ColumnArgs::default(), |scope| {
                                    scope.child(row_showcase);
                                    scope.child(column_showcase);
                                    scope.child(flow_row_showcase);
                                    scope.child(flow_column_showcase);
                                    scope.child(boxed_showcase);
                                })
                            },
                        ));
                    }),
            )
        },
    ));
}
fn showcase_box(color: Color) {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .style(color.into())
            .modifier(Modifier::new().size(Dp(50.0), Dp(50.0))),
        || {},
    ));
}
fn row_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text(&TextArgs::from("Row Showcase".to_string())));
        scope.child(|| {
            text(
                &TextArgs::default()
                    .text("Arranges children horizontally.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(&SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
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
            ))
        });
    });
}
fn column_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text(&TextArgs::from("Column Showcase")));
        scope.child(|| {
            text(
                &TextArgs::default()
                    .text("Arranges children vertically.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(&SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
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
            ))
        });
    });
}
fn boxed_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text(&TextArgs::from("Boxed Showcase")));
        scope.child(|| {
            text(
                &TextArgs::default()
                    .text("A container that can align its child.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(&SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
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
            ))
        });
    });
}
fn flow_row_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text(&TextArgs::from("FlowRow Showcase")));
        scope.child(|| {
            text(
                &TextArgs::default()
                    .text("Wraps children into rows when there is not enough space.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(&SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    ),
                || {
                    flow_row(
                        FlowRowArgs::default()
                            .modifier(Modifier::new().width(Dp(240.0)))
                            .item_spacing(Dp(8.0))
                            .line_spacing(Dp(8.0))
                            .cross_axis_alignment(CrossAxisAlignment::Center),
                        |scope| {
                            scope.child(|| showcase_box(Color::new(0.8, 0.2, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.8, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.2, 0.8, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.9, 0.6, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.6, 0.2, 0.8, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.7, 0.7, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.8, 0.4, 0.4, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.3, 0.5, 0.9, 1.0)));
                        },
                    )
                },
            ))
        });
    });
}
fn flow_column_showcase() {
    column(ColumnArgs::default(), |scope| {
        scope.child(|| text(&TextArgs::from("FlowColumn Showcase")));
        scope.child(|| {
            text(
                &TextArgs::default()
                    .text("Wraps children into columns when there is not enough space.")
                    .size(Dp(16.0)),
            )
        });
        scope.child(|| {
            surface(&SurfaceArgs::with_child(
                SurfaceArgs::default()
                    .modifier(Modifier::new().padding_all(Dp(10.0)))
                    .style(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .surface_variant
                            .into(),
                    ),
                || {
                    flow_column(
                        FlowColumnArgs::default()
                            .modifier(Modifier::new().height(Dp(200.0)))
                            .item_spacing(Dp(8.0))
                            .line_spacing(Dp(8.0))
                            .cross_axis_alignment(CrossAxisAlignment::Center),
                        |scope| {
                            scope.child(|| showcase_box(Color::new(0.8, 0.2, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.8, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.2, 0.8, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.9, 0.6, 0.2, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.6, 0.2, 0.8, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.2, 0.7, 0.7, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.8, 0.4, 0.4, 1.0)));
                            scope.child(|| showcase_box(Color::new(0.3, 0.5, 0.9, 1.0)));
                        },
                    )
                },
            ))
        });
    });
}
