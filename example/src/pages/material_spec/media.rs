use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    fonts::{Font, Fonts},
    image::image,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    surface::surface,
    text::text,
};
use tessera_ui::{Dp, Modifier, accesskit::Role, tessera};

use crate::res::I18N;

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

pub(crate) const FONTS: Fonts = Fonts(&[
    Font {
        asset: crate::res::material_spec::EXAMPLESANSDISPLAY_TTF,
        family: "Example Sans Display",
        fallback: &[],
        coverage: &[],
    },
    Font {
        asset: crate::res::material_spec::EXAMPLESANSTEXT_TTF,
        family: "Example Sans Text",
        fallback: &[],
        coverage: &[],
    },
    Font {
        asset: crate::res::material_spec::EXAMPLESANSTABS_TTF,
        family: "Example Sans Tabs",
        fallback: &[],
        coverage: &[],
    },
    Font {
        asset: crate::res::material_spec::EXAMPLESANSTABSHOVER_TTF,
        family: "Example Sans Tabs Hover",
        fallback: &[],
        coverage: &[],
    },
    Font {
        asset: crate::res::material_spec::EXAMPLESANSTABSSELECTED_TTF,
        family: "Example Sans Tabs Selected",
        fallback: &[],
        coverage: &[],
    },
    Font {
        asset: crate::res::material_spec::EXAMPLESANSTABSPRESSED_TTF,
        family: "Example Sans Tabs Pressed",
        fallback: &[],
        coverage: &[],
    },
]);

#[cfg(test)]
pub(super) fn load_fonts() {
    use tessera_ui::TesseraPackage;
    Fonts(FONTS.0).register(&mut tessera_ui::EntryRegistry::new());
}

pub(super) fn page_media(title: &str) -> Option<PageMedia> {
    use crate::res::material_spec as assets;
    Some(match title {
        "ui-components" => PageMedia {
            title: "ui-components",
            description: "ui-components-are-interactive-building-blocks-for-creating-a-user-interface-they-can-be-organ",
            slug: "components",
            asset: assets::COMPONENTS_PNG,
            width: 2000.0,
            height: 1125.0,
            thumbnail: None,
        },
        "ui-button-groups" => PageMedia {
            title: "ui-button-groups",
            description: "ui-button-groups-organize-buttons-and-add-interactions-between-them",
            slug: "button-groups",
            asset: assets::BUTTON_GROUPS_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::BUTTON_GROUPS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-buttons" => PageMedia {
            title: "ui-buttons",
            description: "ui-buttons-prompt-most-actions-in-a-ui-3685",
            slug: "buttons",
            asset: assets::BUTTONS_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-extended-fabs" => PageMedia {
            title: "ui-extended-fabs",
            description: "ui-extended-floating-action-buttons-extended-fabs-help-people-take-primary-actions",
            slug: "extended-fab",
            asset: assets::EXTENDED_FAB_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::EXTENDED_FAB_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-fab-menu" => PageMedia {
            title: "ui-fab-menu",
            description: "ui-the-floating-action-button-fab-menu-opens-from-a-fab-to-display-multiple-related-actions",
            slug: "fab-menu",
            asset: assets::FAB_MENU_PNG,
            width: 1000.0,
            height: 500.0,
            thumbnail: Some((assets::FAB_MENU_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-floating-action-buttons-fabs" | "ui-floating-action-buttons" | "FABs" => PageMedia {
            title: "ui-floating-action-buttons-fabs",
            description: "ui-floating-action-buttons-fabs-help-people-take-primary-actions",
            slug: "floating-action-button",
            asset: assets::FLOATING_ACTION_BUTTON_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::FLOATING_ACTION_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-icon-buttons" => PageMedia {
            title: "ui-icon-buttons",
            description: "ui-icon-buttons-help-people-take-actions-with-a-single-tap",
            slug: "icon-buttons",
            asset: assets::ICON_BUTTONS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::ICON_BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-segmented-buttons" => PageMedia {
            title: "ui-segmented-buttons",
            description: "ui-segmented-buttons-help-people-select-options-switch-views-or-sort-elements",
            slug: "segmented-buttons",
            asset: assets::SEGMENTED_BUTTONS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SEGMENTED_BUTTONS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-split-buttons" => PageMedia {
            title: "ui-split-buttons",
            description: "ui-split-buttons-open-a-menu-to-give-people-more-options-related-to-an-action",
            slug: "split-button",
            asset: assets::SPLIT_BUTTON_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::SPLIT_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-date-pickers" => PageMedia {
            title: "ui-date-pickers",
            description: "ui-date-pickers-let-people-select-a-date-or-a-range-of-dates",
            slug: "date-pickers",
            asset: assets::DATE_PICKERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DATE_PICKERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-time-pickers" => PageMedia {
            title: "ui-time-pickers",
            description: "ui-time-pickers-help-people-select-and-set-a-specific-time",
            slug: "time-pickers",
            asset: assets::TIME_PICKERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::TIME_PICKERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-loading-indicator" => PageMedia {
            title: "ui-loading-indicator",
            description: "ui-loading-indicators-show-the-progress-of-a-process-for-a-short-wait-time",
            slug: "loading-indicator",
            asset: assets::LOADING_INDICATOR_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::LOADING_INDICATOR_THUMBNAIL_PNG, 1004.0, 600.0)),
        },
        "ui-progress-indicators" | "ui-progress-indicator" => PageMedia {
            title: "ui-progress-indicators",
            description: "ui-progress-indicators-show-the-status-of-a-process-in-real-time",
            slug: "progress-indicators",
            asset: assets::PROGRESS_INDICATORS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::PROGRESS_INDICATORS_THUMBNAIL_PNG, 2048.0, 822.0)),
        },
        "ui-navigation-bar" => PageMedia {
            title: "ui-navigation-bar",
            description: "ui-navigation-bars-let-people-switch-between-ui-views-on-smaller-devices",
            slug: "navigation-bar",
            asset: assets::NAVIGATION_BAR_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::NAVIGATION_BAR_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-navigation-drawer" => PageMedia {
            title: "ui-navigation-drawer",
            description: "ui-navigation-drawers-let-people-switch-between-ui-views-on-larger-devices",
            slug: "navigation-drawer",
            asset: assets::NAVIGATION_DRAWER_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::NAVIGATION_DRAWER_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-navigation-rail" => PageMedia {
            title: "ui-navigation-rail",
            description: "ui-navigation-rails-let-people-switch-between-ui-views-on-mid-sized-devices",
            slug: "navigation-rail",
            asset: assets::NAVIGATION_RAIL_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::NAVIGATION_RAIL_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-bottom-sheets" => PageMedia {
            title: "ui-bottom-sheets",
            description: "ui-bottom-sheets-show-secondary-content-anchored-to-the-bottom-of-the-screen",
            slug: "bottom-sheets",
            asset: assets::BOTTOM_SHEETS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::BOTTOM_SHEETS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-side-sheets" => PageMedia {
            title: "ui-side-sheets",
            description: "ui-side-sheets-show-secondary-content-anchored-to-the-side-of-the-screen",
            slug: "side-sheets",
            asset: assets::SIDE_SHEETS_PNG,
            width: 1028.0,
            height: 640.0,
            thumbnail: Some((assets::SIDE_SHEETS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-app-bars" => PageMedia {
            title: "ui-app-bars",
            description: "ui-app-bars-are-placed-at-the-top-of-the-screen-to-help-people-navigate-through-a-product-3688",
            slug: "app-bars",
            asset: assets::APP_BARS_PNG,
            width: 771.0,
            height: 480.0,
            thumbnail: Some((assets::APP_BARS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-badges" => PageMedia {
            title: "ui-badges",
            description: "ui-badges-show-notifications-counts-or-status-information-on-navigation-items-and-icons-1606",
            slug: "badges",
            asset: assets::BADGES_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::BADGES_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-cards" => PageMedia {
            title: "ui-cards",
            description: "ui-cards-display-content-and-actions-about-a-single-subject",
            slug: "cards",
            asset: assets::CARDS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::CARDS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-carousel" => PageMedia {
            title: "ui-carousel",
            description: "ui-carousels-show-a-collection-of-items-that-can-be-scrolled-on-and-off-the-screen",
            slug: "carousel",
            asset: assets::CAROUSEL_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::CAROUSEL_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-checkbox" => PageMedia {
            title: "ui-checkbox",
            description: "ui-checkboxes-let-users-select-one-or-more-items-from-a-list-or-turn-an-item-on-or-off",
            slug: "checkbox",
            asset: assets::CHECKBOX_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::CHECKBOX_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-chips" => PageMedia {
            title: "ui-chips",
            description: "ui-chips-help-people-enter-information-make-selections-filter-content-or-trigger-actions",
            slug: "chips",
            asset: assets::CHIPS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::CHIPS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-dialogs" => PageMedia {
            title: "ui-dialogs",
            description: "ui-dialogs-provide-important-prompts-in-a-user-flow",
            slug: "dialogs",
            asset: assets::DIALOGS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DIALOGS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-divider" => PageMedia {
            title: "ui-divider",
            description: "ui-dividers-are-thin-lines-that-group-content-in-lists-or-other-containers",
            slug: "divider",
            asset: assets::DIVIDER_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::DIVIDER_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-lists" => PageMedia {
            title: "ui-lists",
            description: "ui-lists-are-continuous-vertical-indexes-of-text-and-images",
            slug: "lists",
            asset: assets::LISTS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::LISTS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-menus" => PageMedia {
            title: "ui-menus",
            description: "ui-menus-display-a-list-of-choices-on-a-temporary-surface",
            slug: "menus",
            asset: assets::MENUS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::MENUS_THUMBNAIL_PNG, 840.0, 560.0)),
        },
        "ui-radio-button" => PageMedia {
            title: "ui-radio-button",
            description: "ui-radio-buttons-let-people-select-one-option-from-a-set-of-options",
            slug: "radio-button",
            asset: assets::RADIO_BUTTON_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::RADIO_BUTTON_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-search" => PageMedia {
            title: "ui-search",
            description: "ui-search-lets-people-enter-a-keyword-or-phrase-to-get-relevant-information",
            slug: "search",
            asset: assets::SEARCH_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SEARCH_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-sliders" => PageMedia {
            title: "ui-sliders",
            description: "ui-sliders-allow-users-to-make-selections-from-a-range-of-values",
            slug: "sliders",
            asset: assets::SLIDERS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SLIDERS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-snackbar" => PageMedia {
            title: "ui-snackbar",
            description: "ui-snackbars-show-short-updates-about-app-processes-at-the-bottom-of-the-screen",
            slug: "snackbar",
            asset: assets::SNACKBAR_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SNACKBAR_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-switch" => PageMedia {
            title: "ui-switch",
            description: "ui-switches-toggle-the-selection-of-an-item-on-and-off",
            slug: "switch",
            asset: assets::SWITCH_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::SWITCH_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-tabs" => PageMedia {
            title: "ui-tabs",
            description: "ui-tabs-organize-content-across-different-screens-and-views",
            slug: "tabs",
            asset: assets::TABS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::TABS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-text-fields" => PageMedia {
            title: "ui-text-fields",
            description: "ui-text-fields-let-users-enter-text-into-a-ui",
            slug: "text-fields",
            asset: assets::TEXT_FIELDS_PNG,
            width: 2000.0,
            height: 1000.0,
            thumbnail: Some((assets::TEXT_FIELDS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-toolbars" => PageMedia {
            title: "ui-toolbars",
            description: "ui-toolbars-display-frequently-used-actions-relevant-to-the-current-page",
            slug: "toolbars",
            asset: assets::TOOLBARS_PNG,
            width: 1500.0,
            height: 750.0,
            thumbnail: Some((assets::TOOLBARS_THUMBNAIL_PNG, 1120.0, 560.0)),
        },
        "ui-tooltips" => PageMedia {
            title: "ui-tooltips",
            description: "ui-tooltips-display-brief-labels-or-messages",
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
        let scale = (width.0 / natural_width).max(height.0 / natural_height);
        boxed()
            .modifier(
                Modifier::new()
                    .size(width, height)
                    .clip_shape(Shape::rounded_rectangle(Dp(24.0)))
                    .semantics(SemanticsArgs {
                        role: Some(Role::Image),
                        label: Some(I18N.format(
                            "ui-value-illustration",
                            [("v0", I18N.text(&media.title).into())],
                        )),
                        test_tag: Some(format!("spec-hero-{}", media.slug)),
                        ..Default::default()
                    }),
            )
            .alignment(Alignment::Center)
            .children(move || {
                image().painter(asset).modifier(
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
                            text(I18N.text(&(title)))
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
                            text(I18N.text(&(description)))
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
