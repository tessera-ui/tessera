use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    checkbox::checkbox,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    radio_button::{RadioGroupOrientation, radio_button, radio_group},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    switch::switch,
    text::text,
    theme::{MaterialTheme, material_theme},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, LayoutCoordinates, Modifier, RenderSlot, State,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

use super::super::{
    ACCENT, gap, selection_articles::option_label, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn overview_article() {
    super::prose(
        "ui-use-radio-buttons-not-switches-when-only-one-item-can-be-selected-from-a-list-labels-shoul",
    );
    gap(Dp(32.0));
    super::overview();
    gap(Dp(48.0));
    subheading("ui-what-s-new".into());
    gap(Dp(24.0));
    super::prose(
        "ui-radio-buttons-use-new-color-mappings-and-support-dynamic-color-selected-icons-use-primary",
    );
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "ui-color-changes-emphasize-the-selection-without-changing-the-label-these-native-choices-demo",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn usage() {
    context(
        RenderSlot::new(|| {
            super::prose(
                "ui-radio-buttons-are-the-recommended-way-to-make-a-single-selection-from-a-visible-list-of-op",
            );
        }),
        RenderSlot::new(language),
    );
    gap(Dp(48.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "ui-use-radio-buttons-to-select-one-option-from-a-set-and-expose-all-available-options-checkbo",
            );
        }),
        RenderSlot::new(notification_settings),
    );
    gap(Dp(48.0));
    subheading("ui-choose-the-right-selection-control".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            choice_card(0);
            choice_card(1);
            choice_card(2);
        });
    gap(Dp(24.0));
    super::prose(
        "ui-do-use-radio-buttons-for-a-single-choice-checkboxes-for-multiple-choices-and-switches-for",
    );
    gap(Dp(24.0));
    super::prose(
        "ui-use-radio-buttons-when-there-are-five-or-fewer-options-consider-a-drop-down-menu-when-savi",
    );
}

#[tessera]
pub(super) fn anatomy(labels: bool) {
    stage(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            if labels {
                let selected = remember(|| Some(0_usize));
                boxed()
                    .modifier(Modifier::new().offset(Dp(400.0), Dp(95.0)).width(Dp(240.0)))
                    .children(move || {
                        options(
                            "ui-radio-anatomy".into(),
                            vec!["ui-option-1".into(), "ui-option-2".into()],
                            selected,
                            true,
                            false,
                        );
                    });
                native_label("1".into(), 400.0, 42.0, 48.0);
                native_label("2".into(), 486.0, 42.0, 48.0);
                native_label("3".into(), 400.0, 230.0, 48.0);
            } else {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(488.0), Dp(100.0))
                            .size(Dp(48.0), Dp(48.0)),
                    )
                    .children(move || {
                        radio_button(true)
                            .accessibility_label(I18N.text("ui-radio-button-icon-anatomy"));
                    });
                spacer().modifier(
                    Modifier::new()
                        .offset(Dp(511.0), Dp(140.0))
                        .size(Dp(1.0), Dp(40.0))
                        .background(
                            use_context::<MaterialTheme>()
                                .unwrap()
                                .get()
                                .color_scheme
                                .outline,
                        ),
                );
                native_label("1".into(), 488.0, 180.0, 48.0);
            }
        }),
    );
    gap(Dp(16.0));
    if labels {
        super::prose("ui-1-selected-icon-2-adjacent-label-text-3-unselected-icon");
    } else {
        super::prose("ui-1-radio-button-icon");
    }
}

#[tessera]
pub(super) fn placement() {
    context(
        RenderSlot::new(|| {
            super::prose(
                "ui-radio-buttons-are-often-arranged-in-stacked-layouts-list-them-vertically-with-an-appropria",
            );
        }),
        RenderSlot::new(|| {
            language_settings();
        }),
    );
    gap(Dp(40.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            layout_comparison(false);
            layout_comparison(true);
        });
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            subheading("ui-adjacent-label-text".into());
            gap(Dp(16.0));
            super::prose(
                "ui-always-pair-a-radio-button-with-a-label-describing-what-it-selects-each-choice-needs-its-o",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn behavior() {
    super::prose(
        "ui-selecting-a-radio-button-deselects-the-previous-option-activating-the-selected-option-does",
    );
    gap(Dp(24.0));
    context(
        RenderSlot::new(|| {
            super::prose(
                "ui-apply-a-settings-change-immediately-when-appropriate-in-a-form-or-dialog-keep-a-draft-sele",
            );
        }),
        RenderSlot::new(language),
    );
}

#[tessera]
pub(super) fn accessibility(section: String) {
    match section.as_str() {
        "ui-use-cases" => {
            super::prose(
                "ui-people-using-assistive-technology-should-be-able-to-navigate-to-a-radio-button-select-a-ra",
            );
        }
        "ui-interaction-style" => {
            context(
                RenderSlot::new(|| {
                    super::prose(
                        "ui-a-radio-button-can-be-selected-or-unselected-selecting-one-deselects-the-others-a-group-ma",
                    );
                    gap(Dp(16.0));
                    super::prose(
                        "ui-to-let-people-opt-out-provide-a-not-applicable-or-no-option-choice-or-a-separate-clear-sel",
                    );
                }),
                RenderSlot::new(|| {
                    checkout(true).initial(0);
                }),
            );
            gap(Dp(32.0));
            subheading("ui-avoid-applying-density-by-default".into());
            gap(Dp(16.0));
            super::prose(
                "ui-keep-targets-at-least-48-48dp-by-default-let-people-explicitly-choose-a-denser-layout-or-t",
            );
        }
        "ui-initial-focus" => {
            super::prose(
                "ui-tab-enters-the-group-at-the-selected-radio-or-the-first-option-if-none-is-selected-shift-t",
            );
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    checkout(false).initial(1);
                    checkout(true);
                });
        }
        "ui-keyboard-navigation" => {
            spec_table(vec!["ui-keys".into(), "ui-actions".into()], vec![
                vec!["ui-tab".into(), "ui-enter-at-the-selected-radio-or-the-first-if-none-is-selected-tab-again-leaves-the-group".into()],
                vec!["ui-shift-tab".into(), "ui-enter-at-the-selected-radio-or-the-last-if-none-is-selected-move-backward-out-of-the-group".into()],
                vec!["ui-space".into(), "ui-select-the-focused-option".into()],
                vec!["ui-right-down-arrow".into(), "ui-focus-and-select-the-next-option-wrapping-to-the-first".into()],
                vec!["ui-left-up-arrow".into(), "ui-focus-and-select-the-previous-option-wrapping-to-the-last".into()],
            ]);
            gap(Dp(24.0));
            checkout(false).initial(0);
        }
        _ => {
            super::prose(
                "ui-give-the-group-a-category-name-and-each-radio-its-own-adjacent-label-assistive-technology",
            );
            gap(Dp(24.0));
            checkout(false).initial(0);
        }
    }
}

#[tessera]
pub(super) fn checkout(initial: Option<usize>, clear: bool) {
    let selected = remember(move || initial);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .background_with_shape(
                    scheme.secondary_container,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .padding_all(Dp(24.0)),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().width(Dp(360.0)).background(scheme.surface))
                .children(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .modifier(
                            Modifier::new()
                                .fill_max_width()
                                .padding_all(Dp(16.0))
                                .background(scheme.surface_container),
                        )
                        .children(move || {
                            icon().painter(filled::ARROW_BACK_SVG).size(Dp(24.0));
                            spacer().modifier(Modifier::new().width(Dp(16.0)));
                            text(I18N.text("ui-checkout")).style(type_style(22.0, 28.0));
                        });
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
                        .children(move || {
                            super::prose("ui-ship-to");
                            gap(Dp(16.0));
                            options(
                                "ui-ship-to".into(),
                                vec!["ui-home".into(), "ui-office".into()],
                                selected,
                                true,
                                false,
                            );
                            gap(Dp(16.0));
                            super::prose(
                                I18N.format(
                                    "ui-shipping-destination-value",
                                    [(
                                        "v0",
                                        I18N.text(
                                            &(match selected.get() {
                                                Some(0) => "ui-home",
                                                Some(_) => "ui-office",
                                                None => "ui-not-selected",
                                            }),
                                        )
                                        .into(),
                                    )],
                                ),
                            );
                            if clear {
                                button()
                                    .text()
                                    .accessibility_label(I18N.text("ui-clear-shipping-selection"))
                                    .on_click(move || selected.set(None))
                                    .child(move || {
                                        text(I18N.text("ui-clear-selection"));
                                    });
                            }
                        });
                });
        });
}

#[tessera]
pub(super) fn options(
    name: String,
    values: Vec<String>,
    selected: State<Option<usize>>,
    enabled: bool,
    horizontal: bool,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    radio_group()
        .accessibility_label(I18N.text(name))
        .orientation(if horizontal {
            RadioGroupOrientation::Horizontal
        } else {
            RadioGroupOrientation::Vertical
        })
        .content(move |group| {
            let values = values.clone();
            let content = RenderSlot::new(move || {
                for (index, value) in values.iter().enumerate() {
                    let value = value.clone();
                    let focus = remember(FocusRequester::new).get();
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            radio_button(selected.get() == Some(index))
                                .group(group)
                                .focus_requester(focus)
                                .enabled(enabled)
                                .accessibility_label(I18N.text(&(value)))
                                .on_select(move |_| selected.set(Some(index)));
                            option_label(
                                value.clone(),
                                Callback::new(move || {
                                    selected.set(Some(index));
                                    focus.request_focus();
                                }),
                                enabled,
                            )
                            .color(if enabled {
                                scheme.on_surface
                            } else {
                                scheme.on_surface.with_alpha(0.38)
                            })
                            .style(type_style(16.0, 24.0));
                        });
                }
            });
            if horizontal {
                flow_row().item_spacing(Dp(16.0)).children_shared(content);
            } else {
                column().children_shared(content);
            }
        });
}

#[tessera]
pub(super) fn language() {
    let draft = remember(|| Some(2_usize));
    let saved = remember(|| Some(2_usize));
    let status = remember(|| "ui-current-language-spanish".to_owned());
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.scrim.with_alpha(0.5),
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            column()
                .modifier(
                    Modifier::new()
                        .width(Dp(320.0))
                        .padding_all(Dp(24.0))
                        .background_with_shape(
                            scheme.surface_container_high,
                            Shape::rounded_rectangle(Dp(16.0)),
                        ),
                )
                .children(move || {
                    icon().painter(filled::LANGUAGE_SVG).size(Dp(24.0));
                    gap(Dp(12.0));
                    text(I18N.text("ui-choose-your-language")).style(type_style(20.0, 28.0));
                    super::prose("ui-you-can-always-change-this-later");
                    gap(Dp(16.0));
                    options(
                        "ui-choose-your-language".into(),
                        vec![
                            "ui-english".into(),
                            "ui-chinese-mandarin".into(),
                            "ui-spanish".into(),
                        ],
                        draft,
                        true,
                        false,
                    );
                    gap(Dp(16.0));
                    flow_row().item_spacing(Dp(12.0)).children(move || {
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-cancel-language-selection"))
                            .on_click(move || {
                                draft.set(saved.get());
                                status.set("ui-selection-restored".into());
                            })
                            .child(move || {
                                text(I18N.text("ui-cancel"));
                            });
                        button()
                            .text()
                            .accessibility_label(I18N.text("ui-confirm-language-selection"))
                            .on_click(move || {
                                saved.set(draft.get());
                                status.set(
                                    I18N.format(
                                        "ui-current-language-value",
                                        [(
                                            "v0",
                                            I18N.text(
                                                &(match draft.get() {
                                                    Some(0) => "ui-english",
                                                    Some(1) => "ui-chinese-mandarin",
                                                    _ => "ui-spanish",
                                                }),
                                            )
                                            .into(),
                                        )],
                                    ),
                                );
                            })
                            .child(move || {
                                text(I18N.text("ui-continue"));
                            });
                    });
                    gap(Dp(12.0));
                    super::prose(status.get());
                });
        });
}

#[tessera]
fn language_settings() {
    let selected = remember(|| Some(0_usize));
    card(
        "ui-settings".into(),
        RenderSlot::new(move || {
            super::prose("ui-language");
            gap(Dp(16.0));
            options(
                "ui-language".into(),
                vec![
                    "ui-english".into(),
                    "ui-chinese-mandarin".into(),
                    "ui-spanish".into(),
                ],
                selected,
                true,
                false,
            );
        }),
    );
}

#[tessera]
fn notification_settings() {
    let selected = remember(|| Some(0_usize));
    card(
        "ui-notifications".into(),
        RenderSlot::new(move || {
            options(
                "ui-notifications".into(),
                vec![
                    "ui-allow-notifications".into(),
                    "ui-turn-off-notifications".into(),
                ],
                selected,
                true,
                false,
            );
            gap(Dp(24.0));
            super::prose("ui-allow-access");
            check_option("ui-microphone-access".into());
            check_option("ui-location-access".into());
        }),
    );
}

#[tessera]
fn check_option(value: String) {
    let checked = remember(|| true);
    let focus = remember(FocusRequester::new).get();
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            checkbox(checked.get())
                .focus_requester(focus)
                .accessibility_label(I18N.text(&(value)))
                .on_toggle(move |value| checked.set(value));
            option_label(
                value.clone(),
                Callback::new(move || {
                    checked.set(!checked.get());
                    focus.request_focus();
                }),
                true,
            );
        });
}

#[tessera]
fn choice_card(kind: usize) {
    card(
        ["ui-filters", "ui-meal-options", "ui-settings"][kind].into(),
        RenderSlot::new(move || {
            if kind == 0 {
                let selected = remember(|| Some(0_usize));
                options(
                    "ui-sort-by".into(),
                    vec![
                        "ui-relevance".into(),
                        "ui-lowest-price".into(),
                        "ui-highest-rating".into(),
                        "ui-most-reviewed".into(),
                    ],
                    selected,
                    true,
                    false,
                );
            } else if kind == 1 {
                for value in ["ui-pickles", "ui-tomato", "ui-lettuce", "ui-cheese"] {
                    check_option(value.into());
                }
            } else {
                let checked = remember(|| true);
                super::prose("ui-allow-notifications");
                switch(checked.get())
                    .accessibility_label(I18N.text("ui-allow-notifications-independently"))
                    .on_toggle(move |value| checked.set(value));
            }
            gap(Dp(16.0));
            super::prose(
                [
                    "ui-do-one-selected-option",
                    "ui-do-multiple-independent-options",
                    "ui-do-one-setting-on-or-off",
                ][kind],
            );
        }),
    );
}

#[tessera]
fn layout_comparison(horizontal: bool) {
    let selected = remember(|| Some(0_usize));
    card(
        if horizontal {
            "ui-caution-horizontal-list".into()
        } else {
            "ui-do-stacked-list".into()
        },
        RenderSlot::new(move || {
            options(
                if horizontal {
                    "ui-horizontal-placement-example".into()
                } else {
                    "ui-stacked-placement-example".into()
                },
                vec![
                    "ui-option-1".into(),
                    "ui-option-2".into(),
                    "ui-option-3".into(),
                ],
                selected,
                true,
                horizontal,
            );
        }),
    );
}

#[tessera]
fn card(title: String, content: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(
            Modifier::new()
                .width(Dp(320.0))
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            text(I18N.text(&(title))).style(type_style(20.0, 28.0));
            gap(Dp(16.0));
            content.render();
        });
}

#[tessera]
pub(super) fn colors() {
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                stage(
                    1024.0,
                    220.0,
                    RenderSlot::new(move || {
                        for (index, selected) in [true, false].into_iter().enumerate() {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(
                                    Modifier::new()
                                        .offset(Dp(376.0 + index as f64 * 224.0), Dp(66.0))
                                        .size(Dp(48.0), Dp(48.0)),
                                )
                                .children(move || {
                                    radio_button(selected).accessibility_label(
                                        I18N.format(
                                            "ui-value-value-color-role",
                                            [
                                                (
                                                    "v0",
                                                    I18N.text(
                                                        &(if dark {
                                                            "ui-dark"
                                                        } else {
                                                            "ui-light"
                                                        }),
                                                    )
                                                    .into(),
                                                ),
                                                (
                                                    "v1",
                                                    I18N.text(
                                                        &(if selected {
                                                            "ui-selected"
                                                        } else {
                                                            "ui-unselected"
                                                        }),
                                                    )
                                                    .into(),
                                                ),
                                            ],
                                        ),
                                    );
                                });
                            native_label(
                                (index + 1).to_string(),
                                376.0 + index as f64 * 224.0,
                                142.0,
                                48.0,
                            );
                        }
                    }),
                );
            });
    }
    gap(Dp(16.0));
    super::prose(
        "ui-radio-button-color-roles-in-light-and-dark-themes-1-primary-2-on-surface-variant",
    );
    gap(Dp(40.0));
    context(
        RenderSlot::new(|| {
            subheading("ui-adjacent-text-label-color".into());
            gap(Dp(16.0));
            super::prose(
                "ui-use-on-surface-for-adjacent-text-labels-this-color-remains-the-same-when-the-label-or-radi",
            );
        }),
        RenderSlot::new(|| {
            checkout(false).initial(0);
        }),
    );
}

#[tessera]
pub(super) fn states() {
    super::prose(
        "ui-states-communicate-the-status-of-a-component-these-examples-retain-real-radio-controls-the",
    );
    gap(Dp(24.0));
    for dark in [false, true] {
        material_theme()
            .theme(move || MaterialTheme::from_seed(ACCENT, dark))
            .child(move || {
                stage(
                    1024.0,
                    300.0,
                    RenderSlot::new(move || {
                        for state in 0..5 {
                            native_label(
                                (state + 1).to_string(),
                                152.0 + state as f64 * 176.0,
                                34.0,
                                48.0,
                            );
                            for (row, selected) in [true, false].into_iter().enumerate() {
                                state_item(
                                    selected,
                                    state,
                                    dark,
                                    176.0 + state as f64 * 176.0,
                                    142.0 + row as f64 * 80.0,
                                );
                            }
                        }
                    }),
                );
            });
    }
    gap(Dp(16.0));
    super::prose("ui-1-enabled-2-hover-3-focused-4-pressed-5-disabled");
}

#[tessera]
fn state_item(initial: bool, state: usize, dark: bool, x: f64, y: f64) {
    let selected = remember(move || initial);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let color = if initial {
        scheme.primary
    } else {
        scheme.on_surface
    };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x - 24.0), Dp(y - 24.0))
                .size(Dp(48.0), Dp(48.0)),
        )
        .children(move || {
            if state == 1 || state == 3 {
                boxed().modifier(
                    Modifier::new()
                        .size(Dp(40.0), Dp(40.0))
                        .background_with_shape(
                            color.with_alpha(if state == 1 { 0.08 } else { 0.10 }),
                            Shape::Ellipse,
                        ),
                );
            }
            if state == 2 {
                boxed().modifier(Modifier::new().size(Dp(30.0), Dp(30.0)).border_with_shape(
                    Dp(2.0),
                    color,
                    Shape::Ellipse,
                ));
            }
            radio_button(selected.get())
                .enabled(state != 4)
                .accessibility_label(
                    I18N.format(
                        "ui-value-value-value-radio",
                        [
                            (
                                "v0",
                                I18N.text(&(if dark { "ui-dark" } else { "ui-light" }))
                                    .into(),
                            ),
                            (
                                "v1",
                                I18N.text(
                                    &(if initial {
                                        "ui-selected"
                                    } else {
                                        "ui-unselected"
                                    }),
                                )
                                .into(),
                            ),
                            (
                                "v2",
                                I18N.text(
                                    &([
                                        "ui-enabled-7077",
                                        "ui-hover-preview",
                                        "ui-focus-preview",
                                        "ui-pressed-preview",
                                        "ui-disabled-7078",
                                    ][state]),
                                )
                                .into(),
                            ),
                        ],
                    ),
                )
                .on_select(move |_| selected.set(true));
        });
}

#[tessera]
pub(super) fn measurements() {
    stage(
        1024.0,
        330.0,
        RenderSlot::new(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            for (index, selected) in [true, false].into_iter().enumerate() {
                let x = 376.0 + index as f64 * 224.0;
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(x), Dp(64.0))
                            .size(Dp(48.0), Dp(48.0))
                            .border(Dp(1.0), scheme.outline),
                    )
                    .children(move || {
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(40.0), Dp(40.0))
                                .background_with_shape(
                                    scheme.primary.with_alpha(0.10),
                                    Shape::Ellipse,
                                ),
                        );
                        radio_button(selected).accessibility_label(
                            I18N.format(
                                "ui-value-radio-dimensions",
                                [(
                                    "v0",
                                    I18N.text(
                                        &(if selected {
                                            "ui-selected"
                                        } else {
                                            "ui-unselected"
                                        }),
                                    )
                                    .into(),
                                )],
                            ),
                        );
                    });
                for (row, length) in [20.0, 40.0, 48.0].into_iter().enumerate() {
                    spacer().modifier(
                        Modifier::new()
                            .offset(Dp(x + 24.0 - length * 0.5), Dp(148.0 + row as f64 * 50.0))
                            .size(Dp(length), Dp(1.0))
                            .background(scheme.outline),
                    );
                }
            }
            native_label("ui-20dp-icon".into(), 680.0, 134.0, 200.0);
            native_label("ui-40dp-state-layer".into(), 680.0, 184.0, 200.0);
            native_label("ui-48-48dp-target".into(), 680.0, 234.0, 200.0);
        }),
    );
    gap(Dp(24.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-icon-diameter".into(), "20dp".into()],
            vec!["ui-stroke".into(), "2dp".into()],
            vec!["ui-selected-dot".into(), "10dp".into()],
            vec!["ui-state-layer".into(), "40dp".into()],
            vec!["ui-minimum-target".into(), "48 × 48dp".into()],
        ],
    );
}

#[tessera]
fn stage(width: f64, height: f64, content: RenderSlot) {
    let measured = remember(move || Dp(width));
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if measured.get() != next {
                    measured.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box((measured.get().0 / width) as f32).child(move || {
                boxed()
                    .modifier(
                        Modifier::new()
                            .required_size(Dp(width), Dp(height))
                            .background_with_shape(
                                scheme.surface,
                                Shape::rounded_rectangle(Dp(24.0)),
                            )
                            .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                    )
                    .children_shared(content);
            });
        });
}

#[tessera]
fn native_label(value: String, x: f64, y: f64, width: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(width), Dp(32.0)),
        )
        .children(move || {
            text(I18N.text(&(value)))
                .style(type_style(16.0, 24.0))
                .color(scheme.on_surface);
        });
}

#[tessera]
fn context(prose: RenderSlot, figure: RenderSlot) {
    let width = remember(|| Dp(1024.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            if width.get().0 < 760.0 {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        prose.render();
                        gap(Dp(24.0));
                        figure.render();
                    });
            } else {
                let track = (width.get().0 - 40.0) / 3.0;
                row().children(move || {
                    column()
                        .modifier(Modifier::new().width(Dp(track)))
                        .children_shared(prose);
                    spacer().modifier(Modifier::new().width(Dp(20.0)));
                    column()
                        .modifier(Modifier::new().width(Dp(track * 2.0 + 20.0)))
                        .children_shared(figure);
                });
            }
        });
}
