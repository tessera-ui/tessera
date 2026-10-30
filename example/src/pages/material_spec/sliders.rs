//! Live slider illustrations for specification reading.
//!
//! ## Usage
//!
//! Adjust slider variants and compare their sizes, colors and enabled states.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    icon::icon,
    material_icons::filled,
    modifier::{InteractionVisualState, ModifierExt as _, SemanticsArgs},
    painter::Painter,
    row::row,
    shape_def::Shape,
    slider::{
        SliderController, SliderOrientation, SliderSize, SliderValueIndicatorVisibility,
        centered_slider, range_slider, slider, slider_value_indicator_provider,
    },
    surface::surface,
    text::text,
    text_field::TextFieldBuilder,
    text_input::TextInputController,
    theme::{MaterialColorScheme, MaterialTheme, material_theme},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, modifier::FocusModifierExt as _,
    remember, scaled_box, tessera, use_context,
};

use super::{
    ACCENT, INK,
    annotations::{self, PreciseGeometry, Rect, dotted, watch_precise},
    article::{ArticlePage, SpecSection, article},
    gap, spec_table, srgb, subheading, type_style,
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
            section("Availability & resources",RenderSlot::new(availability)),
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
                gap(Dp(32.0));subheading("Select & arrow".into());prose("Tab to a handle and use arrow keys to adjust it. In Tessera, Page Up and Page Down move a larger interval, while Home and End select an endpoint.");gap(Dp(24.0));value_example();
            })),
        ]},
        ArticlePage{title:"Accessibility",intro:None,sections:vec![
            section("Use cases",RenderSlot::new(||{prose("Assistive technology must identify each slider, adjust its handle along the range, and report the selected value. Range sliders expose both handles separately.");gap(Dp(24.0));sample("Price range".into(),2,SliderSize::Medium,false,0);})),
            section("Interaction & style",RenderSlot::new(||{prose("The handle narrows and the value appears during touch, pointer, or keyboard interaction.");gap(Dp(24.0));states();gap(Dp(24.0));subheading("Focus and navigation".into());prose("Initial focus lands on the handle. Arrow keys adjust its value without requiring pointer precision.");gap(Dp(24.0));value_example();})),
            section("Color contrast",RenderSlot::new(||{prose("The end of the inactive track needs a visual anchor with at least 3:1 contrast against the background. Use an end stop or a contrasting icon or label at the endpoint.");gap(Dp(24.0));colors();})),
            section("Keyboard navigation",RenderSlot::new(||{spec_table(vec!["Keys".into(),"Actions".into()],vec![vec!["Tab".into(),"Focus a slider handle".into()],vec!["Arrows".into(),"Change by one value or stop".into()],vec!["Page Up or Page Down".into(),"Change by a larger interval".into()],vec!["Home or End".into(),"Select the first or last value".into()]]);gap(Dp(24.0));value_example();})),
            section("Labeling elements",RenderSlot::new(||{prose("The accessibility label should match the adjacent visible label and use the slider role. Label external icon controls as buttons. Include units when they clarify the value.");gap(Dp(24.0));sample("Volume".into(),0,SliderSize::Medium,false,0).leading_icon(filled::VOLUME_UP_SVG);})),
        ]},
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn availability() {
    spec_table(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        vec![
            vec![
                "Design".into(),
                "Design Kit (Figma)".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Jetpack Compose: Expressive".into(),
                "Available".into(),
            ],
            vec![
                "Implementation".into(),
                "Android Views (MDC-Android): Expressive".into(),
                "Unavailable".into(),
            ],
            vec![
                "Implementation".into(),
                "Web: Expressive".into(),
                "Unavailable".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Tessera examples on this page use the native slider with standard, centered and range configurations, five sizes, both orientations and an optional inset icon.",
    );
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
    let invalid = remember(|| false);
    let editor = remember(|| {
        let mut editor = TextInputController::new(Dp(16.0), Some(Dp(24.0)));
        editor.set_text("50");
        editor
    });
    #[cfg(all(test, not(target_family = "wasm")))]
    size_figure_tests::VALUE_EDITOR.set(Some(editor));
    text()
        .content("Keyboard-adjustable value")
        .style(type_style(16.0, 24.0));
    gap(Dp(8.0));
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            slider()
                .modifier(Modifier::new().weight(1.0))
                .value(value.get())
                .on_change(move |next| {
                    value.set(next);
                    let formatted = format!("{:.2}", next * 100.0);
                    let formatted = formatted.trim_end_matches('0').trim_end_matches('.');
                    editor.with_mut(|editor| editor.set_text(formatted));
                    invalid.set(false);
                })
                .accessibility_label("Keyboard-adjustable value");
            tessera_components::spacer::spacer().modifier(Modifier::new().width(Dp(24.0)));
            TextFieldBuilder::outlined()
                .controller(editor)
                .label("Value")
                .accessibility_label("Slider value")
                .accessibility_description("Enter a value from 0 to 100")
                .modifier(Modifier::new().width(Dp(88.0)))
                .min_width(Dp::ZERO)
                .is_error(invalid.get())
                .on_change(move |raw: String| {
                    if let Some(next) = raw
                        .trim()
                        .parse::<f32>()
                        .ok()
                        .filter(|next| next.is_finite() && (0.0..=100.0).contains(next))
                    {
                        value.set(next / 100.0);
                        invalid.set(false);
                    } else {
                        invalid.set(true);
                    }
                    raw
                });
        });
    if invalid.get() {
        gap(Dp(4.0));
        let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
        text()
            .content("Enter a number from 0 to 100")
            .style(type_style(12.0, 16.0))
            .color(scheme.error)
            .modifier(Modifier::new().semantics(SemanticsArgs {
                role: Some(tessera_ui::accesskit::Role::Alert),
                ..Default::default()
            }));
    }
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
                .reverse_direction(true)
                .modifier(Modifier::new().height(Dp(240.0)))
                .value(standard.get())
                .on_change(move |v| standard.set(v))
                .accessibility_label("Vertical standard value");
            centered_slider()
                .orientation(SliderOrientation::Vertical)
                .reverse_direction(true)
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
    variants_figure();
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
            vec![
                "Standard".into(),
                "Available as “continuous” slider".into(),
                "Available".into(),
            ],
            vec![
                "Centered".into(),
                "Available (web only)".into(),
                "Available".into(),
            ],
            vec!["Range".into(), "Available".into(), "Available".into()],
            vec![
                "Discrete".into(),
                "Available".into(),
                "Available as “stops” configuration".into(),
            ],
        ],
    );
}

#[tessera]
fn variants_figure() {
    slider_canvas(
        1024.0,
        368.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let width = 1024.0;
            let scale = width / 1024.0;
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(368.0 * scale))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    for (index, x) in [(0, 137.0), (1, 420.0), (2, 704.0)] {
                        variant_specimen(
                            index,
                            Dp(x * scale),
                            Dp(164.0 * scale - 22.0),
                            Dp(185.0 * scale),
                        );
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(
                                Modifier::new()
                                    .offset(
                                        Dp([230.0, 512.0, 794.0][index] * scale - 20.0),
                                        Dp(246.0 * scale - 20.0),
                                    )
                                    .size(Dp(40.0), Dp(40.0))
                                    .background_with_shape(scheme.inverse_surface, Shape::Ellipse)
                                    .semantics(SemanticsArgs {
                                        role: Some(tessera_ui::accesskit::Role::Note),
                                        label: Some(format!("Slider variant marker {}", index + 1)),
                                        merge_descendants: true,
                                        ..Default::default()
                                    }),
                            )
                            .children(move || {
                                text()
                                    .content((index + 1).to_string())
                                    .style(type_style(16.0, 24.0))
                                    .color(scheme.inverse_on_surface);
                            });
                    }
                });
        }),
    );
}

#[tessera]
fn variant_specimen(index: usize, x: Dp, y: Dp, width: Dp) {
    let value = remember(|| if index == 1 { 0.25_f32 } else { 0.5 });
    let range = remember(|| (0.2_f32, 0.6_f32));
    boxed()
        .modifier(Modifier::new().offset(x, y).width(width))
        .children(move || match index {
            1 => {
                centered_slider()
                    .modifier(Modifier::new().fill_max_width())
                    .size(SliderSize::ExtraSmall)
                    .value(value.get())
                    .on_change(move |next| value.set(next))
                    .accessibility_label("Centered slider variant");
            }
            2 => {
                range_slider()
                    .modifier(Modifier::new().fill_max_width())
                    .size(SliderSize::ExtraSmall)
                    .value(range.get())
                    .on_change(move |next| range.set(next))
                    .accessibility_label("Range slider variant");
            }
            _ => {
                slider()
                    .modifier(Modifier::new().fill_max_width())
                    .size(SliderSize::ExtraSmall)
                    .value(value.get())
                    .on_change(move |next| value.set(next))
                    .accessibility_label("Standard slider variant");
            }
        });
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
    configuration_figure(false);
    gap(Dp(16.0));
    configuration_legend(1, "Orientation: Horizontal, vertical".into());
    configuration_legend(2, "Size: XS, S, M, L, XL".into());
    gap(Dp(32.0));
    configuration_figure(true);
    gap(Dp(16.0));
    configuration_legend(1, "Inset icon".into());
    configuration_legend(2, "Stops".into());
    configuration_legend(3, "Value indicator".into());
    gap(Dp(32.0));
    super::sliders_tokens::availability();
}

#[tessera]
fn configuration_legend(number: usize, label: String) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            configuration_number(number);
            text()
                .content(label.clone())
                .modifier(Modifier::new().padding_symmetric(Dp(12.0), Dp(4.0)))
                .style(type_style(16.0, 24.0));
        });
    gap(Dp(12.0));
}

#[tessera]
fn configuration_number(number: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(scheme.inverse_surface, Shape::rounded_rectangle(Dp(12.0))),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(16.0, 24.0))
                .color(scheme.inverse_on_surface);
        });
}

#[tessera]
fn configuration_marker(number: usize, x: Dp, y: Dp) {
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x.0 - 12.0), Dp(y.0 - 12.0))
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Note),
                    label: Some(format!("Slider configuration marker {number}")),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            configuration_number(number);
        });
}

#[tessera]
fn configuration_figure(optional: bool) {
    slider_canvas(
        1024.0,
        if optional { 461.0 } else { 733.0 },
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            let width = 1024.0;
            let scale = width / 1024.0;
            let height = if optional { 461.0 } else { 733.0 } * scale;
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
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
                                if optional {
                                    configuration_marker(1, Dp(197.0 * scale), Dp(159.0 * scale));
                                    configuration_marker(2, Dp(505.0 * scale), Dp(159.0 * scale));
                                    configuration_marker(3, Dp(815.0 * scale), Dp(159.0 * scale));
                                    configuration_slider(
                                        "Inset icon configuration".into(),
                                        SliderSize::Medium,
                                        Dp(78.0 * scale),
                                        Dp(273.0 * scale - 26.0),
                                        Dp(234.0 * scale),
                                        Dp(52.0),
                                        false,
                                        true,
                                        0,
                                        false,
                                        false,
                                    );
                                    configuration_slider(
                                        "Stops configuration".into(),
                                        SliderSize::ExtraSmall,
                                        Dp(388.0 * scale),
                                        Dp(273.0 * scale - 22.0),
                                        Dp(234.0 * scale),
                                        Dp(44.0),
                                        false,
                                        false,
                                        9,
                                        false,
                                        false,
                                    );
                                    configuration_slider(
                                        "Value indicator configuration".into(),
                                        SliderSize::ExtraSmall,
                                        Dp(698.0 * scale),
                                        Dp(273.0 * scale - 22.0),
                                        Dp(234.0 * scale),
                                        Dp(44.0),
                                        false,
                                        false,
                                        0,
                                        true,
                                        false,
                                    );
                                } else {
                                    configuration_marker(1, Dp(241.0 * scale), Dp(99.0 * scale));
                                    configuration_marker(2, Dp(719.0 * scale), Dp(99.0 * scale));
                                    configuration_slider(
                                        "Horizontal orientation configuration".into(),
                                        SliderSize::Medium,
                                        Dp(121.0 * scale),
                                        Dp(223.0 * scale - 26.0),
                                        Dp(248.0 * scale),
                                        Dp(52.0),
                                        false,
                                        false,
                                        0,
                                        false,
                                        true,
                                    );
                                    configuration_slider(
                                        "Vertical orientation configuration".into(),
                                        SliderSize::Medium,
                                        Dp(247.0 * scale - 26.0),
                                        Dp(317.0 * scale),
                                        Dp(52.0),
                                        Dp(288.0 * scale),
                                        true,
                                        false,
                                        0,
                                        false,
                                        true,
                                    );
                                    for (index, center_y) in
                                        [185.0, 265.0, 349.0, 445.0, 569.0].into_iter().enumerate()
                                    {
                                        let item_height = [44.0, 44.0, 52.0, 68.0, 108.0][index];
                                        configuration_slider(
                                            format!(
                                                "{} size configuration",
                                                ["XS", "S", "M", "L", "XL"][index]
                                            ),
                                            SIZES[index],
                                            Dp(530.0 * scale),
                                            Dp(center_y * scale - item_height * 0.5),
                                            Dp(380.0 * scale),
                                            Dp(item_height),
                                            false,
                                            false,
                                            0,
                                            false,
                                            true,
                                        );
                                    }
                                }
                            });
                    });
                });
        }),
    );
}

#[tessera]
fn configuration_slider(
    label: String,
    size: SliderSize,
    x: Dp,
    y: Dp,
    width: Dp,
    height: Dp,
    vertical: bool,
    inset: bool,
    steps: usize,
    indicator: bool,
    end_stop: bool,
) {
    let value = remember(|| 0.5_f32);
    boxed()
        .modifier(Modifier::new().offset(x, y).width(width).height(height))
        .children(move || {
            let mut control = slider()
                .modifier(Modifier::new().fill_max_size())
                .size(size)
                .orientation(if vertical {
                    SliderOrientation::Vertical
                } else {
                    SliderOrientation::Horizontal
                })
                .reverse_direction(vertical)
                .value(value.get())
                .on_change(move |next| value.set(next))
                .steps(steps)
                .show_stop_indicator(end_stop)
                .value_indicator(if indicator {
                    SliderValueIndicatorVisibility::Always
                } else {
                    SliderValueIndicatorVisibility::Hidden
                })
                .accessibility_label(label.clone());
            if inset {
                control = control.inset_icon(filled::VOLUME_UP_SVG);
            }
            drop(control);
        });
}

#[tessera]
pub(super) fn tokens() {
    super::sliders_tokens::tokens();
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
    color_figure();
    gap(Dp(24.0));
    prose("Slider color roles used for light and dark schemes:");
    gap(Dp(16.0));
    for (index, label) in [
        "Inverse surface",
        "Inverse on surface",
        "Primary",
        "On primary",
        "Primary",
        "Secondary container",
        "On secondary container",
        "On secondary container",
        "On primary",
    ]
    .into_iter()
    .enumerate()
    {
        configuration_legend(index + 1, label.to_owned());
    }
}

#[tessera]
fn color_figure() {
    slider_canvas(
        1024.0,
        473.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let width = 1024.0;
            let height = width * 946.0 / 2048.0;
            let theme = use_context::<MaterialTheme>().unwrap();
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(height))
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
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
                                color_panel(Dp(width * 0.5), Dp(height), dark);
                            });
                    }
                });
        }),
    );
}

#[tessera]
fn color_panel(width: Dp, height: Dp, dark: bool) {
    let geometry = remember(PreciseGeometry::new);
    let value = remember(|| 0.5_f32);
    let empty = remember(|| 0.0_f32);
    let inset = remember(|| 0.5_f32);
    let controller = remember(SliderController::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let scale = width.0 / 512.0;
    let label = if dark { "Dark" } else { "Light" };
    boxed()
        .modifier(
            watch_precise(geometry, "figure")
                .offset(if dark { width } else { Dp::ZERO }, Dp::ZERO)
                .size(width, height)
                .background(scheme.surface_container_low),
        )
        .children(move || {
            slider_value_indicator_provider().child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        boxed()
                            .modifier(
                                Modifier::new()
                                    .offset(Dp(156.0 * scale), Dp(168.0 * scale - 22.0))
                                    .width(Dp(200.0 * scale)),
                            )
                            .children(move || {
                                slider()
                                    .modifier(watch_precise(geometry, "standard").fill_max_width())
                                    .size(SliderSize::ExtraSmall)
                                    .value(value.get())
                                    .on_change(move |next| value.set(next))
                                    .controller(controller)
                                    .steps(9)
                                    .value_indicator(SliderValueIndicatorVisibility::Always)
                                    .accessibility_label(format!("{label} slider color roles"));
                            });
                        for (state, center, name) in
                            [(empty, 331.0, "inactive"), (inset, 391.0, "active")]
                        {
                            boxed()
                                .modifier(
                                    Modifier::new()
                                        .offset(Dp(166.0 * scale), Dp(center * scale - 26.0))
                                        .width(Dp(185.0 * scale)),
                                )
                                .children(move || {
                                    slider()
                                        .modifier(watch_precise(geometry, name).fill_max_width())
                                        .size(SliderSize::Medium)
                                        .value(state.get())
                                        .on_change(move |next| state.set(next))
                                        .inset_icon(filled::VOLUME_UP_SVG)
                                        .accessibility_label(format!(
                                            "{label} {name} inset color roles"
                                        ));
                                });
                        }
                    });
            });
            color_annotations(geometry, value.get(), controller, dark);
        });
}

#[tessera]
fn color_marker(number: usize, center_x: Dp, center_y: Dp, dark: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(center_x.0 - 12.0), Dp(center_y.0 - 16.0))
                .size(Dp(24.0), Dp(32.0))
                .background_with_shape(scheme.inverse_surface, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Note),
                    label: Some(format!(
                        "Slider color marker {number} {}",
                        if dark { "Dark" } else { "Light" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.inverse_on_surface);
        });
}

#[tessera]
fn color_annotations(
    geometry: State<PreciseGeometry>,
    value: f32,
    controller: State<SliderController>,
    dark: bool,
) {
    color_annotation_geometry(
        annotations::relative_precise_geometry(&geometry.get()),
        value,
        controller,
        dark,
    );
}

#[tessera]
fn color_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    value: f32,
    controller: State<SliderController>,
    dark: bool,
) {
    let (Some(root), Some(top), Some(inactive), Some(active)) = (
        values.get("figure"),
        values.get("standard").copied(),
        values.get("inactive").copied(),
        values.get("active").copied(),
    ) else {
        return;
    };
    let width = Dp(root.w);
    let height = Dp(root.h);
    let scale = width.0 / 512.0;
    let center = top.y + top.h * 0.5;
    let base = Dp(4.0).to_px().0;
    let handle_pixels =
        if controller.with(|controller| controller.is_dragging() || controller.is_focused()) {
            (base / 2).max(1)
        } else {
            base
        };
    let handle_width = Dp::from_pixels_f32(handle_pixels as f32).0;
    let gap = Dp::from_pixels_f32(Dp(6.0).to_px().to_f32()).0;
    let active_width = Dp::from_pixels_f32(
        (Dp((top.w - handle_width - gap * 2.0).max(0.0)).to_pixels_f32() * value).round(),
    )
    .0;
    let handle = top.x + gap + handle_width * 0.5 + active_width;
    let line = if dark { "#e6e1e5" } else { "#49454f" };
    let mut svg = String::new();
    let upper = center - 110.0;
    dotted(&mut svg, handle, upper + 16.0, 24.0, true, line);
    color_marker(1, Dp(handle), Dp(upper), dark);
    dotted(&mut svg, handle + 24.0, center - 48.0, 14.0, false, line);
    color_marker(2, Dp(handle + 50.0), Dp(center - 48.0), dark);
    for (number, x, start) in [
        (3, top.x + 6.0, center + 2.0),
        (4, top.x + 44.0 * scale, center + 8.0),
        (5, handle, center + 22.0),
        (6, top.x + 156.0 * scale, center + 8.0),
        (7, top.x + top.w - 3.0, center + 2.0),
    ] {
        let marker_y = 229.0 * scale;
        dotted(&mut svg, x, start, marker_y - 16.0 - start, true, line);
        color_marker(number, Dp(x), Dp(marker_y), dark);
    }
    for (number, rect) in [(8, inactive), (9, active)] {
        let y = rect.y + rect.h * 0.5;
        let x = 135.0 * scale;
        dotted(&mut svg, x + 12.0, y, rect.x + 20.0 - x - 12.0, false, line);
        color_marker(number, Dp(x), Dp(y), dark);
    }
    annotations::lines(width, height, svg);
}

#[tessera]
pub(super) fn states() {
    state_figure();
    gap(Dp(16.0));
    for (index, label) in ["Enabled", "Disabled", "Hovered", "Focused", "Pressed"]
        .into_iter()
        .enumerate()
    {
        configuration_legend(index + 1, label.to_owned());
    }
}

#[tessera]
fn state_figure() {
    slider_canvas(
        1024.0,
        720.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let width = 1024.0;
            let height = width * 1440.0 / 2048.0;
            let theme = use_context::<MaterialTheme>().unwrap();
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(height))
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
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
                                state_panel(Dp(width * 0.5), Dp(height), dark);
                            });
                    }
                });
        }),
    );
}

#[tessera]
fn state_panel(width: Dp, height: Dp, dark: bool) {
    let scale = width.0 / 512.0;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().modifier(Modifier::new().offset(if dark { width } else { Dp::ZERO }, Dp::ZERO).size(width, height).background(scheme.surface_container_low)).children(move || {
        slider_value_indicator_provider().child(move || {
            boxed().modifier(Modifier::new().fill_max_size()).children(move || {
                for (index, label, state) in [(0, "Enabled", InteractionVisualState::Rest), (1, "Disabled", InteractionVisualState::Rest), (2, "Hovered", InteractionVisualState::Hover), (3, "Focused", InteractionVisualState::Focus), (4, "Pressed", InteractionVisualState::Pressed)] {
                    state_specimen(index, label.to_owned(), state, width, dark);
                }
            });
        });
        let mut svg = String::new();
        for index in [2, 4] {
            let x = (289.0 + if dark { 12.0 } else { 0.0 }) * scale;
            let y = (112.0 + index as f64 * 124.0) * scale + 12.0;
            svg.push_str(&format!("<path d='M{x},{y}v20l5,-5l5,9l4,-2l-5,-9h8z' fill='{}' stroke='{}' stroke-width='1.2' stroke-linejoin='round'/>", if dark { "#e6e1e5" } else { "#1c1b1f" }, if dark { "#1c1b1f" } else { "#ffffff" }));
        }
        annotations::lines(width, height, svg);
    });
}

#[tessera]
fn state_specimen(
    index: usize,
    label: String,
    state: InteractionVisualState,
    width: Dp,
    dark: bool,
) {
    let value = remember(|| 0.5_f32);
    let scale = width.0 / 512.0;
    let center = (112.0 + index as f64 * 124.0) * scale;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let inset = if dark { 12.0 } else { 0.0 };
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp((181.0 + inset) * scale), Dp(center - 22.0))
                .width(Dp(200.0 * scale)),
        )
        .children(move || {
            slider()
                .modifier(Modifier::new().fill_max_width())
                .size(SliderSize::ExtraSmall)
                .value(value.get())
                .on_change(move |next| value.set(next))
                .disabled(index == 1)
                .presentation_state(state)
                .value_indicator(SliderValueIndicatorVisibility::OnInteraction)
                .accessibility_label(format!(
                    "{} {label} slider state",
                    if dark { "Dark" } else { "Light" }
                ));
        });
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((109.0 + inset) * scale - 12.0), Dp(center - 12.0))
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(scheme.inverse_surface, Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Note),
                    label: Some(format!(
                        "Slider state marker {} {}",
                        index + 1,
                        if dark { "Dark" } else { "Light" }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content((index + 1).to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.inverse_on_surface);
        });
}

#[tessera]
pub(super) fn measurements() {
    slider_figure(true);
    gap(Dp(32.0));
    slider_size_figure();
    gap(Dp(32.0));
    spec_table(
        ["Attribute", "XS", "S", "M", "L", "XL"]
            .map(str::to_owned)
            .to_vec(),
        vec![
            ["Track height", "16dp", "24dp", "40dp", "56dp", "96dp"]
                .map(str::to_owned)
                .to_vec(),
            [
                "Label container height",
                "44dp",
                "44dp",
                "44dp",
                "44dp",
                "44dp",
            ]
            .map(str::to_owned)
            .to_vec(),
            [
                "Label container width",
                "48dp",
                "48dp",
                "48dp",
                "48dp",
                "48dp",
            ]
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
            ["Inset icon size", "—", "—", "24dp", "24dp", "32dp"]
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

    slider_canvas(
        1024.0,
        473.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let value = remember(|| 0.5_f32);
            let inset = remember(|| 0.5_f32);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            let width = 1024.0;
            let height = width / 2.1649;
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
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
                                            .modifier(
                                                watch_precise(geometry, "standard")
                                                    .fill_max_width(),
                                            )
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
                                            .modifier(
                                                watch_precise(geometry, "inset").fill_max_width(),
                                            )
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
        }),
    );
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
fn slider_annotations(geometry: State<PreciseGeometry>, value: f32, measurements: bool) {
    slider_annotation_geometry(
        annotations::relative_precise_geometry(&geometry.get()),
        value,
        measurements,
    );
}

#[tessera]
fn slider_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    value: f32,
    measurements: bool,
) {
    let (Some(root), Some(top), Some(bottom)) = (
        values.get("figure"),
        values.get("standard").copied(),
        values.get("inset").copied(),
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
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[tessera]
fn slider_measurement_figure() {
    slider_canvas(
        1024.0,
        300.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let first = remember(|| 0.4_f32);
            let second = remember(|| 0.4_f32);
            let first_controller = remember(SliderController::new);
            let second_controller = remember(SliderController::new);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            let width = 1024.0;
            let height = width * 600.0 / 2048.0;
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
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
                                        .modifier(Modifier::new().width(Dp(296.0)).offset(
                                            Dp(width * x - 148.0),
                                            Dp(height * 0.5533 - 24.0),
                                        ))
                                        .children(move || {
                                            slider()
                                                .modifier(
                                                    watch_precise(geometry, name).fill_max_width(),
                                                )
                                                .value(state.get())
                                                .controller(controller)
                                                .on_change(move |v| state.set(v))
                                                .size(SliderSize::ExtraSmall)
                                                .value_indicator(
                                                    SliderValueIndicatorVisibility::Always,
                                                )
                                                .accessibility_label(format!(
                                                    "{name} slider measurements"
                                                ));
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
        }),
    );
}

#[tessera]
fn slider_measurement_annotations(
    geometry: State<PreciseGeometry>,
    first: f32,
    second: f32,
    first_controller: State<SliderController>,
    second_controller: State<SliderController>,
) {
    slider_measurement_geometry(
        annotations::relative_precise_geometry(&geometry.get()),
        first,
        second,
        first_controller,
        second_controller,
    );
}

#[tessera]
fn slider_measurement_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    first: f32,
    second: f32,
    first_controller: State<SliderController>,
    second_controller: State<SliderController>,
) {
    let Some(root) = values.get("figure") else {
        return;
    };
    let mut svg = String::new();
    for (name, value, controller) in [
        ("first", first, first_controller),
        ("second", second, second_controller),
    ] {
        let Some(rect) = values.get(name).copied() else {
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
    annotations::lines(Dp(root.w), Dp(root.h), svg);
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

#[tessera]
fn slider_size_figure() {
    slider_canvas(
        1022.0,
        546.0,
        RenderSlot::new(move || {
            let geometry = remember(PreciseGeometry::new);
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            let width = 1022.0;
            let height = width * 1092.0 / 2044.0;
            boxed()
                .modifier(
                    watch_precise(geometry, "figure")
                        .fill_max_width()
                        .height(Dp(height))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        ),
                )
                .children(move || {
                    figure_grid(Dp(width), Dp(height));
                    for (index, x, y, item_width) in [
                        (0, 382.0, 288.0, 464.0),
                        (1, 1022.0, 288.0, 464.0),
                        (2, 1662.0, 288.0, 464.0),
                        (3, 496.0, 740.0, 712.0),
                        (4, 1474.0, 740.0, 712.0),
                    ] {
                        size_specimen(
                            geometry,
                            index,
                            Dp(width * x / 2044.0),
                            Dp(height * y / 1092.0),
                            Dp(width * item_width / 2044.0),
                        );
                    }
                });
        }),
    );
    gap(Dp(16.0));
    text()
        .content("1  XS     2  S     3  M     4  L     5  XL")
        .style(type_style(16.0, 24.0));
}

#[tessera]
fn size_specimen(
    geometry: State<PreciseGeometry>,
    index: usize,
    center_x: Dp,
    center_y: Dp,
    width: Dp,
) {
    let value = remember(|| 0.5_f32);
    let controller = remember(SliderController::new);
    let name = ["size-xs", "size-s", "size-m", "size-l", "size-xl"][index];
    let component_height = [44.0, 44.0, 52.0, 68.0, 108.0][index];
    boxed()
        .modifier(Modifier::new().fill_max_size())
        .children(move || {
            boxed()
                .modifier(Modifier::new().width(width).offset(
                    Dp(center_x.0 - width.0 * 0.5),
                    Dp(center_y.0 - component_height * 0.5),
                ))
                .children(move || {
                    let mut specimen = slider()
                        .modifier(watch_precise(geometry, name).fill_max_width())
                        .size(SIZES[index])
                        .value(value.get())
                        .controller(controller)
                        .on_change(move |next| value.set(next))
                        .accessibility_label(format!(
                            "{} size measurements",
                            ["XS", "S", "M", "L", "XL"][index]
                        ));
                    if index >= 2 {
                        specimen = specimen.inset_icon(filled::VOLUME_UP_SVG);
                    }
                    drop(specimen);
                });
            size_annotations(geometry, index, value.get(), controller);
        });
}

#[tessera]
fn size_annotations(
    geometry: State<PreciseGeometry>,
    index: usize,
    value: f32,
    controller: State<SliderController>,
) {
    size_annotation_geometry(
        annotations::relative_precise_geometry(&geometry.get()),
        index,
        value,
        controller,
    );
}

#[tessera]
fn size_annotation_geometry(
    values: std::collections::BTreeMap<&'static str, Rect>,
    index: usize,
    value: f32,
    controller: State<SliderController>,
) {
    let name = ["size-xs", "size-s", "size-m", "size-l", "size-xl"][index];
    let (Some(root), Some(rect)) = (values.get("figure"), values.get(name).copied()) else {
        return;
    };
    let track = [16.0, 24.0, 40.0, 56.0, 96.0][index];
    let handle_height = [44.0, 44.0, 52.0, 68.0, 108.0][index];
    let radius = [8.0, 8.0, 12.0, 16.0, 28.0][index];
    let center = rect.y + rect.h * 0.5;
    let base_width = Dp(4.0).to_px().0;
    let width_pixels = if controller.with(|c| c.is_dragging() || c.is_focused()) {
        (base_width / 2).max(1)
    } else {
        base_width
    };
    let handle_width = Dp::from_pixels_f32(width_pixels as f32).0;
    let gap = Dp::from_pixels_f32(Dp(6.0).to_px().to_f32()).0;
    let active = Dp::from_pixels_f32(
        (Dp((rect.w - handle_width - gap * 2.0).max(0.0)).to_pixels_f32() * value).round(),
    )
    .0;
    let handle = rect.x + gap + handle_width * 0.5 + active;
    let mut svg = String::new();
    measurement_span(
        &mut svg,
        handle + 8.0,
        center - handle_height * 0.5,
        handle_height,
        true,
    );
    annotations::dimension(
        format!("{handle_height:.0}"),
        Dp(handle + 14.0),
        Dp(center - 10.0),
    );
    measurement_span(
        &mut svg,
        rect.x + rect.w + 8.0,
        center - track * 0.5,
        track,
        true,
    );
    annotations::dimension(
        format!("{track:.0}"),
        Dp(rect.x + rect.w + 14.0),
        Dp(center - 10.0),
    );
    let marker_y = Dp(root.h).0
        * if index < 3 {
            156.0 / 1092.0
        } else {
            566.0 / 1092.0
        };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .offset(Dp(handle - 12.0), Dp(marker_y - 12.0))
                .background_with_shape(scheme.inverse_surface, Shape::Ellipse)
                .semantics(SemanticsArgs {
                    role: Some(tessera_ui::accesskit::Role::Note),
                    label: Some(format!("Slider size marker {}", index + 1)),
                    ..Default::default()
                }),
        )
        .children(move || {
            text()
                .content((index + 1).to_string())
                .style(type_style(14.0, 20.0))
                .color(scheme.inverse_on_surface);
        });
    if index >= 2 {
        let icon = [24.0, 24.0, 32.0][index - 2];
        let icon_x = rect.x + 8.0;
        let icon_box = annotations::Rect {
            x: icon_x,
            y: center - icon * 0.5,
            w: icon,
            h: icon,
        };
        annotations::outline(&mut svg, icon_box);
        measurement_span(&mut svg, icon_x - 8.0, icon_box.y, icon, true);
        annotations::dimension(format!("{icon:.0}"), Dp(icon_x - 32.0), Dp(center - 10.0));
        let x = rect.x + rect.w - radius;
        let y = center + track * 0.5 - radius;
        svg.push_str(&format!("<path d='M{x},{y}h{radius}a{radius},{radius} 0 0 1 -{radius},{radius}z' fill='#e883aa' fill-opacity='.32'/>"));
        let leader = rect.x + rect.w - radius * 0.5;
        dotted(
            &mut svg,
            leader,
            center + track * 0.5,
            24.0,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{radius:.0}"),
            Dp(leader - 8.0),
            Dp(center + track * 0.5 + 24.0),
        );
    } else {
        let stop = rect.x + rect.w - 8.0;
        dotted(&mut svg, stop, center + 2.0, 24.0, true, "#b14e70");
        annotations::dimension("8".into(), Dp(stop - 4.0), Dp(center + 28.0));
    }
    annotations::lines(Dp(root.w), Dp(root.h), svg);
}

#[cfg(all(test, not(target_family = "wasm")))]
mod size_figure_tests {
    use std::{cell::Cell, time::Duration};

    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::{Action, ActionRequest, Node, NodeId, Role, TreeId},
    };

    use super::*;

    thread_local! {
        pub(super) static VALUE_EDITOR: Cell<Option<State<TextInputController>>> = const { Cell::new(None) };
        static OPTIONAL_CONFIGURATIONS: Cell<bool> = const { Cell::new(false) };
        static NARROW_SCROLL: Cell<Option<State<tessera_components::scrollable::ScrollableController>>> = const { Cell::new(None) };
    }

    struct Pipelines;
    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn narrow_configuration_fixture() {
        let scroll = remember(tessera_components::scrollable::ScrollableController::default);
        NARROW_SCROLL.set(Some(scroll));
        material_theme().child(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .fill_max_width()
                        .height(Dp(350.0))
                        .vertical_scroll(scroll),
                )
                .children(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width())
                        .children(|| {
                            configuration_figure(true);
                            tessera_components::spacer::spacer()
                                .modifier(Modifier::new().height(Dp(400.0)));
                        });
                });
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for scaled slider input and value indicators"]
    fn narrow_native_configuration_maps_drag_and_value_indicator_without_scrolling() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(narrow_configuration_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 640,
                height: 350,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .unwrap();
        renderer.render_frames(6);
        let control = node(&renderer, "Value indicator configuration", Role::Slider)
            .1
            .bounds()
            .unwrap();
        assert_eq!(control.width(), 146.25);
        assert_eq!(control.height(), 27.5);
        assert_eq!(control.x0, 436.25);
        assert_eq!(control.y0, 156.875);
        let scroll = NARROW_SCROLL.get().unwrap();
        let before =
            scroll.with(tessera_components::scrollable::ScrollableController::child_position);
        let start = PxPosition::from_f32_xy(
            ((control.x0 + control.x1) * 0.5) as f32,
            ((control.y0 + control.y1) * 0.5) as f32,
        );
        let end = PxPosition::from_f32_xy((control.x0 + 183.0 * 0.625) as f32, start.y);
        renderer.pointer_move(MOUSE_POINTER_ID, start);
        renderer.pointer_press(MOUSE_POINTER_ID, start, PressKeyEventType::Left);
        renderer.render_frames(1);
        renderer.pointer_move(MOUSE_POINTER_ID, end);
        renderer.render_frames(2);
        renderer.pointer_release(MOUSE_POINTER_ID, end, PressKeyEventType::Left);
        renderer.render_frames(24);
        let value = node(&renderer, "Value indicator configuration", Role::Slider)
            .1
            .numeric_value()
            .unwrap();
        assert!(
            (value - 0.8).abs() < 0.005,
            "mapped native drag selected {value}"
        );
        assert_eq!(
            node(&renderer, "Stops configuration", Role::Slider)
                .1
                .numeric_value(),
            Some(0.5)
        );
        let snapshot = renderer.snapshot();
        let mut pending = vec![(&snapshot["root"], false)];
        let mut label_bounds = None;
        while let Some((node, inherited_hidden)) = pending.pop() {
            let hidden =
                inherited_hidden || node["accessibility"]["hidden"].as_bool().unwrap_or(false);
            if hidden && node["accessibility"]["label"].as_str() == Some("80") {
                let bounds = &node["bounds"];
                label_bounds = Some((
                    bounds["x"].as_f64().unwrap(),
                    bounds["y"].as_f64().unwrap(),
                    bounds["width"].as_f64().unwrap(),
                    bounds["height"].as_f64().unwrap(),
                ));
                break;
            }
            if let Some(children) = node["children"].as_array() {
                pending.extend(children.iter().map(|child| (child, hidden)));
            }
        }
        let (x, y, width, height) =
            label_bounds.expect("the native decorative value indicator must display 80");
        assert!(width > 0.0 && height > 0.0);
        assert!(x >= 0.0 && y >= 0.0 && x + width <= 640.0 && y + height <= 350.0);
        let pixels = renderer.read_pixels().unwrap();
        let surface_x = (x - 2.0).floor().max(0.0) as usize;
        let surface = &pixels[(((y + height / 2.0) as usize * 640 + surface_x) * 4)..][..3];
        let mut foreground = 0;
        let mut background = 0;
        let mut maximum_contrast = 0;
        for row in y.floor() as usize..(y + height).ceil() as usize {
            for column in x.floor() as usize..(x + width).ceil() as usize {
                let pixel = &pixels[(row * 640 + column) * 4..][..3];
                let contrast = pixel
                    .iter()
                    .zip(surface)
                    .map(|(value, surface)| value.abs_diff(*surface))
                    .max()
                    .unwrap();
                maximum_contrast = maximum_contrast.max(contrast);
                foreground += usize::from(contrast > 64);
                background += usize::from(contrast < 8);
            }
        }
        if let Some(directory) = std::env::var_os("TESSERA_MENU_QA_DIR") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            tessera_ui::headless::write_png_file(
                directory.join("slider-narrow-indicator80.png"),
                640,
                350,
                &pixels,
            )
            .unwrap();
            std::fs::write(
                directory.join("slider-narrow-indicator80.json"),
                snapshot.to_string(),
            )
            .unwrap();
        }
        assert!(
            foreground > 2 && background > 2 && maximum_contrast > 96,
            "the 80 glyphs must paint contrasting ink on the native indicator surface"
        );
        let outside = &pixels[(((y - 24.0).max(0.0) as usize * 640 + surface_x) * 4)..][..3];
        assert!(
            surface.iter().all(|channel| *channel < 120),
            "the native indicator must paint its inverse surface around the value label"
        );
        assert!(
            outside.iter().all(|channel| *channel > 180),
            "the surrounding figure must remain distinct from the inverse indicator surface"
        );
        assert_eq!(
            scroll.with(tessera_components::scrollable::ScrollableController::child_position),
            before
        );
    }

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    column()
                        .modifier(Modifier::new().width(Dp(1022.0)))
                        .children(|| {
                            slider_size_figure();
                        });
                });
        });
    }

    fn node(renderer: &HeadlessRenderer, label: &str, role: Role) -> (NodeId, Node) {
        renderer
            .accessibility_snapshot()
            .expect("size figure semantics")
            .nodes
            .into_iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing {label}: {role:?}"))
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for the actual slider and annotation pipelines"]
    fn slider_size_figure_places_five_native_sizes_and_tracks_accessible_value_changes() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1200,
                height: 700,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native size figure renderer");
        renderer.render_frames(6);
        for (label, width, height, center_x, center_y) in [
            ("XS", 232.0, 44.0, 191.0, 144.0),
            ("S", 232.0, 44.0, 511.0, 144.0),
            ("M", 232.0, 52.0, 831.0, 144.0),
            ("L", 356.0, 68.0, 248.0, 370.0),
            ("XL", 356.0, 108.0, 737.0, 370.0),
        ] {
            let (_, slider) = node(
                &renderer,
                &format!("{label} size measurements"),
                Role::Slider,
            );
            let bounds = slider.bounds().expect("native slider bounds");
            assert_eq!(bounds.width(), width, "{label} width");
            assert_eq!(bounds.height(), height, "{label} height");
            assert_eq!(
                (bounds.x0 + bounds.x1) * 0.5,
                center_x,
                "{label} horizontal placement"
            );
            assert_eq!(
                (bounds.y0 + bounds.y1) * 0.5,
                center_y,
                "{label} row placement"
            );
            assert_eq!(slider.numeric_value(), Some(0.5));
        }
        let marker = node(&renderer, "Slider size marker 1", Role::Note)
            .1
            .bounds()
            .expect("original marker bounds");
        assert_eq!(marker.width(), 24.0);
        assert_eq!(marker.height(), 24.0);
        assert_eq!((marker.x0 + marker.x1) * 0.5, 191.0);
        let (slider_id, _) = node(&renderer, "XS size measurements", Role::Slider);
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action: Action::Increment,
            target_tree: TreeId::ROOT,
            target_node: slider_id,
            data: None
        }));
        renderer.render_frames(4);
        let value = node(&renderer, "XS size measurements", Role::Slider)
            .1
            .numeric_value()
            .expect("updated slider value");
        assert!(
            (value - 0.55).abs() < 0.000001,
            "the native increment action updates the figure state"
        );
        let moved = node(&renderer, "Slider size marker 1", Role::Note)
            .1
            .bounds()
            .expect("updated marker bounds");
        assert!(
            moved.x0 > marker.x0,
            "the annotation follows the live handle rather than staying at 50 percent"
        );
        assert_eq!(moved.y0, marker.y0);
        assert_eq!(
            node(&renderer, "S size measurements", Role::Slider)
                .1
                .numeric_value(),
            Some(0.5)
        );
    }

    #[tessera]
    fn value_fixture() {
        material_theme().child(|| {
            column()
                .modifier(Modifier::new().width(Dp(480.0)))
                .children(value_example);
        });
    }

    fn value_action(renderer: &mut HeadlessRenderer, label: &str, role: Role, action: Action) {
        let target_node = node(renderer, label, role).0;
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        }));
        renderer.render_frames(4);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual slider, field and focus routing"]
    fn external_value_field_synchronizes_both_directions_and_follows_the_slider_in_focus_order() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(value_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 640,
                height: 160,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native external value renderer");
        renderer.render_frames(4);
        value_action(
            &mut renderer,
            "Keyboard-adjustable value",
            Role::Slider,
            Action::Increment,
        );
        assert_eq!(
            node(&renderer, "Slider value", Role::TextInput).1.value(),
            Some("55")
        );
        value_action(
            &mut renderer,
            "Keyboard-adjustable value",
            Role::Slider,
            Action::Focus,
        );
        assert!(
            tessera_ui::FocusManager::current()
                .request_move_focus(tessera_ui::FocusDirection::Next)
        );
        renderer.render_frames(4);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert_eq!(
            snapshot.focus,
            node(&renderer, "Slider value", Role::TextInput).0
        );
        let editor = VALUE_EDITOR.get().unwrap();
        editor.with_mut(|editor| editor.set_text(""));
        renderer.render_frames(2);
        renderer.inject_text("75");
        renderer.render_frames(4);
        assert_eq!(
            node(&renderer, "Slider value", Role::TextInput).1.value(),
            Some("75")
        );
        assert_eq!(
            node(&renderer, "Keyboard-adjustable value", Role::Slider)
                .1
                .numeric_value(),
            Some(0.75)
        );
        for invalid in ["NaN", "101", "-1"] {
            editor.with_mut(|editor| editor.set_text(""));
            renderer.render_frames(2);
            renderer.inject_text(invalid);
            renderer.render_frames(4);
            assert_eq!(
                node(&renderer, "Slider value", Role::TextInput).1.value(),
                Some(invalid)
            );
            assert_eq!(
                node(&renderer, "Keyboard-adjustable value", Role::Slider)
                    .1
                    .numeric_value(),
                Some(0.75)
            );
            node(&renderer, "Enter a number from 0 to 100", Role::Alert);
        }
        value_action(
            &mut renderer,
            "Keyboard-adjustable value",
            Role::Slider,
            Action::Increment,
        );
        assert_eq!(
            node(&renderer, "Slider value", Role::TextInput).1.value(),
            Some("80")
        );
        assert!(
            !renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Alert)
        );
        value_action(
            &mut renderer,
            "Slider value",
            Role::TextInput,
            Action::Focus,
        );
        editor.with_mut(|editor| editor.set_text(""));
        renderer.render_frames(2);
        renderer.inject_text("12.5");
        renderer.render_frames(4);
        assert_eq!(
            node(&renderer, "Keyboard-adjustable value", Role::Slider)
                .1
                .numeric_value(),
            Some(0.125)
        );
        value_action(
            &mut renderer,
            "Keyboard-adjustable value",
            Role::Slider,
            Action::Increment,
        );
        assert_eq!(
            node(&renderer, "Slider value", Role::TextInput).1.value(),
            Some("17.5")
        );
        value_action(
            &mut renderer,
            "Slider value",
            Role::TextInput,
            Action::Focus,
        );
        for (input, expected) in [("0", 0.0), ("100", 1.0)] {
            editor.with_mut(|editor| editor.set_text(""));
            renderer.render_frames(2);
            renderer.inject_text(input);
            renderer.render_frames(4);
            assert_eq!(
                node(&renderer, "Keyboard-adjustable value", Role::Slider)
                    .1
                    .numeric_value(),
                Some(expected),
                "the editable value accepts the inclusive {input} endpoint"
            );
            assert!(
                !renderer
                    .accessibility_snapshot()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Alert)
            );
        }
    }

    #[tessera]
    fn color_fixture() {
        material_theme().child(|| {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    column()
                        .modifier(Modifier::new().width(Dp(1024.0)))
                        .children(color_figure);
                });
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual slider color roles and annotations"]
    fn color_figure_places_nine_callouts_in_each_scheme_and_tracks_the_live_handle() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(color_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 1100,
                height: 550,
                sample_count: 1,
                frame_time: Duration::from_nanos(16_666_667),
                ..Default::default()
            },
        )
        .expect("native slider color renderer");
        renderer.render_frames(6);
        for (scheme, offset) in [("Light", 0.0), ("Dark", 512.0)] {
            let bounds = node(
                &renderer,
                &format!("{scheme} slider color roles"),
                Role::Slider,
            )
            .1
            .bounds()
            .unwrap();
            assert_eq!(
                (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
                (156.0 + offset, 146.0, 200.0, 44.0)
            );
            for (number, x, y) in [
                (1, 256.0, 58.0),
                (2, 306.0, 120.0),
                (3, 162.0, 229.0),
                (4, 200.0, 229.0),
                (5, 256.0, 229.0),
                (6, 312.0, 229.0),
                (7, 353.0, 229.0),
                (8, 135.0, 331.0),
                (9, 135.0, 391.0),
            ] {
                let bounds = node(
                    &renderer,
                    &format!("Slider color marker {number} {scheme}"),
                    Role::Note,
                )
                .1
                .bounds()
                .unwrap();
                assert_eq!(
                    ((bounds.x0 + bounds.x1) * 0.5, (bounds.y0 + bounds.y1) * 0.5,),
                    (x + offset, y),
                    "{scheme} color role {number} anchor"
                );
                assert_eq!((bounds.width(), bounds.height()), (24.0, 32.0));
            }
        }
        let marker = node(&renderer, "Slider color marker 5 Light", Role::Note)
            .1
            .bounds()
            .unwrap();
        let other_marker = node(&renderer, "Slider color marker 5 Dark", Role::Note)
            .1
            .bounds()
            .unwrap();
        value_action(
            &mut renderer,
            "Light slider color roles",
            Role::Slider,
            Action::Increment,
        );
        let moved = node(&renderer, "Slider color marker 5 Light", Role::Note)
            .1
            .bounds()
            .unwrap();
        assert!(moved.x0 > marker.x0);
        assert_eq!(moved.y0, marker.y0);
        assert_eq!(
            node(&renderer, "Slider color marker 5 Dark", Role::Note)
                .1
                .bounds()
                .unwrap(),
            other_marker
        );
        assert_eq!(
            node(&renderer, "Dark slider color roles", Role::Slider)
                .1
                .numeric_value(),
            Some(0.5)
        );
    }

    #[tessera]
    fn configuration_fixture() {
        material_theme().child(|| {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    column()
                        .modifier(Modifier::new().width(Dp(1024.0)))
                        .children(|| {
                            configuration_figure(OPTIONAL_CONFIGURATIONS.get());
                        });
                });
        });
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for actual slider configurations and geometry"]
    fn configuration_figures_keep_official_orientation_size_and_optional_control_geometry() {
        for optional in [false, true] {
            OPTIONAL_CONFIGURATIONS.set(optional);
            let mut renderer = HeadlessRenderer::new(
                Box::new(configuration_fixture),
                vec![Box::new(Pipelines)],
                HeadlessConfig {
                    width: 1100,
                    height: 800,
                    sample_count: 1,
                    frame_time: Duration::from_nanos(16_666_667),
                    ..Default::default()
                },
            )
            .expect("native slider configuration renderer");
            renderer.render_frames(6);
            if let Some(directory) = std::env::var_os("TESSERA_SLIDER_QA_DIR") {
                let file = if optional {
                    "slider-optional-configurations-native.png"
                } else {
                    "slider-orientation-size-configurations-native.png"
                };
                tessera_ui::write_png_file(
                    std::path::Path::new(&directory).join(file),
                    1100,
                    800,
                    &renderer.read_pixels().unwrap(),
                )
                .unwrap();
            }
            let expected = if optional {
                vec![
                    ("Inset icon configuration", 78.0, 247.0, 234.0, 52.0),
                    ("Stops configuration", 388.0, 251.0, 234.0, 44.0),
                    ("Value indicator configuration", 698.0, 251.0, 234.0, 44.0),
                ]
            } else {
                vec![
                    (
                        "Horizontal orientation configuration",
                        121.0,
                        197.0,
                        248.0,
                        52.0,
                    ),
                    (
                        "Vertical orientation configuration",
                        221.0,
                        317.0,
                        52.0,
                        288.0,
                    ),
                    ("XS size configuration", 530.0, 163.0, 380.0, 44.0),
                    ("S size configuration", 530.0, 243.0, 380.0, 44.0),
                    ("M size configuration", 530.0, 323.0, 380.0, 52.0),
                    ("L size configuration", 530.0, 411.0, 380.0, 68.0),
                    ("XL size configuration", 530.0, 515.0, 380.0, 108.0),
                ]
            };
            let snapshot = renderer.accessibility_snapshot().unwrap();
            assert_eq!(
                snapshot
                    .nodes
                    .iter()
                    .filter(|(_, node)| node.role() == Role::Slider)
                    .count(),
                expected.len()
            );
            for (label, x, y, width, height) in &expected {
                let control = node(&renderer, label, Role::Slider).1;
                let bounds = control.bounds().unwrap();
                assert_eq!(
                    (bounds.x0, bounds.y0, bounds.width(), bounds.height()),
                    (*x, *y, *width, *height),
                    "{label}"
                );
                assert_eq!(control.numeric_value(), Some(0.5));
            }
            for number in 1..=if optional { 3 } else { 2 } {
                let bounds = node(
                    &renderer,
                    &format!("Slider configuration marker {number}"),
                    Role::Note,
                )
                .1
                .bounds()
                .unwrap();
                assert_eq!((bounds.width(), bounds.height()), (24.0, 24.0));
            }
            let label = expected[0].0;
            value_action(&mut renderer, label, Role::Slider, Action::Increment);
            assert!(
                (node(&renderer, label, Role::Slider)
                    .1
                    .numeric_value()
                    .unwrap()
                    - 0.55)
                    .abs()
                    < 0.000001
            );
            assert_eq!(
                node(&renderer, expected[1].0, Role::Slider)
                    .1
                    .numeric_value(),
                Some(0.5)
            );
        }
    }
}

#[tessera]
fn slider_canvas(canonical_width: f64, canonical_height: f64, child: RenderSlot) {
    let available_width = remember(move || Dp(canonical_width));
    let scale = (available_width.get().0 / canonical_width) as f32;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                if next != available_width.get() {
                    available_width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .modifier(
                        Modifier::new().required_size(Dp(canonical_width), Dp(canonical_height)),
                    )
                    .children_shared(child);
            });
        });
}
