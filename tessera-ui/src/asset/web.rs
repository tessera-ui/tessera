use std::{io, sync::Arc};

use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

pub(super) async fn read(path: &str) -> io::Result<Arc<[u8]>> {
    let window = web_sys::window().ok_or_else(|| io::Error::other("missing window"))?;
    let base = window
        .document()
        .and_then(|document| document.base_uri().ok().flatten())
        .ok_or_else(|| io::Error::other("missing document base URL"))?;
    let encoded = path
        .split('/')
        .map(|part| String::from(js_sys::encode_uri_component(part)))
        .collect::<Vec<_>>()
        .join("/");
    let url = web_sys::Url::new_with_base(&encoded, &base).map_err(js_error)?;
    let response = JsFuture::from(window.fetch_with_str(&url.href()))
        .await
        .map_err(js_error)?;
    let response = response.dyn_into::<web_sys::Response>().map_err(js_error)?;
    if !response.ok() {
        return Err(io::Error::other(format!(
            "asset {path}: HTTP {}",
            response.status()
        )));
    }
    let buffer = JsFuture::from(response.array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;
    Ok(Arc::from(Uint8Array::new(&buffer).to_vec()))
}

fn js_error(error: JsValue) -> io::Error {
    io::Error::other(format!("{error:?}"))
}
