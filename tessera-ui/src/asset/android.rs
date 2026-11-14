use std::{io, ptr::NonNull, sync::Arc};

use super::{Bytes, BytesData};

/// An open Android asset whose buffer stays valid until it is closed.
pub(super) struct AssetBuffer {
    asset: NonNull<crate::ndk_sys::AAsset>,
    data: NonNull<u8>,
    len: usize,
}

// SAFETY: the buffer is immutable and only read after opening; the asset is
// closed exactly once when the last handle drops.
unsafe impl Send for AssetBuffer {}
unsafe impl Sync for AssetBuffer {}

impl AssetBuffer {
    pub(super) fn bytes(&self) -> &[u8] {
        // SAFETY: `AAsset_getBuffer` returned `len` readable bytes that stay
        // valid until the asset is closed in `drop`.
        unsafe { std::slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }
}

impl Drop for AssetBuffer {
    fn drop(&mut self) {
        unsafe { crate::ndk_sys::AAsset_close(self.asset.as_ptr()) };
    }
}

/// Opens a packaged asset and borrows the buffer the asset manager provides.
pub(super) fn read(path: &str) -> io::Result<Bytes> {
    use crate::jni::{JavaVM, objects::JObject};
    use std::ffi::CString;

    fn map_android_error(message: impl Into<String>) -> io::Error {
        io::Error::other(message.into())
    }

    let android_context = crate::ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(android_context.vm().cast()) }
        .map_err(|err| map_android_error(format!("Failed to load JavaVM: {err}")))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|err| map_android_error(format!("Failed to attach JNI thread: {err}")))?;

    let context = unsafe { JObject::from_raw(android_context.context().cast()) };
    let asset_manager_object = env
        .call_method(
            &context,
            "getAssets",
            "()Landroid/content/res/AssetManager;",
            &[],
        )
        .and_then(|value| value.l())
        .map_err(|err| map_android_error(format!("Failed to get AssetManager: {err}")))?;

    if asset_manager_object.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Android AssetManager was null",
        ));
    }

    let manager = unsafe {
        crate::ndk_sys::AAssetManager_fromJava(
            env.get_native_interface(),
            asset_manager_object.into_raw(),
        )
    };
    if manager.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to convert AssetManager handle",
        ));
    }

    let c_path = CString::new(path).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid asset path `{path}`: {err}"),
        )
    })?;
    let asset = NonNull::new(unsafe {
        crate::ndk_sys::AAssetManager_open(
            manager,
            c_path.as_ptr(),
            crate::ndk_sys::AASSET_MODE_BUFFER as i32,
        )
    })
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Asset not found: {path}")))?;

    let length = unsafe { crate::ndk_sys::AAsset_getLength64(asset.as_ptr()) };
    let buffer = unsafe { crate::ndk_sys::AAsset_getBuffer(asset.as_ptr()) };
    match (
        usize::try_from(length),
        NonNull::new(buffer.cast_mut().cast::<u8>()),
    ) {
        (Ok(len), Some(data)) => Ok(Bytes(BytesData::Android(Arc::new(AssetBuffer {
            asset,
            data,
            len,
        })))),
        (Ok(0), None) => {
            unsafe { crate::ndk_sys::AAsset_close(asset.as_ptr()) };
            Ok(Vec::new().into())
        }
        _ => {
            unsafe { crate::ndk_sys::AAsset_close(asset.as_ptr()) };
            Err(io::Error::other(format!("cannot read asset `{path}`")))
        }
    }
}
