use std::{io, sync::Arc};

pub(super) fn read(path: &str) -> io::Result<Arc<[u8]>> {
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
    let asset = unsafe {
        crate::ndk_sys::AAssetManager_open(
            manager,
            c_path.as_ptr(),
            crate::ndk_sys::AASSET_MODE_BUFFER as i32,
        )
    };
    if asset.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Asset not found: {path}"),
        ));
    }

    let length = unsafe { crate::ndk_sys::AAsset_getLength64(asset) };
    if length < 0 {
        unsafe {
            crate::ndk_sys::AAsset_close(asset);
        }
        return Err(io::Error::other(format!(
            "Invalid asset length for `{path}`"
        )));
    }

    let mut bytes = vec![0u8; length as usize];
    let mut offset = 0usize;
    while offset < bytes.len() {
        let read = unsafe {
            crate::ndk_sys::AAsset_read(
                asset,
                bytes[offset..].as_mut_ptr().cast(),
                (bytes.len() - offset) as _,
            )
        };
        if read <= 0 {
            break;
        }
        offset += read as usize;
    }
    unsafe {
        crate::ndk_sys::AAsset_close(asset);
    }

    if offset != bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("incomplete asset: {path}"),
        ));
    }
    Ok(Arc::from(bytes))
}
