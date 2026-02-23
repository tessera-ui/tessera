use tessera_components::{
    alignment::CrossAxisAlignment,
    column::{ColumnArgs, column},
    floating_action_button::{
        FloatingActionButtonArgs, FloatingActionButtonDefaults, FloatingActionButtonSize,
        floating_action_button,
    },
    icon::{IconArgs, icon},
    material_icons::filled,
    modifier::ModifierExt as _,
    row::{RowArgs, row},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, surface},
    text::{TextArgs, text},
};
use tessera_ui::{Dp, Modifier, remember, shard};
#[shard]
pub fn floating_action_button_showcase() {
    let clicks = remember(|| 0u32);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            column(
                ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size().padding_all(Dp(16.0))),
                |scope| {
                    scope.child(|| {
                        text(
                            &TextArgs::default()
                                .text("Floating Action Button Showcase")
                                .size(Dp(20.0)),
                        );
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(&TextArgs::default().text("Sizes").size(Dp(14.0)));
                    });

                    scope.child(move || {
                        row(
                            RowArgs::default()
                                .modifier(Modifier::new().fill_max_width())
                                .cross_axis_alignment(CrossAxisAlignment::Center),
                            |row_scope| {
                                row_scope.child(move || {
                                    let component_args = FloatingActionButtonArgs::with_content(
                                        FloatingActionButtonArgs::default()
                                            .size(FloatingActionButtonSize::Small)
                                            .on_click(move || {
                                                clicks.with_mut(|value| *value += 1);
                                            }),
                                        || {
                                            icon(&IconArgs::from(filled::home_icon()).size(
                                                FloatingActionButtonDefaults::icon_size(
                                                    FloatingActionButtonSize::Small,
                                                ),
                                            ));
                                        },
                                    );
                                    floating_action_button(&component_args);
                                });

                                row_scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(16.0))));
                                });

                                row_scope.child(move || {
                                    let component_args = FloatingActionButtonArgs::with_content(
                                        FloatingActionButtonArgs::default()
                                            .size(FloatingActionButtonSize::Standard)
                                            .on_click(move || {
                                                clicks.with_mut(|value| *value += 1);
                                            }),
                                        || {
                                            icon(&IconArgs::from(filled::home_icon()).size(
                                                FloatingActionButtonDefaults::icon_size(
                                                    FloatingActionButtonSize::Standard,
                                                ),
                                            ));
                                        },
                                    );
                                    floating_action_button(&component_args);
                                });

                                row_scope.child(|| {
                                    spacer(&SpacerArgs::new(Modifier::new().width(Dp(16.0))));
                                });

                                row_scope.child(move || {
                                    let component_args = FloatingActionButtonArgs::with_content(
                                        FloatingActionButtonArgs::default()
                                            .size(FloatingActionButtonSize::Large)
                                            .on_click(move || {
                                                clicks.with_mut(|value| *value += 1);
                                            }),
                                        || {
                                            icon(&IconArgs::from(filled::home_icon()).size(
                                                FloatingActionButtonDefaults::icon_size(
                                                    FloatingActionButtonSize::Large,
                                                ),
                                            ));
                                        },
                                    );
                                    floating_action_button(&component_args);
                                });
                            },
                        );
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                    });

                    scope.child(move || {
                        text(
                            &TextArgs::default()
                                .text(format!("Clicks: {}", clicks.get()))
                                .size(Dp(14.0)),
                        )
                    });

                    scope.child(|| {
                        spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                    });

                    scope.child(|| {
                        text(&TextArgs::default().text("Disabled").size(Dp(14.0)));
                    });

                    scope.child(|| {
                        let component_args = FloatingActionButtonArgs::with_content(
                            FloatingActionButtonArgs::default().enabled(false),
                            || {
                                icon(&IconArgs::from(filled::home_icon()).size(
                                    FloatingActionButtonDefaults::icon_size(
                                        FloatingActionButtonSize::Standard,
                                    ),
                                ));
                            },
                        );
                        floating_action_button(&component_args);
                    });
                },
            );
        },
    ));
}
