//! Browser-facing layer: the IndexedDB storage implementation in this change,
//! and the Dioxus views in `rust-ui`.
//!
//! Everything here needs a browser, so its tests run under `wasm-bindgen-test`
//! in headless Chrome rather than under `cargo test`. Keeping this split at the
//! crate boundary means the fast native tier in `trainer-core` cannot quietly
//! acquire browser dependencies.

pub mod build_info;

#[cfg(target_arch = "wasm32")]
pub mod clock;
#[cfg(target_arch = "wasm32")]
pub mod download;
#[cfg(target_arch = "wasm32")]
pub mod geolocation;
#[cfg(target_arch = "wasm32")]
pub mod idb;
#[cfg(target_arch = "wasm32")]
pub mod notifications;
#[cfg(target_arch = "wasm32")]
pub mod routes;
#[cfg(target_arch = "wasm32")]
pub mod scroll;
#[cfg(target_arch = "wasm32")]
pub mod state;
#[cfg(target_arch = "wasm32")]
pub mod views;

#[cfg(target_arch = "wasm32")]
pub mod local;

#[cfg(all(test, target_arch = "wasm32"))]
mod browser_api_tests;

#[cfg(all(test, target_arch = "wasm32"))]
mod idb_tests;

#[cfg(all(test, target_arch = "wasm32"))]
mod shim_interop_tests;

#[cfg(all(test, target_arch = "wasm32"))]
mod tier_check {
    //! Verifies the browser test tier is wired up and can reach the APIs the
    //! storage layer depends on. Real coverage arrives with `IdbStorage` in
    //! section 7.

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn indexed_db_is_reachable() {
        let window = web_sys::window().expect("a window exists in the browser tier");
        let idb = window
            .indexed_db()
            .expect("indexedDB is accessible")
            .expect("indexedDB is present in this browser");

        // Opening without an explicit version can never fire an upgrade
        // transaction, which is how the real storage layer must open the
        // existing "Trainer" database.
        let _ = idb.open("__tier_check__").expect("open request is created");
    }

    #[wasm_bindgen_test]
    fn core_crate_is_linked() {
        // Proves the browser tier links against the shared domain crate, so
        // section 7 can implement trainer-core's Storage trait here.
        assert_eq!(trainer_core::CRATE_NAME, "trainer-core");
    }
}
