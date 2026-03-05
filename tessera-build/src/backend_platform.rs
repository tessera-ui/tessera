use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn generate_platform_backend_tokens() -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub struct Asset {
            index: usize,
            path: &'static str,
        }

        impl Asset {
            const fn new_platform(index: usize, path: &'static str) -> Self {
                Self { index, path }
            }
        }

        fn __tessera_read_platform_asset(path: &str) -> io::Result<Arc<[u8]>> {
            #[cfg(target_os = "android")]
            {
                return __tessera_read_android_asset(path);
            }

            #[cfg(not(target_os = "android"))]
            {
                let root = std::env::current_exe()?
                    .parent()
                    .map(std::path::PathBuf::from)
                    .ok_or_else(|| io::Error::other("Failed to resolve executable directory"))?;
                let file_path = root.join("assets").join(path);
                let bytes = std::fs::read(&file_path)?;
                Ok(Arc::from(bytes))
            }
        }

        #[cfg(target_os = "android")]
        fn __tessera_read_android_asset(path: &str) -> io::Result<Arc<[u8]>> {
            use std::ffi::CString;
            use tessera_ui::jni::{objects::JObject, JavaVM};

            fn map_android_error(message: impl Into<String>) -> io::Error {
                io::Error::other(message.into())
            }

            let android_context = tessera_ui::ndk_context::android_context();
            let vm = unsafe { JavaVM::from_raw(android_context.vm().cast()) }
                .map_err(|err| map_android_error(format!("Failed to load JavaVM: {err}")))?;
            let mut env = vm.attach_current_thread().map_err(|err| {
                map_android_error(format!("Failed to attach JNI thread: {err}"))
            })?;

            let context = unsafe { JObject::from_raw(android_context.context().cast()) };
            let asset_manager_object = env
                .call_method(
                    &context,
                    "getAssets",
                    "()Landroid/content/res/AssetManager;",
                    &[],
                )
                .and_then(|value| value.l())
                .map_err(|err| {
                    map_android_error(format!("Failed to get AssetManager: {err}"))
                })?;

            if asset_manager_object.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Android AssetManager was null",
                ));
            }

            let manager = unsafe {
                tessera_ui::ndk_sys::AAssetManager_fromJava(
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
                tessera_ui::ndk_sys::AAssetManager_open(
                    manager,
                    c_path.as_ptr(),
                    tessera_ui::ndk_sys::AASSET_MODE_BUFFER as i32,
                )
            };
            if asset.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Asset not found: {path}"),
                ));
            }

            let length = unsafe { tessera_ui::ndk_sys::AAsset_getLength64(asset) };
            if length < 0 {
                unsafe {
                    tessera_ui::ndk_sys::AAsset_close(asset);
                }
                return Err(io::Error::other(format!(
                    "Invalid asset length for `{path}`"
                )));
            }

            let mut bytes = vec![0u8; length as usize];
            let mut offset = 0usize;
            while offset < bytes.len() {
                let read = unsafe {
                    tessera_ui::ndk_sys::AAsset_read(
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
                tessera_ui::ndk_sys::AAsset_close(asset);
            }

            if offset != bytes.len() {
                bytes.truncate(offset);
            }
            Ok(Arc::from(bytes))
        }

        impl tessera_ui::AssetExt for Asset {
            fn read(self) -> io::Result<Arc<[u8]>> {
                tessera_ui::asset::read_with_lru_cache::<Asset, _>(self.index as u64, || {
                    __tessera_read_platform_asset(self.path)
                })
            }
        }
    }
}
