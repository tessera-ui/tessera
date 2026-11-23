//! DirectComposition surfaces for native views hosted in Tessera windows.

use std::{collections::HashMap, rc::Rc};

pub use windows::Win32::Graphics::DirectComposition::IDCompositionVisual as NativeVisual;

use windows::{
    Win32::{
        Foundation::HWND,
        Graphics::DirectComposition::{
            DCompositionCreateDevice2, IDCompositionDevice, IDCompositionRectangleClip,
            IDCompositionTarget, IDCompositionVisual, IDCompositionVisual3,
        },
    },
    core::{IUnknown, Interface},
};
use winit::{
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};

use crate::{PlatformViewId, render_graph::PlatformViewPlacement};

/// Creates and updates a native visual hosted in a Tessera window.
pub trait WindowsPlatformViewFactory: 'static {
    /// Creates the DirectComposition visual on the renderer thread.
    fn create(&self, device: &IDCompositionDevice) -> Result<NativeVisual, windows::core::Error>;

    /// Updates native content after a Tessera frame is built.
    fn update(&self, _visual: &NativeVisual) {}

    /// Releases resources owned by the factory for this visual.
    fn release(&self, _visual: NativeVisual) {}
}

struct Entry {
    factory: Rc<dyn WindowsPlatformViewFactory>,
    visual: NativeVisual,
    container: IDCompositionVisual,
    clip: IDCompositionRectangleClip,
}

pub(crate) struct WindowsPlatformViews {
    device: IDCompositionDevice,
    target: IDCompositionTarget,
    root: IDCompositionVisual,
    base: IDCompositionVisual,
    layers: Vec<IDCompositionVisual>,
    entries: HashMap<u64, Entry>,
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
            device,
            target,
            root,
            base,
            layers: Vec::new(),
            entries: HashMap::new(),
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
        let visual = factory.create(&self.device)?;
        let container = unsafe { self.device.CreateVisual()? };
        let clip = unsafe { self.device.CreateRectangleClip()? };
        unsafe {
            container.AddVisual(&visual, true, None::<&IDCompositionVisual>)?;
        }
        self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                visual,
                container,
                clip,
            },
        );
        Ok(())
    }

    pub(crate) fn unregister(&mut self, id: PlatformViewId) {
        if let Some(entry) = self.entries.remove(&id.as_raw()) {
            entry.factory.release(entry.visual);
        }
    }

    pub(crate) fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        overlays_after: &[usize],
    ) -> Result<(), windows::core::Error> {
        let mut order = Vec::with_capacity(1 + placements.len() + overlays_after.len());
        for entry in self.entries.values() {
            let visible: IDCompositionVisual3 = entry.container.cast()?;
            unsafe { visible.SetVisible(false)? };
        }
        order.push(self.base.clone());
        let mut overlay_index = 0;
        for (index, placement) in placements.iter().enumerate() {
            if let Some(entry) = self.entries.get(&placement.id) {
                let bounds = placement.geometry.window_bounds();
                let visible = !placement.unsupported_shape_clip && placement.opacity > 0.0;
                let clip = placement
                    .clip_bounds
                    .map_or(Some(bounds), |rect| bounds.intersection(rect.into()))
                    .filter(|rect| visible && rect.width > 0.0 && rect.height > 0.0);
                if let Some(clip_bounds) = clip {
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
                    entry.factory.update(&entry.visual);
                    unsafe { visible.SetVisible(true)? };
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
        Ok(())
    }

    pub(crate) fn commit(&self) -> Result<(), windows::core::Error> {
        let _ = &self.target;
        unsafe { self.device.Commit() }
    }
}

impl Drop for WindowsPlatformViews {
    fn drop(&mut self) {
        for (_, entry) in self.entries.drain() {
            entry.factory.release(entry.visual);
        }
    }
}
