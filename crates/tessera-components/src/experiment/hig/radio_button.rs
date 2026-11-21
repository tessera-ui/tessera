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

/// Height of one radio row; three rows with [`RADIO_ROW_SPACING`] span 61pt.
pub(super) const RADIO_ROW_HEIGHT: Dp = Dp(49.0 / 3.0);
/// Space between radio rows.
pub(super) const RADIO_ROW_SPACING: Dp = Dp(6.0);

#[cfg(test)]
#[path = "radio_button_tests.rs"]
mod tests;

/// Enables group navigation for choices inside [`radio_group`].
#[derive(Clone, Copy, PartialEq)]
pub(super) struct RadioGroupScope(());

/// The single-Tab-stop container behind the radio group style of [`picker`](super::picker::picker).
#[tessera]
pub(super) fn radio_group(
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

/// One choice of the radio group style of [`picker`](super::picker::picker).
#[tessera]
pub(super) fn radio_button(
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
            label_gap: Dp(5.0),
            label_line_height: RADIO_ROW_HEIGHT,
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
