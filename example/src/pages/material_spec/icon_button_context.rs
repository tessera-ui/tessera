//! Contextual icon button placement and accessibility diagrams.
//!
//! ## Usage
//!
//! Explore actions in app bars, restaurant cards and floating toolbars.

use std::sync::OnceLock;

use tessera_components::{
    alignment::{Alignment, CrossAxisAlignment},
    boxed::boxed,
    chip::chip,
    column::column,
    floating_action_button::floating_action_button,
    icon::icon,
    icon_button::{IconButtonWidth, icon_button},
    image::{ImageData, TryIntoImageData as _, image},
    material_icons::{Asset, filled, outlined},
    modifier::{ModifierExt as _, SemanticsArgs},
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
    toolbar::floating_toolbar,
};
use tessera_ui::{
    Color, Dp, LayoutCoordinates, Modifier, RenderSlot, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use super::super::{
    annotations::{self, ObservedGeometry, observed_relative_geometry, watch_observed},
    gap, media, srgb, subheading, type_style,
};

#[tessera]
pub(in crate::pages::material_spec) fn placement() {
    context_row(
        RenderSlot::new(|| {
            paragraph(
                "Icon buttons are commonly used in other components, such as app bars and cards.",
            );
            gap(Dp(16.0));
            paragraph("These buttons should be used for common, easily understandable actions.");
            gap(Dp(16.0));
            paragraph("Only use a few icon buttons at once.");
        }),
        RenderSlot::new(|| {
            app_bar_figure();
            caption("App bars often contain icon buttons");
        }),
    );
    gap(Dp(72.0));
    context_row(
        RenderSlot::new(|| {
            paragraph(
                "In dense layouts, group popular actions by placing many icon buttons next to each other in components like a toolbar or button group.",
            );
            gap(Dp(16.0));
            paragraph("These components draw attention or add interaction between buttons.");
        }),
        RenderSlot::new(|| {
            toolbar_figure();
            caption("A toolbar is a collection of icon buttons and other components");
        }),
    );
}

#[tessera]
pub(in crate::pages::material_spec) fn labeling() {
    context_row(
        RenderSlot::new(|| {
            paragraph(
                "The accessibility label for icon buttons describes the action the button is executing, such as Add to favorites, Bookmark, or Send message.",
            );
        }),
        RenderSlot::new(|| {
            labeling_figure();
            caption(
                "The icon button label describes the action, such as Add to favorites for the heart icon",
            );
        }),
    );
}

#[tessera]
pub(in crate::pages::material_spec) fn density() {
    context_row(
        RenderSlot::new(|| {
            paragraph(
                "Groups of similar components can be nested together inside a component, or they can stand alone.",
            );
            gap(Dp(24.0));
            paragraph(
                "The target size of each icon button should be at least 48dp, even when nested.",
            );
        }),
        RenderSlot::new(|| {
            density_figure();
            caption("Icon buttons can be used within other components, such as an app bar");
        }),
    );
    gap(Dp(48.0));
    subheading("Avoid applying density by default".into());
    gap(Dp(16.0));
    paragraph(
        "Don't apply density to icon buttons by default. This lowers their targets below the required 48x48 CSS pixels minimum size.",
    );
    gap(Dp(16.0));
    paragraph(
        "Provide density options that allow people to choose a higher density, such as selecting a denser layout or changing the theme. Controls for adjusting density must maintain a target size of at least 48x48 CSS pixels.",
    );
}

#[tessera]
fn paragraph(#[prop(into)] value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn caption(#[prop(into)] value: String) {
    gap(Dp(24.0));
    text()
        .content(value)
        .style(type_style(16.0, 24.0))
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp::ZERO));
}

#[tessera]
fn context_row(prose: RenderSlot, figure: RenderSlot) {
    let width = remember(|| Dp(1024.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            if width.get().0 < 760.0 {
                column()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        prose.render();
                        gap(Dp(24.0));
                        figure.render();
                    });
            } else {
                let track = (width.get().0 - 40.0) / 3.0;
                row()
                    .modifier(Modifier::new().fill_max_width())
                    .children(move || {
                        column()
                            .modifier(Modifier::new().width(Dp(track)))
                            .children_shared(prose);
                        spacer().modifier(Modifier::new().width(Dp(20.0)));
                        column()
                            .modifier(Modifier::new().width(Dp(track * 2.0 + 20.0)))
                            .children_shared(figure);
                    });
            }
        });
}

#[tessera]
fn canvas(height: f64, geometry: State<ObservedGeometry>, child: RenderSlot) {
    media::load_fonts();
    annotations::publish_observed_geometry(geometry);
    let width = remember(|| Dp(676.0));
    let scale = (width.get().0 / 676.0) as f32;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if width.get() != next {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box(scale).child(move || {
                boxed()
                    .modifier(
                        watch_observed(geometry, "figure")
                            .required_size(Dp(676.0), Dp(height))
                            .background_with_shape(
                                scheme.secondary_container,
                                Shape::rounded_rectangle(Dp(16.0)),
                            )
                            .clip_shape(Shape::rounded_rectangle(Dp(16.0))),
                    )
                    .children_shared(child);
            });
        });
}

#[tessera]
fn placed_text(#[prop(into)] value: String, x: f64, y: f64, width: f64, size: f64, line: f64) {
    text()
        .content(value)
        .style(type_style(size, line))
        .modifier(Modifier::new().offset(Dp(x), Dp(y)).width(Dp(width)));
}

fn action_icon(index: usize) -> Asset {
    match index {
        0 => filled::ARROW_BACK_SVG,
        1 => filled::SHARE_SVG,
        2 => filled::MORE_VERT_SVG,
        3 => outlined::ARCHIVE_SVG,
        4 => outlined::DELETE_SVG,
        5 => outlined::MAIL_SVG,
        6 => outlined::SNOOZE_SVG,
        _ => filled::UNDO_SVG,
    }
}

#[tessera]
fn action(#[prop(into)] label: String, index: usize) {
    let activations = remember(|| 0_u32);
    icon_button()
        .standard()
        .icon(action_icon(index))
        .shape(Shape::CAPSULE)
        .accessibility_label(label)
        .accessibility_description(format!("Activations: {}", activations.get()))
        .on_click(move || activations.set(activations.get() + 1));
}

#[tessera]
fn placed_action(#[prop(into)] label: String, index: usize, x: f64, y: f64) {
    boxed()
        .modifier(Modifier::new().offset(Dp(x), Dp(y)))
        .children(move || {
            action(label.clone(), index);
        });
}

#[tessera]
fn favorite(selected: State<bool>, tonal: bool) {
    icon_button()
        .variant(if tonal {
            tessera_components::icon_button::IconButtonVariant::FilledTonal
        } else {
            tessera_components::icon_button::IconButtonVariant::Standard
        })
        .width(if tonal {
            IconButtonWidth::Uniform
        } else {
            IconButtonWidth::Wide
        })
        .shape(Shape::CAPSULE)
        .selected(selected.get())
        .icon(if selected.get() {
            filled::FAVORITE_SVG
        } else {
            filled::FAVORITE_BORDER_SVG
        })
        .accessibility_label(if selected.get() {
            "Remove from favorites"
        } else {
            "Add to favorites"
        })
        .on_click(move || selected.set(!selected.get()));
}

#[tessera]
fn device(x: f64, y: f64, height: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .required_size(Dp(428.0), Dp(height))
                .background_with_shape(scheme.outline_variant, Shape::rounded_rectangle(Dp(28.0))),
        )
        .children(move || {
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(8.0), Dp(8.0))
                    .required_size(Dp(412.0), Dp(height - 16.0))
                    .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(20.0))),
            );
        });
}

fn photo_data(source: usize) -> ImageData {
    static APPBAR: OnceLock<ImageData> = OnceLock::new();
    static LABEL: OnceLock<ImageData> = OnceLock::new();
    static DENSITY: OnceLock<ImageData> = OnceLock::new();
    let (slot, bytes): (&OnceLock<ImageData>, &[u8]) = match source {
        0 => (
            &APPBAR,
            include_bytes!("../../../assets/material_spec/icon-context-appbar.png"),
        ),
        1 => (
            &LABEL,
            include_bytes!("../../../assets/material_spec/icon-context-label.png"),
        ),
        _ => (
            &DENSITY,
            include_bytes!("../../../assets/material_spec/icon-context-density.png"),
        ),
    };
    slot.get_or_init(|| {
        bytes
            .try_into_image_data()
            .expect("icon button context photograph")
    })
    .clone()
}

#[tessera]
fn photo_region(source: usize, x: f64, y: f64, width: f64, height: f64) {
    let data = photo_data(source);
    let source_height = [400.0, 258.0, 300.0][source];
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x), Dp(y))
                .size(Dp(width), Dp(height))
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some(
                        if source == 2 && height < 45.0 && width < 45.0 {
                            "So Duri profile photograph"
                        } else {
                            "Restaurant photograph"
                        }
                        .into(),
                    ),
                    ..Default::default()
                }),
        )
        .children(move || {
            image().raster(data.clone()).modifier(
                Modifier::new()
                    .offset(Dp(-x), Dp(-y))
                    .required_size(Dp(676.0), Dp(source_height)),
            );
        });
}

#[tessera]
fn app_bar_figure() {
    let geometry = remember(ObservedGeometry::new);
    let selected = remember(|| false);
    canvas(
        400.0,
        geometry,
        RenderSlot::new(move || {
            device(124.0, 82.0, 440.0);
            placed_text("9:30", 156.0, 114.0, 60.0, 14.0, 20.0);
            for (index, vector) in [
                filled::WIFI_SVG,
                filled::SIGNAL_CELLULAR_4_BAR_SVG,
                filled::BATTERY_FULL_SVG,
            ]
            .into_iter()
            .enumerate()
            {
                boxed()
                    .modifier(Modifier::new().offset(Dp(474.0 + index as f64 * 16.0), Dp(116.0)))
                    .children(move || {
                        icon().vector(vector).size(Dp(16.0));
                    });
            }
            placed_action("Back to restaurants", 0, 136.0, 155.0);
            placed_text("Terracotta", 184.0, 164.0, 224.0, 22.0, 28.0);
            placed_action("Share restaurant", 1, 434.0, 155.0);
            placed_action("More restaurant options", 2, 482.0, 155.0);
            row()
                .modifier(Modifier::new().offset(Dp(148.0), Dp(210.0)))
                .children(|| {
                    for index in 0..5 {
                        icon()
                            .vector(if index < 4 {
                                filled::STAR_SVG
                            } else {
                                filled::STAR_BORDER_SVG
                            })
                            .size(Dp(20.0));
                    }
                });
            placed_text(
                "American · $$ · 2.3 miles away",
                148.0,
                236.0,
                380.0,
                14.0,
                20.0,
            );
            placed_text(
                "Farm-to-table fare with a Southern twist",
                148.0,
                256.0,
                380.0,
                14.0,
                20.0,
            );
            photo_region(0, 148.0, 291.0, 380.0, 109.0);
            boxed()
                .modifier(Modifier::new().offset(Dp(472.0), Dp(299.0)))
                .children(move || {
                    favorite(selected, true);
                });
        }),
    );
}

#[tessera]
fn toolbar_figure() {
    let geometry = remember(ObservedGeometry::new);
    let undo = remember(|| 0_u32);
    canvas(
        400.0,
        geometry,
        RenderSlot::new(move || {
            floating_toolbar()
                .modifier(Modifier::new().offset(Dp(166.0), Dp(168.0)))
                .elevation(Dp(1.0))
                .accessibility_label("Mail actions")
                .content(|| {
                    row()
                        .modifier(Modifier::new().padding_symmetric(Dp(4.0), Dp::ZERO))
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(|| {
                            for (position, (index, label)) in [
                                (3, "Archive message"),
                                (4, "Delete message"),
                                (5, "Mark unread"),
                                (6, "Snooze message"),
                                (2, "More message options"),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                if position > 0 {
                                    spacer().modifier(Modifier::new().width(Dp(4.0)));
                                }
                                action(label, index);
                            }
                        });
                });
            floating_action_button()
                .modifier(Modifier::new().offset(Dp(454.0), Dp(172.0)))
                .accessibility_label("Undo message action")
                .accessibility_description(format!("Activations: {}", undo.get()))
                .on_click(move || undo.set(undo.get() + 1))
                .content(|| {
                    icon().vector(filled::UNDO_SVG);
                });
        }),
    );
}

#[tessera]
fn labeling_figure() {
    let geometry = remember(ObservedGeometry::new);
    let selected = remember(|| false);
    canvas(
        258.0,
        geometry,
        RenderSlot::new(move || {
            device(22.0, -160.0, 520.0);
            photo_region(1, 54.0, 0.0, 364.0, 79.0);
            placed_text("New Tyrrhenia", 54.0, 96.0, 300.0, 24.0, 32.0);
            placed_text("Mediterranean", 54.0, 134.0, 300.0, 16.0, 24.0);
            boxed()
                .alignment(Alignment::Center)
                .modifier(
                    watch_observed(geometry, "favorite")
                        .offset(Dp(362.0), Dp(103.0))
                        .size(Dp(60.0), Dp(48.0)),
                )
                .children(move || {
                    favorite(selected, false);
                });
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(54.0), Dp(178.0))
                    .size(Dp(364.0), Dp(1.0))
                    .background(scheme.outline_variant),
            );
            placed_text("Dinner", 54.0, 204.0, 100.0, 14.0, 20.0);
            boxed()
                .modifier(
                    Modifier::new()
                        .required_size(Dp(442.0), Dp(258.0))
                        .clip_to_bounds(),
                )
                .children(|| {
                    row()
                        .modifier(Modifier::new().offset(Dp(54.0), Dp(239.0)))
                        .children(|| {
                            for label in ["5:30pm", "6:00pm", "6:30pm", "7:00pm", "7:30pm"] {
                                let selected = remember(|| false);
                                chip()
                                    .label(label)
                                    .selected(selected.get())
                                    .on_click(move || selected.set(!selected.get()));
                                spacer().modifier(Modifier::new().width(Dp(8.0)));
                            }
                        });
                });
            labeling_annotation(geometry, selected);
        }),
    );
}

#[tessera]
fn labeling_annotation(geometry: State<ObservedGeometry>, selected: State<bool>) {
    let rects = geometry.with(observed_relative_geometry);
    let Some(target) = rects.get("favorite").copied() else {
        return;
    };
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let tint = scheme.tertiary;
    spacer().modifier(
        Modifier::new()
            .offset(Dp(target.x), Dp(target.y))
            .size(Dp(target.w), Dp(target.h))
            .border_with_shape(Dp(4.0), tint, Shape::CAPSULE),
    );
    spacer().modifier(
        Modifier::new()
            .offset(Dp(target.x + target.w), Dp(target.y + target.h * 0.5 - 2.0))
            .size(Dp(474.0 - target.x - target.w), Dp(4.0))
            .background(tint),
    );
    boxed()
        .modifier(
            Modifier::new()
                .offset(
                    Dp(474.0),
                    Dp(target.y + target.h * 0.5 - if selected.get() { 70.0 } else { 60.0 }),
                )
                .size(
                    Dp(if selected.get() { 176.0 } else { 160.0 }),
                    Dp(if selected.get() { 140.0 } else { 120.0 }),
                )
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(12.0)))
                .border_with_shape(Dp(4.0), tint, Shape::rounded_rectangle(Dp(12.0)))
                .padding_all(Dp(16.0))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    label: Some("Icon button label and role".into()),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            column().children(move || {
                text().content("Label").style(type_style(14.0, 20.0));
                text()
                    .content(if selected.get() {
                        "Remove from favorites"
                    } else {
                        "Add to favorites"
                    })
                    .color(tint)
                    .style(tessera_components::theme::TextStyle {
                        font_weight: 700,
                        ..type_style(16.0, 20.0)
                    });
                gap(Dp(8.0));
                text().content("Role").style(type_style(14.0, 20.0));
                text()
                    .content("Button")
                    .color(tint)
                    .style(tessera_components::theme::TextStyle {
                        font_weight: 700,
                        ..type_style(16.0, 20.0)
                    });
            });
        });
}

#[tessera]
fn density_figure() {
    let geometry = remember(ObservedGeometry::new);
    let starred = remember(|| false);
    canvas(
        300.0,
        geometry,
        RenderSlot::new(move || {
            let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
            device(194.0, -240.0, 500.0);
            boxed().modifier(Modifier::new().required_size(Dp(676.0), Dp(252.0)).clip_to_bounds()).children(move || {
            spacer().modifier(
                Modifier::new()
                    .offset(Dp(218.0), Dp(34.0))
                    .size(Dp(380.0), Dp(270.0))
                    .background_with_shape(
                        scheme.surface_container,
                        Shape::rounded_rectangle(Dp(16.0)),
                    ),
            );
            photo_region(2, 238.0, 55.0, 40.0, 40.0);
            placed_text("So Duri", 290.0, 58.0, 180.0, 14.0, 20.0);
            placed_text("4 hours ago", 290.0, 78.0, 180.0, 12.0, 16.0);
            boxed()
                .modifier(Modifier::new().offset(Dp(534.0), Dp(49.0)))
                .children(move || {
                    icon_button()
                        .filled_tonal()
                        .selected(starred.get())
                        .icon(if starred.get() {
                            filled::STAR_SVG
                        } else {
                            filled::STAR_BORDER_SVG
                        })
                        .accessibility_label(if starred.get() {
                            "Remove message star"
                        } else {
                            "Star message"
                        })
                        .on_click(move || starred.set(!starred.get()));
                });
            placed_text("Dinner Club", 238.0, 105.0, 340.0, 24.0, 32.0);
            placed_text(
                "I think it’s time we finally get a reservation for that new spot downtown that does Northern Kyoto stuff.",
                238.0,
                143.0,
                340.0,
                14.0,
                20.0,
            );
            photo_region(2, 238.0, 214.0, 340.0, 38.0);
            });
            floating_toolbar()
                .modifier(Modifier::new().offset(Dp(280.0), Dp(159.0)))
                .accessibility_label("Nested message actions")
                .content(move || {
                    row()
                        .cross_axis_alignment(CrossAxisAlignment::Center)
                        .children(move || {
                            for (index, label, name) in [
                                (4, "Delete dinner message", "delete"),
                                (3, "Archive dinner message", "archive"),
                                (5, "Mark dinner message unread", "mail"),
                                (2, "More dinner message options", "more"),
                            ] {
                                boxed()
                                    .modifier(
                                        watch_observed(geometry, name)
                                            .background(Color::new(0.92, 0.40, 0.96, 0.4))
                                            .border(Dp(0.5), scheme.surface_container),
                                    )
                                    .children(move || {
                                        action(label, index);
                                    });
                            }
                            let undo = remember(|| 0_u32);
                            icon_button()
                                .filled()
                                .width(IconButtonWidth::Wide)
                                .icon(filled::UNDO_SVG)
                                .accessibility_label("Undo dinner message action")
                                .accessibility_description(format!("Activations: {}", undo.get()))
                                .on_click(move || undo.set(undo.get() + 1));
                        });
                });
            density_annotation(geometry);
        }),
    );
}

#[tessera]
fn density_annotation(geometry: State<ObservedGeometry>) {
    let rects = geometry.with(observed_relative_geometry);
    let Some(target) = rects.get("delete").copied() else {
        return;
    };
    let y = target.y + target.h * 0.5;
    let end = target.x + 4.0;
    let ink = srgb(48, 45, 53);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(20.0), Dp(y - 16.0))
                .size(Dp(156.0), Dp(32.0))
                .background_with_shape(ink, Shape::CAPSULE)
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(|| {
            text()
                .content("Target size 48")
                .style(type_style(16.0, 24.0))
                .color(Color::WHITE);
        });
    spacer().modifier(
        Modifier::new()
            .offset(Dp(176.0), Dp(y - 0.5))
            .size(Dp(end - 176.0), Dp(1.0))
            .background(ink),
    );
    spacer().modifier(
        Modifier::new()
            .offset(Dp(end - 4.0), Dp(y - 4.0))
            .size(Dp(8.0), Dp(8.0))
            .border_with_shape(Dp(1.0), ink, Shape::CAPSULE),
    );
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "icon_button_context_tests.rs"]
mod tests;
