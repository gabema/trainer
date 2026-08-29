//! Trainer, a Dioxus client-side application.
//!
//! Rendered entirely in the browser: no server, no SSR, no hydration. The whole
//! app is a static bundle deployed to GitHub Pages.
//!
//! Dioxus is a `wasm32`-only dependency, matching the rest of this crate, so the
//! app itself is gated the same way. Without that, `cargo clippy --all-targets`
//! on the host — which CI runs — fails to compile this binary.

#[cfg(target_arch = "wasm32")]
fn main() {
    use dioxus::prelude::*;
    use trainer_web::routes::Route;

    #[component]
    fn App() -> Element {
        // Installs the active-activity signals and starts the clocks. Must be
        // above the router so every route can read them from context.
        trainer_web::state::use_active_activities();
        rsx! { Router::<Route> {} }
    }

    dioxus::launch(App);
}

/// Building this binary for the host is not meaningful; it exists so that
/// host-target lints and checks still cover the crate.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!(
        "trainer-web is a WebAssembly application. Build it with:\n  \
         dx build --release --platform web\n  \
         cargo build -p trainer-web --target wasm32-unknown-unknown"
    );
    std::process::exit(1);
}
