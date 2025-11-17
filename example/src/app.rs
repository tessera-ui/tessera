use std::sync::Arc;

use dashmap::DashMap;
use tessera_ui::{
    Color, DimensionValue, Dp,
    router::{Router, router_root},
    shard, tessera,
};
use tessera_ui_basic_components::{
    RippleState,
    alignment::{Alignment, CrossAxisAlignment},
    bottom_nav_bar::{BottomNavBarState, bottom_nav_bar},
    bottom_sheet::{
        BottomSheetProviderArgsBuilder, BottomSheetProviderState, BottomSheetStyle,
        bottom_sheet_provider,
    },
    boxed::{BoxedArgsBuilder, boxed},
    button::{ButtonArgsBuilder, button},
    column::{ColumnArgs, column},
    dialog::{DialogProviderArgsBuilder, DialogProviderState, DialogStyle, dialog_provider},
    lazy_list::{LazyColumnArgsBuilder, LazyListState, lazy_column},
    pipelines::ShadowProps,
    row::{RowArgsBuilder, row},
    scrollable::ScrollableArgsBuilder,
    shape_def::Shape,
    side_bar::{SideBarProviderArgsBuilder, SideBarProviderState, SideBarStyle, side_bar_provider},
    surface::{SurfaceArgsBuilder, surface},
    text::{TextArgsBuilder, text},
};

use crate::example_components::{
    button::ButtonShowcaseDestination,
    checkbox::CheckboxShowcaseDestination,
    fluid_glass::FluidGlassShowcaseDestination,
    glass_button::GlassButtonShowcaseDestination,
    glass_progress::GlassProgressShowcaseDestination,
    glass_slider::GlassSliderShowcaseDestination,
    glass_switch::GlassSwitchShowcaseDestination,
    image::{IconShowcaseDestination, ImageShowcaseDestination},
    layouts::LayoutsShowcaseDestination,
    lazy_lists::LazyListsShowcaseDestination,
    progress::ProgressShowcaseDestination,
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
    bottom_nav_bar_state: BottomNavBarState,
    bottom_sheet_state: BottomSheetProviderState,
    side_bar_state: SideBarProviderState,
    dialog_state: DialogProviderState,
}

#[tessera]
#[shard]
pub fn app(#[state] app_state: AppState) {
    let state_for_bottom_sheet = app_state.clone();
    let state_for_side_bar = app_state.clone();
    let state_for_dialog = app_state.clone();
    side_bar_provider(
        SideBarProviderArgsBuilder::default()
            .on_close_request(Arc::new(move || {
                state_for_side_bar.side_bar_state.close();
            }))
            .style(SideBarStyle::Glass)
            .build()
            .unwrap(),
        app_state.side_bar_state.clone(),
        move || {
            bottom_sheet_provider(
                BottomSheetProviderArgsBuilder::default()
                    .on_close_request(Arc::new(move || {
                        state_for_bottom_sheet.bottom_sheet_state.close();
                    }))
                    .style(BottomSheetStyle::Glass)
                    .build()
                    .unwrap(),
                app_state.bottom_sheet_state.clone(),
                move || {
                    let dialog_state = app_state.dialog_state.clone();
                    dialog_provider(
                        DialogProviderArgsBuilder::default()
                            .on_close_request(Arc::new(move || {
                                dialog_state.close();
                            }))
                            .style(DialogStyle::Glass)
                            .build()
                            .unwrap(),
                        state_for_dialog.dialog_state.clone(),
                        move || {
                            column(ColumnArgs::default(), |scope| {
                                scope.child(|| {
                                    top_app_bar();
                                });
                                let bottom_sheet_state = app_state.bottom_sheet_state.clone();
                                let side_bar_state = app_state.side_bar_state.clone();
                                let dialog_state = app_state.dialog_state.clone();
                                scope.child_weighted(
                                    move || {
                                        router_root(HomeDestination {
                                            bottom_sheet_state,
                                            side_bar_state,
                                            dialog_state,
                                        });
                                    },
                                    1.0,
                                );
                                let bottom_sheet_state = app_state.bottom_sheet_state.clone();
                                let side_bar_state = app_state.side_bar_state.clone();
                                let dialog_state = app_state.dialog_state.clone();
                                scope.child(move || {
                                    bottom_nav_bar(
                                        app_state.bottom_nav_bar_state.clone(),
                                        |scope| {
                                            scope.child(
                                                move || {
                                                    text("Home");
                                                },
                                                move || {
                                                    Router::with_mut(|router| {
                                                        router.reset_with(HomeDestination {
                                                            bottom_sheet_state: bottom_sheet_state
                                                                .clone(),
                                                            side_bar_state: side_bar_state.clone(),
                                                            dialog_state: dialog_state.clone(),
                                                        });
                                                    });
                                                },
                                            );

                                            scope.child(
                                                || {
                                                    text("About");
                                                },
                                                || {
                                                    Router::with_mut(|router| {
                                                        router.reset_with(AboutDestination {});
                                                    });
                                                },
                                            );
                                        },
                                    );
                                });
                            });
                        },
                        move |alpha| {
                            text(
                                TextArgsBuilder::default()
                                    .text("Hello from Dialog!")
                                    .size(Dp(20.0))
                                    .color(Color::BLACK.with_alpha(alpha))
                                    .build()
                                    .unwrap(),
                            );
                        },
                    );
                },
                || {
                    text(
                        r#"Hi, I'm bottom sheet!

Bottom sheets are sheets at bottom, bottom at sheets, sheets bottom at, at bottom sheets..."#,
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

#[derive(Default)]
struct HomeState {
    lazy_list_state: Arc<LazyListState>,
    example_cards_ripple_state: DashMap<usize, RippleState>,
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
    #[state] home_state: HomeState,
    bottom_sheet_state: BottomSheetProviderState,
    side_bar_state: SideBarProviderState,
    dialog_state: DialogProviderState,
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
                dialog_state.open();
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
            "Button",
            "A clickable component with ripple effects for user interaction.",
            || {
                Router::with_mut(|router| {
                    router.push(ButtonShowcaseDestination {});
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
                bottom_sheet_state.open();
            },
        ),
        ComponentExampleDesc::new(
            "Side Bar",
            "side bar displays content sliding in from the left side of the screen.",
            move || {
                side_bar_state.open();
            },
        ),
    ]);

    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .style(Color::WHITE.into())
            .build()
            .unwrap(),
        None,
        move || {
            let state_clone = home_state.clone();
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
                home_state.lazy_list_state.clone(),
                move |scope| {
                    let ripple_map = state_clone.example_cards_ripple_state.clone();
                    scope.items_from_iter(examples_clone.iter().cloned(), move |index, example| {
                        let on_click = example.on_click.clone();
                        let surface_ripple_state = ripple_map
                            .entry(index)
                            .or_insert_with(RippleState::default)
                            .clone();
                        let title = example.title.clone();
                        let description = example.desription.clone();
                        component_card(&title, &description, surface_ripple_state, on_click);
                    });
                },
            );
        },
    );
}

#[tessera]
fn component_card(
    title: &str,
    description: &str,
    surface_ripple_state: RippleState,
    on_click: Arc<dyn Fn() + Send + Sync>,
) {
    let title = title.to_string();
    let description = description.to_string();
    surface(
        SurfaceArgsBuilder::default()
            .width(DimensionValue::FILLED)
            .padding(Dp(25.0))
            .on_click(on_click)
            .shape(Shape::rounded_rectangle(Dp(25.0)))
            .shadow(ShadowProps::default())
            .build()
            .unwrap(),
        Some(surface_ripple_state),
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
                            .color(Color::GRAY)
                            .build()
                            .unwrap(),
                    );
                });
            });
        },
    );
}

#[derive(Default)]
struct TopAppBarState {
    back_button_ripple_state: RippleState,
}

#[tessera]
#[shard]
fn top_app_bar(#[state] state: TopAppBarState) {
    surface(
        SurfaceArgsBuilder::default()
            .style(Color::WHITE.into())
            .shadow(ShadowProps::default())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::Fixed(Dp(55.0).into()))
            .padding(Dp(5.0))
            .block_input(true)
            .build()
            .unwrap(),
        None,
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
                            .hover_color(Some(Color::GRAY.with_alpha(0.1)))
                            .width(DimensionValue::Fixed(Dp(40.0).into()))
                            .height(DimensionValue::Fixed(Dp(40.0).into()));
                        if Router::with(|router| router.len()) > 1 {
                            button_args = button_args.on_click(Arc::new(|| {
                                Router::with_mut(|router| {
                                    router.pop();
                                });
                            }));
                        }

                        button(
                            button_args.build().unwrap(),
                            state.back_button_ripple_state.clone(),
                            || {
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
                            },
                        );
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
            .style(Color::WHITE.into())
            .width(DimensionValue::FILLED)
            .height(DimensionValue::FILLED)
            .padding(Dp(16.0))
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
                                .color(Color::BLACK)
                                .build()
                                .unwrap(),
                        );
                    });
                },
            );
        },
    );
}
