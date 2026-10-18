//! Example page that showcases Material components not covered by the basic
//! page.

use std::sync::Arc;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    badge::{badge, badge_with_content, badged_box},
    bottom_sheet::bottom_sheet_provider,
    boxed::boxed,
    button::button,
    button_groups::button_groups,
    card::card,
    carousel::{CarouselController, CarouselVariant, carousel},
    chip::chip,
    column::column,
    date_picker::{
        DatePickerState, DateRangePickerState, Weekday, date_picker_dialog, date_range_picker,
    },
    dialog::{basic_dialog, dialog_provider},
    divider::{horizontal_divider, vertical_divider},
    extended_floating_action_button::{
        ExtendedFloatingActionButtonSize, extended_floating_action_button,
    },
    floating_action_button::{FloatingActionButtonSize, floating_action_button},
    floating_action_button_menu::{
        FloatingActionButtonMenuColor, FloatingActionButtonMenuItem, floating_action_button_menu,
    },
    flow_row::flow_row,
    icon::icon,
    icon_button::{
        IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button,
    },
    lazy_list::lazy_column,
    loading_indicator::loading_indicator,
    material_icons::filled,
    menus::{
        MenuAnchor, MenuController, MenuPlacement, MenuPosition, menu_group, menu_item,
        menu_provider,
    },
    modifier::ModifierExt,
    navigation_bar::{navigation_bar, navigation_bar_item},
    navigation_drawer::{navigation_drawer, navigation_drawer_item},
    navigation_rail::{navigation_rail, navigation_rail_item},
    painter::try_painter_asset,
    radio_button::{RadioGroupOrientation, radio_button, radio_group},
    row::row,
    search::{docked_search_bar, search_bar},
    segmented_buttons::{
        SegmentedButtonDefaults, multi_choice_segmented_button_row, segmented_button,
        single_choice_segmented_button_row,
    },
    slider::{
        SliderOrientation, SliderSize, SliderValueIndicatorVisibility, centered_slider,
        range_slider, slider, slider_value_indicator_provider,
    },
    snackbar::{SnackbarHostState, SnackbarRequest, snackbar_host},
    spacer::spacer,
    split_buttons::{
        SplitButtonDefaults, SplitButtonSize, SplitButtonVariant, split_button_layout,
        split_leading_button, split_trailing_button,
    },
    surface::surface,
    tabs::{tab, tabs},
    text::text,
    text_field::{TextFieldBuilder, text_field},
    theme::MaterialTheme,
    time_picker::{TimePickerState, time_picker_dialog},
    toolbar::{ToolbarColorStyle, ToolbarOrientation, docked_toolbar, floating_toolbar},
    tooltip::{TooltipController, TooltipVariant, tooltip_anchor, tooltip_provider},
    wavy_progress::{circular_wavy_progress_indicator, linear_wavy_progress_indicator},
};
use tessera_shard::shard;
use tessera_ui::{
    Dp, Modifier, PxPosition, PxRect, PxSize, RenderSlot, State, modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

fn slider_demo_modifier(orientation: SliderOrientation) -> Modifier {
    match orientation {
        SliderOrientation::Horizontal => Modifier::new().width(Dp(320.0)),
        SliderOrientation::Vertical => Modifier::new().height(Dp(208.0)),
    }
}

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

#[tessera]
fn icon_button_demo(
    variant: State<IconButtonVariant>,
    width: State<IconButtonWidth>,
    shape: State<IconButtonShape>,
    toggle: State<bool>,
    enabled: State<bool>,
    selected: State<[bool; 5]>,
    actions: State<u32>,
) {
    column().children(move || {
        section_header("Icon buttons", "Five Expressive sizes with narrow, uniform, and wide containers. Tab to an action, then press Enter or Space.");
        flow_row().item_spacing(Dp(8.0)).line_spacing(Dp(8.0)).children(move || {
            button().tonal().on_click(move || variant.set(match variant.get() {
                IconButtonVariant::Filled => IconButtonVariant::FilledTonal,
                IconButtonVariant::FilledTonal => IconButtonVariant::Outlined,
                IconButtonVariant::Outlined => IconButtonVariant::Standard,
                IconButtonVariant::Standard => IconButtonVariant::Filled,
            })).child(move || {
                text().content(match variant.get() {
                    IconButtonVariant::Filled => "Style: Filled",
                    IconButtonVariant::FilledTonal => "Style: Tonal",
                    IconButtonVariant::Outlined => "Style: Outlined",
                    IconButtonVariant::Standard => "Style: Standard",
                });
            });
            button().tonal().on_click(move || width.set(match width.get() {
                IconButtonWidth::Uniform => IconButtonWidth::Wide,
                IconButtonWidth::Wide => IconButtonWidth::Narrow,
                IconButtonWidth::Narrow => IconButtonWidth::Uniform,
            })).child(move || {
                text().content(match width.get() {
                    IconButtonWidth::Uniform => "Width: Uniform",
                    IconButtonWidth::Wide => "Width: Wide",
                    IconButtonWidth::Narrow => "Width: Narrow",
                });
            });
            button().tonal().on_click(move || shape.set(if shape.get() == IconButtonShape::Round {
                IconButtonShape::Square
            } else { IconButtonShape::Round })).child(move || {
                text().content(if shape.get() == IconButtonShape::Round { "Shape: Round" } else { "Shape: Square" });
            });
            button().tonal().on_click(move || toggle.set(!toggle.get())).child(move || {
                text().content(if toggle.get() { "Mode: Toggle" } else { "Mode: Action" });
            });
            button().text().on_click(move || enabled.set(!enabled.get())).child(move || {
                text().content(if enabled.get() { "Disable icon buttons" } else { "Enable icon buttons" });
            });
        });
        spacer().modifier(Modifier::new().height(Dp(16.0)));
        flow_row().item_spacing(Dp(16.0)).line_spacing(Dp(16.0))
            .cross_axis_alignment(CrossAxisAlignment::Center).children(move || {
                for (index, size, label) in [
                    (0, IconButtonSize::ExtraSmall, "Extra-small favorite"),
                    (1, IconButtonSize::Small, "Small favorite"),
                    (2, IconButtonSize::Medium, "Medium favorite"),
                    (3, IconButtonSize::Large, "Large favorite"),
                    (4, IconButtonSize::ExtraLarge, "Extra-large favorite"),
                ] {
                    icon_button().icon(filled::STAR_SVG).size(size).width(width.get())
                        .variant(variant.get()).shape_style(shape.get()).enabled(enabled.get())
                        .selected_optional(if toggle.get() { Some(selected.get()[index]) } else { None })
                        .accessibility_label(label).on_click(move || {
                            actions.set(actions.get() + 1);
                            if toggle.get() { selected.with_mut(|values| values[index] = !values[index]); }
                        });
                }
            });
        spacer().modifier(Modifier::new().height(Dp(8.0)));
        text().content(format!("Icon actions: {}", actions.get()));
    });
}

struct MenuSourceLease {
    controller: State<MenuController>,
    split_index: State<Option<usize>>,
    split: bool,
}

impl Drop for MenuSourceLease {
    fn drop(&mut self) {
        if self.controller.is_alive()
            && self.split_index.is_alive()
            && self.split_index.get().is_some() == self.split
            && self.controller.with(MenuController::is_open)
        {
            self.controller.with_mut(MenuController::close);
        }
    }
}

#[tessera]
fn split_button_demo(
    variant: State<SplitButtonVariant>,
    enabled: State<bool>,
    status: State<String>,
    actions: State<u32>,
    menu: State<MenuController>,
    menu_index: State<Option<usize>>,
    anchors: State<[PxRect; 5]>,
) {
    let _lease = remember(move || MenuSourceLease {
        controller: menu,
        split_index: menu_index,
        split: true,
    });
    column().children(move || {
        section_header(
            "Split buttons",
            "Five sizes with primary actions and related menu options.",
        );
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                button()
                    .tonal()
                    .on_click(move || {
                        variant.set(match variant.get() {
                            SplitButtonVariant::Filled => SplitButtonVariant::Tonal,
                            SplitButtonVariant::Tonal => SplitButtonVariant::Outlined,
                            SplitButtonVariant::Outlined => SplitButtonVariant::Elevated,
                            SplitButtonVariant::Elevated => SplitButtonVariant::Filled,
                        })
                    })
                    .child(move || {
                        text().content(match variant.get() {
                            SplitButtonVariant::Filled => "Split style: Filled",
                            SplitButtonVariant::Tonal => "Split style: Tonal",
                            SplitButtonVariant::Outlined => "Split style: Outlined",
                            SplitButtonVariant::Elevated => "Split style: Elevated",
                        });
                    });
                button()
                    .text()
                    .on_click(move || enabled.set(!enabled.get()))
                    .child(move || {
                        text().content(if enabled.get() {
                            "Disable split buttons"
                        } else {
                            "Enable split buttons"
                        });
                    });
            });
        spacer().modifier(Modifier::new().height(Dp(12.0)));
        for (index, (size, name)) in [
            (SplitButtonSize::ExtraSmall, "extra-small"),
            (SplitButtonSize::Small, "small"),
            (SplitButtonSize::Medium, "medium"),
            (SplitButtonSize::Large, "large"),
            (SplitButtonSize::ExtraLarge, "extra-large"),
        ]
        .into_iter()
        .enumerate()
        {
            split_button_layout()
                .leading_button(move || {
                    split_leading_button()
                        .variant(variant.get())
                        .size(size)
                        .enabled(enabled.get())
                        .accessibility_label(format!("Send {name}"))
                        .on_click(move || {
                            actions.set(actions.get() + 1);
                            status.set(format!("Send {name} activated"));
                        })
                        .content(move || {
                            icon().painter(filled::SEND_SVG);
                            spacer().modifier(
                                Modifier::new().width(SplitButtonDefaults::icon_spacing(size)),
                            );
                            text().content("Send");
                        });
                })
                .trailing_button(move || {
                    split_trailing_button()
                        .variant(variant.get())
                        .size(size)
                        .enabled(enabled.get())
                        .modifier(Modifier::new().on_globally_positioned(move |bounds| {
                            if anchors.get()[index] != bounds {
                                anchors.with_mut(|anchors| anchors[index] = bounds);
                            }
                        }))
                        .expanded(
                            menu.with(MenuController::is_open) && menu_index.get() == Some(index),
                        )
                        .accessibility_label(format!("Send options {name}"))
                        .on_click(move || {
                            if menu.with(MenuController::is_open) && menu_index.get() == Some(index)
                            {
                                menu.with_mut(MenuController::close);
                            } else {
                                menu_index.set(Some(index));
                                menu.with_mut(MenuController::open);
                            }
                        })
                        .content(|| {
                            icon().painter(filled::ARROW_DROP_DOWN_SVG);
                        });
                });
            spacer().modifier(Modifier::new().height(Dp(12.0)));
        }
        text().content(format!("Split actions: {}", actions.get()));
        text().content(status.get());
    });
}

#[tessera]
fn carousel_demo(
    controller: State<CarouselController>,
    variant: State<CarouselVariant>,
    enabled: State<bool>,
    reverse: State<bool>,
    selected: State<String>,
) {
    let theme = use_context::<MaterialTheme>().expect("material theme");
    let painter = remember(|| try_painter_asset(crate::res::LOGO_PNG).expect("carousel artwork"));
    let aspect_ratios = remember(|| {
        Arc::<[f32]>::from([
            16.0 / 9.0,
            9.0 / 16.0,
            1.0,
            3.0 / 4.0,
            4.0 / 3.0,
            1.0,
            16.0 / 9.0,
            9.0 / 16.0,
            1.0,
            3.0 / 4.0,
            4.0 / 3.0,
            1.0,
        ])
    });
    column().children(move || {
        section_header("Carousel", if variant.get() == CarouselVariant::FullScreen { "Portrait full-screen preview. Drag vertically or use Up, Down, Home, and End." } else { "Items retain their full size as masks expand and collapse. Drag horizontally or use arrow keys, Home, and End." });
        flow_row().item_spacing(Dp(8.0)).line_spacing(Dp(8.0)).children(move || {
            button().tonal().on_click(move || {
                variant.set(match variant.get() {
                    CarouselVariant::MultiBrowse => CarouselVariant::Uncontained,
                    CarouselVariant::Uncontained => CarouselVariant::UncontainedMultiAspectRatio,
                    CarouselVariant::UncontainedMultiAspectRatio => CarouselVariant::Hero,
                    CarouselVariant::Hero => CarouselVariant::CenteredHero,
                    CarouselVariant::CenteredHero => CarouselVariant::FullScreen,
                    CarouselVariant::FullScreen => CarouselVariant::MultiBrowse,
                });
                controller.with_mut(|state| state.jump_to_item(0));
            }).child(move || {
                text().content(match variant.get() {
                    CarouselVariant::MultiBrowse => "Layout: Multi-browse",
                    CarouselVariant::Uncontained => "Layout: Uncontained",
                    CarouselVariant::UncontainedMultiAspectRatio => "Layout: Multi-aspect",
                    CarouselVariant::Hero => "Layout: Hero",
                    CarouselVariant::CenteredHero => "Layout: Centered hero",
                    CarouselVariant::FullScreen => "Layout: Full screen",
                });
            });
            button().text().on_click(move || controller.with_mut(|state| state.animate_to_item(state.current_item().saturating_sub(1)))).child(|| { text().content("Previous item"); });
            button().text().on_click(move || controller.with_mut(|state| state.animate_to_item(state.current_item().saturating_add(1)))).child(|| { text().content("Next item"); });
            button().text().on_click(move || reverse.set(!reverse.get())).child(move || { text().content(if reverse.get() { "Use forward layout" } else { "Use reverse layout" }); });
            button().text().on_click(move || enabled.set(!enabled.get())).child(move || { text().content(if enabled.get() { "Disable scrolling" } else { "Enable scrolling" }); });
        });
        spacer().modifier(Modifier::new().height(Dp(12.0)));
        carousel(12).modifier(if variant.get() == CarouselVariant::FullScreen { Modifier::new().width(Dp(320.0)) } else { Modifier::new().fill_max_width() }).height(Dp(if variant.get() == CarouselVariant::FullScreen { 480.0 } else { 180.0 }))
            .variant(variant.get()).controller(controller).item_aspect_ratios(aspect_ratios.get()).content_padding_start(Dp(0.0)).item_spacing(Dp(if variant.get() == CarouselVariant::FullScreen { 16.0 } else { 8.0 }))
            .preferred_item_width_optional(matches!(variant.get(), CarouselVariant::MultiBrowse | CarouselVariant::Uncontained).then_some(Dp(186.0)))
            .user_scroll_enabled(enabled.get()).reverse_layout(reverse.get()).accessibility_label("Featured collections")
            .content(move |item| {
                let scheme = theme.get().color_scheme;
                let (background, foreground) = match item.index % 3 {
                    0 => (scheme.primary_container, scheme.on_primary_container),
                    1 => (scheme.secondary_container, scheme.on_secondary_container),
                    _ => (scheme.tertiary_container, scheme.on_tertiary_container),
                };
                button().modifier(Modifier::new().fill_max_size()).padding(Dp(0.0))
                    .shape(tessera_components::shape_def::Shape::RECTANGLE).color(background).content_color(foreground)
                    .accessibility_label(format!("Open collection {}", item.index + 1))
                    .on_click(move || selected.set(format!("Collection {} activated", item.index + 1)))
                    .child(move || {
                        boxed().modifier(Modifier::new().fill_max_size()).alignment(Alignment::Center).children(move || {
                            tessera_components::image::image().painter(painter.get()).modifier(Modifier::new().size(Dp(100.0), Dp(100.0)));
                            column().modifier(Modifier::new().align(Alignment::BottomStart).padding_all(Dp(16.0))).children(move || {
                                let width = item.masked_width();
                                if width >= Dp(120.0) && width.0 + 0.5 >= item.full_width().0 {
                                    text().content(format!("Collection {}", item.index + 1)).style(theme.get().typography.title_medium);
                                }
                            });
                        });
                    });
            });
        text().content(format!("Current item: {} / 12", controller.with(CarouselController::current_item) + 1));
        text().content(selected.get());
    });
}

#[shard]
pub fn material_components_page() {
    let theme = use_context::<MaterialTheme>().unwrap();

    let carousel_controller = remember(CarouselController::default);
    let carousel_variant = remember(CarouselVariant::default);
    let carousel_enabled = remember(|| true);
    let carousel_reverse = remember(|| false);
    let carousel_selected = remember(|| "No item activated".to_string());
    let icon_variant = remember(|| IconButtonVariant::Filled);
    let icon_width = remember(IconButtonWidth::default);
    let icon_shape = remember(IconButtonShape::default);
    let icon_toggle = remember(|| false);
    let icon_enabled = remember(|| true);
    let icon_selected = remember(|| [false; 5]);
    let icon_actions = remember(|| 0_u32);
    let chip_selected = remember(|| false);
    let segmented_index = remember(|| 0_usize);
    let multi_segmented = remember(|| [false, false, false]);
    let tab_selected = remember(|| 0_usize);
    let radio_value = remember(|| 0_usize);
    let dialog_open = remember(|| false);
    let bottom_sheet_open = remember(|| false);
    let menu_controller = remember(MenuController::new);
    let menu_actions = remember(|| 0_u32);
    let menu_status = remember(|| "No menu action yet".to_string());
    let menu_trigger_bounds = remember(|| PxRect::ZERO);
    let page_bounds = remember(|| PxRect::ZERO);
    let group_status = remember(|| "No button group action yet".to_string());
    let split_status = remember(|| "No split action yet".to_string());
    let split_variant = remember(SplitButtonVariant::default);
    let split_enabled = remember(|| true);
    let split_actions = remember(|| 0_u32);
    let split_menu_index = remember(|| None::<usize>);
    let split_anchors = remember(|| [PxRect::ZERO; 5]);
    let fab_status = remember(|| "No floating action yet".to_string());
    let fab_expanded = remember(|| true);
    let fab_menu_color = remember(|| FloatingActionButtonMenuColor::Primary);
    let loading_paused = remember(|| false);
    let loading_progress = remember(|| 0.5_f32);
    let wave_progress = remember(|| 0.5_f32);
    let wave_paused = remember(|| false);
    let wave_indeterminate = remember(|| false);
    let wave_reversed = remember(|| false);
    let wave_thick = remember(|| false);
    let wave_flat = remember(|| false);
    let wave_stop = remember(|| true);
    let keyboard_slider_value = remember(|| 0.5_f32);
    let centered_slider_value = remember(|| 0.5_f32);
    let slider_range = remember(|| (0.2_f32, 0.8_f32));
    let slider_steps = remember(|| false);
    let slider_orientation = remember(SliderOrientation::default);
    let slider_reversed = remember(|| false);
    let slider_size = remember(SliderSize::default);
    let slider_custom_colors = remember(|| false);
    let slider_stop_indicators = remember(|| true);
    let slider_labels = remember(SliderValueIndicatorVisibility::default);
    let sliders_disabled = remember(|| false);
    let plain_tooltip = remember(TooltipController::new);
    let rich_tooltip = remember(TooltipController::new);
    let tooltip_clicks = remember(|| 0_u32);
    let tooltip_help = remember(|| false);
    let tooltip_action = remember(|| false);
    let snackbar_host_state = remember(SnackbarHostState::new);
    let date_state = remember(DatePickerState::default);
    let date_range_state = remember(DateRangePickerState::default);
    let date_range_status = remember(String::new);
    let time_state = remember(|| TimePickerState::new(10, 8, false, Default::default()));
    let picker_status = remember(String::new);
    let navigation_bar_selected = remember(|| 0_usize);
    let navigation_rail_selected = remember(|| 0_usize);
    let drawer_selected = remember(|| 0_usize);
    let toolbar_expanded = remember(|| true);
    let toolbar_status = remember(|| "No toolbar action yet".to_string());

    let page_content = RenderSlot::new(move || {
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
            .item(move || { carousel_demo(carousel_controller, carousel_variant, carousel_enabled, carousel_reverse, carousel_selected); })
            .item(move || { icon_button_demo(icon_variant, icon_width, icon_shape, icon_toggle, icon_enabled, icon_selected, icon_actions); })
            .item(move || {
                column().children(move || {
                    section_header("Tooltips", "Hover, focus with Tab, or hold on touch. Escape dismisses without activating the control.");
                    flow_row().item_spacing(Dp(16.0)).line_spacing(Dp(8.0)).children(move || {
                        tooltip_anchor().controller(plain_tooltip).child(move || {
                            button().tonal().on_click(move || tooltip_clicks.set(tooltip_clicks.get() + 1)).child(|| {
                                text().content("Plain help");
                            });
                        });
                        tooltip_anchor().controller(rich_tooltip).child(move || {
                            button().tonal().on_click(move || rich_tooltip.with_mut(TooltipController::show)).child(|| {
                                text().content("Rich help");
                            });
                        });
                        button().text().on_click(move || tooltip_help.set(!tooltip_help.get())).child(|| {
                            text().content("Change help text");
                        });
                    });
                    text().content(format!("Plain activations: {} · Rich action: {}", tooltip_clicks.get(), if tooltip_action.get() { "done" } else { "not yet" }));
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Sliders", "Use arrow keys, Home/End, and PageUp/PageDown. Tab between range handles.");
                    flow_row().item_spacing(Dp(12.0)).line_spacing(Dp(8.0)).children(move || {
                        button().tonal().on_click(move || {
                            let discrete = !slider_steps.get();
                            if discrete {
                                keyboard_slider_value.set((keyboard_slider_value.get() * 4.0).round() / 4.0);
                                centered_slider_value.set((centered_slider_value.get() * 4.0).round() / 4.0);
                                slider_range.with_mut(|range| {
                                    range.0 = (range.0 * 4.0).round() / 4.0;
                                    range.1 = (range.1 * 4.0).round() / 4.0;
                                });
                            }
                            slider_steps.set(discrete);
                        }).child(move || {
                            text().content(if slider_steps.get() { "Use continuous values" } else { "Use discrete steps" });
                        });
                        button().tonal().on_click(move || sliders_disabled.set(!sliders_disabled.get())).child(move || {
                            text().content(if sliders_disabled.get() { "Enable sliders" } else { "Disable sliders" });
                        });
                        button().tonal().on_click(move || slider_orientation.set(match slider_orientation.get() {
                            SliderOrientation::Horizontal => SliderOrientation::Vertical,
                            SliderOrientation::Vertical => SliderOrientation::Horizontal,
                        })).child(move || { text().content(format!("Orientation: {:?}", slider_orientation.get())); });
                        button().tonal().on_click(move || slider_reversed.set(!slider_reversed.get())).child(move || {
                            text().content(if slider_reversed.get() { "Direction: Reversed" } else { "Direction: Normal" });
                        });
                        button().tonal().on_click(move || slider_size.set(match slider_size.get() {
                            SliderSize::ExtraSmall => SliderSize::Small,
                            SliderSize::Small => SliderSize::Medium,
                            SliderSize::Medium => SliderSize::Large,
                            SliderSize::Large => SliderSize::ExtraLarge,
                            SliderSize::ExtraLarge => SliderSize::ExtraSmall,
                        })).child(move || { text().content(format!("Size: {:?}", slider_size.get())); });
                        button().tonal().on_click(move || slider_custom_colors.set(!slider_custom_colors.get())).child(move || {
                            text().content(if slider_custom_colors.get() { "Colors: Tertiary" } else { "Colors: Theme" });
                        });
                        button().tonal().on_click(move || slider_stop_indicators.set(!slider_stop_indicators.get())).child(move || {
                            text().content(if slider_stop_indicators.get() { "Hide end stops" } else { "Show end stops" });
                        });
                        button().tonal().on_click(move || slider_labels.set(match slider_labels.get() {
                            SliderValueIndicatorVisibility::Hidden => SliderValueIndicatorVisibility::OnInteraction,
                            SliderValueIndicatorVisibility::OnInteraction => SliderValueIndicatorVisibility::Always,
                            SliderValueIndicatorVisibility::Always => SliderValueIndicatorVisibility::Hidden,
                        })).child(move || { text().content(match slider_labels.get() {
                            SliderValueIndicatorVisibility::Hidden => "Labels: Off",
                            SliderValueIndicatorVisibility::OnInteraction => "Labels: Adjusting",
                            SliderValueIndicatorVisibility::Always => "Labels: Always",
                        }); });
                        button().text().on_click(move || {
                            keyboard_slider_value.set(0.5);
                            centered_slider_value.set(0.5);
                            slider_range.set(if slider_steps.get() { (0.25, 0.75) } else { (0.2, 0.8) });
                        }).child(|| { text().content("Reset values"); });
                    });
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    flow_row().item_spacing(Dp(16.0)).line_spacing(Dp(16.0)).children(move || {
                        column().modifier(Modifier::new().width(if slider_orientation.get() == SliderOrientation::Vertical { Dp(160.0) } else { Dp(320.0) })).children(move || {
                            text().content(format!("Single: {:.0}%", keyboard_slider_value.get() * 100.0));
                            spacer().modifier(Modifier::new().height(Dp(if slider_labels.get() == SliderValueIndicatorVisibility::Hidden { 0.0 } else { 52.0 })));
                            let scheme = theme.with(|theme| theme.color_scheme.clone());
                            let custom_colors = slider_custom_colors.get();
                            slider().modifier(slider_demo_modifier(slider_orientation.get())).orientation(slider_orientation.get()).reverse_direction(slider_reversed.get()).size(slider_size.get()).value_indicator(slider_labels.get()).active_track_color_optional(custom_colors.then_some(scheme.tertiary)).inactive_track_color_optional(custom_colors.then_some(scheme.tertiary_container)).thumb_color_optional(custom_colors.then_some(scheme.tertiary)).active_tick_color_optional(custom_colors.then_some(scheme.on_tertiary)).inactive_tick_color_optional(custom_colors.then_some(scheme.on_tertiary_container)).show_stop_indicator(slider_stop_indicators.get()).value(keyboard_slider_value.get()).on_change(move |value| keyboard_slider_value.set(value)).steps(if slider_steps.get() { 3 } else { 0 }).disabled(sliders_disabled.get()).accessibility_label("Single value");
                        });
                        column().modifier(Modifier::new().width(if slider_orientation.get() == SliderOrientation::Vertical { Dp(160.0) } else { Dp(320.0) })).children(move || {
                            text().content(format!("Centered: {:.0}%", centered_slider_value.get() * 100.0));
                            spacer().modifier(Modifier::new().height(Dp(if slider_labels.get() == SliderValueIndicatorVisibility::Hidden { 0.0 } else { 52.0 })));
                            let scheme = theme.with(|theme| theme.color_scheme.clone());
                            let custom_colors = slider_custom_colors.get();
                            centered_slider().modifier(slider_demo_modifier(slider_orientation.get())).orientation(slider_orientation.get()).reverse_direction(slider_reversed.get()).size(slider_size.get()).value_indicator(slider_labels.get()).active_track_color_optional(custom_colors.then_some(scheme.tertiary)).inactive_track_color_optional(custom_colors.then_some(scheme.tertiary_container)).thumb_color_optional(custom_colors.then_some(scheme.tertiary)).active_tick_color_optional(custom_colors.then_some(scheme.on_tertiary)).inactive_tick_color_optional(custom_colors.then_some(scheme.on_tertiary_container)).show_stop_indicator(slider_stop_indicators.get()).value(centered_slider_value.get()).on_change(move |value| centered_slider_value.set(value)).steps(if slider_steps.get() { 3 } else { 0 }).disabled(sliders_disabled.get()).accessibility_label("Centered value");
                        });
                        column().modifier(Modifier::new().width(if slider_orientation.get() == SliderOrientation::Vertical { Dp(160.0) } else { Dp(320.0) })).children(move || {
                            text().content(format!("Range: {:.0}% - {:.0}%", slider_range.get().0 * 100.0, slider_range.get().1 * 100.0));
                            spacer().modifier(Modifier::new().height(Dp(if slider_labels.get() == SliderValueIndicatorVisibility::Hidden { 0.0 } else { 52.0 })));
                            let scheme = theme.with(|theme| theme.color_scheme.clone());
                            let custom_colors = slider_custom_colors.get();
                            range_slider().modifier(slider_demo_modifier(slider_orientation.get())).orientation(slider_orientation.get()).reverse_direction(slider_reversed.get()).size(slider_size.get()).value_indicator(slider_labels.get()).active_track_color_optional(custom_colors.then_some(scheme.tertiary)).inactive_track_color_optional(custom_colors.then_some(scheme.tertiary_container)).thumb_color_optional(custom_colors.then_some(scheme.tertiary)).active_tick_color_optional(custom_colors.then_some(scheme.on_tertiary)).inactive_tick_color_optional(custom_colors.then_some(scheme.on_tertiary_container)).show_stop_indicator(slider_stop_indicators.get()).value(slider_range.get()).on_change(move |value| slider_range.set(value)).steps(if slider_steps.get() { 3 } else { 0 }).disabled(sliders_disabled.get()).accessibility_label("Value interval").start_accessibility_label("Minimum value").end_accessibility_label("Maximum value");
                        });
                    });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Loading indicators", "Morphing shapes, rotation, and determinate progress with optional containers.");
                    button().tonal().on_click(move || loading_paused.set(!loading_paused.get())).child(move || {
                        text().content(if loading_paused.get() { "Resume animation" } else { "Pause animation" });
                    });
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    flow_row().item_spacing(Dp(24.0)).line_spacing(Dp(16.0)).children(move || {
                        loading_indicator().paused(loading_paused.get()).accessibility_label("Loading content");
                        loading_indicator().contained(true).paused(loading_paused.get()).accessibility_label("Refreshing content");
                        loading_indicator().modifier(Modifier::new().size(Dp(64.0), Dp(64.0))).progress(loading_progress.get()).accessibility_label("Download progress");
                        loading_indicator().modifier(Modifier::new().size(Dp(64.0), Dp(64.0))).contained(true).progress(loading_progress.get()).accessibility_label("Upload progress");
                    });
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    slider().modifier(Modifier::new().width(Dp(240.0))).value(loading_progress.get()).on_change(move |value| loading_progress.set(value)).accessibility_label("Loading progress");
                    text().content(format!("Determinate progress: {:.0}%", loading_progress.get() * 100.0));
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Wavy progress", "Linear and circular waves for known completion or ongoing work, with independent track styling.");
                    flow_row().item_spacing(Dp(12.0)).line_spacing(Dp(12.0)).children(move || {
                        button().tonal().on_click(move || wave_indeterminate.set(!wave_indeterminate.get())).child(move || {
                            text().content(if wave_indeterminate.get() { "Mode: Ongoing" } else { "Mode: Completion" });
                        });
                        button().tonal().on_click(move || wave_paused.set(!wave_paused.get())).child(move || {
                            text().content(if wave_paused.get() { "Resume wave" } else { "Pause wave" });
                        });
                        button().tonal().on_click(move || wave_reversed.set(!wave_reversed.get())).child(move || {
                            text().content(if wave_reversed.get() { "Direction: Reverse" } else { "Direction: Normal" });
                        });
                        button().tonal().on_click(move || wave_thick.set(!wave_thick.get())).child(move || {
                            text().content(if wave_thick.get() { "Stroke: Thick" } else { "Stroke: Standard" });
                        });
                        button().tonal().on_click(move || wave_flat.set(!wave_flat.get())).child(move || {
                            text().content(if wave_flat.get() { "Shape: Flat" } else { "Shape: Wavy" });
                        });
                        button().tonal().on_click(move || wave_stop.set(!wave_stop.get())).child(move || {
                            text().content(if wave_stop.get() { "Hide stop" } else { "Show stop" });
                        });
                    });
                    spacer().modifier(Modifier::new().height(Dp(20.0)));
                    let stroke = if wave_thick.get() { 8.0 } else { 4.0 };
                    let height = if wave_thick.get() { 14.0 } else { 10.0 };
                    let wavelength = if wave_indeterminate.get() { 20.0 } else { 40.0 };
                    linear_wavy_progress_indicator()
                        .modifier(Modifier::new().width(Dp(320.0)).height(Dp(height)))
                        .progress_optional((!wave_indeterminate.get()).then_some(wave_progress.get()))
                        .paused(wave_paused.get())
                        .reverse_direction(wave_reversed.get())
                        .amplitude_optional(wave_flat.get().then_some(0.0))
                        .stroke_width(Dp(stroke))
                        .track_stroke_width(Dp(stroke))
                        .wavelength(Dp(wavelength))
                        .gap_size(Dp(4.0))
                        .stop_size(Dp(if wave_stop.get() { 4.0 } else { 0.0 }))
                        .accessibility_label("File transfer");
                    spacer().modifier(Modifier::new().height(Dp(20.0)));
                    circular_wavy_progress_indicator()
                        .modifier(Modifier::new().size(Dp(if wave_thick.get() { 52.0 } else { 48.0 }), Dp(if wave_thick.get() { 52.0 } else { 48.0 })))
                        .progress_optional((!wave_indeterminate.get()).then_some(wave_progress.get()))
                        .paused(wave_paused.get())
                        .reverse_direction(wave_reversed.get())
                        .amplitude_optional(wave_flat.get().then_some(0.0))
                        .stroke_width(Dp(stroke))
                        .track_stroke_width(Dp(stroke))
                        .accessibility_label("Circular file transfer");
                    spacer().modifier(Modifier::new().height(Dp(20.0)));
                    slider().modifier(Modifier::new().width(Dp(320.0))).value(wave_progress.get()).on_change(move |value| wave_progress.set(value)).accessibility_label("Transfer completion");
                    text().content(format!("Transfer completion: {:.0}%", wave_progress.get() * 100.0));
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "FAB menu",
                        "Open the creation menu. Click outside or press Escape to dismiss.",
                    );
                    button()
                        .tonal()
                        .on_click(move || {
                            fab_menu_color.set(match fab_menu_color.get() {
                                FloatingActionButtonMenuColor::Primary => {
                                    FloatingActionButtonMenuColor::Secondary
                                }
                                FloatingActionButtonMenuColor::Secondary => {
                                    FloatingActionButtonMenuColor::Tertiary
                                }
                                FloatingActionButtonMenuColor::Tertiary => {
                                    FloatingActionButtonMenuColor::Primary
                                }
                            });
                        })
                        .child(move || {
                            text().content(format!("Color: {:?}", fab_menu_color.get()));
                        });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    floating_action_button_menu()
                        .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
                        .color(fab_menu_color.get())
                        .items(vec![
                            FloatingActionButtonMenuItem::new(
                                "New document",
                                Some(filled::EDIT_SVG.into()),
                                tessera_ui::Callback::new(move || {
                                    fab_status.set("New document action".into())
                                }),
                            ),
                            FloatingActionButtonMenuItem::new(
                                "New folder",
                                Some(filled::FOLDER_SVG.into()),
                                tessera_ui::Callback::new(move || {
                                    fab_status.set("New folder action".into())
                                }),
                            ),
                            FloatingActionButtonMenuItem::new(
                                "Upload file",
                                Some(filled::UPLOAD_SVG.into()),
                                tessera_ui::Callback::new(move || {
                                    fab_status.set("Upload file action".into())
                                }),
                            ),
                        ])
                        .main_content(move || {
                            surface()
                                .modifier(Modifier::new().fill_max_size())
                                .style(theme.get().color_scheme.surface_container_low.into())
                                .child(move || {
                                    column()
                                        .modifier(Modifier::new().padding_all(Dp(16.0)))
                                        .children(move || {
                                            text()
                                                .content("Create something")
                                                .style(theme.get().typography.title_large);
                                            text().content(fab_status.get());
                                        });
                                });
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Extended FABs",
                        "Small, medium, and large primary actions with animated labels.",
                    );
                    button()
                        .tonal()
                        .on_click(move || fab_expanded.set(!fab_expanded.get()))
                        .child(move || {
                            text().content(if fab_expanded.get() {
                                "Collapse labels"
                            } else {
                                "Expand labels"
                            });
                        });
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    flow_row()
                        .item_spacing(Dp(16.0))
                        .line_spacing(Dp(16.0))
                        .children(move || {
                            extended_floating_action_button("Compose")
                                .icon(filled::EDIT_SVG)
                                .expanded(fab_expanded.get())
                                .on_click(move || fab_status.set("Compose action".into()));
                            extended_floating_action_button("Create")
                                .size(ExtendedFloatingActionButtonSize::Medium)
                                .icon(filled::ADD_SVG)
                                .expanded(fab_expanded.get())
                                .on_click(move || fab_status.set("Create action".into()));
                            extended_floating_action_button("New project")
                                .size(ExtendedFloatingActionButtonSize::Large)
                                .icon(filled::ADD_SVG)
                                .expanded(fab_expanded.get())
                                .on_click(move || fab_status.set("New project action".into()));
                        });
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    extended_floating_action_button("Label-only action")
                        .on_click(move || fab_status.set("Label-only action".into()));
                    spacer().modifier(Modifier::new().height(Dp(16.0)));
                    floating_action_button()
                        .size(FloatingActionButtonSize::Medium)
                        .accessibility_label("Medium floating action")
                        .on_click(move || fab_status.set("Medium floating action".into()))
                        .content(|| {
                            icon().painter(filled::ADD_SVG).size(Dp(28.0));
                        });
                    text().content(fab_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Toolbars",
                        "Floating and docked contextual actions with 48dp targets.",
                    );
                    floating_toolbar()
                        .accessibility_label("Editing actions")
                        .color_style(ToolbarColorStyle::Vibrant)
                        .expanded(toolbar_expanded.get())
                        .content(move || {
                            button()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .padding(Dp(12.0))
                                .shape(tessera_components::shape_def::Shape::CAPSULE)
                                .color(tessera_ui::Color::TRANSPARENT)
                                .content_color(theme.get().color_scheme.on_primary_container)
                                .accessibility_label("Expand editing actions")
                                .on_click(move || toolbar_expanded.set(!toolbar_expanded.get()))
                                .child(|| {
                                    icon().painter(filled::MENU_SVG);
                                });
                        })
                        .trailing(move || {
                            button()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .padding(Dp(12.0))
                                .shape(tessera_components::shape_def::Shape::CAPSULE)
                                .color(tessera_ui::Color::TRANSPARENT)
                                .content_color(theme.get().color_scheme.on_primary_container)
                                .accessibility_label("Save changes")
                                .on_click(move || toolbar_status.set("Changes saved".into()))
                                .child(|| {
                                    icon().painter(filled::CHECK_SVG);
                                });
                        });
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    floating_toolbar()
                        .orientation(ToolbarOrientation::Vertical)
                        .accessibility_label("Page actions")
                        .content(move || {
                            button()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .padding(Dp(12.0))
                                .color(tessera_ui::Color::TRANSPARENT)
                                .content_color(theme.get().color_scheme.on_surface_variant)
                                .accessibility_label("Go home")
                                .on_click(move || toolbar_status.set("Home selected".into()))
                                .child(|| {
                                    icon().painter(filled::HOME_SVG);
                                });
                        });
                    text().content(toolbar_status.get());
                    spacer().modifier(Modifier::new().height(Dp(12.0)));
                    docked_toolbar()
                        .accessibility_label("Document actions")
                        .content(move || {
                            button()
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .padding(Dp(12.0))
                                .color(tessera_ui::Color::TRANSPARENT)
                                .content_color(theme.get().color_scheme.on_surface_variant)
                                .accessibility_label("Mark document as saved")
                                .on_click(move || {
                                    toolbar_status.set("Document marked as saved".into())
                                })
                                .child(|| {
                                    icon().painter(filled::STAR_SVG);
                                });
                            button()
                                .filled()
                                .on_click(move || toolbar_status.set("Editing complete".into()))
                                .child(|| {
                                    text().content("Done");
                                });
                        });
                });
            })
            .item(move || {
                column().children(move || {
                    section_header(
                        "Navigation drawer",
                        "Select destinations using pointer or keyboard input.",
                    );
                    navigation_drawer()
                        .accessibility_label("Mail folders")
                        .content(move || {
                            text()
                                .content("Mail")
                                .style(theme.get().typography.title_small)
                                .modifier(Modifier::new().padding_all(Dp(16.0)));
                            navigation_drawer_item()
                                .label("Inbox")
                                .selected(drawer_selected.get() == 0)
                                .icon(|| {
                                    icon().painter(filled::HOME_SVG);
                                })
                                .badge(|| {
                                    text().content("24");
                                })
                                .on_click(move || drawer_selected.set(0));
                            navigation_drawer_item()
                                .label("Saved")
                                .selected(drawer_selected.get() == 1)
                                .icon(|| {
                                    icon().painter(filled::STAR_SVG);
                                })
                                .on_click(move || drawer_selected.set(1));
                            navigation_drawer_item()
                                .label("Unavailable")
                                .enabled(false)
                                .icon(|| {
                                    icon().painter(filled::SETTINGS_SVG);
                                });
                        });
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
                split_button_demo(split_variant, split_enabled, split_status, split_actions, menu_controller, split_menu_index, split_anchors);
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
                    let _lease = remember(move || MenuSourceLease {
                        controller: menu_controller,
                        split_index: split_menu_index,
                        split: false,
                    });
                    section_header("Menu", "Show a list of actions in a temporary surface.");
                    button()
                        .outlined()
                        .modifier(Modifier::new().on_globally_positioned(move |bounds| {
                            menu_trigger_bounds.set(bounds);
                        }))
                        .on_click(move || {
                            split_menu_index.set(None);
                            menu_controller.with_mut(MenuController::open);
                        })
                        .child(|| {
                            text().content("Open menu");
                        });
                    text().content(format!("Menu actions: {}", menu_actions.get()));
                    text().content(menu_status.get());
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
                    surface()
                        .style(theme.with(|t| t.color_scheme.surface_container_high).into())
                        .shape(theme.with(|t| t.shapes.extra_large))
                        .child(move || {
                            date_picker_dialog()
                                .state(date_state)
                                .picker_first_day_of_week(Weekday::Sunday)
                                .dismiss_button(move || {
                                    button()
                                        .text()
                                        .on_click(move || {
                                            date_state
                                                .with_mut(DatePickerState::clear_selected_date);
                                            picker_status.set("Date selection cleared".to_owned());
                                        })
                                        .child(|| {
                                            text().content("Cancel");
                                        });
                                })
                                .confirm_button(move || {
                                    button()
                                        .text()
                                        .enabled(date_state.with(|s| s.selected_date().is_some()))
                                        .on_click(move || {
                                            picker_status.set("Date confirmed".to_owned())
                                        })
                                        .child(|| {
                                            text().content("OK");
                                        });
                                });
                        });
                    text().content(picker_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Date range picker", "Select a period across calendar months or enter its dates.");
                    surface()
                        .style(theme.with(|theme| theme.color_scheme.surface_container_high).into())
                        .child(move || {
                            date_range_picker().state(date_range_state).first_day_of_week(Weekday::Sunday);
                        });
                    row().children(move || {
                        button().text().on_click(move || {
                            date_range_state.with_mut(DateRangePickerState::clear_selection);
                            date_range_status.set("Date range cleared".to_owned());
                        }).child(|| { text().content("Clear"); });
                        button().text().enabled(date_range_state.with(DateRangePickerState::is_complete)).on_click(move || {
                            date_range_status.set("Date range confirmed".to_owned());
                        }).child(|| { text().content("Save"); });
                    });
                    text().content(date_range_status.get());
                });
            })
            .item(move || {
                column().children(move || {
                    section_header("Time picker", "Select a time with the clock or keyboard.");
                    surface()
                        .style(theme.with(|t| t.color_scheme.surface_container_high).into())
                        .shape(theme.with(|t| t.shapes.extra_large))
                        .child(move || {
                            column()
                                .modifier(Modifier::new().padding_all(Dp(24.0)))
                                .children(move || {
                                    time_picker_dialog()
                                        .state(time_state)
                                        .dismiss_button(move || {
                                            button()
                                                .text()
                                                .on_click(move || {
                                                    picker_status
                                                        .set("Time selection cancelled".to_owned())
                                                })
                                                .child(|| {
                                                    text().content("Cancel");
                                                });
                                        })
                                        .confirm_button(move || {
                                            button()
                                                .text()
                                                .on_click(move || {
                                                    picker_status.set("Time confirmed".to_owned())
                                                })
                                                .child(|| {
                                                    text().content("OK");
                                                });
                                        });
                                });
                        });
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
            });
    });

    boxed()
        .modifier(
            Modifier::new()
                .fill_max_size()
                .on_globally_positioned(move |bounds| {
                    page_bounds.set(bounds);
                }),
        )
        .children(move || {
            dialog_provider()
                .is_open(dialog_open.get())
                .on_close_request(move || dialog_open.set(false))
                .main_content(move || {
                    bottom_sheet_provider()
                        .is_open(bottom_sheet_open.get())
                        .on_close_request(move || bottom_sheet_open.set(false))
                        .main_content(move || {
                            let trigger = if let Some(index) = split_menu_index.get() {
                                split_anchors.get()[index]
                            } else {
                                menu_trigger_bounds.get()
                            };
                            let page = page_bounds.get();
                            if menu_controller.with(MenuController::is_open)
                                && trigger.intersection(&page).is_none()
                            {
                                menu_controller.with_mut(MenuController::close);
                            }
                            menu_provider()
                                .anchor(MenuAnchor::new(
                                    PxPosition::new(trigger.x - page.x, trigger.y - page.y),
                                    PxSize::new(trigger.width, trigger.height),
                                ))
                                .grouped(true)
                                .placement(MenuPlacement::BelowStart)
                                .controller(menu_controller)
                                .main_content(move || {
                                    // The snackbar host is drawn after the page
                                    // so it overlays the bottom edge.
                                    boxed().modifier(Modifier::new().fill_max_size()).children(
                                        move || {
                                            tooltip_provider("Share this item with your team. You can change access at any time.")
                                                .variant(TooltipVariant::Rich)
                                                .headline("Sharing options")
                                                .actions(move || {
                                                    row().children(move || {
                                                        button().text().accessibility_label("Not now").on_click(move || rich_tooltip.with_mut(TooltipController::dismiss)).child(|| { text().content("Not now"); });
                                                        button().text().accessibility_label("Got it").on_click(move || {
                                                            rich_tooltip.with_mut(TooltipController::dismiss);
                                                            tooltip_action.set(true);
                                                        }).child(|| { text().content("Got it"); });
                                                    });
                                                })
                                                .controller(rich_tooltip)
                                                .child(move || {
                                                    tooltip_provider(if tooltip_help.get() { "This updated description wraps across several lines while staying within the tooltip width." } else { "Explain this action" })
                                                        .controller(plain_tooltip)
                                                        .child(move || {
                                                            slider_value_indicator_provider().child_shared(page_content);
                                                        });
                                                });
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
                                    if split_menu_index.get().is_some() {
                                        menu_group().children(move || {
                                        menu_item().label("Send later").position(MenuPosition::First).on_click(move || {
                                            split_actions.set(split_actions.get() + 1);
                                            split_status.set("Scheduled send selected".to_string());
                                            menu_controller.with_mut(MenuController::close);
                                        });
                                        menu_item().label("Save draft").position(MenuPosition::Last).on_click(move || {
                                            split_actions.set(split_actions.get() + 1);
                                            split_status.set("Draft saved".to_string());
                                            menu_controller.with_mut(MenuController::close);
                                        });
                                        });
                                    } else {
                                        menu_group().position(MenuPosition::First).children(move || {
                                            menu_item().label("Edit").position(MenuPosition::First).on_click(move || {
                                                menu_actions.set(menu_actions.get() + 1);
                                                menu_status.set("Edit selected".to_string());
                                            });
                                            menu_item().label("Duplicate").position(MenuPosition::Middle).on_click(move || {
                                                menu_actions.set(menu_actions.get() + 1);
                                                menu_status.set("Duplicate selected".to_string());
                                            });
                                            menu_item().label("Share").position(MenuPosition::Last).submenu_content(move || {
                                                menu_item().label("Copy link").position(MenuPosition::First).on_click(move || {
                                                    menu_actions.set(menu_actions.get() + 1);
                                                    menu_status.set("Copy link selected".to_string());
                                                });
                                                menu_item().label("More options").position(MenuPosition::Last).submenu_content(move || {
                                                    menu_item().label("Export link").on_click(move || {
                                                        menu_actions.set(menu_actions.get() + 1);
                                                        menu_status.set("Export link selected".to_string());
                                                    });
                                                });
                                            });
                                        });
                                        menu_group().position(MenuPosition::Last).children(move || {
                                            menu_item().label("Delete").on_click(move || {
                                                menu_actions.set(menu_actions.get() + 1);
                                                menu_status.set("Delete selected".to_string());
                                            });
                                        });
                                    }
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
