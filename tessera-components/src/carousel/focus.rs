//! Focus navigation through interactive carousel items.
//!
//! ## Usage
//!
//! Keep keyboard activation on the item selected by carousel navigation.

use std::collections::BTreeMap;

use parking_lot::Mutex;
use tessera_ui::{FocusManager, FocusRequester, FocusScopeNode, State, winit::keyboard::NamedKey};

use super::CarouselController;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct Navigation {
    pub(super) viewport: FocusRequester,
    pub(super) items: State<Mutex<BTreeMap<usize, FocusScopeNode>>>,
    pub(super) pending: State<Option<usize>>,
}

impl Navigation {
    pub(super) fn retained_item(self) -> Option<usize> {
        self.pending.get().or_else(|| {
            self.items.with(|items| {
                items
                    .lock()
                    .iter()
                    .find_map(|(&index, scope)| scope.has_focus().then_some(index))
            })
        })
    }

    pub(super) fn navigate(
        self,
        controller: State<CarouselController>,
        key: NamedKey,
        vertical: bool,
        reverse: bool,
        shift: bool,
    ) -> bool {
        let Some(count) = controller.with(|state| state.config.map(|config| config.count)) else {
            return false;
        };
        if count == 0 {
            return false;
        }
        let focused = self.items.with(|items| {
            items
                .lock()
                .iter()
                .find_map(|(&index, scope)| scope.has_focus().then_some(index))
        });
        let current = self
            .pending
            .get()
            .or(focused)
            .unwrap_or_else(|| controller.with(CarouselController::current_item));
        let forward = if vertical {
            NamedKey::ArrowDown
        } else {
            NamedKey::ArrowRight
        };
        let backward = if vertical {
            NamedKey::ArrowUp
        } else {
            NamedKey::ArrowLeft
        };
        let target = if key == NamedKey::Tab {
            let Some(current) = focused else {
                return false;
            };
            if shift {
                if current == 0 {
                    return false;
                }
                current - 1
            } else {
                if current + 1 >= count {
                    return false;
                }
                current + 1
            }
        } else if key == NamedKey::Home {
            0
        } else if key == NamedKey::End {
            count - 1
        } else if (key == forward && !reverse) || (key == backward && reverse) {
            current.saturating_add(1).min(count - 1)
        } else if (key == backward && !reverse) || (key == forward && reverse) {
            current.saturating_sub(1)
        } else {
            return false;
        };
        FocusManager::current().defer_keyboard_until_layout();
        self.pending.set(Some(target));
        if !self.items.with(|items| items.lock().contains_key(&target)) {
            self.viewport.request_focus();
        }
        if controller.with(|state| state.navigation_required(target)) {
            controller.with_mut(|state| state.animate_to_item(target));
        }
        self.finish(controller.with(|state| !state.needs_frame()));
        true
    }

    pub(super) fn finish(self, settled: bool) -> bool {
        let Some(target) = self.pending.get() else {
            return true;
        };
        if !self.viewport.has_focus() {
            self.pending.set(None);
            return true;
        }
        let scope = self.items.with(|items| items.lock().get(&target).copied());
        if let Some(scope) = scope {
            scope.request_focus();
            if scope.has_focus() || settled {
                self.pending.set(None);
                return true;
            }
        }
        false
    }
}

pub(super) struct ItemLease {
    pub(super) navigation: Navigation,
    pub(super) index: usize,
}

impl Drop for ItemLease {
    fn drop(&mut self) {
        if self.navigation.items.is_alive() {
            self.navigation.items.with(|items| {
                items.lock().remove(&self.index);
            });
        }
    }
}
