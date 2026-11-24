use tessera_ui::{
    Callback, CursorEventContent, FloatRect, FocusScopeNode, PointerEventPass, PointerInput,
    PointerInputModifierNode, PxPosition, State, focus,
};

use super::MenuController;

#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuCascade {
    pub root: State<MenuController>,
    pub entries: State<Vec<State<MenuBranch>>>,
    pub root_scope: FocusScopeNode,
    pub restore_parent: State<Option<FocusScopeNode>>,
    pub bounds: State<Option<FloatRect>>,
    pub on_dismiss: Option<Callback>,
    pub close_on_escape: bool,
}

impl MenuCascade {
    pub fn has_descendants(&self, parent: State<MenuController>) -> bool {
        self.entries.with(|entries| {
            entries
                .iter()
                .any(|entry| entry.is_alive() && entry.get().parent == parent)
        })
    }

    pub fn descendant_has_focus(&self, parent: State<MenuController>) -> bool {
        self.entries.with(|entries| {
            entries
                .iter()
                .filter(|entry| entry.is_alive())
                .any(|entry| {
                    let branch = entry.get();
                    branch.parent == parent && branch.scope.has_focus()
                })
        })
    }

    pub fn allows_hover(
        &self,
        controller: State<MenuController>,
        point: Option<PxPosition>,
    ) -> bool {
        let Some(point) = point else { return false };
        self.panel_at(point) == Some(controller)
    }

    fn panel_at(&self, point: PxPosition) -> Option<State<MenuController>> {
        self.entries
            .with(|entries| {
                entries
                    .iter()
                    .rev()
                    .filter(|entry| entry.is_alive())
                    .find_map(|entry| {
                        let branch = entry.get();
                        (branch.controller.with(MenuController::is_open)
                            && branch
                                .bounds
                                .get()
                                .is_some_and(|bounds| bounds.contains(point)))
                        .then_some(branch.controller)
                    })
            })
            .or_else(|| {
                (self.root.with(MenuController::is_open)
                    && self
                        .bounds
                        .get()
                        .is_some_and(|bounds| bounds.contains(point)))
                .then_some(self.root)
            })
    }

    pub fn apply_hover(&self, parent: State<MenuController>, branch: Option<State<MenuBranch>>) {
        if let Some(branch) = branch {
            self.open(branch);
        } else {
            self.prune_after(parent, false);
        }
    }

    fn prune_after(&self, controller: State<MenuController>, restore: bool) {
        let child = self.entries.with(|entries| {
            entries
                .iter()
                .copied()
                .find(|entry| entry.is_alive() && entry.get().parent == controller)
        });
        if let Some(child) = child {
            self.remove(child, controller, restore);
        }
    }

    pub fn remove(&self, branch: State<MenuBranch>, parent: State<MenuController>, restore: bool) {
        let entries = self.entries.get();
        let Some(index) = entries.iter().position(|entry| *entry == branch) else {
            return;
        };
        if restore
            && self.root.with(MenuController::is_open)
            && entries[index..]
                .iter()
                .any(|entry| entry.is_alive() && entry.get().scope.has_focus())
        {
            let scope = entries[..index]
                .iter()
                .filter(|entry| entry.is_alive())
                .find_map(|entry| {
                    let branch = entry.get();
                    (branch.controller == parent).then_some(branch.scope)
                })
                .unwrap_or(self.root_scope);
            self.restore_parent.set(Some(scope));
        }
        self.entries.set(entries[..index].to_vec());
        focus::defer_keyboard_until_layout();
        for entry in &entries[index..] {
            if entry.is_alive() {
                let controller = entry.get().controller;
                if controller.is_alive() {
                    controller.with_mut(MenuController::close);
                }
            }
        }
    }

    pub fn open(&self, entry: State<MenuBranch>) {
        let branch = entry.get();
        if !branch.enabled || !self.root.with(MenuController::is_open) {
            return;
        }
        if branch.controller.with(MenuController::is_open) {
            return;
        }
        self.prune_after(branch.parent, false);
        branch.controller.with_mut(MenuController::open);
        self.entries.with_mut(|entries| entries.push(entry));
        focus::defer_keyboard_until_layout();
    }

    pub fn close_all(&self) {
        self.root.with_mut(MenuController::close);
        focus::defer_keyboard_until_layout();
    }
}

pub(super) struct MenuHierarchyPointer(pub MenuCascade);

impl PointerInputModifierNode for MenuHierarchyPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass == PointerEventPass::Initial
            && input
                .pointer_changes
                .iter()
                .any(|change| matches!(change.content, CursorEventContent::Pressed(_)))
            && let Some(controller) = input
                .cursor_position_abs()
                .and_then(|point| self.0.panel_at(point))
        {
            self.0.prune_after(controller, true);
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct MenuBranch {
    pub parent: State<MenuController>,
    pub controller: State<MenuController>,
    pub scope: FocusScopeNode,
    pub enabled: bool,
    pub bounds: State<Option<FloatRect>>,
}

pub(super) struct MenuBranchLease {
    pub cascade: MenuCascade,
    pub entry: State<MenuBranch>,
    pub parent: State<MenuController>,
}

impl Drop for MenuBranchLease {
    fn drop(&mut self) {
        if self.cascade.entries.is_alive() && self.cascade.root.is_alive() {
            self.cascade.remove(self.entry, self.parent, true);
        }
    }
}
