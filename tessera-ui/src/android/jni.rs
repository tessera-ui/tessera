//! JNI helpers for Android plugin integration.
//!
//! ## Usage
//!
//! Bridge Tessera plugins to Android platform APIs with ergonomic JNI bindings.

use std::{borrow::Cow, fmt, marker::PhantomData};

use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jobject};
use winit::platform::android::activity::AndroidApp;

/// Hidden JNI re-exports for macro expansion.
#[doc(hidden)]
#[allow(missing_docs)]
pub mod internal {
    pub use jni::errors::Error as JniError;
    pub use jni::objects::{GlobalRef, JByteArray, JClass, JObject, JString, JValue, JValueOwned};
    pub use jni::{JNIEnv, JavaVM};
}

/// Errors returned by Android JNI helper calls.
#[derive(Debug)]
pub enum AndroidJniError {
    /// A JNI call failed.
    Jni(internal::JniError),
    /// A Java method returned null for a non-nullable type.
    NullReturn(&'static str),
    /// A required Java object was null.
    NullObject(&'static str),
}

impl fmt::Display for AndroidJniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jni(err) => write!(f, "JNI error: {err}"),
            Self::NullReturn(name) => write!(f, "Java returned null for {name}"),
            Self::NullObject(name) => write!(f, "Java object {name} was null"),
        }
    }
}

impl std::error::Error for AndroidJniError {}

impl From<internal::JniError> for AndroidJniError {
    fn from(err: internal::JniError) -> Self {
        Self::Jni(err)
    }
}

/// Hidden JNI error mapper used by generated bindings.
#[doc(hidden)]
pub fn map_jni_error(env: &mut internal::JNIEnv<'_>, err: internal::JniError) -> AndroidJniError {
    if matches!(err, internal::JniError::JavaException) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    AndroidJniError::Jni(err)
}

/// Converts Rust values into JNI arguments.
pub trait JniArg {
    /// Returns the JNI signature for this argument type.
    fn signature() -> Cow<'static, str>;
    /// Converts the argument into a JNI value.
    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError>;
}

/// Converts JNI return values into Rust values.
pub trait JniReturn: Sized {
    /// Returns the JNI signature for this return type.
    fn signature() -> Cow<'static, str>;
    /// Converts a JNI value into the Rust type.
    fn from_jvalue<'a>(
        env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError>;
}

/// Identifies a Java class by its fully qualified name.
pub trait JavaClass {
    /// Returns the Java class name in dot notation.
    fn name() -> &'static str;
    /// Returns the JNI signature for the class type.
    fn signature() -> Cow<'static, str> {
        Cow::Owned(format!("L{};", Self::name().replace('.', "/")))
    }
}

/// Owned reference to a Java object backed by a global reference.
#[derive(Clone, Debug)]
pub struct JavaObject<C: JavaClass> {
    inner: internal::GlobalRef,
    _marker: PhantomData<C>,
}

impl<C: JavaClass> JavaObject<C> {
    /// Creates a new Java object from a local reference.
    pub fn new(
        env: &mut internal::JNIEnv<'_>,
        object: internal::JObject<'_>,
    ) -> Result<Self, AndroidJniError> {
        if object.is_null() {
            return Err(AndroidJniError::NullObject("JavaObject"));
        }
        let global = env
            .new_global_ref(object)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(Self::from_global(global))
    }

    /// Creates a Java object from an existing global reference.
    pub fn from_global(inner: internal::GlobalRef) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying global reference as a JNI object.
    pub fn as_obj(&self) -> &internal::JObject<'static> {
        self.inner.as_obj()
    }
}

/// Reference to the Android Activity used for JNI calls.
#[derive(Clone, Copy, Debug)]
pub struct ActivityRef {
    raw: jobject,
}

/// Reference to the Android Context used for JNI calls.
#[derive(Clone, Copy, Debug)]
pub struct ContextRef {
    raw: jobject,
}

/// Returns an activity reference for JNI calls.
pub fn activity(android_app: &AndroidApp) -> ActivityRef {
    ActivityRef {
        raw: android_app.activity_as_ptr().cast(),
    }
}

/// Returns a context reference for JNI calls.
pub fn context(android_app: &AndroidApp) -> ContextRef {
    ContextRef {
        raw: android_app.activity_as_ptr().cast(),
    }
}

impl JniArg for bool {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Z")
    }

    fn to_jvalue<'a>(
        &self,
        _env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let value: jboolean = if *self { JNI_TRUE } else { JNI_FALSE };
        Ok(internal::JValueOwned::Bool(value))
    }
}

impl JniReturn for bool {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Z")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        Ok(value.z().map_err(AndroidJniError::from)?)
    }
}

impl JniArg for i32 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("I")
    }

    fn to_jvalue<'a>(
        &self,
        _env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        Ok(internal::JValueOwned::Int(*self))
    }
}

impl JniReturn for i32 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("I")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        Ok(value.i().map_err(AndroidJniError::from)?)
    }
}

impl JniArg for i64 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("J")
    }

    fn to_jvalue<'a>(
        &self,
        _env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        Ok(internal::JValueOwned::Long(*self))
    }
}

impl JniReturn for i64 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("J")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        Ok(value.j().map_err(AndroidJniError::from)?)
    }
}

impl JniArg for f32 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("F")
    }

    fn to_jvalue<'a>(
        &self,
        _env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        Ok(internal::JValueOwned::Float(*self))
    }
}

impl JniReturn for f32 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("F")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        Ok(value.f().map_err(AndroidJniError::from)?)
    }
}

impl JniArg for f64 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("D")
    }

    fn to_jvalue<'a>(
        &self,
        _env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        Ok(internal::JValueOwned::Double(*self))
    }
}

impl JniReturn for f64 {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("D")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        Ok(value.d().map_err(AndroidJniError::from)?)
    }
}

impl JniArg for String {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Ljava/lang/String;")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let value = env
            .new_string(self)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(value))
    }
}

impl JniArg for &str {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Ljava/lang/String;")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let value = env
            .new_string(*self)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(value))
    }
}

impl JniReturn for String {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Ljava/lang/String;")
    }

    fn from_jvalue<'a>(
        env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        let value = value.l().map_err(AndroidJniError::from)?;
        if value.is_null() {
            return Err(AndroidJniError::NullReturn("String"));
        }
        let value = internal::JString::from(value);
        let value = env
            .get_string(&value)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(value.into())
    }
}

impl JniArg for Vec<u8> {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("[B")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let array = env
            .byte_array_from_slice(self)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(array))
    }
}

impl JniArg for &[u8] {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("[B")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let array = env
            .byte_array_from_slice(self)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(array))
    }
}

impl JniReturn for Vec<u8> {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("[B")
    }

    fn from_jvalue<'a>(
        env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        let value = value.l().map_err(AndroidJniError::from)?;
        if value.is_null() {
            return Err(AndroidJniError::NullReturn("byte[]"));
        }
        let array = internal::JByteArray::from(value);
        env.convert_byte_array(array)
            .map_err(|err| map_jni_error(env, err))
    }
}

impl JniArg for ActivityRef {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Landroid/app/Activity;")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        if self.raw.is_null() {
            return Err(AndroidJniError::NullObject("Activity"));
        }
        let obj = unsafe { internal::JObject::from_raw(self.raw) };
        let obj = env
            .new_local_ref(obj)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(obj))
    }
}

impl JniArg for ContextRef {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("Landroid/content/Context;")
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        if self.raw.is_null() {
            return Err(AndroidJniError::NullObject("Context"));
        }
        let obj = unsafe { internal::JObject::from_raw(self.raw) };
        let obj = env
            .new_local_ref(obj)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(obj))
    }
}

impl<C: JavaClass> JniArg for JavaObject<C> {
    fn signature() -> Cow<'static, str> {
        C::signature()
    }

    fn to_jvalue<'a>(
        &self,
        env: &mut internal::JNIEnv<'a>,
    ) -> Result<internal::JValueOwned<'a>, AndroidJniError> {
        let obj = env
            .new_local_ref(self.inner.as_obj())
            .map_err(|err| map_jni_error(env, err))?;
        Ok(internal::JValueOwned::from(obj))
    }
}

impl<C: JavaClass> JniReturn for JavaObject<C> {
    fn signature() -> Cow<'static, str> {
        C::signature()
    }

    fn from_jvalue<'a>(
        env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        let value = value.l().map_err(AndroidJniError::from)?;
        if value.is_null() {
            return Err(AndroidJniError::NullReturn(C::name()));
        }
        let global = env
            .new_global_ref(value)
            .map_err(|err| map_jni_error(env, err))?;
        Ok(JavaObject::from_global(global))
    }
}

impl JniReturn for () {
    fn signature() -> Cow<'static, str> {
        Cow::Borrowed("V")
    }

    fn from_jvalue<'a>(
        _env: &mut internal::JNIEnv<'a>,
        value: internal::JValueOwned<'a>,
    ) -> Result<Self, AndroidJniError> {
        value.v().map_err(AndroidJniError::from)?;
        Ok(())
    }
}

/// Hidden helper to load an app class using the Activity class loader.
#[doc(hidden)]
pub fn load_class<'a>(
    env: &mut internal::JNIEnv<'a>,
    activity: &internal::JObject<'a>,
    class_name: &str,
) -> Result<internal::JClass<'a>, AndroidJniError> {
    let class_loader = env
        .call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .and_then(|value| value.l())
        .map_err(|err| map_jni_error(env, err))?;
    let class_name = env
        .new_string(class_name)
        .map_err(|err| map_jni_error(env, err))?;
    let class_name = internal::JObject::from(class_name);
    let class = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[internal::JValue::Object(&class_name)],
        )
        .and_then(|value| value.l())
        .map_err(|err| map_jni_error(env, err))?;
    Ok(internal::JClass::from(class))
}

/// Generates a Java class descriptor for use with JNI bindings.
///
/// ```
/// tessera_ui::android::java_class!(MyClass = "com.example.MyClass");
/// ```
#[macro_export]
macro_rules! java_class {
    ($vis:vis $name:ident = $path:literal) => {
        $vis struct $name;

        impl $crate::android::jni::JavaClass for $name {
            fn name() -> &'static str {
                $path
            }
        }
    };
}

/// Generates JNI bindings for static methods on an Android class.
///
/// ```
/// tessera_ui::android::jni_bind! {
///     class "com.example.MyClass" as MyClass {
///         fn hello(activity: tessera_ui::android::ActivityRef) -> String;
///     }
/// }
/// ```
#[macro_export]
macro_rules! jni_bind {
    (
        class $class:literal as $name:ident {
            $(
                $(#[$meta:meta])*
                fn $method:ident ( $($arg_name:ident : $arg_ty:ty),* $(,)? ) -> $ret:ty;
            )+
        }
    ) => {
        #[doc = concat!("JNI bindings for `", $class, "`.")]
        pub struct $name;

        impl $name {
            $(
                $(#[$meta])*
                #[allow(clippy::needless_pass_by_value)]
                #[allow(non_snake_case)]
                pub fn $method(
                    android_app: &$crate::winit::platform::android::activity::AndroidApp,
                    $($arg_name: $arg_ty),*
                ) -> Result<$ret, $crate::android::jni::AndroidJniError> {
                    let jvm = unsafe {
                        $crate::android::jni::internal::JavaVM::from_raw(
                            android_app.vm_as_ptr().cast(),
                        )
                    }
                    .map_err($crate::android::jni::AndroidJniError::from)?;
                    let mut env = jvm
                        .attach_current_thread()
                        .map_err($crate::android::jni::AndroidJniError::from)?;
                    let activity = unsafe {
                        $crate::android::jni::internal::JObject::from_raw(
                            android_app.activity_as_ptr().cast(),
                        )
                    };
                    let class = $crate::android::jni::load_class(&mut env, &activity, $class)?;

                    let mut signature = String::from("(");
                    $(
                        signature.push_str(
                            <$arg_ty as $crate::android::jni::JniArg>::signature().as_ref(),
                        );
                    )*
                    signature.push(')');
                    signature.push_str(
                        <$ret as $crate::android::jni::JniReturn>::signature().as_ref(),
                    );

                    let mut args = Vec::new();
                    $(
                        args.push(
                            <$arg_ty as $crate::android::jni::JniArg>::to_jvalue(
                                &$arg_name,
                                &mut env,
                            )?,
                        );
                    )*
                    let args_ref: Vec<$crate::android::jni::internal::JValue<'_, '_>> =
                        args.iter().map(|value| value.borrow()).collect();

                    let value = env
                        .call_static_method(class, stringify!($method), &signature, &args_ref)
                        .map_err(|err| $crate::android::jni::map_jni_error(&mut env, err))?;
                    <$ret as $crate::android::jni::JniReturn>::from_jvalue(&mut env, value)
                }
            )+
        }
    };
}
