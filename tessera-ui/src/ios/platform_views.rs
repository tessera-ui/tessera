//! UIKit native-view factory and lifecycle integration.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, hash_map},
    rc::Rc,
};

use objc2::{
    ClassType, DefinedClass, MainThreadMarker, Message, define_class, msg_send, rc::Retained,
    runtime::NSObjectProtocol,
};
use objc2_core_foundation::{CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_quartz_core::CALayer;
use objc2_ui_kit::{UIEvent, UITextField, UITextView, UIView};

use crate::{
    FloatRect, PlatformViewId,
    render_graph::{PlatformViewInputBlocker, PlatformViewPlacement},
};

/// Creates and manages the UIKit view associated with a Tessera platform view.
pub trait IosPlatformViewFactory: 'static {
    /// Creates the native view on the UIKit main thread.
    fn create(&self, main_thread: MainThreadMarker) -> Retained<UIView>;

    /// Updates the native view after a Tessera frame is built.
    fn update(&self, _view: &UIView, _main_thread: MainThreadMarker) {}

    /// Clears transient state just before Tessera detaches the view for release.
    fn reset(&self, _view: &UIView, _main_thread: MainThreadMarker) {}

    /// Releases resources owned by the factory for this view.
    fn release(&self, _view: Retained<UIView>, _main_thread: MainThreadMarker) {}
}

define_class!(
    #[unsafe(super(UIView))]
    #[name = "TesseraPlatformViewContainer"]
    #[ivars = RefCell<Vec<CGRect>>]
    struct PlatformViewContainer;

    impl PlatformViewContainer {
        #[unsafe(method(pointInside:withEvent:))]
        fn point_inside(&self, point: CGPoint, event: Option<&UIEvent>) -> bool {
            // SAFETY: This is the superclass implementation of the overridden method.
            let inside: bool = unsafe { msg_send![super(self), pointInside: point, withEvent: event] };
            inside && !self.ivars().borrow().iter().any(|rect| {
                point.x >= rect.origin.x
                    && point.y >= rect.origin.y
                    && point.x < rect.origin.x + rect.size.width
                    && point.y < rect.origin.y + rect.size.height
            })
        }
    }
);

struct Entry {
    factory: Rc<dyn IosPlatformViewFactory>,
    container: Retained<PlatformViewContainer>,
    placement: Retained<UIView>,
    view: Retained<UIView>,
    retiring: bool,
}

impl Entry {
    fn release(self, main_thread: MainThreadMarker) {
        self.container.endEditing(true);
        self.factory.reset(&self.view, main_thread);
        self.view.removeFromSuperview();
        self.container.removeFromSuperview();
        self.factory.release(self.view, main_thread);
    }
}

#[derive(Default)]
pub(crate) struct IosPlatformViews {
    root: Option<Retained<UIView>>,
    entries: HashMap<u64, Entry>,
    view_order: Vec<u64>,
    scale_factor: f64,
    observed_first_responder: u64,
    suspended: bool,
}

impl IosPlatformViews {
    pub(crate) fn sync_focus(&mut self, owner: &mut crate::focus::FocusOwner) {
        let mut focusable = Vec::new();
        let mut focused = 0;
        for (&id, entry) in &self.entries {
            if entry.retiring || entry.container.isHidden() {
                continue;
            }
            let mut responders = Vec::new();
            collect_keyboard_responders(&entry.view, &mut responders);
            if !responders.is_empty() {
                focusable.push(id);
            }
            if responders.iter().any(|view| view.isFirstResponder()) {
                focused = id;
            }
        }
        let last = std::mem::replace(&mut self.observed_first_responder, focused);
        let (gained, lost) = if focused == last {
            (0, 0)
        } else {
            (focused, last)
        };
        owner.sync_native_focus(&focusable, gained, lost);
    }

    pub(crate) fn keyboard_responders(&self, id: u64) -> Vec<Retained<UIView>> {
        let mut responders = Vec::new();
        if let Some(entry) = self.entries.get(&id)
            && !entry.retiring
            && !entry.container.isHidden()
        {
            collect_keyboard_responders(&entry.view, &mut responders);
        }
        responders
    }

    pub(crate) fn keyboard_root(&self) -> Option<Retained<UIView>> {
        if self.view_order.is_empty() {
            return None;
        }
        self.root
            .clone()
            .filter(|root| root.window().is_some_and(|window| window.isKeyWindow()))
    }

    pub(crate) fn set_root(
        &mut self,
        root: Retained<UIView>,
        scale_factor: f64,
        main_thread: MainThreadMarker,
    ) {
        self.suspended = false;
        self.root = Some(root.clone());
        self.view_order.clear();
        self.scale_factor = scale_factor.max(1.0);
        for entry in self.entries.values() {
            entry.container.setHidden(true);
            root.addSubview(&entry.container);
            entry.factory.update(&entry.view, main_thread);
        }
    }

    pub(crate) fn register(
        &mut self,
        id: PlatformViewId,
        factory: Rc<dyn IosPlatformViewFactory>,
        main_thread: MainThreadMarker,
    ) {
        if self
            .entries
            .get(&id.as_raw())
            .is_some_and(|entry| !entry.retiring)
        {
            return;
        }
        if let Some(previous) = self.entries.remove(&id.as_raw()) {
            previous.release(main_thread);
        }
        let view = factory.create(main_thread);
        let container = main_thread
            .alloc::<PlatformViewContainer>()
            .set_ivars(RefCell::default());
        // SAFETY: The UIView subclass has no initialization requirements beyond its ivars.
        let container: Retained<PlatformViewContainer> =
            unsafe { msg_send![super(container), init] };
        container.setClipsToBounds(true);
        container.setHidden(true);
        let placement = UIView::new(main_thread);
        container.addSubview(&placement);
        placement.addSubview(&view);
        if let Some(root) = &self.root {
            root.addSubview(&container);
        }
        self.view_order.clear();
        if let Some(previous) = self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                container,
                placement,
                view,
                retiring: false,
            },
        ) {
            previous.release(main_thread);
        }
    }

    pub(crate) fn unregister(
        &mut self,
        id: PlatformViewId,
        main_thread: MainThreadMarker,
    ) -> Result<(), IosPlatformViewError> {
        if self.suspended {
            return Ok(());
        }
        let hash_map::Entry::Occupied(mut entry) = self.entries.entry(id.as_raw()) else {
            return Err(IosPlatformViewError::NotRegistered(id));
        };
        if self.root.is_some() && !entry.get().container.isHidden() {
            entry.get_mut().retiring = true;
        } else {
            let entry = entry.remove();
            self.view_order.retain(|&view_id| view_id != id.as_raw());
            entry.release(main_thread);
        }
        Ok(())
    }

    pub(crate) fn suspend(&mut self) {
        self.suspended = true;
    }

    pub(crate) fn root_layer(&self) -> Option<Retained<CALayer>> {
        self.root.as_ref().map(|root| root.layer())
    }

    pub(crate) fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        blockers: &[PlatformViewInputBlocker],
        overlays_after: &[usize],
        layers: &[Retained<CALayer>],
        viewport: [f32; 2],
        main_thread: MainThreadMarker,
    ) {
        let Some(root) = self.root.clone() else {
            return;
        };
        let mut view_order = Vec::with_capacity(placements.len());
        let mut visible_views = HashSet::with_capacity(placements.len());
        for (index, placement) in placements.iter().enumerate() {
            let Some(entry) = self.entries.get(&placement.id) else {
                continue;
            };
            if entry.retiring {
                continue;
            }
            view_order.push(placement.id);
            let bounds = placement.geometry.window_bounds();
            let clip = placement
                .clip_bounds
                .map_or(Some(bounds), |clip| bounds.intersection(clip.into()))
                .filter(|clip| {
                    placement.opacity > 0.0
                        && !placement.unsupported_shape_clip
                        && clip.width > 0.0
                        && clip.height > 0.0
                });
            if let Some(clip) = clip {
                let scale = self.scale_factor;
                let container_frame = view_frame(clip, scale);
                if entry.container.frame() != container_frame {
                    entry.container.setFrame(container_frame);
                }
                let local_size = placement.geometry.local_size;
                let native_size = CGSize::new(
                    f64::from(local_size.width.0) / scale,
                    f64::from(local_size.height.0) / scale,
                );
                let placement_bounds = CGRect::new(CGPoint::new(0.0, 0.0), native_size);
                if entry.placement.bounds() != placement_bounds {
                    entry.placement.setBounds(placement_bounds);
                }
                let center = CGPoint::new(
                    (f64::from(bounds.x - clip.x) + f64::from(bounds.width) * 0.5) / scale,
                    (f64::from(bounds.y - clip.y) + f64::from(bounds.height) * 0.5) / scale,
                );
                if entry.placement.center() != center {
                    entry.placement.setCenter(center);
                }
                let zoom = f64::from(placement.geometry.local_to_window.scale());
                let transform = CGAffineTransform {
                    a: zoom,
                    b: 0.0,
                    c: 0.0,
                    d: zoom,
                    tx: 0.0,
                    ty: 0.0,
                };
                if entry.placement.transform() != transform {
                    entry.placement.setTransform(transform);
                }
                // Size and transform are separate so UIKit lays out descendants in local units.
                let mut native_bounds = entry.view.bounds();
                if native_bounds.size != native_size {
                    native_bounds.size = native_size;
                    entry.view.setBounds(native_bounds);
                }
                let native_center = CGPoint::new(native_size.width * 0.5, native_size.height * 0.5);
                if entry.view.center() != native_center {
                    entry.view.setCenter(native_center);
                }
                let opacity = f64::from(placement.opacity);
                if entry.container.alpha() != opacity {
                    entry.container.setAlpha(opacity);
                }
                let mut blocked_regions = entry.container.ivars().borrow_mut();
                blocked_regions.clear();
                blocked_regions.extend(
                    blockers
                        .iter()
                        .filter(|blocker| blocker.id == placement.id)
                        .filter_map(|blocker| clip.intersection(blocker.bounds.into()))
                        .map(|region| {
                            view_frame(
                                FloatRect {
                                    x: region.x - clip.x,
                                    y: region.y - clip.y,
                                    ..region
                                },
                                scale,
                            )
                        }),
                );
                visible_views.insert(placement.id);
            }
            // The window's WGPU layer is not a UIKit subview. Give it depth zero
            // rather than relying on UIKit's insertion indices to keep it below all views.
            let layer = entry.container.layer();
            let depth = (index * 2 + 1) as f64;
            if layer.zPosition() != depth {
                layer.setZPosition(depth);
            }
            entry.factory.update(&entry.view, main_thread);
        }
        for (id, entry) in &self.entries {
            let hidden = !visible_views.contains(id);
            if entry.container.isHidden() != hidden {
                entry.container.setHidden(hidden);
            }
        }
        if self.view_order != view_order {
            // UIKit's hit-testing order follows subviews, not layer depth.
            for id in &view_order {
                root.bringSubviewToFront(&self.entries[id].container);
            }
            self.view_order = view_order;
        }
        let retired: Vec<_> = self
            .entries
            .extract_if(|_, entry| entry.retiring)
            .map(|(_, entry)| entry)
            .collect();
        for entry in retired {
            entry.release(main_thread);
        }

        let viewport = CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(f64::from(viewport[0]), f64::from(viewport[1])),
        );
        for (index, layer) in layers.iter().enumerate() {
            if layer.frame() != viewport {
                layer.setFrame(viewport);
            }
            if layer.contentsScale() != self.scale_factor {
                layer.setContentsScale(self.scale_factor);
            }
            let placement = overlays_after.get(index);
            if let Some(&placement) = placement {
                let z = (placement * 2 + 2) as f64;
                if layer.zPosition() != z {
                    layer.setZPosition(z);
                }
            }
            let hidden = placement.is_none();
            if layer.isHidden() != hidden {
                layer.setHidden(hidden);
            }
        }
    }
}

fn collect_keyboard_responders(view: &UIView, out: &mut Vec<Retained<UIView>>) {
    if view.isHidden() || !view.isUserInteractionEnabled() || view.alpha() <= 0.0 {
        return;
    }
    if view.canBecomeFirstResponder() || view.isFirstResponder() {
        out.push(view.retain());
    }
    // UIKit can report both the text control and its private field editor as
    // first responders. They form one traversal stop in either direction.
    if view.isKindOfClass(UITextField::class()) || view.isKindOfClass(UITextView::class()) {
        return;
    }
    for child in view.subviews().iter() {
        collect_keyboard_responders(&child, out);
    }
}

fn view_frame(bounds: FloatRect, scale: f64) -> CGRect {
    CGRect::new(
        CGPoint::new(f64::from(bounds.x) / scale, f64::from(bounds.y) / scale),
        CGSize::new(
            f64::from(bounds.width) / scale,
            f64::from(bounds.height) / scale,
        ),
    )
}

pub(crate) struct IosCompositionTransaction;

impl IosCompositionTransaction {
    pub(crate) fn begin() -> Self {
        objc2_quartz_core::CATransaction::begin();
        objc2_quartz_core::CATransaction::setDisableActions(true);
        Self
    }
}

impl Drop for IosCompositionTransaction {
    fn drop(&mut self) {
        objc2_quartz_core::CATransaction::commit();
    }
}

#[derive(Debug)]
pub(crate) enum IosPlatformViewError {
    /// The identity has no registered native view.
    NotRegistered(PlatformViewId),
}

impl std::fmt::Display for IosPlatformViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRegistered(id) => {
                write!(formatter, "platform view {} is not registered", id.as_raw())
            }
        }
    }
}

impl std::error::Error for IosPlatformViewError {}

impl Drop for IosPlatformViews {
    fn drop(&mut self) {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        for (_, entry) in self.entries.drain() {
            entry.release(main_thread);
        }
    }
}
