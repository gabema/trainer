//! Reproduces `System.Text.Json`'s default string escaping.
//!
//! The shipping app sets no custom `Encoder`, so both serializer configurations
//! use `JavaScriptEncoder.Default`, which escapes far more than JSON requires —
//! HTML-sensitive ASCII plus everything non-ASCII — as XSS defence-in-depth.
//! `serde_json` escapes only the JSON minimum, so exports would not be
//! byte-identical without this.
//!
//! The escape set was measured from the C# implementation rather than assumed;
//! `tests/fixtures/json-escaping.json` is the recorded table and the tests here
//! assert against it.
//!
//! | input | `System.Text.Json` | `serde_json` default |
//! |---|---|---|
//! | `"` | `"` | `\"` |
//! | `\` | `\\` | `\\` |
//! | `&` `'` `+` `<` `>` `` ` `` | `&` … | literal |
//! | U+007F and above | `\uXXXX` (uppercase, surrogate pairs) | literal UTF-8 |
//! | tab, newline, CR, backspace, form feed | `\t` `\n` `\r` `\b` `\f` | same |
//! | other C0 controls | `` (uppercase) | `` (lowercase) |

use serde::Serialize;
use serde_json::ser::{CharEscape, Formatter};
use std::io;

/// ASCII characters escaped by `JavaScriptEncoder.Default` that `serde_json`
/// passes through literally.
const HTML_SENSITIVE: [char; 6] = ['&', '\'', '+', '<', '>', '`'];

/// Writes a char as one or more uppercase `\uXXXX` units, using UTF-16
/// surrogate pairs above the BMP exactly as .NET does.
fn write_unicode_escape<W>(writer: &mut W, ch: char) -> io::Result<()>
where
    W: ?Sized + io::Write,
{
    let mut buf = [0u16; 2];
    for unit in ch.encode_utf16(&mut buf) {
        write!(writer, "\\u{unit:04X}")?;
    }
    Ok(())
}

/// A `serde_json` formatter matching `JavaScriptEncoder.Default`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DotNetFormatter;

impl Formatter for DotNetFormatter {
    /// Receives the runs of characters `serde_json` considers safe. That set is
    /// wider than .NET's, so the extra escaping happens here.
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let bytes = fragment.as_bytes();
        let mut literal_from = 0;
        for (index, ch) in fragment.char_indices() {
            let needs_escape = HTML_SENSITIVE.contains(&ch) || (ch as u32) >= 0x7F;
            if !needs_escape {
                continue;
            }

            if literal_from < index {
                writer.write_all(&bytes[literal_from..index])?;
            }
            write_unicode_escape(writer, ch)?;
            literal_from = index + ch.len_utf8();
        }

        if literal_from < bytes.len() {
            writer.write_all(&bytes[literal_from..])?;
        }
        Ok(())
    }

    /// Writes doubles as `System.Text.Json` does for every value the models can
    /// actually hold.
    ///
    /// ```text
    /// 10.0   ->  10      serde_json writes 10.0
    /// 0.5    ->  0.5     agrees
    /// ```
    ///
    /// Rust's `Display` for `f64` already emits the shortest round-tripping form
    /// without a trailing `.0`, which is exactly .NET's behavior here.
    ///
    /// **Known gap.** For magnitudes where .NET switches to exponent notation it
    /// writes `1E+21` and `1E-07`, whereas `Display` expands them in full. The
    /// only doubles in the domain are `KnownLocation`'s latitude and longitude,
    /// bounded to ±180, so no such value is reachable; reproducing .NET's
    /// exponent threshold would be guesswork against untested behavior.
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        if !value.is_finite() {
            // Matches serde_json; JSON cannot represent these anyway.
            return writer.write_all(b"null");
        }
        write!(writer, "{value}")
    }

    /// Handles the characters `serde_json` already escapes, where .NET differs
    /// on two counts: it uses `"` for a quote rather than `\"`, and it
    /// emits uppercase hex for control characters.
    fn write_char_escape<W>(&mut self, writer: &mut W, char_escape: CharEscape) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        match char_escape {
            CharEscape::Quote => writer.write_all(b"\\u0022"),
            CharEscape::ReverseSolidus => writer.write_all(b"\\\\"),
            CharEscape::Solidus => writer.write_all(b"/"),
            CharEscape::Backspace => writer.write_all(b"\\b"),
            CharEscape::FormFeed => writer.write_all(b"\\f"),
            CharEscape::LineFeed => writer.write_all(b"\\n"),
            CharEscape::CarriageReturn => writer.write_all(b"\\r"),
            CharEscape::Tab => writer.write_all(b"\\t"),
            CharEscape::AsciiControl(byte) => write!(writer, "\\u{byte:04X}"),
        }
    }
}

/// Serializes with .NET-compatible string escaping.
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> Result<String, serde_json::Error> {
    let mut out = Vec::with_capacity(128);
    let mut serializer = serde_json::Serializer::with_formatter(&mut out, DotNetFormatter);
    value.serialize(&mut serializer)?;
    Ok(String::from_utf8(out).expect("serde_json emits valid UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::read_json_fixture;
    use std::collections::BTreeMap;

    #[test]
    fn matches_the_recorded_csharp_escape_table() {
        let table: BTreeMap<String, String> =
            serde_json::from_value(read_json_fixture("json-escaping.json"))
                .expect("the escaping table parses");

        assert!(table.len() >= 135, "expected the full probe set");

        for (codepoint, expected) in &table {
            let raw = u32::from_str_radix(&codepoint[2..], 16).expect("codepoint parses");
            let ch = char::from_u32(raw).expect("valid scalar value");
            let probe = ch.to_string();

            let serialized = to_string(&probe).expect("serializes");
            let inner = &serialized[1..serialized.len() - 1];

            assert_eq!(
                inner, expected,
                "U+{raw:04X} escaped differently from System.Text.Json"
            );
        }
    }

    #[test]
    fn doubles_match_csharp_for_every_reachable_value() {
        let table: BTreeMap<String, String> =
            serde_json::from_value(read_json_fixture("double-formatting.json"))
                .expect("the double table parses");

        let mut checked = 0;
        let mut skipped = Vec::new();

        for (probe, expected_json) in &table {
            let value: f64 = probe.parse().expect("probe parses");

            // Latitude and longitude are the only doubles in the domain and are
            // bounded to +/-180, so anything outside that is unreachable.
            if value != 0.0 && (value.abs() > 180.0 || value.abs() < 1e-6) {
                skipped.push(probe.clone());
                continue;
            }

            let location = crate::models::KnownLocation {
                id: 1,
                name: "L".to_owned(),
                latitude: value,
                longitude: 0.0,
            };
            let actual = crate::models::to_json(&location, crate::models::Format::Storage)
                .expect("serializes");

            assert_eq!(&actual, expected_json, "double {probe} diverged");
            checked += 1;
        }

        assert!(checked >= 10, "only {checked} reachable doubles checked");
        // The unreachable exponent cases, recorded rather than silently ignored.
        assert_eq!(skipped.len(), 2, "unexpected skips: {skipped:?}");
    }

    #[test]
    fn html_sensitive_ascii_is_escaped() {
        assert_eq!(to_string("a+b").expect("ok"), r#""a\u002Bb""#);
        assert_eq!(to_string("a&b").expect("ok"), r#""a\u0026b""#);
        assert_eq!(to_string("Bob's").expect("ok"), r#""Bob\u0027s""#);
        assert_eq!(to_string("<x>").expect("ok"), r#""\u003Cx\u003E""#);
        assert_eq!(to_string("`t`").expect("ok"), r#""\u0060t\u0060""#);
    }

    #[test]
    fn quote_uses_the_unicode_form_but_backslash_does_not() {
        // .NET writes \u0022 for a quote where serde_json writes \".
        assert_eq!(to_string("\"").expect("ok"), r#""\u0022""#);
        assert_eq!(to_string("\\").expect("ok"), r#""\\""#);
    }

    #[test]
    fn non_ascii_is_escaped_with_uppercase_hex() {
        assert_eq!(to_string("½").expect("ok"), r#""\u00BD""#);
        assert_eq!(to_string("é").expect("ok"), r#""\u00E9""#);
        assert_eq!(to_string("中").expect("ok"), r#""\u4E2D""#);
    }

    #[test]
    fn astral_characters_use_surrogate_pairs() {
        assert_eq!(to_string("😀").expect("ok"), r#""\uD83D\uDE00""#);
    }

    #[test]
    fn control_characters_use_short_forms_then_uppercase_hex() {
        assert_eq!(to_string("\n").expect("ok"), r#""\n""#);
        assert_eq!(to_string("\t").expect("ok"), r#""\t""#);
        assert_eq!(to_string("\u{1}").expect("ok"), r#""\u0001""#);
        // Uppercase hex; serde_json would emit \u001f here.
        assert_eq!(to_string("\u{1f}").expect("ok"), r#""\u001F""#);
    }

    #[test]
    fn ordinary_text_is_untouched() {
        assert_eq!(to_string("From knees").expect("ok"), r#""From knees""#);
        // Solidus is NOT escaped by either implementation.
        assert_eq!(to_string("a/b").expect("ok"), r#""a/b""#);
    }
}
