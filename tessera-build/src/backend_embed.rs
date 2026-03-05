use proc_macro2::{Literal, TokenStream};
use quote::quote;

use crate::AssetEntry;

pub(super) fn generate_embed_backend_tokens(entries: &[AssetEntry]) -> TokenStream {
    let len = entries.len();
    let bytes = entries.iter().map(|entry| {
        let path = entry.absolute_path.to_string_lossy().into_owned();
        let path_literal = Literal::string(&path);
        quote! { include_bytes!(#path_literal) as &[u8] }
    });

    quote! {
        const __TESSERA_ASSET_BYTES: [&[u8]; #len] = [#(#bytes,)*];

        impl tessera_ui::AssetExt for Asset {
            fn read(self) -> io::Result<Arc<[u8]>> {
                tessera_ui::asset::read_with_lru_cache::<Asset, _>(self.index as u64, || {
                    if let Some(bytes) = __TESSERA_ASSET_BYTES.get(self.index) {
                        return Ok(Arc::<[u8]>::from(*bytes));
                    }
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "asset index out of range",
                    ))
                })
            }
        }
    }
}
