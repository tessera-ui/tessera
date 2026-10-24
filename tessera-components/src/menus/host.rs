//! Cascading action menus anchored to their parent items.
//!
//! ## Usage
//!
//! Keep nested action menus above scrolling and clipped menu content.

use std::sync::Arc;

use parking_lot::RwLock;
use tessera_ui::{
    Callback, ComputedData, CursorEventContent, FocusManager, FocusRequester, FocusScopeNode,
    LayoutResult, MeasurementError, Modifier, PointerInput, PointerInputModifierNode, PxPosition,
    RenderSlot, State, key,
    layout::{MeasureScope, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera, use_context,
};

use crate::{modifier::ModifierExt as _, spacer::spacer};

use super::{
    MenuBounds, MenuBoundsHandle, MenuColorStyle, MenuController, MenuPlacement,
    MenuProviderConfig, is_click_outside_menu, menu_provider_content,
};

#[derive(Clone)]
pub(super) struct MenuHost {
    pub root: State<MenuController>,
    entries: State<Vec<State<MenuPopupRequest>>>,
    restore_root: State<bool>,
    root_scope: FocusScopeNode,
    restore_parent: State<Option<FocusScopeNode>>,
    origin: State<tessera_ui::LayoutCoordinates>,
    bounds: Arc<RwLock<Option<MenuBounds>>>,
    on_dismiss: Option<Callback>,
}

impl PartialEq for MenuHost {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
            && self.entries == other.entries
            && self.restore_root == other.restore_root
            && self.root_scope == other.root_scope
            && self.restore_parent == other.restore_parent
            && self.origin == other.origin
            && Arc::ptr_eq(&self.bounds, &other.bounds)
            && self.on_dismiss == other.on_dismiss
    }
}

impl MenuHost {
    pub fn has_descendants(&self, parent: State<MenuController>) -> bool {
        self.entries
            .get()
            .iter()
            .any(|request| request.is_alive() && request.get().parent == parent)
    }

    pub fn apply_hover(
        &self,
        parent: State<MenuController>,
        request: Option<State<MenuPopupRequest>>,
    ) {
        if let Some(request) = request {
            self.open(request);
        } else if let Some(request) = self
            .entries
            .get()
            .into_iter()
            .find(|request| request.is_alive() && request.get().parent == parent)
        {
            self.remove_branch(request);
        }
    }

    pub fn allows_hover(
        &self,
        controller: State<MenuController>,
        point: Option<PxPosition>,
    ) -> bool {
        let origin = self.origin.get();
        let point = point.map(|point| origin.local_to_window.inverse_point(point));
        let mut hit = self
            .bounds
            .read()
            .filter(|bounds| {
                self.root.with(MenuController::is_open)
                    && !is_click_outside_menu(point, Some(*bounds))
            })
            .map(|_| self.root);
        for request in self.entries.get() {
            if request.is_alive() {
                let request = request.get();
                if request.controller.with(MenuController::is_open)
                    && request.bounds.read().is_some()
                    && !is_click_outside_menu(point, *request.bounds.read())
                {
                    hit = Some(request.controller);
                }
            }
        }
        point.is_some() && hit == Some(controller)
    }

    pub fn close_disabled_branch(&self, request: State<MenuPopupRequest>) {
        if self.entries.get().contains(&request) {
            if self.popup_has_focus() {
                self.restore_parent(request.get().parent);
            }
            self.remove_branch(request);
        }
    }
    fn restore_parent(&self, parent: State<MenuController>) {
        let scope = self
            .entries
            .get()
            .iter()
            .filter(|request| request.is_alive())
            .map(|request| request.get())
            .find(|request| request.controller == parent)
            .map(|request| request.scope)
            .unwrap_or(self.root_scope);
        self.restore_parent.set(Some(scope));
    }
    pub fn popup_has_focus(&self) -> bool {
        self.entries
            .get()
            .iter()
            .any(|request| request.is_alive() && request.get().scope.has_focus())
    }

    pub fn take_root_restore(&self) -> bool {
        self.restore_root.with_mut(std::mem::take)
    }

    pub fn close_all(&self) {
        if self.popup_has_focus() {
            self.restore_root.set(true);
        }
        self.close_descendants();
        if self.root.is_alive() {
            self.root.with_mut(MenuController::close);
        }
        FocusManager::current().defer_keyboard_until_layout();
    }

    pub fn dismiss_all(&self) {
        if let Some(callback) = self.on_dismiss {
            callback.call();
        }
        self.close_all();
    }

    pub fn exit(&self, reverse: bool) {
        self.dismiss_all();
        super::close_for_traversal(self.root, &None, reverse);
    }

    fn close_descendants(&self) {
        let entries = self.entries.get();
        if entries.is_empty() {
            return;
        }
        self.entries.set(Vec::new());
        for request in entries {
            if request.is_alive() {
                let controller = request.get().controller;
                if controller.is_alive() {
                    controller.with_mut(MenuController::close);
                }
            }
        }
    }

    pub fn remove_branch(&self, request: State<MenuPopupRequest>) {
        let entries = self.entries.get();
        let Some(index) = entries.iter().position(|entry| *entry == request) else {
            return;
        };
        self.entries.set(entries[..index].to_vec());
        FocusManager::current().defer_keyboard_until_layout();
        for request in &entries[index..] {
            if request.is_alive() {
                let controller = request.get().controller;
                if controller.is_alive() {
                    controller.with_mut(MenuController::close);
                }
            }
        }
    }

    fn open(&self, request: State<MenuPopupRequest>) {
        let value = request.get();
        if !value.enabled || !self.root.with(MenuController::is_open) {
            return;
        }
        let entries = self.entries.get();
        let keep = if value.parent == self.root {
            0
        } else {
            let Some(index) = entries
                .iter()
                .position(|entry| entry.is_alive() && entry.get().controller == value.parent)
            else {
                return;
            };
            index + 1
        };
        if entries.get(keep) == Some(&request) && value.controller.with(MenuController::is_open) {
            return;
        }
        if let Some(previous) = entries.get(keep) {
            self.remove_branch(*previous);
        }
        value.controller.with_mut(MenuController::open);
        let mut entries = self.entries.get();
        entries.truncate(keep);
        entries.push(request);
        self.entries.set(entries);
        FocusManager::current().defer_keyboard_until_layout();
    }
}

struct MenuHierarchyPointer(MenuHost);

impl PointerInputModifierNode for MenuHierarchyPointer {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if !input
            .pointer_changes
            .iter()
            .any(|change| matches!(change.content, CursorEventContent::Pressed(_)))
        {
            return;
        }
        let entries = self.0.entries.get();
        let point = input.cursor_position_rel;
        let mut keep = if !is_click_outside_menu(point, *self.0.bounds.read()) {
            Some(0)
        } else {
            None
        };
        for (index, request) in entries.iter().enumerate() {
            if request.is_alive() && !is_click_outside_menu(point, *request.get().bounds.read()) {
                keep = Some(index + 1);
            }
        }
        if let Some(keep) = keep
            && let Some(request) = entries.get(keep)
            && request.is_alive()
        {
            let value = request.get();
            if self.0.popup_has_focus() {
                self.0.restore_parent(value.parent);
            }
            self.0.remove_branch(*request);
        }
    }
}

#[derive(Clone)]
pub(super) struct MenuPopupRequest {
    pub host: MenuHost,
    pub parent: State<MenuController>,
    pub controller: State<MenuController>,
    pub anchor: State<tessera_ui::LayoutCoordinates>,
    pub scope: FocusScopeNode,
    pub fallback: FocusRequester,
    pub content: RenderSlot,
    pub placement: MenuPlacement,
    pub color_style: MenuColorStyle,
    pub enabled: bool,
    pub bounds: Arc<RwLock<Option<MenuBounds>>>,
}

impl PartialEq for MenuPopupRequest {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.parent == other.parent
            && self.controller == other.controller
            && self.anchor == other.anchor
            && self.scope == other.scope
            && self.fallback == other.fallback
            && self.content == other.content
            && self.placement == other.placement
            && self.color_style == other.color_style
            && self.enabled == other.enabled
            && Arc::ptr_eq(&self.bounds, &other.bounds)
    }
}

impl MenuPopupRequest {
    pub fn open(request: State<Self>) {
        request.get().host.open(request);
    }
}

pub(super) struct MenuPopupLease {
    pub host: MenuHost,
    pub request: State<MenuPopupRequest>,
    pub parent: State<MenuController>,
    pub scope: FocusScopeNode,
}

impl Drop for MenuPopupLease {
    fn drop(&mut self) {
        if self.host.entries.is_alive() {
            if self.host.entries.get().contains(&self.request)
                && self.host.root.is_alive()
                && self.host.root.with(MenuController::is_open)
                && (self.scope.has_focus() || self.host.popup_has_focus())
            {
                self.host.restore_parent(self.parent);
            }
            self.host.remove_branch(self.request);
        }
    }
}

#[derive(Clone, PartialEq)]
struct MenuHostLayout;

impl tessera_ui::LayoutModifierNode for MenuHostLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let constraint = scope.parent_constraint().without_min();
        let mut size = ComputedData::ZERO;
        let mut result = LayoutResult::default();
        for (index, child) in scope.children().into_iter().enumerate() {
            let measured = child.measure(scope, &constraint)?;
            if index == 0 {
                size = measured.size();
            }
            result.place_child(child, PxPosition::ZERO);
        }
        Ok(result.with_size(size))
    }
}

#[tessera]
pub(super) fn menu_host(provider: Option<MenuProviderConfig>) {
    let mut provider = provider.unwrap_or_default();
    let root = provider
        .controller
        .expect("menu host requires a root controller");
    let host = MenuHost {
        root,
        entries: remember(Vec::new),
        restore_root: remember(|| false),
        root_scope: remember(FocusScopeNode::new).get(),
        restore_parent: remember(|| None),
        origin: remember(tessera_ui::LayoutCoordinates::default),
        bounds: remember(|| Arc::new(RwLock::new(None))).get(),
        on_dismiss: provider.on_dismiss,
    };
    provider.bounds = Some(MenuBoundsHandle(host.bounds.clone()));
    provider.focus_scope = Some(host.root_scope);
    let pending = host.restore_parent;
    let mut modifier = Modifier::new();
    if pending.get().is_some() {
        modifier = modifier.on_globally_positioned(move |_| {
            if let Some(scope) = pending.with_mut(Option::take)
                && root.with(MenuController::is_open)
            {
                if !scope.has_focus() {
                    scope.request_focus();
                }
            }
        });
    }
    modifier = modifier.push_pointer_preview_input(MenuHierarchyPointer(host.clone()));
    let initial = host.clone();
    provide_context(
        move || initial,
        move || {
            let inherited = use_context::<MenuHost>().expect("menu host");
            if inherited.get() != host {
                inherited.set(host.clone());
            }
            let origin = host.origin;
            layout()
                .policy(MenuHostLayout)
                .modifier(modifier.clone().on_layout_coordinates(move |bounds| {
                    if origin.get() != bounds {
                        origin.set(bounds);
                    }
                }))
                .child(move || {
                    menu_provider_content().provider(provider.clone());
                    if host.root.with(MenuController::is_open) {
                        for request in host.entries.get() {
                            if request.is_alive() {
                                key(request, || {
                                    menu_popup_layer().request(request);
                                });
                            }
                        }
                    } else {
                        host.close_descendants();
                    }
                });
        },
    );
}

#[tessera]
fn menu_popup_layer(request: Option<State<MenuPopupRequest>>) {
    let request = request.expect("popup request");
    let value = request.get();
    let anchor = value.anchor.get();
    let origin = value.host.origin.get();
    let anchor = origin.local_to_window.inverse_rect(anchor.window_bounds());
    let mut modifier = Modifier::new();
    if !value.controller.with(MenuController::is_open) {
        let host = value.host.clone();
        modifier = modifier.on_globally_positioned(move |_| {
            host.remove_branch(request);
        });
    }
    let provider = MenuProviderConfig {
        controller: Some(value.controller),
        placement: value.placement,
        offset: [tessera_ui::Dp::ZERO; 2],
        precise_anchor: Some(anchor),
        focus_restorer_fallback: Some(value.fallback),
        focus_scope: Some(value.scope),
        parent_request: Some(request),
        bounds: Some(MenuBoundsHandle(value.bounds.clone())),
        color_style: value.color_style,
        container_color: value.color_style.container(
            &use_context::<crate::theme::MaterialTheme>()
                .unwrap()
                .get()
                .color_scheme,
        ),
        main_content: Some(RenderSlot::new(|| {
            spacer().modifier(Modifier::new().fill_max_size());
        })),
        menu_content: Some(value.content),
        ..Default::default()
    };
    layout().modifier(modifier).child(move || {
        menu_provider_content().provider(provider.clone());
    });
}
