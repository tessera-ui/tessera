use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    experiment::hig::{
        button::button,
        checkbox::{CheckboxState, checkbox},
        disclosure_button::disclosure_button,
        divider::{DividerOrientation, divider},
        glass::{GlassAppearance, GlassMaterial, glass},
        glass_group::glass_group,
        group_box::group_box,
        help_button::help_button,
        level_indicator::capacity_indicator,
        progress::{indeterminate_progress_bar, progress_bar, progress_spinner},
        radio_button::{radio_button, radio_group},
        segmented_control::{Segment, segmented_control},
        slider::slider,
        stepper::stepper,
        switch::switch,
        text::text,
        text_field::text_field,
        theme::{Appearance, HigTheme, TextStyle},
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

pub(super) const CATEGORIES: [&str; 4] = [
    "ui-hig-actions",
    "ui-hig-selection",
    "ui-hig-values",
    "ui-hig-content",
];

const SINGLE: &[&str] = &[""];
const ENABLED: &[&str] = &["ui-hig-enabled", "ui-hig-disabled"];

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
                rows: &["ui-hig-push", "ui-hig-default"],
                height: 64.0,
            },
        ],
    },
    Component {
        title: "ui-hig-help-buttons",
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
        title: "ui-checkbox",
        description: "ui-hig-checkbox-description",
        category: 1,
        sections: &[
            example(120.0),
            states(&["ui-hig-off", "ui-hig-on", "ui-hig-mixed"], ENABLED),
        ],
    },
    Component {
        title: "ui-radio-button",
        description: "ui-hig-radio-description",
        category: 1,
        sections: &[
            example(120.0),
            states(&["ui-hig-unselected", "ui-hig-selected"], ENABLED),
        ],
    },
    Component {
        title: "ui-switch",
        description: "ui-hig-switch-description",
        category: 1,
        sections: &[
            example(120.0),
            states(&["ui-hig-off", "ui-hig-on"], ENABLED),
        ],
    },
    Component {
        title: "ui-hig-segmented-controls",
        description: "ui-hig-segments-description",
        category: 1,
        sections: &[example(120.0), states(ENABLED, SINGLE)],
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
        title: "ui-hig-capacity-indicators",
        description: "ui-hig-capacity-description",
        category: 2,
        sections: &[
            example(120.0),
            states(
                &["ui-hig-normal", "ui-hig-warning", "ui-hig-critical"],
                ENABLED,
            ),
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
            example(120.0),
            states(&["ui-hig-empty", "ui-hig-filled"], ENABLED),
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
    checked: CheckboxState,
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
            checked: CheckboxState::Off,
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
    let backdrop = index == 13;
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
        .children(move || match index {
            0 => {
                button(I18N.text("ui-hig-action"))
                    .default_action(true)
                    .on_click(move || state.with_mut(|v| v.clicks += 1));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(values.clicks.to_string(), appearance);
            }
            1 => {
                help_button()
                    .accessibility_label(I18N.text("ui-hig-help-buttons"))
                    .on_click(move || state.with_mut(|v| v.expanded = !v.expanded));
                if values.expanded {
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    caption(I18N.text("ui-hig-help-content"), appearance);
                }
            }
            2 => {
                disclosure_button(values.expanded)
                    .accessibility_label(I18N.text("ui-hig-details"))
                    .on_expanded_change(move |next| state.with_mut(|v| v.expanded = next));
                if values.expanded {
                    spacer().modifier(Modifier::new().height(Dp(8.0)));
                    caption(I18N.text("ui-hig-details"), appearance);
                }
            }
            3 => {
                checkbox(values.checked)
                    .allows_mixed(true)
                    .label(I18N.text("ui-hig-option"))
                    .on_value_change(move |next| state.with_mut(|v| v.checked = next));
            }
            4 => {
                radio_group()
                    .accessibility_label(I18N.text("ui-hig-choice"))
                    .child(move |group| {
                        column().children(move || {
                            for (option, key) in
                                ["ui-hig-first", "ui-hig-second"].into_iter().enumerate()
                            {
                                radio_button(values.selected == option)
                                    .group(group)
                                    .label(I18N.text(key))
                                    .on_select(move || state.with_mut(|v| v.selected = option));
                            }
                        });
                    });
            }
            5 => {
                switch(values.on)
                    .accessibility_label(I18N.text("ui-hig-option"))
                    .on_value_change(move |next| state.with_mut(|v| v.on = next));
            }
            6 => {
                segmented_control(vec![
                    Segment::new(I18N.text("ui-hig-first")),
                    Segment::new(I18N.text("ui-hig-second")),
                ])
                .selected(values.selected)
                .on_select(move |next| state.with_mut(|v| v.selected = next));
            }
            7 => {
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            8 => {
                stepper(values.count)
                    .range(0.0..=10.0)
                    .accessibility_label(I18N.text("ui-hig-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.count = next));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                caption(format!("{:.0}", values.count), appearance);
            }
            9 => {
                progress_bar(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"));
                spacer().modifier(Modifier::new().height(Dp(16.0)));
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-progress-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            10 => {
                capacity_indicator(values.fraction * 5.0)
                    .warning_value(3.0)
                    .critical_value(4.0)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-value"));
                spacer().modifier(Modifier::new().height(Dp(16.0)));
                slider(values.fraction)
                    .modifier(Modifier::new().width(Dp(160.0)))
                    .accessibility_label(I18N.text("ui-hig-capacity-value"))
                    .on_value_change(move |next| state.with_mut(|v| v.fraction = next));
            }
            11 => {
                text(I18N.text("ui-hig-first"));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                divider().modifier(Modifier::new().width(Dp(160.0)));
                spacer().modifier(Modifier::new().height(Dp(8.0)));
                text(I18N.text("ui-hig-second"));
            }
            12 => {
                group_box()
                    .title(I18N.text("ui-hig-group-boxes"))
                    .modifier(Modifier::new().width(Dp(200.0)))
                    .child(|| {
                        text(I18N.text("ui-hig-details"));
                    });
            }
            13 => {
                glass()
                    .material(GlassMaterial::Clear)
                    .appearance(glass_appearance(appearance))
                    .padding(Dp(20.0))
                    .child(|| {
                        text(I18N.text("ui-hig-glass"));
                    });
            }
            14 => {
                text(I18N.text("ui-hig-text")).style(TextStyle {
                    font_size: Dp(22.0),
                    line_height: Dp(28.0),
                    font_weight: 700,
                    ..Default::default()
                });
                caption(I18N.text("ui-hig-details"), appearance);
            }
            15 => {
                text_field()
                    .placeholder(I18N.text("ui-hig-search"))
                    .modifier(Modifier::new().width(Dp(200.0)))
                    .accessibility_label(I18N.text("ui-hig-search"));
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
    match (index, section) {
        (0, _) => {
            button(I18N.text("ui-hig-action"))
                .enabled(column == 0)
                .default_action(row == 1)
                .accessibility_label(name);
        }
        (1, _) => {
            help_button().enabled(column == 0).accessibility_label(name);
        }
        (2, _) => {
            disclosure_button(column == 1)
                .enabled(enabled)
                .accessibility_label(name);
        }
        (3, _) => {
            checkbox([CheckboxState::Off, CheckboxState::On, CheckboxState::Mixed][column])
                .allows_mixed(true)
                .enabled(enabled)
                .accessibility_label(name);
        }
        (4, _) => {
            radio_button(column == 1)
                .enabled(enabled)
                .accessibility_label(name);
        }
        (5, _) => {
            switch(column == 1)
                .enabled(enabled)
                .accessibility_label(name);
        }
        (6, _) => {
            segmented_control(vec![
                Segment::new(I18N.text("ui-hig-first")),
                Segment::new(I18N.text("ui-hig-second")),
            ])
            .selected(0)
            .enabled(column == 0)
            .accessibility_label(name);
        }
        (7, _) => {
            slider(column as f64 * 0.5)
                .enabled(enabled)
                .modifier(Modifier::new().width(Dp(96.0)))
                .accessibility_label(name);
        }
        (8, _) => {
            stepper(5.0)
                .range(0.0..=10.0)
                .enabled(column == 0)
                .accessibility_label(name);
        }
        (9, 1) => {
            progress_bar(column as f64 * 0.5)
                .modifier(Modifier::new().width(Dp(96.0)))
                .accessibility_label(name);
        }
        (9, _) => {
            if column == 0 {
                indeterminate_progress_bar(true)
                    .modifier(Modifier::new().width(Dp(96.0)))
                    .accessibility_label(name);
            } else {
                progress_spinner(true).accessibility_label(name);
            }
        }
        (10, _) => {
            capacity_indicator([2.0, 3.5, 4.5][column])
                .warning_value(3.0)
                .critical_value(4.0)
                .enabled(enabled)
                .modifier(Modifier::new().width(Dp(96.0)))
                .accessibility_label(name);
        }
        (11, _) => {
            if column == 0 {
                divider().modifier(Modifier::new().width(Dp(96.0)));
            } else {
                divider()
                    .orientation(DividerOrientation::Vertical)
                    .modifier(Modifier::new().height(Dp(48.0)));
            }
        }
        (12, _) => {
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
        (13, 1) => {
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
        (13, _) => {
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
        (14, _) => {
            text(I18N.text("ui-hig-text")).color(label_colors(appearance)[column]);
        }
        (15, _) => {
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
