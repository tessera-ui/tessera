//! A host for menus and other temporary surfaces presented above window content.
//!
//! ## Usage
//!
//! Wrap window content so menus, pop-up buttons, and context menus can appear above it.

use std::sync::atomic::{AtomicU64, Ordering};

use tessera_ui::{
    Callback, ComputedData, Constraint, CursorEventContent, Dp, FloatRect, LayoutPolicy,
    LayoutResult, MeasurementError, Modifier, PointerEventPass, PointerInput,
    PointerInputModifierNode, Px, PxPosition, RenderSlot, State, key,
    layout::{MeasureScope, Stack, layout},
    modifier::{FocusModifierExt as _, ModifierCapabilityExt as _},
    provide_context, remember, tessera,
};

use super::theme::{HigTheme, hig_theme};

/// Where a popup appears relative to its anchor rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum PopupPlacement {
    /// Below the anchor, aligned to its leading edge.
    Below,
    /// Over the anchor, with the given offset, in physical pixels, from the popup's top to the
    /// row that should cover the anchor's vertical centre.
    Over(f32),
    /// With its top-leading corner at the anchor's origin.
    At,
}

/// A surface shown by a presenting control.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Popup {
    /// Anchor in window coordinates.
    pub anchor: FloatRect,
    pub placement: PopupPlacement,
    pub content: RenderSlot,
    /// Closes the popup after an outside click or Escape.
    pub dismiss: Callback,
    /// Appearance of the presenting control.
    pub theme: HigTheme,
}

/// Popups currently shown by a [`presentation_host`].
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Presentations {
    popups: State<Vec<(u64, Popup)>>,
    origin: State<FloatRect>,
}

impl Presentations {
    /// Shows or updates the popup registered under `key`.
    pub(super) fn show(self, key: u64, popup: Popup) {
        let current = self
            .popups
            .with(|popups| popups.iter().find(|(id, _)| *id == key).map(|(_, p)| *p));
        if current == Some(popup) {
            return;
        }
        self.popups.with_mut(|popups| {
            if let Some(entry) = popups.iter_mut().find(|(id, _)| *id == key) {
                entry.1 = popup;
            } else {
                popups.push((key, popup));
            }
        });
    }

    /// Removes the popup registered under `key`.
    pub(super) fn hide(self, key: u64) {
        if self.popups.is_alive()
            && self
                .popups
                .with(|popups| popups.iter().any(|(id, _)| *id == key))
        {
            self.popups
                .with_mut(|popups| popups.retain(|(id, _)| *id != key));
        }
    }

    fn dismiss_all(self) {
        let dismiss: Vec<Callback> = self
            .popups
            .with(|popups| popups.iter().map(|(_, popup)| popup.dismiss).collect());
        for callback in dismiss.into_iter().rev() {
            callback.call();
        }
    }
}

/// A key identifying one presenting control.
pub(super) fn next_popup_key() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Removes a control's popup when the control leaves the tree.
pub(super) struct PopupLease {
    pub presentations: Presentations,
    pub key: u64,
}

impl Drop for PopupLease {
    fn drop(&mut self) {
        self.presentations.hide(self.key);
    }
}

#[derive(Clone, PartialEq)]
struct HostLayout {
    origin: FloatRect,
    popups: Vec<(FloatRect, PopupPlacement)>,
}

impl LayoutPolicy for HostLayout {
    fn measure(&self, scope: &mut MeasureScope<'_>) -> Result<LayoutResult, MeasurementError> {
        let parent = *scope.parent_constraint().as_ref();
        let children = scope.children();
        let Some((content, rest)) = children.split_first() else {
            return Ok(LayoutResult::new(ComputedData::min_from_constraint(
                &parent,
            )));
        };
        let measured = content.measure(scope, &parent)?.size();
        // The host covers the window so popups can extend past the content's own bounds.
        let size = ComputedData {
            width: parent.width.resolve_max().unwrap_or(measured.width),
            height: parent.height.resolve_max().unwrap_or(measured.height),
        };
        let mut result = LayoutResult::new(size);
        result.place_child(*content, PxPosition::ZERO);
        let loose = Constraint::new(
            tessera_ui::AxisConstraint::new(Px::ZERO, Some(size.width)),
            tessera_ui::AxisConstraint::new(Px::ZERO, Some(size.height)),
        );
        let mut rest = rest.iter();
        if !self.popups.is_empty()
            && let Some(catcher) = rest.next()
        {
            catcher.measure(
                scope,
                &Constraint::new(
                    tessera_ui::AxisConstraint::exact(size.width),
                    tessera_ui::AxisConstraint::exact(size.height),
                ),
            )?;
            result.place_child(*catcher, PxPosition::ZERO);
        }
        for (child, (anchor, placement)) in rest.zip(&self.popups) {
            let popup = child.measure(scope, &loose)?.size();
            let left = anchor.x - self.origin.x;
            let top = anchor.y - self.origin.y;
            let (x, y) = match placement {
                PopupPlacement::Below => (left, top + anchor.height + Dp(2.0).to_pixels_f32()),
                PopupPlacement::Over(row) => (left, top + anchor.height * 0.5 - row),
                PopupPlacement::At => (left, top),
            };
            let clamp = |value: f32, extent: Px, available: Px| {
                value.min((available - extent).to_f32()).max(0.0).round() as i32
            };
            result.place_child(
                *child,
                PxPosition::new(
                    Px(clamp(x, popup.width, size.width)),
                    Px(clamp(y, popup.height, size.height)),
                ),
            );
        }
        Ok(result)
    }
}

struct DismissOnPress(Presentations);

impl PointerInputModifierNode for DismissOnPress {
    fn on_pointer_input(&self, input: PointerInput<'_>) {
        if input.pass != PointerEventPass::Main {
            return;
        }
        let mut pressed = false;
        for change in input.pointer_changes.iter_mut() {
            if change.is_consumed() {
                continue;
            }
            if matches!(change.content, CursorEventContent::Pressed(_)) {
                pressed = true;
            }
            change.consume();
        }
        if pressed {
            self.0.dismiss_all();
        }
    }
}

/// # presentation_host
///
/// Shows menus and other temporary surfaces presented by HIG controls above window content.
///
/// ## Usage
///
/// Wrap a window's content once so its menus, pop-up buttons, and context menus can appear.
///
/// ## Parameters
///
/// - `child` — window content.
///
/// ## Examples
///
/// ```
/// use tessera_components::experiment::hig::{
///     menu::{menu, menu_item},
///     presentation::presentation_host,
/// };
/// # use tessera_ui::tessera;
/// # #[tessera]
/// # fn component() {
///
/// presentation_host().child(|| {
///     menu("File").content(|| {
///         menu_item("Open").on_click(|| println!("open"));
///     });
/// });
/// # }
/// # tessera_ui::testing::UiTest::new().content(component).frame();
/// ```
#[tessera]
pub fn presentation_host(child: Option<RenderSlot>) {
    let popups = remember(Vec::new);
    let origin = remember(|| FloatRect::ZERO);
    let presentations = Presentations { popups, origin };
    let shown = popups.get();
    let policy = HostLayout {
        origin: origin.get(),
        popups: shown
            .iter()
            .map(|(_, popup)| (popup.anchor, popup.placement))
            .collect(),
    };
    provide_context(presentations, move || {
        let shown = shown.clone();
        layout(policy.clone())
            .modifier(Modifier::new().on_globally_positioned_precise(move |rect| {
                if origin.get() != rect {
                    origin.set(rect);
                }
            }))
            .child(move || {
                if let Some(child) = child {
                    host_content(child);
                }
                if !shown.is_empty() {
                    dismiss_layer(presentations);
                }
                for (id, popup) in shown.iter().copied() {
                    key(id, move || popup_layer(popup));
                }
            });
    });
}

#[tessera]
fn host_content(child: RenderSlot) {
    layout(Stack).child(child);
}

#[tessera]
fn dismiss_layer(presentations: Presentations) {
    layout(Stack).modifier(Modifier::new().push_pointer_input(DismissOnPress(presentations)));
}

#[tessera]
fn popup_layer(popup: Popup) {
    layout(Stack).child(move || {
        hig_theme().theme(popup.theme).child_shared(popup.content);
    });
}
