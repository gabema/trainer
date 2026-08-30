//! Geolocation, replacing `wwwroot/js/geolocation-helper.js`.
//!
//! The shim never rejected: it resolved with `{error: 'denied'}` or
//! `{error: 'unavailable'}` so the caller could tell "the user said no" from
//! "this browser cannot". That distinction is preserved as an enum.

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Position, PositionError, PositionOptions};

/// Matches the shim's options exactly.
const HIGH_ACCURACY: bool = true;
const TIMEOUT_MS: f64 = 10_000.0;
const MAXIMUM_AGE_MS: f64 = 0.0;

/// `PositionError.PERMISSION_DENIED`, which web-sys exposes only as a numeric
/// code.
const PERMISSION_DENIED: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
}

/// Why a fix could not be obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeolocationError {
    /// The user refused. Distinct because the UI says so rather than offering
    /// a retry.
    Denied,
    /// No geolocation support, a timeout, or a position-unavailable failure.
    Unavailable,
}

/// Requests a single position fix.
pub async fn current_position() -> Result<Coordinates, GeolocationError> {
    let Some(geolocation) = web_sys::window()
        .map(|w| w.navigator())
        .and_then(|n| n.geolocation().ok())
    else {
        return Err(GeolocationError::Unavailable);
    };

    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        // One of the two fires, and `once_into_js` frees whichever does. The
        // other leaks a small allocation — the same trade the IndexedDB adapter
        // makes, and for the same reason: dropping a Closure from inside its
        // own callback would be unsound.
        let success = Closure::once_into_js(move |position: JsValue| {
            let _ = resolve.call1(&JsValue::NULL, &position);
        });
        let failure = Closure::once_into_js(move |error: JsValue| {
            let _ = reject.call1(&JsValue::NULL, &error);
        });

        let options = PositionOptions::new();
        options.set_enable_high_accuracy(HIGH_ACCURACY);
        options.set_timeout(TIMEOUT_MS as u32);
        options.set_maximum_age(MAXIMUM_AGE_MS as u32);

        let _ = geolocation.get_current_position_with_error_callback_and_options(
            success.unchecked_ref(),
            Some(failure.unchecked_ref()),
            &options,
        );
    });

    match JsFuture::from(promise).await {
        Ok(value) => value
            .dyn_into::<Position>()
            .map(|position| {
                let coords = position.coords();
                Coordinates {
                    latitude: coords.latitude(),
                    longitude: coords.longitude(),
                    accuracy: coords.accuracy(),
                }
            })
            .map_err(|_| GeolocationError::Unavailable),
        Err(error) => Err(classify(&error)),
    }
}

/// The shim's error branch: `PERMISSION_DENIED` is `denied`, everything else —
/// including a timeout — is `unavailable`.
fn classify(error: &JsValue) -> GeolocationError {
    match error.clone().dyn_into::<PositionError>() {
        Ok(position_error) if position_error.code() == PERMISSION_DENIED => {
            GeolocationError::Denied
        }
        _ => GeolocationError::Unavailable,
    }
}
