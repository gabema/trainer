//! Browser notifications, replacing `wwwroot/js/notification-helper.js`.
//!
//! Every call is best-effort: if notifications are unsupported, permission was
//! never granted, or no service worker is ready, the call returns quietly. The
//! shim did the same with `try`/`catch`, and the timer must keep working
//! whether or not notifications do.

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Notification, NotificationOptions, NotificationPermission, ServiceWorkerRegistration,
};

/// The tag the shim used, which is what makes a repeat `showNotification`
/// replace the existing notification rather than stack a new one.
fn tag_for(activity_id: i32) -> String {
    format!("active-{activity_id}")
}

/// Resolves `favicon.png` against the document's base path.
///
/// The shim read the `<base>` element for this so a subpath deployment
/// (`/trainer/`) pointed at the right icon.
fn icon_url() -> String {
    let fallback = "/favicon.png".to_owned();
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return fallback;
    };
    let Ok(Some(base)) = document.query_selector("base") else {
        return fallback;
    };
    let Some(href) = base.get_attribute("href") else {
        return fallback;
    };

    let trimmed = href.trim_end_matches('/');
    if trimmed.is_empty() {
        fallback
    } else {
        format!("{trimmed}/favicon.png")
    }
}

fn permission_granted() -> bool {
    Notification::permission() == NotificationPermission::Granted
}

/// Asks for permission once, on mount. Already-granted and already-denied both
/// short-circuit, matching the shim.
pub async fn request_permission() {
    match Notification::permission() {
        NotificationPermission::Granted | NotificationPermission::Denied => {}
        _ => {
            if let Ok(promise) = Notification::request_permission() {
                let _ = JsFuture::from(promise).await;
            }
        }
    }
}

/// The ready service worker registration, or `None` if there is none.
async fn registration() -> Option<ServiceWorkerRegistration> {
    let container = web_sys::window()?.navigator().service_worker();
    let ready = container.ready().ok()?;
    JsFuture::from(ready).await.ok()?.dyn_into().ok()
}

/// Shows or replaces the notification for an active activity.
///
/// The shim's `startActiveNotification` and `updateActiveNotification` had
/// identical bodies — the shared `tag` is what makes the second call replace
/// the first — so they are one function here.
pub async fn show(activity_id: i32, name: &str, elapsed: &str) {
    if !permission_granted() {
        return;
    }
    let Some(registration) = registration().await else {
        return;
    };

    let icon = icon_url();
    let options = NotificationOptions::new();
    options.set_tag(&tag_for(activity_id));
    options.set_body(&format!("Active — {elapsed}"));
    options.set_icon(&icon);
    options.set_badge(&icon);
    options.set_renotify(false);
    options.set_silent(Some(true));

    if let Ok(promise) = registration.show_notification_with_options(name, &options) {
        let _ = JsFuture::from(promise).await;
    }
}

/// Closes the notification for a finished activity.
///
/// The shim called `getNotifications({tag})`. `web-sys` 0.3 binds
/// `get_notifications()` with no filter argument, so the filtering happens here
/// against `Notification::tag()` — the same result, and it keeps the call on
/// typed bindings rather than reaching for `Reflect`.
pub async fn close(activity_id: i32) {
    let Some(registration) = registration().await else {
        return;
    };
    let Ok(promise) = registration.get_notifications() else {
        return;
    };
    let Ok(list) = JsFuture::from(promise).await else {
        return;
    };

    let wanted = tag_for(activity_id);
    for value in Array::from(&list).iter() {
        if let Ok(notification) = value.dyn_into::<Notification>()
            && notification.tag().as_deref() == Some(wanted.as_str())
        {
            notification.close();
        }
    }
}
