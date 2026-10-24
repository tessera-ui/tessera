//! Official illustration and typography assets for specification pages.
//!
//! ## Usage
//!
//! Present component articles with their attributed Material Design artwork.

use std::sync::Once;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    image::{TryIntoImageData as _, image},
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    surface::surface,
    text::{text, write_font_system},
};
use tessera_ui::{AssetExt as _, Dp, Modifier, accesskit::Role, remember_with_key, tessera};

use super::{INK, PANEL, gap, type_style};

#[derive(Clone, Copy)]
pub(super) struct PageMedia {
    pub title: &'static str,
    pub description: &'static str,
    pub slug: &'static str,
    asset: crate::res::Asset,
    width: f64,
    height: f64,
    thumbnail: Option<(crate::res::Asset, f64, f64)>,
}

pub(super) fn load_fonts() {
    static LOADED: Once = Once::new();
    LOADED.call_once(|| {
        let mut fonts = write_font_system();
        for asset in [
            crate::res::material_spec::EXAMPLESANSDISPLAY_TTF,
            crate::res::material_spec::EXAMPLESANSTEXT_TTF,
            crate::res::material_spec::EXAMPLESANSTABS_TTF,
            crate::res::material_spec::EXAMPLESANSTABSHOVER_TTF,
            crate::res::material_spec::EXAMPLESANSTABSSELECTED_TTF,
            crate::res::material_spec::EXAMPLESANSTABSPRESSED_TTF,
        ] {
            fonts.db_mut().load_font_data(
                asset
                    .read()
                    .expect("bundled specification font")
                    .as_ref()
                    .to_vec(),
            );
        }
    });
}

pub(super) fn page_media(title: &str) -> Option<PageMedia> {
    use crate::res::material_spec as assets;
    Some(match title {
        "Components" => PageMedia {
            title: "Components",
            description: "Components are interactive building blocks for creating a user interface. They can be organized into categories based on their purpose: Action, containment, communication, navigation, selection, and text input.",
            slug: "components",
            asset: assets::COMPONENTS_PNG,
            width: 3840.0,
            height: 2160.0,
            thumbnail: None,
        },
        "Button groups" => PageMedia {
            title: "Button groups",
            description: "Button groups organize buttons and add interactions between them",
            slug: "button-groups",
            asset: assets::BUTTON_GROUPS_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::BUTTON_GROUPS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Buttons" => PageMedia {
            title: "Buttons",
            description: "Buttons prompt most actions in a UI.",
            slug: "buttons",
            asset: assets::BUTTONS_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Extended FABs" => PageMedia {
            title: "Extended FABs",
            description: "Extended floating action buttons (extended FABs) help people take primary actions",
            slug: "extended-fab",
            asset: assets::EXTENDED_FAB_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::EXTENDED_FAB_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "FAB menu" => PageMedia {
            title: "FAB menu",
            description: "The floating action button (FAB) menu opens from a FAB to display multiple related actions",
            slug: "fab-menu",
            asset: assets::FAB_MENU_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::FAB_MENU_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Floating action buttons (FABs)" | "Floating action buttons" | "FABs" => PageMedia {
            title: "Floating action buttons (FABs)",
            description: "Floating action buttons (FABs) help people take primary actions",
            slug: "floating-action-button",
            asset: assets::FLOATING_ACTION_BUTTON_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::FLOATING_ACTION_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Icon buttons" => PageMedia {
            title: "Icon buttons",
            description: "Icon buttons help people take actions with a single tap",
            slug: "icon-buttons",
            asset: assets::ICON_BUTTONS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::ICON_BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Segmented buttons" => PageMedia {
            title: "Segmented buttons",
            description: "Segmented buttons help people select options, switch views, or sort elements",
            slug: "segmented-buttons",
            asset: assets::SEGMENTED_BUTTONS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SEGMENTED_BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Split buttons" => PageMedia {
            title: "Split buttons",
            description: "Split buttons open a menu to give people more options related to an action",
            slug: "split-button",
            asset: assets::SPLIT_BUTTON_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::SPLIT_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Date pickers" => PageMedia {
            title: "Date pickers",
            description: "Date pickers let people select a date, or a range of dates",
            slug: "date-pickers",
            asset: assets::DATE_PICKERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DATE_PICKERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Time pickers" => PageMedia {
            title: "Time pickers",
            description: "Time pickers help people select and set a specific time",
            slug: "time-pickers",
            asset: assets::TIME_PICKERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::TIME_PICKERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Loading indicator" => PageMedia {
            title: "Loading indicator",
            description: "Loading indicators show the progress of a process for a short wait time",
            slug: "loading-indicator",
            asset: assets::LOADING_INDICATOR_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::LOADING_INDICATOR_THUMBNAIL_PNG, 1004.0, 600.0)),
        },
        "Progress indicators" | "Progress indicator" => PageMedia {
            title: "Progress indicators",
            description: "Progress indicators show the status of a process in real time",
            slug: "progress-indicators",
            asset: assets::PROGRESS_INDICATORS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::PROGRESS_INDICATORS_THUMBNAIL_PNG, 2048.0, 822.0)),
        },
        "Navigation bar" => PageMedia {
            title: "Navigation bar",
            description: "Navigation bars let people switch between UI views on smaller devices",
            slug: "navigation-bar",
            asset: assets::NAVIGATION_BAR_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::NAVIGATION_BAR_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Navigation drawer" => PageMedia {
            title: "Navigation drawer",
            description: "Navigation drawers let people switch between UI views on larger devices",
            slug: "navigation-drawer",
            asset: assets::NAVIGATION_DRAWER_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::NAVIGATION_DRAWER_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Navigation rail" => PageMedia {
            title: "Navigation rail",
            description: "Navigation rails let people switch between UI views on mid-sized devices",
            slug: "navigation-rail",
            asset: assets::NAVIGATION_RAIL_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::NAVIGATION_RAIL_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Bottom sheets" => PageMedia {
            title: "Bottom sheets",
            description: "Bottom sheets show secondary content anchored to the bottom of the screen",
            slug: "bottom-sheets",
            asset: assets::BOTTOM_SHEETS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::BOTTOM_SHEETS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Side sheets" => PageMedia {
            title: "Side sheets",
            description: "Side sheets show secondary content anchored to the side of the screen",
            slug: "side-sheets",
            asset: assets::SIDE_SHEETS_PNG,
            width: 1028.0,
            height: 640.0,
            thumbnail: Some((assets::SIDE_SHEETS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "App bars" => PageMedia {
            title: "App bars",
            description: "App bars are placed at the top of the screen to help people navigate through a product.",
            slug: "app-bars",
            asset: assets::APP_BARS_PNG,
            width: 771.0,
            height: 480.0,
            thumbnail: Some((assets::APP_BARS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Badges" => PageMedia {
            title: "Badges",
            description: "Badges show notifications, counts, or status information on navigation items and icons",
            slug: "badges",
            asset: assets::BADGES_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::BADGES_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Cards" => PageMedia {
            title: "Cards",
            description: "Cards display content and actions about a single subject",
            slug: "cards",
            asset: assets::CARDS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::CARDS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Carousel" => PageMedia {
            title: "Carousel",
            description: "Carousels show a collection of items that can be scrolled on and off the screen",
            slug: "carousel",
            asset: assets::CAROUSEL_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::CAROUSEL_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Checkbox" => PageMedia {
            title: "Checkbox",
            description: "Checkboxes let users select one or more items from a list, or turn an item on or off",
            slug: "checkbox",
            asset: assets::CHECKBOX_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::CHECKBOX_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Chips" => PageMedia {
            title: "Chips",
            description: "Chips help people enter information, make selections, filter content, or trigger actions",
            slug: "chips",
            asset: assets::CHIPS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::CHIPS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Dialogs" => PageMedia {
            title: "Dialogs",
            description: "Dialogs provide important prompts in a user flow",
            slug: "dialogs",
            asset: assets::DIALOGS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DIALOGS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Divider" => PageMedia {
            title: "Divider",
            description: "Dividers are thin lines that group content in lists or other containers",
            slug: "divider",
            asset: assets::DIVIDER_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DIVIDER_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Lists" => PageMedia {
            title: "Lists",
            description: "Lists are continuous, vertical indexes of text and images",
            slug: "lists",
            asset: assets::LISTS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::LISTS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Menus" => PageMedia {
            title: "Menus",
            description: "Menus display a list of choices on a temporary surface",
            slug: "menus",
            asset: assets::MENUS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::MENUS_THUMBNAIL_PNG, 840.0, 560.0)),
        },
        "Radio button" => PageMedia {
            title: "Radio button",
            description: "Radio buttons let people select one option from a set of options",
            slug: "radio-button",
            asset: assets::RADIO_BUTTON_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::RADIO_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Search" => PageMedia {
            title: "Search",
            description: "Search lets people enter a keyword or phrase to get relevant information",
            slug: "search",
            asset: assets::SEARCH_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SEARCH_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Sliders" => PageMedia {
            title: "Sliders",
            description: "Sliders allow users to make selections from a range of values",
            slug: "sliders",
            asset: assets::SLIDERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SLIDERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Snackbar" => PageMedia {
            title: "Snackbar",
            description: "Snackbars show short updates about app processes at the bottom of the screen",
            slug: "snackbar",
            asset: assets::SNACKBAR_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SNACKBAR_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Switch" => PageMedia {
            title: "Switch",
            description: "Switches toggle the selection of an item on and off",
            slug: "switch",
            asset: assets::SWITCH_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SWITCH_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Tabs" => PageMedia {
            title: "Tabs",
            description: "Tabs organize content across different screens and views",
            slug: "tabs",
            asset: assets::TABS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::TABS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Text fields" => PageMedia {
            title: "Text fields",
            description: "Text fields let users enter text into a UI",
            slug: "text-fields",
            asset: assets::TEXT_FIELDS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::TEXT_FIELDS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Toolbars" => PageMedia {
            title: "Toolbars",
            description: "Toolbars display frequently used actions relevant to the current page",
            slug: "toolbars",
            asset: assets::TOOLBARS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::TOOLBARS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "Tooltips" => PageMedia {
            title: "Tooltips",
            description: "Tooltips display brief labels or messages",
            slug: "tooltips",
            asset: assets::TOOLTIPS_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::TOOLTIPS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        _ => return None,
    })
}

#[tessera]
pub(super) fn picture(#[prop(into)] title: String, width: Dp, height: Dp, thumbnail: Option<bool>) {
    if let Some(media) = page_media(&title) {
        let thumbnail = thumbnail.unwrap_or(false);
        let (asset, natural_width, natural_height) = if thumbnail {
            media
                .thumbnail
                .unwrap_or((media.asset, media.width, media.height))
        } else {
            (media.asset, media.width, media.height)
        };
        let pixels = remember_with_key((media.slug, thumbnail), || {
            asset
                .read()
                .expect("bundled specification illustration")
                .as_ref()
                .try_into_image_data()
                .expect("valid specification illustration")
        });
        let scale = (width.0 / natural_width).max(height.0 / natural_height);
        boxed()
            .modifier(
                Modifier::new()
                    .size(width, height)
                    .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                    .semantics(SemanticsArgs {
                        role: Some(Role::Image),
                        label: Some(format!("{} illustration", media.title)),
                        test_tag: Some(format!("spec-hero-{}", media.slug)),
                        ..Default::default()
                    }),
            )
            .alignment(Alignment::Center)
            .children(move || {
                image().painter(pixels.get()).modifier(
                    Modifier::new()
                        .required_size(Dp(natural_width * scale), Dp(natural_height * scale)),
                );
            });
    }
}

#[tessera]
pub(super) fn page_header(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    width: Dp,
) {
    let wide = width.0 >= 1295.0;
    let compact = width.0 <= 600.0;
    let media = page_media(&title);
    let title = media.map(|media| media.title.to_owned()).unwrap_or(title);
    let description = media
        .map(|media| media.description.to_owned())
        .unwrap_or(description);
    let available = (width.0 - 16.0).max(0.0);
    let cell_width = if wide && media.is_some() {
        (available - 8.0) * 0.5
    } else {
        available
    };
    boxed()
        .modifier(Modifier::new().fill_max_width().padding_all(Dp(8.0)))
        .children(move || {
            let title = title.clone();
            let description = description.clone();
            if wide && media.is_some() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .children(move || {
                        header_title(
                            title.clone(),
                            description.clone(),
                            Dp(cell_width),
                            compact,
                            true,
                        );
                        tessera_components::spacer::spacer()
                            .modifier(Modifier::new().width(Dp(8.0)));
                        picture(title.clone(), Dp(cell_width), Dp(544.0));
                    });
            } else {
                column().children(move || {
                    header_title(
                        title.clone(),
                        description.clone(),
                        Dp(cell_width),
                        compact,
                        false,
                    );
                    if media.is_some() {
                        gap(Dp(8.0));
                        picture(title.clone(), Dp(cell_width), Dp(cell_width * 0.5));
                    }
                });
            }
        });
}

#[tessera]
fn header_title(title: String, description: String, width: Dp, compact: bool, wide: bool) {
    let inset = if compact { Dp(32.0) } else { Dp(56.0) };
    let mut modifier = Modifier::new().width(width);
    if wide {
        modifier = modifier.height(Dp(544.0));
    }
    surface()
        .style(PANEL.into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(modifier)
        .content_alignment(if wide {
            Alignment::CenterStart
        } else {
            Alignment::TopStart
        })
        .child(move || {
            let title = title.clone();
            let description = description.clone();
            boxed()
                .modifier(Modifier::new().fill_max_width().padding_all(inset))
                .alignment(Alignment::CenterStart)
                .children(move || {
                    let title = title.clone();
                    let description = description.clone();
                    column()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || {
                            text()
                                .content(title.clone())
                                .style(if compact {
                                    type_style(45.0, 52.0)
                                } else if wide {
                                    type_style(96.0, 96.0)
                                } else {
                                    type_style(88.0, 96.0)
                                })
                                .color(INK)
                                .modifier(Modifier::new().semantics(SemanticsArgs {
                                    role: Some(Role::Heading),
                                    heading_level: Some(1),
                                    ..Default::default()
                                }));
                            gap(Dp(16.0));
                            text()
                                .content(description.clone())
                                .style(if compact {
                                    type_style(16.0, 24.0)
                                } else {
                                    type_style(22.0, 30.0)
                                })
                                .color(INK);
                        });
                });
        });
}
