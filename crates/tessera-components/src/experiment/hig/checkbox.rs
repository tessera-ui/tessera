//! Checkboxes with optional mixed-state selection.
//!
//! ## Usage
//!
//! Enable settings independently or represent a partially selected group.

use tessera_ui::{
    Callback, CallbackWith, FocusRequester, Modifier, accesskit::Role, remember, tessera,
};

use super::choice::{ChoiceMark, ChoiceStyle, choice_control};

#[cfg(test)]
#[path = "checkbox_tests.rs"]
mod tests;

/// The selection represented by a checkbox.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxState {
    /// No selection.
    #[default]
    Off,
    /// Complete selection.
    On,
    /// Partial selection; requires `allows_mixed`.
    Mixed,
}

impl CheckboxState {
    fn normalized(self, allows_mixed: bool) -> Self {
        if self == Self::Mixed && !allows_mixed {
            Self::On
        } else {
            self
        }
    }

    fn next(self, allows_mixed: bool) -> Self {
        match self.normalized(allows_mixed) {
            Self::Off if allows_mixed => Self::Mixed,
            Self::Off | Self::Mixed => Self::On,
            Self::On => Self::Off,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Action {
    value: CheckboxState,
    allows_mixed: bool,
    enabled: bool,
    callback: CallbackWith<CheckboxState>,
}

/// # checkbox
///
/// Reports selection changes from a labeled checkbox in settings, forms, and selection groups.
///
/// ## Usage
///
/// Let users enable independent options or cycle through partial group selection.
///
/// ## Parameters
///
/// - `value`, `on_value_change` — controlled selection and requested updates.
/// - `label` — optional visible title and default accessible name.
/// - `allows_mixed` — cycles Off → Mixed → On → Off; defaults to false, treating Mixed as On.
/// - `enabled` — input and focus availability; defaults to true.
/// - `window_active` — containing window's active appearance; defaults to true.
/// - `modifier`, `focus_requester` — layout, semantics, and optional external focus handle.
/// - `accessibility_label`, `accessibility_description` — spoken content overrides.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::checkbox::{CheckboxState, checkbox};
/// use tessera_foundation::modifier::{ModifierExt as _, SemanticsArgs};
/// use tessera_ui::{Modifier, State, remember, tessera, testing::{TestProbe, UiTest}};
/// #[tessera]
/// fn app(probe: TestProbe<Option<State<CheckboxState>>>) {
///     let selected = remember(|| CheckboxState::Off);
///     probe.set(Some(selected));
///     checkbox(selected.get()).label("Notifications")
///         .on_value_change(move |next| selected.set(next))
///         .modifier(Modifier::new().semantics(SemanticsArgs {
///             test_tag: Some("notifications".into()), ..Default::default()
///         }));
/// }
/// let probe = TestProbe::default();
/// let mut ui = UiTest::new().content(|| app(probe.clone()));
/// ui.frame();
/// ui.node("notifications").click();
/// assert_eq!(probe.get().unwrap().get(), CheckboxState::On);
/// ```
#[tessera]
pub fn checkbox(
    value: CheckboxState,
    on_value_change: Option<CallbackWith<CheckboxState>>,
    #[prop(into)] label: Option<String>,
    allows_mixed: Option<bool>,
    enabled: Option<bool>,
    window_active: Option<bool>,
    modifier: Option<Modifier>,
    focus_requester: Option<FocusRequester>,
    #[prop(into)] accessibility_label: Option<String>,
    #[prop(into)] accessibility_description: Option<String>,
) {
    let enabled = enabled.unwrap_or(true);
    let allows_mixed = allows_mixed.unwrap_or(false);
    let value = value.normalized(allows_mixed);
    let callback = on_value_change.unwrap_or_else(CallbackWith::default_value);
    let next = Action {
        value,
        allows_mixed,
        enabled,
        callback,
    };
    let action = remember(|| next);
    if action.get() != next {
        action.set(next);
    }
    let activate = Callback::new(move || {
        let action = action.get();
        if action.enabled {
            action.callback.call(action.value.next(action.allows_mixed));
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
            mark: match value {
                CheckboxState::Off => None,
                CheckboxState::On => Some(ChoiceMark::Check),
                CheckboxState::Mixed => Some(ChoiceMark::Dash),
            },
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
