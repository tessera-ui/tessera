//! Native views in Tessera layouts.
//!
//! ## Usage
//!
//! Embed platform-native controls such as web views and maps.

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm",
    test
))]
use std::num::NonZeroU64;

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
use crate::PluginContext;
#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
use crate::remember;
#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
use crate::{DrawModifierContent, DrawModifierContext, DrawModifierNode};
use crate::{Modifier, Stack, tessera};

#[cfg(target_os = "android")]
pub use crate::android::platform_views::PlatformViewFactory;
#[cfg(target_os = "ios")]
/// Factory callbacks for a UIKit view hosted in Tessera.
pub trait PlatformViewFactory: crate::ios::platform_views::IosPlatformViewFactory + Clone {}

#[cfg(target_os = "ios")]
impl<T: crate::ios::platform_views::IosPlatformViewFactory + Clone> PlatformViewFactory for T {}

#[cfg(target_os = "macos")]
/// Factory callbacks for an AppKit view hosted in Tessera.
pub trait PlatformViewFactory:
    crate::macos_platform_views::MacosPlatformViewFactory + Clone
{
}

#[cfg(target_os = "macos")]
pub use crate::macos_platform_views::MacosPlatformViewFactory;

#[cfg(target_os = "macos")]
impl<T: crate::macos_platform_views::MacosPlatformViewFactory + Clone> PlatformViewFactory for T {}

#[cfg(target_os = "windows")]
/// Factory callbacks for a DirectComposition visual hosted in Tessera.
pub trait PlatformViewFactory:
    crate::windows_platform_views::WindowsPlatformViewFactory + Clone
{
}

#[cfg(target_os = "windows")]
pub use crate::windows_platform_views::{NativeVisual, WindowsPlatformViewFactory};

#[cfg(target_os = "windows")]
impl<T: crate::windows_platform_views::WindowsPlatformViewFactory + Clone> PlatformViewFactory
    for T
{
}

#[cfg(target_os = "linux")]
/// Factory callbacks for an X11 child window hosted in Tessera.
pub trait PlatformViewFactory:
    crate::linux_platform_views::LinuxPlatformViewFactory + Clone
{
}

#[cfg(target_os = "linux")]
pub use crate::linux_platform_views::{
    LinuxPlatformViewFactory, WaylandChildSurface, X11ChildWindow,
};

#[cfg(target_os = "linux")]
impl<T: crate::linux_platform_views::LinuxPlatformViewFactory + Clone> PlatformViewFactory for T {}

#[cfg(target_family = "wasm")]
pub use crate::web_platform_views::WebPlatformViewFactory;

#[cfg(target_family = "wasm")]
/// Factory callbacks for a DOM element hosted in the Web renderer.
pub trait PlatformViewFactory: WebPlatformViewFactory + Clone {}

#[cfg(target_family = "wasm")]
impl<T: WebPlatformViewFactory + Clone> PlatformViewFactory for T {}

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
)))]
/// Factory marker accepted by `platform_view` on platforms without native-view support.
pub trait PlatformViewFactory: Clone + 'static {}

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
)))]
impl<T: Clone + 'static> PlatformViewFactory for T {}

#[cfg(test)]
static NEXT_PLATFORM_VIEW_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm",
    test
))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlatformViewId(NonZeroU64);

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm",
    test
))]
impl PlatformViewId {
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_family = "wasm"
    ))]
    pub(crate) fn from_instance_key(instance_key: u64) -> Self {
        Self(NonZeroU64::new(instance_key).unwrap_or(NonZeroU64::MAX))
    }

    #[cfg(test)]
    pub(crate) fn new() -> Self {
        let mut id = NEXT_PLATFORM_VIEW_ID.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let next = id
                .checked_add(1)
                .expect("platform view identity space exhausted");
            match NEXT_PLATFORM_VIEW_ID.compare_exchange_weak(
                id,
                next,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => id = actual,
            }
        }
        Self(NonZeroU64::new(id).expect("platform view identities start at one"))
    }

    pub(crate) const fn as_raw(self) -> u64 {
        self.0.get()
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
struct PlatformViewRegistration {
    id: PlatformViewId,
    context: Option<PluginContext>,
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
impl PlatformViewRegistration {
    fn new<F: PlatformViewFactory>(factory: &F) -> Self {
        let id = PlatformViewId::from_instance_key(crate::runtime::current_instance_key());
        let Some(context) = PluginContext::current() else {
            return Self { id, context: None };
        };

        #[cfg(target_os = "android")]
        {
            if let Err(error) = context.register_platform_view(id, factory.java_factory()) {
                tracing::warn!(%error, "Failed to register an Android platform view");
                return Self { id, context: None };
            }
        }

        #[cfg(target_os = "ios")]
        context.register_platform_view(id, std::rc::Rc::new(factory.clone()));

        #[cfg(target_os = "macos")]
        context.register_platform_view(id, std::rc::Rc::new(factory.clone()));

        #[cfg(target_os = "windows")]
        if let Err(error) = context.register_platform_view(id, std::rc::Rc::new(factory.clone())) {
            tracing::warn!(%error, "Failed to register a Windows platform view");
            return Self { id, context: None };
        }

        #[cfg(target_os = "linux")]
        if let Err(error) = context.register_platform_view(id, std::rc::Rc::new(factory.clone())) {
            tracing::warn!(%error, "Failed to register a Linux platform view");
            return Self { id, context: None };
        }

        #[cfg(target_family = "wasm")]
        if let Err(error) = context.register_platform_view(id, std::rc::Rc::new(factory.clone())) {
            tracing::warn!(%error, "Failed to register a Web platform view");
            return Self { id, context: None };
        }

        Self {
            id,
            context: Some(context),
        }
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
impl Drop for PlatformViewRegistration {
    fn drop(&mut self) {
        let Some(context) = &self.context else {
            return;
        };
        #[cfg(any(target_os = "android", target_os = "ios"))]
        if let Err(error) = context.unregister_platform_view(self.id) {
            tracing::warn!(%error, "Failed to unregister a platform view");
        }
        #[cfg(target_os = "macos")]
        context.unregister_platform_view(self.id);
        #[cfg(target_os = "windows")]
        context.unregister_platform_view(self.id);
        #[cfg(target_os = "linux")]
        context.unregister_platform_view(self.id);
        #[cfg(target_family = "wasm")]
        context.unregister_platform_view(self.id);
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
#[derive(Clone, Copy, PartialEq)]
struct PlatformViewBoundary(PlatformViewId);

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_family = "wasm"
))]
impl DrawModifierNode for PlatformViewBoundary {
    fn draw(
        &self,
        context: &mut DrawModifierContext<'_, '_>,
        content: &mut dyn DrawModifierContent,
    ) {
        context
            .render_input
            .metadata_mut()
            .fragment_mut()
            .push_platform_view(self.0.as_raw());
        content.draw(context.render_input);
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl crate::BuildModifierNode for PlatformViewBoundary {
    fn apply(&self, runtime: &mut crate::runtime::TesseraRuntime) {
        let target = crate::runtime::persistent_focus_target_for_current_instance((
            "__tessera_platform_view_target",
            self.0,
        ));
        runtime.register_current_platform_focus(target, self.0);
    }
}

#[cfg(target_os = "macos")]
impl crate::BuildModifierNode for PlatformViewBoundary {
    fn apply(&self, _runtime: &mut crate::runtime::TesseraRuntime) {}
}

#[cfg(target_os = "windows")]
impl crate::BuildModifierNode for PlatformViewBoundary {
    fn apply(&self, _runtime: &mut crate::runtime::TesseraRuntime) {}
}

#[cfg(target_os = "linux")]
impl crate::BuildModifierNode for PlatformViewBoundary {
    fn apply(&self, _runtime: &mut crate::runtime::TesseraRuntime) {}
}

#[cfg(target_family = "wasm")]
impl crate::BuildModifierNode for PlatformViewBoundary {
    fn apply(&self, _runtime: &mut crate::runtime::TesseraRuntime) {}
}

/// # platform_view
///
/// Places a native view in Tessera layout and paint order on supported platforms.
///
/// Native views are available on Android API 30 or newer, iOS, macOS, Windows,
/// and Linux on X11 or Wayland.
/// Tessera assigns the size; native intrinsic sizes do not participate in
/// measurement. Native views support translation, uniform scale, rectangular
/// ancestor clips, and opacity. On Android 11–14, fractional opacity is not
/// supported for independently composited `SurfaceView` pixels; use zero
/// opacity to hide the view or full opacity to show it.
///
/// Rotation and shear are unsupported. A shaped ancestor clip hides the view.
/// Tessera draw effects cannot sample native pixels. When an effect would
/// sample across a platform-view boundary, Tessera warns and renders that
/// frame without native views.
///
/// Tessera uses the first factory supplied to a component instance. Pass changing
/// data through shared state read by the factory's `update` callback. To create a
/// fresh view for a different item, wrap the call in [`key`](crate::key) with the
/// item's logical key. Removing the component releases the view; keeping it
/// composed but not placing it in layout hides it without discarding its state.
/// X11 child windows support translation and rectangular ancestor clipping, but
/// not scaling, fractional opacity, or shaped clipping. X11 overlays require an
/// ARGB visual, an active compositing manager, and XFixes input regions; otherwise
/// native views are hidden when a later Tessera draw needs an overlay. Wayland factories attach
/// content at the measured local pixel size to the supplied
/// `WaylandChildSurface`. Tessera applies output
/// scale, uniform placement, rectangular cropping, and scene order with
/// `wp_viewporter` when available; without it, non-integer output scale,
/// transforms, and clipping hide the view. A newly visible Wayland view must
/// submit a buffer when `WaylandViewPlacement::needs_buffer` is true. Fractional
/// opacity and shaped clipping are unsupported. Native pixels cannot be sampled
/// by Tessera draw effects. Sampling across a native-view boundary warns and
/// renders that frame without native views.
///
/// On Android below API 30, this component is an empty layout. Web uses DOM
/// elements interleaved with transparent WebGPU canvas slices.
///
/// ## Usage
///
/// Reserve a layout region for a platform-hosted control such as a web view or map.
///
/// ## Parameters
///
/// - `factory` — creates and manages the platform-native view
/// - `modifier` — layout, clipping, input, and semantics modifiers for the node
///
/// ## Examples
///
/// ```
/// use tessera_ui::{Modifier, PlatformViewFactory, platform_view};
///
/// #[tessera_ui::tessera]
/// fn content<F: PlatformViewFactory>(#[prop(skip_eq)] factory: F) {
///     platform_view(factory, Modifier::new());
/// }
/// ```
#[tessera]
pub fn platform_view<F: PlatformViewFactory>(#[prop(skip_eq)] factory: F, modifier: Modifier) {
    let _ = &factory;
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux",
        target_family = "wasm"
    ))]
    {
        let registration = remember(|| PlatformViewRegistration::new(&factory));
        if let Some(id) = registration.with(|entry| entry.context.as_ref().map(|_| entry.id)) {
            let modifier = Modifier::new()
                .push_build(PlatformViewBoundary(id))
                .then(modifier);
            crate::layout::layout(Stack)
                .draw(PlatformViewBoundary(id))
                .modifier(modifier);
            return;
        }
    }
    crate::layout::layout(Stack).modifier(modifier);
}
