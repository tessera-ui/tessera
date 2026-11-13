use crate::{
    WindowInsetsController,
    winit::{event_loop::ActiveEventLoop, window::Window},
};

#[cfg(any(not(target_family = "wasm"), test))]
use crate::Insets;

#[cfg(any(target_os = "android", target_os = "macos", target_os = "ios", test))]
use crate::Dp;

#[cfg(target_os = "android")]
use crate::winit::platform::android::{ActiveEventLoopExtAndroid, activity::AndroidApp};

#[cfg(target_family = "wasm")]
mod web;

pub(crate) fn refresh(window: &Window, _event_loop: &ActiveEventLoop) {
    #[cfg(target_family = "wasm")]
    let values = web::read(window);
    #[cfg(target_os = "android")]
    let values = android(window, _event_loop.android_app());
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
fn android(window: &Window, app: &AndroidApp) -> Option<(Insets, Insets)> {
    use crate::jni::{
        JavaVM,
        objects::{JObject, JValue},
    };
    use crate::winit::platform::android::WindowExtAndroid;

    let rect = window.content_rect();
    if rect.right <= rect.left || rect.bottom <= rect.top {
        return None;
    }
    let scale = window.scale_factor() as f64;
    let surface = [rect.left, rect.top, rect.right, rect.bottom];
    // SAFETY: Android owns the VM and activity for the active window lifecycle.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let values = env.with_local_frame(
        16,
        |env| -> crate::jni::errors::Result<Option<(Insets, Insets)>> {
            // SAFETY: The active event loop keeps the activity's global reference alive.
            let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
            let window = env
                .call_method(&activity, "getWindow", "()Landroid/view/Window;", &[])?
                .l()?;
            let decor = env
                .call_method(&window, "getDecorView", "()Landroid/view/View;", &[])?
                .l()?;
            let root = [
                env.call_method(&decor, "getWidth", "()I", &[])?.i()?,
                env.call_method(&decor, "getHeight", "()I", &[])?.i()?,
            ];
            if root[0] <= 0 || root[1] <= 0 {
                return Ok(None);
            }
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
                let edges = [
                    env.get_field(&value, "left", "I")?.i()?,
                    env.get_field(&value, "top", "I")?.i()?,
                    env.get_field(&value, "right", "I")?.i()?,
                    env.get_field(&value, "bottom", "I")?.i()?,
                ];
                Ok(android_surface_insets(edges, surface, root, scale))
            };
            Ok(Some((read(7 | 128)?, read(8)?)))
        },
    );
    if values.is_err() {
        let _ = env.exception_clear();
    }
    values.ok().flatten()
}

#[cfg(any(target_os = "android", test))]
fn android_surface_insets(
    edges: [i32; 4],
    surface: [i32; 4],
    root: [i32; 2],
    scale: f64,
) -> Insets {
    // The host already accounts for the part of the window outside the surface.
    let occupied = [
        surface[0],
        surface[1],
        root[0] - surface[2],
        root[1] - surface[3],
    ];
    let edge = |index: usize| Dp((edges[index] - occupied[index].max(0)).max(0) as f64 / scale);
    Insets::new(edge(0), edge(1), edge(2), edge(3))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_insets_only_pad_occlusion_inside_the_render_surface() {
        for (edges, surface, root, expected) in [
            (
                [0, 66, 0, 66],
                [0, 0, 1080, 2400],
                [1080, 2400],
                Insets::vertical(Dp(24.0)),
            ),
            (
                [0, 66, 0, 66],
                [0, 66, 1080, 2334],
                [1080, 2400],
                Insets::default(),
            ),
            (
                [110, 0, 66, 55],
                [55, 0, 2378, 1080],
                [2400, 1080],
                Insets::new(Dp(20.0), Dp::ZERO, Dp(16.0), Dp(20.0)),
            ),
            (
                [0, 0, 0, 880],
                [0, 0, 1080, 1520],
                [1080, 2400],
                Insets::default(),
            ),
        ] {
            assert_eq!(android_surface_insets(edges, surface, root, 2.75), expected);
        }
    }
}
