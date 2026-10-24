//! Accessible dialog interaction and labeling examples.
//!
//! ## Usage
//!
//! Explore keyboard navigation and labels for dialog controls.

use std::sync::OnceLock;

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    button::button,
    icon::icon,
    image::{ImageData, TryIntoImageData as _, image},
    material_icons::filled,
    modifier::{
        InteractionVisualState, MinimumInteractiveComponentEnforcement, ModifierExt as _, Padding,
        SemanticsArgs,
    },
    row::row,
    shape_def::Shape,
    spacer::spacer,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Callback, Dp, FocusRequester, Modifier, RenderSlotWith, accesskit::Role,
    modifier::FocusModifierExt as _, provide_context, remember, tessera, use_context,
};

use super::{
    annotations, dialog_marker, dialog_phone_status, dialog_scene_action, dialog_scene_input,
    dialog_type_style, gap,
};

#[tessera]
pub(super) fn initial_focus() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                keyboard_figure(false);
            });
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                keyboard_figure(true);
            });
        });
}

#[tessera]
pub(super) fn labeling() {
    figure(
        676.0,
        363.0,
        RenderSlotWith::new(|scale| {
            traffic_labels(scale);
        }),
    );
    gap(Dp(24.0));
    figure(
        676.0,
        363.0,
        RenderSlotWith::new(|scale| {
            event_labels(scale);
        }),
    );
}

#[tessera]
pub(super) fn interaction_style() {
    use_sparingly();
    gap(Dp(24.0));
    figure(
        676.0,
        459.0,
        RenderSlotWith::new(|scale| {
            enlarged_text(scale);
        }),
    );
    gap(Dp(24.0));
    figure(
        676.0,
        363.0,
        RenderSlotWith::new(|scale| {
            contained_elements(scale);
        }),
    );
}

#[tessera]
fn figure(base_width: f64, height: f64, content: RenderSlotWith<f64>) {
    let width = remember(move || base_width);
    let scale = width.get() / base_width;
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .fill_max_width()
                .height(Dp(height * scale))
                .clip_to_bounds()
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                )
                .on_globally_positioned(move |rect: tessera_ui::PxRect| {
                    let next = Dp::from_pixels_f32(rect.width.to_f32()).0;
                    if (width.get() - next).abs() > 0.1 {
                        width.set(next);
                    }
                }),
        )
        .children(move || {
            provide_context(
                || MinimumInteractiveComponentEnforcement { enabled: false },
                move || {
                    content.render(scale);
                },
            );
        });
}

#[tessera]
fn keyboard_figure(reverse: bool) {
    figure(
        652.0,
        600.0,
        RenderSlotWith::new(move |scale| {
            keyboard_content(reverse, scale);
        }),
    );
}

#[tessera]
fn keyboard_content(reverse: bool, scale: f64) {
    let shown = remember(|| true);
    let selected = remember(move || if reverse { 0_usize } else { 1 });
    let cancel = remember(FocusRequester::new).get();
    let delete = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    grid(652.0, 600.0, scale);
    if shown.get() {
        boxed().modifier(Modifier::new().offset(Dp(170.0 * scale), Dp(71.0 * scale))
            .size(Dp(312.0 * scale), Dp(260.0 * scale))
            .background_with_shape(scheme.surface_container_high, Shape::rounded_rectangle(Dp(28.0 * scale))))
            .children(move || {
                boxed().modifier(Modifier::new().offset(Dp(144.0 * scale), Dp(24.0 * scale)))
                    .children(move || {
                        icon().vector(filled::DELETE_OUTLINE_SVG).size(Dp(24.0 * scale)).tint(scheme.secondary);
                    });
                placed_text("Permanently delete?".into(), 24.0, 32.0, 46.0, 64.0, 264.0, scale);
                text().content("Deleting the selected messages will also remove them from all synced devices. You won’t be able to recover them later.")
                    .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                    .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(112.0 * scale)).width(Dp(264.0 * scale)));
                for (index, label, x, width, focus) in [
                    (0, "Cancel", 127.0, 78.0, cancel),
                    (1, "Delete", 213.0, 75.0, delete),
                ] {
                    button().text().accessibility_label(format!("{} example {label}", if reverse { "Reverse" } else { "Forward" }))
                        .presentation_state(if selected.get() == index { InteractionVisualState::Focus } else { InteractionVisualState::Rest })
                        .shape(Shape::rounded_rectangle(Dp(12.0 * scale)))
                        .content_padding(Padding::all(Dp::ZERO))
                        .modifier(Modifier::new().offset(Dp(x * scale), Dp(197.0 * scale))
                            .size(Dp(width * scale), Dp(40.0 * scale)).focus_requester(focus))
                        .on_click(move || shown.set(false))
                        .child(move || {
                            text().content(label).style(dialog_type_style(14.0, 20.0, scale));
                        });
                }
            });
    } else {
        dialog_scene_action(
            "Reset example".into(),
            "Reset keyboard dialog example".into(),
            246.0,
            210.0,
            160.0,
            scale,
            Callback::new(move || shown.set(true)),
        );
    }
    let (x, width) = if reverse {
        (208.0, 236.0)
    } else {
        (254.0, 144.0)
    };
    button()
        .filled()
        .shape(Shape::rounded_rectangle(Dp(8.0 * scale)))
        .color(scheme.tertiary)
        .content_color(scheme.on_tertiary)
        .content_padding(Padding::all(Dp::ZERO))
        .accessibility_label(if reverse {
            "Move to the previous dialog action"
        } else {
            "Move to the next dialog action"
        })
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(371.0 * scale))
                .size(Dp(width * scale), Dp(80.0 * scale)),
        )
        .on_click(move || {
            let next = 1 - selected.get();
            selected.set(next);
            if next == 0 {
                cancel.request_focus();
            } else {
                delete.request_focus();
            }
        })
        .child(move || {
            text()
                .content(if reverse { "Shift + Tab" } else { "Tab" })
                .style(dialog_type_style(28.0, 36.0, scale));
        });
    numbered(if reverse { 2 } else { 1 }, 326.0, 505.0, scale);
}

#[tessera]
fn traffic_labels(scale: f64) {
    let shown = remember(|| true);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    map_background(scale);
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(420.0 * scale), Dp(363.0 * scale))
                .background(scheme.scrim.with_alpha(0.32)),
        )
        .children(|| {});
    if shown.get() {
        boxed().modifier(Modifier::new().offset(Dp(50.0 * scale), Dp(60.0 * scale))
            .size(Dp(312.0 * scale), Dp(240.0 * scale))
            .background_with_shape(scheme.surface_container_high, Shape::rounded_rectangle(Dp(28.0 * scale)))
            .border_with_shape(Dp(4.0 * scale), scheme.tertiary, Shape::rounded_rectangle(Dp(28.0 * scale)))
            .semantics(SemanticsArgs { role: Some(Role::AlertDialog), label: Some("Set up traffic updates?".into()), ..Default::default() }))
            .children(move || {
                placed_text("Set up traffic updates?".into(), 24.0, 32.0, 24.0, 24.0, 264.0, scale);
                text().content("Get notified when traffic is impacted along your usual routes. Add your preferred routes and travel times to get started with traffic updates.")
                    .style(dialog_type_style(14.0, 20.0, scale)).color(scheme.on_surface_variant)
                    .modifier(Modifier::new().offset(Dp(24.0 * scale), Dp(72.0 * scale)).width(Dp(264.0 * scale)));
                dialog_scene_action("No thanks".into(), "Decline traffic updates".into(), 128.0, 180.0, 96.0, scale,
                    Callback::new(move || shown.set(false)));
                dialog_scene_action("OK".into(), "Enable traffic updates".into(), 240.0, 180.0, 48.0, scale,
                    Callback::new(move || shown.set(false)));
            });
    } else {
        dialog_scene_action(
            "Show traffic dialog".into(),
            "Show traffic label example".into(),
            110.0,
            160.0,
            200.0,
            scale,
            Callback::new(move || shown.set(true)),
        );
    }
    annotation_card(
        440.0,
        108.0,
        224.0,
        148.0,
        "Set up traffic updates?".into(),
        "Alert dialog".into(),
        scale,
    );
    connector(362.0, 180.0, 440.0, scale);
}

#[tessera]
fn event_labels(scale: f64) {
    let shown = remember(|| true);
    let saved = remember(|| false);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(412.0 * scale), Dp(363.0 * scale))
                .background(scheme.surface),
        )
        .children(move || {
            if shown.get() {
                dialog_phone_status(scale);
                event_header(
                    "New event".into(),
                    scale,
                    Callback::new(move || shown.set(false)),
                    Callback::new(move || {
                        saved.set(true);
                        shown.set(false);
                    }),
                );
                dialog_scene_input(
                    24.0,
                    118.0,
                    364.0,
                    "Label".into(),
                    "Input".into(),
                    "Event category".into(),
                    true,
                    scale,
                );
                dialog_scene_input(
                    24.0,
                    190.0,
                    364.0,
                    "".into(),
                    "Carmen’s Birthday Surprise Party".into(),
                    "Enter event name".into(),
                    false,
                    scale,
                );
                dialog_scene_input(
                    24.0,
                    262.0,
                    364.0,
                    "".into(),
                    "123 El Camino, Cleveland, OH 44113".into(),
                    "Event location".into(),
                    false,
                    scale,
                );
                placed_text("From".into(), 12.0, 16.0, 24.0, 350.0, 364.0, scale);
                outline(335.0, 48.0, 65.0, 40.0, scale);
                outline(23.0, 190.0, 365.0, 56.0, scale);
            } else {
                placed_text(
                    if saved.get() {
                        "Event saved"
                    } else {
                        "Event closed"
                    }
                    .into(),
                    16.0,
                    24.0,
                    24.0,
                    118.0,
                    364.0,
                    scale,
                );
                dialog_scene_action(
                    "Show event".into(),
                    "Reset event label example".into(),
                    24.0,
                    170.0,
                    140.0,
                    scale,
                    Callback::new(move || {
                        saved.set(false);
                        shown.set(true);
                    }),
                );
            }
        });
    annotation_card(
        440.0,
        13.0,
        170.0,
        144.0,
        "UI text (“Save”)".into(),
        "Button".into(),
        scale,
    );
    annotation_card(
        440.0,
        176.0,
        182.0,
        144.0,
        "Enter event name".into(),
        "Text field".into(),
        scale,
    );
    connector(400.0, 69.0, 440.0, scale);
    connector(388.0, 219.0, 440.0, scale);
}

#[tessera]
fn event_header(title: String, scale: f64, close: Callback, save: Callback) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    button()
        .text()
        .accessibility_label(format!("Close {title}"))
        .on_click_shared(close)
        .content_padding(Padding::all(Dp::ZERO))
        .modifier(
            Modifier::new()
                .offset(Dp(8.0 * scale), Dp(56.0 * scale))
                .size(Dp(48.0 * scale), Dp(64.0 * scale)),
        )
        .child(move || {
            icon()
                .vector(filled::CLOSE_SVG)
                .size(Dp(24.0 * scale))
                .tint(scheme.on_surface);
        });
    placed_text(title, 22.0, 28.0, 56.0, 74.0, 268.0, scale);
    dialog_scene_action("Save".into(), "Save".into(), 332.0, 68.0, 64.0, scale, save);
}

#[tessera]
fn annotation_card(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    label: String,
    role: String,
    scale: f64,
) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(height * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(12.0 * scale)))
                .border_with_shape(
                    Dp(4.0 * scale),
                    scheme.tertiary,
                    Shape::rounded_rectangle(Dp(12.0 * scale)),
                )
                .padding_all(Dp(22.0 * scale))
                .semantics(SemanticsArgs {
                    role: Some(Role::Note),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            placed_text("Label".into(), 16.0, 20.0, 0.0, 0.0, width - 44.0, scale);
            text()
                .content(label.clone())
                .style(dialog_type_style(16.0, 20.0, scale))
                .color(scheme.tertiary)
                .modifier(
                    Modifier::new()
                        .offset(Dp::ZERO, Dp(22.0 * scale))
                        .width(Dp((width - 44.0) * scale)),
                );
            placed_text("Role".into(), 16.0, 20.0, 0.0, 56.0, width - 44.0, scale);
            text()
                .content(role.clone())
                .style(dialog_type_style(16.0, 20.0, scale))
                .color(scheme.tertiary)
                .modifier(
                    Modifier::new()
                        .offset(Dp::ZERO, Dp(80.0 * scale))
                        .width(Dp((width - 44.0) * scale)),
                );
        });
}

#[tessera]
fn use_sparingly() {
    row()
        .modifier(Modifier::new().fill_max_width())
        .children(|| {
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                figure(
                    676.0,
                    600.0,
                    RenderSlotWith::new(|scale| {
                        photo_content(false, scale);
                    }),
                );
            });
            spacer().modifier(Modifier::new().width(Dp(24.0)));
            boxed().modifier(Modifier::new().weight(1.0)).children(|| {
                figure(
                    676.0,
                    600.0,
                    RenderSlotWith::new(|scale| {
                        photo_content(true, scale);
                    }),
                );
            });
        });
}

fn photo_data() -> ImageData {
    static PHOTO: OnceLock<ImageData> = OnceLock::new();
    PHOTO
        .get_or_init(|| {
            include_bytes!("../../../assets/material_spec/dialog-accessibility-photos.png")
                .as_slice()
                .try_into_image_data()
                .expect("dialog photo examples")
        })
        .clone()
}

#[tessera]
fn photo_region(x: f64, y: f64, width: f64, height: f64, radius: f64, scale: f64) {
    let photo = photo_data();
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(height * scale))
                .clip_shape(Shape::rounded_rectangle(Dp(radius * scale)))
                .semantics(SemanticsArgs {
                    role: Some(Role::Image),
                    label: Some("Family photograph".into()),
                    ..Default::default()
                }),
        )
        .children(move || {
            image().raster(photo.clone()).modifier(
                Modifier::new()
                    .offset(Dp(-x * scale), Dp(-y * scale))
                    .required_size(Dp(676.0 * scale), Dp(600.0 * scale)),
            );
        });
}

#[tessera]
fn photo_content(modal: bool, scale: f64) {
    let shown = remember(|| true);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(114.0 * scale), Dp(-20.0 * scale))
                .size(Dp(428.0 * scale), Dp(564.0 * scale))
                .background_with_shape(
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(28.0 * scale)),
                ),
        )
        .children(|| {});
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(122.0 * scale), Dp::ZERO)
                .size(Dp(412.0 * scale), Dp(536.0 * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(20.0 * scale))),
        )
        .children(|| {});
    photo_region(122.0, 83.0, 274.0, 274.0, 80.0, scale);
    photo_region(398.0, 83.0, 136.0, 136.0, 4.0, scale);
    photo_region(398.0, 221.0, 136.0, 136.0, 4.0, scale);
    photo_region(122.0, 359.0, 136.0, 136.0, 4.0, scale);
    photo_region(260.0, 359.0, 274.0, 177.0, 4.0, scale);
    photo_region(122.0, 497.0, 136.0, 39.0, 0.0, scale);
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp(146.0 * scale), Dp(12.0 * scale))
                .size(Dp(28.0 * scale), Dp(42.0 * scale))
                .background_with_shape(
                    scheme.primary_container,
                    Shape::rounded_rectangle(Dp(16.0 * scale)),
                ),
        )
        .children(move || {
            text()
                .content("Jun\n8")
                .style(dialog_type_style(12.0, 16.0, scale))
                .color(scheme.on_primary_container);
        });
    placed_text("Today".into(), 48.0, 56.0, 188.0, 6.0, 300.0, scale);
    if modal && shown.get() {
        boxed()
            .modifier(
                Modifier::new()
                    .offset(Dp(122.0 * scale), Dp::ZERO)
                    .size(Dp(412.0 * scale), Dp(536.0 * scale))
                    .background(scheme.scrim.with_alpha(0.32)),
            )
            .children(|| {});
        boxed()
            .modifier(
                Modifier::new()
                    .offset(Dp(171.0 * scale), Dp(175.0 * scale))
                    .size(Dp(314.0 * scale), Dp(200.0 * scale))
                    .background_with_shape(
                        scheme.surface_container_high,
                        Shape::rounded_rectangle(Dp(28.0 * scale)),
                    ),
            )
            .children(move || {
                placed_text("Sync complete".into(), 24.0, 32.0, 24.0, 24.0, 266.0, scale);
                text()
                    .content("Three new photos were synced to your device")
                    .style(dialog_type_style(14.0, 20.0, scale))
                    .color(scheme.on_surface_variant)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(24.0 * scale), Dp(72.0 * scale))
                            .width(Dp(266.0 * scale)),
                    );
                dialog_scene_action(
                    "OK".into(),
                    "Acknowledge sync complete".into(),
                    238.0,
                    136.0,
                    52.0,
                    scale,
                    Callback::new(move || shown.set(false)),
                );
            });
    } else {
        boxed()
            .alignment(Alignment::Center)
            .modifier(
                Modifier::new()
                    .offset(Dp(156.0 * scale), Dp(448.0 * scale))
                    .size(Dp(344.0 * scale), Dp(48.0 * scale))
                    .background_with_shape(
                        scheme.inverse_surface,
                        Shape::rounded_rectangle(Dp(4.0 * scale)),
                    ),
            )
            .children(move || {
                text()
                    .content("Three new photos were synced to your device")
                    .style(dialog_type_style(14.0, 20.0, scale))
                    .color(scheme.inverse_on_surface);
            });
        if modal {
            dialog_scene_action(
                "Show sync dialog".into(),
                "Reset blocking message example".into(),
                220.0,
                554.0,
                216.0,
                scale,
                Callback::new(move || shown.set(true)),
            );
        }
    }
}

#[tessera]
fn enlarged_text(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let shown = remember(|| true);
    map_background(scale);
    boxed()
        .modifier(
            Modifier::new()
                .size(Dp(676.0 * scale), Dp(459.0 * scale))
                .background(scheme.scrim.with_alpha(0.32)),
        )
        .children(|| {});
    if shown.get() {
        boxed()
            .modifier(
                Modifier::new()
                    .offset(Dp(181.0 * scale), Dp(24.0 * scale))
                    .width(Dp(312.0 * scale))
                    .height(Dp(520.0 * scale))
                    .background_with_shape(
                        scheme.surface_container_high,
                        Shape::rounded_rectangle(Dp(28.0 * scale)),
                    ),
            )
            .children(move || {
                placed_text(
                    "Clear your location history?".into(),
                    48.0,
                    64.0,
                    24.0,
                    24.0,
                    264.0,
                    scale,
                );
                text()
                    .content("You won’t be able to restore your history once it’s cleared.")
                    .style(dialog_type_style(28.0, 36.0, scale))
                    .color(scheme.on_surface_variant)
                    .modifier(
                        Modifier::new()
                            .offset(Dp(24.0 * scale), Dp(232.0 * scale))
                            .width(Dp(264.0 * scale)),
                    );
                button()
                    .text()
                    .accessibility_label("Don't clear location history")
                    .content_padding(Padding::all(Dp::ZERO))
                    .modifier(
                        Modifier::new()
                            .offset(Dp(120.0 * scale), Dp(368.0 * scale))
                            .size(Dp(168.0 * scale), Dp(64.0 * scale)),
                    )
                    .on_click(move || shown.set(false))
                    .child(move || {
                        text()
                            .content("Don’t clear")
                            .style(dialog_type_style(28.0, 36.0, scale));
                    });
            });
    } else {
        dialog_scene_action(
            "Show enlarged dialog".into(),
            "Reset enlarged text example".into(),
            210.0,
            200.0,
            220.0,
            scale,
            Callback::new(move || shown.set(true)),
        );
    }
}

#[tessera]
fn contained_elements(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let shown = remember(|| true);
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(121.0 * scale), Dp(85.0 * scale))
                .size(Dp(428.0 * scale), Dp(600.0 * scale))
                .background_with_shape(
                    scheme.outline_variant,
                    Shape::rounded_rectangle(Dp(28.0 * scale)),
                ),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(8.0 * scale), Dp(8.0 * scale))
                        .size(Dp(412.0 * scale), Dp(592.0 * scale))
                        .background_with_shape(
                            scheme.surface,
                            Shape::rounded_rectangle(Dp(20.0 * scale)),
                        ),
                )
                .children(move || {
                    dialog_phone_status(scale);
                    if shown.get() {
                        event_header(
                            "New contact".into(),
                            scale,
                            Callback::new(move || shown.set(false)),
                            Callback::new(move || shown.set(false)),
                        );
                        boxed()
                            .modifier(Modifier::new().offset(Dp(16.0 * scale), Dp(134.0 * scale)))
                            .children(move || {
                                icon()
                                    .vector(filled::PERSON_SVG)
                                    .size(Dp(24.0 * scale))
                                    .tint(scheme.primary);
                            });
                        placed_text("Name".into(), 14.0, 20.0, 56.0, 136.0, 320.0, scale);
                        dialog_scene_input(
                            56.0,
                            182.0,
                            332.0,
                            "First Name".into(),
                            "Carmen".into(),
                            "First Name".into(),
                            false,
                            scale,
                        );
                        dialog_scene_input(
                            56.0,
                            254.0,
                            332.0,
                            "Last Name".into(),
                            "".into(),
                            "Last Name".into(),
                            false,
                            scale,
                        );
                    } else {
                        dialog_scene_action(
                            "Show contact".into(),
                            "Reset contact elements example".into(),
                            56.0,
                            182.0,
                            180.0,
                            scale,
                            Callback::new(move || shown.set(true)),
                        );
                    }
                });
        });
    dialog_marker(1, 70.0 * scale, 272.0 * scale, scale);
    dialog_marker(2, 203.0 * scale, 30.0 * scale, scale);
    dialog_marker(3, 582.0 * scale, 161.0 * scale, scale);
    let mut svg = String::new();
    for (x, y, length, vertical) in [
        (96.0 * scale, 288.0 * scale, 89.0 * scale, false),
        (215.0 * scale, 62.0 * scale, 106.0 * scale, true),
        (516.0 * scale, 177.0 * scale, 64.0 * scale, false),
    ] {
        let mut distance = 0.0_f64;
        while distance < length {
            let span = (length - distance).min(1.0);
            let (x, y, axis) = if vertical {
                (x, y + distance, "v")
            } else {
                (x + distance, y, "h")
            };
            svg.push_str(&format!(
                "<path d='M{x} {y}{axis}{span}' stroke='#79747e' stroke-width='{scale}' fill='none'/>"
            ));
            distance += 3.0;
        }
    }
    annotations::lines(Dp(676.0 * scale), Dp(363.0 * scale), svg);
}

#[tessera]
fn map_background(scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed().modifier(Modifier::new().width(Dp(420.0 * scale)).height(Dp(459.0 * scale)).background(scheme.surface_container))
        .children(move || {
            annotations::lines(Dp(420.0 * scale), Dp(459.0 * scale), format!("<g fill='none' stroke='#dad4c5' stroke-width='{}'><path d='M0 {}L{} 0M0 {}L{} 0M0 {}L{} {}M{} 0L{} {}M{} 0L{} {}'/></g><path d='M0 {}C{} {},{} {},{} 0' fill='none' stroke='#c2d9e9' stroke-width='{}'/>", 6.0*scale,200.0*scale,180.0*scale,330.0*scale,360.0*scale,50.0*scale,400.0*scale,240.0*scale,120.0*scale,40.0*scale,420.0*scale,459.0*scale,200.0*scale,390.0*scale,230.0*scale,90.0*scale,180.0*scale,120.0*scale,90.0*scale,140.0*scale,20.0*scale));
            for (label, x, y) in [("2nd St",45.0,45.0),("Firestone Ave",115.0,24.0),("Springdale St",320.0,88.0),("Mud Island Amphitheater",16.0,300.0),("Music venue",16.0,328.0)] {
                placed_text(label.into(), if y>290.0 {16.0} else {8.0}, if y>290.0 {24.0} else {12.0}, x,y,300.0,scale);
            }
        });
}

#[tessera]
fn placed_text(value: String, size: f64, line: f64, x: f64, y: f64, width: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    text()
        .content(value)
        .style(dialog_type_style(size, line, scale))
        .color(scheme.on_surface)
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .width(Dp(width * scale)),
        );
}

#[tessera]
fn numbered(value: usize, x: f64, y: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(Dp((x - 24.0) * scale), Dp((y - 24.0) * scale))
                .size(Dp(48.0 * scale), Dp(48.0 * scale))
                .background_with_shape(scheme.inverse_surface, Shape::Ellipse),
        )
        .children(move || {
            text()
                .content(value.to_string())
                .style(dialog_type_style(28.0, 36.0, scale))
                .color(scheme.inverse_on_surface);
        });
}

#[tessera]
fn connector(x: f64, y: f64, end: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    spacer().modifier(
        Modifier::new()
            .offset(Dp(x * scale), Dp((y - 2.0) * scale))
            .size(Dp((end - x) * scale), Dp(4.0 * scale))
            .background(scheme.tertiary),
    );
}

#[tessera]
fn outline(x: f64, y: f64, width: f64, height: f64, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .offset(Dp(x * scale), Dp(y * scale))
                .size(Dp(width * scale), Dp(height * scale))
                .border_with_shape(
                    Dp(4.0 * scale),
                    scheme.tertiary,
                    Shape::rounded_rectangle(Dp(6.0 * scale)),
                ),
        )
        .children(|| {});
}

#[tessera]
fn grid(width: f64, height: f64, scale: f64) {
    let mut svg = String::new();
    for row in 1..(height / 8.0) as usize {
        for column in 1..(width / 8.0) as usize {
            svg.push_str(&format!(
                "<rect x='{}' y='{}' width='{}' height='{}' fill='#79747e' fill-opacity='.4'/>",
                (column as f64 * 8.0 + 2.0) * scale,
                (row as f64 * 8.0 + 2.0) * scale,
                0.6 * scale,
                0.6 * scale
            ));
        }
    }
    annotations::lines(Dp(width * scale), Dp(height * scale), svg);
}
