use std::sync::Arc;

use closure::closure;
use tessera_ui::{
    Color, Dp, Modifier, State, remember,
    router::{Router, router_root},
    shard, tessera, use_context,
};
use tessera_ui_basic_components::{
    alignment::CrossAxisAlignment,
    app_bar::{AppBarArgs, TopAppBarArgs, top_app_bar as material_top_app_bar},
    bottom_sheet::{
        BottomSheetController, BottomSheetProviderArgs, BottomSheetStyle,
        bottom_sheet_provider_with_controller,
    },
    button::{ButtonArgs, button},
    column::{ColumnArgs, column},
    dialog::{
        BasicDialogArgs, DialogController, DialogProviderArgs, DialogStyle, basic_dialog,
        dialog_provider_with_controller,
    },
    icon::{IconArgs, icon},
    lazy_list::{LazyColumnArgs, lazy_column},
    material_icons::filled,
    modifier::{ModifierExt as _, Padding},
    navigation_bar::{NavigationBarItem, navigation_bar},
    scrollable::ScrollableArgs,
    shape_def::Shape,
    side_bar::{
        SideBarController, SideBarProviderArgs, SideBarStyle, side_bar_provider_with_controller,
    },
    spacer::spacer,
    surface::{SurfaceArgs, SurfaceStyle, surface},
    text::{TextArgs, text},
    theme::MaterialTheme,
};

use crate::example_components::{
    badge::BadgeShowcaseDestination,
    button::ButtonShowcaseDestination,
    button_group::ButtonGroupShowcaseDestination,
    card::CardShowcaseDestination,
    checkbox::CheckboxShowcaseDestination,
    chip::ChipShowcaseDestination,
    divider::DividerShowcaseDestination,
    fluid_glass::FluidGlassShowcaseDestination,
    glass_button::GlassButtonShowcaseDestination,
    glass_progress::GlassProgressShowcaseDestination,
    glass_slider::GlassSliderShowcaseDestination,
    glass_switch::GlassSwitchShowcaseDestination,
    image::{IconShowcaseDestination, ImageShowcaseDestination},
    layouts::LayoutsShowcaseDestination,
    lazy_grids::LazyGridsShowcaseDestination,
    lazy_lists::LazyListsShowcaseDestination,
    menus::MenusShowcaseDestination,
    progress::ProgressShowcaseDestination,
    progress_indicator::ProgressIndicatorShowcaseDestination,
    radio_button::RadioButtonShowcaseDestination,
    slider::SliderShowcaseDestination,
    spacer::SpacerShowcaseDestination,
    staggered_grids::StaggeredGridsShowcaseDestination,
    surface::SurfaceShowcaseDestination,
    switch::SwitchShowcaseDestination,
    tabs::TabsShowcaseDestination,
    text::TextShowcaseDestination,
    text_editor::TextEditorShowcaseDestination,
};

#[tessera]
pub fn app() {
    let side_bar_controller = remember(SideBarController::default);
    let bottom_sheet_controller = remember(BottomSheetController::default);
    let dialog_controller = remember(DialogController::default);

    side_bar_provider_with_controller(
        SideBarProviderArgs::new(move || {
            side_bar_controller.with_mut(|c| c.close());
        })
        .style(SideBarStyle::Glass),
        side_bar_controller,
        move || {
            bottom_sheet_provider_with_controller(
                BottomSheetProviderArgs::new(move || {
                    bottom_sheet_controller.with_mut(|c| c.close());
                })
                .style(BottomSheetStyle::Material),
                bottom_sheet_controller,
                move || {
                    dialog_provider_with_controller(
                        DialogProviderArgs::new(move || {
                            dialog_controller.with_mut(|c| c.close());
                        })
                        .style(DialogStyle::Material),
                        dialog_controller,
                        move || {
                            column(ColumnArgs::default(), |scope| {
                                scope.child(top_app_bar);
                                scope.child_weighted(
                                    move || {
                                        router_root(HomeDestination {
                                            bottom_sheet_controller,
                                            side_bar_controller,
                                            dialog_controller,
                                        });
                                    },
                                    1.0,
                                );
                                scope.child(move || {
                                    let home_icon_args = IconArgs::from(filled::home_icon());
                                    let about_icon_args = IconArgs::from(filled::info_icon());

                                    navigation_bar(move |scope| {
                                        scope.item(
                                            NavigationBarItem::new("Home")
                                                .icon(closure!(
                                                    clone home_icon_args,
                                                    || {
                                                        icon(home_icon_args.clone());
                                                    }
                                                ))
                                                .on_click(move || {
                                                    Router::with_mut(|router| {
                                                        router.reset_with(HomeDestination {
                                                            bottom_sheet_controller,
                                                            side_bar_controller,
                                                            dialog_controller,
                                                        });
                                                    });
                                                }),
                                        );

                                        scope.item(
                                            NavigationBarItem::new("About")
                                                .icon(closure!(
                                                    clone about_icon_args,
                                                    || {
                                                        icon(about_icon_args.clone());
                                                    }
                                                ))
                                                .on_click(|| {
                                                    Router::with_mut(|router| {
                                                        router.reset_with(AboutDestination {});
                                                    });
                                                }),
                                        );
                                    });
                                });
                            });
                        },
                        move |_alpha| {
                            basic_dialog(
                                BasicDialogArgs::new("This is a basic dialog component.")
                                    .headline("Basic Dialog")
                                    .icon(|| {
                                        icon(IconArgs::from(filled::info_icon()));
                                    })
                                    .confirm_button(move || {
                                        button(
                                            ButtonArgs::text(move || {
                                                dialog_controller.with_mut(|c| c.close());
                                            }),
                                            || text("Confirm"),
                                        );
                                    })
                                    .dismiss_button(move || {
                                        button(
                                            ButtonArgs::text(move || {
                                                dialog_controller.with_mut(|c| c.close());
                                            }),
                                            || text("Dismiss"),
                                        );
                                    }),
                            );
                        },
                    );
                },
                || {
                    column(
                        ColumnArgs {
                            modifier: Modifier::new().padding_all(Dp(16.0)),
                            ..Default::default()
                        },
                        |scope| {
                            scope.child(|| text("Hello from bottom sheet!"));
                            scope.child(|| spacer(Modifier::new().height(Dp(250.0))));
                        },
                    );
                },
            );
        },
        || {
            text(
                r#"Hi, I'm side bar!

Side bars are bars at side, side at bars, bars side at, at side bars..."#,
            );
        },
    );
}

#[derive(Clone)]
struct ComponentExampleDesc {
    title: String,
    desription: String,
    on_click: Arc<dyn Fn() + Send + Sync>,
}

impl ComponentExampleDesc {
    fn new<F: Fn() + 'static + Send + Sync>(title: &str, description: &str, on_click: F) -> Self {
        Self {
            title: title.to_string(),
            desription: description.to_string(),
            on_click: Arc::new(on_click),
        }
    }
}

#[tessera]
#[shard]
fn home(
    bottom_sheet_controller: State<BottomSheetController>,
    side_bar_controller: State<SideBarController>,
    dialog_controller: State<DialogController>,
) {
    let examples = Arc::new(vec![
        ComponentExampleDesc::new(
            "Text Editor",
            "A basic component for multiline text input.",
            || {
                Router::with_mut(|router| {
                    router.push(TextEditorShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Tabs",
            "A component for switching between different views.",
            || {
                Router::with_mut(|router| {
                    router.push(TabsShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Divider",
            "Material 3 horizontal and vertical dividers.",
            || {
                Router::with_mut(|router| {
                    router.push(DividerShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new("Card", "Material 3 cards for grouped content.", || {
            Router::with_mut(|router| {
                router.push(CardShowcaseDestination {});
            });
        }),
        ComponentExampleDesc::new(
            "Badge",
            "Material 3 badges for status indicators and counts.",
            || {
                Router::with_mut(|router| {
                    router.push(BadgeShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Chip",
            "Material 3 chips for actions, filters, and input tokens.",
            || {
                Router::with_mut(|router| {
                    router.push(ChipShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Switch",
            "A control that allows toggling between on and off states.",
            || {
                Router::with_mut(|router| {
                    router.push(SwitchShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Spacer",
            "A component to create empty space in a layout.",
            || {
                Router::with_mut(|router| {
                    router.push(SpacerShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Slider",
            "A control that allows selecting a value from a range.",
            || {
                Router::with_mut(|router| {
                    router.push(SliderShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new("Progress", "A standard component to show progress.", || {
            Router::with_mut(|router| {
                router.push(ProgressShowcaseDestination {});
            });
        }),
        ComponentExampleDesc::new(
            "Progress Indicators",
            "Material 3 linear and circular progress indicators.",
            || {
                Router::with_mut(|router| {
                    router.push(ProgressIndicatorShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new("Image", "A component to display images.", || {
            Router::with_mut(|router| {
                router.push(ImageShowcaseDestination {});
            });
        }),
        ComponentExampleDesc::new("Icon", "A component to display vector icons.", || {
            Router::with_mut(|router| {
                router.push(IconShowcaseDestination {});
            });
        }),
        ComponentExampleDesc::new(
            "Glass Switch",
            "A switch with a frosted glass effect.",
            || {
                Router::with_mut(|router| {
                    router.push(GlassSwitchShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Glass Slider",
            "A slider with a frosted glass effect.",
            || {
                Router::with_mut(|router| {
                    router.push(GlassSliderShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Glass Progress",
            "A progress bar with a frosted glass effect.",
            || {
                Router::with_mut(|router| {
                    router.push(GlassProgressShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Glass Button",
            "A button with a frosted glass effect.",
            || {
                Router::with_mut(|router| {
                    router.push(GlassButtonShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Fluid Glass",
            "A component that creates a frosted glass effect over a background.",
            || {
                Router::with_mut(|router| {
                    router.push(FluidGlassShowcaseDestination {});
                });
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
            "Checkbox",
            "A control that allows the user to select a binary 'on' or 'off' option.",
            || {
                Router::with_mut(|router| {
                    router.push(CheckboxShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Radio Button",
            "A single-choice control that selects exactly one option in a group.",
            || {
                Router::with_mut(|router| {
                    router.push(RadioButtonShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Button",
            "A clickable component with ripple effects for user interaction.",
            || {
                Router::with_mut(|router| {
                    router.push(ButtonShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Button Group",
            "Material 3 segmented buttons supporting single or multiple selection.",
            || {
                Router::with_mut(|router| {
                    router.push(ButtonGroupShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Menus",
            "Material 3 anchored menus with selection and pin toggle.",
            || {
                Router::with_mut(|router| {
                    router.push(MenusShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Layouts (Row, Column, Flow, Boxed)",
            "Components for arranging items in rows, columns, and wrapping layouts.",
            || {
                Router::with_mut(|router| {
                    router.push(LayoutsShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Lazy Lists",
            "Virtualized row/column components that only instantiate visible items.",
            || {
                Router::with_mut(|router| {
                    router.push(LazyListsShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Lazy Grids",
            "Virtualized grids for tiled content and galleries.",
            || {
                Router::with_mut(|router| {
                    router.push(LazyGridsShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Staggered Grids",
            "Masonry-style grids for variable-size tiles.",
            || {
                Router::with_mut(|router| {
                    router.push(StaggeredGridsShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Text",
            "Basic text component, support colorful emoji",
            || {
                Router::with_mut(|router| {
                    router.push(TextShowcaseDestination {});
                });
            },
        ),
        ComponentExampleDesc::new(
            "Surface",
            "surface acts as a visual and interactive container, supporting background color, shape, shadow, border, padding, and optional ripple effects for user interaction.",
            || {
                Router::with_mut(|router| {
                    router.push(SurfaceShowcaseDestination {});
                });
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
            "Side Bar",
            "side bar displays content sliding in from the left side of the screen.",
            move || {
                side_bar_controller.with_mut(|c| c.open());
            },
        ),
    ]);

    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        move || {
            lazy_column(
                LazyColumnArgs::default()
                    .scrollable(ScrollableArgs::default().modifier(Modifier::new().fill_max_size()))
                    .item_spacing(Dp(16.0))
                    .content_padding(Dp(8.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .estimated_item_size(Dp(140.0))
                    .content_padding(Dp(16.0)),
                move |scope| {
                    scope.items_from_iter(examples.iter().cloned(), move |_, example| {
                        let on_click = example.on_click.clone();
                        let title = example.title.clone();
                        let description = example.desription.clone();
                        component_card(&title, &description, on_click);
                    });
                },
            );
        },
    );
}

#[tessera]
fn component_card(title: &str, description: &str, on_click: Arc<dyn Fn() + Send + Sync>) {
    let title = title.to_string();
    let description = description.to_string();
    surface(
        SurfaceArgs::default()
            .modifier(Modifier::new().fill_max_width())
            .on_click_shared(on_click)
            .style(SurfaceStyle::Filled {
                color: use_context::<MaterialTheme>()
                    .get()
                    .color_scheme
                    .primary_container,
            })
            .shape(Shape::rounded_rectangle(Dp(25.0)))
            .elevation(Dp(6.0)),
        || {
            column(
                ColumnArgs {
                    modifier: Modifier::new().fill_max_width().padding_all(Dp(25.0)),
                    ..Default::default()
                },
                |scope| {
                    scope.child(move || {
                        text(TextArgs::default().text(title).size(Dp(20.0)));
                    });
                    scope.child(move || {
                        text(
                            TextArgs::default().text(description).size(Dp(14.0)).color(
                                use_context::<MaterialTheme>()
                                    .get()
                                    .color_scheme
                                    .on_surface_variant,
                            ),
                        );
                    });
                },
            );
        },
    );
}

#[tessera]
fn top_app_bar() {
    let app_bar_args = AppBarArgs::default().elevation(Dp(4.0));
    let args = TopAppBarArgs::new("Tessera UI")
        .app_bar(app_bar_args)
        .navigation_icon(|| {
            let mut button_args = ButtonArgs::default()
                .padding(Dp(5.0))
                .color(Color::TRANSPARENT)
                .modifier(Modifier::new().size(Dp(40.0), Dp(40.0)));

            if Router::with(|router| router.len()) > 1 {
                button_args = button_args.on_click(|| {
                    Router::with_mut(|router| {
                        router.pop();
                    });
                });
            }

            button(button_args, || {
                icon(IconArgs::from(filled::arrow_back_icon()).size(Dp(20.0)));
            });
        });

    material_top_app_bar(args);
}

#[tessera]
#[shard]
fn about() {
    surface(
        SurfaceArgs::default().modifier(Modifier::new().fill_max_size()),
        || {
            text(
                TextArgs::default()
                    .modifier(Modifier::new().padding(Padding::all(Dp(16.0))))
                    .text(
                        r#"This is an example app of Tessera UI Framework.,
Made with ❤️ by tessera-ui devs.,

Copyright 2025 Tessera UI Framework Developers,
"#
                        .to_string(),
                    )
                    .size(Dp(20.0))
                    .color(use_context::<MaterialTheme>().get().color_scheme.on_surface),
            );
        },
    );
}
