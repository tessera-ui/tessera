use tessera_components::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    bottom_sheet::{
        BottomSheetController, BottomSheetProviderArgs, BottomSheetStyle, bottom_sheet_provider,
    },
    button::{ButtonArgs, button},
    checkbox::{CheckboxArgs, checkbox},
    column::{ColumnArgs, column},
    dialog::{DialogController, DialogProviderArgs, DialogStyle, dialog_provider},
    icon::{IconArgs, icon},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    list_item::{ListItemArgs, list_item},
    material_icons::filled,
    menus::{
        MenuAnchor, MenuController, MenuItemArgs, MenuPlacement, MenuProviderArgs, menu_provider,
    },
    modifier::ModifierExt as _,
    navigation_bar::{
        NavigationBarArgs, NavigationBarController, NavigationBarItem, navigation_bar,
    },
    pager::{PagerArgs, PagerController, PagerPageSize, horizontal_pager},
    radio_button::{
        RadioButtonArgs, RadioGroupArgs, RadioGroupOrientation, radio_button, radio_group,
    },
    row::{RowArgs, row},
    segmented_buttons::{
        SegmentedButtonArgs, SegmentedButtonDefaults, SegmentedButtonRowArgs,
        multi_choice_segmented_button_row, segmented_button, single_choice_segmented_button_row,
    },
    shape_def::Shape,
    side_sheet::{SideSheetController, SideSheetProviderArgs, modal_side_sheet_provider},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, SurfaceStyle, surface},
    switch::{SwitchArgs, SwitchController, switch},
    tabs::{TabsArgs, TabsVariant, tabs},
    text::{TextArgs, text},
    text_input::{TextInputArgs, TextInputController, text_input},
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, Modifier, Prop, RenderSlot, State, remember, retain, shard, tessera, use_context,
};

#[shard]
pub fn focus_navigation_showcase() {
    let dialog_controller = remember(DialogController::default);
    let bottom_sheet_controller = remember(BottomSheetController::default);
    let side_sheet_controller = remember(SideSheetController::default);

    modal_side_sheet_provider(
        &SideSheetProviderArgs::new(move || {
            side_sheet_controller.with_mut(|controller| controller.close());
        })
        .controller(side_sheet_controller)
        .main_content(move || {
            bottom_sheet_provider(
                &BottomSheetProviderArgs::new(move || {
                    bottom_sheet_controller.with_mut(|controller| controller.close());
                })
                .style(BottomSheetStyle::Material)
                .controller(bottom_sheet_controller)
                .main_content(move || {
                    dialog_provider(
                        &DialogProviderArgs::new(move || {
                            dialog_controller.with_mut(|controller| controller.close());
                        })
                        .style(DialogStyle::Material)
                        .controller(dialog_controller)
                        .main_content(move || {
                            focus_navigation_page(&FocusNavigationPageArgs {
                                dialog_controller,
                                bottom_sheet_controller,
                                side_sheet_controller,
                            });
                        })
                        .dialog_content(move || {
                            dialog_focus_content(&DialogFocusContentArgs { dialog_controller });
                        }),
                    );
                })
                .bottom_sheet_content(move || {
                    bottom_sheet_focus_content(&BottomSheetFocusContentArgs {
                        bottom_sheet_controller,
                    });
                }),
            );
        })
        .side_sheet_content(move || {
            side_sheet_focus_content(&SideSheetFocusContentArgs {
                side_sheet_controller,
            });
        }),
    );
}

#[derive(Clone, Prop)]
struct FocusNavigationPageArgs {
    dialog_controller: State<DialogController>,
    bottom_sheet_controller: State<BottomSheetController>,
    side_sheet_controller: State<SideSheetController>,
}

#[tessera]
fn focus_navigation_page(args: &FocusNavigationPageArgs) {
    let dialog_controller = args.dialog_controller;
    let bottom_sheet_controller = args.bottom_sheet_controller;
    let side_sheet_controller = args.side_sheet_controller;
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            let page_controller = retain(LazyListController::new);
            let action_count = remember(|| 0usize);
            let checkbox_value = remember(|| true);
            let switch_controller = remember(|| SwitchController::new(false));
            let selected_list_item = remember(|| "None".to_string());
            let radio_selection = remember(|| 0usize);
            let segmented_single = remember(|| 0usize);
            let segmented_multi = remember(|| [true, false, false]);
            let pager_controller = remember(|| PagerController::new(0));
            let reveal_selection = remember(|| "Row 1".to_string());

            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size())
                    .controller(page_controller)
                    .content_padding(Dp(24.0))
                    .estimated_item_size(Dp(1600.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .content(move |scope| {
                        scope.item(move || {
                            column(
                                    &ColumnArgs::default()
                                        .modifier(Modifier::new().fill_max_width())
                                        .cross_axis_alignment(CrossAxisAlignment::Start)
                                        .children(move |scope| {
                                            scope.child(|| {
                                                text(
                                                    &TextArgs::default()
                                                        .text("Focus & Keyboard Navigation")
                                                        .size(Dp(24.0)),
                                                );
                                            });
                                            scope.child(|| {
                                                text(
                                                    &TextArgs::default()
                                                        .text("Use Tab and Shift+Tab for global traversal. Use arrows inside radio groups, segmented rows, tabs, pagers, menus, and navigation bars. Open the modal surfaces to verify trap and restore behavior.")
                                                        .size(Dp(14.0))
                                                        .color(section_supporting_color()),
                                                );
                                            });
                                            scope.child(|| {
                                                spacer(&SpacerArgs::new(
                                                    Modifier::new().height(Dp(20.0)),
                                                ));
                                            });
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Traversal Basics".into(),
                                                    description: "These controls should participate in the global focus order and activate from Enter or Space.".into(),
                                                    child: RenderSlot::new(move || {
                                                        column(
                                                            &ColumnArgs::default()
                                                                .modifier(Modifier::new().fill_max_width())
                                                                .cross_axis_alignment(CrossAxisAlignment::Start)
                                                                .children(move |section_scope| {
                                                                    section_scope.child(move || {
                                                                        row(&RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center).children(move |row_scope| {
                                                                            row_scope.child(move || {
                                                                                demo_button(&DemoButtonArgs {
                                                                                    label: "Primary Action".into(),
                                                                                    on_click: Callback::new(move || {
                                                                                        action_count.with_mut(|count| *count += 1);
                                                                                    }),
                                                                                });
                                                                            });
                                                                            row_scope.child(|| {
                                                                                spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))));
                                                                            });
                                                                            row_scope.child(move || {
                                                                                text(&TextArgs::default().text(format!("Button activations: {}", action_count.get())));
                                                                            });
                                                                        }));
                                                                    });
                                                                    section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));
                                                                    section_scope.child(move || {
                                                                        row(&RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center).children(move |row_scope| {
                                                                            row_scope.child(move || {
                                                                                checkbox(&CheckboxArgs::default().checked(checkbox_value.get()).on_toggle(move |value| {
                                                                                    checkbox_value.set(value);
                                                                                }));
                                                                            });
                                                                            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0)))));
                                                                            row_scope.child(move || {
                                                                                text(&TextArgs::default().text(format!("Checkbox is {}", if checkbox_value.get() { "checked" } else { "unchecked" })));
                                                                            });
                                                                        }));
                                                                    });
                                                                    section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));
                                                                    section_scope.child(move || {
                                                                        row(&RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center).children(move |row_scope| {
                                                                            row_scope.child(move || {
                                                                                switch(&SwitchArgs::default().controller(switch_controller).on_toggle(|_| {}));
                                                                            });
                                                                            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0)))));
                                                                            row_scope.child(move || {
                                                                                text(&TextArgs::default().text(format!("Switch is {}", if switch_controller.with(|controller| controller.is_checked()) { "on" } else { "off" })));
                                                                            });
                                                                        }));
                                                                    });
                                                                    section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));
                                                                    section_scope.child(move || {
                                                                        list_item(
                                                                            &ListItemArgs::new("Focusable list row")
                                                                                .supporting_text(format!("Selected item: {}", selected_list_item.get()))
                                                                                .on_click(move || {
                                                                                    selected_list_item.set("Focusable list row".to_string());
                                                                                })
                                                                                .leading(|| {
                                                                                    icon(&IconArgs::default().vector(filled::LIST_SVG).size(Dp(20.0)));
                                                                                }),
                                                                        );
                                                                    });
                                                                }),
                                                        );
                                                    }),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Radio Group".into(),
                                                    description: "Use Up and Down to move through the group. Focus movement should also update the selected radio.".into(),
                                                    child: RenderSlot::new(move || {
                                                        column(&ColumnArgs::default().modifier(Modifier::new().fill_max_width()).cross_axis_alignment(CrossAxisAlignment::Start).children(move |section_scope| {
                                                            section_scope.child(move || {
                                                                radio_group(
                                                                    &RadioGroupArgs::default()
                                                                        .orientation(RadioGroupOrientation::Vertical)
                                                                        .content(move || {
                                                                            column(&ColumnArgs::default().cross_axis_alignment(CrossAxisAlignment::Start).children(move |radio_scope| {
                                                                                radio_scope.child(move || radio_option_row(&RadioOptionRowArgs { label: "Inbox".to_string(), selected_index: radio_selection, index: 0 }));
                                                                                radio_scope.child(move || radio_option_row(&RadioOptionRowArgs { label: "Mentions".to_string(), selected_index: radio_selection, index: 1 }));
                                                                                radio_scope.child(move || radio_option_row(&RadioOptionRowArgs { label: "Archived".to_string(), selected_index: radio_selection, index: 2 }));
                                                                            }));
                                                                        }),
                                                                );
                                                            });
                                                            section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));
                                                            section_scope.child(move || {
                                                                let label = match radio_selection.get() { 0 => "Inbox", 1 => "Mentions", _ => "Archived" };
                                                                text(&TextArgs::default().text(format!("Selected radio: {label}")));
                                                            });
                                                        }));
                                                    }),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Tabs, Segmented Rows, And Navigation".into(),
                                                    description: "Tabs and single-choice segmented rows should behave like roving-focus groups. Navigation bar should cycle with Left and Right.".into(),
                                                    child: RenderSlot::new(move || {
                                                        let base_shape = SegmentedButtonDefaults::shape();
                                                        let navigation_bar_controller = remember(|| NavigationBarController::new(0));
                                                        column(&ColumnArgs::default().modifier(Modifier::new().fill_max_width()).cross_axis_alignment(CrossAxisAlignment::Start).children(move |section_scope| {
                                                            section_scope.child(move || {
                                                                tabs(
                                                                    &TabsArgs::default()
                                                                        .modifier(Modifier::new().fill_max_width())
                                                                        .variant(TabsVariant::Primary)
                                                                        .children(|tabs_scope| {
                                                                            tabs_scope.child_label("Overview", || text(&TextArgs::from("Overview content")));
                                                                            tabs_scope.child_label("Activity", || text(&TextArgs::from("Activity content")));
                                                                            tabs_scope.child_label("Files", || text(&TextArgs::from("Files content")));
                                                                        }),
                                                                );
                                                            });
                                                            section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                                            section_scope.child(move || {
                                                                single_choice_segmented_button_row(
                                                                    &SegmentedButtonRowArgs::default().content(move || {
                                                                        for (index, label) in ["Day", "Week", "Month"].iter().enumerate() {
                                                                            segmented_button(
                                                                                &SegmentedButtonArgs::new((*label).to_string())
                                                                                    .selected(segmented_single.get() == index)
                                                                                    .shape(SegmentedButtonDefaults::item_shape(index, 3, base_shape))
                                                                                    .on_click(move || segmented_single.set(index)),
                                                                            );
                                                                        }
                                                                    }),
                                                                );
                                                            });
                                                            section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0)))));
                                                            section_scope.child(move || {
                                                                text(&TextArgs::default().text(format!("Single choice: {}", ["Day", "Week", "Month"][segmented_single.get()])));
                                                            });
                                                            section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                                            section_scope.child(move || {
                                                                multi_choice_segmented_button_row(
                                                                    &SegmentedButtonRowArgs::default().content(move || {
                                                                        for (index, label) in ["Email", "Push", "SMS"].iter().enumerate() {
                                                                            segmented_button(
                                                                                &SegmentedButtonArgs::new((*label).to_string())
                                                                                    .selected(segmented_multi.with(|values| values[index]))
                                                                                    .shape(SegmentedButtonDefaults::item_shape(index, 3, base_shape))
                                                                                    .on_click(move || {
                                                                                        segmented_multi.with_mut(|values| values[index] = !values[index]);
                                                                                    }),
                                                                            );
                                                                        }
                                                                    }),
                                                                );
                                                            });
                                                            section_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                                            section_scope.child(move || {
                                                                surface(&SurfaceArgs::with_child(
                                                                    SurfaceArgs::default().modifier(Modifier::new().fill_max_width()).shape(Shape::rounded_rectangle(Dp(20.0))),
                                                                    move || {
                                                                        navigation_bar(
                                                                            &NavigationBarArgs::default()
                                                                                .controller(navigation_bar_controller)
                                                                                .item(NavigationBarItem::new("Home").icon(|| icon(&IconArgs::default().vector(filled::HOME_SVG).size(Dp(20.0)))))
                                                                                .item(NavigationBarItem::new("Search").icon(|| icon(&IconArgs::default().vector(filled::SEARCH_SVG).size(Dp(20.0)))))
                                                                                .item(NavigationBarItem::new("Profile").icon(|| icon(&IconArgs::default().vector(filled::PERSON_SVG).size(Dp(20.0))))),
                                                                        );
                                                                    },
                                                                ));
                                                            });
                                                        }));
                                                    }),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Menus And Submenus".into(),
                                                    description: "Open the menu, use Up and Down to move, Right to enter the submenu, and Left to return.".into(),
                                                    child: RenderSlot::new(submenu_demo_section),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Pager Navigation".into(),
                                                    description: "Tab to the pager viewport, then use Left and Right, PageUp and PageDown, or Home and End to change pages.".into(),
                                                    child: RenderSlot::new(move || {
                                                        pager_demo_section(&PagerDemoSectionArgs {
                                                            controller: pager_controller,
                                                        });
                                                    }),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Modal Focus Trap".into(),
                                                    description: "Open the dialog, bottom sheet, and side sheet. Tab should stay inside each surface until you close it.".into(),
                                                    child: RenderSlot::new(move || {
                                                        row(&RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center).children(move |row_scope| {
                                                            row_scope.child(move || {
                                                                demo_button(&DemoButtonArgs {
                                                                    label: "Open Dialog".into(),
                                                                    on_click: Callback::new(move || {
                                                                        dialog_controller.with_mut(|controller| controller.open());
                                                                    }),
                                                                });
                                                            });
                                                            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0)))));
                                                            row_scope.child(move || {
                                                                demo_button(&DemoButtonArgs {
                                                                    label: "Open Bottom Sheet".into(),
                                                                    on_click: Callback::new(move || {
                                                                        bottom_sheet_controller.with_mut(|controller| controller.open());
                                                                    }),
                                                                });
                                                            });
                                                            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0)))));
                                                            row_scope.child(move || {
                                                                demo_button(&DemoButtonArgs {
                                                                    label: "Open Side Sheet".into(),
                                                                    on_click: Callback::new(move || {
                                                                        side_sheet_controller.with_mut(|controller| controller.open());
                                                                    }),
                                                                });
                                                            });
                                                        }));
                                                    }),
                                                });
                                            });
                                            scope.child(|| spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0)))));
                                            scope.child(move || {
                                                showcase_section(&ShowcaseSectionArgs {
                                                    title: "Lazy Reveal".into(),
                                                    description: "Tab through this inner lazy column until focus moves beyond the viewport. The container should reveal the focused row automatically.".into(),
                                                    child: RenderSlot::new(move || {
                                                        reveal_demo_section(&RevealDemoSectionArgs {
                                                            selected_row: reveal_selection,
                                                        });
                                                    }),
                                                });
                                            });
                                        }),
                                );
                        });
                    }),
            );
        },
    ));
}

#[derive(Clone, Prop)]
struct ShowcaseSectionArgs {
    #[prop(into)]
    title: String,
    #[prop(into)]
    description: String,
    child: RenderSlot,
}

#[tessera]
fn showcase_section(args: &ShowcaseSectionArgs) {
    let container_color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .surface_container_low;
    let title = args.title.clone();
    let description = args.description.clone();
    let child = args.child.clone();
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(16.0)))
            .shape(Shape::rounded_rectangle(Dp(24.0)))
            .style(SurfaceStyle::Filled {
                color: container_color,
            }),
        move || {
            let title = title.clone();
            let description = description.clone();
            let child = child.clone();
            column(
                &ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .children(move |scope| {
                        scope.child({
                            let title = title.clone();
                            move || {
                                text(&TextArgs::default().text(title.clone()).size(Dp(18.0)));
                            }
                        });
                        scope.child({
                            let description = description.clone();
                            move || {
                                text(
                                    &TextArgs::default()
                                        .text(description.clone())
                                        .size(Dp(14.0))
                                        .color(section_supporting_color()),
                                );
                            }
                        });
                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                        });
                        scope.child(move || {
                            child.render();
                        });
                    }),
            );
        },
    ));
}

fn section_supporting_color() -> tessera_ui::Color {
    use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .on_surface_variant
}

#[derive(Clone, Prop)]
struct DemoButtonArgs {
    #[prop(into)]
    label: String,
    on_click: Callback,
}

#[tessera]
fn demo_button(args: &DemoButtonArgs) {
    let label = args.label.clone();
    let on_click = args.on_click.clone();
    button(&ButtonArgs::with_child(
        ButtonArgs::filled(move || {
            on_click.call();
        }),
        move || {
            text(&TextArgs::from(label.clone()));
        },
    ));
}

#[derive(Clone, Prop)]
struct RadioOptionRowArgs {
    #[prop(into)]
    label: String,
    selected_index: State<usize>,
    index: usize,
}

#[tessera]
fn radio_option_row(args: &RadioOptionRowArgs) {
    let label = args.label.clone();
    let selected_index = args.selected_index;
    let index = args.index;
    row(&RowArgs::default()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move |scope| {
            scope.child({
                let label = label.clone();
                move || {
                    radio_button(
                        &RadioButtonArgs::default()
                            .selected(selected_index.get() == index)
                            .accessibility_label(label.clone())
                            .on_select({
                                let selected_index = selected_index;
                                move |_| {
                                    selected_index.set(index);
                                }
                            }),
                    );
                }
            });
            scope.child(|| {
                spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))));
            });
            scope.child(move || {
                text(&TextArgs::default().text(label.clone()));
            });
        }));
}

#[tessera]
fn submenu_demo_section() {
    let menu_controller = remember(MenuController::new);
    let selected_path = remember(|| "None".to_string());
    let anchor = MenuAnchor::from_dp((Dp::ZERO, Dp::ZERO), (Dp(200.0), Dp(40.0)));

    menu_provider(
        &MenuProviderArgs::default()
            .placement(MenuPlacement::BelowStart)
            .offset([Dp::ZERO, Dp(8.0)])
            .controller(menu_controller)
            .main_content(move || {
                column(
                    &ColumnArgs::default()
                        .modifier(Modifier::new().fill_max_width())
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children(move |scope| {
                            scope.child(move || {
                                button(&ButtonArgs::with_child(
                                    ButtonArgs::filled(move || {
                                        menu_controller
                                            .with_mut(|controller| controller.open_at(anchor));
                                    })
                                    .modifier(Modifier::new().width(Dp(200.0))),
                                    || {
                                        text(&TextArgs::from("Open nested menu"));
                                    },
                                ));
                            });
                            scope.child(|| {
                                spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                            });
                            scope.child(move || {
                                text(
                                    &TextArgs::default()
                                        .text(format!("Last action: {}", selected_path.get())),
                                );
                            });
                        }),
                );
            })
            .menu_content(move |scope| {
                scope.menu_item(&MenuItemArgs::default().label("Sort").submenu_content(
                    move |submenu_scope| {
                        submenu_scope.menu_item(
                            &MenuItemArgs::default().label("By Name").on_click(move || {
                                selected_path.set("Sort / By Name".to_string());
                            }),
                        );
                        submenu_scope.menu_item(
                            &MenuItemArgs::default().label("By Date").on_click(move || {
                                selected_path.set("Sort / By Date".to_string());
                            }),
                        );
                        submenu_scope.menu_item(
                            &MenuItemArgs::default().label("By Size").on_click(move || {
                                selected_path.set("Sort / By Size".to_string());
                            }),
                        );
                    },
                ));
                scope.menu_item(&MenuItemArgs::default().label("View").submenu_content(
                    move |submenu_scope| {
                        submenu_scope.menu_item(&MenuItemArgs::default().label("List").on_click(
                            move || {
                                selected_path.set("View / List".to_string());
                            },
                        ));
                        submenu_scope.menu_item(&MenuItemArgs::default().label("Grid").on_click(
                            move || {
                                selected_path.set("View / Grid".to_string());
                            },
                        ));
                    },
                ));
                scope.menu_item(&MenuItemArgs::default().label("Rename").on_click(move || {
                    selected_path.set("Rename".to_string());
                }));
            }),
    );
}

#[derive(Clone, Prop)]
struct PagerDemoSectionArgs {
    controller: State<PagerController>,
}

#[tessera]
fn pager_demo_section(args: &PagerDemoSectionArgs) {
    let controller = args.controller;
    let current_page = controller.with(|state| state.current_page());
    let color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .surface_container;

    column(
        &ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move |scope| {
                scope.child(move || {
                    text(
                        &TextArgs::default()
                            .text(format!("Current page: {} / 5", current_page + 1)),
                    );
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                });
                scope.child(move || {
                    surface(&SurfaceArgs::with_child(
                        SurfaceArgs::default()
                            .modifier(Modifier::new().fill_max_width().height(Dp(220.0)))
                            .shape(Shape::rounded_rectangle(Dp(20.0)))
                            .style(SurfaceStyle::Filled { color }),
                        move || {
                            horizontal_pager(
                                &PagerArgs::default()
                                    .page_count(5)
                                    .page_size(PagerPageSize::Fill)
                                    .page_spacing(Dp(12.0))
                                    .content_padding(Dp(16.0))
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .modifier(Modifier::new().fill_max_size())
                                    .controller(controller)
                                    .page_content(|page| {
                                        focus_navigation_pager_page(
                                            &FocusNavigationPagerPageArgs { page },
                                        );
                                    }),
                            );
                        },
                    ));
                });
            }),
    );
}

#[derive(Clone, Prop)]
struct FocusNavigationPagerPageArgs {
    page: usize,
}

#[tessera]
fn focus_navigation_pager_page(args: &FocusNavigationPagerPageArgs) {
    let page = args.page;
    let palette = [
        filled::HOME_SVG,
        filled::SEARCH_SVG,
        filled::PERSON_SVG,
        filled::LIST_SVG,
        filled::HOME_SVG,
    ];

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_size())
            .shape(Shape::rounded_rectangle(Dp(18.0)))
            .style(section_supporting_color().with_alpha(0.12).into())
            .content_alignment(tessera_components::alignment::Alignment::Center),
        move || {
            column(
                &ColumnArgs::default()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move |scope| {
                        scope.child(move || {
                            icon(
                                &IconArgs::default()
                                    .vector(palette[page % palette.len()])
                                    .size(Dp(28.0)),
                            );
                        });
                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                        });
                        scope.child(move || {
                            text(
                                &TextArgs::default()
                                    .text(format!("Keyboard Page {}", page + 1))
                                    .size(Dp(18.0)),
                            );
                        });
                        scope.child(|| {
                            text(
                                &TextArgs::default()
                                    .text("Use the pager itself as the focus target.")
                                    .size(Dp(14.0))
                                    .color(section_supporting_color()),
                            );
                        });
                    }),
            );
        },
    ));
}

#[derive(Clone, Prop)]
struct RevealDemoSectionArgs {
    selected_row: State<String>,
}

#[tessera]
fn reveal_demo_section(args: &RevealDemoSectionArgs) {
    let selected_row = args.selected_row;
    let reveal_controller = retain(LazyListController::new);
    let color = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme
        .surface_container;
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width().height(Dp(280.0)))
            .shape(Shape::rounded_rectangle(Dp(20.0)))
            .style(SurfaceStyle::Filled { color }),
        move || {
            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size())
                    .controller(reveal_controller)
                    .content_padding(Dp(12.0))
                    .item_spacing(Dp(8.0))
                    .estimated_item_size(Dp(72.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .content(move |scope| {
                        scope.item(move || {
                            text(
                                &TextArgs::default()
                                    .text(format!("Focused row: {}", selected_row.get())),
                            );
                        });
                        scope.items(16, move |index| {
                            let title = format!("Row {}", index + 1);
                            list_item(
                                &ListItemArgs::new(title.clone())
                                    .supporting_text(
                                        "Tab forward until this list reveals the next focused row.",
                                    )
                                    .on_click({
                                        let selected_row = selected_row;
                                        move || {
                                            selected_row.set(title.clone());
                                        }
                                    }),
                            );
                        });
                    }),
            );
        },
    ));
}

#[derive(Clone, Prop)]
struct DialogFocusContentArgs {
    dialog_controller: State<DialogController>,
}

#[tessera]
fn dialog_focus_content(args: &DialogFocusContentArgs) {
    let dialog_controller = args.dialog_controller;
    let editor_controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), None);
        controller.set_text("Press Tab inside this dialog to verify focus stays trapped.");
        controller
    });
    let checkbox_value = remember(|| true);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().width(Dp(420.0)).padding_all(Dp(24.0)))
            .shape(Shape::rounded_rectangle(Dp(28.0))),
        move || {
            column(
                &ColumnArgs::default()
                    .modifier(Modifier::new().fill_max_width())
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .children(move |scope| {
                        scope.child(|| {
                            text(&TextArgs::default().text("Dialog Focus Trap").size(Dp(20.0)));
                        });
                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                        });
                        scope.child(move || {
                            text_input(
                                &TextInputArgs::default()
                                    .modifier(Modifier::new().fill_max_width().height(Dp(120.0)))
                                    .controller(editor_controller)
                                    .on_change(|value| value),
                            );
                        });
                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                        });
                        scope.child(move || {
                            row(&RowArgs::default()
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move |row_scope| {
                                    row_scope.child(move || {
                                        checkbox(
                                            &CheckboxArgs::default()
                                                .checked(checkbox_value.get())
                                                .on_toggle(move |value| {
                                                    checkbox_value.set(value);
                                                }),
                                        );
                                    });
                                    row_scope.child(|| {
                                        spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))))
                                    });
                                    row_scope.child(|| {
                                        text(&TextArgs::from("Keep notifications enabled"));
                                    });
                                }));
                        });
                        scope.child(|| {
                            spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                        });
                        scope.child(move || {
                            row(&RowArgs::default()
                                .main_axis_alignment(MainAxisAlignment::End)
                                .children(move |row_scope| {
                                    row_scope.child(move || {
                                        demo_button(&DemoButtonArgs {
                                            label: "Cancel".into(),
                                            on_click: Callback::new(move || {
                                                dialog_controller
                                                    .with_mut(|controller| controller.close());
                                            }),
                                        });
                                    });
                                    row_scope.child(|| {
                                        spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))))
                                    });
                                    row_scope.child(move || {
                                        demo_button(&DemoButtonArgs {
                                            label: "Save".into(),
                                            on_click: Callback::new(move || {
                                                dialog_controller
                                                    .with_mut(|controller| controller.close());
                                            }),
                                        });
                                    });
                                }));
                        });
                    }),
            );
        },
    ));
}

#[derive(Clone, Prop)]
struct BottomSheetFocusContentArgs {
    bottom_sheet_controller: State<BottomSheetController>,
}

#[tessera]
fn bottom_sheet_focus_content(args: &BottomSheetFocusContentArgs) {
    let bottom_sheet_controller = args.bottom_sheet_controller;
    let editor_controller = remember(|| {
        let mut controller = TextInputController::new(Dp(16.0), None);
        controller.set_text("Bottom sheets should keep Tab traversal inside the sheet.");
        controller
    });

    column(
        &ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(20.0)))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move |scope| {
                scope.child(|| {
                    text(
                        &TextArgs::default()
                            .text("Bottom Sheet Focus Trap")
                            .size(Dp(20.0)),
                    );
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                });
                scope.child(move || {
                    text_input(
                        &TextInputArgs::default()
                            .modifier(Modifier::new().fill_max_width().height(Dp(100.0)))
                            .controller(editor_controller)
                            .on_change(|value| value),
                    );
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                });
                scope.child(move || {
                    demo_button(&DemoButtonArgs {
                        label: "Close bottom sheet".into(),
                        on_click: Callback::new(move || {
                            bottom_sheet_controller.with_mut(|controller| controller.close());
                        }),
                    });
                });
            }),
    );
}

#[derive(Clone, Prop)]
struct SideSheetFocusContentArgs {
    side_sheet_controller: State<SideSheetController>,
}

#[tessera]
fn side_sheet_focus_content(args: &SideSheetFocusContentArgs) {
    let side_sheet_controller = args.side_sheet_controller;
    let switch_controller = remember(|| SwitchController::new(true));
    let sheet_selection = remember(|| "Overview".to_string());

    column(
        &ColumnArgs::default()
            .modifier(Modifier::new().fill_max_width().padding_all(Dp(20.0)))
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .children(move |scope| {
                scope.child(|| {
                    text(
                        &TextArgs::default()
                            .text("Side Sheet Focus Trap")
                            .size(Dp(20.0)),
                    );
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                });
                scope.child(move || {
                    list_item(
                        &ListItemArgs::new("Overview")
                            .supporting_text(format!("Current: {}", sheet_selection.get()))
                            .on_click(move || {
                                sheet_selection.set("Overview".to_string());
                            }),
                    );
                });
                scope.child(move || {
                    list_item(&ListItemArgs::new("Activity").on_click(move || {
                        sheet_selection.set("Activity".to_string());
                    }));
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(12.0))));
                });
                scope.child(move || {
                    row(&RowArgs::default()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move |row_scope| {
                            row_scope.child(move || {
                                switch(
                                    &SwitchArgs::default()
                                        .controller(switch_controller)
                                        .on_toggle(|_| {}),
                                );
                            });
                            row_scope.child(|| {
                                spacer(&SpacerArgs::new(Modifier::new().width(Dp(12.0))))
                            });
                            row_scope.child(|| {
                                text(&TextArgs::from("Pin this sheet"));
                            });
                        }));
                });
                scope.child(|| {
                    spacer(&SpacerArgs::new(Modifier::new().height(Dp(16.0))));
                });
                scope.child(move || {
                    demo_button(&DemoButtonArgs {
                        label: "Close side sheet".into(),
                        on_click: Callback::new(move || {
                            side_sheet_controller.with_mut(|controller| controller.close());
                        }),
                    });
                });
            }),
    );
}
