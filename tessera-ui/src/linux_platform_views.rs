//! X11 and Wayland subsurface composition for native views hosted in Tessera windows.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::{CString, c_void},
    io::Write,
    os::fd::{AsFd, FromRawFd},
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
        wl_buffer, wl_callback, wl_compositor, wl_region, wl_registry, wl_shm, wl_shm_pool,
        wl_subcompositor, wl_subsurface, wl_surface,
    },
};
use wayland_protocols::wp::viewporter::client::{wp_viewport, wp_viewporter};
use winit::{
    event::{ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, Touch, TouchPhase},
    keyboard::ModifiersState,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle},
    window::Window,
};
use x11_dl::{
    xfixes,
    xlib::{self, Display, Window as XWindow},
    xrender,
};

use crate::{
    FloatRect, ImeRequest, PlatformViewId, PxPosition, PxRect, PxSize,
    render_graph::{PlatformViewInputBlocker, PlatformViewPlacement},
};

/// Input delivered to an interactive Wayland platform view.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum LinuxViewInputEvent {
    /// Pointer position in view-local physical pixels.
    Move(PxPosition),
    /// Mouse button change in view-local physical pixels.
    Button {
        /// View-local position.
        position: PxPosition,
        /// Button that changed.
        button: MouseButton,
        /// New button state.
        state: ElementState,
    },
    /// Scroll input in view-local physical pixels.
    Wheel {
        /// View-local position.
        position: PxPosition,
        /// Platform scroll delta.
        delta: MouseScrollDelta,
    },
    /// Touch sample with its location converted to view-local pixels.
    Touch(Touch),
    /// Keyboard input for the view that last accepted a pointer press.
    Key(KeyEvent),
    /// Input method text for the focused view.
    Ime(Ime),
    /// Keyboard focus entered or left the view.
    Focused(bool),
    /// Pointer exited the view.
    Leave,
    /// Active input was interrupted by focus loss, hiding, or removal.
    Cancel,
}

/// Manages native content on the current Linux window backend.
pub trait LinuxPlatformViewFactory: 'static {
    /// Initializes native content in the supplied child window.
    fn create(&self, _window: X11ChildWindow) {}

    /// Updates the child after Tessera changes its placement.
    fn update(&self, _window: X11ChildWindow) {}

    /// Reports the current X11 view size in physical pixels, or `None` when hidden.
    fn place_x11(&self, _window: X11ChildWindow, _size: Option<PxSize>) {}

    /// Releases resources associated with the child window.
    fn release(&self, _window: X11ChildWindow) {}

    /// Initializes native content in the supplied Wayland subsurface.
    fn create_wayland(&self, _surface: WaylandChildSurface) {}

    /// Updates visible Wayland content. Attach and commit a buffer when the
    /// view first becomes visible or becomes visible again after being hidden.
    /// If a buffer becomes available asynchronously, request a window redraw
    /// and attach it during this callback to present it with the scene.
    fn update_wayland(&self, _surface: WaylandChildSurface, _placement: WaylandViewPlacement) {}

    /// Stops presenting content when this view becomes hidden.
    fn hide_wayland(&self, _surface: WaylandChildSurface) {}

    /// Reports the physical size of the buffer attached by `update_wayland`.
    /// By default, the buffer is assumed to match the measured size.
    fn wayland_buffer_size(&self, _surface: WaylandChildSurface) -> Option<PxSize> {
        None
    }

    /// Notifies the producer that the compositor released one of its buffers.
    fn wayland_buffer_released(&self, _surface: WaylandChildSurface, _buffer_id: u64) {}

    /// Notifies the producer that the compositor signaled frame readiness for a buffer.
    fn wayland_buffer_rendered(&self, _surface: WaylandChildSurface, _buffer_id: u64) {}

    /// Releases resources associated with the Wayland subsurface.
    fn release_wayland(&self, _surface: WaylandChildSurface) {}

    /// Enables Wayland input routing for this surface.
    fn accepts_wayland_input(&self) -> bool {
        false
    }

    /// Receives input for an interactive Wayland surface. Return `true` to
    /// consume a key event. IME events are exclusive to the active editor.
    fn input_wayland(
        &self,
        _surface: WaylandChildSurface,
        _event: LinuxViewInputEvent,
        _modifiers: ModifiersState,
    ) -> bool {
        false
    }

    /// Publishes the focused editor's IME state and caret in local pixels.
    /// Return `None` when no editable element has focus.
    fn wayland_ime_request(&self, _surface: WaylandChildSurface) -> Option<ImeRequest> {
        None
    }
}

/// A borrowed handle to a Wayland subsurface owned by Tessera.
///
/// The factory may attach buffers but must not destroy the surface, assign a
/// role, or use the handles after its `release_wayland` callback returns.
#[derive(Clone)]
pub struct WaylandChildSurface {
    display: NonNull<c_void>,
    context: Rc<WaylandSurfaceContext>,
}

struct WaylandSurfaceContext {
    surface: wl_surface::WlSurface,
    shm: wl_shm::WlShm,
    queue: QueueHandle<WaylandQueueState>,
    buffers: WaylandBuffers,
    view_id: u64,
    focus_request: WaylandFocusRequest,
}

type WaylandFocusRequest = Rc<Cell<Option<WaylandViewIdentity>>>;
type WaylandBuffers = Rc<RefCell<HashMap<WaylandSubmission, wl_buffer::WlBuffer>>>;

#[derive(Clone, PartialEq, Eq, Hash)]
struct WaylandViewIdentity {
    id: u64,
    surface: ObjectId,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct WaylandSubmission {
    view: WaylandViewIdentity,
    id: u64,
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
    /// The window's physical pixels per Wayland surface unit.
    pub scale_factor: f64,
    /// Whether the producer must submit a new buffer to map the view.
    pub needs_buffer: bool,
}

impl WaylandChildSurface {
    /// Returns the native `wl_display` pointer.
    pub fn display(&self) -> *mut c_void {
        self.display.as_ptr()
    }

    /// Returns the native `wl_surface` pointer.
    pub fn surface(&self) -> *mut c_void {
        self.context.surface.id().as_ptr().cast()
    }

    /// Requests keyboard focus for this input-enabled view. The host applies
    /// the request after composing the scene if the view is visible.
    /// Request a window redraw when calling this outside of a frame.
    pub fn request_focus(&self) {
        if self.context.surface.is_alive() {
            self.context.focus_request.set(Some(self.identity()));
        }
    }

    fn identity(&self) -> WaylandViewIdentity {
        WaylandViewIdentity {
            id: self.context.view_id,
            surface: self.context.surface.id(),
        }
    }

    /// Attaches ARGB8888 pixels to the subsurface and keeps them alive until
    /// the compositor releases the buffer.
    pub fn attach_shm_buffer(
        &self,
        buffer_id: u64,
        width: u32,
        height: u32,
        stride: u32,
        pixels: &[u8],
    ) -> Result<(), String> {
        if !self.context.surface.is_alive() {
            return Err("Wayland subsurface has been released".to_owned());
        }
        let expected = usize::try_from(stride)
            .ok()
            .and_then(|stride| stride.checked_mul(height as usize))
            .ok_or_else(|| "Wayland shared-memory buffer size overflow".to_owned())?;
        if width == 0 || height == 0 || stride < width.saturating_mul(4) || pixels.len() < expected
        {
            return Err("Invalid Wayland ARGB8888 shared-memory buffer".to_owned());
        }
        let size = i32::try_from(expected)
            .map_err(|_| "Wayland shared-memory buffer exceeds protocol limits".to_owned())?;
        if width > i32::MAX as u32 || height > i32::MAX as u32 || stride > i32::MAX as u32 {
            return Err(
                "Wayland shared-memory buffer dimensions exceed protocol limits".to_owned(),
            );
        }
        let key = WaylandSubmission {
            view: self.identity(),
            id: buffer_id,
        };
        if self.context.buffers.borrow().contains_key(&key) {
            return Err("Wayland buffer ID is already in use".to_owned());
        }
        let fd = create_wayland_memfd(&pixels[..expected])?;
        let pool = self
            .context
            .shm
            .create_pool(fd.as_fd(), size, &self.context.queue, ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Argb8888,
            &self.context.queue,
            key.clone(),
        );
        pool.destroy();
        self.context
            .buffers
            .borrow_mut()
            .insert(key.clone(), buffer.clone());
        self.context.surface.attach(Some(&buffer), 0, 0);
        self.context.surface.frame(&self.context.queue, key);
        if self.context.surface.version() >= 4 {
            self.context
                .surface
                .damage_buffer(0, 0, width as i32, height as i32);
        } else {
            self.context.surface.damage(0, 0, i32::MAX, i32::MAX);
        }
        Ok(())
    }

    /// Returns raw window handles suitable for creating a WGPU surface.
    pub fn raw_handles(&self) -> (RawDisplayHandle, RawWindowHandle) {
        (
            RawDisplayHandle::Wayland(winit::raw_window_handle::WaylandDisplayHandle::new(
                self.display,
            )),
            RawWindowHandle::Wayland(winit::raw_window_handle::WaylandWindowHandle::new(
                NonNull::new(self.context.surface.id().as_ptr().cast()).expect("live wl_surface"),
            )),
        )
    }
}

fn create_wayland_memfd(pixels: &[u8]) -> Result<std::fs::File, String> {
    let fd = unsafe { libc::memfd_create(c"tessera-platform-view".as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    file.write_all(pixels).map_err(|error| error.to_string())?;
    Ok(file)
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
    size: std::cell::Cell<Option<PxSize>>,
}

#[derive(Default)]
struct WaylandQueueState {
    buffers: WaylandBuffers,
    released: Vec<WaylandSubmission>,
    rendered: Vec<WaylandSubmission>,
}

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
impl Dispatch<wl_callback::WlCallback, WaylandSubmission> for WaylandQueueState {
    fn event(
        state: &mut Self,
        _proxy: &wl_callback::WlCallback,
        event: wl_callback::Event,
        data: &WaylandSubmission,
        _connection: &WaylandConnection,
        _queue: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.rendered.push(data.clone());
        }
    }
}
delegate_noop!(WaylandQueueState: ignore wl_shm::WlShm);
delegate_noop!(WaylandQueueState: ignore wl_shm_pool::WlShmPool);

impl Dispatch<wl_buffer::WlBuffer, WaylandSubmission> for WaylandQueueState {
    fn event(
        state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        event: wl_buffer::Event,
        data: &WaylandSubmission,
        _connection: &WaylandConnection,
        _queue: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            if let Some(buffer) = state.buffers.borrow_mut().remove(data) {
                buffer.destroy();
            }
            state.released.push(data.clone());
        }
    }
}
delegate_noop!(WaylandQueueState: wp_viewporter::WpViewporter);
delegate_noop!(WaylandQueueState: wp_viewport::WpViewport);

struct WaylandHost {
    root: wl_surface::WlSurface,
    compositor: wl_compositor::WlCompositor,
    subcompositor: wl_subcompositor::WlSubcompositor,
    shm: wl_shm::WlShm,
    viewporter: Option<wp_viewporter::WpViewporter>,
    queue: EventQueue<WaylandQueueState>,
    queue_state: WaylandQueueState,
    entries: HashMap<u64, WaylandEntry>,
    input: WaylandInput,
    focus_request: WaylandFocusRequest,
    slices: Vec<WaylandSlice>,
    connection: WaylandConnection,
}

struct WaylandEntry {
    factory: Rc<dyn LinuxPlatformViewFactory>,
    child: WaylandChild,
    viewport: Option<wp_viewport::WpViewport>,
    visible: bool,
    accepts_input: bool,
}

struct WaylandInputTarget {
    id: u64,
    coordinates: crate::LocalToWindow,
    visible_bounds: FloatRect,
    blockers: Vec<PxRect>,
    accepts_input: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaylandInputOwner {
    Tessera,
    View(u64),
    Cancelled,
}

impl WaylandInputOwner {
    fn view(self) -> Option<u64> {
        match self {
            Self::View(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Default)]
struct WaylandInput {
    targets: Vec<WaylandInputTarget>,
    position: Option<PxPosition>,
    hover: Option<u64>,
    owner: Option<WaylandInputOwner>,
    mouse_buttons: std::collections::HashSet<MouseButton>,
    touches: HashMap<u64, WaylandInputOwner>,
    keyboard: Option<u64>,
    modifiers: ModifiersState,
}

impl WaylandInput {
    fn hit(&self, point: PxPosition) -> WaylandInputOwner {
        for target in self.targets.iter().rev() {
            if target.visible_bounds.contains(point) {
                return (target.accepts_input
                    && !target.blockers.iter().any(|bounds| bounds.contains(point)))
                .then_some(target.id)
                .map_or(WaylandInputOwner::Tessera, WaylandInputOwner::View);
            }
        }
        WaylandInputOwner::Tessera
    }

    fn local(&self, id: u64, point: PxPosition) -> Option<PxPosition> {
        self.targets
            .iter()
            .find(|target| target.id == id)
            .map(|target| target.coordinates.inverse_point(point))
    }

    fn cancel_view(&mut self, id: u64) -> bool {
        self.targets.retain(|target| target.id != id);
        let mut cancelled = false;
        if self.owner == Some(WaylandInputOwner::View(id)) {
            self.owner = Some(WaylandInputOwner::Cancelled);
            cancelled = true;
        }
        for owner in self.touches.values_mut() {
            if *owner == WaylandInputOwner::View(id) {
                *owner = WaylandInputOwner::Cancelled;
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

struct WaylandChild {
    surface: wl_surface::WlSurface,
    subsurface: wl_subsurface::WlSubsurface,
    view_id: u64,
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
            (self.connection.xlib.XSync)(self.connection.display.as_ptr(), xlib::False);
        }
        factory.create(factory_window);
        self.entries.insert(
            id.as_raw(),
            Entry {
                factory,
                container,
                window,
                colormap,
                size: Default::default(),
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
                if entry.size.take().is_some() {
                    entry
                        .factory
                        .place_x11(self.child_handle(entry.window), None);
                }
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
                    let child = self.child_handle(entry.window);
                    let size = Some(placement.geometry.local_size);
                    if entry.size.replace(size) != size {
                        entry.factory.place_x11(child, size);
                    }
                    entry.factory.update(child);
                } else {
                    unsafe { (self.connection.xlib.XUnmapWindow)(display, entry.container) };
                    if entry.size.take().is_some() {
                        entry
                            .factory
                            .place_x11(self.child_handle(entry.window), None);
                    }
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

    fn child_handle(&self, window: XWindow) -> X11ChildWindow {
        X11ChildWindow {
            display: self.connection.display.cast(),
            window,
            screen: self.connection.screen,
            visual_id: self.visual_id,
        }
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
        let shm = globals
            .bind(&queue_handle, 1..=1, ())
            .map_err(|error| format!("Wayland wl_shm unavailable: {error:?}"))?;
        let viewporter = globals
            .bind::<wp_viewporter::WpViewporter, _, _>(&queue_handle, 1..=1, ())
            .ok();
        Ok(Self {
            root,
            compositor,
            subcompositor,
            shm,
            viewporter,
            queue,
            queue_state: WaylandQueueState::default(),
            focus_request: Rc::default(),
            entries: HashMap::new(),
            input: WaylandInput::default(),
            slices: Vec::new(),
            connection,
        })
    }

    fn create_child(&self, view_id: u64) -> WaylandChild {
        let queue = self.queue.handle();
        let surface = self.compositor.create_surface(&queue, ());
        let subsurface = self
            .subcompositor
            .get_subsurface(&surface, &self.root, &queue, ());
        WaylandChild {
            surface,
            subsurface,
            view_id,
        }
    }

    fn child_handle(&self, child: &WaylandChild) -> WaylandChildSurface {
        WaylandChildSurface {
            display: self.display(),
            context: Rc::new(WaylandSurfaceContext {
                surface: child.surface.clone(),
                shm: self.shm.clone(),
                queue: self.queue.handle(),
                buffers: Rc::clone(&self.queue_state.buffers),
                view_id: child.view_id,
                focus_request: self.focus_request.clone(),
            }),
        }
    }

    fn register(&mut self, id: u64, factory: Rc<dyn LinuxPlatformViewFactory>) {
        if self.entries.contains_key(&id) {
            return;
        }
        let child = self.create_child(id);
        let accepts_input = factory.accepts_wayland_input();
        if accepts_input {
            let empty = self.compositor.create_region(&self.queue.handle(), ());
            child.surface.set_input_region(Some(&empty));
            empty.destroy();
        }
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(&child.surface, &self.queue.handle(), ()));
        factory.create_wayland(self.child_handle(&child));
        // A factory may have submitted buffer state during creation. Do not map it yet.
        child.unmap();
        self.entries.insert(
            id,
            WaylandEntry {
                factory,
                child,
                viewport,
                visible: false,
                accepts_input,
            },
        );
    }

    fn unregister(&mut self, id: u64) {
        if self.input.keyboard == Some(id) {
            self.focus(None);
        }
        let cancelled = self.input.cancel_view(id);
        if let Some(entry) = self.entries.remove(&id) {
            if cancelled {
                entry.factory.input_wayland(
                    self.child_handle(&entry.child),
                    LinuxViewInputEvent::Cancel,
                    self.input.modifiers,
                );
            }
            entry
                .factory
                .release_wayland(self.child_handle(&entry.child));
            drop(entry);
            let _ = self.connection.flush();
        }
    }

    fn create_slice_surface(&mut self) -> (RawDisplayHandle, RawWindowHandle) {
        let child = self.create_child(0);
        let queue = self.queue.handle();
        let empty = self.compositor.create_region(&queue, ());
        child.surface.set_input_region(Some(&empty));
        empty.destroy();
        let viewport = self
            .viewporter
            .as_ref()
            .map(|viewporter| viewporter.get_viewport(&child.surface, &queue, ()));
        let handles = self.child_handle(&child).raw_handles();
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
        blockers: &[PlatformViewInputBlocker],
        overlays_after: &[usize],
        physical_size: [u32; 2],
        scale_factor: f64,
    ) {
        if let Err(error) = self.queue.dispatch_pending(&mut self.queue_state) {
            tracing::warn!(%error, "Failed to dispatch Wayland composition events");
        }
        let rendered = std::mem::take(&mut self.queue_state.rendered);
        for submission in rendered {
            if let Some(entry) = self.live_entry(&submission.view) {
                entry
                    .factory
                    .wayland_buffer_rendered(self.child_handle(&entry.child), submission.id);
            }
        }
        let released = std::mem::take(&mut self.queue_state.released);
        for submission in released {
            if let Some(entry) = self.live_entry(&submission.view) {
                entry
                    .factory
                    .wayland_buffer_released(self.child_handle(&entry.child), submission.id);
            }
        }
        let window = FloatRect {
            x: 0.0,
            y: 0.0,
            width: physical_size[0] as f32,
            height: physical_size[1] as f32,
        };
        let mut visible_ids = std::collections::HashSet::new();
        let mut input_targets = Vec::new();
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
                input_targets.push(WaylandInputTarget {
                    id: placement.id,
                    coordinates: placement.geometry.local_to_window,
                    visible_bounds: geometry.visible_bounds,
                    blockers: blockers
                        .iter()
                        .filter(|blocker| blocker.id == placement.id)
                        .map(|blocker| blocker.bounds)
                        .collect(),
                    accepts_input: entry.accepts_input,
                });
                let child = self.child_handle(&entry.child);
                if entry.viewport.is_none() {
                    entry
                        .child
                        .surface
                        .set_buffer_scale(scale_factor.round() as i32);
                }
                entry.factory.update_wayland(child.clone(), geometry);
                let origin = geometry.visible_bounds;
                if let Some(viewport) = &entry.viewport {
                    let size = entry
                        .factory
                        .wayland_buffer_size(child)
                        .filter(|size| size.width.0 > 0 && size.height.0 > 0)
                        .unwrap_or(geometry.local_size);
                    let [x, y, width, height] = wayland_buffer_crop(geometry, size);
                    viewport.set_source(x, y, width, height);
                    viewport.set_destination(
                        (f64::from(origin.width) / scale_factor).round().max(1.0) as i32,
                        (f64::from(origin.height) / scale_factor).round().max(1.0) as i32,
                    );
                }
                entry.child.subsurface.set_position(
                    (f64::from(origin.x) / scale_factor).round() as i32,
                    (f64::from(origin.y) / scale_factor).round() as i32,
                );
                entry.child.subsurface.place_above(previous);
                previous = &entry.child.surface;
                entry.child.surface.commit();
            }
            if overlays_after.get(overlay_index) == Some(&index) {
                let child = &self.slices[overlay_index].child;
                child.subsurface.place_above(previous);
                previous = &child.surface;
                overlay_index += 1;
            }
        }
        let mut hidden = Vec::new();
        for (&id, entry) in &self.entries {
            let visible = visible_ids.contains(&id);
            if entry.visible && !visible {
                entry.factory.hide_wayland(self.child_handle(&entry.child));
                entry.child.unmap();
                hidden.push(id);
            }
            if !visible && let Some(viewport) = &entry.viewport {
                viewport.set_source(-1.0, -1.0, -1.0, -1.0);
                viewport.set_destination(-1, -1);
            }
        }
        for (&id, entry) in &mut self.entries {
            entry.visible = visible_ids.contains(&id);
        }
        self.input.targets = input_targets;
        for id in hidden {
            if self.input.keyboard == Some(id) {
                self.focus(None);
            }
            if self.input.cancel_view(id) {
                self.dispatch(id, LinuxViewInputEvent::Cancel);
            }
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

    fn live_entry(&self, view: &WaylandViewIdentity) -> Option<&WaylandEntry> {
        self.entries
            .get(&view.id)
            .filter(|entry| entry.child.surface.id() == view.surface)
    }

    fn dispatch(&self, id: u64, event: LinuxViewInputEvent) -> bool {
        self.entries.get(&id).is_some_and(|entry| {
            entry.factory.input_wayland(
                self.child_handle(&entry.child),
                event,
                self.input.modifiers,
            )
        })
    }

    fn focus(&mut self, next: Option<u64>) {
        if next == self.input.keyboard {
            return;
        }
        if let Some(previous) = self.input.keyboard.take() {
            self.dispatch(previous, LinuxViewInputEvent::Focused(false));
        }
        self.input.keyboard = next;
        if let Some(id) = next {
            self.dispatch(id, LinuxViewInputEvent::Focused(true));
        }
    }

    fn focused_ime_request(&self) -> Option<(u64, Option<ImeRequest>)> {
        let id = self.input.keyboard?;
        let entry = self.entries.get(&id)?;
        let coordinates = self
            .input
            .targets
            .iter()
            .find(|target| target.id == id)?
            .coordinates;
        let request = entry
            .factory
            .wayland_ime_request(self.child_handle(&entry.child))
            .map(|mut request| {
                request.position = Some(coordinates.map_point(request.local_position));
                let [width, height] = coordinates.map_size(request.size);
                request.size = PxSize::new(
                    crate::Px(width.ceil() as i32),
                    crate::Px(height.ceil() as i32),
                );
                request
            });
        Some((id, request))
    }

    fn route_pointer_move(&mut self, point: PxPosition) -> bool {
        self.input.position = Some(point);
        let owner = self.input.owner.unwrap_or_else(|| self.input.hit(point));
        let consumed = owner != WaylandInputOwner::Tessera;
        if self.input.owner.is_none() {
            let next = owner.view();
            if self.input.hover != next {
                if let Some(previous) = self.input.hover.take() {
                    self.dispatch(previous, LinuxViewInputEvent::Leave);
                }
                self.input.hover = next;
            }
        }
        if let Some(id) = owner.view()
            && let Some(local) = self.input.local(id, point)
        {
            self.dispatch(id, LinuxViewInputEvent::Move(local));
        }
        consumed
    }

    fn route_pointer_leave(&mut self) {
        if self.input.owner.is_none() {
            self.input.position = None;
            if let Some(id) = self.input.hover.take() {
                self.dispatch(id, LinuxViewInputEvent::Leave);
            }
        }
    }

    fn route_button(&mut self, button: MouseButton, state: ElementState) -> (bool, bool) {
        if state == ElementState::Pressed {
            if self.input.owner.is_none() {
                self.input.owner = Some(
                    self.input
                        .position
                        .map_or(WaylandInputOwner::Tessera, |point| self.input.hit(point)),
                );
            }
            self.input.mouse_buttons.insert(button);
        }
        if state == ElementState::Released && !self.input.mouse_buttons.remove(&button) {
            return (false, false);
        }
        let owner = self.input.owner.unwrap_or(WaylandInputOwner::Tessera);
        let native_press =
            state == ElementState::Pressed && button == MouseButton::Left && owner.view().is_some();
        if state == ElementState::Pressed && button == MouseButton::Left {
            self.focus(owner.view());
        }
        if let (Some(id), Some(position)) = (owner.view(), self.input.position)
            && let Some(local) = self.input.local(id, position)
        {
            self.dispatch(
                id,
                LinuxViewInputEvent::Button {
                    position: local,
                    button,
                    state,
                },
            );
        }
        if self.input.mouse_buttons.is_empty() {
            self.input.owner = None;
        }
        (owner != WaylandInputOwner::Tessera, native_press)
    }

    fn route_wheel(&self, delta: MouseScrollDelta) -> bool {
        let owner = self.input.owner.unwrap_or_else(|| {
            self.input
                .position
                .map_or(WaylandInputOwner::Tessera, |point| self.input.hit(point))
        });
        if let (Some(id), Some(position)) = (owner.view(), self.input.position)
            && let Some(local) = self.input.local(id, position)
        {
            self.dispatch(
                id,
                LinuxViewInputEvent::Wheel {
                    position: local,
                    delta,
                },
            );
        }
        owner != WaylandInputOwner::Tessera
    }

    fn route_touch(&mut self, touch: Touch) -> (bool, bool) {
        let point = PxPosition::from_f64_arr2([touch.location.x, touch.location.y]);
        let owner = match touch.phase {
            TouchPhase::Started => self.input.hit(point),
            _ => self
                .input
                .touches
                .get(&touch.id)
                .copied()
                .unwrap_or(WaylandInputOwner::Cancelled),
        };
        match touch.phase {
            TouchPhase::Started => {
                self.focus(owner.view());
                self.input.touches.insert(touch.id, owner);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.input.touches.remove(&touch.id);
            }
            TouchPhase::Moved => {}
        }
        if let Some(id) = owner.view()
            && let Some(local) = self.input.local(id, point)
        {
            let mut touch = touch;
            touch.location =
                winit::dpi::PhysicalPosition::new(f64::from(local.x), f64::from(local.y));
            self.dispatch(id, LinuxViewInputEvent::Touch(touch));
        }
        (
            owner != WaylandInputOwner::Tessera,
            owner.view().is_some() && touch.phase == TouchPhase::Started,
        )
    }

    fn route_key(&self, key: KeyEvent) -> bool {
        self.input
            .keyboard
            .is_some_and(|id| self.dispatch(id, LinuxViewInputEvent::Key(key)))
    }

    fn clear_input(&mut self) {
        let mut ids = self
            .input
            .touches
            .values()
            .filter_map(|owner| owner.view())
            .collect::<std::collections::HashSet<_>>();
        ids.extend(self.input.owner.and_then(WaylandInputOwner::view));
        ids.extend(self.input.hover.take());
        self.input.owner = None;
        self.focus(None);
        self.input.mouse_buttons.clear();
        self.input.touches.clear();
        self.input.position = None;
        for id in ids {
            self.dispatch(id, LinuxViewInputEvent::Cancel);
        }
    }
}

fn integer_buffer_scale(size: [u32; 2], scale: f64) -> Option<i32> {
    let integer = scale as u32;
    (integer > 0
        && f64::from(integer) == scale
        && size.iter().all(|pixels| pixels.is_multiple_of(integer)))
    .then_some(integer as i32)
}

fn wayland_buffer_crop(placement: WaylandViewPlacement, size: PxSize) -> [f64; 4] {
    let bounds = placement.bounds;
    let visible = placement.visible_bounds;
    let crop = |origin, extent, start, length, pixels| {
        let scale = f64::from(pixels) / f64::from(extent);
        (
            (f64::from(start) - f64::from(origin)) * scale,
            f64::from(length) * scale,
        )
    };
    let (x, width) = crop(
        bounds.x,
        bounds.width,
        visible.x,
        visible.width,
        size.width.0,
    );
    let (y, height) = crop(
        bounds.y,
        bounds.height,
        visible.y,
        visible.height,
        size.height.0,
    );
    [x, y, width, height]
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
        blockers: &[PlatformViewInputBlocker],
        overlays_after: &[usize],
        overlay_input_regions: &[Option<crate::PxRect>],
        viewport: [u32; 2],
        scale_factor: f64,
    ) {
        if let Some(host) = &mut self.x11 {
            host.apply_composition(placements, overlays_after, overlay_input_regions, viewport);
        } else if let Some(host) = &mut self.wayland {
            host.apply_composition(placements, blockers, overlays_after, viewport, scale_factor);
        }
    }

    pub(crate) fn set_modifiers(&mut self, modifiers: ModifiersState) {
        if let Some(host) = &mut self.wayland {
            host.input.modifiers = modifiers;
        }
    }

    pub(crate) fn route_pointer_move(&mut self, position: PxPosition) -> bool {
        if let Some(host) = &mut self.wayland {
            return host.route_pointer_move(position);
        }
        false
    }

    pub(crate) fn route_pointer_leave(&mut self) {
        if let Some(host) = &mut self.wayland {
            host.route_pointer_leave();
        }
    }

    pub(crate) fn route_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> (bool, bool) {
        if let Some(host) = &mut self.wayland {
            return host.route_button(button, state);
        }
        (false, false)
    }

    pub(crate) fn route_wheel(&self, delta: MouseScrollDelta) -> bool {
        self.wayland
            .as_ref()
            .is_some_and(|host| host.route_wheel(delta))
    }

    pub(crate) fn route_touch(&mut self, touch: Touch) -> (bool, bool) {
        if let Some(host) = &mut self.wayland {
            return host.route_touch(touch);
        }
        (false, false)
    }

    pub(crate) fn route_key(&self, key: KeyEvent) -> bool {
        self.wayland
            .as_ref()
            .is_some_and(|host| host.route_key(key))
    }

    pub(crate) fn route_ime(&self, id: u64, event: Ime) {
        if let Some(host) = &self.wayland
            && host.input.keyboard == Some(id)
        {
            host.dispatch(id, LinuxViewInputEvent::Ime(event));
        }
    }

    pub(crate) fn focused_ime_request(&self) -> Option<(u64, Option<ImeRequest>)> {
        self.wayland.as_ref()?.focused_ime_request()
    }

    pub(crate) fn apply_wayland_focus_request(&mut self) -> bool {
        let Some(host) = &mut self.wayland else {
            return false;
        };
        let Some(view) = host.focus_request.take() else {
            return false;
        };
        if !host
            .live_entry(&view)
            .is_some_and(|entry| entry.visible && entry.accepts_input)
        {
            return false;
        }
        host.focus(Some(view.id));
        true
    }

    pub(crate) fn blur_wayland(&mut self) {
        if let Some(host) = &mut self.wayland {
            host.focus(None);
        }
    }

    pub(crate) fn clear_input(&mut self) {
        if let Some(host) = &mut self.wayland {
            host.clear_input();
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
        self.clear_input();
        for entry in std::mem::take(&mut self.entries).into_values() {
            entry
                .factory
                .release_wayland(self.child_handle(&entry.child));
        }
        self.slices.clear();
        if let Some(viewporter) = &self.viewporter {
            viewporter.destroy();
        }
        self.subcompositor.destroy();
        let _ = self.connection.flush();
    }
}

#[cfg(test)]
mod tests {
    use crate::{LocalToWindow, Px};

    use super::*;

    fn target(id: u64, accepts_input: bool, blockers: Vec<PxRect>) -> WaylandInputTarget {
        WaylandInputTarget {
            id,
            coordinates: LocalToWindow::new(2.0, [10, 20].into())
                .expect("valid scale and translation"),
            visible_bounds: FloatRect {
                x: 10.0,
                y: 20.0,
                width: 80.0,
                height: 80.0,
            },
            blockers,
            accepts_input,
        }
    }

    #[test]
    fn wayland_clip_tracks_the_buffer_during_async_resize() {
        let placement = WaylandViewPlacement {
            local_size: PxSize::new(Px(200), Px(100)),
            bounds: FloatRect {
                x: 40.0,
                y: 20.0,
                width: 400.0,
                height: 200.0,
            },
            visible_bounds: FloatRect {
                x: 140.0,
                y: 70.0,
                width: 200.0,
                height: 100.0,
            },
            scale_factor: 1.25,
            needs_buffer: false,
        };
        assert_eq!(
            wayland_buffer_crop(placement, PxSize::new(Px(160), Px(80))),
            [40.0, 20.0, 80.0, 40.0],
        );
        assert_eq!(
            wayland_buffer_crop(placement, placement.local_size),
            [50.0, 25.0, 100.0, 50.0]
        );
    }

    #[test]
    fn wayland_hit_routing_respects_scene_order_overlay_blockers_and_local_scale() {
        let input = WaylandInput {
            targets: vec![
                target(1, true, Vec::new()),
                target(2, true, vec![PxRect::new(Px(30), Px(40), Px(10), Px(10))]),
            ],
            ..WaylandInput::default()
        };
        assert_eq!(input.hit([20, 30].into()), WaylandInputOwner::View(2));
        assert_eq!(input.local(2, [20, 30].into()), Some([5, 5].into()));
        assert_eq!(input.hit([35, 45].into()), WaylandInputOwner::Tessera);
        assert_eq!(input.hit([5, 5].into()), WaylandInputOwner::Tessera);

        let noninteractive = WaylandInput {
            targets: vec![target(3, false, Vec::new())],
            ..WaylandInput::default()
        };
        assert_eq!(
            noninteractive.hit([20, 30].into()),
            WaylandInputOwner::Tessera
        );
    }
}
