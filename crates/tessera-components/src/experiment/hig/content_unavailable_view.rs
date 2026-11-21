//! Views that explain why content is unavailable.
//!
//! ## Usage
//!
//! Show an empty search result, an empty mailbox, or a missing connection.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{Dp, Modifier, RenderSlot, accesskit::Role, tessera, use_context};

use crate::{
    alignment::{CrossAxisAlignment, MainAxisAlignment},
    column::column,
    spacer::spacer,
};

use super::{
    text::text,
    theme::{Appearance, HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "content_unavailable_view_tests.rs"]
mod tests;

/// # content_unavailable_view
///
/// Explains that content is unavailable, with an optional symbol, description, and actions.
///
/// ## Usage
///
/// Show an empty search result, an empty list, or a failed connection in place of content.
///
/// ## Parameters
///
/// - `title` — short statement of what is missing.
/// - `icon` — optional symbol above the title.
/// - `description` — optional explanation or suggestion below the title.
/// - `actions` — optional buttons below the description.
/// - `modifier` — size and placement; the view centres itself in the space offered.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     button::button,
///     content_unavailable_view::content_unavailable_view,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// content_unavailable_view("No Results")
///     .description("Check the spelling or try a new search.")
///     .actions(|| {
///         button("Clear Search");
///     });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn content_unavailable_view(
    #[prop(into)] title: String,
    icon: Option<RenderSlot>,
    #[prop(into)] description: Option<String>,
    actions: Option<RenderSlot>,
    modifier: Option<Modifier>,
) {
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let tint = if dark {
        tessera_ui::Color::WHITE
    } else {
        tessera_ui::Color::BLACK
    };
    column()
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(
            modifier
                .unwrap_or_default()
                .fill_max_size()
                .semantics(SemanticsArgs {
                    role: Some(Role::Group),
                    label: Some(title.clone()),
                    description: description.clone(),
                    ..Default::default()
                }),
        )
        .children(move || {
            if let Some(icon) = icon {
                icon.render();
                spacer().modifier(Modifier::new().height(Dp(12.0)));
            }
            text(title.clone())
                .style(TextStyle {
                    font_size: Dp(22.0),
                    line_height: Dp(28.0),
                    font_weight: 700,
                    ..Default::default()
                })
                .color(tint.with_alpha(0.85));
            if let Some(description) = description.clone() {
                spacer().modifier(Modifier::new().height(Dp(6.0)));
                text(description).color(tint.with_alpha(0.5));
            }
            if let Some(actions) = actions {
                spacer().modifier(Modifier::new().height(Dp(16.0)));
                actions.render();
            }
        });
}
