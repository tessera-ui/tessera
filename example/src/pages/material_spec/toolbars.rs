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
    theme::MaterialTheme,
    toolbar::{ToolbarColorStyle, ToolbarOrientation, docked_toolbar, floating_toolbar},
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::res::I18N;

use super::{
    INK,
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
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
pub(super) fn toolbars_spec(route: Option<crate::pages::material_spec::article::ArticleRoute>) {
    article(
        "ui-toolbars",
        "ui-toolbars-display-frequently-used-actions-relevant-to-the-current-page",
        vec![
            SpecSection {
                title: "ui-variants",
                content: RenderSlot::new(spec_variants),
            },
            SpecSection {
                title: "ui-configurations",
                content: RenderSlot::new(spec_configurations),
            },
            SpecSection {
                title: "ui-tokens-specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "ui-anatomy",
                content: RenderSlot::new(spec_anatomy),
            },
            SpecSection {
                title: "ui-color",
                content: RenderSlot::new(spec_color),
            },
            SpecSection {
                title: "ui-measurements",
                content: RenderSlot::new(measurements),
            },
            SpecSection {
                title: "ui-bottom-app-bar-baseline",
                content: RenderSlot::new(baseline_specs),
            },
        ],
    )
    .route_optional(route.clone())
    .pages(vec![
        ArticlePage {
            title: "ui-overview",
            intro: Some(RenderSlot::new(overview)),
            sections: vec![],
        },
        ArticlePage {
            title: "ui-guidelines",
            intro: Some(RenderSlot::new(|| {
                gallery(GalleryKind::UseCases);
                gap(Dp(16.0));
                caption("ui-toolbars-can-be-used-for-a-wide-variety-of-use-cases");
            })),
            sections: vec![
                SpecSection {
                    title: "ui-usage",
                    content: RenderSlot::new(usage),
                },
                SpecSection {
                    title: "ui-anatomy",
                    content: RenderSlot::new(guide_anatomy),
                },
                SpecSection {
                    title: "ui-position-orientation",
                    content: RenderSlot::new(position),
                },
                SpecSection {
                    title: "ui-adaptive-design",
                    content: RenderSlot::new(adaptive),
                },
                SpecSection {
                    title: "ui-behavior",
                    content: RenderSlot::new(behavior),
                },
            ],
        },
        ArticlePage {
            title: "ui-accessibility",
            intro: None,
            sections: vec![
                SpecSection {
                    title: "ui-use-cases",
                    content: RenderSlot::new(use_cases),
                },
                SpecSection {
                    title: "ui-interaction-style",
                    content: RenderSlot::new(interaction_style),
                },
                SpecSection {
                    title: "ui-keyboard-navigation",
                    content: RenderSlot::new(keyboard),
                },
            ],
        },
    ]);
}

#[tessera]
fn paragraph(content: String) {
    text(I18N.text(content)).style(type_style(16.0, 24.0));
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
        0 => ("ui-select", filled::CHECK_BOX_SVG),
        1 => ("ui-brush", filled::BRUSH_SVG),
        2 => ("ui-record", filled::MIC_SVG),
        _ => ("ui-add-image", filled::IMAGE_SVG),
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
            .accessibility_label(I18N.text(label))
            .on_click(move || {
                selected.set(!selected.get());
                notice.set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]));
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
            .accessibility_label(I18N.text(label))
            .on_click(move || {
                notice.set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]));
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
            .accessibility_label(I18N.text(label))
            .on_click(move || {
                notice.set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]));
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
            .accessibility_label(I18N.text("ui-docked-editing-actions"))
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
            .accessibility_label(I18N.text("ui-floating-editing-actions"))
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
                                    label: Some(I18N.text("ui-baseline-bottom-app-bar")),
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
                                        ("ui-select", filled::CHECK_SVG),
                                        ("ui-search", filled::SEARCH_SVG),
                                        ("ui-record", filled::MIC_SVG),
                                        ("ui-image", filled::IMAGE_SVG),
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
                                            .accessibility_label(I18N.format(
                                                "ui-baseline-value-format-430",
                                                [("v0", I18N.text(&label).into())],
                                            ))
                                            .on_click(move || {
                                                notice.set(I18N.format(
                                                    "ui-baseline-value-activated",
                                                    [("v0", I18N.text(&label).into())],
                                                ))
                                            });
                                    }
                                    spacer().modifier(Modifier::new().weight(1.0));
                                    floating_action_button()
                                        .modifier(watch(geometry, "fab").semantics(SemanticsArgs {
                                            test_tag: Some("baseline-fab".into()),
                                            ..Default::default()
                                        }))
                                        .accessibility_label(I18N.text("ui-baseline-create"))
                                        .on_click(move || {
                                            notice.set("ui-baseline-create-activated".into())
                                        })
                                        .content(|| {
                                            icon().painter(filled::ADD_SVG);
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
                            label: Some(
                                I18N.format(
                                    "ui-toolbar-value-figure",
                                    [(
                                        "v0",
                                        I18N.text(
                                            &(if measurements {
                                                "ui-measurement-7071"
                                            } else {
                                                "ui-anatomy-7072"
                                            }),
                                        )
                                        .into(),
                                    )],
                                ),
                            ),
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
                                                        .accessibility_label(
                                                            I18N.text("ui-create-item"),
                                                        )
                                                        .elevation(
                                                            FloatingActionButtonElevation::new(
                                                                Dp(1.0),
                                                                Dp(1.0),
                                                                Dp(3.0),
                                                                Dp(1.0),
                                                            ),
                                                        )
                                                        .on_click(move || {
                                                            notice.set(
                                                                "ui-create-item-activated".into(),
                                                            )
                                                        })
                                                        .content(|| {
                                                            icon()
                                                                .painter(filled::ADD_SVG)
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
                legend(vec!["ui-container".into(), "ui-placed-components".into()]);
            }
            if !notice.get().is_empty() {
                text(I18N.text(notice.get())).style(type_style(14.0, 20.0));
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
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text(I18N.text(value))
        .color(scheme.on_surface_variant)
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn bullets(items: Vec<String>) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            for item in items.clone() {
                let item = I18N.text(item);
                let (indent, item) = match item.strip_prefix("- ") {
                    Some(nested) => (Dp(24.0), nested.to_string()),
                    None => (Dp::ZERO, item),
                };
                text(format!("•  {}", I18N.text(item)))
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
            text((number).to_string())
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
                        text(I18N.text(&(label)))
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
                    text(I18N.text(&(mark)))
                        .style(type_style(14.0, 20.0))
                        .color(Color::WHITE);
                });
            spacer().modifier(Modifier::new().width(Dp(12.0)));
            text(I18N.text(&(note)))
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
    guidance("ui-caution", note, CAUTION_COLOR);
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
            action("ui-archive", Standard),
            action("ui-delete", Standard),
            action("ui-mark-unread", Standard),
            action("ui-snooze", Standard),
            action("ui-star", Standard),
        ],
        Sample::Format => vec![
            action("ui-bold", Toggle),
            action("ui-italic", Standard),
            action("ui-underline", Standard),
            action("ui-text-color", Standard),
            action("ui-highlight", Standard),
            action("ui-more-options", Standard),
        ],
        Sample::FormatTonal => vec![
            action("ui-bold", Toggle),
            action("ui-italic", Toggle),
            action("ui-underline", Standard),
            action("ui-text-color", Standard),
            action("ui-more-options", Standard),
        ],
        Sample::Navigate => vec![
            action("ui-back", Standard),
            action("ui-forward", Standard),
            action("ui-add", Filled),
            action("ui-picture-in-picture", Standard),
            action("ui-more-options", Standard),
        ],
        Sample::Edit => vec![
            action("ui-camera-off", Standard),
            action("ui-microphone", Standard),
            action("ui-raise-hand", Toggle),
            action("ui-more-options", Standard),
        ],
        Sample::Vertical => vec![
            action("ui-undo", Standard),
            action("ui-redo", Standard),
            action("ui-add", Standard),
            action("ui-text-color", Standard),
            action("ui-more-options", Standard),
        ],
        Sample::History => vec![action("ui-undo", Standard), action("ui-redo", Standard)],
        Sample::ColorRoles => vec![
            action("ui-add", Filled),
            action("ui-bold", Toggle),
            action("ui-italic", Standard),
            action("ui-underline", Standard),
        ],
    }
}

fn action_icon(label: &str) -> tessera_components::material_icons::Asset {
    match label {
        "ui-archive" => filled::ARCHIVE_SVG,
        "ui-delete" => filled::DELETE_SVG,
        "ui-mark-unread" => filled::MARK_EMAIL_UNREAD_SVG,
        "ui-snooze" => filled::SNOOZE_SVG,
        "ui-star" => filled::STAR_SVG,
        "ui-bold" => filled::FORMAT_BOLD_SVG,
        "ui-italic" => filled::FORMAT_ITALIC_SVG,
        "ui-underline" => filled::FORMAT_UNDERLINED_SVG,
        "ui-text-color" => filled::FORMAT_COLOR_TEXT_SVG,
        "ui-highlight" => filled::FORMAT_COLOR_FILL_SVG,
        "ui-back" => filled::ARROW_BACK_SVG,
        "ui-forward" => filled::ARROW_FORWARD_SVG,
        "ui-add" => filled::ADD_SVG,
        "ui-picture-in-picture" => filled::PICTURE_IN_PICTURE_SVG,
        "ui-camera-off" => filled::VIDEOCAM_OFF_SVG,
        "ui-microphone" => filled::MIC_SVG,
        "ui-raise-hand" => filled::BACK_HAND_SVG,
        "ui-mail" => filled::MAIL_SVG,
        "ui-chat" => filled::PEOPLE_SVG,
        "ui-settings" => filled::SETTINGS_SVG,
        "ui-reply" => filled::REPLY_SVG,
        "ui-undo" => filled::UNDO_SVG,
        "ui-redo" => filled::REDO_SVG,
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
        Some(last) if last.label == "ui-more-options" => {
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
        Sample::Mail => "ui-mail-actions",
        Sample::Format | Sample::FormatTonal => "ui-formatting-actions",
        Sample::Navigate => "ui-browsing-actions",
        Sample::Edit => "ui-call-actions",
        Sample::Vertical => "ui-canvas-actions",
        Sample::History => "ui-history-actions",
        Sample::ColorRoles => "ui-color-role-actions",
    }
}

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
    let message = I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]);
    let modifier = tagged(tag);
    match emphasis {
        Emphasis::Toggle => {
            icon_button()
                .modifier(modifier)
                .icon(vector)
                .accessibility_label(I18N.text(&(label)))
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
                .accessibility_label(I18N.text(&(label)))
                .on_click(move || notice.set(message.clone()))
                .filled();
        }
        Emphasis::Standard => {
            icon_button()
                .modifier(modifier)
                .icon(vector)
                .accessibility_label(I18N.text(&(label)))
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
    gallery_tag: Option<usize>,
) {
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    let selected = remember(|| true);
    let docked = docked.unwrap_or(false);
    let vibrant = vibrant.unwrap_or(false);
    let vertical = vertical.unwrap_or(false);
    let dividers = dividers.unwrap_or(false);
    let actions = visible_actions(sample_actions(sample), limit.unwrap_or(usize::MAX));
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
            .accessibility_label(I18N.text(sample_label(sample)))
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
            .accessibility_label(I18N.text(sample_label(sample)))
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
    gallery_tag: Option<usize>,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let vibrant = vibrant.unwrap_or(false);
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    let fab_tag = gallery_tag.map(|item| format!("toolbar-gallery-{item}-fab"));
    let (label, container, content) = match fab {
        FabKind::EndCall => (
            "ui-end-call",
            scheme.error_container,
            scheme.on_error_container,
        ),
        FabKind::Reply if vibrant => (
            "ui-reply",
            scheme.tertiary_container,
            scheme.on_tertiary_container,
        ),
        FabKind::Add if vibrant => (
            "ui-create",
            scheme.tertiary_container,
            scheme.on_tertiary_container,
        ),
        FabKind::Reply => (
            "ui-reply",
            scheme.secondary_container,
            scheme.on_secondary_container,
        ),
        FabKind::Add => (
            "ui-create",
            scheme.secondary_container,
            scheme.on_secondary_container,
        ),
    };
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .children(move || {
            sample_toolbar(sample)
                .gallery_tag_optional(gallery_tag)
                .vibrant(vibrant)
                .limit_optional(limit)
                .notice(notice);
            spacer().modifier(Modifier::new().width(Dp(8.0)));
            floating_action_button()
                .modifier(tagged(fab_tag.clone()))
                .size(FloatingActionButtonSize::Standard)
                .container_color(container)
                .content_color(content)
                .accessibility_label(I18N.text(label))
                .elevation(FloatingActionButtonElevation::new(
                    Dp(1.0),
                    Dp(1.0),
                    Dp(3.0),
                    Dp(1.0),
                ))
                .on_click(move || {
                    notice
                        .set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]))
                })
                .content(move || {
                    icon()
                        .painter(match fab {
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
    content: Option<tessera_ui::RenderSlotWith<usize>>,
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
                content.render(index);
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
                    gallery_item(0).content(move |gallery_tag| {
                        sample_toolbar(Sample::Vertical)
                            .gallery_tag(gallery_tag)
                            .vertical(true)
                            .notice(notice);
                    });
                    gallery_item(1).content(move |gallery_tag| {
                        fab_pair(Sample::Edit, FabKind::EndCall)
                            .gallery_tag(gallery_tag)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                    gallery_item(2).content(move |gallery_tag| {
                        sample_toolbar(Sample::FormatTonal)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(3).content(move |gallery_tag| {
                        sample_toolbar(Sample::Navigate)
                            .gallery_tag(gallery_tag)
                            .limit(limit)
                            .notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(4).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Navigate)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .vibrant(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(5).fill(true).content(move |gallery_tag| {
                    text_nav_toolbar(false)
                        .gallery_tag(gallery_tag)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(6).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Mail)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .vibrant(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
            }
            GalleryKind::Variants => {
                gallery_item(0)
                    .fill(true)
                    .legend(1)
                    .content(move |gallery_tag| {
                        sample_toolbar(Sample::Mail)
                            .gallery_tag(gallery_tag)
                            .docked(true)
                            .limit(docked_limit)
                            .notice(notice);
                    });
                gap(Dp(32.0));
                gallery_item(1).legend(2).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .limit(limit)
                        .notice(notice);
                });
            }
            GalleryKind::Configurations => {
                gallery_item(0).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(1).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .vibrant(true)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(2).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Mail)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(3)
                    .fill(true)
                    .legend(1)
                    .content(move |gallery_tag| {
                        sample_toolbar(Sample::Mail)
                            .gallery_tag(gallery_tag)
                            .docked(true)
                            .vibrant(true)
                            .limit(docked_limit)
                            .notice(notice);
                    });
                gap(Dp(32.0));
                floating_group(RenderSlot::new(move || {
                    gallery_item(4).legend(2).content(move |gallery_tag| {
                        sample_toolbar(Sample::Format)
                            .gallery_tag(gallery_tag)
                            .vertical(true)
                            .limit(5)
                            .notice(notice);
                    });
                    gallery_item(5).legend(3).content(move |gallery_tag| {
                        fab_pair(Sample::Mail, FabKind::Reply)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::UseCases => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move |gallery_tag| {
                        sample_toolbar(Sample::Mail)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(1).content(move |gallery_tag| {
                        sample_toolbar(Sample::Format)
                            .gallery_tag(gallery_tag)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(2).content(move |gallery_tag| {
                        fab_pair(Sample::Edit, FabKind::EndCall)
                            .gallery_tag(gallery_tag)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                    gallery_item(3).content(move |gallery_tag| {
                        sample_toolbar(Sample::Navigate)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(4).fill(true).content(move |gallery_tag| {
                    text_nav_toolbar(false)
                        .gallery_tag(gallery_tag)
                        .notice(notice);
                });
            }
            GalleryKind::DockedVsFloating => {
                gallery_item(0).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Mail)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption("ui-docked-toolbar-shows-global-controls");
                gap(Dp(32.0));
                gallery_item(1).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption("ui-floating-toolbar-shows-controls-relevant-to-the-current-page");
            }
            GalleryKind::ColorEmphasis => {
                gallery_item(0).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption(
                    "ui-use-the-standard-color-scheme-to-draw-focus-to-content-outside-the-toolbar",
                );
                gap(Dp(32.0));
                gallery_item(1).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .vibrant(true)
                        .limit(limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                caption("ui-use-the-vibrant-color-scheme-to-emphasize-controls-or-actions");
            }
            GalleryKind::Corners => {
                gallery_item(0).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Mail)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                do_note("ui-use-straight-corners-for-docked-toolbars");
                gap(Dp(32.0));
                gallery_item(1).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Mail)
                        .gallery_tag(gallery_tag)
                        .docked(true)
                        .rounded(true)
                        .limit(docked_limit)
                        .notice(notice);
                });
                gap(Dp(12.0));
                dont_note("ui-avoid-modifying-the-container-shape");
            }
            GalleryKind::Elevation => {
                gallery_item(0).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
                        .vibrant(true)
                        .elevation(Dp::ZERO)
                        .limit(limit)
                        .notice(notice);
                });
            }
            GalleryKind::Slots => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move |gallery_tag| {
                        sample_toolbar(Sample::FormatTonal)
                            .gallery_tag(gallery_tag)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(1).content(move |gallery_tag| {
                        sample_toolbar(Sample::Navigate)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(2).content(move |gallery_tag| {
                        sample_toolbar(Sample::Edit)
                            .gallery_tag(gallery_tag)
                            .limit(limit)
                            .notice(notice);
                    });
                }));
                gap(Dp(24.0));
                gallery_item(3).fill(true).content(move |gallery_tag| {
                    text_nav_toolbar(false)
                        .gallery_tag(gallery_tag)
                        .notice(notice);
                });
                gap(Dp(16.0));
                gallery_item(4).fill(true).content(move |gallery_tag| {
                    search_toolbar(notice).gallery_tag(gallery_tag);
                });
            }
            GalleryKind::Emphasis => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move |gallery_tag| {
                        sample_toolbar(Sample::Navigate)
                            .gallery_tag(gallery_tag)
                            .limit(limit)
                            .notice(notice);
                    });
                    gallery_item(1).content(move |gallery_tag| {
                        fab_pair(Sample::Mail, FabKind::Reply)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(fab_limit)
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::EmphasisDont => {
                gallery_item(0).content(move |gallery_tag| {
                    fab_pair(Sample::Navigate, FabKind::Add)
                        .gallery_tag(gallery_tag)
                        .vibrant(true)
                        .limit(fab_limit)
                        .notice(notice);
                });
            }
            GalleryKind::Presentation => {
                floating_group(RenderSlot::new(move || {
                    gallery_item(0).content(move |gallery_tag| {
                        sample_toolbar(Sample::History)
                            .gallery_tag(gallery_tag)
                            .notice(notice);
                    });
                    gallery_item(1).content(move |gallery_tag| {
                        sample_toolbar(Sample::Format)
                            .gallery_tag(gallery_tag)
                            .vibrant(true)
                            .limit(limit.saturating_sub(2).max(2))
                            .notice(notice);
                    });
                }));
            }
            GalleryKind::RoundedDocked => {
                gallery_item(0).fill(true).content(move |gallery_tag| {
                    sample_toolbar(Sample::Format)
                        .gallery_tag(gallery_tag)
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
fn text_nav_toolbar(rtl: bool, notice: Option<State<String>>, gallery_tag: Option<usize>) {
    let local = remember(String::new);
    let notice = notice.unwrap_or(local);
    docked_toolbar()
        .accessibility_label(I18N.text(if rtl {
            "ui-mirrored-page-navigation"
        } else {
            "ui-page-navigation"
        }))
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
    let label = if next { "ui-next" } else { "ui-back" };
    if next {
        button()
            .modifier(tagged(tag))
            .filled()
            .accessibility_label(I18N.text(label))
            .on_click(move || {
                notice.set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]))
            })
            .child(move || {
                text(I18N.text(label));
            });
    } else {
        button()
            .modifier(tagged(tag))
            .text()
            .accessibility_label(I18N.text(label))
            .on_click(move || {
                notice.set(I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]))
            })
            .child(move || {
                text(I18N.text(label));
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
            "ui-filled-button-primary-on-primary".into(),
            "ui-toggle-tonal-button-surface-container-on-surface".into(),
            "ui-standard-button-on-primary-container".into(),
        ]
    } else {
        vec![
            "Surface container".into(),
            "ui-filled-button-primary-on-primary".into(),
            "ui-toggle-tonal-button-secondary-container-on-secondary-container".into(),
            "ui-standard-button-on-surface-variant".into(),
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
                    label: Some(I18N.text(match placement {
                        Placement::Docked => "ui-docked-toolbar-at-the-bottom-of-the-window",
                        Placement::Horizontal => "ui-horizontal-floating-toolbar-margins",
                        Placement::Vertical => "ui-vertical-floating-toolbar-margins",
                    })),
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
                                                label: Some(I18N.text("ui-navigation-rail")),
                                                ..Default::default()
                                            }),
                                    )
                                    .cross_axis_alignment(CrossAxisAlignment::Center)
                                    .children(move || {
                                        for vector in
                                            [filled::INBOX_SVG, filled::STAR_SVG, filled::SEND_SVG]
                                        {
                                            icon().painter(vector).size(Dp(24.0));
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
    let message = I18N.format("ui-value-activated", [("v0", I18N.text(&label).into())]);
    if filled {
        icon_button()
            .modifier(modifier)
            .icon(vector)
            .accessibility_label(I18N.text(&(label)))
            .on_click(move || notice.set(message.clone()))
            .filled();
    } else {
        icon_button()
            .modifier(modifier)
            .icon(vector)
            .accessibility_label(I18N.text(&(label)))
            .on_click(move || notice.set(message.clone()))
            .standard();
    }
}

#[tessera]
fn docked_layout_figure(layout: DockedLayout) {
    let width = remember(|| Dp::ZERO);
    let notice = remember(String::new);
    let labels: Vec<&'static str> = if compact(width.get()) {
        vec!["ui-back", "ui-add", "ui-more-options"]
    } else {
        vec![
            "ui-back",
            "ui-forward",
            "ui-add",
            "ui-picture-in-picture",
            "ui-more-options",
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
                        .accessibility_label(I18N.text(match layout {
                            DockedLayout::Edges => "ui-edge-aligned-docked-toolbar",
                            DockedLayout::Centered => "ui-center-aligned-docked-toolbar",
                        }))
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
                                                layout_action(
                                                    index,
                                                    label,
                                                    label == "ui-add",
                                                    notice,
                                                );
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
                                                let center = label == "ui-add";
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
                .accessibility_label(I18N.text("ui-mailbox-destinations"))
                .content(move || {
                    for label in ["ui-inbox", "ui-starred"] {
                        button()
                            .text()
                            .accessibility_label(I18N.text(label))
                            .on_click(move || {
                                notice.set(I18N.format(
                                    "ui-value-selected",
                                    [("v0", I18N.text(&label).into())],
                                ));
                                open.set(false);
                            })
                            .child(move || {
                                text(I18N.text(label));
                            });
                    }
                    icon_button()
                        .icon(filled::MORE_VERT_SVG)
                        .accessibility_label(I18N.text("ui-more-destinations"))
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
                    .accessibility_label(I18N.text("ui-more-destinations"))
                    .child(move || {
                        column()
                            .modifier(Modifier::new().padding_symmetric(Dp::ZERO, Dp(8.0)))
                            .children(move || {
                                for label in ["ui-sent", "ui-drafts", "ui-spam"] {
                                    button()
                                        .text()
                                        .accessibility_label(I18N.text(label))
                                        .on_click(move || {
                                            notice.set(I18N.format(
                                                "ui-value-selected",
                                                [("v0", I18N.text(&label).into())],
                                            ));
                                            open.set(false);
                                        })
                                        .child(move || {
                                            text(I18N.text(label));
                                        });
                                }
                            });
                    });
            }
        }),
    );
    gap(Dp(12.0));
    caption(if notice.get().is_empty() {
        "ui-toolbar-actions-can-open-a-menu".to_string()
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
                .accessibility_label(I18N.text("ui-toggle-window-size"))
                .on_click(move || compact_window.set(!compact_window.get()))
                .child(move || {
                    text(I18N.text(if compact_window.get() {
                        "ui-show-larger-window"
                    } else {
                        "ui-show-compact-window"
                    }));
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
                "ui-in-ltr-languages-the-next-button-is-intentionally-placed-on-the-trailing-right-edge",
            );
            gap(Dp(24.0));
            text_nav_toolbar(true).notice(notice);
            gap(Dp(12.0));
            caption(
                "ui-in-rtl-languages-reverse-the-order-so-next-remains-on-the-trailing-edge-now-on-the-left",
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
                    label: Some(I18N.text(if fab {
                        "ui-bottom-app-bar-with-fab"
                    } else {
                        "ui-bottom-app-bar"
                    })),
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
                    for label in ["ui-archive", "ui-delete", "ui-mark-unread", "ui-snooze"]
                        .into_iter()
                        .take(icons)
                    {
                        icon_button()
                            .icon(action_icon(label))
                            .accessibility_label(I18N.text(label))
                            .on_click(move || {
                                notice.set(I18N.format(
                                    "ui-value-activated",
                                    [("v0", I18N.text(&label).into())],
                                ))
                            })
                            .standard();
                    }
                    if fab {
                        spacer().modifier(Modifier::new().weight(1.0));
                        floating_action_button()
                            .accessibility_label(I18N.text("ui-compose"))
                            .on_click(move || notice.set("ui-compose-activated".into()))
                            .content(|| {
                                icon().painter(filled::ADD_SVG);
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
            gallery_item(0).fill(true).legend(1).content(move |_| {
                bottom_bar(icons, true, notice);
            });
            gap(Dp(24.0));
            gallery_item(1).fill(true).legend(2).content(move |_| {
                bottom_bar(4, false, notice);
            });
        }),
    );
    gap(Dp(16.0));
    legend(vec![
        "ui-icon-buttons-and-fab".into(),
        "ui-icon-buttons-and-no-fab".into(),
    ]);
    if !notice.get().is_empty() {
        gap(Dp(12.0));
        caption(notice.get());
    }
}

#[tessera]
fn overview() {
    bullets(vec![
        "ui-two-expressive-variants-docked-toolbar-and-floating-toolbar".into(),
        "ui-use-the-vibrant-color-style-for-greater-emphasis".into(),
        "ui-can-display-a-wide-variety-of-control-types-like-buttons-icon-buttons-and-text-fields"
            .into(),
        "ui-can-be-paired-with-fabs-to-emphasize-certain-actions".into(),
        "ui-don-t-show-at-the-same-time-as-a-navigation-bar".into(),
    ]);
    gap(Dp(32.0));
    gallery(GalleryKind::Overview);
    gap(Dp(16.0));
    caption("ui-configurations-of-floating-toolbars");
}

#[tessera]
fn spec_variants() {
    gallery(GalleryKind::Variants);
    gap(Dp(16.0));
    legend(vec![
        "ui-docked-toolbar".into(),
        "ui-floating-toolbar".into(),
    ]);
    gap(Dp(32.0));
    subheading("ui-baseline-variant".into());
    gap(Dp(16.0));
    prose(
        "ui-the-baseline-bottom-app-bar-is-no-longer-recommended-it-should-be-replaced-with-the-docked",
    );
    gap(Dp(24.0));
    baseline_figure();
    gap(Dp(16.0));
    caption("ui-bottom-app-bar-not-recommended");
    gap(Dp(32.0));
    prose(
        "ui-note-implementation-differs-per-platform-in-tessera-docked-toolbar-and-floating-toolb",
    );
}

#[tessera]
fn spec_configurations() {
    gallery(GalleryKind::Configurations);
    gap(Dp(16.0));
    legend(vec![
        "ui-standard-and-vibrant-toolbars".into(),
        "ui-vertical-floating-toolbar".into(),
        "ui-floating-toolbar-with-fab".into(),
    ]);
    gap(Dp(32.0));
    prose(
        "ui-note-implementation-differs-per-platform-in-tessera-the-fab-is-placed-next-to-the-floating",
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
            ("ui-toolbar-standard-container-color", "Surface container"),
            (
                "ui-toolbar-standard-button-container-color",
                "Surface container",
            ),
            (
                "ui-toolbar-standard-selected-button-container-color",
                "Secondary container",
            ),
            ("ui-toolbar-standard-icon-color", "On surface variant"),
            (
                "ui-toolbar-standard-selected-icon-color",
                "On secondary container",
            ),
            ("ui-toolbar-standard-label-color", "On surface variant"),
            (
                "ui-toolbar-standard-selected-label-color",
                "On secondary container",
            ),
            ("ui-toolbar-standard-container-shape", "ui-full"),
            ("ui-toolbar-standard-disabled-icon-color", "On surface"),
            ("ui-toolbar-standard-disabled-icon-opacity", "0.38"),
        ],
        TokenSet::Vibrant => &[
            ("ui-toolbar-vibrant-container-color", "Primary container"),
            (
                "ui-toolbar-vibrant-button-container-color",
                "Primary container",
            ),
            (
                "ui-toolbar-vibrant-selected-button-container-color",
                "Surface container",
            ),
            ("ui-toolbar-vibrant-icon-color", "On primary container"),
            ("ui-toolbar-vibrant-selected-icon-color", "On surface"),
            ("ui-toolbar-vibrant-label-color", "On primary container"),
            ("ui-toolbar-vibrant-selected-label-color", "On surface"),
            ("ui-toolbar-vibrant-container-shape", "ui-full"),
        ],
        TokenSet::Docked => &[
            ("ui-docked-toolbar-container-color", "Surface container"),
            ("ui-docked-toolbar-container-height", "64dp"),
            ("ui-docked-toolbar-leading-padding", "16dp"),
            ("ui-docked-toolbar-trailing-padding", "16dp"),
            ("ui-docked-toolbar-max-space-between-actions", "32dp"),
            ("ui-docked-toolbar-min-space-between-actions", "4dp"),
            ("ui-docked-toolbar-container-shape", "ui-none-rectangle"),
        ],
        TokenSet::Floating => &[
            ("ui-floating-toolbar-standard-color", "Surface container"),
            ("ui-floating-toolbar-vibrant-color", "Primary container"),
            ("ui-floating-toolbar-container-height-horizontal", "64dp"),
            ("ui-floating-toolbar-container-width-vertical", "64dp"),
            ("ui-floating-toolbar-leading-space", "8dp"),
            ("ui-floating-toolbar-trailing-space", "8dp"),
            (
                "ui-floating-toolbar-margin-from-screen-edge-horizontal",
                "16dp",
            ),
            (
                "ui-floating-toolbar-margin-from-screen-edge-vertical",
                "24dp",
            ),
            ("ui-floating-toolbar-space-between-actions", "4dp"),
            ("ui-floating-toolbar-container-shape", "ui-full"),
            (
                "ui-floating-toolbar-container-elevation",
                "ui-level-3-6dp-6307",
            ),
        ],
        TokenSet::FloatingFab => &[
            (
                "ui-toolbar-floating-fab-space-between-toolbar-and-fab",
                "8dp",
            ),
            (
                "ui-toolbar-floating-fab-standard-container-color",
                "Secondary container",
            ),
            (
                "ui-toolbar-floating-fab-standard-icon-color",
                "On secondary container",
            ),
            (
                "ui-toolbar-floating-fab-vibrant-container-color",
                "Tertiary container",
            ),
            (
                "ui-toolbar-floating-fab-vibrant-icon-color",
                "On tertiary container",
            ),
            (
                "ui-toolbar-floating-fab-container-size-expanded",
                "56dp × 56dp",
            ),
            ("ui-toolbar-floating-fab-icon-size-expanded", "24dp"),
            (
                "ui-toolbar-floating-fab-medium-container-size-collapsed",
                "80dp × 80dp",
            ),
            ("ui-toolbar-floating-fab-medium-icon-size-collapsed", "28dp"),
        ],
    };
    rows.iter()
        .map(|(token, value)| vec![(*token).to_string(), (*value).to_string()])
        .collect()
}

fn token_set_label(set: TokenSet) -> &'static str {
    match set {
        TokenSet::Standard => "ui-toolbar-color-standard",
        TokenSet::Vibrant => "ui-toolbar-color-vibrant",
        TokenSet::Docked => "ui-toolbar-docked",
        TokenSet::Floating => "ui-toolbar-floating",
        TokenSet::FloatingFab => "ui-toolbar-floating-fab",
    }
}

#[tessera]
fn token_chip(set: TokenSet, selected: State<TokenSet>) {
    let label = token_set_label(set);
    if selected.get() == set {
        button()
            .tonal()
            .selected(true)
            .accessibility_label(I18N.text(label))
            .on_click(move || selected.set(set))
            .child(move || {
                text(I18N.text(label));
            });
    } else {
        button()
            .outlined()
            .selected(false)
            .accessibility_label(I18N.text(label))
            .on_click(move || selected.set(set))
            .child(move || {
                text(I18N.text(label));
            });
    }
}

#[tessera]
fn tokens() {
    let selected = remember(TokenSet::default);
    prose(
        "ui-browse-the-component-elements-attributes-tokens-and-their-values-color-tokens-resolve-thro",
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
        vec!["ui-token".into(), "ui-value".into()],
        token_rows(selected.get()),
    );
}

#[tessera]
fn spec_anatomy() {
    figure(Variant::Floating, ToolbarColorStyle::Standard, false);
    gap(Dp(32.0));
    subheading("ui-flexibility-slots".into());
    gap(Dp(16.0));
    prose(
        "ui-when-configuring-a-toolbar-think-of-it-as-a-container-with-several-slots-each-slot-can-be",
    );
    gap(Dp(24.0));
    slots();
    gap(Dp(16.0));
    caption("ui-a-toolbar-is-essentially-a-container-with-configurable-slots");
    gap(Dp(24.0));
    search_example();
}

#[tessera]
fn spec_color() {
    prose(
        "ui-color-values-are-implemented-through-design-tokens-for-design-this-means-working-with-colo",
    );
    gap(Dp(32.0));
    subheading("ui-standard".into());
    gap(Dp(16.0));
    prose("ui-standard-color-schemes-and-icon-button-types");
    gap(Dp(16.0));
    color_roles(false);
    gap(Dp(32.0));
    subheading("ui-vibrant".into());
    gap(Dp(16.0));
    prose("ui-vibrant-color-scheme-and-icon-button-types");
    gap(Dp(16.0));
    color_roles(true);
}

#[tessera]
fn measurements() {
    prose(
        "ui-by-default-all-toolbars-are-64dp-high-center-aligned-have-equal-padding-between-items-and",
    );
    gap(Dp(32.0));
    subheading("ui-docked-toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Docked, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("ui-default-margins-and-padding");
    gap(Dp(32.0));
    prose("ui-alignment-and-padding-can-be-configured-to-create-unique-layouts");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Edges);
    gap(Dp(16.0));
    caption("ui-left-and-right-alignment");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Centered);
    gap(Dp(16.0));
    caption("ui-center-aligned-8dp-padding-between-items");
    gap(Dp(32.0));
    subheading("ui-floating-toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Floating, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("ui-default-padding-of-floating-toolbar");
    gap(Dp(24.0));
    figure(Variant::WithFab, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("ui-floating-toolbar-size-and-padding-measurements");
    gap(Dp(24.0));
    placement_figure(Placement::Horizontal);
    gap(Dp(16.0));
    placement_figure(Placement::Vertical);
    gap(Dp(16.0));
    caption("ui-floating-toolbar-margins");
    gap(Dp(32.0));
    spec_table(
        vec!["ui-attribute".into(), "ui-value".into()],
        vec![
            vec!["ui-container-height-vertical-width".into(), "64dp".into()],
            vec!["ui-floating-internal-padding".into(), "8dp".into()],
            vec!["ui-floating-space-between-actions".into(), "4dp".into()],
            vec!["ui-docked-leading-trailing-padding".into(), "16dp".into()],
            vec![
                "ui-docked-space-between-actions".into(),
                "4dp - 32dp".into(),
            ],
            vec![
                "ui-horizontal-outside-margins".into(),
                "ui-at-least-16dp".into(),
            ],
            vec![
                "ui-vertical-outside-margins".into(),
                "ui-at-least-24dp".into(),
            ],
            vec!["ui-fab-spacing".into(), "8dp".into()],
            vec!["ui-interactive-target".into(), "ui-at-least-48-48dp".into()],
        ],
    );
}

#[tessera]
fn baseline_specs() {
    prose(
        "ui-the-baseline-bottom-app-bar-is-no-longer-recommended-use-the-docked-toolbar-for-new-page-a",
    );
    gap(Dp(24.0));
    baseline_figure();
    gap(Dp(16.0));
    caption("ui-container");
    gap(Dp(32.0));
    subheading("ui-tokens-specs".into());
    gap(Dp(16.0));
    prose("ui-bottom-app-bar-tokens-are-in-one-token-set");
    gap(Dp(16.0));
    spec_table(
        vec!["ui-token".into(), "ui-value".into()],
        vec![
            vec![
                "ui-bottom-app-bar-container-color".into(),
                "Surface container".into(),
            ],
            vec!["ui-bottom-app-bar-container-height".into(), "80dp".into()],
            vec![
                "ui-bottom-app-bar-container-shape".into(),
                "ui-none-rectangle".into(),
            ],
            vec!["ui-bottom-app-bar-icon-button-target".into(), "48dp".into()],
            vec![
                "ui-bottom-app-bar-icon-color".into(),
                "On surface variant".into(),
            ],
            vec!["ui-bottom-app-bar-fab-size".into(), "56dp".into()],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-color".into());
    gap(Dp(16.0));
    prose(
        "ui-color-values-are-implemented-through-design-tokens-bottom-app-bar-color-role-used-for-ligh",
    );
    gap(Dp(32.0));
    subheading("ui-measurements".into());
    gap(Dp(16.0));
    prose(
        "ui-the-baseline-bottom-app-bar-uses-an-80dp-container-with-48dp-action-targets-a-4dp-leading",
    );
    gap(Dp(32.0));
    subheading("ui-common-layouts".into());
    gap(Dp(24.0));
    baseline_layouts();
}

#[tessera]
fn usage() {
    prose(
        "ui-use-a-toolbar-to-provide-actions-related-to-the-current-page-toolbars-can-contain-many-act",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Elevation);
    gap(Dp(16.0));
    caption("ui-a-toolbar-provides-actions-related-to-the-current-page");
    gap(Dp(32.0));
    prose("ui-there-are-two-variants-of-toolbars");
    gap(Dp(8.0));
    bullets(vec![
        "ui-docked-toolbar-spans-the-full-width-of-the-window-it-s-best-used-for-global-actions-that-r".into(),
        "ui-floating-toolbar-floats-above-the-body-content-it-s-best-used-for-contextual-actions-relev".into(),
    ]);
    gap(Dp(8.0));
    prose("ui-the-baseline-bottom-app-bar-is-no-longer-recommended-but-is-still-supported");
    gap(Dp(24.0));
    gallery(GalleryKind::DockedVsFloating);
    gap(Dp(32.0));
    prose("ui-when-actions-don-t-fit-in-a-toolbar-add-a-menu");
    gap(Dp(24.0));
    overflow_example();
    gap(Dp(32.0));
    prose("ui-there-are-two-color-configurations");
    gap(Dp(8.0));
    bullets(vec![
        "ui-standard-a-low-emphasis-color-scheme-best-used-for-focusing-attention-on-the-body-content".into(),
        "ui-vibrant-a-high-emphasis-color-scheme-that-draws-attention-to-the-controls-it-can-also-indi".into(),
    ]);
    gap(Dp(8.0));
    prose(
        "ui-consider-using-alternative-color-roles-to-create-greater-or-lesser-emphasis-depending-on-t",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::ColorEmphasis);
    gap(Dp(32.0));
    subheading("ui-toolbars-navigation-bars".into());
    gap(Dp(16.0));
    prose(
        "ui-the-toolbar-and-navigation-bar-are-both-placed-at-the-bottom-of-the-window-so-should-not-b",
    );
    gap(Dp(24.0));
    navigation_diagram(NavDiagram::Pages);
    gap(Dp(32.0));
    prose(
        "ui-floating-toolbars-can-be-used-as-tabs-between-related-subsequent-pages-in-the-product-hier",
    );
    gap(Dp(24.0));
    navigation_diagram(NavDiagram::LocalNavigation);
    gap(Dp(16.0));
    do_note(
        "ui-keep-navigation-distinct-and-use-a-toolbar-to-display-local-navigation-on-a-specific-page",
    );
    gap(Dp(12.0));
    prose(
        "ui-consider-the-existing-app-hierarchy-when-using-a-toolbar-for-local-navigation-avoid-redund",
    );
    gap(Dp(12.0));
    dont_note(
        "ui-don-t-show-a-navigation-bar-and-a-toolbar-with-navigation-controls-at-the-same-time",
    );
}

#[tessera]
fn guide_anatomy() {
    figure(Variant::Floating, ToolbarColorStyle::Standard, false);
    gap(Dp(32.0));
    subheading("ui-container".into());
    gap(Dp(16.0));
    prose(
        "ui-the-docked-toolbar-s-container-spans-the-full-width-of-the-window-avoid-applying-rounded-c",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Corners);
    gap(Dp(32.0));
    prose(
        "ui-as-long-as-there-s-a-minimum-of-16dp-padding-on-the-leading-and-trailing-edge-arrange-cont",
    );
    gap(Dp(16.0));
    prose("ui-be-cautious-of-including-too-many-controls-as-it-can-be-overwhelming");
    gap(Dp(12.0));
    dont_note("ui-don-t-overwhelm-people-with-too-many-controls");
    gap(Dp(32.0));
    prose(
        "ui-the-floating-toolbar-s-container-should-be-fully-visible-on-screen-if-more-actions-are-nee",
    );
    gap(Dp(12.0));
    do_note("ui-choose-the-most-essential-actions-to-show-on-screen-by-default");
    gap(Dp(12.0));
    dont_note("ui-floating-toolbars-shouldn-t-exceed-the-edge-of-the-window-or-pane");
    gap(Dp(32.0));
    subheading("ui-elevation".into());
    gap(Dp(16.0));
    prose(
        "ui-floating-toolbars-have-elevation-by-default-if-the-content-beneath-the-toolbar-is-visually",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Elevation);
    gap(Dp(16.0));
    caption(
        "ui-the-elevation-on-floating-toolbars-can-be-removed-if-on-a-visually-distinct-background",
    );
    gap(Dp(32.0));
    subheading("ui-flexibility-slots".into());
    gap(Dp(16.0));
    prose(
        "ui-when-configuring-a-toolbar-think-of-it-as-a-container-with-several-slots-these-slots-can-b",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Slots);
    gap(Dp(16.0));
    caption("ui-toolbars-are-made-of-slots-that-can-contain-many-kinds-of-actions");
    gap(Dp(32.0));
    prose(
        "ui-visually-emphasizing-a-single-action-more-than-others-is-an-effective-way-to-create-hierar",
    );
    gap(Dp(8.0));
    bullets(vec![
        "ui-use-different-icon-button-color-styles-such-as-filled-tonal-and-standard".into(),
        "ui-customize-the-color-roles-of-a-single-action-such-as-a-primary-or-secondary-palette"
            .into(),
        "ui-use-wide-and-narrow-icon-buttons".into(),
        "ui-pair-the-toolbar-with-a-fab".into(),
    ]);
    gap(Dp(24.0));
    gallery(GalleryKind::Emphasis);
    gap(Dp(16.0));
    caption("ui-two-different-ways-to-create-a-high-emphasis-action-in-toolbars");
    gap(Dp(32.0));
    gallery(GalleryKind::EmphasisDont);
    gap(Dp(12.0));
    dont_note(
        "ui-don-t-emphasize-multiple-buttons-with-bold-primary-colors-such-as-a-button-and-fab-togethe",
    );
    gap(Dp(12.0));
    dont_note(
        "ui-avoid-mixing-too-many-different-controls-in-the-same-toolbar-a-consistent-control-design-k",
    );
    gap(Dp(12.0));
    prose(
        "ui-avoid-using-square-icon-buttons-in-floating-toolbars-their-square-shape-conflicts-with-the",
    );
    gap(Dp(12.0));
    dont_note("ui-don-t-use-square-filled-icon-buttons-in-floating-toolbars");
    gap(Dp(32.0));
    subheading("ui-floating-toolbar-with-fab".into());
    gap(Dp(16.0));
    prose(
        "ui-a-fab-can-be-placed-next-to-a-floating-toolbar-to-present-one-high-priority-action-alongsi",
    );
    gap(Dp(24.0));
    figure(Variant::WithFab, ToolbarColorStyle::Vibrant, false);
    gap(Dp(16.0));
    caption("ui-floating-toolbars-can-be-paired-with-fabs");
}

#[tessera]
fn position() {
    prose(
        "ui-only-place-docked-toolbars-at-the-bottom-of-the-window-if-using-other-bottom-aligned-eleme",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Docked);
    gap(Dp(16.0));
    caption("ui-docked-toolbars-are-always-at-the-bottom-of-the-window");
    gap(Dp(32.0));
    prose(
        "ui-floating-toolbars-can-be-horizontal-or-vertical-horizontal-toolbars-should-have-a-minimum",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Horizontal);
    gap(Dp(16.0));
    caption("ui-horizontal-floating-toolbars-should-be-at-least-16dp-from-the-edge-of-the-window");
    gap(Dp(32.0));
    prose(
        "ui-in-larger-breakpoints-floating-toolbars-can-be-vertical-and-placed-on-either-side-of-the-s",
    );
    gap(Dp(24.0));
    figure(Variant::Vertical, ToolbarColorStyle::Standard, true);
    gap(Dp(16.0));
    caption("ui-maintain-at-least-a-24dp-margin-for-vertical-toolbars");
    gap(Dp(32.0));
    prose(
        "ui-to-keep-vertical-toolbars-compact-don-t-use-wide-icon-buttons-use-narrow-or-default-icon-b",
    );
    gap(Dp(12.0));
    dont_note(
        "ui-using-wide-buttons-with-vertical-toolbars-can-unnecessarily-widen-toolbar-containers-and-h",
    );
    gap(Dp(32.0));
    prose(
        "ui-vertical-toolbars-should-be-positioned-opposite-the-navigation-rail-to-balance-out-the-scr",
    );
    gap(Dp(24.0));
    placement_figure(Placement::Vertical);
    gap(Dp(16.0));
    caption(
        "ui-when-a-nav-rail-is-visible-the-floating-toolbar-should-be-vertical-on-the-opposite-edge-of",
    );
}

#[tessera]
fn adaptive() {
    prose(
        "ui-adaptive-design-allows-an-interface-to-respond-or-change-based-on-context-such-as-the-user",
    );
    gap(Dp(32.0));
    subheading("ui-resizing".into());
    gap(Dp(16.0));
    prose(
        "ui-docked-the-docked-toolbar-should-always-span-100-of-the-screen-width-in-compact-breakpoint",
    );
    gap(Dp(24.0));
    figure(Variant::Docked, ToolbarColorStyle::Standard, false);
    gap(Dp(16.0));
    caption("ui-docked-toolbar-items-should-be-evenly-spaced-in-compact-windows");
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Centered);
    gap(Dp(16.0));
    caption(
        "ui-in-medium-breakpoints-and-larger-create-a-spacious-layout-by-centering-all-elements",
    );
    gap(Dp(24.0));
    docked_layout_figure(DockedLayout::Edges);
    gap(Dp(16.0));
    caption(
        "ui-align-controls-to-the-edge-of-the-screen-to-make-them-easier-to-reach-on-tablets-and-to-be",
    );
    gap(Dp(32.0));
    prose(
        "ui-on-web-and-large-screens-the-docked-toolbar-can-be-rounded-dividers-can-be-used-to-organiz",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::RoundedDocked);
    gap(Dp(16.0));
    caption(
        "ui-on-web-and-other-large-screens-docked-toolbars-can-be-rounded-and-placed-in-different-part",
    );
    gap(Dp(32.0));
    prose(
        "ui-floating-the-container-should-only-be-as-big-as-needed-to-hold-the-items-inside-before-rea",
    );
    gap(Dp(12.0));
    dont_note("ui-don-t-add-extra-space-to-a-toolbar-beyond-its-necessary-items");
    gap(Dp(24.0));
    overflow_collapse_example();
    gap(Dp(16.0));
    caption(
        "ui-at-larger-screen-sizes-the-container-can-display-more-controls-before-hitting-the-16dp-mar",
    );
    gap(Dp(32.0));
    prose(
        "ui-vertical-toolbars-aren-t-recommended-for-compact-windows-they-take-up-a-significant-area-o",
    );
    gap(Dp(12.0));
    caution_note("ui-vertical-toolbars-can-cover-important-content-in-compact-windows");
    gap(Dp(32.0));
    subheading("ui-presentation".into());
    gap(Dp(16.0));
    prose(
        "ui-in-larger-breakpoints-floating-toolbars-can-be-aligned-to-opposite-edges-of-the-screen-so",
    );
    gap(Dp(24.0));
    gallery(GalleryKind::Presentation);
    gap(Dp(16.0));
    caption(
        "ui-multiple-toolbars-with-different-stylistic-treatments-can-create-hierarchy-and-distinguish",
    );
    gap(Dp(24.0));
    prose(
        "ui-don-t-use-multiple-toolbars-in-compact-windows-there-typically-isn-t-enough-room-on-screen",
    );
    gap(Dp(12.0));
    dont_note("ui-avoid-using-multiple-toolbars-in-smaller-windows");
    gap(Dp(24.0));
    prose(
        "ui-actions-at-the-trailing-edge-of-the-toolbar-can-collapse-into-an-overflow-menu-at-smaller",
    );
    gap(Dp(32.0));
    subheading("ui-right-to-left-languages".into());
    gap(Dp(16.0));
    prose(
        "ui-in-right-to-left-rtl-languages-mirror-individual-items-that-need-it-like-icons-and-text-di",
    );
    gap(Dp(24.0));
    rtl_example();
}

#[tessera]
fn behavior() {
    subheading("ui-scrolling".into());
    gap(Dp(16.0));
    prose(
        "ui-docked-toolbars-can-either-remain-on-the-screen-during-scroll-or-animate-offscreen-floatin",
    );
    gap(Dp(16.0));
    prose(
        "ui-use-the-controls-below-to-move-the-toolbar-offscreen-or-collapse-it-to-its-key-action-only",
    );
    gap(Dp(24.0));
    collapsing_example();
    gap(Dp(16.0));
    dont_note("ui-toolbars-shouldn-t-both-collapse-and-transition-off-page");
}

#[tessera]
fn use_cases() {
    prose("ui-people-should-be-able-to-do-the-following-with-assistive-technology");
    gap(Dp(8.0));
    bullets(vec![
        "ui-navigate-and-activate-any-actions-in-the-toolbar".into(),
        "ui-select-a-destination-from-a-menu".into(),
        "ui-activate-a-back-button".into(),
        "ui-maintain-access-to-toolbar-controls-when-the-content-is-scrolled-or-collapsed".into(),
    ]);
}

#[tessera]
fn interaction_style() {
    prose(
        "ui-the-toolbar-has-no-interactions-by-default-all-interactions-are-with-the-elements-placed-i",
    );
    gap(Dp(32.0));
    subheading("ui-touch".into());
    gap(Dp(16.0));
    prose(
        "ui-when-tapping-on-an-icon-button-in-the-toolbar-a-touch-ripple-appears-indicating-interactio",
    );
    gap(Dp(32.0));
    subheading("ui-cursor".into());
    gap(Dp(16.0));
    prose(
        "ui-when-hovered-the-hover-state-provides-a-visual-cue-to-the-user-that-the-element-is-interac-6431",
    );
    gap(Dp(32.0));
    subheading("ui-initial-focus".into());
    gap(Dp(16.0));
    prose(
        "ui-focus-lands-on-the-first-interactive-element-use-tab-to-navigate-through-all-other-actions",
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
        "ui-use-tab-to-navigate-through-interactive-elements-use-space-or-enter-to-activate-actions"
            .to_string()
    } else {
        notice.get()
    });
}

#[tessera]
fn keyboard() {
    spec_table(
        vec!["ui-keys".into(), "ui-actions".into()],
        vec![
            vec![
                "ui-tab-or-arrows".into(),
                "ui-navigate-between-interactive-elements".into(),
            ],
            vec![
                "ui-space-or-enter".into(),
                "ui-activate-the-focused-element".into(),
            ],
        ],
    );
    gap(Dp(32.0));
    subheading("ui-labeling-elements".into());
    gap(Dp(16.0));
    prose(
        "ui-on-web-the-toolbar-container-should-have-the-toolbar-role-on-mobile-it-can-be-a-generic-co",
    );
    gap(Dp(16.0));
    prose(
        "ui-tessera-exposes-the-toolbar-role-on-the-container-labels-it-and-labels-each-action-toggle",
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
                            text((index).to_string()).style(type_style(16.0, 24.0));
                        });
                }
            });
    }));
}

#[tessera]
fn search_toolbar(notice: State<String>, gallery_tag: Option<usize>) {
    let controller = remember(|| {
        tessera_components::text_input::TextInputController::new(Dp(16.0), Some(Dp(24.0)))
    });
    let selected = remember(|| false);
    let tag = gallery_tag;
    docked_toolbar()
        .accessibility_label(I18N.text("ui-search-actions"))
        .content(move || {
            row()
                .modifier(Modifier::new().fill_max_width())
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(move || {
                    sample_action("ui-back", Emphasis::Standard, false, selected, notice)
                        .tag_optional(tag.map(|item| format!("toolbar-gallery-{item}-first")));
                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                    boxed()
                        .modifier(Modifier::new().weight(1.0))
                        .children(move || {
                            text_field()
                                .placeholder(I18N.text("ui-search-mail"))
                                .accessibility_label(I18N.text("ui-search-mail"))
                                .shape(Shape::CAPSULE)
                                .min_width(Dp::ZERO)
                                .min_height(Dp(40.0))
                                .controller(controller)
                                .on_submit(move || {
                                    notice.set(
                                        I18N.format(
                                            "ui-searched-for-value",
                                            [(
                                                "v0",
                                                (controller.with(|controller| controller.text()))
                                                    .to_string()
                                                    .into(),
                                            )],
                                        ),
                                    )
                                })
                                .modifier(Modifier::new().fill_max_width());
                        });
                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                    sample_action("ui-microphone", Emphasis::Standard, false, selected, notice)
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
        "ui-a-text-field-placed-in-a-toolbar-slot-between-icon-buttons".to_string()
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
                for (index, label) in ["ui-all", "ui-unread", "ui-starred"]
                    .into_iter()
                    .enumerate()
                {
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
                            text(I18N.text(label)).style(type_style(12.0, 16.0)).color(
                                if selected {
                                    scheme.on_surface
                                } else {
                                    scheme.on_primary_container
                                },
                            );
                        });
                }
            } else {
                for (index, label) in ["ui-archive", "ui-delete", "ui-reply", "ui-more-options"]
                    .into_iter()
                    .enumerate()
                {
                    if index > 0 {
                        spacer().modifier(Modifier::new().width(Dp(12.0)));
                    }
                    icon()
                        .painter(action_icon(label))
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
            for (index, label) in ["ui-mail", "ui-chat", "ui-settings"]
                .into_iter()
                .enumerate()
            {
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
                                icon().painter(action_icon(label)).size(Dp(18.0)).tint(
                                    if selected {
                                        scheme.on_secondary_container
                                    } else {
                                        scheme.on_surface_variant
                                    },
                                );
                            });
                        text(I18N.text(label))
                            .style(type_style(11.0, 14.0))
                            .color(scheme.on_surface_variant);
                    });
            }
        });
}

#[tessera]
fn phone_mock(#[prop(into)] title: String, toolbar: MockBar, nav_bar: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let description = I18N.format(
        "ui-value-screenvaluevalue",
        [
            ("v0", I18N.text(&title).into()),
            (
                "v1",
                I18N.text(
                    &(match toolbar {
                        MockBar::None => "",
                        MockBar::Actions => "ui-with-a-floating-toolbar-of-actions",
                        MockBar::Tabs => "ui-with-a-floating-toolbar-of-local-navigation",
                    }),
                )
                .into(),
            ),
            (
                "v2",
                I18N.text(
                    &(if nav_bar {
                        "ui-above-a-navigation-bar"
                    } else {
                        ""
                    }),
                )
                .into(),
            ),
        ],
    );
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .size(Dp(208.0), Dp(308.0))
                .background_with_shape(scheme.outline_variant, Shape::rounded_rectangle(Dp(28.0)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(I18N.text(description)),
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
                    text(I18N.text(&(title)))
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
                        "ui-inbox",
                        MockBar::None,
                        true,
                        "ui-navigation-bar-on-a-primary-page",
                    );
                    mock_screen(
                        "ui-message",
                        MockBar::Actions,
                        false,
                        "ui-toolbar-on-a-secondary-page-with-contextual-actions",
                    );
                }
                NavDiagram::LocalNavigation => {
                    mock_screen(
                        "ui-inbox",
                        MockBar::Tabs,
                        false,
                        "ui-do-use-a-toolbar-to-display-local-navigation-on-a-specific-page",
                    );
                    mock_screen(
                        "ui-inbox",
                        MockBar::Tabs,
                        true,
                        "ui-don-t-show-a-navigation-bar-and-a-toolbar-with-navigation-controls-at-the-same-time-6443",
                    );
                }
            }));
        }),
    );
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
                        .accessibility_label(I18N.text("ui-toggle-toolbar-expansion"))
                        .on_click(move || expanded.set(!expanded.get()))
                        .child(move || {
                            text(I18N.text(if expanded.get() {
                                "ui-collapse-actions"
                            } else {
                                "ui-expand-actions"
                            }));
                        });
                    button()
                        .text()
                        .enabled(expanded.get())
                        .accessibility_label(I18N.text("ui-toggle-toolbar-visibility"))
                        .on_click(move || visible.set(!visible.get()))
                        .child(move || {
                            text(I18N.text(if visible.get() {
                                "ui-move-offscreen"
                            } else {
                                "ui-show-toolbar"
                            }));
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
            text(I18N.text(if notice.get().is_empty() {
                "ui-no-toolbar-action-yet".to_owned()
            } else {
                notice.get()
            }))
            .style(type_style(14.0, 20.0));
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {

    use tessera_components::theme::material_theme;
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer,
        accesskit::{Action, ActionRequest, Node, NodeId, Toggled, TreeId},
        testing::layout_test,
    };

    use super::*;

    #[tessera]
    fn fixture(variant: Variant, expanded: bool) {
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
    fn compact_figure_fixture(variant: Variant) {
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
            let snapshot = layout_test(move || compact_figure_fixture(variant))
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
            .find(|(_, node)| crate::has_label(&node, label))
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
            vec![Box::new(crate::TestPipelines)],
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
            let snapshot = layout_test(move || fixture(variant, expanded))
                .viewport_px(600, 400)
                .run();
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

    #[derive(Clone, Copy)]
    enum Scene {
        Gallery(GalleryKind),
        Placement(Placement),
        Docked(DockedLayout),
    }

    fn settled(scene: Scene, width: u32, height: u32) -> tessera_ui::testing::LayoutSnapshot {
        let mut session = tessera_ui::testing::start_layout_test_session(
            layout_test(move || {
                material_theme().child(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width())
                        .children(move || match scene {
                            Scene::Gallery(kind) => {
                                gallery(kind);
                            }
                            Scene::Placement(placement) => {
                                placement_figure(placement);
                            }
                            Scene::Docked(layout) => {
                                docked_layout_figure(layout);
                            }
                        });
                });
            })
            .viewport_px(width, height),
        );
        for frame in 0..3 {
            tessera_ui::testing::pump_layout_test_session(&mut session, frame * 16_000_000);
        }
        tessera_ui::testing::pump_layout_test_session(&mut session, 48_000_000)
    }

    #[test]
    fn toolbar_galleries_keep_every_configuration_and_target_inside_compact_and_medium_stages() {
        for (kind, items, fabs) in [
            (GalleryKind::Overview, 7, &[1usize][..]),
            (GalleryKind::Variants, 2, &[][..]),
            (GalleryKind::Configurations, 6, &[5][..]),
            (GalleryKind::UseCases, 5, &[2][..]),
            (GalleryKind::Slots, 5, &[][..]),
            (GalleryKind::Emphasis, 2, &[1][..]),
            (GalleryKind::EmphasisDont, 1, &[0][..]),
            (GalleryKind::Presentation, 2, &[][..]),
            (GalleryKind::RoundedDocked, 1, &[][..]),
        ] {
            for width in [320, 600] {
                let snapshot = settled(Scene::Gallery(kind), width, 1600);
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

    #[test]
    fn placement_figures_keep_docked_flush_and_floating_margins_from_the_window() {
        for width in [320, 720] {
            for placement in [
                Placement::Docked,
                Placement::Horizontal,
                Placement::Vertical,
            ] {
                let snapshot = settled(Scene::Placement(placement), width, 600);
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

    #[test]
    fn docked_alignment_figures_center_with_8dp_gaps_or_pin_actions_to_16dp_edges() {
        for width in [320, 720] {
            let count = if width < 400 { 3 } else { 5 };
            let snapshot = settled(Scene::Docked(DockedLayout::Centered), width, 400);
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

            let snapshot = settled(Scene::Docked(DockedLayout::Edges), width, 400);
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
        let first = action("first", Emphasis::Standard);
        let second = action("second", Emphasis::Toggle);
        let last = action("ui-more-options", Emphasis::Filled);
        assert_eq!(visible_actions(vec![first, second, last], 2), [first, last]);
        assert_eq!(visible_actions(vec![first, second], 1), [first]);
        assert_eq!(visible_actions(vec![first, second], 6), [first, second]);
    }
}
