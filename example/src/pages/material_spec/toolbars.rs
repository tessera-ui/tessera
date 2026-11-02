//! Interactive expressive toolbar specifications.
//!
//! ## Usage
//!
//! Explore toolbar action groups, placement and keyboard navigation.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment, MainAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    divider::vertical_divider,
    floating_action_button::{
        FloatingActionButtonElevation, FloatingActionButtonSize, floating_action_button,
    },
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, Padding, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    surface::{SurfaceStyle, surface},
    text::text,
    text_field::text_field,
    text_table::TextTableCell,
    theme::MaterialTheme,
    toolbar::{ToolbarColorStyle, ToolbarOrientation, docked_toolbar, floating_toolbar},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, provide_context, remember, tessera, use_context,
};

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, spec_table_cells, subheading, type_style,
};

const DO_COLOR: Color = Color::from_rgb(0.121, 0.478, 0.196);
const DONT_COLOR: Color = Color::from_rgb(0.741, 0.161, 0.161);
const CAUTION_COLOR: Color = Color::from_rgb(0.62, 0.38, 0.0);

#[derive(Clone, Copy, Default, PartialEq)]
enum Variant {
    Docked,
    #[default]
    Floating,
    Vertical,
    WithFab,
}

#[tessera]
pub(super) fn toolbars_spec() {
    article(
        "Toolbars",
        "Toolbars display frequently used actions relevant to the current page",
        vec![
            SpecSection {
                title: "Variants",
                content: RenderSlot::new(spec_variants),
            },
            SpecSection {
                title: "Configurations",
                content: RenderSlot::new(spec_configurations),
            },
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Anatomy",
                content: RenderSlot::new(spec_anatomy),
            },
            SpecSection {
                title: "Color",
                content: RenderSlot::new(spec_color),
            },
            SpecSection {
                title: "Measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "Bottom app bar (baseline)",
                content: RenderSlot::new(baseline_specs),
            },
        ],
    )
    .pages(vec![
        ArticlePage {
            title: "Overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![
                SpecSection {
                    title: "Availability & resources",
                    content: RenderSlot::new(availability),
                },
                SpecSection {
                    title: "M3 Expressive update",
                    content: RenderSlot::new(expressive_update),
                },
                SpecSection {
                    title: "Differences from M2",
                    content: RenderSlot::new(differences),
                },
            ],
        },
        ArticlePage {
            title: "Guidelines",
            intro: Some(RenderSlot::new(|| {
                gallery(GalleryKind::UseCases);
                gap(Dp(16.0));
                caption("Toolbars can be used for a wide variety of use cases");
            })),
            sections: vec![
                SpecSection {
                    title: "Usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "Anatomy",
                    content: RenderSlot::new(guide_anatomy),
                },
                SpecSection {
                    title: "Position & orientation",
                    content: RenderSlot::new(position),
                },
                SpecSection {
                    title: "Adaptive design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "Behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "Accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "Use cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "Interaction & style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "Keyboard navigation",
                    content: RenderSlot::new(keyboard),
                },
            ],
        },
    ]);
}

#[tessera]
fn paragraph(content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn toolbar_control(
    index: usize,
    vibrant: bool,
    selected: State<bool>,
    notice: State<String>,
    geometry: Option<State<Geometry>>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let (label, vector) = match index {
        0 => ("Select", filled::CHECK_BOX_SVG),
        1 => ("Brush", filled::BRUSH_SVG),
        2 => ("Record", filled::MIC_SVG),
        _ => ("Add image", filled::IMAGE_SVG),
    };
    if index == 0 {
        icon_button()
            .modifier(
                geometry
                    .map(|geometry| watch(geometry, "select"))
                    .unwrap_or_default()
                    .semantics(SemanticsArgs {
                        test_tag: Some("spec-toolbar-action-0".into()),
                        merge_descendants: true,
                        ..Default::default()
                    }),
            )
            .icon(vector)
            .accessibility_label(label)
            .on_click(move || {
                selected.set(!selected.get());
                notice.set(format!("{label} activated"));
            })
            .filled_tonal()
            .selected(selected.get())
            .color(if vibrant {
                scheme.surface_container
            } else {
                scheme.secondary_container
            })
            .content_color(if vibrant {
                scheme.on_surface
            } else {
                scheme.on_secondary_container
            });
    } else if index == 3 {
        icon_button()
            .modifier(
                geometry
                    .map(|geometry| watch(geometry, ["select", "brush", "record", "image"][index]))
                    .unwrap_or_default()
                    .semantics(SemanticsArgs {
                        test_tag: Some(format!("spec-toolbar-action-{index}")),
                        merge_descendants: true,
                        ..Default::default()
                    }),
            )
            .icon(vector)
            .accessibility_label(label)
            .on_click(move || {
                notice.set(format!("{label} activated"));
            })
            .filled();
    } else {
        icon_button()
            .modifier(
                geometry
                    .map(|geometry| watch(geometry, ["select", "brush", "record", "image"][index]))
                    .unwrap_or_default()
                    .semantics(SemanticsArgs {
                        test_tag: Some(format!("spec-toolbar-action-{index}")),
                        merge_descendants: true,
                        ..Default::default()
                    }),
            )
            .icon(vector)
            .accessibility_label(label)
            .on_click(move || {
                notice.set(format!("{label} activated"));
            })
            .standard()
            .content_color(if vibrant {
                scheme.on_primary_container
            } else {
                scheme.on_surface_variant
            });
    }
}

#[tessera]
fn toolbar_content(
    variant: Variant,
    color: ToolbarColorStyle,
    geometry: Option<State<Geometry>>,
    expanded: bool,
    notice: State<String>,
) {
    let selected = remember(|| true);
    let modifier = geometry
        .map(|geometry| watch(geometry, "container"))
        .unwrap_or_default()
        .semantics(SemanticsArgs {
            test_tag: Some("spec-toolbar-container".into()),
            merge_descendants: true,
            ..Default::default()
        });
    let vibrant = color == ToolbarColorStyle::Vibrant;
    if variant == Variant::Docked {
        docked_toolbar()
            .modifier(modifier.fill_max_width())
            .color_style(color)
            .accessibility_label("Docked editing actions")
            .content(move || {
                for index in 0..4 {
                    toolbar_control(index, vibrant, selected, notice).geometry_optional(geometry);
                }
            });
    } else {
        floating_toolbar()
            .modifier(modifier)
            .color_style(color)
            .orientation(if variant == Variant::Vertical {
                ToolbarOrientation::Vertical
            } else {
                ToolbarOrientation::Horizontal
            })
            .expanded(expanded)
            .accessibility_label("Floating editing actions")
            .content(move || {
                toolbar_control(0, vibrant, selected, notice).geometry_optional(geometry);
            })
            .trailing(move || {
                for index in 1..4 {
                    toolbar_control(index, vibrant, selected, notice).geometry_optional(geometry);
                }
            });
    }
}

#[tessera]
fn baseline_figure() {
    let geometry = remember(Geometry::new);
    let notice = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(240.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                    Dp(24.0),
                    Dp(80.0),
                    Dp(24.0),
                    Dp::ZERO,
                )))
                .children(move || {
                    surface()
                        .style(SurfaceStyle::Filled {
                            color: scheme.surface_container,
                        })
                        .modifier(
                            watch(geometry, "container")
                                .fill_max_width()
                                .height(Dp(80.0))
                                .semantics(SemanticsArgs {
                                    role: Some(Role::Toolbar),
                                    label: Some("Baseline bottom app bar".into()),
                                    test_tag: Some("baseline-bottom-bar".into()),
                                    ..Default::default()
                                }),
                        )
                        .child(move || {
                            row()
                                .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                                    Dp(4.0),
                                    Dp::ZERO,
                                    Dp(16.0),
                                    Dp::ZERO,
                                )))
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .children(move || {
                                    for (index, (label, vector)) in [
                                        ("Select", filled::CHECK_SVG),
                                        ("Search", filled::SEARCH_SVG),
                                        ("Record", filled::MIC_SVG),
                                        ("Image", filled::IMAGE_SVG),
                                    ]
                                    .into_iter()
                                    .enumerate()
                                    {
                                        icon_button()
                                            .modifier(
                                                watch(
                                                    geometry,
                                                    [
                                                        "action-0", "action-1", "action-2",
                                                        "action-3",
                                                    ][index],
                                                )
                                                .semantics(SemanticsArgs {
                                                    test_tag: Some(format!(
                                                        "baseline-action-{index}"
                                                    )),
                                                    ..Default::default()
                                                }),
                                            )
                                            .standard()
                                            .icon(vector)
                                            .accessibility_label(format!("Baseline {label}"))
                                            .on_click(move || {
                                                notice.set(format!("Baseline {label} activated"))
                                            });
                                    }
                                    spacer().modifier(Modifier::new().weight(1.0));
                                    floating_action_button()
                                        .modifier(watch(geometry, "fab").semantics(SemanticsArgs {
                                            test_tag: Some("baseline-fab".into()),
                                            ..Default::default()
                                        }))
                                        .accessibility_label("Baseline create")
                                        .on_click(move || {
                                            notice.set("Baseline create activated".into())
                                        })
                                        .content(|| {
                                            icon().vector(filled::ADD_SVG);
                                        });
                                });
                        });
                });
            baseline_annotations(geometry);
        });
    if !notice.get().is_empty() {
        paragraph(notice.get());
    }
}

#[tessera]
fn baseline_annotations(geometry: State<Geometry>) {
    let values = geometry.get();
    let (Some(root), Some(container), Some(first), Some(fab)) = (
        values.get("figure"),
        relative(&values, "container"),
        relative(&values, "action-0"),
        relative(&values, "fab"),
    ) else {
        return;
    };
    let mut svg = String::new();
    dotted(
        &mut svg,
        container.x - 8.0,
        container.y,
        container.h,
        true,
        "#b14e70",
    );
    for (value, x, y) in [
        (container.h, container.x - 24.0, container.y + 30.0),
        (first.w, first.x, first.y - 40.0),
        (
            (first.x - container.x).max(0.0),
            container.x,
            container.y + container.h + 16.0,
        ),
        (
            (container.x + container.w - fab.x - fab.w).max(0.0),
            fab.x + fab.w,
            container.y + container.h + 16.0,
        ),
        (
            (fab.y - container.y).max(0.0),
            fab.x + fab.w + 4.0,
            container.y - 28.0,
        ),
    ] {
        annotations::dimension(format!("{value:.0}"), Dp(x), Dp(y));
    }
    annotations::outline(&mut svg, first);
    annotations::outline(&mut svg, fab);
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn figure(variant: Variant, color: ToolbarColorStyle, measurements: bool) {
    let geometry = remember(Geometry::new);
    let notice = remember(String::new);
    let width = remember(|| Dp::ZERO);
    let compact = width.get() < Dp(400.0);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    watch(geometry, "figure")
                        .on_layout_coordinates(move |coordinates: LayoutCoordinates| {
                            let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
                            if width.get() != next {
                                width.set(next);
                            }
                        })
                        .fill_max_width()
                        .height(Dp(if variant == Variant::Vertical {
                            384.0
                        } else {
                            280.0
                        }))
                        .background_with_shape(
                            scheme.surface_container_low,
                            Shape::rounded_rectangle(Dp(24.0)),
                        )
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            test_tag: Some("spec-toolbar-figure".into()),
                            merge_descendants: true,
                            label: Some(format!(
                                "Toolbar {} figure",
                                if measurements {
                                    "measurement"
                                } else {
                                    "anatomy"
                                }
                            )),
                            ..Default::default()
                        }),
                )
                .children(move || {
                    row()
                        .modifier(
                            Modifier::new()
                                .align(Alignment::Center)
                                .fill_max_width()
                                .padding_symmetric(Dp(if compact { 16.0 } else { 48.0 }), Dp(48.0)),
                        )
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            if variant == Variant::Docked {
                                toolbar_content(variant, color, true, notice).geometry(geometry);
                            } else {
                                boxed()
                                    .alignment(Alignment::Center)
                                    .modifier(Modifier::new().fill_max_width())
                                    .children(move || {
                                        row()
                                            .cross_axis_alignment(CrossAxisAlignment::Center)
                                            .children(move || {
                                                toolbar_content(variant, color, true, notice)
                                                    .geometry(geometry);
                                                if variant == Variant::WithFab {
                                                    spacer()
                                                        .modifier(Modifier::new().width(Dp(8.0)));
                                                    floating_action_button()
                                                        .modifier(watch(geometry, "fab").semantics(
                                                            SemanticsArgs {
                                                                test_tag: Some(
                                                                    "spec-toolbar-fab".into(),
                                                                ),
                                                                ..Default::default()
                                                            },
                                                        ))
                                                        .accessibility_label("Create item")
                                                        .elevation(
                                                            FloatingActionButtonElevation::new(
                                                                Dp(1.0),
                                                                Dp(1.0),
                                                                Dp(3.0),
                                                                Dp(1.0),
                                                            ),
                                                        )
                                                        .on_click(move || {
                                                            notice
                                                                .set("Create item activated".into())
                                                        })
                                                        .content(|| {
                                                            icon()
                                                                .vector(filled::ADD_SVG)
                                                                .size(Dp(24.0));
                                                        });
                                                }
                                            });
                                    });
                            }
                        });
                    toolbar_annotations(geometry, measurements, variant);
                });
            if !measurements {
                gap(Dp(16.0));
                legend(vec!["Container".into(), "Placed components".into()]);
            }
            if !notice.get().is_empty() {
                text().content(notice.get()).style(type_style(14.0, 20.0));
            }
        });
}

#[tessera]
fn toolbar_annotations(geometry: State<Geometry>, measurements: bool, variant: Variant) {
    let values = geometry.get();
    let (Some(root), Some(container)) = (values.get("figure"), relative(&values, "container"))
    else {
        return;
    };
    let width = Dp::from_pixels_f32(root.width.to_f32());
    let height = Dp::from_pixels_f32(root.height.to_f32());
    let mut svg = String::new();
    if measurements {
        let x = container.x - 16.0;
        dotted(&mut svg, x, container.y, container.h, true, "#b14e70");
        annotations::dimension(
            format!(
                "{:.0}",
                if variant == Variant::Vertical {
                    container.w
                } else {
                    container.h
                }
            ),
            Dp(if variant == Variant::Vertical {
                container.x + 20.0
            } else {
                x - 28.0
            }),
            Dp(if variant == Variant::Vertical {
                container.y - 36.0
            } else {
                container.y + 22.0
            }),
        );
        if variant == Variant::Vertical {
            dotted(
                &mut svg,
                container.x,
                container.y - 12.0,
                container.w,
                false,
                "#b14e70",
            );
        }
        if variant != Variant::Vertical {
            let controls = ["select", "brush", "record", "image"]
                .into_iter()
                .filter_map(|key| relative(&values, key))
                .collect::<Vec<_>>();
            let mut spaces = Vec::new();
            if let (Some(first), Some(last)) = (controls.first(), controls.last()) {
                spaces.push((container.x, first.x - container.x));
                for pair in controls.windows(2) {
                    let start = pair[0].x + pair[0].w;
                    spaces.push((start, pair[1].x - start));
                }
                spaces.push((last.x + last.w, container.x + container.w - last.x - last.w));
            }
            for (x, padding) in spaces.into_iter().filter(|(_, padding)| *padding > 0.0) {
                svg.push_str(&format!("<rect x='{x}' y='{}' width='{padding}' height='{}' fill='#e883aa' fill-opacity='.32'/>", container.y, container.h));
                dotted(&mut svg, x, container.y - 16.0, 16.0, true, "#b14e70");
                annotations::dimension(format!("{padding:.0}"), Dp(x), Dp(container.y - 40.0));
            }
        }
        if let Some(fab) = relative(&values, "fab") {
            dotted(
                &mut svg,
                container.x + container.w,
                container.y + container.h + 16.0,
                fab.x - container.x - container.w,
                false,
                "#b14e70",
            );
            annotations::dimension(
                format!("{:.0}", fab.x - container.x - container.w),
                Dp(container.x + container.w),
                Dp(container.y + container.h + 24.0),
            );
            dotted(
                &mut svg,
                fab.x + fab.w + 12.0,
                fab.y,
                fab.h,
                true,
                "#b14e70",
            );
            annotations::dimension(
                format!("{:.0}", fab.h),
                Dp(fab.x + fab.w + 16.0),
                Dp(fab.y + 16.0),
            );
        }
    } else {
        annotations::outline(&mut svg, container);
        dotted(
            &mut svg,
            container.x - 20.0,
            container.y + 16.0,
            20.0,
            false,
            "#49454f",
        );
        annotations::marker(1, Dp(container.x - 44.0), Dp(container.y));
        dotted(
            &mut svg,
            container.x + container.w / 2.0,
            container.y + container.h,
            28.0,
            true,
            "#49454f",
        );
        annotations::marker(
            2,
            Dp(container.x + container.w / 2.0 - 12.0),
            Dp(container.y + container.h + 28.0),
        );
    }
    annotations::lines(width, height, svg);
}

#[tessera]
fn prose(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                let (indent, item) = match item.strip_prefix("- ") {
                    Some(nested) => (Dp(24.0), nested.to_string()),
                    None => (Dp::ZERO, item),
                };
                text()
                    .content(format!("•  {item}"))
                    .modifier(Modifier::new().padding(Padding::new(
                        indent,
                        Dp::ZERO,
                        Dp::ZERO,
                        Dp::ZERO,
                    )))
                    .style(type_style(16.0, 24.0));
                gap(Dp(8.0));
            }
        });
}

#[tessera]
fn number_badge(number: usize) {
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(24.0), Dp(24.0))
                .background_with_shape(INK, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(12.0, 16.0))
                .color(Color::WHITE);
        });
}

#[tessera]
fn legend(items: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for (index, label) in items.clone().into_iter().enumerate() {
                row()
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .padding_symmetric(Dp::ZERO, Dp(4.0)),
                    )
                    .children(move || {
                        number_badge(index + 1);
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                        text()
                            .content(label.clone())
                            .modifier(Modifier::new().weight(1.0))
                            .style(type_style(14.0, 20.0));
                    });
            }
        });
}

#[tessera]
fn guidance(#[prop(into)] mark: String, #[prop(into)] note: String, color: Color) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            let mark = mark.clone();
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .padding(Padding::symmetric(Dp(12.0), Dp(4.0)))
                        .background_with_shape(color, Shape::rounded_rectangle(Dp(8.0))),
                )
                .children(move || {
                    text()
                        .content(mark.clone())
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text()
                .content(note.clone())
                .modifier(Modifier::new().weight(1.0))
                .color(scheme.on_surface_variant)
                .style(type_style(14.0, 20.0));
        });
}

#[tessera]
fn do_note(#[prop(into)] note: String) {
    guidance("Do", note, DO_COLOR);
}

#[tessera]
fn dont_note(#[prop(into)] note: String) {
    guidance("Don't", note, DONT_COLOR);
}

#[tessera]
fn caution_note(#[prop(into)] note: String) {
    guidance("Caution", note, CAUTION_COLOR);
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Emphasis {
    #[default]
    Standard,
    Filled,
    Toggle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Sample {
    #[default]
    Mail,
    Format,
    FormatTonal,
    Navigate,
    Edit,
    Vertical,
    History,
    ColorRoles,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SampleAction {
    label: &'static str,
    emphasis: Emphasis,
}

const fn action(label: &'static str, emphasis: Emphasis) -> SampleAction {
    SampleAction { label, emphasis }
}

fn sample_actions(sample: Sample) -> Vec<SampleAction> {
    use Emphasis::{Filled, Standard, Toggle};
    match sample {
        Sample::Mail => vec![
            action("Archive", Standard),
            action("Delete", Standard),
            action("Mark unread", Standard),
            action("Snooze", Standard),
            action("Star", Standard),
        ],
        Sample::Format => vec![
            action("Bold", Toggle),
            action("Italic", Standard),
            action("Underline", Standard),
            action("Text color", Standard),
            action("Highlight", Standard),
            action("More options", Standard),
        ],
        Sample::FormatTonal => vec![
            action("Bold", Toggle),
            action("Italic", Toggle),
            action("Underline", Standard),
            action("Text color", Standard),
            action("More options", Standard),
        ],
        Sample::Navigate => vec![
            action("Back", Standard),
            action("Forward", Standard),
            action("Add", Filled),
            action("Picture in picture", Standard),
            action("More options", Standard),
        ],
        Sample::Edit => vec![
            action("Camera off", Standard),
            action("Microphone", Standard),
            action("Raise hand", Toggle),
            action("More options", Standard),
        ],
        Sample::Vertical => vec![
            action("Undo", Standard),
            action("Redo", Standard),
            action("Add", Standard),
            action("Text color", Standard),
            action("More options", Standard),
        ],
        Sample::History => vec![action("Undo", Standard), action("Redo", Standard)],
        Sample::ColorRoles => vec![
            action("Add", Filled),
            action("Bold", Toggle),
            action("Italic", Standard),
            action("Underline", Standard),
        ],
    }
}

fn action_icon(label: &str) -> tessera_components::material_icons::Asset {
    match label {
        "Archive" => filled::ARCHIVE_SVG,
        "Delete" => filled::DELETE_SVG,
        "Mark unread" => filled::MARK_EMAIL_UNREAD_SVG,
        "Snooze" => filled::SNOOZE_SVG,
        "Star" => filled::STAR_SVG,
        "Bold" => filled::FORMAT_BOLD_SVG,
        "Italic" => filled::FORMAT_ITALIC_SVG,
        "Underline" => filled::FORMAT_UNDERLINED_SVG,
        "Text color" => filled::FORMAT_COLOR_TEXT_SVG,
        "Highlight" => filled::FORMAT_COLOR_FILL_SVG,
        "Back" => filled::ARROW_BACK_SVG,
        "Forward" => filled::ARROW_FORWARD_SVG,
        "Add" => filled::ADD_SVG,
        "Picture in picture" => filled::PICTURE_IN_PICTURE_SVG,
        "Camera off" => filled::VIDEOCAM_OFF_SVG,
        "Microphone" => filled::MIC_SVG,
        "Raise hand" => filled::BACK_HAND_SVG,
        "Mail" => filled::MAIL_SVG,
        "Chat" => filled::PEOPLE_SVG,
        "Settings" => filled::SETTINGS_SVG,
        "Reply" => filled::REPLY_SVG,
        "Undo" => filled::UNDO_SVG,
        "Redo" => filled::REDO_SVG,
        _ => filled::MORE_VERT_SVG,
    }
}

/// Keeps the leading actions and a trailing overflow action within `limit`.
fn visible_actions(actions: Vec<SampleAction>, limit: usize) -> Vec<SampleAction> {
    let limit = limit.max(1);
    if actions.len() <= limit {
        return actions;
    }
    match actions.last().copied() {
        Some(last) if last.label == "More options" => {
            let mut kept = actions[..limit - 1].to_vec();
            kept.push(last);
            kept
        }
        _ => actions[..limit].to_vec(),
    }
}

/// Number of 48dp actions with 4dp gaps that fit inside `width` after
/// `padding`.
fn actions_that_fit(width: Dp, padding: f64) -> usize {
    (((width.0 - padding + 4.0) / 52.0).floor().max(2.0)) as usize
}

fn sample_label(sample: Sample) -> &'static str {
    match sample {
        Sample::Mail => "Mail actions",
        Sample::Format | Sample::FormatTonal => "Formatting actions",
        Sample::Navigate => "Browsing actions",
        Sample::Edit => "Call actions",
        Sample::Vertical => "Canvas actions",
        Sample::History => "History actions",
        Sample::ColorRoles => "Color role actions",
    }
}

/// Identifies the gallery item that renders a sample so its controls can be
/// located.
#[derive(Clone)]
struct GalleryTag(usize);

fn tagged(tag: Option<String>) -> Modifier {
    match tag {
        Some(tag) => Modifier::new().semantics(SemanticsArgs {
            test_tag: Some(tag),
            merge_descendants: true,
            ..Default::default()
        }),
        None => Modifier::new(),
    }
}

#[tessera]
fn sample_action(
    #[prop(into)] label: String,
    emphasis: Emphasis,
    vibrant: bool,
    selected: State<bool>,
    notice: State<String>,
    tag: Option<String>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let vector = action_icon(&label);
    let message = format!("{label} activated");
    let modifier = tagged(tag);
    match emphasis {
        Emphasis::Toggle => {
            icon_button()
                .modifier(modifier)
                .icon(vector)
                .accessibility_label(label.clone())
                .on_click(move || {
                    selected.set(!selected.get());
                    notice.set(message.clone());
                })
                .filled_tonal()
                .selected(selected.get())
                .color(if vibrant {
                    scheme.surface_container
                } else {
                    scheme.secondary_container
                })
                .content_color(if vibrant {
                    scheme.on_surface
                } else {
                    scheme.on_secondary_container
                });
        }
        Emphasis::Filled => {
            icon_button()
                .modifier(modifier)
                .icon(vector)
                .accessibility_label(label.clone())
                .on_click(move || notice.set(message.clone()))
                .filled();
        }
        Emphasis::Standard => {
            icon_button()
                .modifier(modifier)
                .icon(vector)
                .accessibility_label(label.clone())
                .on_click(move || notice.set(message.clone()))
                .standard()
                .content_color(if vibrant {
                    scheme.on_primary_container
                } else {
                    scheme.on_surface_variant
                });
        }
    }
}

#[tessera]
fn sample_toolbar(
    sample: Sample,
    docked: Option<bool>,
    vibrant: Option<bool>,
    vertical: Option<bool>,
    rounded: Option<bool>,
    dividers: Option<bool>,
    elevation: Option<Dp>,
    limit: Option<usize>,
    notice: Option<State<String>>,
) {
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    let selected = remember(|| true);
    let docked = docked.unwrap_or(false);
    let vibrant = vibrant.unwrap_or(false);
    let vertical = vertical.unwrap_or(false);
    let dividers = dividers.unwrap_or(false);
    let actions = visible_actions(sample_actions(sample), limit.unwrap_or(usize::MAX));
    let gallery_tag = use_context::<GalleryTag>().map(|tag| tag.get().0);
    let color = if vibrant {
        ToolbarColorStyle::Vibrant
    } else {
        ToolbarColorStyle::Standard
    };
    let content = RenderSlot::new(move || {
        let count = actions.len();
        for (index, action) in actions.clone().into_iter().enumerate() {
            if dividers && index > 0 && index % 3 == 0 {
                vertical_divider().modifier(Modifier::new().height(Dp(32.0)));
            }
            let tag = gallery_tag.and_then(|item| match index {
                0 => Some(format!("toolbar-gallery-{item}-first")),
                last if last + 1 == count => Some(format!("toolbar-gallery-{item}-last")),
                _ => None,
            });
            sample_action(action.label, action.emphasis, vibrant, selected, notice)
                .tag_optional(tag);
        }
    });
    if docked {
        let modifier = if rounded.unwrap_or(false) {
            Modifier::new().clip_shape(Shape::rounded_rectangle(Dp(16.0)))
        } else {
            Modifier::new()
        };
        docked_toolbar()
            .modifier(modifier)
            .color_style(color)
            .accessibility_label(sample_label(sample))
            .content_shared(content);
    } else {
        floating_toolbar()
            .color_style(color)
            .orientation(if vertical {
                ToolbarOrientation::Vertical
            } else {
                ToolbarOrientation::Horizontal
            })
            .elevation(elevation.unwrap_or(Dp(6.0)))
            .accessibility_label(sample_label(sample))
            .content_shared(content);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum FabKind {
    #[default]
    Reply,
    Add,
    EndCall,
}

#[tessera]
fn fab_pair(
    sample: Sample,
    fab: FabKind,
    vibrant: Option<bool>,
    limit: Option<usize>,
    notice: Option<State<String>>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let vibrant = vibrant.unwrap_or(false);
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    let fab_tag = use_context::<GalleryTag>()
        .map(|tag| tag.get().0)
        .map(|item| format!("toolbar-gallery-{item}-fab"));
    let (label, container, content) = match fab {
        FabKind::EndCall => (
            "End call",
            scheme.error_container,
            scheme.on_error_container,
        ),
        FabKind::Reply if vibrant => (
            "Reply",
            scheme.tertiary_container,
            scheme.on_tertiary_container,
        ),
        FabKind::Add if vibrant => (
            "Create",
            scheme.tertiary_container,
            scheme.on_tertiary_container,
        ),
        FabKind::Reply => (
            "Reply",
            scheme.secondary_container,
            scheme.on_secondary_container,
        ),
        FabKind::Add => (
            "Create",
            scheme.secondary_container,
            scheme.on_secondary_container,
        ),
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            sample_toolbar(sample)
                .vibrant(vibrant)
                .limit_optional(limit)
                .notice(notice);
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            floating_action_button()
                .modifier(tagged(fab_tag.clone()))
                .size(FloatingActionButtonSize::Standard)
                .container_color(container)
                .content_color(content)
                .accessibility_label(label)
                .elevation(FloatingActionButtonElevation::new(
                    Dp(1.0),
                    Dp(1.0),
                    Dp(3.0),
                    Dp(1.0),
                ))
                .on_click(move || notice.set(format!("{label} activated")))
                .content(move || {
                    icon()
                        .vector(match fab {
                            FabKind::Reply => filled::REPLY_SVG,
                            FabKind::Add => filled::ADD_SVG,
                            FabKind::EndCall => filled::CALL_END_SVG,
                        })
                        .size(Dp(24.0));
                });
        });
}

/// Records the measured width of a stage so its content can adapt.
fn watch_width(modifier: Modifier, width: State<Dp>) -> Modifier {
    modifier.on_layout_coordinates(move |coordinates: LayoutCoordinates| {
        let next = Dp::from_pixels_f32(coordinates.local_size.width.to_f32());
        if width.get() != next {
            width.set(next);
        }
    })
}

fn compact(width: Dp) -> bool {
    width < Dp(400.0)
}

#[tessera]
fn stage(width: State<Dp>, content: RenderSlot) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let padding = if compact(width.get()) { 16.0 } else { 40.0 };
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            watch_width(Modifier::new(), width)
                .fill_max_width()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    test_tag: Some("toolbar-stage".into()),
                    merge_descendants: true,
                    ..Default::default()
                })
                .padding_symmetric(Dp(padding), Dp(32.0)),
        )
        .children(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children_shared(content);
        });
}

#[tessera]
fn gallery_item(
    index: usize,
    legend: Option<usize>,
    fill: Option<bool>,
    content: Option<RenderSlot>,
) {
    let fill = fill.unwrap_or(false);
    let modifier = Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(format!("toolbar-gallery-{index}")),
        ..Default::default()
    });
    column()
        .modifier(if fill {
            modifier.fill_max_width()
        } else {
            modifier
        })
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            if let Some(content) = content {
                provide_context(move || GalleryTag(index), move || content.render());
            }
            if let Some(number) = legend {
                gap(Dp(12.0));
                number_badge(number);
            }
        });
}

#[tessera]
fn floating_group(content: RenderSlot) {
    flow_row()
        .modifier(Modifier::new().fill_max_width())
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children_shared(content);
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum GalleryKind {
    #[default]
    Overview,
    Variants,
    Configurations,
    Expressive,
    UseCases,
    DockedVsFloating,
    ColorEmphasis,
    Corners,
    Elevation,
    Slots,
    Emphasis,
    EmphasisDont,
    Presentation,
    RoundedDocked,
}

#[tessera]
fn gallery(kind: GalleryKind) {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    let inner = Dp(width.get().0 - if compact(width.get()) { 32.0 } else { 80.0 });
    let limit = actions_that_fit(inner, 16.0);
    let docked_limit = actions_that_fit(inner, 32.0);
    let fab_limit = actions_that_fit(Dp(inner.0 - 64.0), 16.0);
    stage(
        width,
        RenderSlot::new(move || match kind {
            GalleryKind::Overview => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move || {
                        sample_toolbar(Sample::Vertical)
                            .vertical(true)
                            .notice(notice);
                    });
                    gallery_item(1).content(move || {
                        fab_pair(Sample::Edit, FabKind::EndCall)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                    gallery_item(2).content(move || {
                        sample_toolbar(Sample::FormatTonal)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(3).content(move || {
                        sample_toolbar(Sample::Navigate).limit(limit).notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(4).fill(true).content(move || {
                    sample_toolbar(Sample::Navigate)
                        .docked(true)
                        .vibrant(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(5).fill(true).content(move || {
                    text_nav_toolbar(false).notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(6).fill(true).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .vibrant(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
            }
            GalleryKind::Variants => {
                gallery_item(0).fill(true).legend(1).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(32.0));
                gallery_item(1).legend(2).content(move || {
                    sample_toolbar(Sample::Format).limit(limit).notice(notice);
                });
            }
            GalleryKind::Configurations => {
                gallery_item(0).content(move || {
                    sample_toolbar(Sample::Format).limit(limit).notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(1).content(move || {
                    sample_toolbar(Sample::Format)
                        .vibrant(true)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(2).fill(true).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(3).fill(true).legend(1).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .vibrant(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(32.0));
                floating_group(RenderSlot::new(move || {
                    gallery_item(4).legend(2).content(move || {
                        sample_toolbar(Sample::Format)
                            .vertical(true)
                            .limit(5)
                            .notice(notice);
                    });
                    gallery_item(5).legend(3).content(move || {
                        fab_pair(Sample::Mail, FabKind::Reply)
                            .vibrant(true)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::Expressive => {
                gallery_item(0).legend(1).content(move || {
                    fab_pair(Sample::Mail, FabKind::Reply)
                        .vibrant(true)
                        .limit(fab_limit)
                        .notice(notice);
                });
                gap(Dp(32.0));
                gallery_item(1).fill(true).legend(2).content(move || {
                    sample_toolbar(Sample::Navigate)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
            }
            GalleryKind::UseCases => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move || {
                        sample_toolbar(Sample::Mail)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(1).content(move || {
                        sample_toolbar(Sample::Format).limit(limit).notice(notice);
                    });
                    gallery_item(2).content(move || {
                        fab_pair(Sample::Edit, FabKind::EndCall)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                    gallery_item(3).content(move || {
                        sample_toolbar(Sample::Navigate)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(4).fill(true).content(move || {
                    text_nav_toolbar(false).notice(notice);
                });
            }
            GalleryKind::DockedVsFloating => {
                gallery_item(0).fill(true).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption("Docked toolbar shows global controls");
                gap(Dp(32.0));
                gallery_item(1).content(move || {
                    sample_toolbar(Sample::Format).limit(limit).notice(notice);
                });
                gap(Dp(12.0));
                caption("Floating toolbar shows controls relevant to the current page");
            }
            GalleryKind::ColorEmphasis => {
                gallery_item(0).content(move || {
                    sample_toolbar(Sample::Format).limit(limit).notice(notice);
                });
                gap(Dp(12.0));
                caption(
                    "Use the standard color scheme to draw focus to content outside the toolbar",
                );
                gap(Dp(32.0));
                gallery_item(1).content(move || {
                    sample_toolbar(Sample::Format)
                        .vibrant(true)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption("Use the vibrant color scheme to emphasize controls or actions");
            }
            GalleryKind::Corners => {
                gallery_item(0).fill(true).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                do_note("Use straight corners for docked toolbars");
                gap(Dp(32.0));
                gallery_item(1).fill(true).content(move || {
                    sample_toolbar(Sample::Mail)
                        .docked(true)
                        .rounded(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                dont_note("Avoid modifying the container shape");
            }
            GalleryKind::Elevation => {
                gallery_item(0).content(move || {
                    sample_toolbar(Sample::Format)
                        .vibrant(true)
                        .elevation(Dp::ZERO)
                        .limit(limit)
                        .notice(notice);
                });
            }
            GalleryKind::Slots => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move || {
                        sample_toolbar(Sample::FormatTonal)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(1).content(move || {
                        sample_toolbar(Sample::Navigate)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(2).content(move || {
                        sample_toolbar(Sample::Edit).limit(limit).notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(3).fill(true).content(move || {
                    text_nav_toolbar(false).notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(4).fill(true).content(move || {
                    search_toolbar(notice);
                });
            }
            GalleryKind::Emphasis => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move || {
                        sample_toolbar(Sample::Navigate).limit(limit).notice(notice);
                    });
                    gallery_item(1).content(move || {
                        fab_pair(Sample::Mail, FabKind::Reply)
                            .vibrant(true)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::EmphasisDont => {
                gallery_item(0).content(move || {
                    fab_pair(Sample::Navigate, FabKind::Add)
                        .vibrant(true)
                        .limit(fab_limit)
                        .notice(notice);
                });
            }
            GalleryKind::Presentation => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move || {
                        sample_toolbar(Sample::History).notice(notice);
                    });
                    gallery_item(1).content(move || {
                        sample_toolbar(Sample::Format)
                            .vibrant(true)
                            .limit(limit.saturating_sub(2).max(2))
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::RoundedDocked => {
                gallery_item(0).fill(true).content(move || {
                    sample_toolbar(Sample::Format)
                        .docked(true)
                        .rounded(true)
                        .dividers(true)
                        .limit(docked_limit.saturating_sub(1).max(2))
                        .notice(notice);
                });
            }
        }),
    );
    if !notice.get().is_empty() {
        gap(Dp(12.0));
        caption(notice.get());
    }
}

#[tessera]
fn text_nav_toolbar(rtl: bool, notice: Option<State<String>>) {
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    let gallery_tag = use_context::<GalleryTag>().map(|tag| tag.get().0);
    docked_toolbar()
        .accessibility_label(if rtl {
            "Mirrored page navigation"
        } else {
            "Page navigation"
        })
        .content(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    nav_button(rtl, notice).tag_optional(
                        gallery_tag.map(|item| format!("toolbar-gallery-{item}-first")),
                    );
                    spacer().modifier(Modifier::new().weight(1.0));
                    nav_button(!rtl, notice).tag_optional(
                        gallery_tag.map(|item| format!("toolbar-gallery-{item}-last")),
                    );
                });
        });
}

#[tessera]
fn nav_button(next: bool, notice: State<String>, tag: Option<String>) {
    let label = if next { "Next" } else { "Back" };
    if next {
        button()
            .modifier(tagged(tag))
            .filled()
            .accessibility_label(label)
            .on_click(move || notice.set(format!("{label} activated")))
            .child(move || {
                text().content(label);
            });
    } else {
        button()
            .modifier(tagged(tag))
            .text()
            .accessibility_label(label)
            .on_click(move || notice.set(format!("{label} activated")))
            .child(move || {
                text().content(label);
            });
    }
}

#[tessera]
fn color_roles(vibrant: bool) {
    let width = remember(|| Dp::ZERO);
    stage(
        width,
        RenderSlot::new(move || {
            sample_toolbar(Sample::ColorRoles).vibrant(vibrant);
        }),
    );
    gap(Dp(16.0));
    legend(if vibrant {
        vec![
            "Primary container".into(),
            "Filled button (Primary, On primary)".into(),
            "Toggle tonal button (Surface container, On surface)".into(),
            "Standard button (On primary container)".into(),
        ]
    } else {
        vec![
            "Surface container".into(),
            "Filled button (Primary, On primary)".into(),
            "Toggle tonal button (Secondary container, On secondary container)".into(),
            "Standard button (On surface variant)".into(),
        ]
    });
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Placement {
    #[default]
    Docked,
    Horizontal,
    Vertical,
}

fn placement_tag(geometry: State<Geometry>, name: &'static str, tag: &str) -> Modifier {
    watch(geometry, name).semantics(SemanticsArgs {
        test_tag: Some(tag.into()),
        ..Default::default()
    })
}

#[tessera]
fn placement_figure(placement: Placement) {
    let geometry = remember(Geometry::new);
    let width = remember(|| Dp::ZERO);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let window_width = Dp(width.get().0 - 48.0);
    let horizontal_limit = actions_that_fit(Dp(window_width.0 - 32.0), 16.0).min(4);
    let docked_limit = actions_that_fit(window_width, 32.0);
    let vertical_limit = if compact(width.get()) { 3 } else { 5 };
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch_width(watch(geometry, "figure"), width)
                .fill_max_width()
                .height(Dp(360.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        match placement {
                            Placement::Docked => "Docked toolbar at the bottom of the window",
                            Placement::Horizontal => "Horizontal floating toolbar margins",
                            Placement::Vertical => "Vertical floating toolbar margins",
                        }
                        .into(),
                    ),
                    test_tag: Some("placement-figure".into()),
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .modifier(Modifier::new().fill_max_size().padding_all(Dp(24.0)))
                .children(move || {
                    boxed()
                        .modifier(
                            placement_tag(geometry, "window", "placement-window")
                                .fill_max_size()
                                .clip_shape(Shape::rounded_rectangle(Dp(16.0)))
                                .background_with_shape(
                                    scheme.surface,
                                    Shape::rounded_rectangle(Dp(16.0)),
                                ),
                        )
                        .children(move || match placement {
                            Placement::Docked => {
                                boxed()
                                    .modifier(
                                        placement_tag(geometry, "toolbar", "placement-toolbar")
                                            .align(Alignment::BottomCenter)
                                            .fill_max_width(),
                                    )
                                    .children(move || {
                                        sample_toolbar(Sample::Mail)
                                            .docked(true)
                                            .limit(docked_limit);
                                    });
                            }
                            Placement::Horizontal => {
                                boxed()
                                    .modifier(
                                        Modifier::new().align(Alignment::BottomCenter).padding(
                                            Padding::new(Dp::ZERO, Dp::ZERO, Dp::ZERO, Dp(16.0)),
                                        ),
                                    )
                                    .children(move || {
                                        boxed()
                                            .modifier(placement_tag(
                                                geometry,
                                                "toolbar",
                                                "placement-toolbar",
                                            ))
                                            .children(move || {
                                                sample_toolbar(Sample::Format)
                                                    .limit(horizontal_limit);
                                            });
                                    });
                            }
                            Placement::Vertical => {
                                column()
                                    .modifier(
                                        Modifier::new()
                                            .align(Alignment::CenterStart)
                                            .padding(Padding::new(
                                                Dp(16.0),
                                                Dp::ZERO,
                                                Dp::ZERO,
                                                Dp::ZERO,
                                            ))
                                            .semantics(SemanticsArgs {
                                                role: Some(Role::Group),
                                                label: Some("Navigation rail".into()),
                                                ..Default::default()
                                            }),
                                    )
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        for vector in
                                            [filled::INBOX_SVG, filled::STAR_SVG, filled::SEND_SVG]
                                        {
                                            icon().vector(vector).size(Dp(24.0));
                                            gap(Dp(24.0));
                                        }
                                    });
                                boxed()
                                    .modifier(Modifier::new().align(Alignment::CenterEnd).padding(
                                        Padding::new(Dp::ZERO, Dp::ZERO, Dp(24.0), Dp::ZERO),
                                    ))
                                    .children(move || {
                                        boxed()
                                            .modifier(placement_tag(
                                                geometry,
                                                "toolbar",
                                                "placement-toolbar",
                                            ))
                                            .children(move || {
                                                sample_toolbar(Sample::Vertical)
                                                    .vertical(true)
                                                    .limit(vertical_limit);
                                            });
                                    });
                            }
                        });
                });
            placement_annotations(geometry, placement);
        });
}

#[tessera]
fn placement_annotations(geometry: State<Geometry>, placement: Placement) {
    let values = geometry.get();
    let (Some(root), Some(window), Some(toolbar)) = (
        values.get("figure"),
        relative(&values, "window"),
        relative(&values, "toolbar"),
    ) else {
        return;
    };
    let mut svg = String::new();
    match placement {
        Placement::Docked => return,
        Placement::Horizontal => {
            let bottom = toolbar.y + toolbar.h;
            let margin = window.y + window.h - bottom;
            let x = toolbar.x + toolbar.w + 12.0;
            dotted(&mut svg, x, bottom, margin, true, "#b14e70");
            annotations::dimension(format!("{margin:.0}"), Dp(x + 6.0), Dp(bottom - 4.0));
        }
        Placement::Vertical => {
            let right = toolbar.x + toolbar.w;
            let margin = window.x + window.w - right;
            let y = toolbar.y - 12.0;
            dotted(&mut svg, right, y, margin, false, "#b14e70");
            annotations::dimension(format!("{margin:.0}"), Dp(right - 4.0), Dp(y - 28.0));
        }
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum DockedLayout {
    #[default]
    Edges,
    Centered,
}

#[tessera]
fn layout_action(index: usize, #[prop(into)] label: String, filled: bool, notice: State<String>) {
    let vector = action_icon(&label);
    let modifier = Modifier::new().semantics(SemanticsArgs {
        test_tag: Some(format!("layout-action-{index}")),
        ..Default::default()
    });
    let message = format!("{label} activated");
    if filled {
        icon_button()
            .modifier(modifier)
            .icon(vector)
            .accessibility_label(label.clone())
            .on_click(move || notice.set(message.clone()))
            .filled();
    } else {
        icon_button()
            .modifier(modifier)
            .icon(vector)
            .accessibility_label(label.clone())
            .on_click(move || notice.set(message.clone()))
            .standard();
    }
}

#[tessera]
fn docked_layout_figure(layout: DockedLayout) {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    let labels: Vec<&'static str> = if compact(width.get()) {
        vec!["Back", "Add", "More options"]
    } else {
        vec![
            "Back",
            "Forward",
            "Add",
            "Picture in picture",
            "More options",
        ]
    };
    stage(
        width,
        RenderSlot::new(move || {
            let labels = labels.clone();
            boxed()
                .modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {
                    test_tag: Some("docked-layout".into()),
                    ..Default::default()
                }))
                .children(move || {
                    let labels = labels.clone();
                    docked_toolbar()
                        .accessibility_label(match layout {
                            DockedLayout::Edges => "Edge-aligned docked toolbar",
                            DockedLayout::Centered => "Center-aligned docked toolbar",
                        })
                        .content(move || {
                            let labels = labels.clone();
                            match layout {
                                DockedLayout::Centered => {
                                    row()
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            for (index, label) in
                                                labels.clone().into_iter().enumerate()
                                            {
                                                if index > 0 {
                                                    spacer()
                                                        .modifier(Modifier::new().width(Dp(8.0)));
                                                }
                                                layout_action(index, label, label == "Add", notice);
                                            }
                                        });
                                }
                                DockedLayout::Edges => {
                                    row()
                                        .modifier(Modifier::new().fill_max_width())
                                        .cross_axis_alignment(CrossAxisAlignment::Center)
                                        .children(move || {
                                            for (index, label) in
                                                labels.clone().into_iter().enumerate()
                                            {
                                                let center = label == "Add";
                                                if center {
                                                    spacer().modifier(Modifier::new().weight(1.0));
                                                }
                                                layout_action(index, label, center, notice);
                                                if center {
                                                    spacer().modifier(Modifier::new().weight(1.0));
                                                }
                                            }
                                        });
                                }
                            }
                        });
                });
        }),
    );
    if !notice.get().is_empty() {
        gap(Dp(12.0));
        caption(notice.get());
    }
}

#[tessera]
fn overflow_example() {
    let open = remember(|| false);
    let notice = remember(String::new);
    let width = remember(|| Dp::ZERO);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    stage(
        width,
        RenderSlot::new(move || {
            floating_toolbar()
                .accessibility_label("Mailbox destinations")
                .content(move || {
                    for label in ["Inbox", "Starred"] {
                        button()
                            .text()
                            .accessibility_label(label)
                            .on_click(move || {
                                notice.set(format!("{label} selected"));
                                open.set(false);
                            })
                            .child(move || {
                                text().content(label);
                            });
                    }
                    icon_button()
                        .icon(filled::MORE_VERT_SVG)
                        .accessibility_label("More destinations")
                        .on_click(move || open.set(!open.get()))
                        .standard()
                        .selected(open.get());
                });
            if open.get() {
                gap(Dp(8.0));
                surface()
                    .style(SurfaceStyle::Filled {
                        color: scheme.surface_container,
                    })
                    .shape(Shape::rounded_rectangle(Dp(16.0)))
                    .elevation(Dp(3.0))
                    .accessibility_role(Role::Menu)
                    .accessibility_label("More destinations")
                    .child(move || {
                        column()
                            .modifier(Modifier::new().padding_symmetric(Dp::ZERO, Dp(8.0)))
                            .children(move || {
                                for label in ["Sent", "Drafts", "Spam"] {
                                    button()
                                        .text()
                                        .accessibility_label(label)
                                        .on_click(move || {
                                            notice.set(format!("{label} selected"));
                                            open.set(false);
                                        })
                                        .child(move || {
                                            text().content(label);
                                        });
                                }
                            });
                    });
            }
        }),
    );
    gap(Dp(12.0));
    caption(if notice.get().is_empty() {
        "Toolbar actions can open a menu".to_string()
    } else {
        notice.get()
    });
}

#[tessera]
fn overflow_collapse_example() {
    let compact_window = remember(|| true);
    let width = remember(|| Dp::ZERO);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .outlined()
                .accessibility_label("Toggle window size")
                .on_click(move || compact_window.set(!compact_window.get()))
                .child(move || {
                    text().content(if compact_window.get() {
                        "Show larger window"
                    } else {
                        "Show compact window"
                    });
                });
            gap(Dp(16.0));
            stage(
                width,
                RenderSlot::new(move || {
                    let fit = actions_that_fit(Dp(width.get().0 - 80.0), 16.0);
                    sample_toolbar(Sample::Format)
                        .vibrant(true)
                        .limit(if compact_window.get() { 3 } else { fit });
                }),
            );
        });
}

#[tessera]
fn rtl_example() {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    stage(
        width,
        RenderSlot::new(move || {
            text_nav_toolbar(false).notice(notice);
            gap(Dp(12.0));
            caption(
                "In LTR languages, the Next button is intentionally placed on the trailing (right) edge",
            );
            gap(Dp(24.0));
            text_nav_toolbar(true).notice(notice);
            gap(Dp(12.0));
            caption(
                "In RTL languages, reverse the order so Next remains on the trailing edge, now on the left",
            );
        }),
    );
}

#[tessera]
fn bottom_bar(icons: usize, fab: bool, notice: State<String>) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    surface()
        .style(SurfaceStyle::Filled {
            color: scheme.surface_container,
        })
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(80.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Toolbar),
                    label: Some(if fab {
                        "Bottom app bar with FAB".into()
                    } else {
                        "Bottom app bar".into()
                    }),
                    ..Default::default()
                }),
        )
        .child(move || {
            row()
                .modifier(Modifier::new().fill_max_size().padding(Padding::new(
                    Dp(4.0),
                    Dp::ZERO,
                    Dp(16.0),
                    Dp::ZERO,
                )))
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    for label in ["Archive", "Delete", "Mark unread", "Snooze"]
                        .into_iter()
                        .take(icons)
                    {
                        icon_button()
                            .icon(action_icon(label))
                            .accessibility_label(label)
                            .on_click(move || notice.set(format!("{label} activated")))
                            .standard();
                    }
                    if fab {
                        spacer().modifier(Modifier::new().weight(1.0));
                        floating_action_button()
                            .accessibility_label("Compose")
                            .on_click(move || notice.set("Compose activated".into()))
                            .content(|| {
                                icon().vector(filled::ADD_SVG);
                            });
                    }
                });
        });
}

#[tessera]
fn baseline_layouts() {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    stage(
        width,
        RenderSlot::new(move || {
            let icons = if compact(width.get()) { 2 } else { 4 };
            gallery_item(0).fill(true).legend(1).content(move || {
                bottom_bar(icons, true, notice);
            });
            gap(Dp(24.0));
            gallery_item(1).fill(true).legend(2).content(move || {
                bottom_bar(4, false, notice);
            });
        }),
    );
    gap(Dp(16.0));
    legend(vec![
        "Icon buttons and FAB".into(),
        "Icon buttons and no FAB".into(),
    ]);
    if !notice.get().is_empty() {
        gap(Dp(12.0));
        caption(notice.get());
    }
}

#[tessera]
fn overview() {
    bullets(vec![
        "Two expressive variants: docked toolbar and floating toolbar".into(),
        "Use the vibrant color style for greater emphasis".into(),
        "Can display a wide variety of control types, like buttons, icon buttons, and text fields"
            .into(),
        "Can be paired with FABs to emphasize certain actions".into(),
        "Don't show at the same time as a navigation bar".into(),
    ]);
    gap(Dp(32.0));
    gallery(GalleryKind::Overview);
    gap(Dp(16.0));
    caption("Configurations of floating toolbars");
}

#[tessera]
fn availability() {
    let rows = [
        ("Design", "Design Kit (Figma)", "Available"),
        (
            "Implementation",
            "Jetpack Compose - Bottom app bar",
            "Available",
        ),
        (
            "",
            "Jetpack Compose: Expressive - Docked toolbar",
            "Available",
        ),
        (
            "",
            "Jetpack Compose: Expressive - Floating toolbar",
            "Available",
        ),
        (
            "",
            "Android Views (MDC-Android) - Bottom app bar",
            "Available",
        ),
        (
            "",
            "Android Views (MDC-Android): Expressive - Docked toolbar",
            "Available",
        ),
        (
            "",
            "Android Views (MDC-Android): Expressive - Floating toolbar",
            "Available",
        ),
        ("", "Flutter - Bottom app bar", "Available"),
        ("", "Web", "Unavailable"),
    ];
    let mut cells = Vec::new();
    for (index, (kind, resource, status)) in rows.into_iter().enumerate() {
        let row = index + 1;
        if !kind.is_empty() {
            cells.push(TextTableCell {
                text: kind.into(),
                row,
                column: 0,
                row_span: if index == 0 { 1 } else { rows.len() - 1 },
                ..Default::default()
            });
        }
        cells.push(TextTableCell {
            text: resource.into(),
            row,
            column: 1,
            ..Default::default()
        });
        cells.push(TextTableCell {
            text: status.into(),
            row,
            column: 2,
            ..Default::default()
        });
    }
    spec_table_cells(
        vec!["Type".into(), "Resource".into(), "Status".into()],
        cells,
    );
    gap(Dp(24.0));
    prose(
        "The live examples on this page use Tessera's docked_toolbar and floating_toolbar components with standard and vibrant colors, horizontal and vertical layouts, and paired FABs.",
    );
}

#[tessera]
fn expressive_update() {
    prose(
        "The bottom app bar is no longer recommended and should be replaced with the docked toolbar, which functions similarly, but is shorter and has more flexibility. The floating toolbar was created for more versatility, greater amounts of actions, and more variety in where it's placed.",
    );
    gap(Dp(24.0));
    prose("Variants and naming:");
    gap(Dp(8.0));
    bullets(vec![
        "Added docked toolbar to replace bottom app bar".into(),
        "- Size: Shorter height".into(),
        "- Color: Standard or vibrant".into(),
        "- Flexibility: More layout and element options".into(),
        "Added floating toolbar with the following configurations:".into(),
        "- Layout: Horizontal or vertical".into(),
        "- Color: Standard or vibrant".into(),
        "- Flexibility: Can hold many elements and components. Can be paired with FAB.".into(),
        "Bottom app bar is still available, but not recommended".into(),
    ]);
    gap(Dp(24.0));
    gallery(GalleryKind::Expressive);
    gap(Dp(16.0));
    legend(vec![
        "Floating, vibrant color scheme and paired with FAB".into(),
        "Docked with embedded primary action instead of FAB".into(),
    ]);
}

#[tessera]
fn differences() {
    bullets(vec![
        "Color: New color mappings and compatibility with dynamic color".into(),
        "Elevation: No shadow".into(),
        "Layout: Container height is taller and the FAB is now contained within the app bar container".into(),
    ]);
    gap(Dp(24.0));
    m2_comparison();
}

#[tessera]
fn spec_variants() {
    gallery(GalleryKind::Variants);
    gap(Dp(16.0));
    legend(vec!["Docked toolbar".into(), "Floating toolbar".into()]);
    gap(Dp(32.0));
    subheading("Baseline variant".into());
    gap(Dp(16.0));
    prose(
        "The baseline bottom app bar is no longer recommended. It should be replaced with the docked toolbar, which is very similar and more flexible.",
    );
    gap(Dp(24.0));
    baseline_figure();
    gap(Dp(16.0));
    caption("Bottom app bar (not recommended)");
    gap(Dp(32.0));
    spec_table(
        vec!["Variant".into(), "M3".into(), "M3 Expressive".into()],
        vec![
            vec!["Docked toolbar".into(), "--".into(), "Available".into()],
            vec!["Floating toolbar".into(), "--".into(), "Available".into()],
            vec![
                "Bottom app bar".into(),
                "Available".into(),
                "Not recommended. Use docked toolbar.".into(),
            ],
        ],
    );
    gap(Dp(24.0));
    prose(
        "Note: Implementation differs per platform. In Tessera, docked_toolbar and floating_toolbar are separate components; the baseline bottom app bar is composed from a surface, icon buttons and a FAB.",
    );
}

fn table_cell(text: &str, row: usize, column: usize, row_span: usize) -> TextTableCell {
    TextTableCell {
        text: text.into(),
        row,
        column,
        row_span,
        ..Default::default()
    }
}

#[tessera]
fn spec_configurations() {
    gallery(GalleryKind::Configurations);
    gap(Dp(16.0));
    legend(vec![
        "Standard and vibrant toolbars".into(),
        "Vertical floating toolbar".into(),
        "Floating toolbar with FAB".into(),
    ]);
    gap(Dp(32.0));
    spec_table_cells(
        vec![
            "Category".into(),
            "Configuration".into(),
            "M3".into(),
            "M3 Expressive".into(),
        ],
        vec![
            table_cell("Color", 1, 0, 2),
            table_cell("Standard (default)", 1, 1, 1),
            table_cell("Available as bottom app bar", 1, 2, 1),
            table_cell("Available", 1, 3, 1),
            table_cell("Vibrant", 2, 1, 1),
            table_cell("--", 2, 2, 1),
            table_cell("Available", 2, 3, 1),
            table_cell("Floating toolbar layout", 3, 0, 2),
            table_cell("Horizontal (default)", 3, 1, 1),
            table_cell("--", 3, 2, 1),
            table_cell("Available", 3, 3, 1),
            table_cell("Vertical", 4, 1, 1),
            table_cell("--", 4, 2, 1),
            table_cell("Available", 4, 3, 1),
            table_cell("Other elements", 5, 0, 1),
            table_cell("With FAB", 5, 1, 1),
            table_cell("Available as bottom app bar", 5, 2, 1),
            table_cell("Available*", 5, 3, 1),
        ],
    );
    gap(Dp(24.0));
    prose(
        "Note: *Implementation differs per platform. In Tessera, the FAB is placed next to the floating toolbar as a separate component, 8dp from its container.",
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum TokenSet {
    #[default]
    Standard,
    Vibrant,
    Docked,
    Floating,
    FloatingFab,
}

fn token_rows(set: TokenSet) -> Vec<Vec<String>> {
    let rows: &[(&str, &str)] = match set {
        TokenSet::Standard => &[
            ("Toolbar standard container color", "Surface container"),
            (
                "Toolbar standard button container color",
                "Surface container",
            ),
            (
                "Toolbar standard selected button container color",
                "Secondary container",
            ),
            ("Toolbar standard icon color", "On surface variant"),
            (
                "Toolbar standard selected icon color",
                "On secondary container",
            ),
            ("Toolbar standard label color", "On surface variant"),
            (
                "Toolbar standard selected label color",
                "On secondary container",
            ),
            ("Toolbar standard container shape", "Full"),
            ("Toolbar standard disabled icon color", "On surface"),
            ("Toolbar standard disabled icon opacity", "0.38"),
        ],
        TokenSet::Vibrant => &[
            ("Toolbar vibrant container color", "Primary container"),
            (
                "Toolbar vibrant button container color",
                "Primary container",
            ),
            (
                "Toolbar vibrant selected button container color",
                "Surface container",
            ),
            ("Toolbar vibrant icon color", "On primary container"),
            ("Toolbar vibrant selected icon color", "On surface"),
            ("Toolbar vibrant label color", "On primary container"),
            ("Toolbar vibrant selected label color", "On surface"),
            ("Toolbar vibrant container shape", "Full"),
        ],
        TokenSet::Docked => &[
            ("Docked toolbar container color", "Surface container"),
            ("Docked toolbar container height", "64dp"),
            ("Docked toolbar leading padding", "16dp"),
            ("Docked toolbar trailing padding", "16dp"),
            ("Docked toolbar max space between actions", "32dp"),
            ("Docked toolbar min space between actions", "4dp"),
            ("Docked toolbar container shape", "None (rectangle)"),
        ],
        TokenSet::Floating => &[
            ("Floating toolbar standard color", "Surface container"),
            ("Floating toolbar vibrant color", "Primary container"),
            ("Floating toolbar container height - horizontal", "64dp"),
            ("Floating toolbar container width - vertical", "64dp"),
            ("Floating toolbar leading space", "8dp"),
            ("Floating toolbar trailing space", "8dp"),
            (
                "Floating toolbar margin from screen edge - horizontal",
                "16dp",
            ),
            (
                "Floating toolbar margin from screen edge - vertical",
                "24dp",
            ),
            ("Floating toolbar space between actions", "4dp"),
            ("Floating toolbar container shape", "Full"),
            ("Floating toolbar container elevation", "Level 3 (6dp)"),
        ],
        TokenSet::FloatingFab => &[
            ("Toolbar floating FAB space between toolbar and FAB", "8dp"),
            (
                "Toolbar floating FAB standard container color",
                "Secondary container",
            ),
            (
                "Toolbar floating FAB standard icon color",
                "On secondary container",
            ),
            (
                "Toolbar floating FAB vibrant container color",
                "Tertiary container",
            ),
            (
                "Toolbar floating FAB vibrant icon color",
                "On tertiary container",
            ),
            (
                "Toolbar floating FAB container size (expanded)",
                "56dp × 56dp",
            ),
            ("Toolbar floating FAB icon size (expanded)", "24dp"),
            (
                "Toolbar floating FAB medium container size (collapsed)",
                "80dp × 80dp",
            ),
            ("Toolbar floating FAB medium icon size (collapsed)", "28dp"),
        ],
    };
    rows.iter()
        .map(|(token, value)| vec![(*token).to_string(), (*value).to_string()])
        .collect()
}

fn token_set_label(set: TokenSet) -> &'static str {
    match set {
        TokenSet::Standard => "Toolbar - Color - Standard",
        TokenSet::Vibrant => "Toolbar - Color - Vibrant",
        TokenSet::Docked => "Toolbar - Docked",
        TokenSet::Floating => "Toolbar - Floating",
        TokenSet::FloatingFab => "Toolbar - Floating - FAB",
    }
}

#[tessera]
fn token_chip(set: TokenSet, selected: State<TokenSet>) {
    let label = token_set_label(set);
    if selected.get() == set {
        button()
            .tonal()
            .selected(true)
            .accessibility_label(label)
            .on_click(move || selected.set(set))
            .child(move || {
                text().content(label);
            });
    } else {
        button()
            .outlined()
            .selected(false)
            .accessibility_label(label)
            .on_click(move || selected.set(set))
            .child(move || {
                text().content(label);
            });
    }
}

#[tessera]
fn tokens() {
    let selected = remember(TokenSet::default);
    prose(
        "Browse the component elements, attributes, tokens, and their values. Color tokens resolve through the active Material theme; the baseline bottom app bar specs follow at the end of this tab.",
    );
    gap(Dp(24.0));
    flow_row()
        .modifier(Modifier::new().fill_max_width())
        .item_spacing(Dp(8.0))
        .line_spacing(Dp(8.0))
        .children(move || {
            for set in [
                TokenSet::Standard,
                TokenSet::Vibrant,
                TokenSet::Docked,
                TokenSet::Floating,
                TokenSet::FloatingFab,
            ] {
                token_chip(set, selected);
            }
        });
    gap(Dp(24.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        token_rows(selected.get()),
    );
}

#[tessera]
fn spec_anatomy() {
    figure(Variant::Floating, ToolbarColorStyle::Standard, false);
    gap(Dp(32.0));
    subheading("Flexibility & slots".into());
    gap(Dp(16.0));
    prose(
        "When configuring a toolbar, think of it as a container with several slots. Each slot can be a different element. The most common elements are icon buttons, buttons, and text fields.",
    );
    gap(Dp(24.0));
    slots();
    gap(Dp(16.0));
    caption("A toolbar is essentially a container with configurable slots");
    gap(Dp(24.0));
    search_example();
}

#[tessera]
fn spec_color() {
    prose(
        "Color values are implemented through design tokens. For design, this means working with color values that correspond with tokens. For implementation, a color value will be a token that references a value.",
    );
    gap(Dp(32.0));
    subheading("Standard".into());
    gap(Dp(16.0));
    prose("Standard color schemes and icon button types:");
    gap(Dp(16.0));
    color_roles(false);
    gap(Dp(32.0));
    subheading("Vibrant".into());
    gap(Dp(16.0));
    prose("Vibrant color scheme and icon button types:");
    gap(Dp(16.0));
    color_roles(true);
}

#[tessera]
fn measurements() {
    prose(
        "By default all toolbars are 64dp high, center-aligned, have equal padding between items, and have a minimum outside padding of 16dp.",
    );
    gap(Dp(32.0));
    subheading("Docked toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Docked, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("Default margins and padding");
    gap(Dp(32.0));
    prose("Alignment and padding can be configured to create unique layouts:");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Edges);
    gap(Dp(16.0));
    caption("Left and right alignment");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Centered);
    gap(Dp(16.0));
    caption("Center-aligned, 8dp padding between items");
    gap(Dp(32.0));
    subheading("Floating toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Floating, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("Default padding of floating toolbar");
    gap(Dp(24.0));
    figure(Variant::WithFab, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("Floating toolbar size and padding measurements");
    gap(Dp(24.0));
    placement_figure(Placement::Horizontal);
    gap(Dp(16.0));
    placement_figure(Placement::Vertical);
    gap(Dp(16.0));
    caption("Floating toolbar margins");
    gap(Dp(32.0));
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height / vertical width".into(), "64dp".into()],
            vec!["Floating internal padding".into(), "8dp".into()],
            vec!["Floating space between actions".into(), "4dp".into()],
            vec!["Docked leading/trailing padding".into(), "16dp".into()],
            vec!["Docked space between actions".into(), "4dp - 32dp".into()],
            vec!["Horizontal outside margins".into(), "At least 16dp".into()],
            vec!["Vertical outside margins".into(), "At least 24dp".into()],
            vec!["FAB spacing".into(), "8dp".into()],
            vec!["Interactive target".into(), "At least 48×48dp".into()],
        ],
    );
}

#[tessera]
fn baseline_specs() {
    prose(
        "The baseline bottom app bar is no longer recommended. Use the docked toolbar for new page actions.",
    );
    gap(Dp(24.0));
    baseline_figure();
    gap(Dp(16.0));
    caption("Container");
    gap(Dp(32.0));
    subheading("Tokens & specs".into());
    gap(Dp(16.0));
    prose("Bottom app bar tokens are in one token set.");
    gap(Dp(16.0));
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec![
                "Bottom app bar container color".into(),
                "Surface container".into(),
            ],
            vec!["Bottom app bar container height".into(), "80dp".into()],
            vec![
                "Bottom app bar container shape".into(),
                "None (rectangle)".into(),
            ],
            vec!["Bottom app bar icon button target".into(), "48dp".into()],
            vec![
                "Bottom app bar icon color".into(),
                "On surface variant".into(),
            ],
            vec!["Bottom app bar FAB size".into(), "56dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("Color".into());
    gap(Dp(16.0));
    prose(
        "Color values are implemented through design tokens. Bottom app bar color role used for light and dark themes: Surface container.",
    );
    gap(Dp(32.0));
    subheading("Measurements".into());
    gap(Dp(16.0));
    prose(
        "The baseline bottom app bar uses an 80dp container with 48dp action targets, a 4dp leading inset, and a 56dp FAB 12dp from the top and 16dp from the trailing edge. The container figure above shows these padding and size measurements.",
    );
    gap(Dp(32.0));
    subheading("Common layouts".into());
    gap(Dp(24.0));
    baseline_layouts();
}

#[tessera]
fn usage() {
    prose(
        "Use a toolbar to provide actions related to the current page. Toolbars can contain many actions and can scale to show more actions in larger windows.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Elevation);
    gap(Dp(16.0));
    caption("A toolbar provides actions related to the current page");
    gap(Dp(32.0));
    prose("There are two variants of toolbars:");
    gap(Dp(8.0));
    bullets(vec![
        "Docked toolbar: Spans the full width of the window. It's best used for global actions that remain the same across multiple pages.".into(),
        "Floating toolbar: Floats above the body content. It's best used for contextual actions relevant to the body content or the specific page.".into(),
    ]);
    gap(Dp(8.0));
    prose("The baseline bottom app bar is no longer recommended, but is still supported.");
    gap(Dp(24.0));
    gallery(GalleryKind::DockedVsFloating);
    gap(Dp(32.0));
    prose("When actions don't fit in a toolbar, add a menu.");
    gap(Dp(24.0));
    overflow_example();
    gap(Dp(32.0));
    prose("There are two color configurations:");
    gap(Dp(8.0));
    bullets(vec![
        "Standard: A low-emphasis color scheme best used for focusing attention on the body content.".into(),
        "Vibrant: A high-emphasis color scheme that draws attention to the controls. It can also indicate a temporary change in the page behavior, such as entering edit mode.".into(),
    ]);
    gap(Dp(8.0));
    prose(
        "Consider using alternative color roles to create greater or lesser emphasis depending on the needs of the app.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::ColorEmphasis);
    gap(Dp(32.0));
    subheading("Toolbars & navigation bars".into());
    gap(Dp(16.0));
    prose(
        "The toolbar and navigation bar are both placed at the bottom of the window, so should not be shown at the same time. Show the navigation bar on primary pages, and toolbars on subsequent pages with actions.",
    );
    gap(Dp(24.0));
    navigation_diagram(NavDiagram::Pages);
    gap(Dp(32.0));
    prose(
        "Floating toolbars can be used as tabs between related subsequent pages in the product hierarchy. This helps group similar pages together, and shows that the selection affects the body content underneath.",
    );
    gap(Dp(24.0));
    navigation_diagram(NavDiagram::LocalNavigation);
    gap(Dp(16.0));
    do_note(
        "Keep navigation distinct, and use a toolbar to display local navigation on a specific page",
    );
    gap(Dp(12.0));
    prose(
        "Consider the existing app hierarchy when using a toolbar for local navigation. Avoid redundant or confusing navigation combinations in the same view.",
    );
    gap(Dp(12.0));
    dont_note(
        "Don't show a navigation bar and a toolbar with navigation controls at the same time",
    );
}

#[tessera]
fn guide_anatomy() {
    figure(Variant::Floating, ToolbarColorStyle::Standard, false);
    gap(Dp(32.0));
    subheading("Container".into());
    gap(Dp(16.0));
    prose(
        "The docked toolbar's container spans the full width of the window. Avoid applying rounded corners to the container. This can imply the container expands or changes upon interaction.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Corners);
    gap(Dp(32.0));
    prose(
        "As long as there's a minimum of 16dp padding on the leading and trailing edge, arrange controls inside however you see fit. The 32dp padding between items is just the default. All elements need a minimum 48x48dp target area to be accessible.",
    );
    gap(Dp(16.0));
    prose("Be cautious of including too many controls as it can be overwhelming.");
    gap(Dp(12.0));
    dont_note("Don't overwhelm people with too many controls");
    gap(Dp(32.0));
    prose(
        "The floating toolbar's container should be fully visible on screen. If more actions are needed, use an overflow menu.",
    );
    gap(Dp(12.0));
    do_note("Choose the most essential actions to show on screen by default");
    gap(Dp(12.0));
    dont_note("Floating toolbars shouldn't exceed the edge of the window or pane");
    gap(Dp(32.0));
    subheading("Elevation".into());
    gap(Dp(16.0));
    prose(
        "Floating toolbars have elevation by default. If the content beneath the toolbar is visually distinct, elevation can be removed.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Elevation);
    gap(Dp(16.0));
    caption(
        "The elevation on floating toolbars can be removed if on a visually distinct background",
    );
    gap(Dp(32.0));
    subheading("Flexibility & slots".into());
    gap(Dp(16.0));
    prose(
        "When configuring a toolbar, think of it as a container with several slots. These slots can be populated by buttons, icon buttons, images, text fields, or any kind of custom component. Icon buttons provide an even hierarchy of controls. Mixing in a filled icon button can help add emphasis to a single action.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Slots);
    gap(Dp(16.0));
    caption("Toolbars are made of slots that can contain many kinds of actions");
    gap(Dp(32.0));
    prose(
        "Visually emphasizing a single action more than others is an effective way to create hierarchy and guide people to controls they use most often. Avoid emphasizing more than one action at a time. Some common ways to add emphasis to toolbar actions include:",
    );
    gap(Dp(8.0));
    bullets(vec![
        "Use different icon button color styles, such as filled, tonal, and standard".into(),
        "Customize the color roles of a single action, such as a primary or secondary palette"
            .into(),
        "Use wide and narrow icon buttons".into(),
        "Pair the toolbar with a FAB".into(),
    ]);
    gap(Dp(24.0));
    gallery(GalleryKind::Emphasis);
    gap(Dp(16.0));
    caption("Two different ways to create a high emphasis action in toolbars");
    gap(Dp(32.0));
    gallery(GalleryKind::EmphasisDont);
    gap(Dp(12.0));
    dont_note(
        "Don't emphasize multiple buttons with bold, primary colors, such as a button and FAB together. Emphasize one action at a time.",
    );
    gap(Dp(12.0));
    dont_note(
        "Avoid mixing too many different controls in the same toolbar. A consistent control design keeps things clear.",
    );
    gap(Dp(12.0));
    prose(
        "Avoid using square icon buttons in floating toolbars. Their square shape conflicts with the fully-rounded shape of the floating toolbar container. Square buttons can be used in the docked toolbar.",
    );
    gap(Dp(12.0));
    dont_note("Don't use square filled icon buttons in floating toolbars");
    gap(Dp(32.0));
    subheading("Floating toolbar with FAB".into());
    gap(Dp(16.0));
    prose(
        "A FAB can be placed next to a floating toolbar to present one high-priority action alongside a unified set of toolbar actions. Use a FAB for the highest-priority action in the view, or to complement the controls.",
    );
    gap(Dp(24.0));
    figure(Variant::WithFab, ToolbarColorStyle::Vibrant, false);
    gap(Dp(16.0));
    caption("Floating toolbars can be paired with FABs");
}

#[tessera]
fn position() {
    prose(
        "Only place docked toolbars at the bottom of the window. If using other bottom-aligned elements, such as a navigation bar, don't use a docked toolbar.",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Docked);
    gap(Dp(16.0));
    caption("Docked toolbars are always at the bottom of the window");
    gap(Dp(32.0));
    prose(
        "Floating toolbars can be horizontal or vertical. Horizontal toolbars should have a minimum 16dp margin from the edge of the window.",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Horizontal);
    gap(Dp(16.0));
    caption("Horizontal floating toolbars should be at least 16dp from the edge of the window");
    gap(Dp(32.0));
    prose(
        "In larger breakpoints, floating toolbars can be vertical and placed on either side of the screen. Vertical toolbars should have a minimum 24dp margin.",
    );
    gap(Dp(24.0));
    figure(Variant::Vertical, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("Maintain at least a 24dp margin for vertical toolbars");
    gap(Dp(32.0));
    prose(
        "To keep vertical toolbars compact, don't use wide icon buttons. Use narrow or default icon buttons instead.",
    );
    gap(Dp(12.0));
    dont_note(
        "Using wide buttons with vertical toolbars can unnecessarily widen toolbar containers and hide other UI elements",
    );
    gap(Dp(32.0));
    prose(
        "Vertical toolbars should be positioned opposite the navigation rail to balance out the screen and keep actions easy to access. When showing a navigation rail and vertical floating toolbar at once, use the centered configuration of the navigation rail.",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Vertical);
    gap(Dp(16.0));
    caption(
        "When a nav rail is visible, the floating toolbar should be vertical on the opposite edge of the window",
    );
}

#[tessera]
fn adaptive() {
    prose(
        "Adaptive design allows an interface to respond or change based on context, such as the user, device, and usage.",
    );
    gap(Dp(32.0));
    subheading("Resizing".into());
    gap(Dp(16.0));
    prose(
        "Docked: The docked toolbar should always span 100% of the screen width. In compact breakpoints, elements in the toolbar should be evenly spaced. In medium breakpoints and larger, adjust the padding between controls to create a comfortable layout by centering all elements, or by centering a key action and aligning other elements to the edges.",
    );
    gap(Dp(24.0));
    figure(Variant::Docked, ToolbarColorStyle::Standard, false);
    gap(Dp(16.0));
    caption("Docked toolbar items should be evenly spaced in compact windows");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Centered);
    gap(Dp(16.0));
    caption("In medium breakpoints and larger, create a spacious layout by centering all elements");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Edges);
    gap(Dp(16.0));
    caption(
        "Align controls to the edge of the screen to make them easier to reach on tablets, and to better highlight a primary action in the middle",
    );
    gap(Dp(32.0));
    prose(
        "On web and large screens, the docked toolbar can be rounded. Dividers can be used to organize large amounts of items. Only shrink the height and use extra small buttons if vertical space is limited.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::RoundedDocked);
    gap(Dp(16.0));
    caption(
        "On web and other large screens, docked toolbars can be rounded and placed in different parts of the page",
    );
    gap(Dp(32.0));
    prose(
        "Floating: The container should only be as big as needed to hold the items inside before reaching the 16dp margin. If there's not enough space for all items, put them in an overflow menu in the trailing slot. As the breakpoint expands, more actions can be revealed. The floating toolbar width can also be capped to keep it smaller and hide more elements.",
    );
    gap(Dp(12.0));
    dont_note("Don't add extra space to a toolbar beyond its necessary items");
    gap(Dp(24.0));
    overflow_collapse_example();
    gap(Dp(16.0));
    caption(
        "At larger screen sizes, the container can display more controls before hitting the 16dp margin",
    );
    gap(Dp(32.0));
    prose(
        "Vertical toolbars aren't recommended for compact windows. They take up a significant area of the screen and may feel visually overwhelming, especially on screens with complex layouts. Only use them when the screen is simple or when the toolbar has a few controls.",
    );
    gap(Dp(12.0));
    caution_note("Vertical toolbars can cover important content in compact windows");
    gap(Dp(32.0));
    subheading("Presentation".into());
    gap(Dp(16.0));
    prose(
        "In larger breakpoints, floating toolbars can be aligned to opposite edges of the screen so they're easy to reach and group similar actions. For example, consider placing the undo and redo actions in one toolbar, and editing controls in another. Stylistic differences can help emphasize each toolbar's purpose and clarify hierarchy.",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Presentation);
    gap(Dp(16.0));
    caption(
        "Multiple toolbars with different stylistic treatments can create hierarchy and distinguish different kinds of actions",
    );
    gap(Dp(24.0));
    prose(
        "Don't use multiple toolbars in compact windows. There typically isn't enough room on screen. Instead, use one toolbar for all actions.",
    );
    gap(Dp(12.0));
    dont_note("Avoid using multiple toolbars in smaller windows");
    gap(Dp(24.0));
    prose(
        "Actions at the trailing edge of the toolbar can collapse into an overflow menu at smaller breakpoints, and become visible again at larger sizes. Use the window size control above to compare both states.",
    );
    gap(Dp(32.0));
    subheading("Right-to-left languages".into());
    gap(Dp(16.0));
    prose(
        "In right-to-left (RTL) languages, mirror individual items that need it, like icons and text direction. If the order of actions is important, flip the order of the actions as well.",
    );
    gap(Dp(24.0));
    rtl_example();
}

#[tessera]
fn behavior() {
    subheading("Scrolling".into());
    gap(Dp(16.0));
    prose(
        "Docked toolbars can either remain on the screen during scroll, or animate offscreen. Floating toolbars can remain on the screen, animate offscreen, or collapse into a single, high-emphasis action on scroll.",
    );
    gap(Dp(16.0));
    prose(
        "Use the controls below to move the toolbar offscreen or collapse it to its key action. Only one transition is available at a time.",
    );
    gap(Dp(24.0));
    collapsing_example();
    gap(Dp(16.0));
    dont_note("Toolbars shouldn't both collapse and transition off page");
}

#[tessera]
fn use_cases() {
    prose("People should be able to do the following with assistive technology:");
    gap(Dp(8.0));
    bullets(vec![
        "Navigate and activate any actions in the toolbar".into(),
        "Select a destination from a menu".into(),
        "Activate a back button".into(),
        "Maintain access to toolbar controls when the content is scrolled or collapsed".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "The toolbar has no interactions by default. All interactions are with the elements placed inside.",
    );
    gap(Dp(32.0));
    subheading("Touch".into());
    gap(Dp(16.0));
    prose(
        "When tapping on an icon button in the toolbar, a touch ripple appears, indicating interaction feedback.",
    );
    gap(Dp(32.0));
    subheading("Cursor".into());
    gap(Dp(16.0));
    prose(
        "When hovered, the hover state provides a visual cue to the user that the element is interactive. When clicked (in both active and inactive states), a ripple appears, showing the user feedback.",
    );
    gap(Dp(32.0));
    subheading("Initial focus".into());
    gap(Dp(16.0));
    prose(
        "Focus lands on the first interactive element. Use Tab to navigate through all other actions, and Space or Enter to activate them. Try it on the toolbar below.",
    );
    gap(Dp(24.0));
    focus_example();
}

#[tessera]
fn focus_example() {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    stage(
        width,
        RenderSlot::new(move || {
            sample_toolbar(Sample::Format).limit(4).notice(notice);
        }),
    );
    gap(Dp(12.0));
    caption(if notice.get().is_empty() {
        "Use Tab to navigate through interactive elements; use Space or Enter to activate actions"
            .to_string()
    } else {
        notice.get()
    });
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["Keys".into(), "Actions".into()],
        vec![
            vec![
                "Tab or Arrows".into(),
                "Navigate between interactive elements".into(),
            ],
            vec![
                "Space or Enter".into(),
                "Activate the focused element".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("Labeling elements".into());
    gap(Dp(16.0));
    prose(
        "On web, the toolbar container should have the toolbar role. On mobile, it can be a generic container. All actions inside the toolbar should follow their respective accessibility guidelines.",
    );
    gap(Dp(16.0));
    prose(
        "Tessera exposes the toolbar role on the container, labels it, and labels each action. Toggle actions also publish their toggled state.",
    );
    gap(Dp(24.0));
    collapsing_example();
}

#[tessera]
fn slots() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    panel(RenderSlot::new(move || {
        flow_row()
            .item_spacing(Dp(8.0))
            .line_spacing(Dp(8.0))
            .children(move || {
                for index in 1..=5 {
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(
                            Modifier::new()
                                .size(Dp(48.0), Dp(48.0))
                                .background_with_shape(
                                    scheme.secondary_container,
                                    Shape::rounded_rectangle(Dp(24.0)),
                                ),
                        )
                        .children(move || {
                            text()
                                .content(index.to_string())
                                .style(type_style(16.0, 24.0));
                        });
                }
            });
    }));
}

#[tessera]
fn search_toolbar(notice: State<String>) {
    let query = remember(String::new);
    let selected = remember(|| false);
    let tag = use_context::<GalleryTag>().map(|tag| tag.get().0);
    docked_toolbar()
        .accessibility_label("Search actions")
        .content(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    sample_action("Back", Emphasis::Standard, false, selected, notice)
                        .tag_optional(tag.map(|item| format!("toolbar-gallery-{item}-first")));
                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                    boxed()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            text_field()
                                .placeholder("Search mail")
                                .accessibility_label("Search mail")
                                .shape(Shape::CAPSULE)
                                .min_width(Dp::ZERO)
                                .min_height(Dp(40.0))
                                .initial_text(query.get())
                                .on_change(move |next: String| {
                                    query.set(next.clone());
                                    next
                                })
                                .on_submit(move || {
                                    notice.set(format!("Searched for \"{}\"", query.get()))
                                })
                                .modifier(Modifier::new().fill_max_width());
                        });
                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                    sample_action("Microphone", Emphasis::Standard, false, selected, notice)
                        .tag_optional(tag.map(|item| format!("toolbar-gallery-{item}-last")));
                });
        });
}

#[tessera]
fn search_example() {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    stage(
        width,
        RenderSlot::new(move || {
            search_toolbar(notice);
        }),
    );
    gap(Dp(12.0));
    caption(if notice.get().is_empty() {
        "A text field placed in a toolbar slot between icon buttons".to_string()
    } else {
        notice.get()
    });
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum MockBar {
    #[default]
    None,
    Actions,
    Tabs,
}

#[tessera]
fn mock_toolbar(kind: MockBar) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    if kind == MockBar::None {
        return;
    }
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            Modifier::new()
                .padding_symmetric(Dp(8.0), Dp(6.0))
                .background_with_shape(scheme.primary_container, Shape::CAPSULE),
        )
        .children(move || {
            if kind == MockBar::Tabs {
                for (index, label) in ["All", "Unread", "Starred"].into_iter().enumerate() {
                    let selected = index == 0;
                    let pill = Modifier::new().padding_symmetric(Dp(10.0), Dp(4.0));
                    let pill = if selected {
                        pill.background_with_shape(scheme.surface_container, Shape::CAPSULE)
                    } else {
                        pill
                    };
                    boxed()
                        .alignment(Alignment::Center)
                        .modifier(pill)
                        .children(move || {
                            text().content(label).style(type_style(12.0, 16.0)).color(
                                if selected {
                                    scheme.on_surface
                                } else {
                                    scheme.on_primary_container
                                },
                            );
                        });
                }
            } else {
                for (index, label) in ["Archive", "Delete", "Reply", "More options"]
                    .into_iter()
                    .enumerate()
                {
                    if index > 0 {
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                    }
                    icon()
                        .vector(action_icon(label))
                        .size(Dp(20.0))
                        .tint(scheme.on_primary_container);
                }
            }
        });
}

#[tessera]
fn mock_nav_bar() {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    row()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(56.0))
                .background_with_shape(scheme.surface_container, Shape::RECTANGLE),
        )
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            for (index, label) in ["Mail", "Chat", "Settings"].into_iter().enumerate() {
                let selected = index == 0;
                column()
                    .modifier(Modifier::new().weight(1.0))
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(move || {
                        let indicator = Modifier::new().size(Dp(48.0), Dp(24.0));
                        let indicator = if selected {
                            indicator
                                .background_with_shape(scheme.secondary_container, Shape::CAPSULE)
                        } else {
                            indicator
                        };
                        boxed()
                            .alignment(Alignment::Center)
                            .modifier(indicator)
                            .children(move || {
                                icon().vector(action_icon(label)).size(Dp(18.0)).tint(
                                    if selected {
                                        scheme.on_secondary_container
                                    } else {
                                        scheme.on_surface_variant
                                    },
                                );
                            });
                        text()
                            .content(label)
                            .style(type_style(11.0, 14.0))
                            .color(scheme.on_surface_variant);
                    });
            }
        });
}

#[tessera]
fn phone_mock(#[prop(into)] title: String, toolbar: MockBar, nav_bar: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let description = format!(
        "{title} screen{}{}",
        match toolbar {
            MockBar::None => "",
            MockBar::Actions => " with a floating toolbar of actions",
            MockBar::Tabs => " with a floating toolbar of local navigation",
        },
        if nav_bar {
            " above a navigation bar"
        } else {
            ""
        }
    );
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(208.0), Dp(308.0))
                .background_with_shape(scheme.outline_variant, Shape::rounded_rectangle(Dp(28.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(description),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            let title = title.clone();
            column()
                .modifier(
                    Modifier::new()
                        .size(Dp(200.0), Dp(300.0))
                        .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(24.0)))
                        .clip_shape(Shape::rounded_rectangle(Dp(24.0))),
                )
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    text()
                        .content(title.clone())
                        .style(type_style(16.0, 24.0))
                        .modifier(Modifier::new().fill_max_width().padding(Padding::new(
                            Dp(16.0),
                            Dp(16.0),
                            Dp(16.0),
                            Dp(8.0),
                        )));
                    for width in [168.0, 140.0, 168.0, 120.0] {
                        boxed().modifier(
                            Modifier::new()
                                .size(Dp(width), Dp(12.0))
                                .background_with_shape(
                                    scheme.surface_container_high,
                                    Shape::rounded_rectangle(Dp(6.0)),
                                ),
                        );
                        gap(Dp(10.0));
                    }
                    spacer().modifier(Modifier::new().weight(1.0));
                    mock_toolbar(toolbar);
                    if toolbar != MockBar::None {
                        gap(Dp(12.0));
                    }
                    if nav_bar {
                        mock_nav_bar();
                    } else {
                        gap(Dp(4.0));
                    }
                });
        });
}

#[tessera]
fn mock_screen(
    #[prop(into)] title: String,
    toolbar: MockBar,
    nav_bar: bool,
    #[prop(into)] note: String,
) {
    column()
        .modifier(Modifier::new().width(Dp(208.0)))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            phone_mock(title.clone(), toolbar, nav_bar);
            gap(Dp(12.0));
            caption(note.clone());
        });
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum NavDiagram {
    #[default]
    Pages,
    LocalNavigation,
}

#[tessera]
fn navigation_diagram(kind: NavDiagram) {
    let width = remember(|| Dp::ZERO);
    stage(
        width,
        RenderSlot::new(move || {
            floating_group(RenderSlot::new(move || match kind {
                NavDiagram::Pages => {
                    mock_screen(
                        "Inbox",
                        MockBar::None,
                        true,
                        "Navigation bar on a primary page",
                    );
                    mock_screen(
                        "Message",
                        MockBar::Actions,
                        false,
                        "Toolbar on a secondary page with contextual actions",
                    );
                }
                NavDiagram::LocalNavigation => {
                    mock_screen(
                        "Inbox",
                        MockBar::Tabs,
                        false,
                        "Do: Use a toolbar to display local navigation on a specific page",
                    );
                    mock_screen(
                        "Inbox",
                        MockBar::Tabs,
                        true,
                        "Don't: Show a navigation bar and a toolbar with navigation controls at the same time",
                    );
                }
            }));
        }),
    );
}

#[tessera]
fn m2_bar(icons: usize) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(84.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(
                        "M2 bottom app bar in the primary color with 8dp elevation and a FAB overlapping its top edge"
                            .into(),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            surface()
                .style(SurfaceStyle::Filled {
                    color: scheme.primary,
                })
                .elevation(Dp(8.0))
                .modifier(
                    Modifier::new()
                        .align(Alignment::BottomStart)
                        .fill_max_width()
                        .height(Dp(56.0)),
                )
                .child(move || {
                    row()
                        .modifier(
                            Modifier::new()
                                .fill_max_size()
                                .padding_symmetric(Dp(16.0), Dp::ZERO),
                        )
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            for (index, label) in ["Archive", "Delete", "Mark unread", "Snooze"]
                                .into_iter()
                                .take(icons)
                                .enumerate()
                            {
                                if index > 0 {
                                    spacer().modifier(Modifier::new().width(Dp(24.0)));
                                }
                                icon()
                                    .vector(action_icon(label))
                                    .size(Dp(24.0))
                                    .tint(scheme.on_primary);
                            }
                        });
                });
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    Modifier::new()
                        .align(Alignment::TopEnd)
                        .padding(Padding::new(Dp::ZERO, Dp::ZERO, Dp(16.0), Dp::ZERO))
                        .size(Dp(56.0), Dp(56.0))
                        .background_with_shape(scheme.secondary_container, Shape::Ellipse),
                )
                .children(move || {
                    icon()
                        .vector(filled::ADD_SVG)
                        .size(Dp(24.0))
                        .tint(scheme.on_secondary_container);
                });
        });
}

#[tessera]
fn m2_comparison() {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    stage(
        width,
        RenderSlot::new(move || {
            let icons = if compact(width.get()) { 2 } else { 4 };
            gallery_item(0).fill(true).legend(1).content(move || {
                m2_bar(icons);
            });
            gap(Dp(24.0));
            gallery_item(1).fill(true).legend(2).content(move || {
                bottom_bar(icons, true, notice);
            });
        }),
    );
    gap(Dp(16.0));
    legend(vec![
        "M2: Bottom app bar had higher elevation of 8dp and didn't contain the FAB".into(),
        "M3: Bottom app bar has new colors, a taller container, no elevation or shadow, and contains the FAB".into(),
    ]);
    if !notice.get().is_empty() {
        gap(Dp(12.0));
        caption(notice.get());
    }
}

#[tessera]
fn collapsing_example() {
    let expanded = remember(|| true);
    let visible = remember(|| true);
    let notice = remember(String::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            flow_row()
                .item_spacing(Dp(8.0))
                .line_spacing(Dp(8.0))
                .children(move || {
                    button()
                        .text()
                        .enabled(visible.get())
                        .accessibility_label("Toggle toolbar expansion")
                        .on_click(move || expanded.set(!expanded.get()))
                        .child(move || {
                            text().content(if expanded.get() {
                                "Collapse actions"
                            } else {
                                "Expand actions"
                            });
                        });
                    button()
                        .text()
                        .enabled(expanded.get())
                        .accessibility_label("Toggle toolbar visibility")
                        .on_click(move || visible.set(!visible.get()))
                        .child(move || {
                            text().content(if visible.get() {
                                "Move offscreen"
                            } else {
                                "Show toolbar"
                            });
                        });
                });
            gap(Dp(24.0));
            panel(RenderSlot::new(move || {
                if visible.get() {
                    toolbar_content(
                        Variant::Floating,
                        ToolbarColorStyle::Vibrant,
                        expanded.get(),
                        notice,
                    );
                } else {
                    spacer().modifier(Modifier::new().height(Dp(64.0)));
                }
            }));
            gap(Dp(16.0));
            text()
                .content(if notice.get().is_empty() {
                    "No toolbar action yet".into()
                } else {
                    notice.get()
                })
                .style(type_style(14.0, 20.0));
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use std::cell::Cell;

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::{Action, ActionRequest, Node, NodeId, Toggled, TreeId},
        testing::layout_test,
    };

    use super::*;

    thread_local! {
        static CONFIGURATION: Cell<(Variant, bool)> = const { Cell::new((Variant::Floating, true)) };
    }

    #[tessera]
    fn fixture() {
        let (variant, expanded) = CONFIGURATION.get();
        material_theme().child(move || {
            let notice = remember(String::new);
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    toolbar_content(variant, ToolbarColorStyle::Standard, expanded, notice);
                });
        });
    }

    #[tessera]
    fn compact_figure_fixture() {
        let (variant, _) = CONFIGURATION.get();
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    figure(variant, ToolbarColorStyle::Standard, true);
                });
        });
    }

    #[test]
    fn compact_toolbar_figures_keep_actions_and_paired_fab_inside_the_figure() {
        for variant in [
            Variant::Docked,
            Variant::Floating,
            Variant::Vertical,
            Variant::WithFab,
        ] {
            CONFIGURATION.set((variant, true));
            let snapshot = layout_test(compact_figure_fixture)
                .viewport_px(320, 600)
                .run();
            let figure = snapshot.node("spec-toolbar-figure").bounds();
            let mut tags = vec![
                "spec-toolbar-container",
                "spec-toolbar-action-0",
                "spec-toolbar-action-3",
            ];
            if variant == Variant::WithFab {
                tags.push("spec-toolbar-fab");
            }
            for tag in tags {
                let bounds = snapshot.node(tag).bounds();
                assert!(
                    bounds.x >= figure.x && bounds.y >= figure.y,
                    "{tag}: {bounds:?}"
                );
                assert!(
                    bounds.x + bounds.width <= figure.x + figure.width,
                    "{tag}: {bounds:?} outside {figure:?}"
                );
                assert!(
                    bounds.y + bounds.height <= figure.y + figure.height,
                    "{tag}: {bounds:?} outside {figure:?}"
                );
                if tag != "spec-toolbar-container" {
                    assert!(
                        bounds.width >= Dp(48.0).to_px() && bounds.height >= Dp(48.0).to_px(),
                        "{tag} retains its interactive target: {bounds:?}"
                    );
                }
            }
        }
    }

    #[tessera]
    fn baseline_fixture() {
        material_theme().child(|| {
            baseline_figure();
        });
    }

    #[test]
    fn baseline_bottom_bar_reference_preserves_80dp_and_action_fab_insets() {
        for width in [320, 720] {
            let snapshot = layout_test(baseline_fixture).viewport_px(width, 400).run();
            let bar = snapshot.node("baseline-bottom-bar").bounds();
            let first = snapshot.node("baseline-action-0").bounds();
            let fab = snapshot.node("baseline-fab").bounds();
            assert_eq!(bar.height, Dp(80.0).to_px());
            assert_eq!(
                (first.width, first.height),
                (Dp(48.0).to_px(), Dp(48.0).to_px())
            );
            assert_eq!(first.x - bar.x, Dp(4.0).to_px());
            assert_eq!(
                (fab.width, fab.height),
                (Dp(56.0).to_px(), Dp(56.0).to_px())
            );
            assert_eq!(fab.y - bar.y, Dp(12.0).to_px());
            assert_eq!(bar.x + bar.width - fab.x - fab.width, Dp(16.0).to_px());
        }
    }

    struct Pipelines;

    impl tessera_ui::RenderModule for Pipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn collapse_fixture() {
        material_theme().child(|| {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(collapsing_example);
        });
    }

    fn action_node(renderer: &HeadlessRenderer, label: &str) -> Option<(NodeId, Node)> {
        renderer
            .accessibility_snapshot()?
            .nodes
            .into_iter()
            .find(|(_, node)| node.label() == Some(label))
    }

    fn dispatch(renderer: &mut HeadlessRenderer, label: &str, action: Action) {
        let (target_node, _) = action_node(renderer, label).expect("toolbar action");
        assert!(renderer.dispatch_accessibility_action(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        }));
        renderer.render_frames(8);
    }

    #[test]
    #[ignore = "Requires a native GPU adapter for toolbar actions and focus routing"]
    fn collapsing_toolbar_preserves_toggle_state_without_activating_on_focus() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(collapse_fixture),
            vec![Box::new(Pipelines)],
            HeadlessConfig {
                width: 600,
                height: 360,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("native toolbar renderer");
        renderer.render_frames(8);
        let (_, toolbar) = action_node(&renderer, "Floating editing actions").unwrap();
        assert_eq!(toolbar.role(), Role::Toolbar);
        assert!(!toolbar.supports_action(Action::Click));
        let (select_id, select) = action_node(&renderer, "Select").unwrap();
        assert_eq!(select.toggled(), Some(Toggled::True));
        dispatch(&mut renderer, "Select", Action::Focus);
        assert_eq!(renderer.accessibility_snapshot().unwrap().focus, select_id);
        assert_eq!(
            action_node(&renderer, "Select").unwrap().1.toggled(),
            Some(Toggled::True)
        );
        assert!(action_node(&renderer, "No toolbar action yet").is_some());
        dispatch(&mut renderer, "Select", Action::Click);
        assert_eq!(
            action_node(&renderer, "Select").unwrap().1.toggled(),
            Some(Toggled::False)
        );
        assert!(action_node(&renderer, "Select activated").is_some());
        dispatch(&mut renderer, "Toggle toolbar expansion", Action::Click);
        assert!(action_node(&renderer, "Brush").is_none());
        assert_eq!(action_node(&renderer, "Select").unwrap().0, select_id);
        dispatch(&mut renderer, "Toggle toolbar expansion", Action::Click);
        assert!(action_node(&renderer, "Brush").is_some());
        assert_eq!(
            action_node(&renderer, "Select").unwrap().1.toggled(),
            Some(Toggled::False)
        );
        dispatch(&mut renderer, "Brush", Action::Click);
        assert!(action_node(&renderer, "Brush activated").is_some());
    }

    #[test]
    fn live_toolbar_figures_fit_real_interactive_controls_inside_64dp_cross_axis() {
        for (variant, expanded, expected_width, expected_height, count) in [
            (Variant::Floating, true, 220, 64, 4),
            (Variant::Floating, false, 64, 64, 1),
            (Variant::Vertical, true, 64, 220, 4),
            (Variant::Docked, true, 600, 64, 4),
        ] {
            CONFIGURATION.set((variant, expanded));
            let snapshot = layout_test(fixture).viewport_px(600, 400).run();
            snapshot
                .node("spec-toolbar-container")
                .size(expected_width, expected_height);
            let container = snapshot.node("spec-toolbar-container").bounds();
            for index in 0..count {
                let action = snapshot
                    .node(&format!("spec-toolbar-action-{index}"))
                    .bounds();
                assert!(action.width >= Dp(48.0).to_px() && action.height >= Dp(48.0).to_px());
                assert!(action.x >= container.x && action.y >= container.y);
                assert!(action.x + action.width <= container.x + container.width);
                assert!(action.y + action.height <= container.y + container.height);
                if variant != Variant::Docked {
                    assert_eq!(
                        (action.width, action.height),
                        (Dp(48.0).to_px(), Dp(48.0).to_px())
                    );
                    let main = Dp([8.0, 60.0, 112.0, 164.0][index]).to_px();
                    let cross = Dp(8.0).to_px();
                    assert_eq!(
                        (action.x - container.x, action.y - container.y),
                        if variant == Variant::Vertical {
                            (cross, main)
                        } else {
                            (main, cross)
                        }
                    );
                    if index + 1 == count {
                        let trailing = if variant == Variant::Vertical {
                            container.y + container.height - action.y - action.height
                        } else {
                            container.x + container.width - action.x - action.width
                        };
                        assert_eq!(trailing, Dp(8.0).to_px());
                    }
                }
            }
        }
    }

    thread_local! {
        static GALLERY: Cell<GalleryKind> = const { Cell::new(GalleryKind::Overview) };
        static PLACEMENT: Cell<Placement> = const { Cell::new(Placement::Docked) };
        static DOCKED_LAYOUT: Cell<DockedLayout> = const { Cell::new(DockedLayout::Edges) };
    }

    fn settled<Output: 'static>(
        content: fn() -> Output,
        width: u32,
        height: u32,
    ) -> tessera_ui::testing::LayoutSnapshot {
        let mut session = tessera_ui::testing::start_layout_test_session(
            layout_test(content).viewport_px(width, height),
        );
        for frame in 0..3 {
            tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_000_000);
        }
        tessera_ui::testing::pump_layout_test_session(&mut session, 48_000_000)
    }

    #[tessera]
    fn gallery_fixture() {
        let kind = GALLERY.get();
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    gallery(kind);
                });
        });
    }

    #[test]
    fn toolbar_galleries_keep_every_configuration_and_target_inside_compact_and_medium_stages() {
        for (kind, items, fabs) in [
            (GalleryKind::Overview, 7, &[1usize][..]),
            (GalleryKind::Variants, 2, &[][..]),
            (GalleryKind::Configurations, 6, &[5][..]),
            (GalleryKind::Expressive, 2, &[0][..]),
            (GalleryKind::UseCases, 5, &[2][..]),
            (GalleryKind::Slots, 5, &[][..]),
            (GalleryKind::Emphasis, 2, &[1][..]),
            (GalleryKind::EmphasisDont, 1, &[0][..]),
            (GalleryKind::Presentation, 2, &[][..]),
            (GalleryKind::RoundedDocked, 1, &[][..]),
        ] {
            GALLERY.set(kind);
            for width in [320, 600] {
                let snapshot = settled(gallery_fixture, width, 1600);
                let stage = snapshot.node("toolbar-stage").bounds();
                for item in 0..items {
                    let mut tags = vec![
                        format!("toolbar-gallery-{item}-first"),
                        format!("toolbar-gallery-{item}-last"),
                    ];
                    if fabs.contains(&item) {
                        tags.push(format!("toolbar-gallery-{item}-fab"));
                    }
                    for tag in tags {
                        let bounds = snapshot.node(&tag).bounds();
                        assert!(
                            bounds.width >= Dp(40.0).to_px() && bounds.height >= Dp(40.0).to_px(),
                            "{kind:?} at {width}px: {tag} lost its target {bounds:?}"
                        );
                        assert!(
                            bounds.x >= stage.x
                                && bounds.x + bounds.width <= stage.x + stage.width
                                && bounds.y >= stage.y
                                && bounds.y + bounds.height <= stage.y + stage.height,
                            "{kind:?} at {width}px: {tag} {bounds:?} outside {stage:?}"
                        );
                    }
                }
            }
        }
    }

    #[tessera]
    fn placement_fixture() {
        let placement = PLACEMENT.get();
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    placement_figure(placement);
                });
        });
    }

    #[test]
    fn placement_figures_keep_docked_flush_and_floating_margins_from_the_window() {
        for width in [320, 720] {
            for placement in [
                Placement::Docked,
                Placement::Horizontal,
                Placement::Vertical,
            ] {
                PLACEMENT.set(placement);
                let snapshot = settled(placement_fixture, width, 600);
                let window = snapshot.node("placement-window").bounds();
                let toolbar = snapshot.node("placement-toolbar").bounds();
                assert!(
                    toolbar.x >= window.x
                        && toolbar.y >= window.y
                        && toolbar.x + toolbar.width <= window.x + window.width
                        && toolbar.y + toolbar.height <= window.y + window.height,
                    "{placement:?} at {width}px: {toolbar:?} outside {window:?}"
                );
                match placement {
                    Placement::Docked => {
                        assert_eq!((toolbar.x, toolbar.width), (window.x, window.width));
                        assert_eq!(toolbar.y + toolbar.height, window.y + window.height);
                        assert_eq!(toolbar.height, Dp(64.0).to_px());
                    }
                    Placement::Horizontal => {
                        assert_eq!(
                            window.y + window.height - toolbar.y - toolbar.height,
                            Dp(16.0).to_px()
                        );
                    }
                    Placement::Vertical => {
                        assert_eq!(
                            window.x + window.width - toolbar.x - toolbar.width,
                            Dp(24.0).to_px()
                        );
                        assert_eq!(toolbar.width, Dp(64.0).to_px());
                    }
                }
            }
        }
    }

    #[tessera]
    fn docked_layout_fixture() {
        let layout = DOCKED_LAYOUT.get();
        material_theme().child(move || {
            column()
                .modifier(Modifier::new().fill_max_width())
                .children(move || {
                    docked_layout_figure(layout);
                });
        });
    }

    #[test]
    fn docked_alignment_figures_center_with_8dp_gaps_or_pin_actions_to_16dp_edges() {
        for width in [320, 720] {
            let count = if width < 400 { 3 } else { 5 };
            DOCKED_LAYOUT.set(DockedLayout::Centered);
            let snapshot = settled(docked_layout_fixture, width, 400);
            let container = snapshot.node("docked-layout").bounds();
            let first = snapshot.node("layout-action-0").bounds();
            let last = snapshot
                .node(&format!("layout-action-{}", count - 1))
                .bounds();
            let second = snapshot.node("layout-action-1").bounds();
            assert_eq!(second.x - (first.x + first.width), Dp(8.0).to_px());
            let leading = first.x - container.x;
            let trailing = container.x + container.width - (last.x + last.width);
            assert!(
                (leading.0 - trailing.0).abs() <= 1,
                "{leading:?} vs {trailing:?}"
            );
            assert!(leading >= Dp(16.0).to_px());

            DOCKED_LAYOUT.set(DockedLayout::Edges);
            let snapshot = settled(docked_layout_fixture, width, 400);
            let container = snapshot.node("docked-layout").bounds();
            let first = snapshot.node("layout-action-0").bounds();
            let last = snapshot
                .node(&format!("layout-action-{}", count - 1))
                .bounds();
            assert_eq!(first.x - container.x, Dp(16.0).to_px());
            assert_eq!(
                container.x + container.width - (last.x + last.width),
                Dp(16.0).to_px()
            );
        }
    }

    #[test]
    fn collapsed_sample_actions_keep_leading_actions_and_trailing_overflow() {
        let labels = |sample, limit| {
            visible_actions(sample_actions(sample), limit)
                .into_iter()
                .map(|action| action.label)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            labels(Sample::Format, 3),
            ["Bold", "Italic", "More options"]
        );
        assert_eq!(
            labels(Sample::Mail, 3),
            ["Archive", "Delete", "Mark unread"]
        );
        assert_eq!(labels(Sample::History, 6), ["Undo", "Redo"]);
    }
}
