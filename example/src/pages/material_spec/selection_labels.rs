//! Native adjacent selection label color illustrations.
//!
//! ## Usage
//!
//! Compare setting labels and supporting text beside independent controls.

use tessera_components::{
    alignment::Alignment,
    boxed::boxed,
    column::column,
    md3::{
        app_bar::top_app_bar,
        checkbox::checkbox,
        icon::icon,
        material_icons::filled,
        switch::switch,
        text::text,
        theme::{MaterialTheme, TextStyle},
    },
    modifier::ModifierExt as _,
};
use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, Dp, FocusRequester, LayoutCoordinates, Modifier, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

use crate::res::I18N;

fn style(size: f64, height: f64, weight: u16, tracking: f64) -> TextStyle {
    TextStyle {
        font_size: Dp(size),
        line_height: Some(Dp(height)),
        font_weight: weight,
        letter_spacing: Dp(tracking),
        font_family: None,
    }
}

#[tessera]
pub(super) fn figure(is_switch: bool) {
    let width = remember(|| Dp(676.0));
    boxed()
        .modifier(Modifier::new().fill_max_width().on_layout_coordinates(
            move |coordinates: LayoutCoordinates| {
                let next = Dp::from(coordinates.local_size.width);
                if next.0 > 0.0 && next != width.get() {
                    width.set(next);
                }
            },
        ))
        .children(move || {
            scaled_box((width.get().0 / 676.0) as f32).child(move || {
                canvas(is_switch);
            });
        });
}

#[tessera]
fn canvas(is_switch: bool) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .modifier(
            Modifier::new()
                .required_size(Dp(676.0), Dp(316.0))
                .background(scheme.primary_container)
                .clip_to_bounds()
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(
                        I18N.format(
                            "ui-value-adjacent-label-colors",
                            [(
                                "v0",
                                I18N.text(
                                    &(if is_switch {
                                        "ui-switch"
                                    } else {
                                        "ui-checkbox"
                                    }),
                                )
                                .into(),
                            )],
                        ),
                    ),
                    merge_descendants: true,
                    ..Default::default()
                }),
        )
        .children(move || {
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(124.0), Dp::ZERO)
                        .size(Dp(428.0), Dp(316.0))
                        .background(scheme.outline_variant),
                )
                .children(|| {});
            boxed()
                .modifier(
                    Modifier::new()
                        .offset(Dp(132.0), Dp::ZERO)
                        .size(Dp(412.0), Dp(316.0))
                        .background(scheme.surface)
                        .clip_to_bounds()
                        .semantics(SemanticsArgs {
                            role: Some(Role::Group),
                            label: Some(I18N.text("ui-adjacent-labels-app-viewport")),
                            merge_descendants: true,
                            ..Default::default()
                        }),
                )
                .children(move || {
                    top_app_bar()
                        .modifier(Modifier::new().fill_max_width())
                        .container_color(scheme.surface_container)
                        .elevation(Dp::ZERO)
                        .navigation_icon(|| {
                            boxed()
                                .alignment(Alignment::Center)
                                .modifier(Modifier::new().size(Dp(48.0), Dp(48.0)))
                                .children(|| {
                                    icon().painter(filled::ARROW_BACK_SVG).size(Dp(24.0));
                                });
                        })
                        .title_content(move || {
                            text(I18N.text(if is_switch {
                                "ui-privacy"
                            } else {
                                "ui-meal-options"
                            }))
                            .style(style(22.0, 28.0, 400, 0.0))
                            .color(scheme.on_surface);
                        });
                    if is_switch {
                        for index in 0..3 {
                            setting(index);
                        }
                    } else {
                        boxed()
                            .modifier(Modifier::new().offset(Dp(16.0), Dp(80.0)))
                            .children(move || {
                                text(I18N.text("ui-extras"))
                                    .style(style(14.0, 20.0, 500, 0.1))
                                    .color(scheme.on_surface_variant);
                            });
                        for index in 0..4 {
                            extra(index);
                        }
                    }
                });
        });
}

fn label_modifier(modifier: Modifier, checked: State<bool>, focus: FocusRequester) -> Modifier {
    modifier
        .clickable(ClickableArgs {
            on_click: Callback::new(move || {
                checked.with_mut(|value| *value = !*value);
                focus.request_focus();
            }),
            ..Default::default()
        })
        .semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        })
}

#[tessera]
fn extra(index: usize) {
    let checked = remember(|| index < 2);
    let focus = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let label = ["ui-pickles", "ui-tomato", "ui-lettuce", "ui-cheese"][index];
    let top = Dp(112.0 + index as f64 * 56.0);
    checkbox(checked.get())
        .modifier(Modifier::new().offset(Dp(4.0), top))
        .focus_requester(focus)
        .accessibility_label(I18N.text(label))
        .on_toggle(move |value| checked.set(value));
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(label_modifier(
            Modifier::new()
                .offset(Dp(56.0), top)
                .size(Dp(340.0), Dp(48.0)),
            checked,
            focus,
        ))
        .children(move || {
            text(I18N.text(label))
                .style(style(16.0, 24.0, 400, 0.5))
                .color(scheme.on_surface);
        });
}

#[tessera]
fn setting(index: usize) {
    let checked = remember(|| index < 2);
    let focus = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let label = [
        "ui-permission-manager",
        "ui-camera-access",
        "ui-show-password",
    ][index];
    let supporting = [
        Some("ui-app-has-access-to-your-data"),
        Some("ui-app-has-access-to-your-camera"),
        None,
    ][index];
    let top = Dp([104.0, 176.0, 240.0][index]);
    switch(checked.get())
        .modifier(Modifier::new().offset(Dp(344.0), top))
        .focus_requester(focus)
        .accessibility_label(I18N.text(label))
        .on_toggle(move |value| checked.set(value))
        .child_optional(if checked.get() {
            Some(tessera_ui::RenderSlot::new(|| {
                icon().painter(filled::CHECK_SVG).size(Dp(16.0));
            }))
        } else {
            None
        });
    boxed()
        .alignment(Alignment::CenterStart)
        .modifier(label_modifier(
            Modifier::new()
                .offset(Dp(16.0), top)
                .size(Dp(312.0), Dp(48.0)),
            checked,
            focus,
        ))
        .children(move || {
            column().children(move || {
                text(I18N.text(label))
                    .style(style(16.0, 24.0, 400, 0.5))
                    .color(scheme.on_surface);
                if let Some(supporting) = supporting {
                    text(I18N.text(supporting))
                        .style(style(14.0, 20.0, 400, 0.2))
                        .color(scheme.on_surface_variant);
                }
            });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "selection_labels_tests.rs"]
mod tests;
