//! X11 and Wayland subsurface composition for native views hosted in Tessera windows.

use std::{
    collections::HashMap,
    ffi::{CString, c_void},
    ptr::NonNull,
    rc::Rc,
    sync::Arc,
};

use wayland_client::{
    Connection as WaylandConnection, Dispatch, EventQueue, Proxy, QueueHandle,
    backend::{Backend, ObjectId},
    delegate_noop,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{
        wl_compositor, wl_region, wl_registry, wl_subcompositor, wl_subsurface, wl_surface,
    },
};
use wayland_protocols::wp::viewporter::client::{wp_viewport, wp_viewporter};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
    window::Window,
};
use x11_dl::{
    xfixes,
    xlib::{self, Display, Window as XWindow},
    xrender,
};

use crate::{FloatRect, PlatformViewId, PxSize, render_graph::PlatformViewPlacement};

/// Manages native content on the current Linux window backend.
pub trait LinuxPlatformViewFactory: 'static {
    /// Initializes native content in the supplied child window.
    fn create(&self, _window: X11ChildWindow) {}

    /// Updates the child after Tessera changes its placement.
    fn update(&self, _window: X11ChildWindow) {}

    /// Releases resources associated with the child window.
    fn release(&self, _window: X11ChildWindow) {}

    /// Initializes native content in the supplied Wayland subsurface.
    fn create_wayland(&self, _surface: WaylandChildSurface) {}

    /// Updates visible Wayland content. Attach and commit a buffer when the
    /// view first becomes visible or becomes visible again after being hidden.
    fn update_wayland(&self, _surface: WaylandChildSurface, _placement: WaylandViewPlacement) {}

    /// Releases resources associated with the Wayland subsurface.
    fn release_wayland(&self, _surface: WaylandChildSurface) {}
}

/// A borrowed handle to a Tessera-owned Wayland child surface.
///
/// The factory may attach buffers but must not destroy the surface, assign a
/// role, or use the handles after its `release_wayland` callback returns.
#[derive(Clone, Copy, Debug)]
pub struct WaylandChildSurface {
    display: NonNull<c_void>,
    surface: NonNull<c_void>,
}

/// Dimensions and visible area for a Wayland native content producer.
#[derive(Clone, Copy, Debug)]
pub struct WaylandViewPlacement {
    /// The measured size in Tessera's local physical pixels, before transforms.
    pub local_size: PxSize,
    /// The transformed bounds in window physical pixels.
    pub bounds: FloatRect,
    /// The part of `bounds` left after clipping, in window physical pixels.
    pub visible_bounds: FloatRect,
    /// The factory should attach a buffer rasterized at `local_size` physical
    /// pixels; Tessera's viewport maps and crops it to `visible_bounds`.
    /// The window's physical pixels per Wayland surface unit.
    pub scale_factor: f64,
    /// Whether the producer must submit a new buffer to map the view.
    pub needs_buffer: bool,
}

impl WaylandChildSurface {
    /// Returns the native `wl_display` pointer.
    pub fn display(self) -> *mut c_void {
        self.display.as_ptr()
    }

    /// Returns the native `wl_surface` pointer.
    pub fn surface(self) -> *mut c_void {
        self.surface.as_ptr()
    }

    /// Returns raw window handles suitable for creating a WGPU surface.
    pub fn raw_handles(self) -> (RawDisplayHandle, RawWindowHandle) {
        (
            RawDisplayHandle::Wayland(winit::raw_window_handle::WaylandDisplayHandle::new(
                self.display,
            )),
            RawWindowHandle::Wayland(winit::raw_window_handle::WaylandWindowHandle::new(
                self.surface,
            )),
        )
    }
}

/// An X11 child window owned by a Tessera platform view.
#[derive(Clone, Copy, Debug)]
pub struct X11ChildWindow {
    display: NonNull<c_void>,
    window: XWindow,
    screen: i32,
    visual_id: u64,
}

impl X11ChildWindow {
    /// Returns the native Xlib display pointer.
    pub fn display(self) -> *mut c_void {
        self.display.as_ptr()
    }

    /// Returns the native X11 window identifier.
    pub fn window(self) -> u64 {
        self.window
    }

    /// Returns raw window handles suitable for creating a WGPU surface.
    pub fn raw_handles(self) -> (RawDisplayHandle, RawWindowHandle) {
        (
            RawDisplayHandle::Xlib(winit::raw_window_handle::XlibDisplayHandle::new(
                Some(self.display),
                self.screen,
            )),
            RawWindowHandle::Xlib({
                let mut handle = winit::raw_window_handle::XlibWindowHandle::new(self.window);
                handle.visual_id = self.visual_id;
                handle
            }),
        )
    }
}

struct Connection {
    xlib: xlib::Xlib,
    display: NonNull<Display>,
    screen: i32,
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe { (self.xlib.XCloseDisplay)(self.display.as_ptr()) };
    }
}

struct Entry {
    factory: Rc<dyn LinuxPlatformViewFactory>,
    container: XWindow,
    window: XWindow,
    colormap: xlib::Colormap,
}

struct WaylandQueueState;

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _connection: &WaylandConnection,
        _queue: &QueueHandle<Self>,
    ) {
    }
}

delegate_noop!(WaylandQueueState: ignore wl_compositor::WlCompositor);
delegate_noop!(WaylandQueueState: wl_region::WlRegion);
delegate_noop!(WaylandQueueState: ignore wl_subcompositor::WlSubcompositor);
delegate_noop!(WaylandQueueState: ignore wl_subsurface::WlSubsurface);
delegate_noop!(WaylandQueueState: ignore wl_surface::WlSurface);
delegate_noop!(WaylandQueueState: wp_viewporter::WpViewporter);
delegate_noop!(WaylandQueueState: wp_viewport::WpViewport);

struct WaylandHost {
    root: wl_surface::WlSurface,
    compositor: wl_compositor::WlCompositor,
    subcompositor: wl_subcompositor::WlSubcompositor,
    viewporter: Option<wp_viewporter::WpViewporter>,
    queue: EventQueue<WaylandQueueState>,
    entries: HashMap<u64, WaylandEntry>,
    slices: Vec<WaylandSlice>,
    connection: WaylandConnection,
}

struct WaylandEntry {
    factory: Rc<dyn LinuxPlatformViewFactory>,
    child: WaylandChild,
    viewport: Option<wp_viewport::WpViewport>,
    visible: bool,
}

struct WaylandChild {
    surface: wl_surface::WlSurface,
    subsurface: wl_subsurface::WlSubsurface,
}

struct WaylandSlice {
    viewport: Option<wp_viewport::WpViewport>,
    child: WaylandChild,
    visible: bool,
}

impl Drop for WaylandEntry {
    fn drop(&mut self) {
        if let Some(viewport) = &self.viewport {
            viewport.destroy();
        }
    }
}

pub(crate) struct LinuxPlatformViews {
    x11: Option<X11Host>,
    wayland: Option<WaylandHost>,
    // The native hosts borrow Winit's parent/display and are dropped first.
    _window: Arc<Window>,
}

struct X11Host {
    connection: Arc<Connection>,
    root: XWindow,
    parent: XWindow,
    frame: XWindow,
    visual_id: u64,
    visual: *mut xlib::Visual,
    depth: i32,
    parent_visible: bool,
    entries: HashMap<u64, Entry>,
    overlays: Vec<X11Overlay>,
    compositor_available: bool,
    argb_visual: Option<X11Visual>,
    xfixes: Option<xfixes::Xlib>,
}

struct X11Overlay {
    window: XWindow,
    colormap: xlib::Colormap,
}

#[derive(Clone, Copy)]
struct X11Visual {
    visual: *mut xlib::Visual,
    visual_id: u64,
    depth: i32,
}

fn find_argb_visual(xlib: &xlib::Xlib, display: *mut Display, screen: i32) -> Option<X11Visual> {
    let render = xrender::Xrender::open().ok()?;
    let mut event_base = 0;
    let mut error_base = 0;
    if unsafe { (render.XRenderQueryExtension)(display, &mut event_base, &mut error_base) }
        == xlib::False
    {
        return None;
    }
    let mut template: xlib::XVisualInfo = unsafe { std::mem::zeroed() };
    template.screen = screen;
    template.depth = 32;
    template.class = xlib::TrueColor;
    let mut count = 0;
    let visuals = unsafe {
        (xlib.XGetVisualInfo)(
            display,
            xlib::VisualScreenMask | xlib::VisualDepthMask,
            &mut template,
            &mut count,
        )
    };
    if visuals.is_null() || count <= 0 {
        return None;
    }
    let candidates = unsafe { std::slice::from_raw_parts(visuals, count as usize) };
    let found = candidates.iter().find_map(|visual_info| {
        if visual_info.class != xlib::TrueColor {
            return None;
        }
        let format = unsafe { (render.XRenderFindVisualFormat)(display, visual_info.visual) };
        (!format.is_null() && unsafe { (*format).direct.alphaMask } != 0).then_some(X11Visual {
            visual: visual_info.visual,
            visual_id: visual_info.visualid,
            depth: visual_info.depth,
        })
    });
    unsafe { (xlib.XFree)(visuals.cast()) };
    found
}

impl LinuxPlatformViews {
    pub(crate) fn new(window: &Arc<Window>) -> Result<Self, String> {
        let handle = window.window_handle().map_err(|error| error.to_string())?;
        match (
            handle.as_raw(),
            window
                .display_handle()
                .map_err(|error| error.to_string())?
                .as_raw(),
        ) {
            (RawWindowHandle::Xlib(parent), _) => Ok(Self {
                x11: Some(X11Host::new(parent)?),
                wayland: None,
                _window: window.clone(),
            }),
            (RawWindowHandle::Wayland(parent), RawDisplayHandle::Wayland(display)) => Ok(Self {
                x11: None,
                wayland: Some(WaylandHost::new(display.display, parent.surface)?),
                _window: window.clone(),
            }),
            _ => Err("Linux platform views require an X11 or Wayland window".to_owned()),
        }
    }
}

impl X11Host {
    fn new(parent: winit::raw_window_handle::XlibWindowHandle) -> Result<Self, String> {
        let xlib = xlib::Xlib::open().map_err(|error| error.to_string())?;
        let display = NonNull::new(unsafe { (xlib.XOpenDisplay)(std::ptr::null()) })
            .ok_or_else(|| "XOpenDisplay failed".to_owned())?;
        let screen = unsafe { (xlib.XDefaultScreen)(display.as_ptr()) };
        let root = unsafe { (xlib.XRootWindow)(display.as_ptr(), screen) };
        let mut parent_attributes: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        if unsafe {
            (xlib.XGetWindowAttributes)(display.as_ptr(), parent.window, &mut parent_attributes)
        } == 0
        {
            return Err("Failed to read the X11 parent window visual".to_owned());
        }
        let mut frame = parent.window;
        loop {
            let mut tree_root = 0;
            let mut tree_parent = 0;
            let mut children = std::ptr::null_mut();
            let mut child_count = 0;
            let result = unsafe {
                (xlib.XQueryTree)(
                    display.as_ptr(),
                    frame,
                    &mut tree_root,
                    &mut tree_parent,
                    &mut children,
                    &mut child_count,
                )
            };
            if !children.is_null() {
                unsafe { (xlib.XFree)(children.cast()) };
            }
            if result == 0 || tree_parent == 0 {
                return Err("Failed to find the X11 top-level frame".to_owned());
            }
            if tree_parent == root {
                break;
            }
            frame = tree_parent;
        }
        let compositor_name =
            CString::new(format!("_NET_WM_CM_S{screen}")).map_err(|error| error.to_string())?;
        let compositor_atom =
            unsafe { (xlib.XInternAtom)(display.as_ptr(), compositor_name.as_ptr(), xlib::False) };
        let compositor_available = compositor_atom != 0
            && unsafe { (xlib.XGetSelectionOwner)(display.as_ptr(), compositor_atom) } != 0;
        let argb_visual = find_argb_visual(&xlib, display.as_ptr(), screen);
        let xfixes = xfixes::Xlib::open().ok().filter(|fixes| {
            let mut event_base = 0;
            let mut error_base = 0;
            let mut major_version = 0;
            let mut minor_version = 0;
            unsafe {
                (fixes.XFixesQueryExtension)(display.as_ptr(), &mut event_base, &mut error_base)
                    != xlib::False
                    && (fixes.XFixesQueryVersion)(
                        display.as_ptr(),
                        &mut major_version,
                        &mut minor_version,
                    ) != 0
                    && major_version >= 2
            }
        });
        Ok(Self {
            connection: Arc::new(Connection {
                xlib,
                display,
                screen,
            }),
            root,
            parent: parent.window,
            frame,
            visual_id: parent.visual_id,
            visual: parent_attributes.visual,
            depth: parent_attributes.depth,
            parent_visible: parent_attributes.map_state == xlib::IsViewable,
            entries: HashMap::new(),
            overlays: Vec::new(),
            compositor_available,
            argb_visual,
            xfixes,
        })
    }

    fn create_child(&self, parent: XWindow) -> XWindow {
        let connection = &self.connection;
        unsafe {
            (connection.xlib.XCreateSimpleWindow)(
                connection.display.as_ptr(),
                parent,
                0,
                0,
                1,
                1,
                0,
                0,
                0,
            )
        }
    }

    fn create_layer_window(&self) -> Result<(XWindow, xlib::Colormap), String> {
        let display = self.connection.display.as_ptr();
        let colormap = unsafe {
            (self.connection.xlib.XCreateColormap)(display, self.root, self.visual, xlib::AllocNone)
        };
        let mut attributes: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attributes.override_redirect = xlib::True;
        attributes.background_pixel = 0;
        attributes.border_pixel = 0;
        attributes.colormap = colormap;
        let window = unsafe {
            (self.connection.xlib.XCreateWindow)(
                display,
                self.root,
                0,
                0,
                1,
                1,
                0,
                self.depth,
                xlib::InputOutput as u32,
                self.visual,
                xlib::CWOverrideRedirect
                    | xlib::CWBackPixel
                    | xlib::CWBorderPixel
                    | xlib::CWColormap,
                &mut attributes,
            )
        };
        if window == 0 {
            unsafe { (self.connection.xlib.XFreeColormap)(display, colormap) };
            return Err("XCreateWindow failed for native platform-view layer".to_owned());
        }
        Ok((window, colormap))
    }

    fn parent_origin(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;
        let mut child = 0;
        unsafe {
            (self.connection.xlib.XTranslateCoordinates)(
                self.connection.display.as_ptr(),
                self.parent,
                self.root,
                0,
                0,
                &mut x,
                &mut y,
                &mut child,
            );
        }
        (x, y)
    }

    fn stack_above(&self, window: XWindow, sibling: XWindow) {
        let mut changes = xlib::XWindowChanges {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            sibling,
            stack_mode: xlib::Above,
        };
        unsafe {
            (self.connection.xlib.XConfigureWindow)(
                self.connection.display.as_ptr(),
                window,
                u32::from(xlib::CWSibling | xlib::CWStackMode),
                &mut changes,
            );
        }
    }

    pub(crate) fn register(
        &mut self,
        id: PlatformViewId,
        factory: Rc<dyn LinuxPlatformViewFactory>,
    ) -> Result<(), String> {
        if self.entries.contains_key(&id.as_raw()) {
            return Ok(());
        }
        let (container, colormap) = self.create_layer_window()?;
        let window = self.create_child(container);
        if window == 0 {
            unsafe {
                (self.connection.xlib.XDestroyWindow)(self.connection.display.as_ptr(), container)
            };
            unsafe {
                (self.connection.xlib.XFreeColormap)(self.connection.display.as_ptr(), colormap)
            };
            return Err("XCreateSimpleWindow failed".to_owned());
        }
        let factory_window = X11ChildWindow {
            display: self.connection.display.cast(),
            window,
            screen: self.connection.screen,
            visual_id: self.visual_id,
        };
        unsafe {
            (self.connection.xlib.XFlush)(self.connection.display.as_ptr());
        }
        factory.create(factory_window);
        self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                container,
                window,
                colormap,
            },
        );
        Ok(())
    }

    pub(crate) fn unregister(&mut self, id: PlatformViewId) {
        if let Some(entry) = self.entries.remove(&id.as_raw()) {
            let window = X11ChildWindow {
                display: self.connection.display.cast(),
                window: entry.window,
                screen: self.connection.screen,
                visual_id: self.visual_id,
            };
            entry.factory.release(window);
            unsafe {
                (self.connection.xlib.XDestroyWindow)(
                    self.connection.display.as_ptr(),
                    entry.container,
                );
                (self.connection.xlib.XFreeColormap)(
                    self.connection.display.as_ptr(),
                    entry.colormap,
                );
            }
        }
    }

    fn create_slice_window(
        &mut self,
        size: [u32; 2],
    ) -> Result<(RawDisplayHandle, RawWindowHandle), String> {
        if !self.compositor_available || self.xfixes.is_none() {
            return Err(
                "X11 overlay composition requires a compositing manager and XFixes input regions"
                    .to_owned(),
            );
        }
        let visual = self
            .argb_visual
            .ok_or("X11 server has no ARGB visual for a transparent overlay")?;
        let display = self.connection.display.as_ptr();
        let colormap = unsafe {
            (self.connection.xlib.XCreateColormap)(
                display,
                self.parent,
                visual.visual,
                xlib::AllocNone,
            )
        };
        let mut attributes: xlib::XSetWindowAttributes = unsafe { std::mem::zeroed() };
        attributes.background_pixel = 0;
        attributes.border_pixel = 0;
        attributes.colormap = colormap;
        attributes.override_redirect = xlib::True;
        let window = unsafe {
            (self.connection.xlib.XCreateWindow)(
                display,
                self.root,
                0,
                0,
                size[0].max(1),
                size[1].max(1),
                0,
                visual.depth,
                xlib::InputOutput as u32,
                visual.visual,
                xlib::CWBackPixel
                    | xlib::CWBorderPixel
                    | xlib::CWColormap
                    | xlib::CWOverrideRedirect,
                &mut attributes,
            )
        };
        if window == 0 {
            unsafe { (self.connection.xlib.XFreeColormap)(display, colormap) };
            return Err("XCreateWindow failed for transparent overlay".to_owned());
        }
        unsafe { (self.connection.xlib.XMapWindow)(display, window) };
        self.overlays.push(X11Overlay { window, colormap });
        self.set_overlay_input_region(window, Some(crate::PxRect::ZERO), [1, 1]);
        unsafe { (self.connection.xlib.XSync)(display, xlib::False) };
        let handles = X11ChildWindow {
            display: self.connection.display.cast(),
            window,
            screen: self.connection.screen,
            visual_id: visual.visual_id,
        }
        .raw_handles();
        Ok(handles)
    }

    fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        overlays_after: &[usize],
        overlay_input_regions: &[Option<crate::PxRect>],
        viewport: [u32; 2],
    ) {
        let display = self.connection.display.as_ptr();
        let (origin_x, origin_y) = self.parent_origin();
        let mut stack_sibling = self.frame;
        let visible_ids = placements
            .iter()
            .map(|placement| placement.id)
            .collect::<std::collections::HashSet<_>>();
        let window_bounds = FloatRect {
            x: 0.0,
            y: 0.0,
            width: viewport[0] as f32,
            height: viewport[1] as f32,
        };
        for (&id, entry) in &self.entries {
            if !visible_ids.contains(&id) {
                unsafe { (self.connection.xlib.XUnmapWindow)(display, entry.container) };
            }
        }
        let mut overlay_index = 0;
        for (index, placement) in placements.iter().enumerate() {
            if let Some(entry) = self.entries.get(&placement.id) {
                let bounds = placement.geometry.window_bounds();
                let visible = bounds
                    .intersection(window_bounds)
                    .and_then(|visible| {
                        placement
                            .clip_bounds
                            .map_or(Some(visible), |clip| visible.intersection(clip.into()))
                    })
                    .filter(|rect| {
                        placement.opacity == 1.0
                            && !placement.unsupported_shape_clip
                            && placement.geometry.local_to_window.scale() == 1.0
                            && rect.width == bounds.width
                            && rect.height == bounds.height
                            && rect.width > 0.0
                            && rect.height > 0.0
                    });
                if let Some(rect) = visible {
                    let x = rect.x.round() as i32;
                    let y = rect.y.round() as i32;
                    let width = rect.width.round().max(1.0) as u32;
                    let height = rect.height.round().max(1.0) as u32;
                    unsafe {
                        (self.connection.xlib.XMapWindow)(display, entry.container);
                        (self.connection.xlib.XMoveResizeWindow)(
                            display,
                            entry.container,
                            origin_x.saturating_add(x),
                            origin_y.saturating_add(y),
                            width,
                            height,
                        );
                        (self.connection.xlib.XMoveResizeWindow)(
                            display,
                            entry.window,
                            (bounds.x - rect.x).round() as i32,
                            (bounds.y - rect.y).round() as i32,
                            bounds.width.round().max(1.0) as u32,
                            bounds.height.round().max(1.0) as u32,
                        );
                        (self.connection.xlib.XMapWindow)(display, entry.window);
                    }
                    self.stack_above(entry.container, stack_sibling);
                    stack_sibling = entry.container;
                    entry.factory.update(X11ChildWindow {
                        display: self.connection.display.cast(),
                        window: entry.window,
                        screen: self.connection.screen,
                        visual_id: self.visual_id,
                    });
                } else {
                    unsafe { (self.connection.xlib.XUnmapWindow)(display, entry.container) };
                }
            }
            if overlays_after.get(overlay_index) == Some(&index) {
                if let Some(overlay) = self.overlays.get(overlay_index) {
                    unsafe {
                        (self.connection.xlib.XMapWindow)(display, overlay.window);
                        (self.connection.xlib.XMoveResizeWindow)(
                            display,
                            overlay.window,
                            origin_x,
                            origin_y,
                            viewport[0].max(1),
                            viewport[1].max(1),
                        );
                        self.set_overlay_input_region(
                            overlay.window,
                            overlay_input_regions.get(overlay_index).copied().flatten(),
                            viewport,
                        );
                    }
                    self.stack_above(overlay.window, stack_sibling);
                    stack_sibling = overlay.window;
                }
                overlay_index += 1;
            }
        }
        for (index, overlay) in self.overlays.iter().enumerate() {
            if index >= overlays_after.len() {
                unsafe { (self.connection.xlib.XUnmapWindow)(display, overlay.window) };
            }
        }
        unsafe { (self.connection.xlib.XFlush)(display) };
    }

    fn set_overlay_input_region(
        &self,
        window: XWindow,
        bounds: Option<crate::PxRect>,
        viewport: [u32; 2],
    ) {
        let Some(xfixes) = &self.xfixes else {
            return;
        };
        let input_bounds = bounds.unwrap_or(crate::PxRect::new(
            crate::Px::ZERO,
            crate::Px::ZERO,
            crate::Px(viewport[0] as i32),
            crate::Px(viewport[1] as i32),
        ));
        let input_rectangle = x11_rectangle(input_bounds, viewport);
        let bounding_rectangle = bounds.and_then(|bounds| x11_rectangle(bounds, viewport));
        let mut input_rectangles = input_rectangle.into_iter().collect::<Vec<_>>();
        let mut bounding_rectangles = bounding_rectangle.into_iter().collect::<Vec<_>>();
        let display = self.connection.display.as_ptr();
        let input_region = unsafe {
            (xfixes.XFixesCreateRegion)(
                display,
                input_rectangles.as_mut_ptr(),
                input_rectangles.len() as i32,
            )
        };
        let bounding_region = unsafe {
            (xfixes.XFixesCreateRegion)(
                display,
                bounding_rectangles.as_mut_ptr(),
                bounding_rectangles.len() as i32,
            )
        };
        unsafe {
            (xfixes.XFixesSetWindowShapeRegion)(display, window, 2, 0, 0, input_region);
            (xfixes.XFixesSetWindowShapeRegion)(display, window, 0, 0, 0, bounding_region);
            (xfixes.XFixesDestroyRegion)(display, input_region);
            (xfixes.XFixesDestroyRegion)(display, bounding_region);
        }
    }

    fn sync_parent_visibility(&mut self) -> bool {
        let display = self.connection.display.as_ptr();
        let mut attributes: xlib::XWindowAttributes = unsafe { std::mem::zeroed() };
        if unsafe {
            (self.connection.xlib.XGetWindowAttributes)(display, self.parent, &mut attributes)
        } == 0
        {
            return false;
        }
        let visible = attributes.map_state == xlib::IsViewable;
        if visible == self.parent_visible {
            return false;
        }
        self.parent_visible = visible;
        if !visible {
            for entry in self.entries.values() {
                unsafe { (self.connection.xlib.XUnmapWindow)(display, entry.container) };
            }
            for overlay in &self.overlays {
                unsafe { (self.connection.xlib.XUnmapWindow)(display, overlay.window) };
            }
            unsafe { (self.connection.xlib.XFlush)(display) };
        }
        visible
    }
}

fn x11_rectangle(bounds: crate::PxRect, viewport: [u32; 2]) -> Option<xlib::XRectangle> {
    if bounds.width <= crate::Px::ZERO || bounds.height <= crate::Px::ZERO {
        return None;
    }
    let left = bounds.x.0.max(0).min(viewport[0] as i32);
    let top = bounds.y.0.max(0).min(viewport[1] as i32);
    let right = bounds
        .x
        .0
        .saturating_add(bounds.width.0)
        .max(0)
        .min(viewport[0] as i32);
    let bottom = bounds
        .y
        .0
        .saturating_add(bounds.height.0)
        .max(0)
        .min(viewport[1] as i32);
    let rectangle = xlib::XRectangle {
        x: left.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        y: top.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        width: right.saturating_sub(left).clamp(0, u16::MAX as i32) as u16,
        height: bottom.saturating_sub(top).clamp(0, u16::MAX as i32) as u16,
    };
    (rectangle.width > 0 && rectangle.height > 0).then_some(rectangle)
}

impl WaylandHost {
    fn new(display: NonNull<c_void>, root_surface: NonNull<c_void>) -> Result<Self, String> {
        // SAFETY: LinuxPlatformViews retains the window and its display until this host drops.
        let backend = unsafe { Backend::from_foreign_display(display.as_ptr().cast()) };
        let connection = WaylandConnection::from_backend(backend);
        // SAFETY: Winit supplied a live wl_surface from that same display.
        let root_id = unsafe {
            ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                root_surface.as_ptr().cast(),
            )
        }
        .map_err(|error| format!("Invalid Winit wl_surface handle: {error:?}"))?;
        let root = wl_surface::WlSurface::from_id(&connection, root_id)
            .map_err(|error| format!("Failed to wrap Winit wl_surface: {error:?}"))?;
        let (globals, queue) = registry_queue_init::<WaylandQueueState>(&connection)
            .map_err(|error| error.to_string())?;
        let queue_handle = queue.handle();
        let subcompositor = globals
            .bind(&queue_handle, 1..=1, ())
            .map_err(|error| format!("Wayland wl_subcompositor unavailable: {error:?}"))?;
        let compositor = globals
            .bind(&queue_handle, 3..=6, ())
            .map_err(|error| format!("Wayland wl_compositor unavailable: {error:?}"))?;
        let viewporter = globals
            .bind::<wp_viewporter::WpViewporter, _, _>(&queue_handle, 1..=1, ())
            .ok();
        Ok(Self {
            root,
            compositor,
            subcompositor,
            viewporter,
            queue,
            entries: HashMap::new(),
            slices: Vec::new(),
            connection,
        })
    }

    fn create_child(&self) -> WaylandChild {
        let queue = self.queue.handle();
        let surface = self.compositor.create_surface(&queue, ());
        let subsurface = self
            .subcompositor
            .get_subsurface(&surface, &self.root, &queue, ());
        WaylandChild {
            surface,
            subsurface,
        }
    }

    fn register(&mut self, id: u64, factory: Rc<dyn LinuxPlatformViewFactory>) {
        if self.entries.contains_key(&id) {
            return;
        }
        let child = self.create_child();
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(&child.surface, &self.queue.handle(), ()));
        factory.create_wayland(child.handle(self.display()));
        // A factory may have submitted buffer state during creation. Do not map it yet.
        child.unmap();
        self.entries.insert(
            id,
            WaylandEntry {
                factory,
                child,
                viewport,
                visible: false,
            },
        );
    }

    fn unregister(&mut self, id: u64) {
        if let Some(entry) = self.entries.remove(&id) {
            entry
                .factory
                .release_wayland(entry.child.handle(self.display()));
            let _ = self.connection.flush();
        }
    }

    fn create_slice_surface(&mut self) -> (RawDisplayHandle, RawWindowHandle) {
        let child = self.create_child();
        let queue = self.queue.handle();
        let empty = self.compositor.create_region(&queue, ());
        child.surface.set_input_region(Some(&empty));
        empty.destroy();
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(&child.surface, &queue, ()));
        let handles = child.handle(self.display()).raw_handles();
        self.slices.push(WaylandSlice {
            viewport,
            child,
            visible: false,
        });
        handles
    }

    fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        overlays_after: &[usize],
        physical_size: [u32; 2],
        scale_factor: f64,
    ) {
        if let Err(error) = self.queue.dispatch_pending(&mut WaylandQueueState) {
            tracing::warn!(%error, "Failed to dispatch Wayland composition events");
        }
        let window = FloatRect {
            x: 0.0,
            y: 0.0,
            width: physical_size[0] as f32,
            height: physical_size[1] as f32,
        };
        let display = self.display();
        let mut visible_ids = std::collections::HashSet::new();
        let mut previous = &self.root;
        let mut overlay_index = 0;
        for (index, placement) in placements.iter().enumerate() {
            if let Some(entry) = self.entries.get(&placement.id)
                && let Some(geometry) = wayland_view_placement(
                    placement,
                    window,
                    scale_factor,
                    !entry.visible,
                    entry.viewport.is_some(),
                )
                && visible_ids.insert(placement.id)
            {
                let origin = geometry.visible_bounds;
                let source_x = (origin.x - geometry.bounds.x) * geometry.local_size.width.0 as f32
                    / geometry.bounds.width.max(f32::EPSILON);
                let source_y = (origin.y - geometry.bounds.y) * geometry.local_size.height.0 as f32
                    / geometry.bounds.height.max(f32::EPSILON);
                let source_width = origin.width * geometry.local_size.width.0 as f32
                    / geometry.bounds.width.max(f32::EPSILON);
                let source_height = origin.height * geometry.local_size.height.0 as f32
                    / geometry.bounds.height.max(f32::EPSILON);
                if let Some(viewport) = &entry.viewport {
                    viewport.set_source(
                        f64::from(source_x),
                        f64::from(source_y),
                        f64::from(source_width),
                        f64::from(source_height),
                    );
                    viewport.set_destination(
                        (f64::from(origin.width) / scale_factor).round().max(1.0) as i32,
                        (f64::from(origin.height) / scale_factor).round().max(1.0) as i32,
                    );
                } else {
                    entry
                        .child
                        .surface
                        .set_buffer_scale(scale_factor.round() as i32);
                }
                entry.child.subsurface.set_position(
                    (f64::from(origin.x) / scale_factor).round() as i32,
                    (f64::from(origin.y) / scale_factor).round() as i32,
                );
                entry.child.subsurface.place_above(previous);
                previous = &entry.child.surface;
                entry
                    .factory
                    .update_wayland(entry.child.handle(display), geometry);
                entry.child.surface.commit();
            }
            if overlays_after.get(overlay_index) == Some(&index) {
                let child = &self.slices[overlay_index].child;
                child.subsurface.place_above(previous);
                previous = &child.surface;
                overlay_index += 1;
            }
        }
        for (&id, entry) in &mut self.entries {
            let visible = visible_ids.contains(&id);
            if entry.visible && !visible {
                entry.child.unmap();
            }
            if !visible && let Some(viewport) = &entry.viewport {
                viewport.set_source(-1.0, -1.0, -1.0, -1.0);
                viewport.set_destination(-1, -1);
            }
            entry.visible = visible;
        }
        for (index, slice) in self.slices.iter_mut().enumerate() {
            let visible = index < overlays_after.len();
            if slice.visible && !visible {
                slice.child.unmap();
            }
            slice.visible = visible;
        }
        if let Err(error) = self.connection.flush() {
            tracing::warn!(%error, "Failed to flush Wayland subsurface state");
        }
        // Subsurface state is synchronized with the root's WGPU presentation.
    }

    fn prepare_slices(&self, count: usize, size: [u32; 2], scale: f64) -> Result<(), String> {
        if count > 0 && self.viewporter.is_none() && integer_buffer_scale(size, scale).is_none() {
            return Err(
                "Wayland scene slices need wp_viewporter for this buffer size and scale".to_owned(),
            );
        }
        for slice in self.slices.iter().take(count) {
            if let Some(viewport) = &slice.viewport {
                let [width, height] =
                    size.map(|pixels| (f64::from(pixels) / scale).round().max(1.0) as i32);
                viewport.set_destination(width, height);
            } else if let Some(scale) = integer_buffer_scale(size, scale) {
                slice.child.surface.set_buffer_scale(scale);
            }
        }
        self.connection.flush().map_err(|error| error.to_string())?;
        Ok(())
    }

    fn display(&self) -> NonNull<c_void> {
        NonNull::new(self.connection.backend().display_ptr().cast())
            .expect("Wayland backend has a live display")
    }
}

fn integer_buffer_scale(size: [u32; 2], scale: f64) -> Option<i32> {
    let integer = scale as u32;
    (integer > 0
        && f64::from(integer) == scale
        && size.iter().all(|pixels| pixels.is_multiple_of(integer)))
    .then_some(integer as i32)
}

fn wayland_view_placement(
    placement: &PlatformViewPlacement,
    window: FloatRect,
    scale_factor: f64,
    needs_buffer: bool,
    has_viewport: bool,
) -> Option<WaylandViewPlacement> {
    if placement.opacity <= 0.0 || placement.unsupported_shape_clip {
        return None;
    }
    let bounds = placement.geometry.window_bounds();
    let visible = bounds.intersection(window)?;
    let visible_bounds = placement
        .clip_bounds
        .map_or(Some(visible), |clip| visible.intersection(clip.into()))?;
    if !has_viewport {
        let local_width = placement.geometry.local_size.width.0 as f32;
        let local_height = placement.geometry.local_size.height.0 as f32;
        let zoom = placement.geometry.local_to_window.scale();
        if zoom != 1.0
            || visible_bounds != bounds
            || bounds.width != local_width
            || bounds.height != local_height
            || integer_buffer_scale(
                [local_width.round() as u32, local_height.round() as u32],
                scale_factor,
            )
            .is_none()
        {
            return None;
        }
    }
    Some(WaylandViewPlacement {
        local_size: placement.geometry.local_size,
        bounds,
        visible_bounds,
        scale_factor,
        needs_buffer,
    })
}

impl WaylandChild {
    fn handle(&self, display: NonNull<c_void>) -> WaylandChildSurface {
        let surface = NonNull::new(self.surface.id().as_ptr().cast())
            .expect("Wayland surface proxy has a live object id");
        WaylandChildSurface { display, surface }
    }

    fn unmap(&self) {
        self.surface.attach(None, 0, 0);
        self.surface.commit();
    }
}

impl Drop for WaylandChild {
    fn drop(&mut self) {
        self.subsurface.destroy();
        self.surface.destroy();
    }
}

impl Drop for WaylandSlice {
    fn drop(&mut self) {
        if let Some(viewport) = &self.viewport {
            viewport.destroy();
        }
    }
}

impl LinuxPlatformViews {
    pub(crate) fn register(
        &mut self,
        id: PlatformViewId,
        factory: Rc<dyn LinuxPlatformViewFactory>,
    ) -> Result<(), String> {
        if let Some(host) = &mut self.x11 {
            host.register(id, factory)
        } else if let Some(host) = &mut self.wayland {
            host.register(id.as_raw(), factory);
            Ok(())
        } else {
            Err("Linux platform-view host is unavailable".to_owned())
        }
    }

    pub(crate) fn unregister(&mut self, id: PlatformViewId) {
        if let Some(host) = &mut self.x11 {
            host.unregister(id);
        } else if let Some(host) = &mut self.wayland {
            host.unregister(id.as_raw());
        }
    }

    pub(crate) fn create_slice_window(
        &mut self,
        size: [u32; 2],
    ) -> Result<(RawDisplayHandle, RawWindowHandle), String> {
        if let Some(host) = &mut self.x11 {
            host.create_slice_window(size)
        } else if let Some(host) = &mut self.wayland {
            Ok(host.create_slice_surface())
        } else {
            Err("Linux platform-view host is unavailable".to_owned())
        }
    }

    pub(crate) fn discard_last_slice(&mut self) {
        if let Some(host) = &mut self.wayland {
            host.slices.pop();
        } else if let Some(host) = &mut self.x11
            && let Some(overlay) = host.overlays.pop()
        {
            unsafe {
                (host.connection.xlib.XDestroyWindow)(
                    host.connection.display.as_ptr(),
                    overlay.window,
                );
                (host.connection.xlib.XFreeColormap)(
                    host.connection.display.as_ptr(),
                    overlay.colormap,
                );
            }
        }
    }

    pub(crate) fn prepare_slices(
        &self,
        count: usize,
        size: [u32; 2],
        scale: f64,
    ) -> Result<(), String> {
        if let Some(host) = &self.wayland {
            host.prepare_slices(count, size, scale)?;
        }
        Ok(())
    }

    pub(crate) fn apply_composition(
        &mut self,
        placements: &[PlatformViewPlacement],
        overlays_after: &[usize],
        overlay_input_regions: &[Option<crate::PxRect>],
        viewport: [u32; 2],
        scale_factor: f64,
    ) {
        if let Some(host) = &mut self.x11 {
            host.apply_composition(placements, overlays_after, overlay_input_regions, viewport);
        } else if let Some(host) = &mut self.wayland {
            host.apply_composition(placements, overlays_after, viewport, scale_factor);
        }
    }

    pub(crate) fn sync_parent_visibility(&mut self) -> bool {
        self.x11
            .as_mut()
            .is_some_and(X11Host::sync_parent_visibility)
    }
}

impl Drop for X11Host {
    fn drop(&mut self) {
        for (_, entry) in self.entries.drain() {
            let window = X11ChildWindow {
                display: self.connection.display.cast(),
                window: entry.window,
                screen: self.connection.screen,
                visual_id: self.visual_id,
            };
            entry.factory.release(window);
            unsafe {
                (self.connection.xlib.XDestroyWindow)(
                    self.connection.display.as_ptr(),
                    entry.container,
                );
                (self.connection.xlib.XFreeColormap)(
                    self.connection.display.as_ptr(),
                    entry.colormap,
                );
            }
        }
        for overlay in self.overlays.drain(..) {
            unsafe {
                (self.connection.xlib.XDestroyWindow)(
                    self.connection.display.as_ptr(),
                    overlay.window,
                );
                (self.connection.xlib.XFreeColormap)(
                    self.connection.display.as_ptr(),
                    overlay.colormap,
                );
            }
        }
    }
}

impl Drop for WaylandHost {
    fn drop(&mut self) {
        let display = self.display();
        for (_, entry) in self.entries.drain() {
            entry.factory.release_wayland(entry.child.handle(display));
        }
        self.slices.clear();
        if let Some(viewporter) = &self.viewporter {
            viewporter.destroy();
        }
        self.subcompositor.destroy();
        let _ = self.connection.flush();
    }
}
