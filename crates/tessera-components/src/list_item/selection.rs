//! Controlled selection behavior for list rows.
//!
//! ## Usage
//!
//! Present action rows, exclusive choices, and independently checked items.

use tessera_ui::{
    AccessibilityActionHandler, AccessibilityNode, Callback, CallbackWith, SemanticsModifierNode,
    accesskit::{Role, Toggled},
};

/// Selection behavior and accessibility role of a list row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListItemSelectionMode {
    /// Activate a general list action through `on_click`.
    #[default]
    Action,
    /// Request selection of a radio-style item. The owner manages exclusivity.
    Single,
    /// Request toggling of an independently checked item.
    Multiple,
}

impl ListItemSelectionMode {
    pub(super) fn role(self) -> Role {
        match self {
            Self::Action => Role::ListItem,
            Self::Single => Role::RadioButton,
            Self::Multiple => Role::CheckBox,
        }
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct ListItemActivation {
    pub mode: ListItemSelectionMode,
    pub selected: bool,
    pub on_click: Option<Callback>,
    pub on_selection_change: Option<CallbackWith<bool>>,
}

impl ListItemActivation {
    pub fn has_handler(&self) -> bool {
        self.on_click.is_some()
            || (self.mode != ListItemSelectionMode::Action && self.on_selection_change.is_some())
    }

    pub fn activate(self) {
        if self.mode != ListItemSelectionMode::Action
            && let Some(callback) = self.on_selection_change
        {
            callback.call(self.mode == ListItemSelectionMode::Single || !self.selected);
        } else if let Some(callback) = self.on_click {
            callback.call();
        }
    }
}

#[derive(Clone)]
pub(super) struct ListItemSelectionSemantics {
    pub mode: ListItemSelectionMode,
    pub selected: Option<bool>,
}

impl SemanticsModifierNode for ListItemSelectionSemantics {
    fn apply(&self, node: &mut AccessibilityNode, _: &mut Option<AccessibilityActionHandler>) {
        let selected = self.selected.unwrap_or(false);
        match self.mode {
            ListItemSelectionMode::Action => {
                if self.selected.is_some() {
                    node.selected = self.selected;
                }
                node.toggled = None;
            }
            ListItemSelectionMode::Single => {
                node.selected = Some(selected);
                node.toggled = Some(if selected {
                    Toggled::True
                } else {
                    Toggled::False
                });
            }
            ListItemSelectionMode::Multiple => {
                node.selected = None;
                node.toggled = Some(if selected {
                    Toggled::True
                } else {
                    Toggled::False
                });
            }
        }
    }
}
