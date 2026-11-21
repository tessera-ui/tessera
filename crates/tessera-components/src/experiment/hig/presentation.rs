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
    /// Centred in the host.
    Center,
    /// Centred horizontally at the top of the host.
    TopCenter,
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
    /// Whether outside clicks leave the popup open.
    pub modal: bool,
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
        let dismiss: Vec<Callback> = self.popups.with(|popups| {
            popups
                .iter()
                .filter(|(_, popup)| !popup.modal)
                .map(|(_, popup)| popup.dismiss)
                .collect()
        });
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
                PopupPlacement::Center => (
                    (size.width - popup.width).to_f32() * 0.5,
                    (size.height - popup.height).to_f32() * 0.5,
                ),
                PopupPlacement::TopCenter => ((size.width - popup.width).to_f32() * 0.5, 0.0),
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

struct SurfaceKeys(Callback);

impl tessera_ui::KeyboardInputModifierNode for SurfaceKeys {
    fn on_keyboard_input(&self, input: tessera_ui::KeyboardInput<'_>) {
        let mut dismiss = false;
        input.keyboard_events.retain(|event| {
            if event.logical_key
                == tessera_ui::winit::keyboard::Key::Named(
                    tessera_ui::winit::keyboard::NamedKey::Escape,
                )
            {
                dismiss |= event.state == tessera_ui::winit::event::ElementState::Pressed;
                return false;
            }
            true
        });
        if dismiss {
            self.0.call();
        }
    }
}

/// How a presented surface looks and behaves.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct SurfaceStyle {
    pub placement: PopupPlacement,
    pub modal: bool,
    pub radius: Dp,
    pub padding: Dp,
    /// Moves keyboard focus into the surface when it appears.
    pub focus: bool,
}

/// Shows `content` on a rounded surface while `open`, anchored to `anchor` in window
/// coordinates. Escape calls `dismiss`.
#[tessera]
pub(super) fn present_surface(
    open: bool,
    anchor: FloatRect,
    style: SurfaceStyle,
    content: RenderSlot,
    dismiss: Callback,
) {
    let presentations = tessera_ui::use_context::<Presentations>().map(|context| context.get());
    let theme = tessera_ui::use_context::<HigTheme>()
        .map(|theme| theme.get())
        .unwrap_or_default();
    let key = remember(next_popup_key).get();
    let scope = remember(tessera_ui::FocusScopeNode::new).get();
    let Some(presentations) = presentations else {
        return;
    };
    remember(|| PopupLease { presentations, key });
    if open {
        presentations.show(
            key,
            Popup {
                anchor,
                placement: style.placement,
                content: RenderSlot::new(move || surface_panel(content, dismiss, style, scope)),
                dismiss,
                theme,
                modal: style.modal,
            },
        );
    } else {
        presentations.hide(key);
    }
}

#[derive(Clone, PartialEq)]
struct SurfaceBackground {
    color: tessera_ui::Color,
    border: tessera_ui::Color,
    radius: Dp,
}

impl tessera_ui::DrawModifierNode for SurfaceBackground {
    fn draw(
        &self,
        context: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let fill = |color, outline| crate::pipelines::perceptual_fill::PerceptualFillCommand {
            outline_width: outline,
            snap_to_pixels: false,
            path: None,
            color,
            radii: [self.radius; 2],
            insets: [Dp::ZERO; 4],
            clip_radius: None,
            line: None,
            edge_smoothness: 1.0,
            stroke: None,
        };
        let mut metadata = context.render_input.metadata_mut();
        let fragment = metadata.fragment_mut();
        fragment.push_draw_command(crate::pipelines::shape::command::ShapeCommand::Shadow {
            color: tessera_ui::Color::BLACK.with_alpha(0.25),
            shape: crate::shape_def::ResolvedShape::Rounded {
                corner_radii: [self.radius.to_pixels_f32(); 4],
                corner_g2: [2.0; 4],
            },
            smoothness: Dp(30.0).to_pixels_f32(),
        });
        fragment.push_draw_command(fill(self.color, Dp::ZERO));
        fragment.push_draw_command(fill(self.border, Dp(0.5)));
        content.draw(context.render_input);
    }
}

#[tessera]
fn surface_panel(
    content: RenderSlot,
    dismiss: Callback,
    style: SurfaceStyle,
    scope: tessera_ui::FocusScopeNode,
) {
    let dark = tessera_ui::use_context::<HigTheme>()
        .map(|theme| theme.get().appearance == super::theme::Appearance::Dark)
        .unwrap_or(false);
    // The surface itself takes focus so Escape reaches it even without focusable content.
    let requester = remember(tessera_ui::FocusRequester::new).get();
    let entered = remember(|| false);
    if style.focus && !entered.get() {
        entered.set(true);
        tessera_ui::focus::defer_keyboard_until_layout();
        scope.request_focus();
    }
    let color = if dark {
        tessera_ui::Color::from_rgb(0.03, 0.03, 0.03)
    } else {
        tessera_ui::Color::from_rgb(0.92, 0.92, 0.92)
    };
    let border = if dark {
        tessera_ui::Color::WHITE
    } else {
        tessera_ui::Color::BLACK
    }
    .with_alpha(0.12);
    let modifier = Modifier::new()
        .focus_scope(Some(scope))
        .push_keyboard_input(SurfaceKeys(dismiss))
        .push_draw(SurfaceBackground {
            color,
            border,
            radius: style.radius,
        });
    layout(Stack).modifier(modifier).child(move || {
        surface_content(style.padding, content)
            .requester_optional(style.focus.then_some(requester));
    });
}

#[tessera]
fn surface_content(
    padding: Dp,
    content: RenderSlot,
    requester: Option<tessera_ui::FocusRequester>,
) {
    let mut modifier = Modifier::new();
    if let Some(requester) = requester {
        modifier = modifier.focus_requester(requester).focusable();
    }
    crate::column::column()
        .modifier(tessera_foundation::modifier::ModifierExt::padding_all(
            modifier, padding,
        ))
        .children_shared(content);
}
