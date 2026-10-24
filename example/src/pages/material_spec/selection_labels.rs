//! Native adjacent selection label color illustrations.
//!
//! ## Usage
//!
//! Compare setting labels and supporting text beside independent controls.

use tessera_components::{
    alignment::Alignment,
    app_bar::top_app_bar,
    boxed::boxed,
    checkbox::checkbox,
    column::column,
    icon::icon,
    material_icons::filled,
    modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs},
    switch::switch,
    text::text,
    theme::{MaterialTheme, TextStyle},
};
use tessera_ui::{
    Callback, Dp, FocusRequester, LayoutCoordinates, Modifier, State, accesskit::Role,
    modifier::FocusModifierExt as _, remember, scaled_box, tessera, use_context,
};

fn style(size: f64, height: f64, weight: u16, tracking: f64) -> TextStyle {
    TextStyle {
        font_size: Dp(size),
        line_height: Some(Dp(height)),
        font_weight: weight,
        letter_spacing: Dp(tracking),
        font_family: Some("Roboto"),
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
                    label: Some(format!(
                        "{} adjacent label colors",
                        if is_switch { "Switch" } else { "Checkbox" }
                    )),
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
                            label: Some("Adjacent labels app viewport".into()),
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
                            text()
                                .content(if is_switch { "Privacy" } else { "Meal options" })
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
                                text()
                                    .content("Extras")
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
        .clickable_with(ClickableArgs {
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
    let label = ["Pickles", "Tomato", "Lettuce", "Cheese"][index];
    let top = Dp(112.0 + index as f64 * 56.0);
    checkbox()
        .modifier(Modifier::new().offset(Dp(4.0), top))
        .checked(checked.get())
        .focus_requester(focus)
        .accessibility_label(label)
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
            text()
                .content(label)
                .style(style(16.0, 24.0, 400, 0.5))
                .color(scheme.on_surface);
        });
}

#[tessera]
fn setting(index: usize) {
    let checked = remember(|| index < 2);
    let focus = remember(FocusRequester::new).get();
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    let label = ["Permission manager", "Camera access", "Show password"][index];
    let supporting = [
        Some("App has access to your data"),
        Some("App has access to your camera"),
        None,
    ][index];
    let top = Dp([104.0, 176.0, 240.0][index]);
    switch()
        .modifier(Modifier::new().offset(Dp(344.0), top))
        .checked(checked.get())
        .focus_requester(focus)
        .accessibility_label(label)
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
                text()
                    .content(label)
                    .style(style(16.0, 24.0, 400, 0.5))
                    .color(scheme.on_surface);
                if let Some(supporting) = supporting {
                    text()
                        .content(supporting)
                        .style(style(14.0, 20.0, 400, 0.2))
                        .color(scheme.on_surface_variant);
                }
            });
        });
}

#[cfg(all(test, not(target_family = "wasm")))]
#[path = "selection_labels_tests.rs"]
mod tests;
