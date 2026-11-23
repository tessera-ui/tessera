//! AppKit platform-view factories for native views hosted in Tessera windows.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use objc2::{DefinedClass, define_class, msg_send, runtime::AnyObject};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_quartz_core::CALayer;

pub use objc2::{MainThreadMarker, rc::Retained};
pub use objc2_app_kit::NSView;

use crate::{
    PlatformViewId,
    render_graph::{PlatformViewInputBlocker, PlatformViewPlacement},
};

define_class!(
    #[unsafe(super(NSView))]
    #[name = "TesseraPlatformViewContainer"]
    #[ivars = RefCell<Vec<CGRect>>]
    struct PlatformViewContainer;

    impl PlatformViewContainer {
        #[unsafe(method(hitTest:))]
        fn hit_test(&self, point: CGPoint) -> *mut AnyObject {
            if self.ivars().borrow().iter().any(|rect| {
                point.x >= rect.origin.x
                    && point.y >= rect.origin.y
                    && point.x < rect.origin.x + rect.size.width
                    && point.y < rect.origin.y + rect.size.height
            }) {
                return std::ptr::null_mut();
            }
            unsafe { msg_send![super(self), hitTest: point] }
        }
    }
);

/// Creates and updates an AppKit view hosted in a Tessera window.
pub trait MacosPlatformViewFactory: 'static {
    /// Creates the native view on the AppKit main thread.
    fn create(&self, main_thread: MainThreadMarker) -> Retained<NSView>;

    /// Updates the native view after a Tessera frame is built.
    fn update(&self, _view: &NSView, _main_thread: MainThreadMarker) {}

    /// Releases resources owned by the factory for this view.
    fn release(&self, _view: Retained<NSView>, _main_thread: MainThreadMarker) {}
}

struct Entry {
    factory: Rc<dyn MacosPlatformViewFactory>,
    container: Retained<PlatformViewContainer>,
    view: Retained<NSView>,
}

#[derive(Default)]
pub(crate) struct MacosPlatformViews {
    root: Option<Retained<NSView>>,
    entries: HashMap<u64, Entry>,
    view_order: Vec<u64>,
    scale_factor: f64,
    suspended: bool,
}

impl MacosPlatformViews {
    pub(crate) fn set_root(
        &mut self,
        root: Retained<NSView>,
        scale_factor: f64,
        main_thread: MainThreadMarker,
    ) {
        self.suspended = false;
        self.root = Some(root.clone());
        self.view_order.clear();
        self.scale_factor = scale_factor.max(1.0);
        for entry in self.entries.values() {
            root.addSubview(&entry.container);
            entry.factory.update(&entry.view, main_thread);
        }
    }

    pub(crate) fn register(
        &mut self,
        id: PlatformViewId,
        factory: Rc<dyn MacosPlatformViewFactory>,
        main_thread: MainThreadMarker,
    ) {
        if self.entries.contains_key(&id.as_raw()) {
            return;
        }
        let view = factory.create(main_thread);
        view.setWantsLayer(true);
        view.setHidden(true);
        let container = main_thread
            .alloc::<PlatformViewContainer>()
            .set_ivars(RefCell::default());
        let container: Retained<PlatformViewContainer> =
            unsafe { msg_send![super(container), init] };
        container.setWantsLayer(true);
        container.setHidden(true);
        if let Some(layer) = container.layer() {
            layer.setMasksToBounds(true);
        }
        container.addSubview(&view);
        if let Some(root) = &self.root {
            root.addSubview(&container);
        }
        self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                container,
                view,
            },
        );
        self.view_order.clear();
    }

    pub(crate) fn unregister(&mut self, id: PlatformViewId, main_thread: MainThreadMarker) {
        if let Some(entry) = self.entries.remove(&id.as_raw()) {
            entry.container.removeFromSuperview();
            entry.factory.release(entry.view, main_thread);
            self.view_order.retain(|&entry_id| entry_id != id.as_raw());
        }
    }

    pub(crate) fn suspend(&mut self) {
        self.suspended = true;
    }

    pub(crate) fn root_layer(&self) -> Option<Retained<CALayer>> {
        self.root.as_ref().and_then(|root| root.layer())
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
        let Some(root) = self.root.as_ref() else {
            return;
        };
        if self.suspended {
            return;
        }

        let mut view_order = Vec::with_capacity(placements.len());
        for (index, placement) in placements.iter().enumerate() {
            let Some(entry) = self.entries.get(&placement.id) else {
                continue;
            };
            view_order.push(placement.id);
            let bounds = placement.geometry.window_bounds();
            let clipped = placement
                .clip_bounds
                .map_or(Some(bounds), |clip| bounds.intersection(clip.into()))
                .filter(|rect| {
                    placement.opacity > 0.0
                        && !placement.unsupported_shape_clip
                        && rect.width > 0.0
                        && rect.height > 0.0
                });
            if let Some(clip_rect) = clipped {
                let scale = self.scale_factor;
                let container_frame = CGRect::new(
                    CGPoint::new(
                        f64::from(clip_rect.x) / scale,
                        f64::from(viewport[1] - clip_rect.y - clip_rect.height) / scale,
                    ),
                    CGSize::new(
                        f64::from(clip_rect.width) / scale,
                        f64::from(clip_rect.height) / scale,
                    ),
                );
                if entry.container.frame() != container_frame {
                    entry.container.setFrame(container_frame);
                }
                let zoom = f64::from(placement.geometry.local_to_window.scale());
                let local_size = placement.geometry.local_size;
                let native_size = CGSize::new(
                    f64::from(local_size.width.0) * zoom / scale,
                    f64::from(local_size.height.0) * zoom / scale,
                );
                let view_bounds = CGRect::new(
                    CGPoint::new(0.0, 0.0),
                    CGSize::new(
                        f64::from(local_size.width.0) / scale,
                        f64::from(local_size.height.0) / scale,
                    ),
                );
                if entry.view.bounds() != view_bounds {
                    entry.view.setBounds(view_bounds);
                }
                let view_frame = CGRect::new(
                    CGPoint::new(
                        f64::from(bounds.x - clip_rect.x) / scale,
                        (f64::from(clip_rect.height - (bounds.y - clip_rect.y) - bounds.height))
                            / scale,
                    ),
                    native_size,
                );
                if entry.view.frame() != view_frame {
                    entry.view.setFrame(view_frame);
                }
                if let Some(layer) = entry.container.layer() {
                    let z = (index * 2 + 1) as f64;
                    if layer.zPosition() != z {
                        layer.setZPosition(z);
                    }
                    let alpha = f64::from(placement.opacity);
                    if layer.opacity() != alpha as f32 {
                        layer.setOpacity(alpha as f32);
                    }
                }
                let mut blocked_regions = entry.container.ivars().borrow_mut();
                blocked_regions.clear();
                blocked_regions.extend(
                    blockers
                        .iter()
                        .filter(|blocker| blocker.id == placement.id)
                        .filter_map(|blocker| clip_rect.intersection(blocker.bounds.into()))
                        .map(|region| {
                            CGRect::new(
                                CGPoint::new(
                                    f64::from(region.x - clip_rect.x) / scale,
                                    f64::from(
                                        clip_rect.height - (region.y - clip_rect.y) - region.height,
                                    ) / scale,
                                ),
                                CGSize::new(
                                    f64::from(region.width) / scale,
                                    f64::from(region.height) / scale,
                                ),
                            )
                        }),
                );
            }
            entry.container.setHidden(clipped.is_none());
            entry.factory.update(&entry.view, main_thread);
        }

        for (id, entry) in &self.entries {
            if !view_order.contains(id) {
                entry.container.setHidden(true);
            }
        }
        if self.view_order != view_order {
            for id in &view_order {
                root.addSubview(&self.entries[id].container);
            }
            self.view_order = view_order;
        }

        let size = root.bounds().size;
        let viewport = CGRect::new(CGPoint::new(0.0, 0.0), size);
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
            layer.setHidden(placement.is_none());
        }
    }
}

impl Drop for MacosPlatformViews {
    fn drop(&mut self) {
        let Some(main_thread) = MainThreadMarker::new() else {
            return;
        };
        for (_, entry) in self.entries.drain() {
            entry.container.removeFromSuperview();
            entry.factory.release(entry.view, main_thread);
        }
    }
}

pub(crate) struct MacosCompositionTransaction;

impl MacosCompositionTransaction {
    pub(crate) fn begin() -> Self {
        objc2_quartz_core::CATransaction::begin();
        objc2_quartz_core::CATransaction::setDisableActions(true);
        Self
    }
}

impl Drop for MacosCompositionTransaction {
    fn drop(&mut self) {
        objc2_quartz_core::CATransaction::commit();
    }
}
