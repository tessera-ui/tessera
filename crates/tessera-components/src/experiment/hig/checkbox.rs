use tessera_ui::{
    Callback, CallbackWith, Dp, FocusRequester, Modifier, accesskit::Role, remember, tessera,
};

use super::choice::{ChoiceMark, ChoiceStyle, choice_control};

#[cfg(test)]
#[path = "checkbox_tests.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq)]
struct Action {
    value: bool,
    mixed: bool,
    enabled: bool,
    callback: CallbackWith<bool>,
}

/// The checkbox style of [`toggle`](super::toggle::toggle). Activating a mixed checkbox turns it
/// on.
#[tessera]
pub(super) fn checkbox(
    value: bool,
    mixed: Option<bool>,
    on_value_change: Option<CallbackWith<bool>>,
    #[prop(into)] label: Option<String>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let mixed = mixed.unwrap_or(false);
    let next = Action {
        value,
        mixed,
        enabled,
        callback: on_value_change.unwrap_or_else(CallbackWith::default_value),
    };
    let action = remember(|| next);
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call(action.mixed || !action.value);
        }
    });
    let requester = if let Some(requester) = focus_requester {
        requester
    } else {
        remember(FocusRequester::new).get()
    };
    choice_control(
        ChoiceStyle {
            role: Role::CheckBox,
            mark: if mixed {
                Some(ChoiceMark::Dash)
            } else {
                value.then_some(ChoiceMark::Check)
            },
            label_gap: Dp(5.0),
            label_line_height: Dp(17.0),
        },
        activate,
        label.unwrap_or_default(),
        enabled,
        window_active.unwrap_or(true),
        modifier.unwrap_or_default(),
        requester,
    )
    .accessibility_label_optional(accessibility_label)
    .description_optional(accessibility_description);
}
