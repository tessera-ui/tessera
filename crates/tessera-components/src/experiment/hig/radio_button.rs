//! Single-choice radio buttons and keyboard navigation groups.
//!
//! ## Usage
//!
//! Choose one mutually exclusive setting, mode, or form option.

use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
use tessera_ui::{
    Callback, Dp, FocusDirection, FocusRequester, FocusState, FocusTraversalPolicy, Modifier,
    RenderSlotWith,
    accesskit::Role,
    focus,
    layout::{Stack, layout},
    modifier::FocusModifierExt as _,
    remember, tessera,
};

use super::choice::{ChoiceMark, ChoiceStyle, choice_control};

#[cfg(test)]
#[path = "radio_button_tests.rs"]
mod tests;

/// Enables group navigation for choices inside [`radio_group`].
#[derive(Clone, Copy, PartialEq)]
pub struct RadioGroupScope(());

/// # radio_group
///
/// Provides a single Tab stop and directional navigation for mutually exclusive choices.
///
/// ## Usage
///
/// Group related settings; derive their selection from one application-owned value.
///
/// ## Parameters
///
/// - `child` — choices receiving the explicit group scope; arrange them in a row or column.
/// - `modifier` — group layout and semantics.
/// - `accessibility_label`, `accessibility_description` — spoken group name and instructions.
///
/// ## Examples
///
/// ```
/// use tessera_components::{column::column, experiment::hig::radio_button::{radio_button, radio_group}};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, remember, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     let choice = remember(|| 0);
///     radio_group().accessibility_label("Delivery").child(move |group| {
///         column().children(move || {
///             radio_button(choice.get() == 0).label("Standard").group(group)
///                 .on_select(move || choice.set(0));
///             radio_button(choice.get() == 1).label("Express").group(group)
///                 .on_select(move || choice.set(1))
///                 .modifier(Modifier::new().semantics(SemanticsArgs {
///                     test_tag: Some("express".into()), ..Default::default()
///                 }));
///         });
///     });
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// ui.node("express").click();
/// assert_eq!(ui.accessibility_node("Express").1.toggled(), Some(tessera_ui::accesskit::Toggled::True));
/// ```
#[tessera]
pub fn radio_group(
    child: Option<RenderSlotWith<RadioGroupScope>>,
    modifier: Option<Modifier>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let modifier = modifier
        .unwrap_or_default()
        .focus_group(None)
        .focus_traversal_policy(
            FocusTraversalPolicy::linear()
                .wrap(true)
                .single_tab_stop(true),
        )
        .semantics(SemanticsArgs {
            role: Some(Role::RadioGroup),
            label: accessibility_label,
            description: accessibility_description,
            ..Default::default()
        });
    layout(Stack).modifier(modifier).child(move || {
        if let Some(child) = child {
            child.render(RadioGroupScope(()));
        }
    });
}

#[derive(Clone, Copy, PartialEq)]
struct Action {
    enabled: bool,
    callback: Callback,
}

/// # radio_button
///
/// Requests selection of a single labeled choice without deselecting it on repeated activation.
///
/// ## Usage
///
/// Offer mutually exclusive settings and form choices.
///
/// ## Parameters
///
/// - `selected` — controlled checked state; share a single selection value across related choices.
/// - `on_select` — called on each completed activation, including an already selected choice.
/// - `label` — optional visible title and default accessible name.
/// - `group` — scope from the enclosing group; arrows select enabled choices and Tab enters at the selected choice.
/// - `enabled`, `window_active` — input availability and containing window appearance; both default to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::radio_button::radio_button;
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, remember, tessera, testing::UiTest};
/// #[tessera]
/// fn app() {
///     let selected = remember(|| false);
///     radio_button(selected.get()).label("Use system settings")
///         .on_select(move || selected.set(true))
///         .modifier(Modifier::new().semantics(SemanticsArgs {
///             test_tag: Some("system".into()), ..Default::default()
///         }));
/// }
/// let mut ui = UiTest::new().content(app);
/// ui.frame();
/// ui.node("system").click();
/// assert_eq!(ui.accessibility_node("Use system settings").1.toggled(), Some(tessera_ui::accesskit::Toggled::True));
/// ```
#[tessera]
pub fn radio_button(
    selected: bool,
    on_select: Option<Callback>,
    #[prop(into)] label: Option<String>,
    group: Option<RadioGroupScope>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let next = Action {
        enabled,
        callback: on_select.unwrap_or_else(Callback::noop),
    };
    let action = remember(|| next);
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call();
        }
    });
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    let mut focus_modifier = Modifier::new();
    if enabled && group.is_some() {
        focus_modifier = focus_modifier
            .preferred_tab_stop(selected)
            .on_focus_changed(move |state: FocusState| {
                if state.is_focused()
                    && matches!(
                        focus::focus_change_direction(),
                        Some(
                            FocusDirection::Left
                                | FocusDirection::Right
                                | FocusDirection::Up
                                | FocusDirection::Down
                        )
                    )
                {
                    activate.call();
                }
            });
    }
    choice_control(
        ChoiceStyle {
            role: Role::RadioButton,
            mark: selected.then_some(ChoiceMark::Dot),
            label_gap: Dp(6.0),
            label_line_height: Dp(16.0),
        },
        activate,
        label.unwrap_or_default(),
        enabled,
        window_active.unwrap_or(true),
        modifier.unwrap_or_default(),
        requester,
    )
    .focus_modifier(focus_modifier)
    .accessibility_label_optional(accessibility_label)
    .description_optional(accessibility_description);
}
