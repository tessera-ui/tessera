use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    experiment::hig::{
        button::{ButtonRole, ButtonStyle, button},
        color_picker::color_picker,
        content_unavailable_view::content_unavailable_view,
        control_group::{ControlGroupStyle, control_group},
        date_picker::{CalendarDate, DatePickerStyle, date_picker},
        disclosure_button::disclosure_button,
        disclosure_group::{OutlineItem, disclosure_group, outline_group},
        divider::{DividerOrientation, divider},
        form::{FormStyle, form, form_section, labeled_content},
        gauge::{GaugeStyle, gauge},
        glass::{GlassAppearance, GlassMaterial, glass},
        glass_group::glass_group,
        grid::{GridItem, grid},
        group_box::group_box,
        help_link::help_link,
        label::{LabelStyle, label as hig_label},
        link::link,
        list::{ListStyle, list, list_row, list_section},
        menu::{MenuStyle, context_menu, menu, menu_divider, menu_item, menu_section},
        navigation_split_view::{NavigationSplitViewVisibility, navigation_split_view},
        picker::{PickerStyle, picker},
        progress::{ProgressViewStyle, progress_view},
        scroll_view::scroll_view,
        slider::slider,
        split_view::{h_split_view, v_split_view},
        stepper::stepper,
        tab_view::tab_view,
        table::{TableColumn, TableSort, table},
        text::text,
        text_field::{TextFieldStyle, search_field, secure_field, text_editor, text_field},
        theme::{Appearance, ControlSize, HigTheme, TextStyle},
        toggle::{ToggleStyle, toggle},
    },
    image::{ImageData, image},
    spacer::spacer,
};
use tessera_foundation::modifier::ModifierExt as _;
use tessera_ui::{Color, Dp, Modifier, State, remember, tessera, use_context};

use crate::res::I18N;

/// One article: a titled set of sections rendered side by side in light and dark windows.
#[derive(Clone, Copy)]
pub(super) struct Component {
    pub title: &'static str,
    pub description: &'static str,
    pub category: usize,
    pub sections: &'static [Section],
}

/// A grid of examples. An empty row or column label hides that header.
#[derive(Clone, Copy)]
pub(super) struct Section {
    pub title: &'static str,
    pub columns: &'static [&'static str],
    pub rows: &'static [&'static str],
    pub height: f64,
}

pub(super) const CATEGORIES: [&str; 6] = [
    "ui-hig-actions",
    "ui-hig-selection",
    "ui-hig-values",
    "ui-hig-content",
    "ui-hig-collections",
    "ui-hig-navigation-views",
];

const SINGLE: &[&str] = &[""];
const ENABLED: &[&str] = &["ui-hig-enabled", "ui-hig-disabled"];
const BUTTON_STYLES: &[&str] = &[
    "ui-hig-style-automatic",
    "ui-hig-style-bordered",
    "ui-hig-style-bordered-prominent",
    "ui-hig-style-borderless",
    "ui-hig-style-plain",
    "ui-hig-style-link",
    "ui-hig-style-accessory-bar",
    "ui-hig-style-accessory-bar-action",
    "ui-hig-style-glass",
    "ui-hig-style-glass-prominent",
];
const BUTTON_STYLE_VALUES: [ButtonStyle; 10] = [
    ButtonStyle::Automatic,
    ButtonStyle::Bordered,
    ButtonStyle::BorderedProminent,
    ButtonStyle::Borderless,
    ButtonStyle::Plain,
    ButtonStyle::Link,
    ButtonStyle::AccessoryBar,
    ButtonStyle::AccessoryBarAction,
    ButtonStyle::Glass,
    ButtonStyle::GlassProminent,
];

const fn example(height: f64) -> Section {
    Section {
        title: "ui-hig-example",
        columns: SINGLE,
        rows: SINGLE,
        height,
    }
}

const fn states(columns: &'static [&'static str], rows: &'static [&'static str]) -> Section {
    Section {
        title: "ui-hig-states",
        columns,
        rows,
        height: 64.0,
    }
}

pub(super) const COMPONENTS: &[Component] = &[
    Component {
        title: "ui-buttons",
        description: "ui-hig-buttons-description",
        category: 0,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-styles",
                columns: ENABLED,
                rows: BUTTON_STYLES,
                height: 48.0,
            },
            Section {
                title: "ui-hig-roles",
                columns: &[
                    "ui-hig-role-none",
                    "ui-hig-role-cancel",
                    "ui-hig-role-destructive",
                ],
                rows: &["ui-hig-style-bordered", "ui-hig-style-bordered-prominent"],
                height: 56.0,
            },
            Section {
                title: "ui-hig-control-sizes",
                columns: &[
                    "ui-hig-size-mini",
                    "ui-hig-size-small",
                    "ui-hig-size-regular",
                    "ui-hig-size-large",
                    "ui-hig-size-extra-large",
                ],
                rows: SINGLE,
                height: 72.0,
            },
        ],
    },
    Component {
        title: "ui-hig-links",
        description: "ui-hig-links-description",
        category: 0,
        sections: &[example(120.0), states(ENABLED, SINGLE)],
    },
    Component {
        title: "ui-hig-help-links",
        description: "ui-hig-help-description",
        category: 0,
        sections: &[example(120.0), states(ENABLED, SINGLE)],
    },
    Component {
        title: "ui-hig-disclosure-buttons",
        description: "ui-hig-disclosure-description",
        category: 0,
        sections: &[
            example(120.0),
            states(&["ui-hig-collapsed", "ui-hig-expanded"], ENABLED),
        ],
    },
    Component {
        title: "ui-hig-toggles",
        description: "ui-hig-toggles-description",
        category: 1,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-styles",
                columns: &["ui-hig-off", "ui-hig-on", "ui-hig-mixed"],
                rows: &["ui-hig-checkbox", "ui-hig-switch", "ui-hig-button"],
                height: 56.0,
            },
            Section {
                title: "ui-hig-disabled",
                columns: &["ui-hig-off", "ui-hig-on", "ui-hig-mixed"],
                rows: &["ui-hig-checkbox", "ui-hig-switch", "ui-hig-button"],
                height: 56.0,
            },
        ],
    },
    Component {
        title: "ui-hig-menus",
        description: "ui-hig-menus-description",
        category: 0,
        sections: &[
            example(220.0),
            Section {
                title: "ui-hig-styles",
                columns: ENABLED,
                rows: &["ui-hig-style-bordered", "ui-hig-style-borderless"],
                height: 56.0,
            },
        ],
    },
    Component {
        title: "ui-hig-control-groups",
        description: "ui-hig-control-groups-description",
        category: 0,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-styles",
                columns: &["ui-hig-style-automatic", "ui-hig-style-menu"],
                rows: SINGLE,
                height: 56.0,
            },
        ],
    },
    Component {
        title: "ui-hig-pickers",
        description: "ui-hig-pickers-description",
        category: 1,
        sections: &[
            example(150.0),
            Section {
                title: "ui-hig-styles",
                columns: &["ui-hig-pop-up", "ui-hig-segmented", "ui-hig-radio-group"],
                rows: ENABLED,
                height: 80.0,
            },
        ],
    },
    Component {
        title: "ui-sliders",
        description: "ui-hig-slider-description",
        category: 2,
        sections: &[
            example(120.0),
            states(
                &["ui-hig-minimum", "ui-hig-middle", "ui-hig-maximum"],
                ENABLED,
            ),
        ],
    },
    Component {
        title: "ui-hig-steppers",
        description: "ui-hig-stepper-description",
        category: 2,
        sections: &[example(120.0), states(ENABLED, SINGLE)],
    },
    Component {
        title: "ui-progress-indicators",
        description: "ui-hig-progress-description",
        category: 2,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-determinate",
                columns: &["0%", "50%", "100%"],
                rows: SINGLE,
                height: 64.0,
            },
            Section {
                title: "ui-hig-indeterminate",
                columns: &["ui-hig-bar", "ui-hig-spinner"],
                rows: SINGLE,
                height: 64.0,
            },
        ],
    },
    Component {
        title: "ui-hig-gauges",
        description: "ui-hig-gauges-description",
        category: 2,
        sections: &[
            example(140.0),
            Section {
                title: "ui-hig-linear",
                columns: &[
                    "ui-hig-capacity",
                    "ui-hig-accessory-capacity",
                    "ui-hig-accessory",
                ],
                rows: SINGLE,
                height: 72.0,
            },
            Section {
                title: "ui-hig-circular",
                columns: &["ui-hig-accessory", "ui-hig-accessory-capacity"],
                rows: SINGLE,
                height: 80.0,
            },
        ],
    },
    Component {
        title: "ui-hig-dividers",
        description: "ui-hig-divider-description",
        category: 3,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-orientation",
                columns: &["ui-hig-horizontal", "ui-hig-vertical"],
                rows: SINGLE,
                height: 80.0,
            },
        ],
    },
    Component {
        title: "ui-hig-group-boxes",
        description: "ui-hig-group-description",
        category: 3,
        sections: &[
            example(140.0),
            Section {
                title: "ui-hig-styles",
                columns: &["ui-hig-titled", "ui-hig-untitled"],
                rows: SINGLE,
                height: 120.0,
            },
        ],
    },
    Component {
        title: "ui-hig-glass",
        description: "ui-hig-glass-description",
        category: 3,
        sections: &[
            example(140.0),
            Section {
                title: "ui-hig-materials",
                columns: &["ui-hig-capsule", "ui-hig-rounded"],
                rows: &["ui-hig-regular", "ui-hig-clear"],
                height: 96.0,
            },
            Section {
                title: "ui-hig-fusion",
                columns: &["ui-hig-regular", "ui-hig-clear"],
                rows: SINGLE,
                height: 96.0,
            },
        ],
    },
    Component {
        title: "ui-hig-text",
        description: "ui-hig-text-description",
        category: 3,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-colors",
                columns: &["ui-hig-label", "ui-hig-secondary", "ui-hig-tertiary"],
                rows: SINGLE,
                height: 48.0,
            },
        ],
    },
    Component {
        title: "ui-hig-text-fields",
        description: "ui-hig-text-field-description",
        category: 3,
        sections: &[
            example(260.0),
            states(&["ui-hig-empty", "ui-hig-filled"], ENABLED),
            Section {
                title: "ui-hig-styles",
                columns: &[
                    "ui-hig-style-rounded",
                    "ui-hig-style-square",
                    "ui-hig-style-plain",
                ],
                rows: SINGLE,
                height: 56.0,
            },
        ],
    },
    Component {
        title: "ui-hig-scroll-views",
        description: "ui-hig-scroll-views-description",
        category: 4,
        sections: &[example(180.0)],
    },
    Component {
        title: "ui-hig-lists",
        description: "ui-hig-lists-description",
        category: 4,
        sections: &[
            example(200.0),
            Section {
                title: "ui-hig-styles",
                columns: &[
                    "ui-hig-style-inset",
                    "ui-hig-style-plain",
                    "ui-hig-style-sidebar",
                ],
                rows: SINGLE,
                height: 120.0,
            },
        ],
    },
    Component {
        title: "ui-hig-disclosure-groups",
        description: "ui-hig-disclosure-groups-description",
        category: 4,
        sections: &[example(200.0)],
    },
    Component {
        title: "ui-hig-tables",
        description: "ui-hig-tables-description",
        category: 4,
        sections: &[example(200.0)],
    },
    Component {
        title: "ui-hig-grids",
        description: "ui-hig-grids-description",
        category: 4,
        sections: &[example(200.0)],
    },
    Component {
        title: "ui-hig-split-views",
        description: "ui-hig-split-views-description",
        category: 5,
        sections: &[example(220.0)],
    },
    Component {
        title: "ui-hig-navigation-split-views",
        description: "ui-hig-navigation-split-views-description",
        category: 5,
        sections: &[example(260.0)],
    },
    Component {
        title: "ui-hig-tab-views",
        description: "ui-hig-tab-views-description",
        category: 5,
        sections: &[example(180.0)],
    },
    Component {
        title: "ui-hig-forms",
        description: "ui-hig-forms-description",
        category: 3,
        sections: &[example(240.0)],
    },
    Component {
        title: "ui-hig-date-pickers",
        description: "ui-hig-date-pickers-description",
        category: 2,
        sections: &[
            example(260.0),
            Section {
                title: "ui-hig-styles",
                columns: &[
                    "ui-hig-style-compact",
                    "ui-hig-style-field",
                    "ui-hig-style-stepper-field",
                ],
                rows: SINGLE,
                height: 56.0,
            },
        ],
    },
    Component {
        title: "ui-hig-color-pickers",
        description: "ui-hig-color-pickers-description",
        category: 2,
        sections: &[example(120.0)],
    },
    Component {
        title: "ui-hig-content-unavailable",
        description: "ui-hig-content-unavailable-description",
        category: 3,
        sections: &[example(200.0)],
    },
    Component {
        title: "ui-hig-labels",
        description: "ui-hig-labels-description",
        category: 3,
        sections: &[
            example(120.0),
            Section {
                title: "ui-hig-styles",
                columns: &[
                    "ui-hig-style-title-and-icon",
                    "ui-hig-style-icon-only",
                    "ui-hig-style-title-only",
                ],
                rows: SINGLE,
                height: 56.0,
            },
        ],
    },
];

/// Resolves a localization key; other strings are shown verbatim.
pub(super) fn label(key: &str) -> String {
    if key.starts_with("ui-") {
        I18N.text(key)
    } else {
        key.to_owned()
    }
}

/// Values shared by the interactive examples of one article, so the light and dark windows
/// always show the same state.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ExampleValues {
    clicks: u32,
    expanded: bool,
    mixed: bool,
    selected: usize,
    on: bool,
    fraction: f64,
    count: f64,
}

impl Default for ExampleValues {
    fn default() -> Self {
        Self {
            clicks: 0,
            expanded: false,
            mixed: true,
            selected: 0,
            on: false,
            fraction: 0.5,
            count: 5.0,
        }
    }
}

pub(super) type ExampleState = State<ExampleValues>;

/// Secondary and tertiary label colors of the given appearance.
pub(super) fn label_colors(appearance: Appearance) -> [Color; 3] {
    match appearance {
        Appearance::Light => [
            Color::BLACK.with_alpha(0.847),
            Color::BLACK.with_alpha(0.498),
            Color::BLACK.with_alpha(0.259),
        ],
        Appearance::Dark => [
            Color::WHITE.with_alpha(0.847),
            Color::WHITE.with_alpha(0.549),
            Color::WHITE.with_alpha(0.247),
        ],
    }
}

fn backdrop_image() -> ImageData {
    let (width, height) = (320, 160);
    let mut pixels = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let stripe = if (x + y) / 12 % 2 == 0 { 36 } else { 0 };
            pixels.extend_from_slice(&[
                (60 + x * 150 / width + stripe) as u8,
                (90 + y * 100 / height) as u8,
                (220 - x * 80 / width) as u8,
                255,
            ]);
        }
    }
    ImageData::from_rgba(pixels, width as u32, height as u32)
}

/// One grid cell: the control for `section`, `row` and `column` of component `index`.
#[tessera]
pub(super) fn cell(
    index: usize,
    section: usize,
    row: usize,
    column: usize,
    state: ExampleState,
    width: Dp,
    height: Dp,
) {
    let appearance = use_context::<HigTheme>().unwrap().get().appearance;
    let backdrop = COMPONENTS[index].title == "ui-hig-glass";
    let image_data = remember(backdrop_image);
    boxed()
        .alignment(Alignment::Center)
        .modifier(Modifier::new().size(width, height).clip_to_bounds())
        .children(move || {
            if backdrop {
                image()
                    .painter(image_data.get())
                    .modifier(Modifier::new().fill_max_size());
            }
            if section == 0 {
                interactive(index, state, appearance);
            } else {
                static_state(index, section, row, column, appearance);
            }
        });
}

fn glass_appearance(appearance: Appearance) -> GlassAppearance {
    match appearance {
        Appearance::Light => GlassAppearance::Light,
        Appearance::Dark => GlassAppearance::Dark,
    }
}

#[tessera]
fn interactive(index: usize, state: ExampleState, appearance: Appearance) {
    let values = state.get();
    column()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || match COMPONENTS[index].title {
            "ui-buttons" => {
                button(I18N.text("ui-hig-action"))
                    .default_action(true)
                    .on_click(move || state.with_mut(|v| v.clicks += 1));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(values.clicks.to_string(), appearance);
            }
            "ui-hig-links" => {
                link(
                    I18N.text("ui-hig-link-title"),
                    "https://example.com/privacy",
                )
                .on_open(move |_| state.with_mut(|v| v.clicks += 1));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(values.clicks.to_string(), appearance);
            }
            "ui-hig-help-links" => {
                help_link()
                    .accessibility_label(I18N.text("ui-hig-help-links"))
                    .on_click(move || state.with_mut(|v| v.expanded = !v.expanded));
                if values.expanded {
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    caption(I18N.text("ui-hig-help-content"), appearance);
                }
            }
            "ui-hig-disclosure-buttons" => {
                disclosure_button(values.expanded)
                    .accessibility_label(I18N.text("ui-hig-details"))
                    .on_expanded_change(move |next| state.with_mut(|v| v.expanded = next));
                if values.expanded {
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    caption(I18N.text("ui-hig-details"), appearance);
                }
            }
            "ui-hig-toggles" => {
                toggle(values.on)
                    .mixed(values.mixed)
                    .label(I18N.text("ui-hig-select-all"))
                    .on_value_change(move |next| {
                        state.with_mut(|v| {
                            v.on = next;
                            v.mixed = false;
                        })
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                toggle(values.on)
                    .label(I18N.text("ui-hig-option"))
                    .style(ToggleStyle::Switch)
                    .on_value_change(move |next| {
                        state.with_mut(|v| {
                            v.on = next;
                            v.mixed = false;
                        })
                    });
            }
            "ui-hig-menus" => {
                menu(I18N.text("ui-hig-sort")).content(move || {
                    menu_section()
                        .header(I18N.text("ui-hig-sort-by"))
                        .content(move || {
                            menu_item(I18N.text("ui-hig-first"))
                                .checked(values.selected == 0)
                                .on_click(move || state.with_mut(|v| v.selected = 0));
                            menu_item(I18N.text("ui-hig-second"))
                                .checked(values.selected == 1)
                                .on_click(move || state.with_mut(|v| v.selected = 1));
                        });
                    menu_divider();
                    menu_item(I18N.text("ui-hig-action"))
                        .shortcut("⌘R")
                        .on_click(move || state.with_mut(|v| v.clicks += 1));
                });
                spacer().modifier(Modifier::new().height(Dp(12.0)));
                context_menu()
                    .child(move || caption(I18N.text("ui-hig-context-hint"), appearance))
                    .menu(move || {
                        menu_item(I18N.text("ui-hig-action"))
                            .on_click(move || state.with_mut(|v| v.clicks += 1));
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(values.clicks.to_string(), appearance);
            }
            "ui-hig-control-groups" => {
                control_group()
                    .label(I18N.text("ui-hig-navigation"))
                    .content(move || {
                        button(I18N.text("ui-hig-back")).on_click(move || {
                            state.with_mut(|v| v.clicks = v.clicks.saturating_sub(1))
                        });
                        button(I18N.text("ui-hig-forward"))
                            .on_click(move || state.with_mut(|v| v.clicks += 1));
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(values.clicks.to_string(), appearance);
            }
            "ui-hig-pickers" => {
                let options = vec![I18N.text("ui-hig-first"), I18N.text("ui-hig-second")];
                picker(values.selected, options.clone())
                    .label(I18N.text("ui-hig-choice"))
                    .style(PickerStyle::RadioGroup)
                    .on_select(move |next| state.with_mut(|v| v.selected = next));
                spacer().modifier(Modifier::new().height(Dp(12.0)));
                picker(values.selected, options)
                    .accessibility_label(I18N.text("ui-hig-view"))
                    .on_select(move |next| state.with_mut(|v| v.selected = next));
            }
            "ui-sliders" => {
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            "ui-hig-steppers" => {
                stepper(values.count)
                    .range(0.0..=10.0)
                    .accessibility_label(I18N.text("ui-hig-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.count = next));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(format!("{:.0}", values.count), appearance);
            }
            "ui-progress-indicators" => {
                progress_view()
                    .value(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"));
                spacer().modifier(Modifier::new().height(Dp(16.0)));
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-progress-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            "ui-hig-gauges" => {
                gauge(values.fraction)
                    .label(I18N.text("ui-hig-storage"))
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"));
                spacer().modifier(Modifier::new().height(Dp(16.0)));
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-capacity-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            "ui-hig-dividers" => {
                text(I18N.text("ui-hig-first"));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                divider().modifier(Modifier::new().width(Dp(160.0)));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                text(I18N.text("ui-hig-second"));
            }
            "ui-hig-group-boxes" => {
                group_box()
                    .title(I18N.text("ui-hig-group-boxes"))
                    .modifier(Modifier::new().width(Dp(200.0)))
                    .child(|| {
                        text(I18N.text("ui-hig-details"));
                    });
            }
            "ui-hig-glass" => {
                glass()
                    .material(GlassMaterial::Clear)
                    .appearance(glass_appearance(appearance))
                    .padding(Dp(20.0))
                    .child(|| {
                        text(I18N.text("ui-hig-glass"));
                    });
            }
            "ui-hig-text" => {
                text(I18N.text("ui-hig-text")).style(TextStyle {
                    font_size: Dp(22.0),
                    line_height: Dp(28.0),
                    font_weight: 700,
                    ..Default::default()
                });
                caption(I18N.text("ui-hig-details"), appearance);
            }
            "ui-hig-text-fields" => {
                text_field()
                    .placeholder(I18N.text("ui-hig-full-name"))
                    .modifier(Modifier::new().width(Dp(200.0)))
                    .accessibility_label(I18N.text("ui-hig-full-name"));
                spacer().modifier(Modifier::new().height(Dp(10.0)));
                secure_field()
                    .placeholder(I18N.text("ui-hig-password"))
                    .modifier(Modifier::new().width(Dp(200.0)));
                spacer().modifier(Modifier::new().height(Dp(10.0)));
                search_field()
                    .placeholder(I18N.text("ui-hig-search"))
                    .modifier(Modifier::new().width(Dp(200.0)));
                spacer().modifier(Modifier::new().height(Dp(10.0)));
                text_editor()
                    .accessibility_label(I18N.text("ui-hig-notes"))
                    .modifier(Modifier::new().size(Dp(200.0), Dp(80.0)));
            }
            "ui-hig-scroll-views" => {
                scroll_view()
                    .modifier(Modifier::new().size(Dp(240.0), Dp(140.0)))
                    .child(move || {
                        for line in 1..=30 {
                            text(format!("{} {line}", I18N.text("ui-hig-row")));
                        }
                    });
            }
            "ui-hig-lists" => {
                list()
                    .selection(values.selected)
                    .on_select(move |tag| state.with_mut(|v| v.selected = tag))
                    .alternating_row_backgrounds(true)
                    .modifier(Modifier::new().size(Dp(240.0), Dp(160.0)))
                    .content(move || {
                        list_section()
                            .header(I18N.text("ui-hig-favorites"))
                            .content(move || {
                                for tag in 0..5 {
                                    list_row(tag).content(move || {
                                        text(format!("{} {}", I18N.text("ui-hig-row"), tag + 1));
                                    });
                                }
                            });
                    });
            }
            "ui-hig-disclosure-groups" => {
                disclosure_group(I18N.text("ui-hig-details")).content(move || {
                    caption(I18N.text("ui-hig-help-content"), appearance);
                });
                spacer().modifier(Modifier::new().height(Dp(12.0)));
                outline_group(vec![
                    OutlineItem::new("Documents").with_children(vec![
                        OutlineItem::new("Reports").with_children(vec![OutlineItem::new("Q3.pdf")]),
                        OutlineItem::new("Notes.txt"),
                    ]),
                    OutlineItem::new("Downloads"),
                ]);
            }
            "ui-hig-tables" => {
                let ascending = values.on;
                let mut rows = vec![
                    ("Report.pdf", "2 MB"),
                    ("Notes.txt", "4 KB"),
                    ("Photo.jpg", "3 MB"),
                ];
                rows.sort_by_key(|(name, _)| *name);
                if !ascending {
                    rows.reverse();
                }
                table(
                    vec![
                        TableColumn::new(I18N.text("ui-hig-name")),
                        TableColumn::new(I18N.text("ui-hig-size")).width(Dp(80.0)),
                    ],
                    rows.len(),
                )
                .sort(TableSort {
                    column: 0,
                    ascending,
                })
                .on_sort(move |sort: TableSort| state.with_mut(|v| v.on = sort.ascending))
                .selection(values.selected)
                .on_select(move |row| state.with_mut(|v| v.selected = row))
                .modifier(Modifier::new().size(Dp(300.0), Dp(140.0)))
                .cell(move |(row, column)| {
                    let (name, size) = rows[row];
                    text(if column == 0 { name } else { size });
                });
            }
            "ui-hig-grids" => {
                grid(vec![GridItem::Adaptive { min: Dp(64.0) }])
                    .modifier(Modifier::new().width(Dp(300.0)))
                    .content(move || {
                        for index in 1..=8 {
                            text(format!("{} {index}", I18N.text("ui-hig-tile")));
                        }
                    });
            }
            "ui-hig-split-views" => {
                h_split_view()
                    .leading_width(Dp(120.0))
                    .modifier(Modifier::new().size(Dp(360.0), Dp(180.0)))
                    .leading(move || caption(I18N.text("ui-hig-favorites"), appearance))
                    .trailing(move || {
                        v_split_view()
                            .top_height(Dp(80.0))
                            .top(move || caption(I18N.text("ui-hig-row"), appearance))
                            .bottom(move || caption(I18N.text("ui-hig-details"), appearance));
                    });
            }
            "ui-hig-navigation-split-views" => {
                navigation_split_view()
                    .column_visibility(if values.on {
                        NavigationSplitViewVisibility::DetailOnly
                    } else {
                        NavigationSplitViewVisibility::All
                    })
                    .sidebar_width(Dp(150.0))
                    .modifier(Modifier::new().size(Dp(420.0), Dp(200.0)))
                    .sidebar(move || {
                        list()
                            .style(ListStyle::Sidebar)
                            .selection(values.selected)
                            .on_select(move |tag| state.with_mut(|v| v.selected = tag))
                            .content(move || {
                                for tag in 0..3 {
                                    list_row(tag).content(move || {
                                        text(format!("{} {}", I18N.text("ui-hig-row"), tag + 1));
                                    });
                                }
                            });
                    })
                    .detail(move || {
                        caption(
                            format!("{} {}", I18N.text("ui-hig-row"), values.selected + 1),
                            appearance,
                        );
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                toggle(values.on)
                    .label(I18N.text("ui-hig-hide-sidebar"))
                    .on_value_change(move |on| state.with_mut(|v| v.on = on));
            }
            "ui-hig-tab-views" => {
                tab_view(
                    values.selected.min(1),
                    vec![I18N.text("ui-hig-general"), I18N.text("ui-hig-advanced")],
                )
                .on_select(move |index| state.with_mut(|v| v.selected = index))
                .modifier(Modifier::new().width(Dp(320.0)))
                .content(move |index| {
                    caption(
                        I18N.text(if index == 0 {
                            "ui-hig-general"
                        } else {
                            "ui-hig-advanced"
                        }),
                        appearance,
                    );
                });
            }
            "ui-hig-forms" => {
                form()
                    .style(if values.on {
                        FormStyle::Grouped
                    } else {
                        FormStyle::Columns
                    })
                    .modifier(Modifier::new().width(Dp(360.0)))
                    .content(move || {
                        form_section()
                            .header(I18N.text("ui-hig-general"))
                            .content(move || {
                                labeled_content(I18N.text("ui-hig-full-name")).content(|| {
                                    text_field().modifier(Modifier::new().width(Dp(160.0)));
                                });
                                labeled_content(I18N.text("ui-hig-style-grouped")).content(
                                    move || {
                                        toggle(values.on)
                                            .style(ToggleStyle::Switch)
                                            .on_value_change(move |on| {
                                                state.with_mut(|v| v.on = on)
                                            });
                                    },
                                );
                            });
                    });
            }
            "ui-hig-date-pickers" => {
                let day = CalendarDate::new(2026, 11, 1 + (values.count as u8 % 28)).unwrap_or(
                    CalendarDate {
                        year: 2026,
                        month: 11,
                        day: 1,
                    },
                );
                date_picker(day)
                    .style(DatePickerStyle::Graphical)
                    .on_change(move |date: CalendarDate| {
                        state.with_mut(|v| v.count = f64::from(date.day - 1))
                    });
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                date_picker(day).on_change(move |date: CalendarDate| {
                    state.with_mut(|v| v.count = f64::from(date.day - 1))
                });
            }
            "ui-hig-color-pickers" => {
                color_picker(if values.on {
                    Color::from_rgb(0.0, 0.2, 1.0)
                } else {
                    Color::from_rgb(1.0, 0.6, 0.0)
                })
                .label(I18N.text("ui-hig-highlight"))
                .on_change(move |_| state.with_mut(|v| v.on = !v.on));
            }
            "ui-hig-content-unavailable" => {
                content_unavailable_view(I18N.text("ui-hig-no-results"))
                    .description(I18N.text("ui-hig-no-results-description"))
                    .modifier(Modifier::new().size(Dp(320.0), Dp(160.0)))
                    .actions(|| {
                        button(I18N.text("ui-hig-clear-search"));
                    });
            }
            "ui-hig-labels" => {
                hig_label(I18N.text("ui-hig-favorites")).icon(|| {
                    text("★");
                });
            }
            _ => unreachable!(),
        });
}

#[tessera]
fn caption(content: String, appearance: Appearance) {
    text(content).color(label_colors(appearance)[1]);
}

#[tessera]
fn static_state(index: usize, section: usize, row: usize, column: usize, appearance: Appearance) {
    let enabled = row == 0;
    let name = label(COMPONENTS[index].sections[section].columns[column]);
    match (COMPONENTS[index].title, section) {
        ("ui-buttons", 1) => {
            button(I18N.text("ui-hig-action"))
                .style(BUTTON_STYLE_VALUES[row])
                .enabled(column == 0)
                .accessibility_label(name);
        }
        ("ui-buttons", 2) => {
            let role = [
                None,
                Some(ButtonRole::Cancel),
                Some(ButtonRole::Destructive),
            ][column];
            button(I18N.text("ui-hig-action"))
                .style([ButtonStyle::Bordered, ButtonStyle::BorderedProminent][row])
                .role_optional(role)
                .accessibility_label(name);
        }
        ("ui-buttons", _) => {
            button(I18N.text("ui-hig-action"))
                .control_size(
                    [
                        ControlSize::Mini,
                        ControlSize::Small,
                        ControlSize::Regular,
                        ControlSize::Large,
                        ControlSize::ExtraLarge,
                    ][column],
                )
                .accessibility_label(name);
        }
        ("ui-hig-links", _) => {
            link(
                I18N.text("ui-hig-link-title"),
                "https://example.com/privacy",
            )
            .enabled(column == 0)
            .accessibility_label(name);
        }
        ("ui-hig-help-links", _) => {
            help_link().enabled(column == 0).accessibility_label(name);
        }
        ("ui-hig-disclosure-buttons", _) => {
            disclosure_button(column == 1)
                .enabled(enabled)
                .accessibility_label(name);
        }
        ("ui-hig-toggles", _) => {
            toggle(column == 1)
                .mixed(column == 2)
                .label(I18N.text("ui-hig-option"))
                .style(
                    [
                        ToggleStyle::Checkbox,
                        ToggleStyle::Switch,
                        ToggleStyle::Button,
                    ][row],
                )
                .enabled(section == 1)
                .accessibility_label(name);
        }
        ("ui-hig-menus", _) => {
            menu(I18N.text("ui-hig-sort"))
                .style([MenuStyle::BorderedButton, MenuStyle::BorderlessButton][row])
                .enabled(column == 0)
                .accessibility_label(name)
                .content(|| {
                    menu_item(I18N.text("ui-hig-first"));
                });
        }
        ("ui-hig-control-groups", _) => {
            control_group()
                .label(I18N.text("ui-hig-navigation"))
                .style([ControlGroupStyle::Automatic, ControlGroupStyle::Menu][column])
                .content(|| {
                    button(I18N.text("ui-hig-back"));
                    button(I18N.text("ui-hig-forward"));
                });
        }
        ("ui-hig-pickers", _) => {
            picker(
                0,
                vec![I18N.text("ui-hig-first"), I18N.text("ui-hig-second")],
            )
            .style(
                [
                    PickerStyle::PopUpButton,
                    PickerStyle::Segmented,
                    PickerStyle::RadioGroup,
                ][column],
            )
            .enabled(enabled)
            .accessibility_label(name);
        }
        ("ui-sliders", _) => {
            slider(column as f64 * 0.5)
                .enabled(enabled)
                .modifier(Modifier::new().width(Dp(96.0)))
                .accessibility_label(name);
        }
        ("ui-hig-steppers", _) => {
            stepper(5.0)
                .range(0.0..=10.0)
                .enabled(column == 0)
                .accessibility_label(name);
        }
        ("ui-progress-indicators", 1) => {
            progress_view()
                .value(column as f64 * 0.5)
                .modifier(Modifier::new().width(Dp(96.0)))
                .accessibility_label(name);
        }
        ("ui-progress-indicators", _) => {
            if column == 0 {
                progress_view()
                    .style(ProgressViewStyle::Linear)
                    .modifier(Modifier::new().width(Dp(96.0)))
                    .accessibility_label(name);
            } else {
                progress_view().accessibility_label(name);
            }
        }
        ("ui-hig-gauges", 1) => {
            gauge(0.4)
                .label(I18N.text("ui-hig-storage"))
                .style(
                    [
                        GaugeStyle::LinearCapacity,
                        GaugeStyle::AccessoryLinearCapacity,
                        GaugeStyle::AccessoryLinear,
                    ][column],
                )
                .modifier(Modifier::new().width(Dp(112.0)))
                .accessibility_label(name);
        }
        ("ui-hig-gauges", _) => {
            gauge(0.4)
                .label("L")
                .style(
                    [
                        GaugeStyle::AccessoryCircular,
                        GaugeStyle::AccessoryCircularCapacity,
                    ][column],
                )
                .accessibility_label(name);
        }
        ("ui-hig-dividers", _) => {
            if column == 0 {
                divider().modifier(Modifier::new().width(Dp(96.0)));
            } else {
                divider()
                    .orientation(DividerOrientation::Vertical)
                    .modifier(Modifier::new().height(Dp(48.0)));
            }
        }
        ("ui-hig-group-boxes", _) => {
            group_box()
                .title(if column == 0 {
                    I18N.text("ui-hig-group-boxes")
                } else {
                    String::new()
                })
                .modifier(Modifier::new().width(Dp(140.0)))
                .child(|| {
                    text(I18N.text("ui-hig-details"));
                });
        }
        ("ui-hig-glass", 1) => {
            let rounded = column == 1;
            glass()
                .material(if row == 0 {
                    GlassMaterial::Regular
                } else {
                    GlassMaterial::Clear
                })
                .appearance(glass_appearance(appearance))
                .corner_radius_optional(rounded.then_some(Dp(12.0)))
                .modifier(Modifier::new().size(Dp(96.0), Dp(if rounded { 64.0 } else { 40.0 })));
        }
        ("ui-hig-glass", _) => {
            glass_group()
                .material(if column == 0 {
                    GlassMaterial::Regular
                } else {
                    GlassMaterial::Clear
                })
                .appearance(glass_appearance(appearance))
                .spacing(Dp(16.0))
                .gap(Dp(6.0))
                .child(|| {
                    spacer().modifier(Modifier::new().size(Dp(44.0), Dp(36.0)));
                    spacer().modifier(Modifier::new().size(Dp(44.0), Dp(36.0)));
                });
        }
        ("ui-hig-text", _) => {
            text(I18N.text("ui-hig-text")).color(label_colors(appearance)[column]);
        }
        ("ui-hig-text-fields", 2) => {
            text_field()
                .placeholder(I18N.text("ui-hig-search"))
                .style(
                    [
                        TextFieldStyle::RoundedBorder,
                        TextFieldStyle::SquareBorder,
                        TextFieldStyle::Plain,
                    ][column],
                )
                .modifier(Modifier::new().width(Dp(120.0)))
                .accessibility_label(name);
        }
        ("ui-hig-lists", _) => {
            list()
                .style([ListStyle::Inset, ListStyle::Plain, ListStyle::Sidebar][column])
                .selection(0)
                .modifier(Modifier::new().size(Dp(120.0), Dp(100.0)))
                .content(|| {
                    for tag in 0..3 {
                        list_row(tag).content(move || {
                            text(format!("{}", tag + 1));
                        });
                    }
                });
        }
        ("ui-hig-date-pickers", _) => {
            date_picker(CalendarDate {
                year: 2026,
                month: 11,
                day: 21,
            })
            .style(
                [
                    DatePickerStyle::Compact,
                    DatePickerStyle::Field,
                    DatePickerStyle::StepperField,
                ][column],
            )
            .accessibility_label(name);
        }
        ("ui-hig-labels", _) => {
            hig_label(I18N.text("ui-hig-favorites"))
                .style(
                    [
                        LabelStyle::TitleAndIcon,
                        LabelStyle::IconOnly,
                        LabelStyle::TitleOnly,
                    ][column],
                )
                .icon(|| {
                    text("★");
                });
        }
        ("ui-hig-text-fields", _) => {
            let filled = column == 1;
            text_field()
                .placeholder(I18N.text("ui-hig-search"))
                .initial_text(if filled { "Tessera" } else { "" })
                .enabled(enabled)
                .modifier(Modifier::new().width(Dp(120.0)))
                .accessibility_label(name);
        }
        _ => unreachable!(),
    }
}
