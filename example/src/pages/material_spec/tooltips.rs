//! Plain and rich tooltip specification examples.
//!
//! ## Usage
//!
//! Explain actions with contextual help and optional follow-up actions.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    column::column,
    flow_row::flow_row,
    icon_button::icon_button,
    material_icons::filled,
    modifier::{MinimumInteractiveComponentEnforcement, ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    spacer::spacer,
    surface::surface,
    text::text,
    theme::{MaterialTheme, material_theme},
    tooltip::{
        TooltipController, TooltipPlacement, TooltipVariant, tooltip_anchor, tooltip_provider,
    },
};
use tessera_ui::{
    Dp, Modifier, RenderSlot, State, accesskit::Role, provide_context, remember, tessera,
    use_context,
};

use super::{
    ACCENT,
    annotations::{self, Geometry, Rect, dotted, relative, watch},
    article::{ArticlePage, SpecSection, article},
    gap, panel, spec_table, subheading, type_style,
};

#[tessera]
pub(super) fn tooltips_spec() {
    article(
        "Tooltips",
        "Explain unfamiliar controls with brief contextual messages.",
        vec![
            SpecSection {
                title: "Tokens & specs",
                content: RenderSlot::new(tokens),
            },
            SpecSection {
                title: "Plain tooltip",
                content: RenderSlot::new(|| {
                    detail(false);
                }),
            },
            SpecSection {
                title: "Rich tooltip",
                content: RenderSlot::new(|| {
                    detail(true);
                }),
            },
        ],
    ).pages(vec![
        ArticlePage { title: "Overview", intro: Some(RenderSlot::new(|| {
            prose("Plain tooltips explain icon-only actions. Rich tooltips add context, with an optional subhead and up to two relevant actions. Keep essential information visible outside tooltips.".into());
            gap(Dp(32.0)); live_tooltips();
        })), sections: vec![
            section("Differences from M2", RenderSlot::new(|| {
                prose("Material 3 uses theme color roles and more rounded rich tooltip containers.".into());
                gap(Dp(24.0)); tooltip_figure(true, false);
            })),
        ] },
        ArticlePage { title: "Guidelines", intro: None, sections: vec![
            section("Usage", RenderSlot::new(|| {
                prose("Explain unfamiliar icon-only controls with a plain tooltip. Use rich tooltips for definitions or new-feature explanations. Do not repeat visible labels or hide critical information.".into());
                gap(Dp(24.0)); live_tooltips();
            })),
            section("Anatomy", RenderSlot::new(|| {
                subheading("Plain tooltip".into()); gap(Dp(24.0)); tooltip_figure(false, false);
                gap(Dp(32.0)); subheading("Supporting text".into()); prose("Describe the action briefly, preferably on one line.".into());
                gap(Dp(32.0)); subheading("Rich tooltip".into()); gap(Dp(24.0)); tooltip_figure(true, false);
                gap(Dp(32.0)); subheading("Subhead (optional)".into()); prose("Summarize the message in a short subhead, especially for an automatically presented explanation.".into());
                gap(Dp(24.0)); subheading("Text buttons (optional)".into()); prose("Keep one or two actions brief enough to fit side by side.".into());
            })),
            section("Placement", RenderSlot::new(|| {
                subheading("Plain tooltips".into()); prose("Prefer above the target, or below an app-bar action. Leave 4dp from a visible control boundary and avoid covering its target.".into());
                gap(Dp(24.0)); tooltip_figure(false, true);
                gap(Dp(32.0)); subheading("Rich tooltips".into()); prose("Rich tooltips adjust near window edges. On desktop they can be centered below the target.".into());
                gap(Dp(24.0)); placement_examples();
            })),
            section("Behavior", RenderSlot::new(|| {
                subheading("Transient by default".into()); prose("Hover or keyboard focus explains a control without replacing its action. Opening another tooltip dismisses the previous explanation.".into());
                gap(Dp(24.0)); live_tooltips();
                gap(Dp(32.0)); subheading("Persistent rich tooltips".into()); prose("Use a deliberate click for a persistent explanation with actions. It remains available until dismissed; use labeled triggers rather than icon-only buttons.".into());
            })),
        ] },
        ArticlePage { title: "Accessibility", intro: None, sections: vec![
            section("Use cases", RenderSlot::new(|| { prose("People must receive the explanation and activate its trigger and actions using a keyboard or switch.".into()); })),
            section("Interaction & style", RenderSlot::new(|| {
                prose("Allow enough time to read the message. Hover and focus can reveal a transient explanation; selecting a labeled trigger opens a persistent rich explanation.".into());
                gap(Dp(24.0)); live_tooltips();
            })),
            section("Focus order", RenderSlot::new(|| {
                prose("Keep the trigger and tooltip actions in a linear focus sequence. Do not trap focus or obscure important page content. Escape dismisses the explanation and restores the trigger.".into());
                gap(Dp(24.0)); live_tooltips();
            })),
            section("Keyboard navigation", RenderSlot::new(|| {
                spec_table(vec!["Keys".into(), "Actions".into()], vec![vec!["Tab".into(), "Move between interactive elements".into()], vec!["Space or Enter".into(), "Activate the focused element".into()]]);
                gap(Dp(24.0)); live_tooltips();
            })),
            section("Labeling elements", RenderSlot::new(|| {
                prose("Expose the tooltip message and describe its trigger by action. Rich explanations with interactive actions also provide an accessible popup group.".into());
                gap(Dp(24.0)); tooltip_figure(false, false); gap(Dp(32.0)); tooltip_figure(true, false);
            })),
        ] },
    ]);
}

fn section(title: &'static str, content: RenderSlot) -> SpecSection {
    SpecSection { title, content }
}

#[tessera]
fn prose(content: String) {
    text().content(content).style(type_style(16.0, 24.0));
}

#[tessera]
fn sample(
    #[prop(into)] name: String,
    rich: bool,
    headline: Option<bool>,
    actions: Option<usize>,
    controls: Option<bool>,
) {
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let calls = remember(|| 0_usize);
    let long = remember(|| false);
    let label = remember(|| name.clone());
    let actions = actions.unwrap_or(if rich { 1 } else { 0 });
    let headline = headline.unwrap_or(rich);
    column().modifier(Modifier::new().fill_max_width().semantics(SemanticsArgs {role:Some(Role::Group),label:Some(format!("{name} tooltip preview")),merge_descendants:true,..Default::default()})).children(move || {
        if controls.unwrap_or(false) {
            flow_row().item_spacing(Dp(8.0)).line_spacing(Dp(8.0)).children(move || {
                button().tonal().accessibility_label(format!("Show {}",label.get())).on_click(move ||controller.with_mut(TooltipController::show)).child(|| {text().content("Show");});
                button().text().accessibility_label(format!("Dismiss {}",label.get())).on_click(move ||controller.with_mut(TooltipController::dismiss)).child(|| {text().content("Dismiss");});
                button().tonal().selected(long.get()).accessibility_label(format!("{} long message",label.get())).on_click(move ||long.set(!long.get())).child(|| {text().content("Long message");});
            });gap(Dp(16.0));
        }
        let mut provider = tooltip_provider(if long.get() {"Add this item to your collection so you can find it again later. You can change the collection at any time."} else if rich {"Add this item to your collection so you can find it again later."} else {"Add to favorites"})
            .controller(controller).variant(if rich {TooltipVariant::Rich} else {TooltipVariant::Plain})
            .placement(TooltipPlacement::Above).persistent(rich && actions>0);
        if headline {provider=provider.headline("Rich tooltip");}
        if rich && actions==1 {
            provider=provider.action_label("Learn more").on_action(move ||calls.set(calls.get()+1));
        } else if rich && actions>1 {
            provider=provider.actions(move || {
                flow_row().item_spacing(Dp(8.0)).children(move || {
                    for action in ["Learn more","Dismiss"] {
                        button().text().accessibility_label(format!("{} {action}",label.get())).on_click(move || {calls.set(calls.get()+1);controller.with_mut(TooltipController::dismiss);}).child(move || {text().content(action);});
                    }
                });
            });
        }
        provider.child(move || {
            boxed().modifier(Modifier::new().fill_max_width().height(Dp(240.0))).children(move || {
                tooltip_anchor().controller(controller).modifier(Modifier::new().align(Alignment::BottomCenter)).child(move || {
                    if rich {
                        button().text().accessibility_label(format!("{} anchor",label.get())).on_click(move ||controller.with_mut(TooltipController::show)).child(|| {text().content("More information");});
                    } else {
                        icon_button().standard().icon(filled::INFO_SVG).accessibility_label(format!("{} anchor",label.get())).on_click(move ||controller.with_mut(TooltipController::show));
                    }
                });
            });
        });
        if controls.unwrap_or(false) {
            gap(Dp(16.0));
            text().content(format!("{} actions: {}",label.get(),calls.get())).style(type_style(14.0,20.0));
            text().content(format!("{} visibility: {}",label.get(),if controller.with(TooltipController::is_open) {"Shown"} else {"Hidden"})).style(type_style(14.0,20.0));
        }
    });
}

#[tessera]
fn tokens() {
    spec_table(
        vec!["Element".into(), "Token value".into()],
        vec![
            vec![
                "Plain container / text".into(),
                "Inverse surface / inverse on surface".into(),
            ],
            vec!["Plain supporting text".into(), "Body small".into()],
            vec!["Rich container".into(), "Surface container".into()],
            vec![
                "Rich subhead and supporting text".into(),
                "On surface variant".into(),
            ],
            vec!["Rich action".into(), "Primary".into()],
        ],
    );
}

#[tessera]
fn detail(rich: bool) {
    column()
        .modifier(Modifier::new().fill_max_width())
        .children(move || {
            tooltip_figure(rich, false);
            gap(Dp(32.0));
            let title = if rich {
                "Rich tooltip"
            } else {
                "Plain tooltip"
            };
            panel(RenderSlot::new(move || {
                sample(
                    format!("Configured {}", if rich { "Rich" } else { "Plain" }),
                    rich,
                )
                .controls(true);
            }));
            gap(Dp(24.0));
            text()
        .content(if rich {
            "1  Optional subhead    2  Container    3  Supporting text    4  Optional text actions"
        } else {
            "1  Supporting text    2  Container"
        })
        .style(type_style(16.0, 24.0));
            gap(Dp(112.0));
            subheading(format!("{title} colors"));
            gap(Dp(24.0));
            for dark in [false, true] {
                material_theme()
                    .theme(move || MaterialTheme::from_seed(ACCENT, dark))
                    .child(move || {
                        surface()
                            .modifier(Modifier::new().fill_max_width())
                            .child(move || {
                                column()
                                    .modifier(
                                        Modifier::new().fill_max_width().padding_all(Dp(24.0)),
                                    )
                                    .children(move || {
                                        sample(
                                            format!(
                                                "{} {}",
                                                if dark { "Dark" } else { "Light" },
                                                title
                                            ),
                                            rich,
                                        );
                                    });
                            });
                    });
                gap(Dp(24.0));
            }
            gap(Dp(88.0));
            subheading(format!("{title} measurements"));
            gap(Dp(24.0));
            tooltip_figure(rich, true);
            gap(Dp(24.0));
            spec_table(
                vec!["Attribute".into(), "Value".into()],
                if rich {
                    vec![
                        vec!["Top padding".into(), "12dp".into()],
                        vec!["Bottom padding with actions".into(), "8dp".into()],
                        vec!["Horizontal padding".into(), "16dp".into()],
                        vec!["Maximum width".into(), "320dp".into()],
                    ]
                } else {
                    vec![
                        vec!["Single-line container height".into(), "24dp".into()],
                        vec!["Horizontal padding".into(), "8dp".into()],
                        vec!["Maximum width".into(), "200dp".into()],
                    ]
                },
            );
            if rich {
                gap(Dp(112.0));
                subheading("Rich tooltip configurations".to_owned());
                gap(Dp(24.0));
                for (index, headline, actions) in [
                    (1, true, 2),
                    (2, true, 1),
                    (3, true, 0),
                    (4, false, 1),
                    (5, false, 2),
                ] {
                    panel(RenderSlot::new(move || {
                        sample(format!("Rich configuration {index}"), true)
                            .headline(headline)
                            .actions(actions);
                    }));
                    gap(Dp(24.0));
                }
            }
        });
}

#[tessera]
fn tooltip_figure(rich: bool, measurements: bool) {
    let geometry = remember(Geometry::new);
    let controller = remember(|| {
        let mut controller = TooltipController::new();
        controller.show();
        controller
    });
    let actions = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let figure_width = geometry
        .with(|values| {
            values
                .get("figure")
                .map(|rect| Dp::from_pixels_f32(rect.width.to_f32()).0)
        })
        .unwrap_or(1024.0);
    let figure_height = figure_width / (2048.0 / 600.0);
    let popup_height = geometry
        .with(|values| {
            values
                .get("popup")
                .map(|rect| Dp::from_pixels_f32(rect.height.to_f32()).0)
        })
        .unwrap_or(if rich { 156.0 } else { 24.0 });
    boxed()
        .alignment(Alignment::TopStart)
        .modifier(
            watch(geometry, "figure")
                .fill_max_width()
                .height(Dp(figure_height))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(format!(
                        "{} tooltip {} figure",
                        if rich { "Rich" } else { "Plain" },
                        if measurements {
                            "measurement"
                        } else {
                            "anatomy"
                        }
                    )),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            tooltip_grid(Dp(figure_width),Dp(figure_height));
            provide_context(|| MinimumInteractiveComponentEnforcement { enabled: false }, move || {
            let mut tooltip = tooltip_provider(if rich {
                "Rich tooltips bring attention to a particular element or feature that warrants a person's focus."
            } else {
                "Plain tooltip"
            })
            .controller(controller)
            .variant(if rich {
                TooltipVariant::Rich
            } else {
                TooltipVariant::Plain
            })
            .popup_modifier(if rich { watch(geometry, "popup").width(Dp(312.0)) } else { watch(geometry, "popup") })
            .placement(TooltipPlacement::Above);
            if rich {
                tooltip = tooltip
                    .headline("Rich tooltip")
                    .action_label("Action")
                    .on_action(move || actions.set(actions.get() + 1));
            }
            tooltip.child(move || {
                boxed()
                    .modifier(Modifier::new().fill_max_size())
                    .children(move || {
                        tooltip_anchor()
                            .controller(controller)
                            .modifier(
                                watch(geometry, "anchor")
                                    .align(Alignment::TopCenter)
                                    .offset(Dp(0.0), Dp((figure_height + popup_height) * 0.5 + 4.0)),
                            )
                            .child(|| { spacer().modifier(Modifier::new().size(Dp(1.0), Dp(1.0))); });
                    });
            });
            });
            tooltip_annotations(geometry, rich, measurements);
        });
    gap(Dp(16.0));
    prose(if measurements {
        format!(
            "{} tooltip padding and size measurements",
            if rich { "Rich" } else { "Plain" }
        )
    } else if rich {
        "1  Subhead\n2  Container\n3  Supporting text\n4  Text button".into()
    } else {
        "1  Supporting text\n2  Container".into()
    });
    if actions.get() > 0 {
        text()
            .content(format!("Tooltip figure actions: {}", actions.get()))
            .style(type_style(14.0, 20.0));
    }
}

#[tessera]
fn tooltip_grid(width: Dp, height: Dp) {
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
fn tooltip_annotations(geometry: State<Geometry>, rich: bool, measurements: bool) {
    let values = geometry.get();
    let (Some(root), Some(popup), Some(anchor)) = (
        values.get("figure"),
        relative(&values, "popup"),
        relative(&values, "anchor"),
    ) else {
        return;
    };
    let inset = if rich { 16.0 } else { 8.0 };
    let top = if rich { 12.0 } else { 4.0 };
    let bottom = if rich { 8.0 } else { 4.0 };
    let mut svg = String::new();
    if measurements {
        for rect in [
            Rect {
                x: popup.x,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
            Rect {
                x: popup.x + popup.w - inset,
                y: popup.y,
                w: inset,
                h: popup.h,
            },
            Rect {
                x: popup.x,
                y: popup.y,
                w: popup.w,
                h: top,
            },
            Rect {
                x: popup.x,
                y: popup.y + popup.h - bottom,
                w: popup.w,
                h: bottom,
            },
        ] {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#e883aa' fill-opacity='.32'/>",
                rect.x, rect.y, rect.w, rect.h
            ));
        }
        for (label, x, y) in [
            (format!("{top:.0}"), popup.x - 32.0, popup.y),
            (
                format!("{bottom:.0}"),
                popup.x + popup.w + 16.0,
                popup.y + popup.h - bottom,
            ),
            (format!("{inset:.0}"), popup.x, popup.y + popup.h + 16.0),
            (
                format!("{inset:.0}"),
                popup.x + popup.w - inset,
                popup.y + popup.h + 16.0,
            ),
        ] {
            annotations::dimension(label, Dp(x), Dp(y));
        }
        dotted(
            &mut svg,
            popup.x + popup.w + 8.0,
            popup.y,
            popup.h,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{:.0}", popup.h),
            Dp(popup.x + popup.w + 16.0),
            Dp(popup.y + popup.h * 0.5 - 10.0),
        );
        let gap = (anchor.y - popup.y - popup.h).max(0.0);
        dotted(
            &mut svg,
            anchor.x - 16.0,
            popup.y + popup.h,
            gap,
            true,
            "#b14e70",
        );
        annotations::dimension(
            format!("{gap:.0}"),
            Dp(anchor.x - 36.0),
            Dp(popup.y + popup.h),
        );
    } else if rich {
        let headline = Rect {
            x: popup.x + 16.0,
            y: popup.y + 12.0,
            w: popup.w - 32.0,
            h: 20.0,
        };
        let actions = Rect {
            x: popup.x + 16.0,
            y: popup.y + popup.h - 48.0,
            w: popup.w - 32.0,
            h: 40.0,
        };
        let body = Rect {
            x: headline.x,
            y: headline.y + headline.h + 4.0,
            w: headline.w,
            h: (actions.y - 12.0 - headline.y - headline.h - 4.0).max(0.0),
        };
        for (number, rect, x, above) in [
            (1, headline, headline.x + 8.0, true),
            (2, popup, popup.x + popup.w - 12.0, true),
            (3, body, body.x + body.w - 16.0, false),
            (4, actions, actions.x + 20.0, false),
        ] {
            let end = if above { rect.y } else { rect.y + rect.h };
            let start = if above { popup.y - 40.0 } else { end };
            let length = if above {
                end - start
            } else {
                popup.y + popup.h + 40.0 - end
            };
            dotted(&mut svg, x, start, length, true, "#49454f");
            annotations::marker(
                number,
                Dp(x - 12.0),
                Dp(if above { start - 32.0 } else { start + length }),
            );
            annotations::outline(&mut svg, rect);
        }
    } else {
        let text = Rect {
            x: popup.x + 8.0,
            y: popup.y + 4.0,
            w: popup.w - 16.0,
            h: popup.h - 8.0,
        };
        let text_x = text.x + 8.0;
        dotted(&mut svg, text_x, text.y + text.h, 36.0, true, "#49454f");
        annotations::marker(1, Dp(text_x - 12.0), Dp(text.y + text.h + 36.0));
        let container_x = popup.x + popup.w - 8.0;
        dotted(&mut svg, container_x, popup.y - 36.0, 36.0, true, "#49454f");
        annotations::marker(2, Dp(container_x - 12.0), Dp(popup.y - 68.0));
        annotations::outline(&mut svg, text);
    }
    annotations::lines(
        Dp::from_pixels_f32(root.width.to_f32()),
        Dp::from_pixels_f32(root.height.to_f32()),
        svg,
    );
}

#[tessera]
fn live_tooltips() {
    let plain = remember(TooltipController::new);
    let rich = remember(TooltipController::new);
    let favorite = remember(|| false);
    let actions = remember(|| 0_usize);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    tooltip_provider(if favorite.get() {
        "Remove from favorites"
    } else {
        "Add to favorites"
    })
    .controller(plain)
    .persistent(false)
    .child(move || {
        tooltip_provider("Keep related items together so you can find them again.")
            .controller(rich)
            .variant(TooltipVariant::Rich)
            .headline("Collections")
            .placement(TooltipPlacement::Below)
            .persistent(true)
            .action_label("Got it")
            .on_action(move || actions.set(actions.get() + 1))
            .child(move || {
                boxed()
                    .alignment(Alignment::Center)
                    .modifier(
                        Modifier::new()
                            .fill_max_width()
                            .height(Dp(360.0))
                            .background_with_shape(
                                scheme.surface_container_low,
                                Shape::rounded_rectangle(Dp(24.0)),
                            ),
                    )
                    .children(move || {
                        flow_row()
                            .item_spacing(Dp(24.0))
                            .line_spacing(Dp(16.0))
                            .children(move || {
                                tooltip_anchor().controller(plain).child(move || {
                                    icon_button()
                                        .standard()
                                        .selected(favorite.get())
                                        .icon(if favorite.get() {
                                            filled::FAVORITE_SVG
                                        } else {
                                            filled::FAVORITE_BORDER_SVG
                                        })
                                        .accessibility_label("Favorite tooltip action")
                                        .on_click(move || favorite.set(!favorite.get()));
                                });
                                tooltip_anchor().controller(rich).child(move || {
                                    button()
                                        .text()
                                        .accessibility_label("About collections")
                                        .on_click(move || rich.with_mut(TooltipController::show))
                                        .child(|| {
                                            text().content("About collections");
                                        });
                                });
                            });
                    });
            });
    });
    gap(Dp(16.0));
    text()
        .content(format!(
            "Favorite: {}; tooltip actions: {}",
            if favorite.get() { "on" } else { "off" },
            actions.get()
        ))
        .style(type_style(14.0, 20.0));
}

#[tessera]
fn placement_examples() {
    flow_row()
        .item_spacing(Dp(16.0))
        .line_spacing(Dp(16.0))
        .children(|| {
            for (alignment, placement) in [
                (Alignment::TopStart, TooltipPlacement::Above),
                (Alignment::TopEnd, TooltipPlacement::Above),
                (Alignment::BottomStart, TooltipPlacement::Below),
                (Alignment::BottomEnd, TooltipPlacement::Below),
            ] {
                let controller = remember(|| {
                    let mut controller = TooltipController::new();
                    controller.show();
                    controller
                });
                tooltip_provider("Tooltip stays within the available space.")
                    .controller(controller)
                    .variant(TooltipVariant::Rich)
                    .headline("Dynamic placement")
                    .placement(placement)
                    .persistent(false)
                    .child(move || {
                        boxed()
                            .modifier(Modifier::new().size(Dp(264.0), Dp(220.0)))
                            .children(move || {
                                tooltip_anchor()
                                    .controller(controller)
                                    .modifier(Modifier::new().align(alignment))
                                    .child(move || {
                                        button()
                                            .text()
                                            .accessibility_label(format!(
                                                "Corner tooltip {alignment:?}"
                                            ))
                                            .on_click(move || {
                                                controller.with_mut(TooltipController::show)
                                            })
                                            .child(|| {
                                                text().content("Details");
                                            });
                                    });
                            });
                    });
            }
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use tessera_ui::{
        HeadlessConfig, HeadlessRenderer, MOUSE_POINTER_ID, PressKeyEventType, PxPosition,
        accesskit::Rect,
    };

    use super::*;

    struct ComponentPipelines;
    impl tessera_ui::RenderModule for ComponentPipelines {
        fn register_pipelines(&self, context: &mut tessera_ui::PipelineContext<'_>) {
            tessera_components::pipelines::register_pipelines(context);
        }
    }

    #[tessera]
    fn fixture() {
        material_theme().child(|| {
            boxed()
                .alignment(Alignment::TopStart)
                .modifier(Modifier::new().fill_max_size())
                .children(|| {
                    column()
                        .modifier(Modifier::new().width(Dp(600.0)))
                        .children(|| {
                            live_tooltips();
                        });
                });
        });
    }

    fn bounds(renderer: &HeadlessRenderer, label: &str) -> Rect {
        renderer
            .accessibility_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
            .unwrap_or_else(|| panic!("missing live tooltip control {label}"))
            .1
            .bounds()
            .unwrap()
    }

    fn click(renderer: &mut HeadlessRenderer, bounds: Rect) {
        let position = PxPosition::from_f32_xy(
            ((bounds.x0 + bounds.x1) * 0.5) as f32,
            ((bounds.y0 + bounds.y1) * 0.5) as f32,
        );
        renderer.pointer_move(MOUSE_POINTER_ID, position);
        renderer.pointer_press(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.pointer_release(MOUSE_POINTER_ID, position, PressKeyEventType::Left);
        renderer.render_frames(4);
    }

    #[test]
    #[ignore = "Requires a GPU adapter for the real component pipelines"]
    fn live_tooltips_keep_trigger_actions_and_replace_the_active_explanation() {
        let mut renderer = HeadlessRenderer::new(
            Box::new(fixture),
            vec![Box::new(ComponentPipelines)],
            HeadlessConfig {
                width: 800,
                height: 600,
                sample_count: 1,
                ..Default::default()
            },
        )
        .expect("native tooltip examples");
        renderer.render_frames(4);
        let favorite = bounds(&renderer, "Favorite tooltip action");
        renderer.pointer_move(
            MOUSE_POINTER_ID,
            PxPosition::from_f32_xy(
                ((favorite.x0 + favorite.x1) * 0.5) as f32,
                ((favorite.y0 + favorite.y1) * 0.5) as f32,
            ),
        );
        renderer.render_frames(35);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Tooltip
                    && node.label() == Some("Add to favorites"))
        );
        click(&mut renderer, favorite);
        assert!(
            renderer
                .accessibility_snapshot()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Favorite: on; tooltip actions: 0")),
            "tooltip hover must preserve the trigger's actual toggle action"
        );
        let about = bounds(&renderer, "About collections");
        click(&mut renderer, about);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.role() == Role::Tooltip
                    && matches!(
                        node.label(),
                        Some("Add to favorites" | "Remove from favorites")
                    )),
            "the shared live host dismisses the preceding plain tooltip"
        );
        assert!(snapshot.nodes.iter().any(|(_, node)| node.role() == Role::Dialog && node.label() == Some("Collections")));
        let action = bounds(&renderer, "Got it");
        click(&mut renderer, action);
        let snapshot = renderer.accessibility_snapshot().unwrap();
        assert!(!snapshot.nodes.iter().any(|(_, node)| node.role() == Role::Dialog && node.label() == Some("Collections")));
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Favorite: on; tooltip actions: 1")),
            "the real rich tooltip action updates its owner"
        );
    }
}
