//! Test-support helpers for locating the committed golden fixtures.
//!
//! Enabled by the `test-support` feature so it is not compiled into the shipping
//! library. `trainer-web` turns the feature on as a dev-dependency, since its
//! browser-tier tests assert against the same fixtures.

use std::path::PathBuf;

/// Absolute path to `trainer-rs/tests/fixtures`.
pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("trainer-core has a parent directory")
        .join("tests")
        .join("fixtures")
}

/// Reads a named fixture as a string, panicking with a useful message if the
/// file is missing — fixtures are committed, so absence is a bug, not a case to
/// handle gracefully.
pub fn read_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

/// Reads and parses a named JSON fixture.
pub fn read_json_fixture(name: &str) -> serde_json::Value {
    let text = read_fixture(name);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("fixture {name} is not valid JSON: {e}"))
}

/// Drives a future to completion on the current thread.
///
/// The native tier's storage futures never perform real I/O — `MemStorage` is a
/// map behind a `RefCell` — so they always complete on the first poll. Rather
/// than take a runtime dependency for tests that never yield, this polls once
/// and treats `Pending` as a bug, failing fast instead of spinning.
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());

    match future.as_mut().poll(&mut context) {
        std::task::Poll::Ready(value) => value,
        std::task::Poll::Pending => {
            panic!("native storage futures must complete without yielding")
        }
    }
}

/// Fixtures embedded at compile time.
///
/// The browser tier has no filesystem, so `read_fixture` cannot serve it.
/// Only the fixtures the browser tests actually need are embedded, to keep them
/// out of builds that do not use them.
pub mod embedded {
    /// The de-identified raw IndexedDB dump.
    pub const IDB_SNAPSHOT: &str = include_str!("../../tests/fixtures/idb-snapshot.json");
}
