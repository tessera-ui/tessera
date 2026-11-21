//! Color pickers with a color well and a palette.
//!
//! ## Usage
//!
//! Choose a highlight, label, or accent color in settings and inspectors.

use tessera_foundation::modifier::{ClickableArgs, ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, CallbackWith, Color, Dp, FloatRect, FocusScopeNode, Modifier, RenderSlot,
    accesskit::Role, modifier::FocusModifierExt as _, remember, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment, boxed::boxed, column::column, modifier::ModifierExt as _,
    row::row, shape_def::Shape, spacer::spacer,
};

use super::{
    menu::{MenuContext, present},
    presentation::PopupPlacement,
    text::text,
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "color_picker_tests.rs"]
mod tests;

fn srgb(r: u8, g: u8, b: u8) -> Color {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::from_rgb(channel(r), channel(g), channel(b))
}

/// The palette: named system colors, then a row of grays.
fn palette() -> [(&'static str, Color); 16] {
    [
        ("Red", srgb(255, 56, 60)),
        ("Orange", srgb(255, 141, 40)),
        ("Yellow", srgb(255, 204, 0)),
        ("Green", srgb(52, 199, 89)),
        ("Mint", srgb(0, 200, 179)),
        ("Teal", srgb(0, 195, 208)),
        ("Blue", srgb(0, 136, 255)),
        ("Purple", srgb(203, 48, 224)),
        ("Pink", srgb(255, 45, 85)),
        ("Brown", srgb(172, 127, 94)),
        ("Black", srgb(0, 0, 0)),
        ("Dark Gray", srgb(64, 64, 64)),
        ("Gray", srgb(128, 128, 128)),
        ("Light Gray", srgb(192, 192, 192)),
        ("Silver", srgb(224, 224, 224)),
        ("White", srgb(255, 255, 255)),
    ]
}

/// # color_picker
///
/// Chooses a color from a palette shown by a color well.
///
/// ## Usage
///
/// Pick highlight, label, or tag colors in settings and inspectors.
///
/// ## Parameters
///
/// - `selection`, `on_change` — the chosen color and requested changes.
/// - `label` — leading title and accessible name.
/// - `enabled` — whether the color can change; defaults to true.
/// - `modifier` — layout and additional semantics.
///
/// The palette appears in the nearest
/// [`presentation_host`](super::presentation::presentation_host).
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::color_picker::color_picker;
/// use tessera_ui::{Color, remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let highlight = remember(|| Color::from_rgb(1.0, 0.8, 0.0));
/// color_picker(highlight.get())
///     .label("Highlight")
///     .on_change(move |color| highlight.set(color));
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn color_picker(
    selection: Color,
    on_change: Option<CallbackWith<Color>>,
    #[prop(into)] label: Option<String>,
    enabled: Option<bool>,
    modifier: Option<Modifier>,
) {
    let enabled = enabled.unwrap_or(true);
    let open = remember(|| false);
    let anchor = remember(|| FloatRect::ZERO);
    let scope = remember(FocusScopeNode::new).get();
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark { Color::WHITE } else { Color::BLACK };
    let dismiss = Callback::new(move || {
        if open.get() {
            open.set(false);
        }
    });
    let choose = CallbackWith::new(move |color: Color| {
        if let Some(on_change) = on_change {
            on_change.call(color);
        }
    });
    present(
        open.get() && enabled,
        anchor.get(),
        PopupPlacement::Below,
        RenderSlot::new(move || palette_panel(selection, choose, tint)),
        dismiss,
        scope,
        0.0,
    );
    let shape = Shape::rounded_rectangle(Dp(5.0));
    let well = Modifier::new()
        .size(Dp(44.0), Dp(23.0))
        .background_with_shape(tint.with_alpha(0.06), shape)
        .border_with_shape(Dp(1.0), tint.with_alpha(0.15), shape)
        .padding_all(Dp(4.0))
        .on_globally_positioned_precise(move |rect| {
            if anchor.get() != rect {
                anchor.set(rect);
            }
        })
        .clickable(ClickableArgs {
            on_click: Callback::new(move || open.set(!open.get())),
            enabled,
            role: Some(Role::ColorWell),
            label: label.clone(),
            ..Default::default()
        });
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(modifier.unwrap_or_default())
        .children(move || {
            if let Some(label) = label.clone() {
                text(label).color(tint.with_alpha(0.85)).modifier(
                    Modifier::new().clear_and_set_semantics(SemanticsArgs {
                        hidden: true,
                        ..Default::default()
                    }),
                );
                spacer().modifier(Modifier::new().width(Dp(8.0)));
            }
            boxed().modifier(well.clone()).children(move || {
                spacer().modifier(
                    Modifier::new()
                        .fill_max_size()
                        .background_with_shape(selection, Shape::rounded_rectangle(Dp(2.0))),
                );
            });
        });
}

#[tessera]
fn palette_panel(selection: Color, choose: CallbackWith<Color>, tint: Color) {
    let dismiss = use_context::<MenuContext>().map(|context| context.get().dismiss);
    column()
        .modifier(Modifier::new().padding_symmetric(Dp(8.0), Dp(3.0)))
        .children(move || {
            for row_colors in palette().chunks(8) {
                let row_colors = row_colors.to_vec();
                row().children(move || {
                    for (name, color) in row_colors.iter().copied() {
                        swatch(name.to_string(), color, color == selection, tint).on_choose(
                            move || {
                                choose.call(color);
                                if let Some(dismiss) = dismiss {
                                    dismiss.call();
                                }
                            },
                        );
                    }
                });
            }
        });
}

#[tessera]
fn swatch(name: String, color: Color, selected: bool, tint: Color, on_choose: Option<Callback>) {
    let shape = Shape::rounded_rectangle(Dp(4.0));
    let mut modifier = Modifier::new()
        .size(Dp(24.0), Dp(24.0))
        .padding_all(Dp(2.0))
        .background_with_shape(color, shape)
        .border_with_shape(
            Dp(if selected { 2.0 } else { 1.0 }),
            if selected {
                Color::from_rgb(0.0, 0.19461783, 1.0)
            } else {
                tint.with_alpha(0.12)
            },
            shape,
        );
    if let Some(on_choose) = on_choose {
        modifier = modifier.clickable(ClickableArgs {
            on_click: on_choose,
            role: Some(Role::Button),
            label: Some(name),
            ..Default::default()
        });
    }
    boxed().modifier(modifier).children(|| {});
}
