//! Duration field parsing, ported from `Trainer/Helpers/DurationInput.cs`.
//!
//! Accepts either whole minutes (`20`) or `M:SS` (`5:30`, `0:30`). Blank input
//! means no duration rather than an error.

/// Outcome of parsing the Duration field. `Ok(None)` is blank input.
pub type ParseResult = Result<Option<i32>, &'static str>;

const NOT_A_NUMBER: &str = "Duration must be a number of minutes or in M:SS format.";
const TOO_MANY_PARTS: &str =
    "Duration must be a number of minutes (e.g., 20) or in M:SS format (e.g., 5:30).";
const NON_NUMERIC_PARTS: &str = "Minutes and seconds must be numeric.";
const NEGATIVE: &str = "Duration cannot be negative.";
const SECONDS_RANGE: &str = "Seconds must be between 00 and 59.";
const MINUTES_RANGE: &str = "Minutes must be less than 1000.";

/// `int.TryParse` with the default `NumberStyles.Integer` tolerates surrounding
/// whitespace, so `"5 : 30"` parses in C#. Rust's `str::parse` does not, hence
/// the explicit trim. Both accept a leading `+` or `-`.
fn parse_component(raw: &str) -> Option<i32> {
    raw.trim().parse::<i32>().ok()
}

/// Parses the Duration field into seconds.
///
/// ```text
/// ""      -> Ok(None)      blank means no duration
/// "20"    -> Ok(Some(1200)) whole minutes
/// "5:30"  -> Ok(Some(330))
/// "0:30"  -> Ok(Some(30))   sub-minute entry, no zero-padding required
/// "5:60"  -> Err(..)        seconds out of range
/// ```
pub fn try_parse(input: Option<&str>) -> ParseResult {
    let Some(input) = input else {
        return Ok(None);
    };
    if input.trim().is_empty() {
        return Ok(None);
    }

    let input = input.trim();
    let parts: Vec<&str> = input.split(':').collect();

    if parts.len() == 1 {
        let Some(minutes) = parse_component(parts[0]) else {
            return Err(NOT_A_NUMBER);
        };
        if minutes < 0 {
            return Err(NEGATIVE);
        }
        if minutes > 999 {
            return Err(MINUTES_RANGE);
        }
        return Ok(Some(minutes * 60));
    }

    if parts.len() != 2 {
        return Err(TOO_MANY_PARTS);
    }

    let (Some(minutes), Some(seconds)) = (parse_component(parts[0]), parse_component(parts[1]))
    else {
        return Err(NON_NUMERIC_PARTS);
    };

    if minutes < 0 || seconds < 0 {
        return Err(NEGATIVE);
    }
    if seconds >= 60 {
        return Err(SECONDS_RANGE);
    }
    if minutes > 999 {
        return Err(MINUTES_RANGE);
    }

    Ok(Some(minutes * 60 + seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TryParse_ValidInput_ReturnsExpectedSeconds ────────────────────────

    #[test]
    fn valid_input_returns_expected_seconds() {
        for (input, expected) in [
            ("0:30", 30),  // sub-minute M:SS, issue #83
            ("5:30", 330), // minutes and seconds
            ("0:05", 5),   // leading-zero seconds component
            ("20", 1200),  // plain minutes
            ("0", 0),      // zero minutes
        ] {
            assert_eq!(try_parse(Some(input)), Ok(Some(expected)), "{input}");
        }
    }

    // ── TryParse_BlankInput_MeansNoDuration ───────────────────────────────

    #[test]
    fn blank_input_means_no_duration() {
        for input in [None, Some(""), Some("   ")] {
            assert_eq!(try_parse(input), Ok(None), "{input:?}");
        }
    }

    // ── TryParse_InvalidInput_ReturnsError ────────────────────────────────

    #[test]
    fn invalid_input_is_rejected() {
        for input in [
            "5:60",  // seconds out of range
            "abc",   // non-numeric
            "-1",    // negative minutes
            "1:2:3", // too many parts
            "1000",  // minutes too large
        ] {
            assert!(
                try_parse(Some(input)).is_err(),
                "{input} should be rejected"
            );
        }
    }

    #[test]
    fn error_messages_match_the_csharp_wording() {
        assert_eq!(try_parse(Some("abc")), Err(NOT_A_NUMBER));
        assert_eq!(try_parse(Some("1:2:3")), Err(TOO_MANY_PARTS));
        assert_eq!(try_parse(Some("a:b")), Err(NON_NUMERIC_PARTS));
        assert_eq!(try_parse(Some("-1")), Err(NEGATIVE));
        assert_eq!(try_parse(Some("1:-1")), Err(NEGATIVE));
        assert_eq!(try_parse(Some("5:60")), Err(SECONDS_RANGE));
        assert_eq!(try_parse(Some("1000")), Err(MINUTES_RANGE));
        assert_eq!(try_parse(Some("1000:00")), Err(MINUTES_RANGE));
    }

    #[test]
    fn boundary_values_are_accepted() {
        assert_eq!(try_parse(Some("999")), Ok(Some(999 * 60)));
        assert_eq!(try_parse(Some("999:59")), Ok(Some(999 * 60 + 59)));
        assert_eq!(try_parse(Some("0:59")), Ok(Some(59)));
    }

    #[test]
    fn whitespace_inside_the_field_is_tolerated_as_in_dotnet() {
        // int.TryParse allows leading and trailing whitespace, so these parse
        // in the C# implementation and must parse here too.
        assert_eq!(try_parse(Some("  20  ")), Ok(Some(1200)));
        assert_eq!(try_parse(Some("5 : 30")), Ok(Some(330)));
    }

    #[test]
    fn empty_components_are_rejected() {
        assert_eq!(try_parse(Some("5:")), Err(NON_NUMERIC_PARTS));
        assert_eq!(try_parse(Some(":30")), Err(NON_NUMERIC_PARTS));
        assert_eq!(try_parse(Some(":")), Err(NON_NUMERIC_PARTS));
    }

    #[test]
    fn a_leading_plus_is_accepted_as_dotnet_does() {
        assert_eq!(try_parse(Some("+5")), Ok(Some(300)));
    }
}
