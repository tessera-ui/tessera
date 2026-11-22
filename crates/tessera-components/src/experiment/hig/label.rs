//! Labels that pair a title with an icon.
//!
//! ## Usage
//!
//! Name a command, setting, or list row with text and a symbol.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Dp, Modifier, RenderSlot,
    accesskit::Role,
    layout::{Stack, layout},
    tessera, use_context,
};

use crate::{alignment::CrossAxisAlignment, row::row, spacer::spacer};

use super::{text::text, theme::HigTheme};

#[cfg(test)]
#[path = "label_tests.rs"]
mod tests;

/// The appearance of a [`label`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelStyle {
    /// The platform default, the icon followed by the title on macOS.
    #[default]
    Automatic,
    /// The icon followed by the title.
    TitleAndIcon,
    /// The icon alone; the title remains the accessible name.
    IconOnly,
    /// The title alone.
    TitleOnly,
}

/// # label
///
/// Shows a title with an icon, or either one alone.
///
/// ## Usage
///
/// Name commands in toolbars and menus, or rows in lists and settings, with a symbol.
///
/// ## Parameters
///
/// - `title` — visible text and accessible name.
/// - `icon` — the symbol shown before the title.
/// - `style` — which parts to show; defaults to [`LabelStyle::Automatic`].
/// - `modifier` — layout and additional semantics.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{label::label, text::text};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// label("Favorites").icon(|| {
///     text("★");
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn label(
    #[prop(into)] title: String,
    icon: Option<RenderSlot>,
    style: Option<LabelStyle>,
    modifier: Option<Modifier>,
) {
    let style = style.unwrap_or_default();
    let ink = use_context::<HigTheme>()
        .map(|theme| theme.get().label_color())
        .unwrap_or_else(|| HigTheme::default().label_color());
    let show_icon = style != LabelStyle::TitleOnly && icon.is_some();
    let show_title = style != LabelStyle::IconOnly || icon.is_none();
    layout(Stack)
        .modifier(modifier.unwrap_or_default())
        .child(move || {
            label_content(title.clone(), show_icon, show_title, ink).icon_optional(icon)
        });
}

#[tessera]
fn label_content(
    title: String,
    icon: Option<RenderSlot>,
    show_icon: bool,
    show_title: bool,
    ink: tessera_ui::Color,
) {
    row()
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
            role: Some(Role::Label),
            label: Some(title.clone()),
            ..Default::default()
        }))
        .children(move || {
            if show_icon && let Some(icon) = icon {
                icon.render();
            }
            if show_icon && show_title {
                spacer().modifier(Modifier::new().width(Dp(6.0)));
            }
            if show_title {
                text(title.clone()).color(ink).single_line(true);
            }
        });
}
