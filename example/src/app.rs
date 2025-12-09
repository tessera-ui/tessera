use std::sync::Arc;

use closure::closure;
use tessera_ui::{
    Color, DimensionValue, Dp,
    router::{Router, router_root},
    shard, tessera, use_context,
};
use tessera_ui_basic_components::{
    ShadowProps,
    alignment::{Alignment, CrossAxisAlignment},
    bottom_sheet::{
        BottomSheetController, BottomSheetProviderArgsBuilder, BottomSheetStyle,
        bottom_sheet_provider_with_controller,
    },
    boxed::{BoxedArgsBuilder, boxed},
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgs, column},
    dialog::{
        BasicDialogArgsBuilder, DialogController, DialogProviderArgsBuilder, DialogStyle,
        basic_dialog, dialog_provider_with_controller,
    },
    icon::{IconArgsBuilder, icon},
    lazy_list::{LazyColumnArgsBuilder, lazy_column},
    material_icons::filled,
    navigation_bar::{NavigationBarItemBuilder, navigation_bar},
    row::{RowArgsBuilder, row},
    scrollable::ScrollableArgsBuilder,
    shape_def::Shape,
    side_bar::{
        SideBarController, SideBarProviderArgsBuilder, SideBarStyle,
        side_bar_provider_with_controller,
    },
    surface::{SurfaceArgs, SurfaceArgsBuilder, SurfaceStyle, surface},
    text::{TextArgsBuilder, text},
    theme::MaterialColorScheme,
};

use crate::example_components::{
    button::ButtonShowcaseDestination,
    button_group::ButtonGroupShowcaseDestination,
    checkbox::CheckboxShowcaseDestination,
    fluid_glass::FluidGlassShowcaseDestination,
    glass_button::GlassButtonShowcaseDestination,
    glass_progress::GlassProgressShowcaseDestination,
    glass_slider::GlassSliderShowcaseDestination,
    glass_switch::GlassSwitchShowcaseDestination,
    image::{IconShowcaseDestination, ImageShowcaseDestination},
    layouts::LayoutsShowcaseDestination,
    lazy_lists::LazyListsShowcaseDestination,
    menus::MenusShowcaseDestination,
    progress::ProgressShowcaseDestination,
    radio_button::RadioButtonShowcaseDestination,
    slider::SliderShowcaseDestination,
    spacer::SpacerShowcaseDestination,
    surface::SurfaceShowcaseDestination,
    switch::SwitchShowcaseDestination,
    tabs::TabsShowcaseDestination,
    text::TextShowcaseDestination,
    text_editor::TextEditorShowcaseDestination,
};

#[derive(Default)]
struct AppState {
    bottom_sheet_controller: Arc<BottomSheetController>,
    side_bar_controller: Arc<SideBarController>,
    dialog_controller: Arc<DialogController>,
}

#[tessera]
#[shard]
pub fn app(#[state] app_state: AppState) {
    side_bar_provider_with_controller(
        SideBarProviderArgsBuilder::default()
            .on_close_request(Arc::new(closure!(clone app_state.side_bar_controller, || {
                side_bar_controller.close();
            })))
            .style(SideBarStyle::Glass)
            .build()
            .unwrap(),
        app_state.side_bar_controller.clone(),
        move || {
            bottom_sheet_provider_with_controller(
                BottomSheetProviderArgsBuilder::default()
                    .on_close_request(Arc::new(
                        closure!(clone app_state.bottom_sheet_controller, || {
                            bottom_sheet_controller.close();
                        }),
                    ))
                    .style(BottomSheetStyle::Glass)
                    .build()
                    .unwrap(),
                app_state.bottom_sheet_controller.clone(),
                move || {
                    let dialog_controller = app_state.dialog_controller.clone();
                    dialog_provider_with_controller(
                        DialogProviderArgsBuilder::default()
                            .on_close_request(Arc::new(closure!(clone dialog_controller, || {
                                dialog_controller.close();
                            })))
                            .style(DialogStyle::Glass)
                            .build()
                            .unwrap(),
                        dialog_controller.clone(),
                        move || {
                            column(ColumnArgs::default(), |scope| {
                                scope.child(|| {
                                    top_app_bar();
                                });
                                let bottom_sheet_controller =
                                    app_state.bottom_sheet_controller.clone();
                                let side_bar_controller = app_state.side_bar_controller.clone();
                                let dialog_controller = app_state.dialog_controller.clone();
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
                                let bottom_sheet_controller =
                                    app_state.bottom_sheet_controller.clone();
                                let side_bar_controller = app_state.side_bar_controller.clone();
                                let dialog_controller = app_state.dialog_controller.clone();
                                scope.child(move || {
                                    let home_icon_content = filled::home_icon();
                                    let home_icon_args = IconArgsBuilder::default()
                                        .content(home_icon_content)
                                        .build()
                                        .unwrap();
                                    let about_icon_content = filled::info_icon();
                                    let about_icon_args = IconArgsBuilder::default()
                                        .content(about_icon_content)
                                        .build()
                                        .unwrap();

                                    navigation_bar(|scope| {
                                        scope.item(
                                            NavigationBarItemBuilder::default()
                                                .label("Home")
                                                .icon(Arc::new(closure!(
                                                    clone home_icon_args,
                                                    || {
                                                        icon(home_icon_args.clone());
                                                    }
                                                )))
                                                .on_click(Arc::new(closure!(
                                                    clone bottom_sheet_controller,
                                                    clone side_bar_controller,
                                                    clone dialog_controller,
                                                    || {
                                                        Router::with_mut(|router| {
                                                            router.reset_with(
                                                                HomeDestination {
                                                                    bottom_sheet_controller:
                                                                        bottom_sheet_controller
                                                                            .clone(),
                                                                    side_bar_controller:
                                                                        side_bar_controller.clone(),
                                                                    dialog_controller:
                                                                        dialog_controller.clone(),
                                                                },
                                                            );
                                                        });
                                                    }
                                                )))
                                                .build()
                                                .unwrap(),
                                        );

                                        scope.item(
                                            NavigationBarItemBuilder::default()
                                                .label("About")
                                                .icon(Arc::new(closure!(
                                                    clone about_icon_args,
                                                    || {
                                                        icon(about_icon_args.clone());
                                                    }
                                                )))
                                                .on_click(Arc::new(|| {
                                                    Router::with_mut(|router| {
                                                        router.reset_with(AboutDestination {});
                                                    });
                                                }))
                                                .build()
                                                .unwrap(),
                                        );
                                    });
                                });
                            });
                        },
                        closure!(clone dialog_controller, |_alpha| {
                            basic_dialog(
                                BasicDialogArgsBuilder::default()
                                    .headline("Basic Dialog")
                                    .supporting_text("This is a basic dialog component.")
                                    .icon(Arc::new(|| {
                                        let icon_content = filled::info_icon();
                                        icon(IconArgsBuilder::default().content(icon_content).build().unwrap());
                                    }))
                                    .confirm_button(closure!(clone dialog_controller, || {
                                        button(
                                            ButtonArgsBuilder::default()
                                                .on_click(Arc::new(closure!(clone dialog_controller, || {
                                                    dialog_controller.close();
                                                })))
                                                .build()
                                                .unwrap(),
                                            || text("Confirm"),
                                        );
                                    }))
                                    .dismiss_button(closure!(clone dialog_controller, || {
                                        button(
                                            ButtonArgsBuilder::default()
                                                .on_click(Arc::new(closure!(clone dialog_controller, || {
                                                    dialog_controller.close();
                                                })))
                                                .build()
                                                .unwrap(),
                                            || text("Dismiss"),
                                        );
                                    }))
                                    .build()
                                    .unwrap(),
                            );
                        }),
                    );
                },
                || {
                    surface(
                        SurfaceArgs {
                            style: Color::TRANSPARENT.into(),
                            padding: Dp(16.0),
                            ..Default::default()
                        },
                        || {
                            text("Hello from bottom sheet!");
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
    bottom_sheet_controller: Arc<BottomSheetController>,
    side_bar_controller: Arc<SideBarController>,
    dialog_controller: Arc<DialogController>,
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
                dialog_controller.open();
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
            "Layouts (Row, Column, Boxed)",
            "Components for arranging items horizontally, vertically, and with alignment.",
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
                bottom_sheet_controller.open();
            },
        ),
        ComponentExampleDesc::new(
            "Side Bar",
            "side bar displays content sliding in from the left side of the screen.",
            move || {
                side_bar_controller.open();
            },
        ),
    ]);

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .build()
            .unwrap(),
        move || {
            let examples_clone = examples.clone();

            lazy_column(
                LazyColumnArgsBuilder::default()
                    .scrollable(
                        ScrollableArgsBuilder::default()
                            .width(DimensionValue::FILLED)
                            .height(DimensionValue::FILLED)
                            .build()
                            .unwrap(),
                    )
                    .item_spacing(Dp(16.0))
                    .content_padding(Dp(8.0))
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .estimated_item_size(Dp(140.0))
                    .content_padding(Dp(16.0))
                    .build()
                    .unwrap(),
                move |scope| {
                    scope.items_from_iter(examples_clone.iter().cloned(), move |_, example| {
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
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .padding(Dp(25.0))
            .on_click(on_click)
            .style(SurfaceStyle::Filled {
                color: use_context::<MaterialColorScheme>().primary_container,
            })
            .shape(Shape::rounded_rectangle(Dp(25.0)))
            .shadow(ShadowProps::default())
            .build()
            .unwrap(),
        || {
            column(ColumnArgs::default(), |scope| {
                scope.child(move || {
                    text(
                        TextArgsBuilder::default()
                            .text(title)
                            .size(Dp(20.0))
                            .build()
                            .unwrap(),
                    );
                });
                scope.child(move || {
                    text(
                        TextArgsBuilder::default()
                            .text(description)
                            .size(Dp(14.0))
                            .color(use_context::<MaterialColorScheme>().on_surface_variant)
                            .build()
                            .unwrap(),
                    );
                });
            });
        },
    );
}

#[tessera]
fn top_app_bar() {
    surface(
        SurfaceArgsBuilder::default()
            .shadow(ShadowProps::default())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::Fixed(Dp(55.0).into()))
            .padding(Dp(5.0))
            .block_input(true)
            .build()
            .unwrap(),
        move || {
            row(
                RowArgsBuilder::default()
                    .width(DimensionValue::FILLED)
                    .height(DimensionValue::FILLED)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(move || {
                        let mut button_args = ButtonArgsBuilder::default()
                            .padding(Dp(5.0))
                            .shape(Shape::Ellipse)
                            .color(Color::TRANSPARENT)
                            .hover_color(Some(
                                use_context::<MaterialColorScheme>()
                                    .on_surface
                                    .with_alpha(0.1),
                            ))
                            .width(DimensionValue::Fixed(Dp(40.0).into()))
                            .height(DimensionValue::Fixed(Dp(40.0).into()));
                        if Router::with(|router| router.len()) > 1 {
                            button_args = button_args.on_click(Arc::new(|| {
                                Router::with_mut(|router| {
                                    router.pop();
                                });
                            }));
                        }

                        button(button_args.build().unwrap(), || {
                            boxed(
                                BoxedArgsBuilder::default()
                                    .width(DimensionValue::FILLED)
                                    .height(DimensionValue::FILLED)
                                    .alignment(Alignment::Center)
                                    .build()
                                    .unwrap(),
                                |scope| {
                                    scope.child(|| {
                                        text(
                                            TextArgsBuilder::default()
                                                .text("←".to_string())
                                                .size(Dp(25.0))
                                                .build()
                                                .unwrap(),
                                        );
                                    });
                                },
                            );
                        });
                    });
                },
            );
        },
    );
}

#[tessera]
#[shard]
fn about() {
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .padding(Dp(16.0))
            .build()
            .unwrap(),
        || {
            boxed(
                BoxedArgsBuilder::default()
                    .alignment(Alignment::Center)
                    .build()
                    .unwrap(),
                |scope| {
                    scope.child(|| {
                        text(
                            TextArgsBuilder::default()
                                .text(
                                    r#"This is an example app of Tessera UI Framework.
Made with ❤️ by tessera-ui devs.

Copyright 2025 Tessera UI Framework Developers
"#
                                    .to_string(),
                                )
                                .size(Dp(20.0))
                                .color(use_context::<MaterialColorScheme>().on_surface)
                                .build()
                                .unwrap(),
                        );
                    });
                },
            );
        },
    );
}
