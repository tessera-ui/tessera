//! Overview, guidance and contrast examples for icon buttons.
//!
//! ## Usage
//!
//! Explore action hierarchy, contextual commands and accessible icon contrast.

use tessera_components::{
    boxed::boxed,
    column::column,
    flow_row::flow_row,
    icon_button::{IconButtonSize, IconButtonVariant, IconButtonWidth, icon_button},
    material_icons::filled,
    menus::{MenuController, MenuPlacement, menu_item, menu_provider},
    modifier::ModifierExt as _,
    shape_def::Shape,
    text::text,
    theme::MaterialTheme,
};
use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, Modifier,
    accesskit::{HasPopup, Role},
    modifier::FocusModifierExt as _,
    remember, tessera, use_context,
};

use crate::{
    pages::material_spec::{gap, spec_table, srgb, subheading, type_style},
    res::I18N,
};

#[tessera]
pub(super) fn overview() {
    paragraph("ui-icon-buttons-represent-common-actions-with-a-recognizable-icon-use-them-when-the-action-is".into());
    gap(Dp(16.0));
    paragraph("ui-choose-a-default-button-for-an-action-and-a-toggle-button-for-a-persistent-selection-use-f".into());
    gap(Dp(32.0));
    super::overview_art();
}

#[tessera]
pub(in crate::pages::material_spec) fn guidance(title: String) {
    match title.as_str() {
        "ui-usage" => {
            paragraph("ui-use-icon-buttons-for-familiar-frequently-used-actions-such-as-favoriting-archiving-or-mark".into());
            gap(Dp(24.0));
            mail_scene();
            gap(Dp(24.0));
            paragraph("ui-the-heart-retains-its-selected-state-archive-moves-this-message-out-of-the-inbox-mark-as-r".into());
        }
        "ui-anatomy" => {
            paragraph("ui-an-icon-button-contains-an-icon-an-optional-visible-container-and-a-larger-interactive-tar".into());
            gap(Dp(24.0));
            super::anatomy();
        }
        "ui-color" => {
            paragraph("ui-use-filled-icon-buttons-for-strong-emphasis-tonal-buttons-for-a-softer-container-outlined".into());
            gap(Dp(24.0));
            super::colors();
        }
        "ui-size" => {
            paragraph("ui-choose-a-size-that-matches-the-importance-and-context-of-the-action-small-controls-suit-de".into());
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(24.0))
                .line_spacing(Dp(24.0))
                .children(move || {
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
                                I18N.format(
                                    "ui-size-value-format-174",
                                    [("v0", I18N.text(&label).into())],
                                ),
                            );
                            gap(Dp(12.0));
                            paragraph(label.into());
                        });
                    }
                });
            gap(Dp(24.0));
            super::context::density();
        }
        "ui-width" => {
            paragraph("ui-width-changes-the-container-s-horizontal-proportions-without-changing-the-icon-size-or-the".into());
            gap(Dp(24.0));
            flow_row()
                .item_spacing(Dp(32.0))
                .line_spacing(Dp(24.0))
                .children(move || {
                    for (width, label) in [
                        (IconButtonWidth::Narrow, "ui-narrow"),
                        (IconButtonWidth::Uniform, "ui-default"),
                        (IconButtonWidth::Wide, "ui-wide"),
                    ] {
                        column().children(move || {
                            demo(
                                IconButtonSize::Medium,
                                width,
                                false,
                                I18N.format("ui-width-value", [("v0", I18N.text(&label).into())]),
                            );
                            gap(Dp(12.0));
                            paragraph(label.into());
                        });
                    }
                });
        }
        "ui-shape" => {
            paragraph("ui-round-and-square-shapes-offer-different-visual-character-pressed-shapes-share-the-same-siz".into());
            gap(Dp(24.0));
            super::shape_morph();
        }
        "ui-placement" => {
            super::context::placement();
        }
        "ui-behavior" => {
            paragraph("ui-default-icon-buttons-perform-an-action-when-activated-toggle-icon-buttons-retain-selection".into());
            gap(Dp(24.0));
            super::hover_preview();
            gap(Dp(24.0));
            spec_table(
                vec!["ui-behavior".into(), "ui-expected-result".into()],
                vec![
                    vec![
                        "ui-default-action".into(),
                        "ui-performs-its-command-without-retaining-a-selected-state".into(),
                    ],
                    vec![
                        "ui-toggle-action".into(),
                        "ui-retains-selection-and-changes-its-icon-or-shape".into(),
                    ],
                    vec![
                        "ui-overflow-action".into(),
                        "ui-opens-related-choices-escape-dismisses-the-menu".into(),
                    ],
                    vec![
                        "ui-disabled-action".into(),
                        "ui-does-not-activate-or-participate-in-keyboard-traversal".into(),
                    ],
                ],
            );
        }
        _ => {
            paragraph("ui-use-recognizable-icons-for-contextual-actions-provide-meaningful-labels-preserve-adequate".into());
        }
    }
}

#[tessera]
pub(in crate::pages::material_spec) fn contrast() {
    paragraph("ui-enabled-icon-buttons-need-at-least-3-1-contrast-between-the-icon-and-its-adjacent-backgrou".into());
    gap(Dp(16.0));
    paragraph("ui-the-examples-use-the-same-white-surface-and-a-live-favorite-action-the-dark-icon-is-above".into());
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
    text(I18N.text(value)).style(type_style(16.0, 24.0));
}

#[tessera]
fn demo(size: IconButtonSize, width: IconButtonWidth, play: bool, label: String) {
    let selected = remember(|| false);
    icon_button()
        .size(size)
        .width(width)
        .variant(IconButtonVariant::FilledTonal)
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
        .accessibility_label(I18N.text(label))
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
            text(I18N.text(if good {
                "ui-above-3-1-do"
            } else {
                "ui-below-3-1-don-t"
            }))
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
                .accessibility_label(I18N.text(if good {
                    "ui-favorite-sufficient-contrast-example"
                } else {
                    "ui-favorite-intentionally-insufficient-contrast-example"
                }))
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
    let anchor = remember(tessera_ui::LayoutCoordinates::default);
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(Modifier::new().fill_max_width().height(Dp(320.0)))
        .children(move || {
            menu_provider()
                .controller(menu)
                .placement(MenuPlacement::BelowEnd)
                .window_anchor(anchor)
                .main_content(move || {
                    column()
                        .modifier(Modifier::new().fill_max_width().padding_all(Dp(24.0)).background_with_shape(scheme.surface_container_low, Shape::rounded_rectangle(Dp(24.0))).semantics(SemanticsArgs { role: Some(Role::Group), label: Some(I18N.text("ui-message-actions")), merge_descendants: true, ..Default::default() }))
                        .children(move || {
                            subheading("ui-weekend-plans".into());
                            gap(Dp(8.0));
                            paragraph(if deleted.get() { "ui-message-moved-to-trash-choose-more-options-move-to-inbox-to-restore-it".into() } else { "ui-alex-let-s-meet-for-a-walk-on-saturday".into() });
                            gap(Dp(16.0));
                            flow_row().item_spacing(Dp(12.0)).line_spacing(Dp(8.0)).children(move || {
                                icon_button()
                                    .variant(IconButtonVariant::Standard)
                                    .shape(if favorite.get() { Shape::rounded_rectangle(Dp(12.0)) } else { Shape::CAPSULE })
                                    .selected(favorite.get())
                                    .icon(if favorite.get() { filled::FAVORITE_SVG } else { filled::FAVORITE_BORDER_SVG })
                                    .accessibility_label(I18N.text(if favorite.get() { "ui-remove-favorite" } else { "ui-add-favorite" }))
                                    .on_click(move || favorite.set(!favorite.get()));
                                icon_button().variant(IconButtonVariant::Standard).shape(Shape::CAPSULE).icon(filled::ARCHIVE_SVG).enabled(!archived.get() && !deleted.get()).accessibility_label(I18N.text("ui-archive-message")).on_click(move || archived.set(true));
                                icon_button().variant(IconButtonVariant::Standard).shape(Shape::CAPSULE).icon(filled::MARK_EMAIL_READ_SVG).selected(read.get()).accessibility_label(I18N.text(if read.get() { "ui-mark-as-unread" } else { "ui-mark-as-read" })).on_click(move || read.set(!read.get()));
                                icon_button()
                                    .variant(IconButtonVariant::Standard)
                                    .shape(Shape::CAPSULE)
                                    .icon(filled::MORE_VERT_SVG)
                                    .accessibility_label(I18N.text("ui-more-message-options"))
                                    .modifier(Modifier::new().on_layout_coordinates(move |coordinates| { anchor.set_if_changed(coordinates); }).semantics(SemanticsArgs { expanded: Some(menu.with(MenuController::is_open)), has_popup: Some(HasPopup::Menu), ..Default::default() }))
                                    .on_click(move || menu.with_mut(MenuController::toggle));
                            });
                            gap(Dp(16.0));
                            paragraph(I18N.format("ui-folder-value-value-value", [("v0", I18N.text(&(if deleted.get() { "ui-trash" } else if archived.get() { "ui-archive" } else { "ui-inbox" })).into()), ("v1", I18N.text(&(if read.get() { "ui-read" } else { "ui-unread" })).into()), ("v2", I18N.text(&(if favorite.get() { "ui-favorite" } else { "ui-not-favorited" })).into())]));
                        });
                })
                .menu_content(move |menu| {
                    menu_item().menu(menu).label(I18N.text("ui-move-to-inbox")).leading_icon(filled::INBOX_SVG).on_click(move || { archived.set(false); deleted.set(false); });
                    menu_item().menu(menu).label(I18N.text("ui-mark-as-unread")).leading_icon(filled::EMAIL_SVG).on_click(move || read.set(false));
                    menu_item().menu(menu).label(I18N.text("ui-delete-message")).leading_icon(filled::DELETE_SVG).on_click(move || deleted.set(true));
                });
        });
}
