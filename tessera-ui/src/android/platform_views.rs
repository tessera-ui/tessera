use std::{
    collections::HashMap,
    ptr::NonNull,
    sync::{
        Arc, LazyLock, Weak,
        atomic::{AtomicBool, AtomicI64, Ordering},
    },
};

use jni::{
    JNIEnv, JavaVM, NativeMethod,
    objects::{GlobalRef, JLongArray, JObject, JValue},
    sys::jlong,
};
use parking_lot::Mutex;
use winit::{platform::android::activity::AndroidApp, window::Window};

use crate::{
    android::{activity, jni::map_jni_error},
    render_graph::{PlatformViewInputBlocker, PlatformViewPlacement},
};

mod bridge {
    use super::AndroidPlatformViewFactoryHandle;
    use crate::android::ActivityRef;

    crate::jni_bind! {
        class "com.tessera.TesseraGameActivity" as PlatformViewBridge {
            fn registerTesseraPlatformView(activity: ActivityRef, id: i64, factory: AndroidPlatformViewFactoryHandle) -> ();
            fn unregisterTesseraPlatformView(activity: ActivityRef, id: i64) -> ();
        }
    }
}

/// Java factory retained by the Android platform-view host.
pub trait PlatformViewFactory: Clone + 'static {
    /// Returns the Java object implementing `TesseraPlatformViewFactory`.
    fn java_factory(&self) -> AndroidPlatformViewFactoryHandle;
}

/// Global JNI reference to an Android native-view factory.
#[derive(Clone)]
pub struct AndroidPlatformViewFactoryHandle(GlobalRef);

impl AndroidPlatformViewFactoryHandle {
    /// Wraps a Java object implementing `TesseraPlatformViewFactory`.
    pub fn new(factory: GlobalRef) -> Self {
        Self(factory)
    }
}

impl PlatformViewFactory for AndroidPlatformViewFactoryHandle {
    fn java_factory(&self) -> AndroidPlatformViewFactoryHandle {
        self.clone()
    }
}

impl crate::android::JniArg for AndroidPlatformViewFactoryHandle {
    fn signature() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Lcom/tessera/TesseraPlatformViewFactory;")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut JNIEnv<'a>,
    ) -> Result<jni::objects::JValueOwned<'a>, crate::android::AndroidJniError> {
        env.new_local_ref(self.0.as_obj())
            .map(jni::objects::JValueOwned::from)
            .map_err(|error| map_jni_error(env, error))
    }
}

#[derive(Clone)]
pub(crate) struct PlatformViewHost(Arc<HostInner>);

static NEXT_HOST: AtomicI64 = AtomicI64::new(1);
static HOSTS: LazyLock<Mutex<HashMap<jlong, Weak<HostInner>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
const FRAME_METHOD: &str = "setTesseraPlatformViewFrames";
const FRAME_SIGNATURE: &str = "(JJ[J[F[F[Z[J[F[II)V";

struct HostInner {
    vm: JavaVM,
    activity: GlobalRef,
    id: jlong,
    embeds_native_windows: bool,
    frames: Mutex<FrameGate>,
    focus: Mutex<NativeFocus>,
    suspended: AtomicBool,
}

#[derive(Default)]
struct NativeFocus {
    window: Weak<Window>,
    pending: Option<FocusSnapshot>,
    observed: u64,
    requested: u64,
}

struct FocusSnapshot {
    focused: u64,
    rejected: u64,
    focusable: Vec<u64>,
}

#[derive(Default)]
struct FrameGate {
    serial: jlong,
    slot: usize,
    active: bool,
    pending: Option<(jlong, usize)>,
    redraw: Option<Weak<Window>>,
}

impl Drop for HostInner {
    fn drop(&mut self) {
        HOSTS.lock().remove(&self.id);
    }
}

extern "system" fn frame_committed(
    _env: JNIEnv<'_>,
    _activity: JObject<'_>,
    id: jlong,
    serial: jlong,
) {
    let host = HOSTS.lock().get(&id).and_then(Weak::upgrade);
    if let Some(host) = host {
        host.finish_frame(serial, true);
    }
}

extern "system" fn focus_updated(
    env: JNIEnv<'_>,
    _activity: JObject<'_>,
    id: jlong,
    focused: jlong,
    rejected: jlong,
    ids: JLongArray<'_>,
) {
    let Some(host) = HOSTS.lock().get(&id).and_then(Weak::upgrade) else {
        return;
    };
    let Ok(len) = env.get_array_length(&ids) else {
        return;
    };
    let mut focusable = vec![0; len as usize];
    if env.get_long_array_region(&ids, 0, &mut focusable).is_err() {
        return;
    }
    let window = {
        let mut state = host.focus.lock();
        state.pending = Some(FocusSnapshot {
            focused: focused as u64,
            rejected: rejected as u64,
            focusable: focusable.into_iter().map(|id| id as u64).collect(),
        });
        state.window.upgrade()
    };
    if let Some(window) = window {
        window.request_redraw();
    }
}

impl HostInner {
    fn finish_frame(&self, serial: jlong, committed: bool) {
        let redraw = {
            let mut frames = self.frames.lock();
            let Some((pending, slot)) = frames.pending else {
                return;
            };
            if pending != serial {
                return;
            }
            frames.pending = None;
            if committed {
                frames.slot = slot;
                frames.active = true;
            }
            frames.redraw.take()
        };
        if let Some(window) = redraw.and_then(|window| window.upgrade()) {
            window.request_redraw();
        }
    }
}

impl PlatformViewHost {
    pub(crate) fn new(app: &AndroidApp) -> Option<Self> {
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
        let class = env.get_object_class(&activity).ok()?;
        if env
            .get_method_id(&class, FRAME_METHOD, FRAME_SIGNATURE)
            .is_err()
        {
            let _ = env.exception_clear();
            tracing::warn!("Android Activity does not provide the Tessera platform-view host");
            return None;
        }
        if let Err(error) = env.register_native_methods(
            class,
            &[
                NativeMethod {
                    name: "dispatchTesseraPlatformViewFrameCommitted".into(),
                    sig: "(JJ)V".into(),
                    fn_ptr: frame_committed as *mut _,
                },
                NativeMethod {
                    name: "dispatchTesseraPlatformViewFocus".into(),
                    sig: "(JJJ[J)V".into(),
                    fn_ptr: focus_updated as *mut _,
                },
            ],
        ) {
            let _ = env.exception_clear();
            tracing::warn!(%error, "Failed to register the Android composition commit callback");
            return None;
        }
        let activity = env.new_global_ref(activity).ok()?;
        let supported = env
            .call_method(activity.as_obj(), "stagesTesseraSceneBuffers", "()Z", &[])
            .and_then(|value| value.z());
        if supported.is_err() {
            let _ = env.exception_clear();
        }
        if !supported.ok()? {
            return None;
        }
        drop(env);
        let id = NEXT_HOST.fetch_add(1, Ordering::Relaxed);
        let host = Arc::new(HostInner {
            vm,
            activity,
            id,
            embeds_native_windows: AndroidApp::sdk_version() >= 35,
            frames: Mutex::default(),
            focus: Mutex::default(),
            suspended: AtomicBool::new(false),
        });
        HOSTS.lock().insert(id, Arc::downgrade(&host));
        Some(Self(host))
    }

    pub(crate) fn has_composition(&self) -> bool {
        let frames = self.0.frames.lock();
        frames.active || frames.pending.is_some()
    }

    pub(crate) fn routes_focus_to_embedded_window(&self) -> bool {
        if !self.0.embeds_native_windows {
            return false;
        }
        let state = self.0.focus.lock();
        state.requested != 0 || state.observed != 0
    }

    pub(crate) fn next_slice_slot(&self, window: &Arc<Window>) -> Option<usize> {
        self.0.focus.lock().window = Arc::downgrade(window);
        let mut frames = self.0.frames.lock();
        if frames.pending.is_some() {
            frames.redraw = Some(Arc::downgrade(window));
            return None;
        }
        Some(1 - frames.slot)
    }

    pub(crate) fn receive_focus(&self, owner: &mut crate::focus::FocusOwner) {
        let (snapshot, gained, lost) = {
            let mut state = self.0.focus.lock();
            let Some(snapshot) = state.pending.take() else {
                return;
            };
            let previous = std::mem::replace(&mut state.observed, snapshot.focused);
            let changed = previous != snapshot.focused;
            let gained = if changed { snapshot.focused } else { 0 };
            let lost = if snapshot.rejected != 0 {
                snapshot.rejected
            } else if changed {
                previous
            } else {
                0
            };
            if changed || snapshot.rejected != 0 {
                state.requested = snapshot.focused;
            }
            (snapshot, gained, lost)
        };
        if self.0.embeds_native_windows && gained != 0 {
            // A touch can hand off window focus before the Java host reports
            // the focused editor. The report confirms input is still ours.
            owner.set_owner_focused(true);
        }
        owner.sync_native_focus(&snapshot.focusable, gained, lost);
    }

    pub(crate) fn send_focus(&self, target: Option<(u64, bool)>) {
        let (id, backwards) = target.unwrap_or_default();
        {
            let mut state = self.0.focus.lock();
            if state.requested == id || state.pending.is_some() {
                return;
            }
            state.requested = id;
        }
        let Ok(mut env) = self.0.vm.attach_current_thread() else {
            return;
        };
        if let Err(error) = env.call_method(
            self.0.activity.as_obj(),
            "requestTesseraPlatformViewFocus",
            "(JZ)V",
            &[JValue::Long(id as jlong), JValue::Bool(u8::from(backwards))],
        ) {
            let _ = env.exception_clear();
            tracing::warn!(%error, "Failed to request Android platform-view focus");
        }
    }

    pub(crate) fn update(
        &self,
        placements: &[PlatformViewPlacement],
        blockers: &[PlatformViewInputBlocker],
        overlays_after: &[usize],
        slice_slot: usize,
    ) {
        let Ok(mut env) = self.0.vm.attach_current_thread() else {
            return;
        };
        let ids: Vec<jlong> = placements.iter().map(|view| view.id as jlong).collect();
        let bounds: Vec<f32> = placements
            .iter()
            .flat_map(|view| {
                let rect = view.geometry.window_bounds();
                let crop = view
                    .clip_bounds
                    .map_or([0.0, 0.0, rect.width, rect.height], |clip| {
                        [
                            (clip.x.0 as f32 - rect.x).max(0.0),
                            (clip.y.0 as f32 - rect.y).max(0.0),
                            (clip.x.0 as f32 + clip.width.0 as f32 - rect.x).min(rect.width),
                            (clip.y.0 as f32 + clip.height.0 as f32 - rect.y).min(rect.height),
                        ]
                    });
                [
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    crop[0],
                    crop[1],
                    crop[2],
                    crop[3],
                    view.geometry.local_size.width.0 as f32,
                    view.geometry.local_size.height.0 as f32,
                    view.geometry.local_to_window.scale(),
                ]
            })
            .collect();
        let visible: Vec<u8> = placements
            .iter()
            .map(|view| {
                if view.unsupported_shape_clip {
                    tracing::warn!(
                        platform_view_id = view.id,
                        "Android native platform views do not support shaped clipping"
                    );
                }
                u8::from(
                    view.opacity > 0.0
                        && !view.unsupported_shape_clip
                        && view
                            .clip_bounds
                            .is_none_or(|clip| clip.width.0 > 0 && clip.height.0 > 0),
                )
            })
            .collect();
        let opacity: Vec<f32> = placements.iter().map(|view| view.opacity).collect();
        let blocker_ids: Vec<jlong> = blockers.iter().map(|blocker| blocker.id as jlong).collect();
        let blocker_bounds: Vec<f32> = blockers
            .iter()
            .flat_map(|blocker| {
                let rect = blocker.bounds;
                [
                    rect.x.0 as f32,
                    rect.y.0 as f32,
                    rect.width.0 as f32,
                    rect.height.0 as f32,
                ]
            })
            .collect();
        let overlays_after: Vec<i32> = overlays_after.iter().map(|&index| index as i32).collect();
        let serial = {
            let mut frames = self.0.frames.lock();
            debug_assert!(frames.pending.is_none());
            frames.serial += 1;
            let serial = frames.serial;
            frames.pending = Some((serial, slice_slot));
            serial
        };
        let result = env.with_local_frame(12, |env| -> jni::errors::Result<()> {
            let ids_array = env.new_long_array(ids.len() as i32)?;
            let bounds_array = env.new_float_array(bounds.len() as i32)?;
            let opacity_array = env.new_float_array(opacity.len() as i32)?;
            let visible_array = env.new_boolean_array(visible.len() as i32)?;
            let blocker_ids_array = env.new_long_array(blocker_ids.len() as i32)?;
            let blocker_bounds_array = env.new_float_array(blocker_bounds.len() as i32)?;
            let overlays_after_array = env.new_int_array(overlays_after.len() as i32)?;
            env.set_long_array_region(&ids_array, 0, &ids)?;
            env.set_float_array_region(&bounds_array, 0, &bounds)?;
            env.set_float_array_region(&opacity_array, 0, &opacity)?;
            env.set_boolean_array_region(&visible_array, 0, &visible)?;
            env.set_long_array_region(&blocker_ids_array, 0, &blocker_ids)?;
            env.set_float_array_region(&blocker_bounds_array, 0, &blocker_bounds)?;
            env.set_int_array_region(&overlays_after_array, 0, &overlays_after)?;
            env.call_method(
                self.0.activity.as_obj(),
                FRAME_METHOD,
                FRAME_SIGNATURE,
                &[
                    JValue::Long(self.0.id),
                    JValue::Long(serial),
                    JValue::Object(&ids_array),
                    JValue::Object(&bounds_array),
                    JValue::Object(&opacity_array),
                    JValue::Object(&visible_array),
                    JValue::Object(&blocker_ids_array),
                    JValue::Object(&blocker_bounds_array),
                    JValue::Object(&overlays_after_array),
                    JValue::Int(slice_slot as i32),
                ],
            )?;
            Ok(())
        });
        if let Err(error) = result {
            self.0.finish_frame(serial, false);
            if matches!(error, jni::errors::Error::JavaException) {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            tracing::warn!(%error, "Failed to submit Android platform-view frame");
        }
    }

    pub(crate) fn register_factory(
        &self,
        app: &AndroidApp,
        id: u64,
        factory: AndroidPlatformViewFactoryHandle,
    ) -> Result<(), AndroidPlatformViewError> {
        bridge::PlatformViewBridge::registerTesseraPlatformView(
            app,
            activity(app),
            id as jlong,
            factory,
        )
        .map_err(|error| AndroidPlatformViewError(error.to_string()))
    }

    pub(crate) fn unregister(
        &self,
        app: &AndroidApp,
        id: u64,
    ) -> Result<(), AndroidPlatformViewError> {
        if self.0.suspended.load(Ordering::Acquire) {
            return Ok(());
        }
        bridge::PlatformViewBridge::unregisterTesseraPlatformView(app, activity(app), id as jlong)
            .map_err(|error| AndroidPlatformViewError(error.to_string()))
    }

    pub(crate) fn suspend(&self) {
        self.0.suspended.store(true, Ordering::Release);
    }

    pub(crate) fn resume(&self) {
        self.0.suspended.store(false, Ordering::Release);
    }

    pub(crate) fn slice_surface(
        &self,
        index: i32,
        slot: i32,
        width: i32,
        height: i32,
    ) -> Option<AndroidSliceSurface> {
        let mut env = self.0.vm.attach_current_thread().ok()?;
        let surface = env.with_local_frame(8, |env| -> jni::errors::Result<_> {
            let value = env
                .call_method(
                    self.0.activity.as_obj(),
                    "getTesseraSliceSurface",
                    "(IIII)Landroid/view/Surface;",
                    &[
                        JValue::Int(index),
                        JValue::Int(slot),
                        JValue::Int(width),
                        JValue::Int(height),
                    ],
                )?
                .l()?;
            if value.is_null() {
                return Ok(None);
            }
            let surface = env.new_global_ref(value)?;
            let native_window = NonNull::new(unsafe {
                crate::ndk_sys::ANativeWindow_fromSurface(
                    env.get_native_interface(),
                    surface.as_obj().as_raw(),
                )
            });
            Ok(native_window.map(|native_window| AndroidSliceSurface {
                _surface: surface,
                native_window,
            }))
        });
        match surface {
            Ok(surface) => surface,
            Err(error) => {
                if matches!(error, jni::errors::Error::JavaException) {
                    let _ = env.exception_describe();
                    let _ = env.exception_clear();
                }
                tracing::warn!(%error, index, slot, "Failed to acquire Android slice surface");
                None
            }
        }
    }
}

/// Failure while registering or releasing an Android native platform view.
#[derive(Debug)]
pub(crate) struct AndroidPlatformViewError(String);

impl AndroidPlatformViewError {
    pub(crate) fn host_unavailable() -> Self {
        Self("Android native platform views require a compatible API 30 or newer host".into())
    }
}

impl std::fmt::Display for AndroidPlatformViewError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AndroidPlatformViewError {}

pub(crate) struct AndroidSliceSurface {
    _surface: GlobalRef,
    native_window: NonNull<crate::ndk_sys::ANativeWindow>,
}

impl AndroidSliceSurface {
    pub(crate) fn native_window(&self) -> NonNull<crate::ndk_sys::ANativeWindow> {
        self.native_window
    }

    pub(crate) fn raw_window_handle(&self) -> winit::raw_window_handle::RawWindowHandle {
        winit::raw_window_handle::RawWindowHandle::AndroidNdk(
            winit::raw_window_handle::AndroidNdkWindowHandle::new(self.native_window.cast()),
        )
    }
}

impl Drop for AndroidSliceSurface {
    fn drop(&mut self) {
        unsafe { crate::ndk_sys::ANativeWindow_release(self.native_window.as_ptr()) };
    }
}
