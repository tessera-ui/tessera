//! Android platform helpers for Tessera apps.
//!
//! ## Usage
//!
//! Call Android APIs from Tessera plugins using JNI helpers.

#[cfg(target_os = "android")]
pub mod jni;

#[cfg(target_os = "android")]
pub use crate::{java_class, jni_bind};
#[cfg(target_os = "android")]
pub use jni::{
    ActivityRef, AndroidJniError, ContextRef, JavaClass, JavaObject, JniArg, JniReturn, activity,
    context,
};
