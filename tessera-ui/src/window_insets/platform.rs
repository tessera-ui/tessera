use crate::{WindowInsetsController, winit::window::Window};

#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios"))]
use crate::Dp;

#[cfg(not(target_family = "wasm"))]
use crate::Insets;

#[cfg(target_family = "wasm")]
mod web;

pub(crate) fn refresh(window: &Window) {
    #[cfg(target_family = "wasm")]
    let values = web::read(window);
    #[cfg(target_os = "android")]
    let values = android(window);
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let values = apple(window);
    #[cfg(not(any(
        target_family = "wasm",
        target_os = "android",
        target_os = "macos",
        target_os = "ios"
    )))]
    let values = {
        let _ = window;
        None::<(Insets, Insets)>
    };
    if let Some((system, ime)) = values {
        WindowInsetsController::current().set(system, ime);
    }
}

#[cfg(target_os = "android")]
fn android(window: &Window) -> Option<(Insets, Insets)> {
    use crate::jni::{
        JavaVM,
        objects::{JObject, JValue},
    };
    use crate::winit::platform::android::WindowExtAndroid;

    let size = window.inner_size();
    let rect = window.content_rect();
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return None;
    }
    let scale = window.scale_factor() as f64;
    let dp = |value: i32| Dp(value.max(0) as f64 / scale);
    let available = Insets::new(
        dp(rect.left),
        dp(rect.top),
        dp(size.width as i32 - rect.right),
        dp(size.height as i32 - rect.bottom),
    );
    let context = crate::ndk_context::android_context();
    // SAFETY: Android owns the VM and activity for the active window lifecycle.
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let values = env.with_local_frame(16, |env| -> crate::jni::errors::Result<(Insets, Insets)> {
        // SAFETY: The context exposes a live, Android-owned global activity reference.
        let activity = unsafe { JObject::from_raw(context.context().cast()) };
        let mut read = |mask| -> crate::jni::errors::Result<Insets> {
            let value = env
                .call_method(
                    &activity,
                    "getWindowInsets",
                    "(I)Landroidx/core/graphics/Insets;",
                    &[JValue::Int(mask)],
                )?
                .l()?;
            if value.is_null() {
                return Ok(Insets::default());
            }
            let mut edge = |name, limit: Dp| -> crate::jni::errors::Result<Dp> {
                Ok(Dp(dp(env.get_field(&value, name, "I")?.i()?)
                    .0
                    .min(limit.0)))
            };
            Ok(Insets::new(
                edge("left", available.left)?,
                edge("top", available.top)?,
                edge("right", available.right)?,
                edge("bottom", available.bottom)?,
            ))
        };
        Ok((read(7 | 128)?, read(8)?))
    });
    if values.is_err() {
        let _ = env.exception_clear();
    }
    Some(values.unwrap_or((available, Insets::default())))
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn apple(window: &Window) -> Option<(Insets, Insets)> {
    use crate::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    #[cfg(target_os = "macos")]
    {
        if !objc2::available!(macos = 12.0) {
            return None;
        }
        let RawWindowHandle::AppKit(handle) = window.window_handle().ok()?.as_raw() else {
            return None;
        };
        // SAFETY: Winit owns the view and this query runs on the window thread.
        let view = unsafe { &*handle.ns_view.as_ptr().cast::<objc2_app_kit::NSView>() };
        let edge = view.safeAreaInsets();
        Some((
            Insets::new(
                Dp(edge.left as f64),
                Dp(edge.top as f64),
                Dp(edge.right as f64),
                Dp(edge.bottom as f64),
            ),
            Insets::default(),
        ))
    }
    #[cfg(target_os = "ios")]
    {
        let RawWindowHandle::UiKit(handle) = window.window_handle().ok()?.as_raw() else {
            return None;
        };
        // SAFETY: Winit owns the view and UIKit access occurs on the main thread.
        let view = unsafe { &*handle.ui_view.as_ptr().cast::<objc2_ui_kit::UIView>() };
        let edge = view.safeAreaInsets();
        let system = Insets::new(
            Dp(edge.left as f64),
            Dp(edge.top as f64),
            Dp(edge.right as f64),
            Dp(edge.bottom as f64),
        );
        let ime = if objc2::available!(ios = 15.0) {
            let frame = view.keyboardLayoutGuide().layoutFrame();
            let height = view.bounds().size.height - frame.origin.y;
            Insets::bottom(Dp(if height > edge.bottom {
                height as f64
            } else {
                0.0
            }))
        } else {
            Insets::default()
        };
        Some((system, ime))
    }
}
