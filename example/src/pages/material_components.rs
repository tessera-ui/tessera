//! Example page that showcases Material components not covered by the basic
//! page.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    badge::{badge, badge_with_content, badged_box},
    bottom_sheet::bottom_sheet_provider,
    boxed::boxed,
    button::button,
    button_groups::button_groups,
    card::card,
    chip::chip,
    column::column,
    date_picker::date_picker,
    dialog::{basic_dialog, dialog_provider},
    divider::{horizontal_divider, vertical_divider},
    floating_action_button::floating_action_button,
    flow_row::flow_row,
    icon::icon,
    lazy_list::lazy_column,
    material_icons::filled,
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::ModifierExt,
    navigation_bar::{navigation_bar, navigation_bar_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    radio_button::{RadioGroupOrientation, radio_button, radio_group},
    row::row,
    search::{docked_search_bar, search_bar},
    segmented_buttons::{
        SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
        single_choice_segmented_button_row,
    },
    snackbar::{SnackbarHostState, SnackbarRequest, snackbar_host},
    spacer::spacer,
    split_buttons::{split_button_layout, split_leading_button, split_trailing_button},
    tabs::{tab, tabs},
    text::text,
    text_field::{TextFieldBuilder, text_field},
    theme::MaterialTheme,
    time_picker::time_picker,
};
use tessera_shard::shard;
use tessera_ui::{Dp, Modifier, Px, PxPosition, remember, tessera, use_context};

#[tessera]
fn section_header(#[prop(into)] title: String, #[prop(into)] description: String) {
    let theme = use_context::<MaterialTheme>().unwrap();
    let title = title.clone();
    let description = description.clone();
    column().children(move || {
        text()
            .content(title.clone())
            .style(theme.with(|t| t.typography.title_medium));
        spacer().modifier(Modifier::new().height(Dp(6.0)));
        text()
            .content(description.clone())
            .style(theme.with(|t| t.typography.body_medium));
        spacer().modifier(Modifier::new().height(Dp(10.0)));
    });
}

#[shard]
pub fn material_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();

    let chip_selected = remember(|| false);
    let segmented_index = remember(|| 0_usize);
    let multi_segmented = remember(|| [false, false, false]);
    let tab_selected = remember(|| 0_usize);
    let radio_value = remember(|| 0_usize);
    let dialog_open = remember(|| false);
    let bottom_sheet_open = remember(|| false);
    let menu_controller = remember(MenuController::new);
    let group_status = remember(|| "No button group action yet".to_string());
    let split_status = remember(|| "No split action yet".to_string());
    let fab_status = remember(|| "No floating action yet".to_string());
    let snackbar_host_state = remember(SnackbarHostState::new);
    let navigation_bar_selected = remember(|| 0_usize);
    let navigation_rail_selected = remember(|| 0_usize);

    let page_content = move || {
        lazy_column()
            .modifier(Modifier::new().fill_max_size())
            .estimated_item_size(Dp(280.0))
            .content_padding(Dp(16.0))
            .item_spacing(Dp(24.0))
            .item(move || {
                column().children(move || {
                    text()
                        .content("Material Components")
                        .style(theme.with(|t| t.typography.headline_large));
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    text().content("Additional Material 3 components beyond the basic set.");
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Badge", "Show notifications or counts on top of an icon.");
                    row().children(move || {
                        badged_box()
                            .badge(|| {
                                badge_with_content().content(|| {
                                    text().content("3");
                                });
                            })
                            .content(|| {
                                icon().size(Dp(28.0)).painter(filled::HOME_SVG);
                            });
                        spacer().modifier(Modifier::new().width(Dp(24.0)));
                        badged_box()
                            .badge(|| {
                                badge();
                            })
                            .content(|| {
                                icon().size(Dp(28.0)).painter(filled::SETTINGS_SVG);
                            });
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Dialog & bottom sheet", "Open modal overlays from buttons.");
                    row().children(move || {
                        button()
                            .filled()
                            .on_click(move || dialog_open.set(true))
                            .child(|| {
                                text().content("Open dialog");
                            });
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        button()
                            .tonal()
                            .on_click(move || bottom_sheet_open.set(true))
                            .child(|| {
                                text().content("Open bottom sheet");
                            });
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Divider", "Separate content into distinct sections.");
                    text().content("Above the divider");
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    horizontal_divider();
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    text().content("Below the divider");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    row()
                        .modifier(Modifier::new().height(Dp(40.0)))
                        .children(|| {
                            text().content("Left");
                            spacer().modifier(Modifier::new().width(Dp(12.0)));
                            boxed()
                                .modifier(Modifier::new().height(Dp(40.0)))
                                .children(|| {
                                    vertical_divider();
                                });
                            spacer().modifier(Modifier::new().width(Dp(12.0)));
                            text().content("Right");
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Card", "Group related information into a container.");
                    card().filled().content(|| {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(|| {
                                text().content("Filled card");
                                text().content("Supporting text for the card.");
                            });
                    });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    card().outlined().content(|| {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(|| {
                                text().content("Outlined card");
                                text().content("Supporting text for the card.");
                            });
                    });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    card().elevated().content(|| {
                        column()
                            .modifier(Modifier::new().padding_all(Dp(16.0)))
                            .children(|| {
                                text().content("Elevated card");
                                text().content("Supporting text for the card.");
                            });
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Chip", "Compact actions, filters, and input tokens.");
                    flow_row()
                        .item_spacing(Dp(8.0))
                        .line_spacing(Dp(8.0))
                        .children(move || {
                            chip().assist("Assist").on_click(|| {});
                            chip().suggestion("Suggestion").on_click(|| {});
                            chip()
                                .filter("Filter")
                                .selected(chip_selected.get())
                                .on_click(move || {
                                    chip_selected.with_mut(|value| *value = !*value);
                                });
                            chip().input("Input");
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Button groups",
                        "Group related actions with connected behavior.",
                    );
                    button_groups()
                        .child(
                            || {
                                text().content("One");
                            },
                            move |active| {
                                group_status
                                    .set(format!("One is {}", if active { "on" } else { "off" }));
                            },
                        )
                        .child(
                            || {
                                text().content("Two");
                            },
                            move |active| {
                                group_status
                                    .set(format!("Two is {}", if active { "on" } else { "off" }));
                            },
                        )
                        .child(
                            || {
                                text().content("Three");
                            },
                            move |active| {
                                group_status
                                    .set(format!("Three is {}", if active { "on" } else { "off" }));
                            },
                        );
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    text().content(group_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Floating action button",
                        "Primary action shown above the content.",
                    );
                    floating_action_button()
                        .on_click(move || fab_status.set("Default FAB pressed".to_string()))
                        .content(|| {
                            icon().size(Dp(24.0)).painter(filled::ADD_SVG);
                        });
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    text().content(fab_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Segmented buttons",
                        "Switch between views with a connected control.",
                    );
                    single_choice_segmented_button_row().content(move || {
                        segmented_button()
                            .label("Day")
                            .selected(segmented_index.get() == 0)
                            .shape(SegmentedButtonDefaults::item_shape(
                                0,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || segmented_index.set(0));
                        segmented_button()
                            .label("Week")
                            .selected(segmented_index.get() == 1)
                            .shape(SegmentedButtonDefaults::item_shape(
                                1,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || segmented_index.set(1));
                        segmented_button()
                            .label("Month")
                            .selected(segmented_index.get() == 2)
                            .shape(SegmentedButtonDefaults::item_shape(
                                2,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || segmented_index.set(2));
                    });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    text().content("Multiple selection");
                    multi_choice_segmented_button_row().content(move || {
                        segmented_button()
                            .label("News")
                            .selected(multi_segmented.get()[0])
                            .shape(SegmentedButtonDefaults::item_shape(
                                0,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || multi_segmented.with_mut(|v| v[0] = !v[0]));
                        segmented_button()
                            .label("Mail")
                            .selected(multi_segmented.get()[1])
                            .shape(SegmentedButtonDefaults::item_shape(
                                1,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || multi_segmented.with_mut(|v| v[1] = !v[1]));
                        segmented_button()
                            .label("Social")
                            .selected(multi_segmented.get()[2])
                            .shape(SegmentedButtonDefaults::item_shape(
                                2,
                                3,
                                SegmentedButtonDefaults::shape(),
                            ))
                            .on_click(move || multi_segmented.with_mut(|v| v[2] = !v[2]));
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Tabs", "Organize related destinations into a tab strip.");
                    tabs().selected(tab_selected).content(move || {
                        tab()
                            .label("Flights")
                            .selected(tab_selected.get() == 0)
                            .on_click(move || tab_selected.set(0));
                        tab()
                            .label("Hotels")
                            .selected(tab_selected.get() == 1)
                            .on_click(move || tab_selected.set(1));
                        tab()
                            .label("Trains")
                            .selected(tab_selected.get() == 2)
                            .on_click(move || tab_selected.set(2));
                    });
                    text().content(match tab_selected.get() {
                        0 => "Flights content",
                        1 => "Hotels content",
                        _ => "Trains content",
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Split button",
                        "Pair a primary action with a secondary action.",
                    );
                    split_button_layout()
                        .leading_button(move || {
                            split_leading_button()
                                .filled()
                                .on_click(move || split_status.set("Send pressed".to_string()))
                                .content(|| {
                                    text().content("Send");
                                });
                        })
                        .trailing_button(move || {
                            split_trailing_button()
                                .filled()
                                .on_click(move || split_status.set("More pressed".to_string()))
                                .content(|| {
                                    text().content("More");
                                });
                        });
                    text().content(split_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Radio button", "Pick a single option from a group.");
                    radio_group()
                        .orientation(RadioGroupOrientation::Vertical)
                        .content(move || {
                            column().children(move || {
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        radio_button()
                                            .selected(radio_value.get() == 0)
                                            .on_select(move |_| radio_value.set(0))
                                            .accessibility_label("Option A");
                                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                                        text()
                                            .content("Option A")
                                            .modifier(Modifier::new().offset(Dp(0.0), Dp(3.0)));
                                    });
                                spacer().modifier(Modifier::new().height(Dp(8.0)));
                                row()
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        radio_button()
                                            .selected(radio_value.get() == 1)
                                            .on_select(move |_| radio_value.set(1))
                                            .accessibility_label("Option B");
                                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                                        text()
                                            .content("Option B")
                                            .modifier(Modifier::new().offset(Dp(0.0), Dp(3.0)));
                                    });
                            });
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Text field",
                        "Capture short-form text with Material styling.",
                    );
                    TextFieldBuilder::filled()
                        .label("Filled name")
                        .placeholder("Enter your name");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    TextFieldBuilder::outlined()
                        .label("Outlined city")
                        .placeholder("Enter a city");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    TextFieldBuilder::secure().label("Secure password");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    text_field()
                        .label("Error email")
                        .is_error(true)
                        .initial_text("invalid@");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    text_field()
                        .label("Disabled field")
                        .enabled(false)
                        .initial_text("Unavailable");
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    text_field()
                        .label("Read-only field")
                        .read_only(true)
                        .initial_text("Reference value");
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Snackbar", "Show brief messages about app processes.");
                    button()
                        .tonal()
                        .on_click(move || {
                            snackbar_host_state.with_mut(|host| {
                                let mut request = SnackbarRequest::new("Saved to your library.");
                                request.action_label = Some("Undo".to_string());
                                request.with_dismiss_action = true;
                                host.show_snackbar(request);
                            });
                        })
                        .child(|| {
                            text().content("Show snackbar");
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Menu", "Show a list of actions in a temporary surface.");
                    // The menu itself is hosted by the page-level
                    // `menu_provider`; the trigger only marks the anchor.
                    button()
                        .outlined()
                        .on_click(move || {
                            menu_controller.with_mut(|menu| {
                                menu.open_at(MenuAnchor::at(PxPosition::new(Px(16), Px(16))));
                            });
                        })
                        .child(|| {
                            text().content("Open menu");
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Search", "Embed docked and expanded search fields.");
                    let query = remember(String::new);
                    docked_search_bar()
                        .placeholder("Search components")
                        .on_query_change(move |value| {
                            query.set(value.clone());
                            value
                        })
                        .content(move || {
                            text().content(if query.get().is_empty() {
                                "Type to search".to_string()
                            } else {
                                format!("Results for {}", query.get())
                            });
                        });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    search_bar().placeholder("Search all examples").content(|| {
                        text().content("Expanded search results");
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Date picker", "Select a date from a calendar.");
                    date_picker();
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Time picker", "Select a time with steppers.");
                    time_picker();
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Navigation bar", "Switch between primary destinations.");
                    navigation_bar().content(move || {
                        navigation_bar_item()
                            .label("Home")
                            .selected(navigation_bar_selected.get() == 0)
                            .on_click(move || navigation_bar_selected.set(0))
                            .icon(|| {
                                icon().size(Dp(24.0)).painter(filled::HOME_SVG);
                            });
                        navigation_bar_item()
                            .label("Search")
                            .selected(navigation_bar_selected.get() == 1)
                            .on_click(move || navigation_bar_selected.set(1))
                            .icon(|| {
                                icon().size(Dp(24.0)).painter(filled::SEARCH_SVG);
                            });
                        navigation_bar_item()
                            .label("Settings")
                            .selected(navigation_bar_selected.get() == 2)
                            .on_click(move || navigation_bar_selected.set(2))
                            .icon(|| {
                                icon().size(Dp(24.0)).painter(filled::SETTINGS_SVG);
                            });
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Navigation rail",
                        "Vertical destinations for larger screens.",
                    );
                    boxed()
                        .modifier(Modifier::new().height(Dp(320.0)))
                        .children(move || {
                            navigation_rail().content(move || {
                                navigation_rail_item()
                                    .label("Home")
                                    .selected(navigation_rail_selected.get() == 0)
                                    .on_click(move || navigation_rail_selected.set(0))
                                    .icon(|| {
                                        icon().size(Dp(24.0)).painter(filled::HOME_SVG);
                                    });
                                navigation_rail_item()
                                    .label("Search")
                                    .selected(navigation_rail_selected.get() == 1)
                                    .on_click(move || navigation_rail_selected.set(1))
                                    .icon(|| {
                                        icon().size(Dp(24.0)).painter(filled::SEARCH_SVG);
                                    });
                                navigation_rail_item()
                                    .label("Settings")
                                    .selected(navigation_rail_selected.get() == 2)
                                    .on_click(move || navigation_rail_selected.set(2))
                                    .icon(|| {
                                        icon().size(Dp(24.0)).painter(filled::SETTINGS_SVG);
                                    });
                            });
                        });
                });
            })
    };

    boxed()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            dialog_provider()
                .is_open(dialog_open.get())
                .on_close_request(move || dialog_open.set(false))
                .main_content(move || {
                    bottom_sheet_provider()
                        .is_open(bottom_sheet_open.get())
                        .on_close_request(move || bottom_sheet_open.set(false))
                        .main_content(move || {
                            // Hosting the menu above the whole page keeps it
                            // on top of later list items, like a Compose popup.
                            menu_provider()
                                .placement(MenuPlacement::BelowStart)
                                .controller(menu_controller)
                                .main_content(move || {
                                    // The snackbar host is drawn after the page
                                    // so it overlays the bottom edge.
                                    boxed().modifier(Modifier::new().fill_max_size()).children(
                                        move || {
                                            page_content();
                                            snackbar_host()
                                                .modifier(
                                                    Modifier::new()
                                                        .align(Alignment::BottomCenter)
                                                        .padding_all(Dp(16.0)),
                                                )
                                                .state(snackbar_host_state);
                                        },
                                    );
                                })
                                .menu_content(move || {
                                    let close =
                                        move || menu_controller.with_mut(|menu| menu.close());
                                    menu_item().label("Edit").on_click(close);
                                    menu_item().label("Duplicate").on_click(close);
                                    menu_item().label("Delete").on_click(close);
                                });
                        })
                        .bottom_sheet_content(move || {
                            column()
                                .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)))
                                .children(|| {
                                    text().content("Bottom sheet");
                                    text()
                                        .content("Contextual actions and information appear here.");
                                });
                        });
                })
                .dialog_content(move || {
                    basic_dialog()
                        .headline("Discard draft?")
                        .supporting_text("This action cannot be undone.")
                        .confirm_button(move || {
                            button()
                                .text()
                                .on_click(move || dialog_open.set(false))
                                .child(|| {
                                    text().content("Discard");
                                });
                        })
                        .dismiss_button(move || {
                            button()
                                .text()
                                .on_click(move || dialog_open.set(false))
                                .child(|| {
                                    text().content("Cancel");
                                });
                        });
                });
        });
}
