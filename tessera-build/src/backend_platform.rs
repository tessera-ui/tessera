use proc_macro2::TokenStream;
use quote::quote;

use android::generate_android_asset_read_tokens;

mod android;

pub(super) fn generate_platform_backend_tokens() -> TokenStream {
    let android_tokens = generate_android_asset_read_tokens();

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

        #android_tokens

        #[cfg(not(target_os = "android"))]
        compile_error!(
            "TESSERA_ASSET_BACKEND=platform is currently only implemented for Android. \
             Use TESSERA_ASSET_BACKEND=embed on this target."
        );

        #[cfg(target_os = "android")]
        fn __tessera_read_platform_asset(path: &str) -> io::Result<Arc<[u8]>> {
            __tessera_read_platform_asset_android(path)
        }

        #[cfg(not(target_os = "android"))]
        fn __tessera_read_platform_asset(_path: &str) -> io::Result<Arc<[u8]>> {
            panic!("Unsupported target for TESSERA_ASSET_BACKEND=platform")
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
