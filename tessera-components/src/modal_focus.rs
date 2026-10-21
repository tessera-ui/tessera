//! Focus restoration for temporary modal content.
//!
//! ## Usage
//!
//! Return to the invoking control after a modal's exit transition finishes.

use tessera_ui::{
    FocusManager, FocusRestorationTarget, FocusScopeNode, Modifier, State,
    modifier::FocusModifierExt as _,
};

#[derive(Default)]
pub(crate) struct ModalFocusState {
    visible: bool,
    origin: Option<FocusRestorationTarget>,
    pending: Option<FocusRestorationTarget>,
}

pub(crate) fn modal_focus_modifier(
    modifier: Modifier,
    state: State<ModalFocusState>,
    visible: bool,
    scope: FocusScopeNode,
) -> Modifier {
    if state.with(|state| state.visible != visible) {
        let origin = if visible {
            FocusManager::current().save_focus()
        } else {
            None
        };
        let restore = !visible && scope.has_focus();
        state.with_mut(|state| {
            state.visible = visible;
            if visible {
                state.origin = origin;
                state.pending = None;
            } else {
                state.pending = if restore { state.origin.take() } else { None };
                state.origin = None;
            }
        });
        FocusManager::current().defer_keyboard_until_layout();
    }
    if state.with(|state| state.pending.is_some()) {
        modifier.on_globally_positioned(move |_| {
            if let Some(target) = state.with_mut(|state| state.pending.take()) {
                target.restore();
            }
        })
    } else {
        modifier
    }
}

#[cfg(test)]
mod tests;
