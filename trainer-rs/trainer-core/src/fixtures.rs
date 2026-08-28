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
