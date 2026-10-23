//! Live slider illustrations for specification reading.
//!
//! ## Usage
//!
//! Adjust slider variants and compare their sizes, colors and enabled states.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::ModifierExt as _,
    painter::Painter,
    row::row,
    shape_def::Shape,
    slider::{
        SliderController, SliderOrientation, SliderSize, SliderValueIndicatorVisibility,
        centered_slider, range_slider, slider, slider_value_indicator_provider,
    },
    surface::surface,
    text::text,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};
use tessera_ui::{Color, Dp, Modifier, RenderSlot, State, remember, tessera, use_context};

use super::{
    ACCENT, INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, srgb, subheading, type_style,
};

const SIZES: [SliderSize; 5] = [
    SliderSize::ExtraSmall,
    SliderSize::Small,
    SliderSize::Medium,
    SliderSize::Large,
    SliderSize::ExtraLarge,
];

#[tessera]
pub(super) fn slider_spec() {
    article("Sliders","Sliders allow users to make selections from a range of values",vec![
        section("Variants",RenderSlot::new(variants)),
        section("Configurations",RenderSlot::new(configurations)),
        section("Tokens & specs",RenderSlot::new(tokens)),
        section("Anatomy",RenderSlot::new(anatomy)),
        section("Color",RenderSlot::new(colors)),
        section("States",RenderSlot::new(states)),
        section("Measurements",RenderSlot::new(measurements)),
    ]).hero_shared(RenderSlot::new(hero)).pages(vec![
        ArticlePage{title:"Overview",intro:Some(RenderSlot::new(||{
            prose("Sliders select a value or range along a track. Use standard, centered, or range variants, show the full available range, and apply changes immediately.");gap(Dp(32.0));variants();
        })),sections:vec![
            section("M3 Expressive update",RenderSlot::new(||{prose("The May 2025 update adds horizontal and vertical orientations, five sizes from XS to XL, and an optional inset icon for standard sliders. Continuous sliders are now called standard sliders; discrete sliders use the stops configuration.");gap(Dp(24.0));configurations();})),
            section("Previous updates",RenderSlot::new(||{subheading("Visual refresh to improve non-text contrast".into());prose("The December 2023 refresh adds centered and range configurations, refreshed color mappings, end stops, larger value labels, and a vertical handle that narrows when pressed.");gap(Dp(24.0));anatomy();})),
            section("Differences from M2",RenderSlot::new(||{prose("Material 3 replaces the circular handle with a vertical handle and introduces dynamic color roles. End stops keep the extent of the inactive track distinguishable.");gap(Dp(24.0));colors();})),
        ]},
        ArticlePage{title:"Guidelines",intro:None,sections:vec![
            section("Usage",RenderSlot::new(||{prose("Use sliders to adjust continuous settings such as volume, brightness, or filter intensity. Apply every value change immediately so the result stays connected to the handle movement.");gap(Dp(24.0));immediate_example();gap(Dp(24.0));prose("Standard sliders start at a minimum; centered sliders place zero in the middle; range sliders choose a minimum and maximum. Prefer horizontal range sliders to reduce cognitive effort.");gap(Dp(24.0));variants();})),
            section("Anatomy",RenderSlot::new(||{anatomy();gap(Dp(24.0));subheading("Track".into());prose("The track represents the full available range. The active section runs from the minimum to the handle, or between the two handles of a range slider. Values increase toward the trailing edge in the reading direction.");gap(Dp(24.0));subheading("Handle".into());prose("Move the handle to select a value. A range slider has separate minimum and maximum handles. Each handle narrows during interaction.");})),
            section("Configurations",RenderSlot::new(||{
                subheading("Value".into());prose("Show a value label for the handle being adjusted. If a separate field shows the value, the built-in label is optional. Keep an editable field synchronized with the slider and next in the tab order.");gap(Dp(24.0));value_example();
                gap(Dp(32.0));subheading("Stop indicators".into());prose("Stops represent equally spaced available values, and the handle snaps to the closest stop. Keep the track readable by avoiding excessive stops. End stops may be omitted only when another visual anchor gives at least 3:1 contrast.");gap(Dp(24.0));sample("Stops".into(),0,SliderSize::ExtraSmall,false,9);
                gap(Dp(32.0));subheading("Orientation".into());prose("Use horizontal or vertical standard and centered sliders to fit the task. Vertical values increase from the bottom.");gap(Dp(24.0));orientation_examples();
                gap(Dp(32.0));subheading("Inset icon".into());prose("Only standard sliders in M, L, or XL sizes support an inset icon. At low values it moves to the inactive track. Do not add inset icons to centered or range sliders.");gap(Dp(24.0));inset_examples();
                gap(Dp(32.0));subheading("Size".into());prose("Track heights are XS 16dp, S 24dp, M 40dp, L 56dp, and XL 96dp. Active and inactive tracks use the same size. Reserve XL for a page where the slider is the primary task.");gap(Dp(24.0));measurements();
            })),
            section("Behaviors",RenderSlot::new(||{
                subheading("Select & drag".into());prose("Drag the handle to update a continuous value or snap between stops.");gap(Dp(24.0));sample("Drag a continuous value".into(),0,SliderSize::Medium,false,0);
                gap(Dp(32.0));subheading("Select jump".into());prose("Select the track to move the handle to that position, or the nearest stop.");gap(Dp(24.0));sample("Jump between stops".into(),0,SliderSize::Medium,false,9);
                gap(Dp(32.0));subheading("Select & arrow".into());prose("Tab to a handle and use arrow keys to adjust it. Space with arrows moves a larger interval.");gap(Dp(24.0));value_example();
            })),
        ]},
        ArticlePage{title:"Accessibility",intro:None,sections:vec![
            section("Use cases",RenderSlot::new(||{prose("Assistive technology must identify each slider, adjust its handle along the range, and report the selected value. Range sliders expose both handles separately.");gap(Dp(24.0));sample("Price range".into(),2,SliderSize::Medium,false,0);})),
            section("Interaction & style",RenderSlot::new(||{prose("The handle narrows and the value appears during touch, pointer, or keyboard interaction.");gap(Dp(24.0));states();gap(Dp(24.0));subheading("Focus and navigation".into());prose("Initial focus lands on the handle. Arrow keys adjust its value without requiring pointer precision.");gap(Dp(24.0));value_example();})),
            section("Color contrast",RenderSlot::new(||{prose("The end of the inactive track needs a visual anchor with at least 3:1 contrast against the background. Use an end stop or a contrasting icon or label at the endpoint.");gap(Dp(24.0));colors();})),
            section("Keyboard navigation",RenderSlot::new(||{spec_table(vec!["Keys".into(),"Actions".into()],vec![vec!["Tab".into(),"Focus a slider handle".into()],vec!["Arrows".into(),"Change by one value or stop".into()],vec!["Space & Arrows".into(),"Change by a larger interval".into()],vec!["Home or End".into(),"Select the first or last value".into()]]);gap(Dp(24.0));value_example();})),
            section("Labeling elements",RenderSlot::new(||{prose("The accessibility label should match the adjacent visible label and use the slider role. Label external icon controls as buttons. Include units when they clarify the value.");gap(Dp(24.0));sample("Volume".into(),0,SliderSize::Medium,false,0).leading_icon(filled::VOLUME_UP_SVG);})),
        ]},
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn prose(#[prop(into)] content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn immediate_example() {
    let value = remember(|| 0.5_f32);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            text().content("Volume").style(type_style(16.0, 24.0));
            gap(Dp(16.0));
            slider()
                .size(SliderSize::Medium)
                .modifier(Modifier::new().fill_max_width())
                .value(value.get())
                .on_change(move |v| value.set(v))
                .inset_icon(if value.get() == 0.0 {
                    filled::VOLUME_OFF_SVG
                } else {
                    filled::VOLUME_UP_SVG
                })
                .accessibility_label("Volume");
            gap(Dp(16.0));
            text()
                .content(format!("Volume: {:.0}%", value.get() * 100.0))
                .style(type_style(16.0, 24.0));
        });
}

#[tessera]
fn value_example() {
    let value = remember(|| 0.5_f32);
    slider_value_indicator_provider().child(move || {
        slider()
            .modifier(Modifier::new().fill_max_width())
            .value(value.get())
            .on_change(move |v| value.set(v))
            .value_indicator(SliderValueIndicatorVisibility::OnInteraction)
            .accessibility_label("Keyboard-adjustable value");
    });
    gap(Dp(16.0));
    text()
        .content(format!("Value: {:.0}", value.get() * 100.0))
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn orientation_examples() {
    let standard = remember(|| 0.5_f32);
    let centered = remember(|| 0.25_f32);
    flow_row()
        .item_spacing(Dp(64.0))
        .line_spacing(Dp(24.0))
        .children(move || {
            slider()
                .orientation(SliderOrientation::Vertical)
                .modifier(Modifier::new().height(Dp(240.0)))
                .value(standard.get())
                .on_change(move |v| standard.set(v))
                .accessibility_label("Vertical standard value");
            centered_slider()
                .orientation(SliderOrientation::Vertical)
                .modifier(Modifier::new().height(Dp(240.0)))
                .value(centered.get())
                .on_change(move |v| centered.set(v))
                .accessibility_label("Vertical centered value");
        });
}

#[tessera]
fn inset_examples() {
    let value = remember(|| 0.5_f32);
    slider()
        .modifier(Modifier::new().fill_max_width())
        .size(SliderSize::Medium)
        .value(value.get())
        .on_change(move |v| value.set(v))
        .inset_icon(if value.get() == 0.0 {
            filled::VOLUME_OFF_SVG
        } else {
            filled::VOLUME_UP_SVG
        })
        .accessibility_label("Inset icon volume");
}

#[tessera]
fn sample(
    label: String,
    kind: usize,
    size: SliderSize,
    disabled: bool,
    steps: usize,
    initial: Option<f32>,
    #[prop(into)] leading_icon: Option<Painter>,
) {
    let value = remember(|| initial.unwrap_or(0.5_f32));
    let range = remember(|| (0.25_f32, 0.75_f32));
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            if !label.is_empty() {
                let label = label.clone();
                let leading_icon = leading_icon.clone();
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        if let Some(painter) = &leading_icon {
                            icon().painter(painter.clone()).size(Dp(24.0)).tint(INK);
                            tessera_components::spacer::spacer()
                                .modifier(Modifier::new().width(Dp(12.0)));
                        }
                        text()
                            .content(label.clone())
                            .style(if leading_icon.is_some() {
                                type_style(20.0, 24.0)
                            } else {
                                type_style(16.0, 24.0)
                            });
                    });
                gap(Dp(16.0));
            }
            match kind {
                1 => {
                    centered_slider()
                        .modifier(Modifier::new().fill_max_width())
                        .size(size)
                        .value(value.get())
                        .on_change(move |v| value.set(v))
                        .disabled(disabled)
                        .steps(steps)
                        .accessibility_label(label.clone());
                }
                2 => {
                    range_slider()
                        .modifier(Modifier::new().fill_max_width())
                        .size(size)
                        .value(range.get())
                        .on_change(move |v| range.set(v))
                        .disabled(disabled)
                        .steps(steps)
                        .accessibility_label(label.clone());
                }
                _ => {
                    slider()
                        .modifier(Modifier::new().fill_max_width())
                        .size(size)
                        .value(value.get())
                        .on_change(move |v| value.set(v))
                        .disabled(disabled)
                        .steps(steps)
                        .accessibility_label(label.clone());
                }
            }
        });
}

#[tessera]
pub(super) fn hero() {
    surface()
        .style(srgb(202, 78, 243).into())
        .shape(Shape::rounded_rectangle(Dp(24.0)))
        .modifier(Modifier::new().fill_max_width().height(Dp(432.0)))
        .child(|| {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    surface()
                        .style(srgb(253, 247, 255).into())
                        .modifier(Modifier::new().width(Dp(432.0)).fill_max_height())
                        .child(|| {
                            column()
                                .modifier(
                                    Modifier::new()
                                        .fill_max_width()
                                        .padding_symmetric(Dp(24.0), Dp(24.0)),
                                )
                                .children(|| {
                                    for (index, label) in [
                                        "Call volume",
                                        "Alarm volume",
                                        "Ring volume",
                                        "Media volume",
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    {
                                        sample(
                                            label.to_owned(),
                                            0,
                                            SliderSize::ExtraSmall,
                                            false,
                                            if index == 1 { 9 } else { 0 },
                                        )
                                        .initial(if index < 2 { 0.5 } else { 0.2 })
                                        .leading_icon(
                                            [
                                                filled::CALL_SVG,
                                                filled::ALARM_SVG,
                                                filled::NOTIFICATIONS_NONE_SVG,
                                                filled::MUSIC_NOTE_SVG,
                                            ][index],
                                        );
                                        gap(Dp(8.0));
                                    }
                                });
                        });
                });
        });
}

#[tessera]
pub(super) fn variants() {
    surface()
        .style(super::PANEL.into())
        .shape(Shape::rounded_rectangle(Dp(16.0)))
        .modifier(Modifier::new().fill_max_width().height(Dp(288.0)))
        .child(|| {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(40.0)))
                .children(|| {
                    row()
                        .modifier(Modifier::new().fill_max_width())
                        .children(|| {
                            for index in 0..3 {
                                column()
                                    .modifier(
                                        Modifier::new()
                                            .weight(1.0)
                                            .padding_symmetric(Dp(32.0), Dp::ZERO),
                                    )
                                    .children(move || {
                                        sample(
                                            String::new(),
                                            index,
                                            SliderSize::ExtraSmall,
                                            false,
                                            0,
                                        );
                                        gap(Dp(24.0));
                                        boxed()
                                            .alignment(Alignment::Center)
                                            .modifier(Modifier::new().fill_max_width())
                                            .children(move || {
                                                number(index + 1);
                                            });
                                    });
                            }
                        });
                });
        });
    gap(Dp(16.0));
    for (index, label) in ["Standard", "Centered", "Range"].into_iter().enumerate() {
        row().children(move || {
            number(index + 1);
            text()
                .content(label)
                .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(4.0)))
                .style(type_style(16.0, 24.0));
        });
        gap(Dp(12.0));
    }
    gap(Dp(80.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["Standard".into(), "Continuous".into(), "Available".into()],
            vec!["Centered".into(), "Web".into(), "Available".into()],
            vec!["Range".into(), "Available".into(), "Available".into()],
            vec![
                "Discrete".into(),
                "Available".into(),
                "Stops configuration".into(),
            ],
        ],
    );
}

#[tessera]
fn number(value: usize) {
    surface()
        .style(INK.into())
        .shape(Shape::rounded_rectangle(Dp(16.0)))
        .modifier(Modifier::new().size(Dp(32.0), Dp(32.0)))
        .child(move || {
            boxed()
                .alignment(Alignment::Center)
                .modifier(Modifier::new().fill_max_size())
                .children(move || {
                    text()
                        .content(value.to_string())
                        .color(Color::WHITE)
                        .style(type_style(12.0, 16.0));
                });
        });
}

#[tessera]
pub(super) fn configurations() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for (index, size) in SIZES.into_iter().enumerate() {
                    sample(
                        ["XS", "S", "M", "L", "XL"][index].to_owned(),
                        0,
                        size,
                        false,
                        0,
                    );
                    gap(Dp(32.0));
                }
                let value = remember(|| 0.5_f32);
                row().children(move || {
                    text().content("Vertical").style(type_style(16.0, 24.0));
                    slider()
                        .orientation(SliderOrientation::Vertical)
                        .modifier(Modifier::new().height(Dp(240.0)))
                        .value(value.get())
                        .on_change(move |v| value.set(v))
                        .accessibility_label("Vertical slider");
                });
                sample("Stops".to_owned(), 0, SliderSize::Medium, false, 9);
            });
    }));
}

#[tessera]
pub(super) fn tokens() {
    let expanded = remember(|| [true, false, false, false, false]);
    text()
        .content("Explore slider roles and dimensions. Select a state to inspect its tokens.")
        .style(type_style(16.0, 24.0));
    gap(Dp(32.0));
    for (index, label) in [
        "Enabled",
        "Disabled",
        "Hovered",
        "Focused",
        "Pressed (ripple)",
    ]
    .into_iter()
    .enumerate()
    {
        let open = expanded.with(|e| e[index]);
        button()
            .text()
            .on_click(move || expanded.with_mut(|e| e[index] = !e[index]))
            .child(move || {
                text()
                    .content(format!("{}  {}", if open { "−" } else { "+" }, label))
                    .style(type_style(20.0, 28.0));
            });
        if open {
            spec_table(
                vec!["Token".into(), "Value".into()],
                vec![
                    vec![
                        "Active track color".into(),
                        if index == 1 {
                            "On surface · 38%"
                        } else {
                            "Primary"
                        }
                        .into(),
                    ],
                    vec![
                        "Inactive track color".into(),
                        if index == 1 {
                            "On surface · 12%"
                        } else {
                            "Secondary container"
                        }
                        .into(),
                    ],
                    vec![
                        "Handle width".into(),
                        if index == 4 { "2dp" } else { "4dp" }.into(),
                    ],
                ],
            );
        }
    }
}

#[tessera]
pub(super) fn anatomy() {
    slider_figure(false);
    gap(Dp(16.0));
    for label in [
        "1  Value indicator (optional)",
        "2  Stop indicators (optional)",
        "3  Active track",
        "4  Handle",
        "5  Inactive track",
        "6  Inset icon (optional)",
    ] {
        text().content(label).style(type_style(16.0, 28.0));
    }
}

#[tessera]
pub(super) fn colors() {
    let theme = use_context::<MaterialTheme>().unwrap();
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .children(move || {
            for dark in [false, true] {
                material_theme()
                    .theme(move || {
                        let mut theme = theme.get();
                        theme.color_scheme = if dark {
                            MaterialColorScheme::dark_from_seed(ACCENT)
                        } else {
                            MaterialColorScheme::light_from_seed(ACCENT)
                        };
                        theme
                    })
                    .child(move || {
                        surface()
                            .modifier(Modifier::new().width(Dp(320.0)))
                            .child(move || {
                                column()
                                    .modifier(
                                        Modifier::new().fill_max_width().padding_all(Dp(24.0)),
                                    )
                                    .children(move || {
                                        sample(
                                            if dark { "Dark scheme" } else { "Light scheme" }
                                                .to_owned(),
                                            0,
                                            SliderSize::Medium,
                                            false,
                                            0,
                                        );
                                        sample(
                                            "Range".to_owned(),
                                            2,
                                            SliderSize::ExtraSmall,
                                            false,
                                            0,
                                        );
                                    });
                            });
                    });
            }
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Element".into(), "Color role".into()],
        vec![
            vec!["Active track / handle".into(), "Primary".into()],
            vec!["Inactive track".into(), "Secondary container".into()],
            vec!["Value indicator".into(), "Inverse surface".into()],
        ],
    );
}

#[tessera]
pub(super) fn states() {
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                sample(
                    "Enabled — hover, focus or press to compare".to_owned(),
                    0,
                    SliderSize::Medium,
                    false,
                    0,
                );
                gap(Dp(40.0));
                sample("Disabled".to_owned(), 0, SliderSize::Medium, true, 0);
            });
    }));
}

#[tessera]
pub(super) fn measurements() {
    slider_figure(true);
    gap(Dp(32.0));
    panel(RenderSlot::new(|| {
        column()
            .modifier(Modifier::new().fill_max_width())
            .children(|| {
                for size in SIZES {
                    sample(String::new(), 0, size, false, 0);
                    gap(Dp(24.0));
                }
            });
    }));
    gap(Dp(32.0));
    spec_table(
        ["Attribute", "XS", "S", "M", "L", "XL"]
            .map(str::to_owned)
            .to_vec(),
        vec![
            ["Track height", "16dp", "24dp", "40dp", "56dp", "96dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Handle height", "44dp", "44dp", "52dp", "68dp", "108dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Handle width", "4dp", "4dp", "4dp", "4dp", "4dp"]
                .map(str::to_owned)
                .to_vec(),
            ["Track shape", "8dp", "8dp", "12dp", "16dp", "28dp"]
                .map(str::to_owned)
                .to_vec(),
        ],
    );
    gap(Dp(64.0));
    text()
        .content("Material Design 3")
        .style(type_style(28.0, 36.0))
        .color(INK);
    text()
        .content("Sliders · Interactive specifications")
        .style(type_style(16.0, 24.0))
        .color(INK);
}

#[tessera]
fn slider_figure(measurements: bool) {
    if measurements {
        slider_measurement_figure();
        return;
    }
    let geometry = remember(Geometry::new);
    let value = remember(|| 0.5_f32);
    let inset = remember(|| 0.5_f32);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let height = width / 2.1649;
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            figure_grid(Dp(width), Dp(height));
            slider_value_indicator_provider().child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .align(Alignment::TopCenter)
                                    .offset(Dp(0.0), Dp(height * 0.416 - 24.0))
                                    .width(Dp(208.0)),
                            )
                            .children(move || {
                                slider()
                                    .modifier(watch(geometry, "standard").fill_max_width())
                                    .size(SliderSize::ExtraSmall)
                                    .value(value.get())
                                    .on_change(move |v| value.set(v))
                                    .steps(9)
                                    .value_indicator(SliderValueIndicatorVisibility::Always)
                                    .accessibility_label("Slider anatomy");
                            });
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .align(Alignment::TopCenter)
                                    .offset(Dp(0.0), Dp(height * 0.734 - 26.0))
                                    .width(Dp(192.0)),
                            )
                            .children(move || {
                                slider()
                                    .modifier(watch(geometry, "inset").fill_max_width())
                                    .size(SliderSize::Medium)
                                    .value(inset.get())
                                    .on_change(move |v| inset.set(v))
                                    .inset_icon(filled::VOLUME_UP_SVG)
                                    .accessibility_label("Slider inset anatomy");
                            });
                    });
            });
            slider_annotations(geometry, value.get(), measurements);
        });
}

#[tessera]
fn figure_grid(width: Dp, height: Dp) {
    let mut path = String::new();
    for row in 1..(height.0 / 8.0).ceil() as usize {
        for column in 1..(width.0 / 8.0).ceil() as usize {
            let x = column as f64 * 8.0 + 2.0;
            let y = row as f64 * 8.0 + 2.0;
            path.push_str(&format!("M{x},{y}h0.6v0.6h-0.6z"));
        }
    }
    annotations::lines(
        width,
        height,
        format!("<path d='{path}' fill='#79747e' fill-opacity='.6'/>"),
    );
}

#[tessera]
fn slider_annotations(geometry: State<Geometry>, value: f32, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(top), Some(bottom)) = (
        values.get("figure"),
        relative(&values, "standard"),
        relative(&values, "inset"),
    ) else {
        return;
    };
    let mut svg = String::new();
    let center = top.y + top.h * 0.5;
    let handle = top.x + 2.0 + (top.w - 4.0) * value as f64;
    if measurements {
        annotations::outline(&mut svg, top);
        annotations::outline(&mut svg, bottom);
        dotted(&mut svg, handle - 2.0, center - 22.0, 44.0, true, "#b14e70");
        annotations::dimension("44".into(), Dp(handle + 12.0), Dp(center - 10.0));
        annotations::dimension("4".into(), Dp(handle - 8.0), Dp(center + 36.0));
        annotations::dimension("16".into(), Dp(top.x + top.w + 16.0), Dp(center - 10.0));
        annotations::dimension(
            "40".into(),
            Dp(bottom.x + bottom.w + 16.0),
            Dp(bottom.y + bottom.h * 0.5 - 10.0),
        );
        annotations::dimension(
            "52".into(),
            Dp(bottom.x - 36.0),
            Dp(bottom.y + bottom.h * 0.5 - 10.0),
        );
    } else {
        let points = [
            (1, handle, center - 74.0, true),
            (2, top.x + 8.0, center + 8.0, false),
            (3, top.x + top.w * 0.22, center + 12.0, false),
            (4, handle, center + 22.0, false),
            (5, top.x + top.w * 0.77, center + 12.0, false),
        ];
        for (number, x, y, above) in points {
            let start = if above { y - 24.0 } else { y };
            dotted(
                &mut svg,
                x,
                start,
                if above { 24.0 } else { 36.0 },
                true,
                "#49454f",
            );
            annotations::marker(
                number,
                Dp(x - 12.0),
                Dp(if above { start - 32.0 } else { y + 36.0 }),
            );
        }
        let y = bottom.y + bottom.h * 0.5;
        dotted(&mut svg, bottom.x - 18.0, y, 18.0, false, "#49454f");
        annotations::marker(6, Dp(bottom.x - 42.0), Dp(y - 16.0));
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn slider_measurement_figure() {
    let geometry = remember(Geometry::new);
    let first = remember(|| 0.4_f32);
    let second = remember(|| 0.4_f32);
    let first_controller = remember(SliderController::new);
    let second_controller = remember(SliderController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let width = geometry
        .with(|v| {
            v.get("figure")
                .map(|r| Dp::from_pixels_f32(r.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let height = width * 600.0 / 2048.0;
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            figure_grid(Dp(width), Dp(height));
            slider_value_indicator_provider().child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        for (name, state, controller, x) in [
                            ("first", first, first_controller, 0.3013),
                            ("second", second, second_controller, 0.7265),
                        ] {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .width(Dp(296.0))
                                        .offset(Dp(width * x - 148.0), Dp(height * 0.5533 - 24.0)),
                                )
                                .children(move || {
                                    slider()
                                        .modifier(watch(geometry, name).fill_max_width())
                                        .value(state.get())
                                        .controller(controller)
                                        .on_change(move |v| state.set(v))
                                        .size(SliderSize::ExtraSmall)
                                        .value_indicator(SliderValueIndicatorVisibility::Always)
                                        .accessibility_label(format!("{name} slider measurements"));
                                });
                        }
                    });
            });
            slider_measurement_annotations(
                geometry,
                first.get(),
                second.get(),
                first_controller,
                second_controller,
            );
        });
}

#[tessera]
fn slider_measurement_annotations(
    geometry: State<Geometry>,
    first: f32,
    second: f32,
    first_controller: State<SliderController>,
    second_controller: State<SliderController>,
) {
    let values = geometry.get();
    let Some(root) = values.get("figure") else {
        return;
    };
    let mut svg = String::new();
    for (name, value, controller) in [
        ("first", first, first_controller),
        ("second", second, second_controller),
    ] {
        let Some(rect) = relative(&values, name) else {
            continue;
        };
        let y = rect.y + rect.h * 0.5;
        let base_width = Dp(4.0).to_px().0;
        let handle_pixels = if controller.with(|c| c.is_dragging() || c.is_focused()) {
            (base_width / 2).max(1)
        } else {
            base_width
        };
        let handle_width = Dp::from_pixels_f32(handle_pixels as f32).0;
        let gap = Dp::from_pixels_f32(Dp(6.0).to_px().to_f32()).0;
        let track_width = (rect.w - handle_width - gap * 2.0).max(0.0);
        let active_width = Dp::from_pixels_f32((Dp(track_width).to_pixels_f32() * value).round()).0;
        let handle = rect.x + gap + handle_width * 0.5 + active_width;
        let handle_right = handle + handle_width * 0.5;
        if name == "first" {
            measurement_span(&mut svg, rect.x - 12.0, y - 8.0, 16.0, true);
            annotations::dimension("16".into(), Dp(rect.x - 36.0), Dp(y - 10.0));
            measurement_span(&mut svg, handle_right, y + 14.0, 6.0, false);
            annotations::dimension("6".into(), Dp(handle_right - 2.0), Dp(y + 24.0));
            let stop = rect.x + rect.w - 8.0;
            measurement_span(&mut svg, stop - 2.0, y - 2.0, 4.0, false);
            annotations::dimension("4".into(), Dp(stop - 4.0), Dp(y - 28.0));
            measurement_span(&mut svg, stop + 6.0, y + 2.0, 6.0, true);
            annotations::dimension("6".into(), Dp(stop + 8.0), Dp(y));
            svg.push_str(&format!("<rect x='{handle_right}' y='{}' width='6' height='16' fill='#e883aa' fill-opacity='.32'/>",y-8.0));
        } else {
            let bubble_left = handle - 24.0;
            let bubble_top = y - 70.0;
            let bubble_bottom = y - 26.0;
            measurement_span(&mut svg, bubble_left + 2.0, bubble_top, 12.0, true);
            measurement_span(
                &mut svg,
                bubble_left + 2.0,
                bubble_bottom - 12.0,
                12.0,
                true,
            );
            measurement_span(&mut svg, handle + 8.0, bubble_top - 2.0, 16.0, false);
            measurement_span(&mut svg, handle, bubble_bottom, 4.0, true);
            annotations::dimension("12".into(), Dp(bubble_left - 22.0), Dp(bubble_top - 4.0));
            annotations::dimension(
                "12".into(),
                Dp(bubble_left - 22.0),
                Dp(bubble_bottom - 18.0),
            );
            annotations::dimension("16".into(), Dp(handle + 8.0), Dp(bubble_top - 26.0));
            annotations::dimension("4".into(), Dp(handle + 12.0), Dp(bubble_bottom - 10.0));
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

fn measurement_span(svg: &mut String, x: f64, y: f64, length: f64, vertical: bool) {
    let path = if vertical {
        format!("M{x},{y}v{length}m-3,0h6M{},{y}h6", x - 3.0)
    } else {
        format!("M{x},{y}h{length}m0,-3v6M{x},{}v6", y - 3.0)
    };
    svg.push_str(&format!(
        "<path d='{path}' fill='none' stroke='#e883aa' stroke-width='0.8'/>"
    ));
}
