use std::io;

use js_sys::Uint8Array;
use tessera_ui::asset::Bytes;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub(super) async fn fetch(url: &str) -> io::Result<Bytes> {
    let window = web_sys::window().ok_or_else(|| io::Error::other("missing window"))?;
    let response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(js_error)?
        .dyn_into::<web_sys::Response>()
        .map_err(js_error)?;
    if !response.ok() {
        return Err(io::Error::other(format!(
            "{url}: HTTP {}",
            response.status()
        )));
    }
    let buffer = JsFuture::from(response.array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;
    Ok(Uint8Array::new(&buffer).to_vec().into())
}

fn js_error(error: JsValue) -> io::Error {
    io::Error::other(format!("{error:?}"))
}
