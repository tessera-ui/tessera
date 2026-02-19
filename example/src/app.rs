use std::sync::Arc;

use closure::closure;
use tessera_components::{
    alignment::CrossAxisAlignment,
    app_bar::{AppBarArgs, AppBarDefaults, app_bar},
    bottom_sheet::{
        BottomSheetController, BottomSheetProviderArgs, BottomSheetStyle, bottom_sheet_provider,
    },
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    dialog::{
        BasicDialogArgs, DialogController, DialogProviderArgs, DialogStyle, basic_dialog,
        dialog_provider,
    },
    icon::{IconArgs, icon},
    icon_button::{IconButtonArgs, icon_button},
    lazy_list::{LazyColumnArgs, LazyListController, lazy_column},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding},
    navigation_bar::{NavigationBarItem, navigation_bar},
    navigation_rail::{NavigationRailItem, navigation_rail},
    row::{RowArgs, row},
    scaffold::{ScaffoldArgs, scaffold},
    search::{SearchBarArgs, SearchBarController, docked_search_bar},
    shape_def::Shape,
    side_sheet::{SideSheetController, SideSheetProviderArgs, modal_side_sheet_provider},
    spacer::{SpacerArgs, spacer},
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::{MaterialTheme, MaterialThemeProviderArgs, material_theme, provide_text_style},
};
use tessera_ui::{
    Callback, Color, Dp, Modifier, RenderSlot, State, WindowAction, remember, retain,
    router::{Router, router_scope, router_view},
    shard, tessera, use_context,
};

use crate::example_components::{
    badge::BadgeShowcaseDestination,
    button::ButtonShowcaseDestination,
    button_group::ButtonGroupShowcaseDestination,
    card::CardShowcaseDestination,
    checkbox::CheckboxShowcaseDestination,
    chip::ChipShowcaseDestination,
    date_time_picker::DateTimePickerShowcaseDestination,
    divider::DividerShowcaseDestination,
    floating_action_button::FloatingActionButtonShowcaseDestination,
    fluid_glass::FluidGlassShowcaseDestination,
    glass_button::GlassButtonShowcaseDestination,
    glass_progress::GlassProgressShowcaseDestination,
    glass_slider::GlassSliderShowcaseDestination,
    glass_switch::GlassSwitchShowcaseDestination,
    image::{IconShowcaseDestination, ImageShowcaseDestination},
    layouts::LayoutsShowcaseDestination,
    lazy_grids::LazyGridsShowcaseDestination,
    lazy_lists::LazyListsShowcaseDestination,
    list_item::ListItemShowcaseDestination,
    menus::MenusShowcaseDestination,
    pager::PagerShowcaseDestination,
    progress::ProgressShowcaseDestination,
    progress_indicator::ProgressIndicatorShowcaseDestination,
    pull_refresh::PullRefreshShowcaseDestination,
    radio_button::RadioButtonShowcaseDestination,
    segmented_buttons::SegmentedButtonsShowcaseDestination,
    slider::SliderShowcaseDestination,
    snackbar::SnackbarShowcaseDestination,
    spacer::SpacerShowcaseDestination,
    split_buttons::SplitButtonsShowcaseDestination,
    staggered_grids::StaggeredGridsShowcaseDestination,
    surface::SurfaceShowcaseDestination,
    switch::SwitchShowcaseDestination,
    tabs::TabsShowcaseDestination,
    text::TextShowcaseDestination,
    text_input::TextInputShowcaseDestination,
};

const NAVIGATION_RAIL_BREAKPOINT: Dp = Dp(600.0);

#[derive(Clone, PartialEq)]
struct MeasureParentWidthArgs {
    width_state: State<Dp>,
    child: RenderSlot,
}

fn measure_parent_width(width_state: State<Dp>, child: impl Fn() + Send + Sync + 'static) {
    let args = MeasureParentWidthArgs {
        width_state,
        child: RenderSlot::new(child),
    };
    measure_parent_width_node(&args);
}

#[tessera]
fn measure_parent_width_node(args: &MeasureParentWidthArgs) {
    input_handler({
        let width_state = args.width_state;
        move |input| {
            let width = Dp::from(input.computed_data.width);
            if width_state.get() != width {
                width_state.set(width);
            }
        }
    });

    args.child.render();
}
pub fn app() {
    app_node();
}

#[tessera]
fn app_node() {
    material_theme(&MaterialThemeProviderArgs::new(
        MaterialTheme::default,
        || {
            app_inner();
        },
    ));
}

fn app_inner() {
    app_inner_node();
}

#[tessera]
fn app_inner_node() {
    let side_sheet_controller = remember(SideSheetController::default);
    let bottom_sheet_controller = remember(BottomSheetController::default);
    let dialog_controller = remember(DialogController::default);
    let navigation_width = remember(|| Dp::ZERO);

    let side_sheet_args = SideSheetProviderArgs::new(move || {
        side_sheet_controller.with_mut(|c| c.close());
    })
    .controller(side_sheet_controller)
    .main_content(move || {
            bottom_sheet_provider(
                &BottomSheetProviderArgs::new(move || {
                        bottom_sheet_controller.with_mut(|c| c.close());
                    })
                    .style(BottomSheetStyle::Material)
                    .controller(bottom_sheet_controller)
                    .main_content(move || {
                        dialog_provider(
                            &DialogProviderArgs::new(move || {
                                dialog_controller.with_mut(|c| c.close());
                            })
                            .style(DialogStyle::Material)
                            .controller(dialog_controller)
                            .main_content(move || {
                                measure_parent_width(navigation_width, move || {
                                    let use_navigation_rail =
                                        navigation_width.get().0 >= NAVIGATION_RAIL_BREAKPOINT.0;

                                    router_scope(
                                        HomeDestination {
                                            bottom_sheet_controller,
                                            side_sheet_controller,
                                            dialog_controller,
                                        },
                                        move || {
                                            if use_navigation_rail {
                                                row(
                                                    RowArgs::default()
                                                        .modifier(Modifier::new().fill_max_size())
                                                        .cross_axis_alignment(
                                                            CrossAxisAlignment::Stretch,
                                                        ),
                                                    move |row_scope| {
                                                        row_scope.child(move || {
                                                            let home_icon_args =
                                                                IconArgs::from(filled::home_icon());
                                                            let about_icon_args =
                                                                IconArgs::from(filled::info_icon());

                                                            let home_action =
                                                                Callback::new(move || {
                                                                    Router::reset(
                                                                        HomeDestination {
                                                                            bottom_sheet_controller,
                                                                            side_sheet_controller,
                                                                            dialog_controller,
                                                                        },
                                                                    );
                                                                });
                                                            let about_action =
                                                                Callback::new(|| {
                                                                    Router::reset(
                                                                            AboutDestination {},
                                                                        );
                                                                });

                                                            navigation_rail(move |scope| {
                                                                let navigation_rail_controller =
                                                                    scope.controller();
                                                                scope.header(move || {
                                                                    let is_expanded =
                                                                        navigation_rail_controller
                                                                            .with(|c| c.is_expanded());
                                                                    let button_args = if is_expanded
                                                                    {
                                                                        IconButtonArgs::new(
                                                                            filled::menu_open_icon(),
                                                                        )
                                                                    } else {
                                                                        IconButtonArgs::new(
                                                                            filled::menu_icon(),
                                                                        )
                                                                    };
                                                                    icon_button(
                                                                        &button_args.on_click(
                                                                            move || {
                                                                                navigation_rail_controller
                                                                                    .with_mut(|c| c.toggle());
                                                                            },
                                                                        ),
                                                                    );
                                                                });

                                                                scope.item(
                                                            NavigationRailItem::new("Home")
                                                                .icon(closure!(
                                                                    clone home_icon_args,
                                                                    || {
                                                                        icon(&
                                                                            home_icon_args.clone(),
                                                                        );
                                                                    }
                                                                ))
                                                                .on_click_shared(
                                                                    home_action.clone(),
                                                                ),
                                                        );

                                                                scope.item(
                                                            NavigationRailItem::new("About")
                                                                .icon(closure!(
                                                                    clone about_icon_args,
                                                                    || {
                                                                        icon(&
                                                                            about_icon_args.clone(),
                                                                        );
                                                                    }
                                                                ))
                                                                .on_click_shared(
                                                                    about_action.clone(),
                                                                ),
                                                        );
                                                            });
                                                        });

                                                        row_scope.child_weighted(
                                                        move || {
                                                            let component_args =
                                                                ScaffoldArgs::with_content(
                                                                    ScaffoldArgs::default()
                                                                        .top_bar_height(
                                                                            AppBarDefaults::TOP_APP_BAR_HEIGHT,
                                                                        )
                                                                        .top_bar(top_app_bar),
                                                                    move || {
                                                                        router_view();
                                                                    },
                                                                );
                                                            scaffold(&component_args);
                                                        },
                                                        1.0,
                                                    );
                                                    },
                                                );
                                            } else {
                                                column(ColumnArgs::default(), |scope| {
                                                    scope.child_weighted(
                                                    move || {
                                                        let component_args =
                                                            ScaffoldArgs::with_content(
                                                                ScaffoldArgs::default()
                                                                    .top_bar_height(
                                                                        AppBarDefaults::TOP_APP_BAR_HEIGHT,
                                                                    )
                                                                    .top_bar(top_app_bar),
                                                                move || {
                                                                    router_view();
                                                                },
                                                            );
                                                        scaffold(&component_args);
                                                    },
                                                    1.0,
                                                );
                                                    scope.child(move || {
                                                        let home_icon_args =
                                                            IconArgs::from(filled::home_icon());
                                                        let about_icon_args =
                                                            IconArgs::from(filled::info_icon());

                                                        let home_action =
                                                            Callback::new(move || {
                                                                Router::reset(
                                                                        HomeDestination {
                                                                            bottom_sheet_controller,
                                                                            side_sheet_controller,
                                                                            dialog_controller,
                                                                        },
                                                                    );
                                                            });
                                                        let about_action = Callback::new(|| {
                                                            Router::reset(
                                                                    AboutDestination {},
                                                                );
                                                        });

                                                        navigation_bar(move |scope| {
                                                            scope.item(
                                                    NavigationBarItem::new("Home")
                                                        .icon(closure!(
                                                            clone home_icon_args,
                                                            || {
                                                                icon(&home_icon_args.clone());
                                                            }
                                                        ))
                                                        .on_click_shared(home_action.clone()),
                                                );

                                                            scope.item(
                                                    NavigationBarItem::new("About")
                                                        .icon(closure!(
                                                            clone about_icon_args,
                                                            || {
                                                                icon(&about_icon_args.clone());
                                                            }
                                                        ))
                                                        .on_click_shared(about_action.clone()),
                                                );
                                                        });
                                                    });
                                                });
                                            }
                                        },
                                    );
                                });
                            })
                            .dialog_content(move || {
                                basic_dialog(
                                &BasicDialogArgs::new("This is a basic dialog component.")
                                    .headline("Basic Dialog")
                                    .icon(|| {
                                        icon(&IconArgs::from(filled::info_icon()));
                                    })
                                    .confirm_button(move || {
                                        button(&ButtonArgs::with_child(
                                            ButtonArgs::text(move || {
                                                dialog_controller.with_mut(|c| c.close());
                                            }),
                                            || text(&TextArgs::from("Confirm")),
                                        ));
                                    })
                                    .dismiss_button(move || {
                                        button(&ButtonArgs::with_child(
                                            ButtonArgs::text(move || {
                                                dialog_controller.with_mut(|c| c.close());
                                            }),
                                            || text(&TextArgs::from("Dismiss")),
                                        ));
                                    }),
                            );
                            }),
                        );
                    })
                    .bottom_sheet_content(|| {
                        column(
                            ColumnArgs {
                                modifier: Modifier::new().padding_all(Dp(16.0)),
                                ..Default::default()
                            },
                            |scope| {
                                scope.child(|| {
                                    text(&TextArgs::from(
                                        "Hello from bottom sheet!",
                                    ))
                                });
                                scope.child(|| {
                                    spacer(&SpacerArgs::new(
                                        Modifier::new().height(Dp(250.0)),
                                    ))
                                });
                            },
                        );
                    }),
            );
        })
        .side_sheet_content(|| {
            text(&TextArgs::from(
                r#"Hi, I'm a side sheet!

Side sheets provide secondary content or tools that slide in from the screen edge."#,
            ));
        });
    modal_side_sheet_provider(&side_sheet_args);
}

#[derive(Clone)]
struct ComponentExampleDesc {
    title: String,
    desription: String,
    on_click: Callback,
}

impl ComponentExampleDesc {
    fn new(title: &str, description: &str, on_click: impl Into<Callback>) -> Self {
        Self {
            title: title.to_string(),
            desription: description.to_string(),
            on_click: on_click.into(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct HomeArgs {
    bottom_sheet_controller: State<BottomSheetController>,
    side_sheet_controller: State<SideSheetController>,
    dialog_controller: State<DialogController>,
}

#[shard]
fn home(
    bottom_sheet_controller: State<BottomSheetController>,
    side_sheet_controller: State<SideSheetController>,
    dialog_controller: State<DialogController>,
) {
    let args = HomeArgs {
        bottom_sheet_controller,
        side_sheet_controller,
        dialog_controller,
    };
    home_node(&args);
}

#[tessera]
fn home_node(args: &HomeArgs) {
    let bottom_sheet_controller = args.bottom_sheet_controller;
    let side_sheet_controller = args.side_sheet_controller;
    let dialog_controller = args.dialog_controller;

    let search_query = remember(String::new);
    let search_controller = remember(SearchBarController::default);
    let examples = Arc::new(vec![
        ComponentExampleDesc::new(
            "Text Input",
            "A basic component for multiline text input.",
            || {
                Router::push(TextInputShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Tabs",
            "A component for switching between different views.",
            || {
                Router::push(TabsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new("Pager", "Swipeable pages with snapping behavior.", || {
            Router::push(PagerShowcaseDestination {});
        }),
        ComponentExampleDesc::new(
            "Divider",
            "Material 3 horizontal and vertical dividers.",
            || {
                Router::push(DividerShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new("Card", "Material 3 cards for grouped content.", || {
            Router::push(CardShowcaseDestination {});
        }),
        ComponentExampleDesc::new(
            "Badge",
            "Material 3 badges for status indicators and counts.",
            || {
                Router::push(BadgeShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Chip",
            "Material 3 chips for actions, filters, and input tokens.",
            || {
                Router::push(ChipShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "List Item",
            "Material 3 list rows with leading, supporting, and trailing content.",
            || {
                Router::push(ListItemShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Snackbar",
            "Transient messages with optional actions and dismiss controls.",
            || {
                Router::push(SnackbarShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Switch",
            "A control that allows toggling between on and off states.",
            || {
                Router::push(SwitchShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Spacer",
            "A component to create empty space in a layout.",
            || {
                Router::push(SpacerShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Slider",
            "A control that allows selecting a value from a range.",
            || {
                Router::push(SliderShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new("Progress", "A standard component to show progress.", || {
            Router::push(ProgressShowcaseDestination {});
        }),
        ComponentExampleDesc::new(
            "Progress Indicators",
            "Material 3 linear and circular progress indicators.",
            || {
                Router::push(ProgressIndicatorShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Pull-to-refresh",
            "Pull down to trigger a refresh and show a progress indicator.",
            || {
                Router::push(PullRefreshShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new("Image", "A component to display images.", || {
            Router::push(ImageShowcaseDestination {});
        }),
        ComponentExampleDesc::new("Icon", "A component to display vector icons.", || {
            Router::push(IconShowcaseDestination {});
        }),
        ComponentExampleDesc::new(
            "Glass Switch",
            "A switch with a frosted glass effect.",
            || {
                Router::push(GlassSwitchShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Glass Slider",
            "A slider with a frosted glass effect.",
            || {
                Router::push(GlassSliderShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Glass Progress",
            "A progress bar with a frosted glass effect.",
            || {
                Router::push(GlassProgressShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Glass Button",
            "A button with a frosted glass effect.",
            || {
                Router::push(GlassButtonShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Fluid Glass",
            "A component that creates a frosted glass effect over a background.",
            || {
                Router::push(FluidGlassShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Dialog",
            "A modal window that appears on top of the main content.",
            move || {
                dialog_controller.with_mut(|c| c.open());
            },
        ),
        ComponentExampleDesc::new(
            "Date & Time Pickers",
            "Calendar and clock pickers with inline and dialog variants.",
            || {
                Router::push(DateTimePickerShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Checkbox",
            "A control that allows the user to select a binary 'on' or 'off' option.",
            || {
                Router::push(CheckboxShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Radio Button",
            "A single-choice control that selects exactly one option in a group.",
            || {
                Router::push(RadioButtonShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Button",
            "A clickable component with ripple effects for user interaction.",
            || {
                Router::push(ButtonShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Button Groups",
            "Grouped buttons for related actions with shared spacing and styling.",
            || {
                Router::push(ButtonGroupShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Segmented Buttons",
            "Connected buttons for single or multi-select options.",
            || {
                Router::push(SegmentedButtonsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Split Buttons",
            "Primary and secondary actions combined into a split button.",
            || {
                Router::push(SplitButtonsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Floating Action Button",
            "Material 3 floating action button for primary actions.",
            || {
                Router::push(FloatingActionButtonShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Menus",
            "Material 3 anchored menus with selection and pin toggle.",
            || {
                Router::push(MenusShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Layouts (Row, Column, Flow, Boxed)",
            "Components for arranging items in rows, columns, and wrapping layouts.",
            || {
                Router::push(LayoutsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Lazy Lists",
            "Virtualized row/column components that only instantiate visible items.",
            || {
                Router::push(LazyListsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Lazy Grids",
            "Virtualized grids for tiled content and galleries.",
            || {
                Router::push(LazyGridsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Staggered Grids",
            "Masonry-style grids for variable-size tiles.",
            || {
                Router::push(StaggeredGridsShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Text",
            "Basic text component, support colorful emoji",
            || {
                Router::push(TextShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Surface",
            "surface acts as a visual and interactive container, supporting background color, shape, shadow, border, padding, and optional ripple effects for user interaction.",
            || {
                Router::push(SurfaceShowcaseDestination {});
            },
        ),
        ComponentExampleDesc::new(
            "Bottom Sheet",
            "bottom sheet displays content sliding up from the bottom of the screen.",
            move || {
                bottom_sheet_controller.with_mut(|c| c.open());
            },
        ),
        ComponentExampleDesc::new(
            "Side Sheet",
            "Side sheets display supporting content sliding in from the screen edge.",
            move || {
                side_sheet_controller.with_mut(|c| c.open());
            },
        ),
    ]);

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            let list_controller = retain(LazyListController::new);
            let query = search_query.get();
            let query = query.trim().to_lowercase();
            let filtered: Vec<ComponentExampleDesc> = if query.is_empty() {
                examples.iter().cloned().collect()
            } else {
                examples
                    .iter()
                    .filter(|&example| {
                        let title = example.title.to_lowercase();
                        let description = example.desription.to_lowercase();
                        title.contains(&query) || description.contains(&query)
                    })
                    .cloned()
                    .collect()
            };

            lazy_column(
                &LazyColumnArgs::default()
                    .modifier(Modifier::new().fill_max_size())
                    .item_spacing(Dp(16.0))
                    .content_padding(Dp(8.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .estimated_item_size(Dp(140.0))
                    .content_padding(Dp(16.0))
                    .controller(list_controller)
                    .content(move |scope| {
                        let filtered = filtered.clone();
                        scope.sticky_header(move || {
                            let search_query = search_query;
                            let controller = search_controller;
                            let args = SearchBarArgs::default()
                                .modifier(Modifier::new().fill_max_width())
                                .placeholder("Search components")
                                .controller(controller)
                                .leading_icon(|| {
                                    icon(&IconArgs::from(filled::search_icon()));
                                })
                                .on_query_change(move |text| {
                                    search_query.set(text.clone());
                                    text
                                })
                                .on_active_change(move |is_active| {
                                    if is_active {
                                        controller.with_mut(|c| c.close());
                                    }
                                });

                            docked_search_bar(&args.content(|| {}));
                        });

                        scope.items_from_iter(filtered, move |_, example| {
                            let on_click = example.on_click.clone();
                            let title = example.title.clone();
                            let description = example.desription.clone();
                            component_card(&title, &description, on_click);
                        });
                    }),
            );
        },
    ));
}
#[derive(Clone, PartialEq)]
struct ComponentCardArgs {
    title: String,
    description: String,
    on_click: Callback,
}

fn component_card(title: &str, description: &str, on_click: impl Into<Callback>) {
    let args = ComponentCardArgs {
        title: title.to_string(),
        description: description.to_string(),
        on_click: on_click.into(),
    };
    component_card_node(&args);
}

#[tessera]
fn component_card_node(args: &ComponentCardArgs) {
    let on_click = args.on_click.clone();
    let title = args.title.clone();
    let description = args.description.clone();

    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .on_click_shared(on_click)
            .style(SurfaceStyle::Filled {
                color: use_context::<MaterialTheme>()
                    .expect("MaterialTheme must be provided")
                    .get()
                    .color_scheme
                    .primary_container,
            })
            .shape(Shape::rounded_rectangle(Dp(25.0)))
            .elevation(Dp(6.0)),
        move || {
            let title = title.clone();
            let description = description.clone();
            column(
                ColumnArgs {
                    modifier: Modifier::new().fill_max_width().padding_all(Dp(25.0)),
                    ..Default::default()
                },
                |scope| {
                    scope.child(move || {
                        text(&TextArgs::default().text(title.clone()).size(Dp(20.0)));
                    });
                    scope.child(move || {
                        text(
                            &TextArgs::default()
                                .text(description.clone())
                                .size(Dp(14.0))
                                .color(
                                    use_context::<MaterialTheme>()
                                        .expect("MaterialTheme must be provided")
                                        .get()
                                        .color_scheme
                                        .on_surface_variant,
                                ),
                        );
                    });
                },
            );
        },
    ));
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Clone, PartialEq)]
struct WindowControlButtonArgs {
    icon_args: IconArgs,
    action: tessera_ui::WindowAction,
    tint: Color,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn window_control_button(icon_args: IconArgs, action: tessera_ui::WindowAction, tint: Color) {
    let args = WindowControlButtonArgs {
        icon_args,
        action,
        tint,
    };
    window_control_button_node(&args);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tessera]
fn window_control_button_node(args: &WindowControlButtonArgs) {
    let icon_args = args.icon_args.clone().size(Dp(18.0)).tint(args.tint);
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default()
            .modifier(
                Modifier::new()
                    .size(Dp(40.0), Dp(32.0))
                    .window_action(args.action),
            )
            .style(Color::TRANSPARENT.into())
            .content_color(args.tint)
            .content_alignment(tessera_components::alignment::Alignment::Center),
        move || {
            icon(&icon_args);
        },
    ));
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Clone, PartialEq)]
struct WindowControlsArgs;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn window_controls() {
    window_controls_node(&WindowControlsArgs);
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tessera]
fn window_controls_node(args: &WindowControlsArgs) {
    let _ = args;

    let scheme = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .color_scheme;
    let neutral = scheme.on_surface_variant;
    let destructive = scheme.error;

    row(
        RowArgs::default().cross_axis_alignment(CrossAxisAlignment::Center),
        move |row_scope| {
            row_scope.child(move || {
                window_control_button(
                    IconArgs::from(filled::minimize_icon()),
                    WindowAction::Minimize,
                    neutral,
                );
            });
            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(4.0)))));
            row_scope.child(move || {
                window_control_button(
                    IconArgs::from(filled::fullscreen_icon()),
                    WindowAction::ToggleMaximize,
                    neutral,
                );
            });
            row_scope.child(|| spacer(&SpacerArgs::new(Modifier::new().width(Dp(4.0)))));
            row_scope.child(move || {
                window_control_button(
                    IconArgs::from(filled::close_icon()),
                    WindowAction::Close,
                    destructive,
                );
            });
        },
    );
}

#[derive(Clone, PartialEq)]
struct TopAppBarArgs;

fn top_app_bar() {
    top_app_bar_node(&TopAppBarArgs);
}

#[tessera]
fn top_app_bar_node(args: &TopAppBarArgs) {
    let _ = args;
    let app_bar_args = AppBarArgs::default().elevation(Dp(4.0));
    let title_style = use_context::<MaterialTheme>()
        .expect("MaterialTheme must be provided")
        .get()
        .typography
        .title_large;
    let can_go_back = Router::len() > 1;

    app_bar(&app_bar_args, move |scope| {
        scope.child(move || {
            icon_button(
                &IconButtonArgs::new(filled::arrow_back_icon())
                    .enabled(can_go_back)
                    .color(Color::TRANSPARENT)
                    .on_click(|| {
                        Router::pop();
                    }),
            )
        });

        let title_text = "Tessera UI".to_string();
        scope.child_weighted(
            move || {
                let title_text = title_text.clone();
                row(
                    RowArgs::default()
                        .modifier(Modifier::new().fill_max_size().window_drag_region())
                        .cross_axis_alignment(CrossAxisAlignment::Center),
                    move |row_scope| {
                        let title_text = title_text.clone();
                        row_scope.child(move || {
                            let title_text = title_text.clone();
                            provide_text_style(title_style, move || {
                                text(&TextArgs::default().text(title_text.clone()));
                            });
                        });
                    },
                );
            },
            1.0,
        );

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        scope.child(|| {
            window_controls();
        });
    });
}

#[tessera]
#[shard]
fn about() {
    about_node();
}

#[tessera]
fn about_node() {
    surface(&SurfaceArgs::with_child(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        || {
            text(
                &TextArgs::default()
                    .modifier(Modifier::new().padding(Padding::all(Dp(16.0))))
                    .text(
                        r#"This is an example app of Tessera UI Framework.
Made with ❤️ by tessera-ui devs.

Copyright 2025 Tessera UI Framework Developers
"#
                        .to_string(),
                    )
                    .size(Dp(20.0))
                    .color(
                        use_context::<MaterialTheme>()
                            .expect("MaterialTheme must be provided")
                            .get()
                            .color_scheme
                            .on_surface,
                    ),
            );
        },
    ));
}
