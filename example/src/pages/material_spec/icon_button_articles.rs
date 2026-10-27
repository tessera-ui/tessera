//! Overview, guidance and contrast examples for icon buttons.
//!
//! ## Usage
//!
//! Explore action hierarchy, contextual commands and accessible icon contrast.

use tessera_components::{
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    icon_button::{
        IconButtonShape, IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button,
    },
    material_icons::filled,
    menus::{MenuAnchor, MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::{ModifierExt as _, SemanticsArgs},
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_ui::{
    Color, Dp, Modifier, PxPosition, PxRect, PxSize,
    accesskit::{HasPopup, Role},
    modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use crate::pages::material_spec::{gap, spec_table, srgb, subheading, type_style};

#[tessera]
pub(super) fn overview() {
    paragraph("Icon buttons represent common actions with a recognizable icon. Use them when the action is easy to understand without a persistent text label.".into());
    gap(Dp(16.0));
    paragraph("• Choose a default button for an action and a toggle button for a persistent selection.\n• Use filled, tonal, outlined or standard colors to establish hierarchy.\n• Choose among five sizes, two shapes and three widths.\n• On the web, show a concise tooltip on hover and keyboard focus.\n• Toggle icons can change from outlined when unselected to filled when selected.\n• Give every action a meaningful accessible name and a target of at least 48 × 48dp.".into());
    gap(Dp(32.0));
    super::overview_art();
    gap(Dp(48.0));
    subheading("Availability & resources".into());
    gap(Dp(24.0));
    spec_table(
        vec!["Resource".into(), "Status".into(), "Available here".into()],
        vec![
            vec![
                "Native icon button".into(),
                "Available".into(),
                "Default and toggle actions, sizing, shapes and color styles".into(),
            ],
            vec![
                "Material icon assets".into(),
                "Available".into(),
                "Bundled icons used by the live examples".into(),
            ],
            vec![
                "Theme and accessibility".into(),
                "Available".into(),
                "Theme colors, accessible names, keyboard focus and selection".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("May 2025: Material 3 Expressive".into());
    gap(Dp(24.0));
    paragraph("Material 3 Expressive expands icon buttons with five sizes, narrow and wide proportions, and round and square shapes. Pressing and selecting a button can change its shape, adding feedback beyond color alone.".into());
    gap(Dp(24.0));
    spec_table(
        vec!["Configuration".into(), "Expressive choices".into()],
        vec![
            vec![
                "Size".into(),
                "Extra small, small, medium, large and extra large".into(),
            ],
            vec![
                "Shape".into(),
                "Round and square, with pressed and selected shape feedback".into(),
            ],
            vec!["Width".into(), "Narrow, default and wide".into()],
            vec![
                "Color style".into(),
                "Filled, tonal, outlined and standard".into(),
            ],
        ],
    );
    gap(Dp(48.0));
    subheading("Differences from M2".into());
    gap(Dp(24.0));
    paragraph("Material 2 commonly presents an icon action without a persistent container. Material 3 Expressive retains that standard style and adds more ways to emphasize actions through containers, dimensions and shape. The examples below use native controls to compare those treatments.".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            comparison(false);
            comparison(true);
        });
}

#[tessera]
pub(in crate::pages::material_spec) fn guidance(title: String) {
    match title.as_str() {
        "Usage" => {
            paragraph("Use icon buttons for familiar, frequently used actions such as favoriting, archiving or marking a message as read. Keep the most useful commands visible and place secondary actions in an overflow menu. Prefer a text button when the icon alone would be ambiguous.".into());
            gap(Dp(24.0));
            mail_scene();
            gap(Dp(24.0));
            paragraph("The heart retains its selected state. Archive moves this message out of the inbox, Mark as read changes its read state, and More options opens a real menu with local message actions.".into());
        }
        "Anatomy" => {
            paragraph("An icon button contains an icon, an optional visible container and a larger interactive target. Container shape and color communicate emphasis; the accessible name describes the action rather than the picture.".into());
            gap(Dp(24.0));
            super::anatomy();
        }
        "Color" => {
            paragraph("Use filled icon buttons for strong emphasis, tonal buttons for a softer container, outlined buttons for a visible boundary and standard buttons for the least emphasis. Match colors to the surrounding surface and keep the icon distinguishable in every enabled state.".into());
            gap(Dp(24.0));
            super::colors();
        }
        "Size" => {
            paragraph("Choose a size that matches the importance and context of the action. Small controls suit dense interfaces; larger sizes emphasize key actions or sit alongside larger content. Keep at least a 48 × 48dp interactive target even when the visible extra-small or small container is smaller.".into());
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    for (size, label) in [
                        (IconButtonSize::ExtraSmall, "XS"),
                        (IconButtonSize::Small, "S"),
                        (IconButtonSize::Medium, "M"),
                        (IconButtonSize::Large, "L"),
                        (IconButtonSize::ExtraLarge, "XL"),
                    ] {
                        column().children(move || {
                            demo(
                                size,
                                IconButtonWidth::Uniform,
                                true,
                                format!("Size {label}"),
                            );
                            gap(Dp(12.0));
                            paragraph(label.into());
                        });
                    }
                });
            gap(Dp(24.0));
            super::context::density();
        }
        "Width" => {
            paragraph("Width changes the container's horizontal proportions without changing the icon size or the button's height. Use narrow buttons where space is limited, default width for balanced proportions and wide buttons when an action needs more horizontal emphasis. Do not shrink the interactive target below 48dp.".into());
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(32.0))
                .line_spacing(Dp(24.0))
                .children(|| {
                    for (width, label) in [
                        (IconButtonWidth::Narrow, "Narrow"),
                        (IconButtonWidth::Uniform, "Default"),
                        (IconButtonWidth::Wide, "Wide"),
                    ] {
                        column().children(move || {
                            demo(
                                IconButtonSize::Medium,
                                width,
                                false,
                                format!("Width {label}"),
                            );
                            gap(Dp(12.0));
                            paragraph(label.into());
                        });
                    }
                });
        }
        "Shape" => {
            paragraph("Round and square shapes offer different visual character. Pressed shapes share the same size-specific corner radius; selection swaps round and square shapes. Keep the shape language consistent within a group of related actions.".into());
            gap(Dp(24.0));
            super::shape_morph();
        }
        "Placement" => {
            super::context::placement();
        }
        "Behavior" => {
            paragraph("Default icon buttons perform an action when activated. Toggle icon buttons retain selection and expose that state to assistive technology. Hover and focus feedback help people identify the current target; pressed feedback confirms contact, and activation occurs only for a completed press.".into());
            gap(Dp(24.0));
            super::hover_preview();
            gap(Dp(24.0));
            spec_table(
                vec!["Behavior".into(), "Expected result".into()],
                vec![
                    vec![
                        "Default action".into(),
                        "Performs its command without retaining a selected state".into(),
                    ],
                    vec![
                        "Toggle action".into(),
                        "Retains selection and changes its icon or shape".into(),
                    ],
                    vec![
                        "Overflow action".into(),
                        "Opens related choices; Escape dismisses the menu".into(),
                    ],
                    vec![
                        "Disabled action".into(),
                        "Does not activate or participate in keyboard traversal".into(),
                    ],
                ],
            );
        }
        _ => {
            paragraph("Use recognizable icons for contextual actions, provide meaningful labels, preserve adequate target size and contrast, and communicate persistent selection independently from transient hover, focus or press feedback.".into());
        }
    }
}

#[tessera]
pub(in crate::pages::material_spec) fn contrast() {
    paragraph("Enabled icon buttons need at least 3:1 contrast between the icon and its adjacent background. Any visual boundary needed to identify the control also needs at least 3:1 contrast with adjacent colors. Do not rely on color alone to communicate selection.".into());
    gap(Dp(16.0));
    paragraph("The examples use the same white surface and a live favorite action. The dark icon is above 3:1; the light gray icon is intentionally below 3:1 and is shown only as a negative example. Text labels remain readable.".into());
    gap(Dp(24.0));
    flow_row()
        .item_spacing(Dp(24.0))
        .line_spacing(Dp(24.0))
        .children(|| {
            contrast_example(true);
            contrast_example(false);
        });
}

#[tessera]
fn paragraph(value: String) {
    text().content(value).style(type_style(16.0, 24.0));
}

#[tessera]
fn comparison(modern: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let selected = remember(|| false);
    column()
        .modifier(
            Modifier::new()
                .width(Dp(280.0))
                .padding_all(Dp(24.0))
                .background_with_shape(
                    scheme.surface_container_low,
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            paragraph(if modern {
                "Material 3 Expressive".into()
            } else {
                "Material 2 treatment".into()
            });
            gap(Dp(24.0));
            icon_button()
                .variant(if modern {
                    IconButtonVariant::FilledTonal
                } else {
                    IconButtonVariant::Standard
                })
                .size(IconButtonSize::Small)
                .shape(if modern && selected.get() {
                    Shape::rounded_rectangle(Dp(12.0))
                } else {
                    Shape::CAPSULE
                })
                .selected_optional(if modern { Some(selected.get()) } else { None })
                .icon(filled::EDIT_SVG)
                .accessibility_label(if modern {
                    "Toggle Expressive edit action"
                } else {
                    "Activate standard edit action"
                })
                .on_click(move || selected.set(!selected.get()));
            gap(Dp(16.0));
            paragraph(if modern {
                "Container, shape and persistent selection".into()
            } else if selected.get() {
                "Standard edit action activated".into()
            } else {
                "Recognizable icon without a persistent container".into()
            });
        });
}

#[tessera]
fn demo(size: IconButtonSize, width: IconButtonWidth, play: bool, label: String) {
    let selected = remember(|| false);
    icon_button()
        .size(size)
        .width(width)
        .variant(IconButtonVariant::FilledTonal)
        .shape_style(IconButtonShape::Round)
        .shape(if selected.get() {
            Shape::rounded_rectangle(size.square_radius())
        } else {
            Shape::CAPSULE
        })
        .selected(selected.get())
        .icon(if play {
            filled::PLAY_ARROW_SVG
        } else {
            filled::MORE_VERT_SVG
        })
        .accessibility_label(label)
        .on_click(move || selected.set(!selected.get()));
}

#[tessera]
fn contrast_example(good: bool) {
    let favorite = remember(|| false);
    column()
        .modifier(
            Modifier::new()
                .width(Dp(280.0))
                .padding_all(Dp(24.0))
                .background_with_shape(Color::WHITE, Shape::rounded_rectangle(Dp(24.0)))
                .border_with_shape(
                    Dp(1.0),
                    srgb(150, 150, 150),
                    Shape::rounded_rectangle(Dp(24.0)),
                ),
        )
        .children(move || {
            text()
                .content(if good {
                    "Above 3:1 — Do"
                } else {
                    "Below 3:1 — Don't"
                })
                .style(type_style(16.0, 24.0))
                .color(Color::BLACK);
            gap(Dp(24.0));
            icon_button()
                .variant(IconButtonVariant::Standard)
                .shape(Shape::CAPSULE)
                .color(Color::WHITE)
                .content_color(if good {
                    Color::BLACK
                } else {
                    srgb(210, 210, 210)
                })
                .selected(favorite.get())
                .icon(if favorite.get() {
                    filled::FAVORITE_SVG
                } else {
                    filled::FAVORITE_BORDER_SVG
                })
                .accessibility_label(if good {
                    "Favorite, sufficient contrast example"
                } else {
                    "Favorite, intentionally insufficient contrast example"
                })
                .on_click(move || favorite.set(!favorite.get()));
        });
}

#[tessera]
fn mail_scene() {
    let favorite = remember(|| false);
    let archived = remember(|| false);
    let read = remember(|| false);
    let deleted = remember(|| false);
    let menu = remember(MenuController::new);
    let anchor = remember(PxRect::default);
    let origin = remember(PxRect::default);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().height(Dp(320.0)).on_globally_positioned(move |rect| {
            if origin.get() != rect { origin.set(rect); }
        }))
        .children(move || {
            let a = anchor.get();
            let o = origin.get();
            menu_provider()
                .controller(menu)
                .placement(MenuPlacement::BelowEnd)
                .anchor(MenuAnchor::new(PxPosition::new(a.x - o.x, a.y - o.y), PxSize::new(a.width, a.height)))
                .main_content(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)).background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0))).semantics(SemanticsArgs { role: Some(Role::Group), label: Some("Message actions".into()), merge_descendants: true, ..Default::default() }))
                        .children(move || {
                            subheading("Weekend plans".into());
                            gap(Dp(8.0));
                            paragraph(if deleted.get() { "Message moved to Trash. Choose More options → Move to inbox to restore it.".into() } else { "Alex · Let's meet for a walk on Saturday.".into() });
                            gap(Dp(16.0));
                            flow_row().item_spacing(Dp(12.0)).line_spacing(Dp(8.0)).children(move || {
                                icon_button()
                                    .variant(IconButtonVariant::Standard)
                                    .shape(if favorite.get() { Shape::rounded_rectangle(Dp(12.0)) } else { Shape::CAPSULE })
                                    .selected(favorite.get())
                                    .icon(if favorite.get() { filled::FAVORITE_SVG } else { filled::FAVORITE_BORDER_SVG })
                                    .accessibility_label(if favorite.get() { "Remove favorite" } else { "Add favorite" })
                                    .on_click(move || favorite.set(!favorite.get()));
                                icon_button().variant(IconButtonVariant::Standard).shape(Shape::CAPSULE).icon(filled::ARCHIVE_SVG).enabled(!archived.get() && !deleted.get()).accessibility_label("Archive message").on_click(move || archived.set(true));
                                icon_button().variant(IconButtonVariant::Standard).shape(Shape::CAPSULE).icon(filled::MARK_EMAIL_READ_SVG).selected(read.get()).accessibility_label(if read.get() { "Mark as unread" } else { "Mark as read" }).on_click(move || read.set(!read.get()));
                                icon_button()
                                    .variant(IconButtonVariant::Standard)
                                    .shape(Shape::CAPSULE)
                                    .icon(filled::MORE_VERT_SVG)
                                    .accessibility_label("More message options")
                                    .modifier(Modifier::new().on_globally_positioned(move |rect| {
                                        if anchor.get() != rect { anchor.set(rect); }
                                    }).semantics(SemanticsArgs { expanded: Some(menu.with(MenuController::is_open)), has_popup: Some(HasPopup::Menu), ..Default::default() }))
                                    .on_click(move || menu.with_mut(MenuController::toggle));
                            });
                            gap(Dp(16.0));
                            paragraph(format!("Folder: {} · {} · {}", if deleted.get() { "Trash" } else if archived.get() { "Archive" } else { "Inbox" }, if read.get() { "Read" } else { "Unread" }, if favorite.get() { "Favorite" } else { "Not favorited" }));
                        });
                })
                .menu_content(move || {
                    menu_item().label("Move to inbox").leading_icon(filled::INBOX_SVG).on_click(move || { archived.set(false); deleted.set(false); });
                    menu_item().label("Mark as unread").leading_icon(filled::EMAIL_SVG).on_click(move || read.set(false));
                    menu_item().label("Delete message").leading_icon(filled::DELETE_SVG).on_click(move || deleted.set(true));
                });
        });
}
