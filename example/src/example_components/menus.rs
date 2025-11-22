use std::sync::{Arc, Mutex};

use tessera_ui::{Color, DimensionValue, Dp, shard, tessera};
use tessera_ui_basic_components::{
    RippleState,
    alignment::CrossAxisAlignment,
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgsBuilder, column},
    menus::{
        MenuAnchor, MenuItemArgsBuilder, MenuPlacement, MenuProviderArgsBuilder, MenuState,
        menu_item, menu_provider,
    },
    row::{RowArgsBuilder, row},
    spacer::{SpacerArgsBuilder, spacer},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

#[derive(Clone)]
struct MenusShowcaseState {
    menu_state: MenuState,
    open_button_ripple: RippleState,
    item_ripples: [RippleState; 4],
    selected_label: Arc<Mutex<String>>,
    pinned: Arc<Mutex<bool>>,
}

impl MenusShowcaseState {
    fn new() -> Self {
        Self {
            menu_state: MenuState::new(),
            open_button_ripple: RippleState::new(),
            item_ripples: [
                RippleState::new(),
                RippleState::new(),
                RippleState::new(),
                RippleState::new(),
            ],
            selected_label: Arc::new(Mutex::new("None".to_string())),
            pinned: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for MenusShowcaseState {
    fn default() -> Self {
        Self::new()
    }
}

#[tessera]
#[shard]
pub fn menus_showcase(
    #[state(default_with = "MenusShowcaseState::new")] state: MenusShowcaseState,
) {
    let selected_label = state.selected_label.clone();
    let pinned = state.pinned.clone();
    // Anchor near the trigger button (padding 20dp + title/subtitle + spacer).
    let anchor = MenuAnchor::from_dp((Dp(20.0), Dp(72.0)), (Dp(180.0), Dp(48.0)));
    let menu_state = state.menu_state.clone();
    let item_ripples = state.item_ripples.clone();
    let selection_for_edit = selected_label.clone();
    let selection_for_share = selected_label.clone();
    let pin_state = pinned.clone();
    let pin_selection = selected_label.clone();
    let open_button_ripple = state.open_button_ripple.clone();

    surface(
        SurfaceArgsBuilder::default()
            .style(Color::WHITE.into())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .padding(Dp(20.0))
            .build()
            .expect("builder construction failed"),
        None,
        move || {
            menu_provider(
                MenuProviderArgsBuilder::default()
                    .placement(MenuPlacement::BelowStart)
                    .offset([Dp(0.0), Dp(6.0)])
                    .build()
                    .expect("builder construction failed"),
                menu_state.clone(),
                {
                    let selected_label = selected_label.clone();
                    let pinned = pinned.clone();
                    let menu_state_for_button = menu_state.clone();
                    move || {
                        column(
                            ColumnArgsBuilder::default()
                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                .build()
                                .expect("builder construction failed"),
                            |scope| {
                                scope.child(|| {
                                    text(
                                        TextArgsBuilder::default()
                                            .text("Menus Showcase")
                                            .size(Dp(20.0))
                                            .build()
                                            .expect("builder construction failed"),
                                    );
                                });

                                let selected_label_display = selected_label.lock().unwrap().clone();
                                let pinned_display = *pinned.lock().unwrap();
                                scope.child(move || {
                                    text(
                                        TextArgsBuilder::default()
                                            .text(format!(
                                                "Selected: {} | Pinned: {}",
                                                selected_label_display,
                                                if pinned_display { "Yes" } else { "No" }
                                            ))
                                            .size(Dp(14.0))
                                            .color(Color::GRAY)
                                            .build()
                                            .expect("builder construction failed"),
                                    );
                                });

                                scope.child(|| {
                                    spacer(
                                        SpacerArgsBuilder::default()
                                            .height(DimensionValue::Fixed(Dp(12.0).into()))
                                            .build()
                                            .expect("builder construction failed"),
                                    );
                                });

                                scope.child(move || {
                                    row(
                                        RowArgsBuilder::default()
                                            .width(DimensionValue::FILLED)
                                            .build()
                                            .expect("builder construction failed"),
                                        |row_scope| {
                                            let state_for_click = menu_state_for_button.clone();
                                            let ripple = open_button_ripple.clone();
                                            row_scope.child(move || {
                                                button(
                                                    ButtonArgsBuilder::default()
                                                        .width(DimensionValue::Fixed(
                                                            Dp(180.0).into(),
                                                        ))
                                                        .on_click(Arc::new(move || {
                                                            state_for_click.open_at(anchor);
                                                        }))
                                                        .build()
                                                        .expect("builder construction failed"),
                                                    ripple.clone(),
                                                    || {
                                                        text("Open anchored menu");
                                                    },
                                                );
                                            });

                                            row_scope.child(|| {
                                                spacer(
                                                    SpacerArgsBuilder::default()
                                                        .width(DimensionValue::Fixed(
                                                            Dp(12.0).into(),
                                                        ))
                                                        .build()
                                                        .expect("builder construction failed"),
                                                );
                                            });

                                            row_scope.child(|| {
                                                text(
                                                    TextArgsBuilder::default()
                                                        .text(
                                                            "Click to open at the button's anchor point.",
                                                        )
                                                        .size(Dp(14.0))
                                                        .color(Color::from_rgb(
                                                            0.25, 0.25, 0.28,
                                                        ))
                                                        .build()
                                                        .expect("builder construction failed"),
                                                );
                                            });
                                        },
                                    );
                                });
                            },
                        );
                    }
                },
                move |menu_scope| {
                    let ripple_edit = item_ripples[0].clone();
                    let ripple_share = item_ripples[1].clone();
                    let ripple_pin = item_ripples[2].clone();
                    let ripple_disabled = item_ripples[3].clone();
                    let menu_state_edit = menu_state.clone();
                    let menu_state_share = menu_state.clone();
                    let menu_state_pin = menu_state.clone();
                    let menu_state_disabled = menu_state.clone();

                    menu_scope.item(move || {
                        let menu_state = menu_state_edit.clone();
                        let selection_for_edit = selection_for_edit.clone();
                        let ripple = ripple_edit.clone();
                        menu_item(
                            MenuItemArgsBuilder::default()
                                .label("Revert")
                                .on_click(Arc::new(move || {
                                    *selection_for_edit.lock().unwrap() = "Revert".to_string();
                                }))
                                .build()
                                .expect("builder construction failed"),
                            Some(menu_state),
                            ripple,
                        );
                    });

                    menu_scope.item(move || {
                        let menu_state = menu_state_share.clone();
                        let selection_for_share = selection_for_share.clone();
                        let ripple = ripple_share.clone();
                        menu_item(
                            MenuItemArgsBuilder::default()
                                .label("Settings")
                                .on_click(Arc::new(move || {
                                    *selection_for_share.lock().unwrap() = "Settings".to_string();
                                }))
                                .build()
                                .expect("builder construction failed"),
                            Some(menu_state),
                            ripple,
                        );
                    });

                    menu_scope.item(move || {
                        let menu_state = menu_state_pin.clone();
                        let pin_state = pin_state.clone();
                        let pin_selection = pin_selection.clone();
                        let ripple = ripple_pin.clone();
                        let is_pinned = *pin_state.lock().unwrap();
                        menu_item(
                            MenuItemArgsBuilder::default()
                                .label("Send Feedback")
                                .selected(is_pinned)
                                .on_click(Arc::new(move || {
                                    let mut flag = pin_state.lock().unwrap();
                                    *flag = !*flag;
                                    *pin_selection.lock().unwrap() = if *flag {
                                        "Send Feedback".to_string()
                                    } else {
                                        "Unpinned".to_string()
                                    };
                                }))
                                .build()
                                .expect("builder construction failed"),
                            Some(menu_state),
                            ripple,
                        );
                    });

                    menu_scope.item(move || {
                        let menu_state = menu_state_disabled.clone();
                        let ripple = ripple_disabled.clone();
                        menu_item(
                            MenuItemArgsBuilder::default()
                                .label("Help")
                                .build()
                                .expect("builder construction failed"),
                            Some(menu_state),
                            ripple,
                        );
                    });
                },
            );
        },
    );
}
