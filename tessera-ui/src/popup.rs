//! Popups render content above the window from the component that owns it.
//!
//! ## Usage
//!
//! Show menus, tooltips, dialogs, and sheets above main content and earlier popups.

use crate::{
    Callback, FloatRect, LayoutCoordinates, Modifier, PxPosition, PxSize, RenderSlot,
    component_tree::NodeRole, layout::Stack, runtime::TesseraRuntime,
};

/// Geometry available to a [`PopupPositionPolicy`], in window coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupPositionInput {
    /// The layout node enclosing the popup call, in local units and window
    /// placement.
    pub anchor: LayoutCoordinates,
    /// The part of the anchor left visible by its ancestors' clipping, or
    /// `None` when it is fully clipped.
    pub visible_anchor: Option<FloatRect>,
    /// The area the popup may occupy. Keep the popup inside it; it may differ
    /// from the window.
    pub bounds: FloatRect,
    /// The measured popup content size.
    pub popup_size: PxSize,
}

/// Positions a popup in window coordinates.
///
/// Popup content is measured against the size of
/// [`PopupPositionInput::bounds`] regardless of the anchor, so positioning
/// only moves the measured content.
pub trait PopupPositionPolicy: Clone + PartialEq + 'static {
    /// Returns the window position of the popup's top-left corner.
    fn position(&self, input: &PopupPositionInput) -> PxPosition;
}

/// Places a popup at the window origin, for content that covers the window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WindowOrigin;

impl PopupPositionPolicy for WindowOrigin {
    fn position(&self, _input: &PopupPositionInput) -> PxPosition {
        PxPosition::ZERO
    }
}

pub(crate) trait ErasedPopupPosition {
    fn position(&self, input: &PopupPositionInput) -> PxPosition;
}

impl<P: PopupPositionPolicy> ErasedPopupPosition for P {
    fn position(&self, input: &PopupPositionInput) -> PxPosition {
        PopupPositionPolicy::position(self, input)
    }
}

pub(crate) struct PopupConfig {
    pub(crate) position: Box<dyn ErasedPopupPosition>,
    pub(crate) modal: bool,
    pub(crate) dismiss_on_outside_press: bool,
    pub(crate) dismiss_on_escape: bool,
    pub(crate) dismiss_on_back: bool,
    pub(crate) on_dismiss_request: Option<Callback>,
    /// Opening order among popups; later popups stack above earlier siblings.
    pub(crate) sequence: u64,
}

impl PopupConfig {
    pub(crate) fn dismisses_on_outside_press(&self) -> bool {
        self.dismiss_on_outside_press && self.on_dismiss_request.is_some()
    }

    pub(crate) fn dismisses_on_escape(&self) -> bool {
        self.dismiss_on_escape && self.on_dismiss_request.is_some()
    }

    pub(crate) fn dismisses_on_back(&self) -> bool {
        self.dismiss_on_back && self.on_dismiss_request.is_some()
    }
}

/// Emits the current component's node as the root of a popup layer.
///
/// The popup stays logically owned by its call site: contexts, remembered
/// state, focus restoration, and keyboard bubbling follow the enclosing
/// component. `position` places its content relative to the enclosing layout
/// node, which is the anchor. The
/// popup ignores the anchor's ancestors' clipping, scaling, and opacity, draws
/// above the main content, and receives pointer input first. A popup is hidden
/// while its anchor is not placed.
///
/// Popups stack above the layer that declares them. Among popups declared in
/// the same layer, modal popups stack above non-modal ones, and otherwise a
/// popup opened later stacks above an earlier one.
///
/// A popup never closes itself: dismissal requests invoke
/// [`PopupNode::on_dismiss_request`], and the owner removes the popup by no
/// longer calling it. Like [`layout`](crate::layout::layout), it emits the
/// current component's single node.
///
/// # Examples
///
/// ```
/// use tessera_ui::{
///     Modifier, remember, tessera,
///     popup::{WindowOrigin, popup},
/// };
///
/// #[tessera]
/// fn confirmation() {
///     let open = remember(|| true);
///     if open.get() {
///         popup(WindowOrigin)
///             .modal(true)
///             .on_dismiss_request(move || open.set(false))
///             .child(|| {});
///     }
/// }
/// ```
pub fn popup<P: PopupPositionPolicy>(position: P) -> PopupNode {
    crate::runtime::emit_node(NodeRole::Layout, &Stack);
    TesseraRuntime::with_mut(|runtime| {
        runtime.component_tree.set_emitted_popup(Box::new(position))
    });
    PopupNode { _private: () }
}

/// The node emitted by [`popup`].
pub struct PopupNode {
    _private: (),
}

impl PopupNode {
    fn configure(self, update: impl FnOnce(&mut PopupConfig)) -> Self {
        TesseraRuntime::with_mut(|runtime| runtime.component_tree.configure_emitted_popup(update));
        self
    }

    /// Blocks pointer input, keyboard focus, and accessibility for everything
    /// beneath the popup while it is shown. Defaults to `false`.
    ///
    /// Focus moves into a modal popup when it opens unless its content
    /// requests focus, and returns to the previous target when the popup
    /// closes while it holds focus.
    pub fn modal(self, modal: bool) -> Self {
        self.configure(|config| config.modal = modal)
    }

    /// Requests dismissal when a press lands outside the popup and the popups
    /// it declares. Defaults to `true`. Modal popups consume that press; others
    /// let it reach the content beneath.
    pub fn dismiss_on_outside_press(self, dismiss: bool) -> Self {
        self.configure(|config| config.dismiss_on_outside_press = dismiss)
    }

    /// Requests dismissal when Escape is not handled by the focused content.
    /// The topmost dismissible popup receives it. Defaults to `true`.
    pub fn dismiss_on_escape(self, dismiss: bool) -> Self {
        self.configure(|config| config.dismiss_on_escape = dismiss)
    }

    /// Requests dismissal on system back. Defaults to `true`.
    pub fn dismiss_on_back(self, dismiss: bool) -> Self {
        self.configure(|config| config.dismiss_on_back = dismiss)
    }

    /// Sets the action that dismisses the popup. Without it the popup
    /// ignores dismissal requests.
    pub fn on_dismiss_request(self, on_dismiss_request: impl Into<Callback>) -> Self {
        let callback = on_dismiss_request.into();
        self.configure(|config| config.on_dismiss_request = Some(callback))
    }

    /// Attaches a node-local modifier chain to the popup root.
    pub fn modifier(self, modifier: Modifier) -> Self {
        modifier.attach();
        self
    }

    /// Expands the popup content.
    pub fn child(self, child: impl Into<RenderSlot>) {
        child.into().render();
    }
}
