//! Native views in Tessera layouts.
//!
//! ## Usage
//!
//! Embed Android or iOS controls such as web views and maps.

#[cfg(any(target_os = "android", target_os = "ios", test))]
use std::num::NonZeroU64;

#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::PluginContext;
#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::remember;
#[cfg(any(target_os = "android", target_os = "ios"))]
use crate::{DrawModifierContent, DrawModifierContext, DrawModifierNode};
use crate::{Modifier, Stack, tessera};

#[cfg(target_os = "android")]
pub use crate::android::platform_views::PlatformViewFactory;
#[cfg(target_os = "ios")]
/// Factory callbacks for a UIKit view hosted in Tessera.
pub trait PlatformViewFactory: crate::ios::platform_views::IosPlatformViewFactory + Clone {}

#[cfg(target_os = "ios")]
impl<T: crate::ios::platform_views::IosPlatformViewFactory + Clone> PlatformViewFactory for T {}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
/// Factory marker accepted by `platform_view` on platforms without native-view support.
pub trait PlatformViewFactory: Clone + 'static {}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
impl<T: Clone + 'static> PlatformViewFactory for T {}

#[cfg(test)]
static NEXT_PLATFORM_VIEW_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[cfg(any(target_os = "android", target_os = "ios", test))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlatformViewId(NonZeroU64);

#[cfg(any(target_os = "android", target_os = "ios", test))]
impl PlatformViewId {
    #[cfg(any(target_os = "android", target_os = "ios"))]
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

#[cfg(any(target_os = "android", target_os = "ios"))]
struct PlatformViewRegistration {
    id: PlatformViewId,
    context: Option<PluginContext>,
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl PlatformViewRegistration {
    fn new<F: PlatformViewFactory>(factory: &F) -> Self {
        let id = PlatformViewId::from_instance_key(crate::runtime::current_instance_key());
        let Some(context) = PluginContext::current() else {
            return Self { id, context: None };
        };

        #[cfg(target_os = "android")]
        if let Err(error) = context.register_platform_view(id, factory.java_factory()) {
            tracing::warn!(%error, "Failed to register an Android platform view");
            return Self { id, context: None };
        }

        #[cfg(target_os = "ios")]
        context.register_platform_view(id, std::rc::Rc::new(factory.clone()));

        Self {
            id,
            context: Some(context),
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
impl Drop for PlatformViewRegistration {
    fn drop(&mut self) {
        let Some(context) = &self.context else {
            return;
        };
        if let Err(error) = context.unregister_platform_view(self.id) {
            tracing::warn!(%error, "Failed to unregister a platform view");
        }
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
#[derive(Clone, Copy, PartialEq)]
struct PlatformViewBoundary(PlatformViewId);

#[cfg(any(target_os = "android", target_os = "ios"))]
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

/// # platform_view
///
/// Places an Android or iOS native view in Tessera layout and paint order.
///
/// Native views are available only on iOS and Android API 30 or newer.
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
/// On other targets, or on Android below API 30, this component is an empty layout.
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
    #[cfg(any(target_os = "android", target_os = "ios"))]
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
