use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::{BoxedArgsBuilder, boxed},
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    scrollable::{ScrollableArgsBuilder, ScrollableState, scrollable},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
#[shard]
pub fn layouts_showcase(#[state] scrollable_state: ScrollableState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            scrollable(
                ScrollableArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .build()
                    .unwrap(),
                scrollable_state,
                || {
                    surface(
                        SurfaceArgsBuilder::default()
                            .style(Color::WHITE.into())
                            .padding(Dp(25.0))
                            .width(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                        None,
                        || {
                            column(ColumnArgsBuilder::default().build().unwrap(), |scope| {
                                scope.child(row_showcase);
                                scope.child(column_showcase);
                                scope.child(boxed_showcase);
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
            .width(DimensionValue::from(Dp(50.0)))
            .height(DimensionValue::from(Dp(50.0)))
            .build()
            .unwrap(),
        None,
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
                    .padding(Dp(10.0))
                    .style(Color::new(0.9, 0.9, 0.9, 1.0).into())
                    .build()
                    .unwrap(),
                None,
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
                    .padding(Dp(10.0))
                    .style(Color::new(0.9, 0.9, 0.9, 1.0).into())
                    .build()
                    .unwrap(),
                None,
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
                    .padding(Dp(10.0))
                    .style(Color::new(0.9, 0.9, 0.9, 1.0).into())
                    .build()
                    .unwrap(),
                None,
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
