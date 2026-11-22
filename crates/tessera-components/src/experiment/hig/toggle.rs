//! Toggles for settings that are on or off, as checkboxes, switches, or buttons.
//!
//! ## Usage
//!
//! Turn options and settings on or off, or show a partially applied group setting.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{CallbackWith, Dp, FocusRequester, Modifier, tessera};

use crate::{alignment::CrossAxisAlignment, row::row, spacer::spacer};

use super::{
    button::push_button,
    checkbox::checkbox,
    switch::switch,
    text::text,
    theme::{HigTheme, TextStyle},
};

#[cfg(test)]
#[path = "toggle_tests.rs"]
mod tests;

/// The appearance of a [`toggle`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleStyle {
    /// The platform default, a checkbox on macOS.
    #[default]
    Automatic,
    /// A square box with a checkmark, followed by the label.
    Checkbox,
    /// The label followed by a sliding switch.
    Switch,
    /// A push button that stays highlighted while on.
    Button,
}

/// # toggle
///
/// Switches a setting on or off as a checkbox, switch, or button.
///
/// ## Usage
///
/// Enable options in forms and settings, or represent a group whose members disagree.
///
/// ## Parameters
///
/// - `is_on`, `on_value_change` — controlled value and requested updates.
/// - `label` — visible title and default accessible name.
/// - `style` — checkbox, switch, or button appearance; defaults to [`ToggleStyle::Automatic`].
/// - `mixed` — shows a partially applied group setting; activating it requests on. Button
///   toggles show a mixed value as off.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — containing window's active appearance; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::toggle::{ToggleStyle, toggle};
/// use tessera_ui::{remember, tessera};
/// # #[tessera]
/// # fn component() {
///
/// let notifications = remember(|| true);
/// toggle(notifications.get())
///     .label("Notifications")
///     .on_value_change(move |on| notifications.set(on));
/// toggle(notifications.get())
///     .label("Show previews")
///     .style(ToggleStyle::Switch)
///     .enabled(notifications.get());
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn toggle(
    is_on: bool,
    on_value_change: Option<CallbackWith<bool>>,
    #[prop(into)] label: Option<String>,
    style: Option<ToggleStyle>,
    mixed: Option<bool>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let mixed = mixed.unwrap_or(false);
    let label = label.unwrap_or_default();
    match style.unwrap_or_default() {
        ToggleStyle::Automatic | ToggleStyle::Checkbox => {
            checkbox(is_on)
                .mixed(mixed)
                .on_value_change_optional(on_value_change)
                .label(label)
                .enabled_optional(enabled)
                .window_active_optional(window_active)
                .modifier_optional(modifier)
                .focus_requester_optional(focus_requester)
                .accessibility_label_optional(accessibility_label)
                .accessibility_description_optional(accessibility_description);
        }
        ToggleStyle::Switch => {
            let spoken = accessibility_label.or_else(|| (!label.is_empty()).then(|| label.clone()));
            row()
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .modifier(modifier.unwrap_or_default())
                .children(move || {
                    if !label.is_empty() {
                        switch_label(label.clone());
                        spacer().modifier(Modifier::new().width(Dp(8.0)));
                    }
                    switch(is_on)
                        .mixed(mixed)
                        .on_value_change_optional(on_value_change)
                        .enabled_optional(enabled)
                        .window_active_optional(window_active)
                        .focus_requester_optional(focus_requester)
                        .accessibility_label_optional(spoken.clone())
                        .accessibility_description_optional(accessibility_description.clone());
                });
        }
        ToggleStyle::Button => {
            let on = is_on && !mixed;
            push_button(label)
                .toggled(on)
                .on_click(move || {
                    if let Some(callback) = on_value_change {
                        callback.call(!on);
                    }
                })
                .enabled_optional(enabled)
                .window_active_optional(window_active)
                .modifier_optional(modifier)
                .focus_requester_optional(focus_requester)
                .accessibility_label_optional(accessibility_label)
                .accessibility_description_optional(accessibility_description);
        }
    }
}

#[tessera]
fn switch_label(content: String) {
    let theme = tessera_ui::use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    text(content)
        .style(TextStyle {
            line_height: Dp(17.0),
            ..Default::default()
        })
        .color(theme.label_color())
        .single_line(true)
        .modifier(Modifier::new().clear_and_set_semantics(SemanticsArgs {
            hidden: true,
            ..Default::default()
        }));
}
