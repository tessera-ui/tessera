//! DirectComposition surfaces for native views hosted in Tessera windows.

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub use windows::Win32::Graphics::DirectComposition::IDCompositionVisual as NativeVisual;

use windows::{
    Win32::{
        Foundation::HWND,
        Graphics::DirectComposition::{
            DCompositionCreateDevice2, IDCompositionDevice, IDCompositionRectangleClip,
            IDCompositionTarget, IDCompositionVisual, IDCompositionVisual3,
        },
        UI::WindowsAndMessaging::GetForegroundWindow,
    },
    core::{IUnknown, Interface},
};
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, Touch, TouchPhase},
    keyboard::ModifiersState,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

use crate::{
    FloatRect, LayoutCoordinates, PlatformViewId, PxPosition,
    render_graph::{PlatformViewInputBlocker, PlatformViewPlacement},
};

/// Geometry of a visible Windows platform view in physical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowsViewPlacement {
    /// Unscaled layout size and its exact mapping into the parent window.
    pub coordinates: LayoutCoordinates,
    /// The visible rectangle in parent-window coordinates.
    pub visible_bounds: FloatRect,
}

/// Pointer input routed to a Windows platform view.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum WindowsViewInputEvent {
    /// The mouse moved; the point is in local physical pixels.
    Move(PxPosition),
    /// A mouse button changed state at a point in local physical pixels.
    Button {
        /// Position before any clipping or scene scaling.
        position: PxPosition,
        /// Button that changed.
        button: MouseButton,
        /// New button state.
        state: ElementState,
    },
    /// The wheel moved at a point in local physical pixels.
    Wheel {
        /// Position before any clipping or scene scaling.
        position: PxPosition,
        /// Unmodified platform wheel delta.
        delta: MouseScrollDelta,
    },
    /// A touch sample with its location in local physical pixels.
    Touch(Touch),
    /// The mouse is no longer routed to this view.
    Leave,
    /// All active pointer sequences for this view have ended without release.
    Cancel,
}

/// Creates and updates a native visual hosted in a Tessera window.
pub trait WindowsPlatformViewFactory: 'static {
    /// Creates the DirectComposition visual on the renderer thread.
    fn create(&self, device: &IDCompositionDevice) -> Result<NativeVisual, windows::core::Error>;

    /// Creates a visual for content that needs its parent window handle.
    /// The handle is valid until `release` returns.
    fn create_in_window(
        &self,
        device: &IDCompositionDevice,
        _parent: HWND,
    ) -> Result<NativeVisual, windows::core::Error> {
        self.create(device)
    }

    /// Updates native content after a Tessera frame is built.
    fn update(&self, _visual: &NativeVisual) {}

    /// Updates native size and visibility. `None` means the view is hidden.
    fn place(&self, _visual: &NativeVisual, _placement: Option<WindowsViewPlacement>) {}

    /// Enables pointer routing to this view for its mounted lifetime.
    fn accepts_pointer_input(&self) -> bool {
        false
    }

    /// Receives pointer input. Use the control's native focus API to receive
    /// keyboard and IME input.
    fn pointer_input(
        &self,
        _visual: &NativeVisual,
        _event: WindowsViewInputEvent,
        _modifiers: ModifiersState,
    ) {
    }

    /// Releases resources owned by the factory for this visual.
    fn release(&self, _visual: NativeVisual) {}
}

struct Entry {
    factory: Rc<dyn WindowsPlatformViewFactory>,
    visual: NativeVisual,
    container: IDCompositionVisual,
    clip: IDCompositionRectangleClip,
    interactive: bool,
    placement: Option<WindowsViewPlacement>,
}

pub(crate) struct WindowsPlatformViews {
    hwnd: HWND,
    device: IDCompositionDevice,
    target: IDCompositionTarget,
    root: IDCompositionVisual,
    base: IDCompositionVisual,
    layers: Vec<IDCompositionVisual>,
    entries: HashMap<u64, Entry>,
    input: ViewInput,
}

struct InputTarget {
    id: u64,
    placement: WindowsViewPlacement,
    interactive: bool,
    blockers: Vec<crate::PxRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointerOwner {
    Tessera,
    View(u64),
    Cancelled,
}

impl PointerOwner {
    fn view(self) -> Option<u64> {
        match self {
            Self::View(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Default)]
struct ViewInput {
    targets: Vec<InputTarget>,
    position: Option<PxPosition>,
    hover: Option<u64>,
    mouse: Option<(PointerOwner, HashSet<MouseButton>)>,
    touches: HashMap<u64, PointerOwner>,
    modifiers: ModifiersState,
}

impl ViewInput {
    fn hit(&self, point: PxPosition) -> Option<u64> {
        for target in self.targets.iter().rev() {
            if target.placement.visible_bounds.contains(point) {
                return (target.interactive && !target.blockers.iter().any(|b| b.contains(point)))
                    .then_some(target.id);
            }
        }
        None
    }

    fn local(&self, id: u64, point: PxPosition) -> Option<PxPosition> {
        let target = self.targets.iter().find(|target| target.id == id)?;
        Some(
            target
                .placement
                .coordinates
                .local_to_window
                .inverse_point(point),
        )
    }

    fn mouse_target(&self) -> PointerOwner {
        match &self.mouse {
            Some((owner, _)) => *owner,
            None => self
                .position
                .and_then(|point| self.hit(point))
                .map_or(PointerOwner::Tessera, PointerOwner::View),
        }
    }

    fn change_hover(
        &mut self,
        target: Option<u64>,
        events: &mut Vec<(u64, WindowsViewInputEvent)>,
    ) {
        if self.hover != target
            && let Some(id) = std::mem::replace(&mut self.hover, target)
        {
            events.push((id, WindowsViewInputEvent::Leave));
        }
    }

    fn button(&mut self, button: MouseButton, state: ElementState) -> PointerOwner {
        match state {
            ElementState::Pressed => {
                let owner = self.mouse_target();
                let (_, buttons) = self.mouse.get_or_insert_with(|| (owner, HashSet::new()));
                buttons.insert(button);
                owner
            }
            ElementState::Released => {
                let Some((owner, buttons)) = self.mouse.as_mut() else {
                    return PointerOwner::Cancelled;
                };
                let owner = if buttons.remove(&button) {
                    *owner
                } else {
                    PointerOwner::Cancelled
                };
                if buttons.is_empty() {
                    self.mouse = None;
                }
                owner
            }
        }
    }

    fn remove(&mut self, id: u64) -> bool {
        self.targets.retain(|target| target.id != id);
        let mut cancelled = false;
        if let Some((owner, _)) = &mut self.mouse
            && *owner == PointerOwner::View(id)
        {
            *owner = PointerOwner::Cancelled;
            cancelled = true;
        }
        for owner in self.touches.values_mut() {
            if *owner == PointerOwner::View(id) {
                *owner = PointerOwner::Cancelled;
                cancelled = true;
            }
        }
        if self.hover == Some(id) {
            self.hover = None;
            cancelled = true;
        }
        cancelled
    }
}

pub(crate) struct RoutedPointerInput {
    pub(crate) consumed: bool,
    pub(crate) native_press: bool,
    callbacks: Vec<(
        Rc<dyn WindowsPlatformViewFactory>,
        NativeVisual,
        WindowsViewInputEvent,
    )>,
    modifiers: ModifiersState,
}

impl RoutedPointerInput {
    pub(crate) fn dispatch(self) {
        for (factory, visual, event) in self.callbacks {
            factory.pointer_input(&visual, event, self.modifiers);
        }
    }
}

fn windows_placement(
    placement: &PlatformViewPlacement,
    viewport: [u32; 2],
) -> Option<WindowsViewPlacement> {
    if placement.opacity <= 0.0 || placement.unsupported_shape_clip {
        return None;
    }
    let bounds = placement.geometry.window_bounds();
    let viewport = FloatRect {
        x: 0.0,
        y: 0.0,
        width: viewport[0] as f32,
        height: viewport[1] as f32,
    };
    let visible_bounds = bounds.intersection(viewport)?;
    let visible_bounds = placement.clip_bounds.map_or(Some(visible_bounds), |clip| {
        visible_bounds.intersection(clip.into())
    })?;
    Some(WindowsViewPlacement {
        coordinates: LayoutCoordinates {
            local_size: placement.geometry.local_size,
            local_to_window: placement.geometry.local_to_window,
        },
        visible_bounds,
    })
}

impl WindowsPlatformViews {
    pub(crate) fn new(window: &Window) -> Result<Self, String> {
        let handle = window.window_handle().map_err(|error| error.to_string())?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err("Windows platform views require a Win32 window handle".to_owned());
        };
        let hwnd = HWND(handle.hwnd.get() as *mut _);
        Self::from_hwnd(hwnd).map_err(|error| error.to_string())
    }

    fn from_hwnd(hwnd: HWND) -> Result<Self, windows::core::Error> {
        let device: IDCompositionDevice = unsafe { DCompositionCreateDevice2(None::<&IUnknown>) }?;
        let target = unsafe { device.CreateTargetForHwnd(hwnd, false) }?;
        let root = unsafe { device.CreateVisual() }?;
        let base = unsafe { device.CreateVisual() }?;
        unsafe {
            target.SetRoot(&root)?;
            root.AddVisual(&base, true, None::<&IDCompositionVisual>)?;
            device.Commit()?;
        }
        Ok(Self {
            hwnd,
            device,
            target,
            root,
            base,
            layers: Vec::new(),
            entries: HashMap::new(),
            input: ViewInput::default(),
        })
    }

    pub(crate) fn base_visual_ptr(&self) -> *mut std::ffi::c_void {
        Interface::as_raw(&self.base)
    }

    pub(crate) fn create_slice_visual(&mut self) -> Result<*mut std::ffi::c_void, String> {
        let visual = unsafe { self.device.CreateVisual() }.map_err(|error| error.to_string())?;
        let visible: IDCompositionVisual3 = visual.cast().map_err(|error| error.to_string())?;
        unsafe {
            visible
                .SetVisible(true)
                .map_err(|error| error.to_string())?;
            self.root
                .AddVisual(&visual, true, None::<&IDCompositionVisual>)
                .map_err(|error| error.to_string())?;
        }
        self.layers.push(visual.clone());
        Ok(Interface::as_raw(&visual))
    }

    pub(crate) fn register(
        &mut self,
        id: PlatformViewId,
        factory: Rc<dyn WindowsPlatformViewFactory>,
    ) -> Result<(), windows::core::Error> {
        if self.entries.contains_key(&id.as_raw()) {
            return Ok(());
        }
        let container = unsafe { self.device.CreateVisual()? };
        let clip = unsafe { self.device.CreateRectangleClip()? };
        let visual = factory.create_in_window(&self.device, self.hwnd)?;
        if let Err(error) =
            unsafe { container.AddVisual(&visual, true, None::<&IDCompositionVisual>) }
        {
            factory.release(visual);
            return Err(error);
        }
        let interactive = factory.accepts_pointer_input();
        self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                visual,
                container,
                clip,
                interactive,
                placement: None,
            },
        );
        Ok(())
    }

    pub(crate) fn unregister(&mut self, id: PlatformViewId) {
        if let Some(entry) = self.entries.remove(&id.as_raw()) {
            if self.input.remove(id.as_raw()) {
                entry.factory.pointer_input(
                    &entry.visual,
                    WindowsViewInputEvent::Cancel,
                    self.input.modifiers,
                );
            }
            entry.factory.release(entry.visual);
        }
    }

    pub(crate) fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        blockers: &[PlatformViewInputBlocker],
        overlays_after: &[usize],
        viewport: [u32; 2],
    ) -> Result<(), windows::core::Error> {
        let mut order = Vec::with_capacity(1 + placements.len() + overlays_after.len());
        let mut targets = Vec::with_capacity(placements.len());
        for entry in self.entries.values() {
            let visible: IDCompositionVisual3 = entry.container.cast()?;
            unsafe { visible.SetVisible(false)? };
        }
        order.push(self.base.clone());
        let mut overlay_index = 0;
        for (index, placement) in placements.iter().enumerate() {
            if let Some(entry) = self.entries.get(&placement.id) {
                let bounds = placement.geometry.window_bounds();
                if let Some(geometry) = windows_placement(placement, viewport) {
                    let clip_bounds = geometry.visible_bounds;
                    let zoom = placement.geometry.local_to_window.scale();
                    let left = clip_bounds.x;
                    let top = clip_bounds.y;
                    let right = clip_bounds.x + clip_bounds.width;
                    let bottom = clip_bounds.y + clip_bounds.height;
                    let visible: IDCompositionVisual3 = entry.container.cast()?;
                    unsafe {
                        entry.container.SetOffsetX2(left)?;
                        entry.container.SetOffsetY2(top)?;
                        entry.clip.SetLeft2(0.0)?;
                        entry.clip.SetTop2(0.0)?;
                        entry.clip.SetRight2(right - left)?;
                        entry.clip.SetBottom2(bottom - top)?;
                        entry.container.SetClip(&entry.clip)?;
                        visible.SetOpacity2(placement.opacity)?;
                        entry.visual.SetOffsetX2(bounds.x - left)?;
                        entry.visual.SetOffsetY2(bounds.y - top)?;
                        if zoom != 1.0 {
                            let transform = self.device.CreateScaleTransform()?;
                            transform.SetScaleX2(zoom)?;
                            transform.SetScaleY2(zoom)?;
                            entry.visual.SetTransform(&transform)?;
                        } else {
                            entry.visual.SetTransform(None::<
                                &windows::Win32::Graphics::DirectComposition::IDCompositionTransform,
                            >)?;
                        }
                    }
                    unsafe { visible.SetVisible(true)? };
                    targets.push(InputTarget {
                        id: placement.id,
                        placement: geometry,
                        interactive: entry.interactive,
                        blockers: blockers
                            .iter()
                            .filter(|blocker| blocker.id == placement.id)
                            .map(|blocker| blocker.bounds)
                            .collect(),
                    });
                }
                order.push(entry.container.clone());
            }
            if overlays_after.get(overlay_index) == Some(&index) {
                if let Some(layer) = self.layers.get(overlay_index) {
                    order.push(layer.clone());
                }
                overlay_index += 1;
            }
        }
        unsafe {
            self.root.RemoveAllVisuals()?;
            let mut previous = None;
            for visual in &order {
                self.root.AddVisual(visual, true, previous)?;
                previous = Some(visual);
            }
        }
        for (&id, entry) in &mut self.entries {
            let placement = targets
                .iter()
                .find(|target| target.id == id)
                .map(|target| target.placement);
            if placement.is_none() && entry.placement.is_some() && self.input.remove(id) {
                entry.factory.pointer_input(
                    &entry.visual,
                    WindowsViewInputEvent::Cancel,
                    self.input.modifiers,
                );
            }
            if placement != entry.placement {
                entry.factory.place(&entry.visual, placement);
                entry.placement = placement;
            }
            if placement.is_some() {
                entry.factory.update(&entry.visual);
            }
        }
        self.input.targets = targets;
        Ok(())
    }

    pub(crate) fn commit(&self) -> Result<(), windows::core::Error> {
        let _ = &self.target;
        unsafe { self.device.Commit() }
    }

    pub(crate) fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.input.modifiers = modifiers;
    }

    pub(crate) fn route_cursor_moved(&mut self, position: PxPosition) -> RoutedPointerInput {
        self.input.position = Some(position);
        let owner = self.input.mouse_target();
        let target = owner.view();
        let mut events = Vec::new();
        if self.input.mouse.is_none() {
            self.input.change_hover(target, &mut events);
        }
        if let Some(id) = target
            && let Some(local) = self.input.local(id, position)
        {
            events.push((id, WindowsViewInputEvent::Move(local)));
        }
        self.routed(events, owner != PointerOwner::Tessera, false)
    }

    pub(crate) fn route_cursor_left(&mut self) -> RoutedPointerInput {
        if self.input.mouse.is_none() {
            self.input.position = None;
        }
        let mut events = Vec::new();
        self.input.change_hover(None, &mut events);
        self.routed(events, false, false)
    }

    pub(crate) fn route_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> RoutedPointerInput {
        let owner = self.input.button(button, state);
        let target = owner.view();
        let native_press =
            target.is_some() && state == ElementState::Pressed && button == MouseButton::Left;
        let mut events = Vec::new();
        if let (Some(id), Some(position)) = (target, self.input.position)
            && let Some(local) = self.input.local(id, position)
        {
            events.push((
                id,
                WindowsViewInputEvent::Button {
                    position: local,
                    button,
                    state,
                },
            ));
        }
        self.routed(events, owner != PointerOwner::Tessera, native_press)
    }

    pub(crate) fn route_mouse_wheel(&mut self, delta: MouseScrollDelta) -> RoutedPointerInput {
        let mut events = Vec::new();
        let owner = self.input.mouse_target();
        if let (Some(id), Some(position)) = (owner.view(), self.input.position)
            && let Some(local) = self.input.local(id, position)
        {
            events.push((
                id,
                WindowsViewInputEvent::Wheel {
                    position: local,
                    delta,
                },
            ));
        }
        self.routed(events, owner != PointerOwner::Tessera, false)
    }

    pub(crate) fn route_touch(&mut self, touch: Touch) -> RoutedPointerInput {
        let position = PxPosition::from_f64_arr2([touch.location.x, touch.location.y]);
        let owner = match touch.phase {
            TouchPhase::Started => self
                .input
                .hit(position)
                .map_or(PointerOwner::Tessera, PointerOwner::View),
            _ => self
                .input
                .touches
                .get(&touch.id)
                .copied()
                .unwrap_or(PointerOwner::Cancelled),
        };
        match touch.phase {
            TouchPhase::Started => {
                self.input.touches.insert(touch.id, owner);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.input.touches.remove(&touch.id);
            }
            TouchPhase::Moved => {}
        }
        let mut events = Vec::new();
        if let Some(id) = owner.view()
            && let Some(local) = self.input.local(id, position)
        {
            let mut local_touch = touch;
            local_touch.location =
                winit::dpi::PhysicalPosition::new(f64::from(local.x), f64::from(local.y));
            events.push((id, WindowsViewInputEvent::Touch(local_touch)));
        }
        self.routed(
            events,
            owner != PointerOwner::Tessera,
            owner.view().is_some() && touch.phase == TouchPhase::Started,
        )
    }

    pub(crate) fn clear_input(&mut self) -> RoutedPointerInput {
        let mut ids = HashSet::new();
        ids.extend(
            self.input
                .mouse
                .as_ref()
                .and_then(|(owner, _)| owner.view()),
        );
        ids.extend(self.input.touches.values().filter_map(|owner| owner.view()));
        ids.extend(self.input.hover);
        self.input.position = None;
        self.input.mouse = None;
        self.input.touches.clear();
        self.input.hover = None;
        let events = ids
            .into_iter()
            .map(|id| (id, WindowsViewInputEvent::Cancel))
            .collect();
        self.routed(events, false, false)
    }

    pub(crate) fn is_foreground(&self) -> bool {
        unsafe { GetForegroundWindow() == self.hwnd }
    }

    fn routed(
        &self,
        events: Vec<(u64, WindowsViewInputEvent)>,
        consumed: bool,
        native_press: bool,
    ) -> RoutedPointerInput {
        let callbacks = events
            .into_iter()
            .filter_map(|(id, event)| {
                self.entries
                    .get(&id)
                    .map(|entry| (entry.factory.clone(), entry.visual.clone(), event))
            })
            .collect::<Vec<_>>();
        RoutedPointerInput {
            consumed,
            native_press,
            callbacks,
            modifiers: self.input.modifiers,
        }
    }
}

impl Drop for WindowsPlatformViews {
    fn drop(&mut self) {
        for (_, entry) in self.entries.drain() {
            entry.factory.release(entry.visual);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{LocalToWindow, Px, PxRect, PxSize, render_graph::DrawGeometry};

    use super::*;

    fn target(id: u64, clip_bounds: Option<PxRect>) -> InputTarget {
        let geometry = DrawGeometry::new(
            PxSize::new(Px(40), Px(40)),
            LocalToWindow::new(2.0, [10, 20].into()).expect("valid scale and translation"),
        );
        let placement = windows_placement(
            &PlatformViewPlacement {
                id,
                geometry,
                opacity: 1.0,
                clip_bounds,
                unsupported_shape_clip: false,
            },
            [100, 100],
        )
        .expect("view intersects the window and its ancestor clip");
        InputTarget {
            id,
            placement,
            interactive: true,
            blockers: Vec::new(),
        }
    }

    #[test]
    fn hit_testing_uses_scene_order_and_clip_but_dispatch_uses_the_local_origin() {
        let mut front = target(2, Some(PxRect::new(Px(20), Px(30), Px(30), Px(30))));
        front
            .blockers
            .push(PxRect::new(Px(30), Px(40), Px(10), Px(10)));
        let input = ViewInput {
            targets: vec![target(1, None), front],
            ..ViewInput::default()
        };
        assert_eq!(input.hit([16, 26].into()), Some(1));
        assert_eq!(input.hit([28, 38].into()), Some(2));
        assert_eq!(input.local(2, [28, 38].into()), Some([9, 9].into()));
        assert_eq!(input.hit([34, 44].into()), None);
        assert_eq!(input.hit([50, 50].into()), Some(1));
    }

    #[test]
    fn held_mouse_buttons_keep_their_owner_and_a_removed_view_does_not_retarget_release() {
        let mut input = ViewInput {
            targets: vec![target(1, None), target(2, None)],
            position: Some([20, 30].into()),
            ..ViewInput::default()
        };
        assert_eq!(
            input.button(MouseButton::Left, ElementState::Pressed),
            PointerOwner::View(2)
        );
        input.position = Some([0, 0].into());
        assert_eq!(input.mouse_target(), PointerOwner::View(2));
        assert_eq!(
            input.button(MouseButton::Right, ElementState::Pressed),
            PointerOwner::View(2)
        );
        assert_eq!(
            input.button(MouseButton::Left, ElementState::Released),
            PointerOwner::View(2)
        );
        assert!(input.remove(2));
        input.position = Some([20, 30].into());
        assert_eq!(
            input.button(MouseButton::Right, ElementState::Released),
            PointerOwner::Cancelled
        );
        assert_eq!(input.mouse_target(), PointerOwner::View(1));
    }
}
