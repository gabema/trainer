//! Trainer, a Dioxus client-side application.
//!
//! Rendered entirely in the browser: no server, no SSR, no hydration. The whole
//! app is a static bundle deployed to GitHub Pages.
//!
//! Dioxus is a `wasm32`-only dependency, matching the rest of this crate, so the
//! app itself is gated the same way. Without that, `cargo clippy --all-targets`
//! on the host — which CI runs — fails to compile this binary.

#[cfg(target_arch = "wasm32")]
mod app {
    use dioxus::prelude::*;
    use trainer_web::build_info::VERSION;

    pub fn launch() {
        dioxus::launch(App);
    }

    /// The shell. Routes and pages arrive in section 2.
    #[component]
    fn App() -> Element {
        rsx! {
            main { class: "container py-3",
                h1 { class: "h4", "Trainer" }
                p { class: "text-muted", "Shell is up. Routing and pages follow." }
                footer { class: "text-muted small text-center py-2", "{VERSION}" }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    app::launch();
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
