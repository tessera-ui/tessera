use std::{
    collections::HashMap,
    sync::{
        LazyLock,
        atomic::{AtomicI64, Ordering},
    },
};

use jni::{
    JNIEnv, JavaVM, NativeMethod,
    objects::{GlobalRef, JObject, JValue},
    sys::{jfloat, jint, jlong},
};
use parking_lot::Mutex;
use winit::platform::android::activity::AndroidApp;

use crate::{
    dispatcher::UiDispatcher,
    system_back::{BackSwipeEdge, SystemBackEvent},
};

static NEXT_ID: AtomicI64 = AtomicI64::new(1);
static DISPATCHERS: LazyLock<Mutex<HashMap<jlong, UiDispatcher>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) struct BackBridge {
    vm: JavaVM,
    activity: GlobalRef,
    id: jlong,
    enabled: bool,
}

impl BackBridge {
    pub(crate) fn new(app: &AndroidApp, dispatcher: UiDispatcher) -> Option<Self> {
        // SAFETY: Android owns the VM and keeps the Activity reference alive.
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.ok()?;
        let activity = {
            let mut env = vm.attach_current_thread().ok()?;
            let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
            let class = env.get_object_class(&activity).ok()?;
            if env
                .get_method_id(&class, "setTesseraBackHandler", "(J)V")
                .is_err()
            {
                // Custom and older host activities may not implement the bridge.
                let _ = env.exception_clear();
                return None;
            }
            if let Err(error) = env.register_native_methods(
                class,
                &[NativeMethod {
                    name: "dispatchTesseraBack".into(),
                    sig: "(JIFI)V".into(),
                    fn_ptr: dispatch as *mut _,
                }],
            ) {
                let _ = env.exception_clear();
                tracing::warn!(%error, "Failed to register the Android back callback");
                return None;
            }
            env.new_global_ref(activity).ok()?
        };
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        DISPATCHERS.lock().insert(id, dispatcher);
        Some(Self {
            vm,
            activity,
            id,
            enabled: false,
        })
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        let Ok(mut env) = self.vm.attach_current_thread() else {
            return;
        };
        let id = if enabled { self.id } else { 0 };
        if let Err(error) = env.call_method(
            self.activity.as_obj(),
            "setTesseraBackHandler",
            "(J)V",
            &[JValue::Long(id)],
        ) {
            let _ = env.exception_clear();
            tracing::warn!(%error, "Failed to update the Android back callback");
            return;
        }
        self.enabled = enabled;
    }
}

impl Drop for BackBridge {
    fn drop(&mut self) {
        DISPATCHERS.lock().remove(&self.id);
        self.set_enabled(false);
    }
}

extern "system" fn dispatch(
    _env: JNIEnv<'_>,
    _activity: JObject<'_>,
    id: jlong,
    phase: jint,
    progress: jfloat,
    edge: jint,
) {
    let edge = if edge == 0 {
        BackSwipeEdge::Left
    } else {
        BackSwipeEdge::Right
    };
    let event = match phase {
        0 => SystemBackEvent::Started(edge),
        1 => SystemBackEvent::Progress { progress, edge },
        2 => SystemBackEvent::Cancelled,
        3 => SystemBackEvent::Committed,
        _ => return,
    };
    let dispatcher = DISPATCHERS.lock().get(&id).cloned();
    if let Some(dispatcher) = dispatcher {
        dispatcher.dispatch(move || crate::system_back::dispatch(event));
    }
}
