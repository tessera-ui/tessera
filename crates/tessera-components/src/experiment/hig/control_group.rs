//! Groups of related controls shown together.
//!
//! ## Usage
//!
//! Show back and forward buttons, or a set of related actions, as one joined control.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Color, Dp, DrawModifierContent, DrawModifierContext, DrawModifierNode, Modifier, RenderSlot,
    accesskit::Role, modifier::ModifierCapabilityExt as _, provide_context, tessera, use_context,
};

use crate::{
    alignment::CrossAxisAlignment, pipelines::perceptual_fill::PerceptualFillCommand, row::row,
};

use super::{
    menu::{MenuStyle, menu},
    theme::{Appearance, HigTheme},
};

#[cfg(test)]
#[path = "control_group_tests.rs"]
mod tests;

/// The appearance of a [`control_group`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlGroupStyle {
    /// The platform default, joined buttons on macOS.
    #[default]
    Automatic,
    /// Joined buttons for navigation, such as back and forward.
    Navigation,
    /// The controls laid out side by side as joined buttons.
    Palette,
    /// A pull-down button that shows the controls as menu items.
    Menu,
    /// A compact pull-down button that shows the controls as menu items.
    CompactMenu,
}

/// Marks buttons drawn inside a [`control_group`].
#[derive(Clone, Copy, PartialEq)]
pub(super) struct ControlGroupContext;

#[derive(Clone, PartialEq)]
struct GroupBackground(Color);

impl DrawModifierNode for GroupBackground {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_draw_command(PerceptualFillCommand {
                outline_width: Dp::ZERO,
                snap_to_pixels: false,
                path: None,
                color: self.0,
                radii: [Dp(6.0); 2],
                insets: [Dp::ZERO; 4],
                clip_radius: None,
                line: None,
                edge_smoothness: 1.0,
                stroke: None,
            });
        content.draw(context.render_input);
    }
}

/// # control_group
///
/// Shows related buttons as one joined control, or as the items of a pull-down menu.
///
/// ## Usage
///
/// Group back and forward navigation, text formatting actions, or other related commands.
///
/// ## Parameters
///
/// - `content` — the group's buttons.
/// - `label` — title of the pull-down button in the menu styles, and the group's accessible
///   name.
/// - `style` — joined buttons or a pull-down menu; defaults to [`ControlGroupStyle::Automatic`].
/// - `modifier` — layout and additional semantics.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{button::button, control_group::control_group};
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// control_group().label("Navigation").content(|| {
///     button("Back").on_click(|| println!("back"));
///     button("Forward").on_click(|| println!("forward"));
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn control_group(
    content: Option<RenderSlot>,
    #[prop(into)] label: Option<String>,
    style: Option<ControlGroupStyle>,
    modifier: Option<Modifier>,
) {
    let content = match content {
        Some(slot) => slot,
        None => RenderSlot::new(|| {}),
    };
    if matches!(
        style.unwrap_or_default(),
        ControlGroupStyle::Menu | ControlGroupStyle::CompactMenu
    ) {
        menu(label.unwrap_or_default())
            .style(if style == Some(ControlGroupStyle::CompactMenu) {
                MenuStyle::BorderlessButton
            } else {
                MenuStyle::Automatic
            })
            .modifier_optional(modifier)
            .content_shared(content);
        return;
    }
    let dark = use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == Appearance::Dark)
        .unwrap_or(false);
    let fill = if dark {
        Color::WHITE.with_alpha(0.08)
    } else {
        Color::BLACK.with_alpha(0.075)
    };
    let modifier = modifier
        .unwrap_or_default()
        .push_draw(GroupBackground(fill))
        .clip_to_bounds()
        .semantics(SemanticsArgs {
            role: Some(Role::Group),
            label,
            ..Default::default()
        });
    provide_context(ControlGroupContext, move || {
        row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .modifier(modifier.clone())
            .children_shared(content);
    });
}
