use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn generate_embed_backend_tokens() -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub struct Asset {
            index: usize,
            bytes: &'static [u8],
        }

        impl Asset {
            const fn new_embed(index: usize, bytes: &'static [u8]) -> Self {
                Self { index, bytes }
            }
        }

        impl tessera_ui::AssetExt for Asset {
            fn read(self) -> io::Result<Arc<[u8]>> {
                tessera_ui::asset::read_with_lru_cache::<Asset, _>(self.index as u64, || {
                    Ok(Arc::<[u8]>::from(self.bytes))
                })
            }
        }
    }
}
