//! Focus navigation for horizontal tab strips.
//!
//! ## Usage
//!
//! Navigate related views with arrows and activate them with Enter or Space.

use std::{any::TypeId, sync::Arc};

use tessera_ui::{
    AccessibilityNode, Callback, FocusManager, FocusRequester, KeyboardInput,
    KeyboardInputModifierNode, ParentDataMap, ParentDataModifierNode, SemanticsModifierNode, State,
    winit::{
        event::ElementState,
        keyboard::{Key, NamedKey},
    },
};

use super::{TabActivationMode, TabsController};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct TabFocusData {
    pub requester: FocusRequester,
    pub selected: bool,
    pub enabled: bool,
    pub activate: Callback,
}

impl ParentDataModifierNode for TabFocusData {
    fn apply_parent_data(&self, data: &mut ParentDataMap) {
        data.insert(TypeId::of::<Self>(), Arc::new(*self));
    }
}

pub(super) struct TabListSemantics;

impl SemanticsModifierNode for TabListSemantics {
    fn apply(
        &self,
        node: &mut AccessibilityNode,
        _: &mut Option<tessera_ui::AccessibilityActionHandler>,
    ) {
        node.role = Some(tessera_ui::accesskit::Role::TabList);
        node.merge_descendants = true;
    }
}

#[derive(Clone, Copy)]
pub(super) struct TabNavigation {
    pub controller: State<TabsController>,
    pub requester: FocusRequester,
}

impl KeyboardInputModifierNode for TabNavigation {
    fn on_keyboard_input(&self, mut input: KeyboardInput<'_>) {
        if input.key_modifiers.control_key()
            || input.key_modifiers.alt_key()
            || input.key_modifiers.super_key()
            || input.key_modifiers.shift_key()
        {
            return;
        }
        for event in input.keyboard_events.iter() {
            if event.state != ElementState::Pressed {
                continue;
            }
            let Key::Named(key) = event.logical_key else {
                continue;
            };
            let target = self
                .controller
                .with(|controller| navigation_target(&controller.targets, self.requester, key));
            let Some(target) = target else {
                continue;
            };
            target.requester.request_focus();
            if self
                .controller
                .with(|controller| controller.activation_mode == TabActivationMode::Automatic)
                && !target.selected
            {
                target.activate.call();
            }
            FocusManager::current().defer_keyboard_until_layout();
            input.block_keyboard();
            return;
        }
    }
}

fn navigation_target(
    targets: &[TabFocusData],
    current: FocusRequester,
    key: NamedKey,
) -> Option<TabFocusData> {
    match key {
        NamedKey::Home => return targets.iter().find(|target| target.enabled).copied(),
        NamedKey::End => return targets.iter().rev().find(|target| target.enabled).copied(),
        NamedKey::ArrowLeft | NamedKey::ArrowRight => {}
        _ => return None,
    }
    let Some(index) = targets
        .iter()
        .position(|target| target.requester == current)
    else {
        return if key == NamedKey::ArrowRight {
            targets.iter().find(|target| target.enabled).copied()
        } else {
            targets.iter().rev().find(|target| target.enabled).copied()
        };
    };
    for step in 1..=targets.len() {
        let candidate = if key == NamedKey::ArrowRight {
            (index + step) % targets.len()
        } else {
            (index + targets.len() - step) % targets.len()
        };
        if targets[candidate].enabled {
            return Some(targets[candidate]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use tessera_ui::{Callback, FocusRequester, winit::keyboard::NamedKey};

    use super::{TabFocusData, navigation_target};

    #[test]
    fn arrows_wrap_skip_disabled_tabs_and_home_end_use_enabled_edges() {
        let requesters = [
            FocusRequester::new(),
            FocusRequester::new(),
            FocusRequester::new(),
        ];
        let mut targets = requesters.map(|requester| TabFocusData {
            requester,
            selected: false,
            enabled: true,
            activate: Callback::noop(),
        });
        targets[1].enabled = false;
        for (from, key, to) in [
            (0, NamedKey::ArrowRight, 2),
            (2, NamedKey::ArrowRight, 0),
            (0, NamedKey::ArrowLeft, 2),
            (2, NamedKey::ArrowLeft, 0),
            (2, NamedKey::Home, 0),
            (0, NamedKey::End, 2),
        ] {
            assert_eq!(
                navigation_target(&targets, requesters[from], key)
                    .expect("target")
                    .requester,
                requesters[to]
            );
        }
        assert!(navigation_target(&targets, requesters[0], NamedKey::ArrowDown).is_none());
        targets[0].enabled = false;
        assert_eq!(
            navigation_target(&targets, requesters[2], NamedKey::Home)
                .expect("enabled edge")
                .requester,
            requesters[2]
        );
        targets[2].enabled = false;
        assert!(navigation_target(&targets, requesters[0], NamedKey::ArrowRight).is_none());
    }
}
