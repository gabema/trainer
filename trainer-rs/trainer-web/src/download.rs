//! Saving a file to disk, replacing `window.downloadFile` in `chart-helper.js`.
//!
//! # Blob rather than a data URI
//!
//! The shim built `data:text/json;charset=utf-8,<the whole export>` and put it
//! in an anchor's `href`. That works for small exports and fails for large
//! ones: browsers cap the length of a URL a navigation may carry, and a real
//! profile's export runs to hundreds of kilobytes of percent-encoded JSON. An
//! object URL is a handle, so its size does not matter, and it is revoked
//! immediately after the click rather than leaking for the life of the page.

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Blob, BlobPropertyBag, HtmlElement, Url};

/// Prompts the browser to save `contents` as `file_name`.
pub fn save_text(file_name: &str, contents: &str, mime_type: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;

    let parts = js_sys::Array::of1(&JsValue::from_str(contents));
    let options = BlobPropertyBag::new();
    options.set_type(mime_type);
    let blob = Blob::new_with_str_sequence_and_options(&parts, &options)?;

    let url = Url::create_object_url_with_blob(&blob)?;

    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", file_name)?;
    // The element never needs to be in the document to be clickable, so unlike
    // the shim there is no append/remove pair around this.
    let result = anchor
        .dyn_ref::<HtmlElement>()
        .ok_or_else(|| JsValue::from_str("anchor is not an HtmlElement"))
        .map(HtmlElement::click);

    // Revoked whether or not the click succeeded, so a failure cannot leak the
    // blob for the life of the document.
    Url::revoke_object_url(&url)?;
    result
}
