//! Android platform helpers for Tessera apps.

#[cfg(target_os = "android")]
pub mod jni;
#[cfg(target_os = "android")]
/// Native Android view factories and compositor host integration.
pub mod platform_views;
pub(crate) mod system_back;

#[cfg(target_os = "android")]
pub use crate::{java_class, jni_bind};
#[cfg(target_os = "android")]
pub use jni::{
    ActivityRef, AndroidJniError, ContextRef, JavaClass, JavaObject, JniArg, JniReturn, activity,
    context,
};
