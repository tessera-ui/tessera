//! Interactive expressive toolbar specifications.
//!
//! ## Usage
//!
//! Explore toolbar action groups, placement and keyboard navigation.

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    button::button,
    column::column,
    floating_action_button::floating_action_button,
    flow_row::flow_row,
    icon::icon,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
    toolbar::{ToolbarColorStyle, ToolbarOrientation, docked_toolbar, floating_toolbar},
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, remember, tessera, use_context,
};

use super::{
    annotations::{self, Geometry, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

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
    article("Toolbars", "Toolbars display frequently used actions relevant to the current page", vec![
        SpecSection { title: "Variants", content: RenderSlot::new(variants) },
        SpecSection { title: "Configurations", content: RenderSlot::new(configurations) },
        SpecSection { title: "Tokens & specs", content: RenderSlot::new(tokens) },
        SpecSection { title: "Anatomy", content: RenderSlot::new(|| {
            figure(Variant::Floating, ToolbarColorStyle::Standard, false);
            gap(Dp(32.0));
            subheading("Flexibility & slots".into());
            paragraph("A toolbar provides a container for placed components. Slots can contain icon buttons, buttons, text fields and other actions. Emphasize one primary action and keep related controls consistent.".into());
            gap(Dp(24.0));
            slots();
        }) },
        SpecSection { title: "Color", content: RenderSlot::new(|| {
            subheading("Standard".into());
            paragraph("Surface container; primary filled action; secondary container toggle; primary standard action.".into());
            gap(Dp(24.0));
            figure(Variant::Floating, ToolbarColorStyle::Standard, false);
            gap(Dp(32.0));
            subheading("Vibrant".into());
            paragraph("Primary container; primary filled action; surface container toggle; on primary container standard action.".into());
            gap(Dp(24.0));
            figure(Variant::Floating, ToolbarColorStyle::Vibrant, false);
        }) },
        SpecSection { title: "Measurements", content: RenderSlot::new(measurements) },
        SpecSection { title: "Bottom app bar (baseline)", content: RenderSlot::new(|| {
            paragraph("The baseline bottom app bar remains supported, but the expressive docked toolbar is recommended for new page actions. It uses a shorter, more flexible 64dp container.".into());
            gap(Dp(24.0));
            figure(Variant::Docked, ToolbarColorStyle::Standard, true);
        }) },
    ])
        .pages(vec![
            ArticlePage {
                title: "Overview",
                intro: Some(RenderSlot::new(|| {
                    paragraph("Use docked toolbars for persistent page actions and floating toolbars for contextual actions. Both support standard and vibrant colors; floating toolbars also support vertical layouts and an adjacent FAB.".into());
                    gap(Dp(32.0));
                    configurations();
                })),
                sections: vec![
                    SpecSection { title: "M3 Expressive update", content: RenderSlot::new(|| {
                        paragraph("Docked toolbar replaces the baseline bottom app bar with a shorter container and more flexible slots. Floating toolbar adds horizontal and vertical orientations, vibrant colors and optional FAB pairing.".into());
                        gap(Dp(24.0));
                        figure(Variant::WithFab, ToolbarColorStyle::Vibrant, false);
                    }) },
                    SpecSection { title: "Differences from M2", content: RenderSlot::new(|| {
                        paragraph("Material 3 introduces dynamic color roles and places the FAB alongside page actions. Expressive toolbars extend those actions with flexible content and adaptive placement.".into());
                    }) },
                ],
            },
            ArticlePage {
                title: "Guidelines",
                intro: None,
                sections: vec![
                    SpecSection { title: "Usage", content: RenderSlot::new(|| {
                        paragraph("Use toolbar actions for the current page. Docked spans the window; floating hugs its controls. Use an overflow menu when more actions are needed. Keep toolbars separate from primary navigation bars.".into());
                        gap(Dp(24.0));
                        variants();
                        gap(Dp(32.0));
                        subheading("Toolbars & navigation bars".into());
                        paragraph("Show navigation on primary pages and actions on subsequent pages. Do not combine a navigation bar and toolbar navigation in the same compact view.".into());
                    }) },
                    SpecSection { title: "Anatomy", content: RenderSlot::new(|| {
                        figure(Variant::Floating, ToolbarColorStyle::Standard, false);
                        gap(Dp(32.0));
                        subheading("Container".into());
                        paragraph("Keep docked containers rectangular. A floating container must remain entirely on screen, with at least 48×48dp targets for its actions.".into());
                        gap(Dp(24.0));
                        subheading("Elevation".into());
                        paragraph("Use elevation when needed to separate floating controls from underlying content. Keep visually distinct content and action surfaces clearly distinguishable.".into());
                        gap(Dp(24.0));
                        subheading("Flexibility & slots".into());
                        slots();
                        gap(Dp(24.0));
                        subheading("Floating toolbar with FAB".into());
                        paragraph("Pair one high-priority action with a unified set of toolbar controls. Avoid emphasizing several actions at once.".into());
                        gap(Dp(24.0));
                        figure(Variant::WithFab, ToolbarColorStyle::Vibrant, false);
                    }) },
                    SpecSection { title: "Position & orientation", content: RenderSlot::new(|| {
                        paragraph("Docked toolbars belong at the bottom. Horizontal floating toolbars need at least 16dp outside margins. Use vertical floating toolbars at larger breakpoints with at least 24dp margins, opposite a navigation rail.".into());
                        gap(Dp(24.0));
                        figure(Variant::Vertical, ToolbarColorStyle::Standard, true);
                    }) },
                    SpecSection { title: "Adaptive design", content: RenderSlot::new(|| {
                        subheading("Resizing".into());
                        paragraph("Docked toolbars fill the available width. Floating containers hug their actions and reveal more actions at larger breakpoints. Collapse trailing actions into overflow before the toolbar reaches its outside margins.".into());
                        gap(Dp(24.0));
                        figure(Variant::Docked, ToolbarColorStyle::Standard, true);
                        gap(Dp(32.0));
                        subheading("Presentation".into());
                        paragraph("Use one toolbar in compact windows. Larger windows can separate related groups on opposite edges. Keep actions within reach and avoid covering content.".into());
                        gap(Dp(24.0));
                        subheading("Right-to-left languages".into());
                        paragraph("Mirror directional icons and text. Reverse action order only when the sequence has meaning, keeping Next at the trailing edge.".into());
                    }) },
                    SpecSection { title: "Behavior", content: RenderSlot::new(|| {
                        subheading("Scrolling".into());
                        paragraph("Toolbars can remain visible, move offscreen, or collapse to one primary action. Choose one transition at a time. The controls below demonstrate the expanded and collapsed slot sets.".into());
                        gap(Dp(24.0));
                        collapsing_example();
                    }) },
                ],
            },
            ArticlePage {
                title: "Accessibility",
                intro: None,
                sections: vec![
                    SpecSection { title: "Use cases", content: RenderSlot::new(|| {
                        paragraph("People must be able to reach and activate all page actions, choose menu destinations, return using Back and retain access to actions when content scrolls or collapses.".into());
                    }) },
                    SpecSection { title: "Interaction & style", content: RenderSlot::new(|| {
                        paragraph("The toolbar container has no activation action. Its placed controls supply their own hover, focus and ripple feedback.".into());
                        gap(Dp(24.0));
                        subheading("Touch".into());
                        paragraph("Activate any icon button below to inspect its ripple and action result.".into());
                        gap(Dp(24.0));
                        subheading("Cursor".into());
                        paragraph("Hover and focused states identify individual interactive controls rather than the surrounding container.".into());
                        gap(Dp(24.0));
                        subheading("Initial focus".into());
                        paragraph("Focus starts on the first interactive element. Tab continues through the actions in their visible order.".into());
                        gap(Dp(24.0));
                        figure(Variant::Floating, ToolbarColorStyle::Standard, false);
                    }) },
                    SpecSection { title: "Keyboard navigation", content: RenderSlot::new(|| {
                        spec_table(vec!["Keys".into(), "Actions".into()], vec![
                            vec!["Tab or arrows".into(), "Navigate between interactive elements".into()],
                            vec!["Space or Enter".into(), "Activate the focused element".into()],
                        ]);
                        gap(Dp(32.0));
                        subheading("Labeling elements".into());
                        paragraph("The container exposes the toolbar role on web. Each action has its own descriptive label and selected state where appropriate.".into());
                        gap(Dp(24.0));
                        collapsing_example();
                    }) },
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
    let mut action = icon_button()
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
            if index == 0 {
                selected.set(!selected.get());
            }
            notice.set(format!("{label} activated"));
        });
    if index == 0 {
        action = action
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
    } else {
        action = action.standard().content_color(if vibrant {
            scheme.on_primary_container
        } else {
            scheme.primary
        });
    }
    drop(action);
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
            .elevation(Dp(if variant == Variant::WithFab && expanded {
                3.0
            } else {
                0.0
            }))
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
fn figure(variant: Variant, color: ToolbarColorStyle, measurements: bool) {
    let geometry = remember(Geometry::new);
    let notice = remember(String::new);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(
                    watch(geometry, "figure")
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
                                .padding_all(Dp(48.0)),
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
                                                        .modifier(watch(geometry, "fab"))
                                                        .accessibility_label("Create item")
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
            gap(Dp(16.0));
            paragraph(if measurements {
                "Toolbar size, padding and spacing measurements".into()
            } else {
                "1  Container\n2  Placed components".into()
            });
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
fn variants() {
    subheading("Docked toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Docked, ToolbarColorStyle::Standard, false);
    gap(Dp(32.0));
    subheading("Floating toolbar".into());
    gap(Dp(24.0));
    figure(Variant::Floating, ToolbarColorStyle::Standard, false);
}

#[tessera]
fn configurations() {
    for (variant, color, label) in [
        (Variant::Floating, ToolbarColorStyle::Standard, "Standard"),
        (Variant::Floating, ToolbarColorStyle::Vibrant, "Vibrant"),
        (
            Variant::Vertical,
            ToolbarColorStyle::Standard,
            "Vertical floating toolbar",
        ),
        (
            Variant::WithFab,
            ToolbarColorStyle::Vibrant,
            "Floating toolbar with FAB",
        ),
    ] {
        subheading(label.into());
        gap(Dp(24.0));
        figure(variant, color, false);
        gap(Dp(32.0));
    }
}

#[tessera]
fn measurements() {
    paragraph("All toolbars have a default height of 64dp. Floating containers use 8dp inner padding; horizontal outside margins are at least 16dp, and vertical outside margins are at least 24dp. Docked actions retain 16dp leading and trailing padding.".into());
    gap(Dp(32.0));
    for variant in [
        Variant::Docked,
        Variant::Floating,
        Variant::WithFab,
        Variant::Vertical,
    ] {
        figure(variant, ToolbarColorStyle::Standard, true);
        gap(Dp(32.0));
    }
    spec_table(
        vec!["Attribute".into(), "Value".into()],
        vec![
            vec!["Container height / vertical width".into(), "64dp".into()],
            vec!["Floating internal padding".into(), "8dp".into()],
            vec![
                "Docked leading/trailing padding".into(),
                "At least 16dp".into(),
            ],
            vec!["Horizontal outside margins".into(), "At least 16dp".into()],
            vec!["Vertical outside margins".into(), "At least 24dp".into()],
            vec!["FAB spacing".into(), "8dp".into()],
            vec!["Interactive target".into(), "At least 48×48dp".into()],
        ],
    );
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Token".into(), "Value".into()],
        vec![
            vec!["Standard container".into(), "Surface container".into()],
            vec!["Vibrant container".into(), "Primary container".into()],
            vec!["Floating shape".into(), "Fully rounded".into()],
            vec!["Docked shape".into(), "Rectangle".into()],
            vec!["Container height".into(), "64dp".into()],
        ],
    );
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
fn collapsing_example() {
    let expanded = remember(|| true);
    let notice = remember(String::new);
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            button()
                .text()
                .accessibility_label("Toggle toolbar expansion")
                .on_click(move || expanded.set(!expanded.get()))
                .child(move || {
                    text().content(if expanded.get() {
                        "Collapse actions"
                    } else {
                        "Expand actions"
                    });
                });
            gap(Dp(24.0));
            panel(RenderSlot::new(move || {
                toolbar_content(
                    Variant::Floating,
                    ToolbarColorStyle::Vibrant,
                    expanded.get(),
                    notice,
                );
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
    use tessera_ui::testing::layout_test;

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

    #[test]
    fn live_toolbar_figures_fit_real_interactive_controls_inside_64dp_cross_axis() {
        for (variant, expanded, expected_width, expected_height, count) in [
            (Variant::Floating, true, 208, 64, 4),
            (Variant::Floating, false, 64, 64, 1),
            (Variant::Vertical, true, 64, 208, 4),
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
            }
        }
    }
}
