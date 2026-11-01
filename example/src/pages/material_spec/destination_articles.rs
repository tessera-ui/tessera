//! Shared markers for navigation reference figures.
//!
//! ## Usage
//!
//! Identify labeled elements in navigation diagrams.

use tessera_components::{
    alignment::Alignment, boxed::boxed, modifier::ModifierExt as _, shape_def::Shape, text::text,
    theme::MaterialTheme,
};
use tessera_ui::{Dp, Modifier, tessera, use_context};

use super::super::type_style;

#[tessera]
pub(crate) fn figure_marker(number: usize, x: Dp, y: Dp, scale: f64) {
    let scheme = use_context::<MaterialTheme>().unwrap().get().color_scheme;
    boxed()
        .alignment(Alignment::Center)
        .modifier(
            Modifier::new()
                .offset(x, y)
                .size(Dp(24.0 * scale), Dp(32.0 * scale))
                .background_with_shape(scheme.surface, Shape::rounded_rectangle(Dp(16.0 * scale)))
                .border_with_shape(
                    Dp(scale),
                    scheme.on_surface,
                    Shape::rounded_rectangle(Dp(16.0 * scale)),
                ),
        )
        .children(move || {
            text()
                .content(number.to_string())
                .style(type_style(16.0 * scale, 20.0 * scale))
                .color(scheme.on_surface);
        });
}
