use tessera_ui::{Color, DimensionValue, Dp, Renderer, tessera};
use tessera_ui_basic_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    button_group::{ButtonGroupArgsBuilder, ButtonGroupItem, ButtonGroupState, button_group},
    column::{ColumnArgsBuilder, column},
    row::{RowArgsBuilder, row},
    shape_def::Shape,
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
};

#[tessera]
fn button_group_card(title: &'static str, state: ButtonGroupState, allow_deselect: bool) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fixed(Dp(320.0).into()))
            .style(SurfaceStyle::FilledOutlined {
                fill_color: Color::new(0.98, 0.98, 0.99, 1.0),
                border_color: Color::new(0.85, 0.87, 0.9, 1.0),
                border_width: Dp(1.0),
            })
            .shape(Shape::RoundedRectangle {
                top_left: Dp(16.0),
                top_right: Dp(16.0),
                bottom_right: Dp(16.0),
                bottom_left: Dp(16.0),
                g2_k_value: 2.0,
            })
            .padding(Dp(16.0))
            .build()
            .unwrap(),
        None,
        move || {
            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(move || {
                        text(
                            TextArgsBuilder::default()
                                .text(title.to_string())
                                .color(Color::new(0.12, 0.14, 0.18, 1.0))
                                .size(Dp(18.0))
                                .build()
                                .unwrap(),
                        );
                    });

                    scope.child(|| spacer(SpacerArgs::from(Dp(12.0))));

                    scope.child(move || {
                        let args = ButtonGroupArgsBuilder::default()
                            .allow_deselect_single(allow_deselect)
                            .build()
                            .expect("builder construction failed");
                        button_group(args, state.clone(), |scope| {
                            scope.item(ButtonGroupItem::text("Day"));
                            scope.item(ButtonGroupItem::text("Week"));
                            scope.item(ButtonGroupItem::text("Month"));
                        });
                    });
                },
            );
        },
    );
}

#[tessera]
fn app(single_state: ButtonGroupState, multi_state: ButtonGroupState) {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .height(DimensionValue::Fill {
                min: None,
                max: None,
            })
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            column(
                ColumnArgsBuilder::default()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                |scope| {
                    let single_state_for_scope = single_state.clone();
                    let multi_state_for_scope = multi_state.clone();
                    scope.child(move || {
                        let single_state_row = single_state_for_scope.clone();
                        let multi_state_row = multi_state_for_scope.clone();
                        row(
                            RowArgsBuilder::default()
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                .build()
                                .unwrap(),
                            |row_scope| {
                                let single_state_for_card = single_state_row.clone();
                                row_scope.child(move || {
                                    button_group_card(
                                        "Single (no deselect)",
                                        single_state_for_card.clone(),
                                        false,
                                    );
                                });
                                row_scope.child(|| spacer(SpacerArgs::from(Dp(16.0))));
                                let multi_state_for_card = multi_state_row.clone();
                                row_scope.child(move || {
                                    button_group_card(
                                        "Multi-select",
                                        multi_state_for_card.clone(),
                                        true,
                                    );
                                });
                            },
                        );
                    });
                },
            );
        },
    );
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let single_state = ButtonGroupState::single_with_initial(Some(0), false);
    let multi_state = ButtonGroupState::multiple([0, 2]);

    Renderer::run(
        {
            move || {
                app(single_state.clone(), multi_state.clone());
            }
        },
        |app| {
            tessera_ui_basic_components::pipelines::register_pipelines(app);
        },
    )?;

    Ok(())
}
