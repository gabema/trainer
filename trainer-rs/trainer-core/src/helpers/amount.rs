//! Decimal amount conversion, ported from `Trainer/Helpers/DecimalAmount.cs`
//! and `DecimalPlacesWarning.cs`.
//!
//! An activity's amount is stored as a raw integer scaled by
//! `10^decimal_places`: 125 at 2 places displays as `1.25`. Amounts are always
//! integers on the wire, which the real profile confirms.

/// Digits kept by [`extract_digits`], matching the C# cap that keeps the
/// accumulator inside `i32`.
const MAX_DIGITS: usize = 9;

/// Fixed-precision form for entry fields: the decimal point is inserted
/// `decimal_places` digits from the right, left-padded with zeros.
///
/// ```text
/// (20, 0)  -> "20"
/// (125, 2) -> "1.25"
/// (5, 2)   -> "0.05"
/// (5, 3)   -> "0.005"
/// ```
pub fn format(amount: i32, decimal_places: i32) -> String {
    if decimal_places <= 0 {
        return amount.to_string();
    }

    let negative = amount < 0;
    // Widened to i64 first, so i32::MIN does not overflow when negated.
    let digits = (amount as i64).abs().to_string();
    let width = decimal_places as usize + 1;
    let padded = if digits.len() < width {
        format!("{}{}", "0".repeat(width - digits.len()), digits)
    } else {
        digits
    };

    let dot = padded.len() - decimal_places as usize;
    let result = format!("{}.{}", &padded[..dot], &padded[dot..]);
    if negative {
        format!("-{result}")
    } else {
        result
    }
}

/// Read-only form, dropping insignificant trailing zeros and a bare decimal
/// point.
///
/// ```text
/// (125, 2) -> "1.25"
/// (120, 2) -> "1.2"
/// (100, 2) -> "1"
/// (5, 2)   -> "0.05"    significant zeros are kept
/// ```
pub fn format_display(amount: i32, decimal_places: i32) -> String {
    let formatted = format(amount, decimal_places);
    if decimal_places <= 0 {
        // Guard matters: without it "20" at 0 places would trim to "2".
        return formatted;
    }
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

/// The raw accumulator behind calculator-style entry: keep only digit
/// characters, capped at [`MAX_DIGITS`]. Returns `None` when the text holds no
/// digits at all, which is how a cleared field is distinguished from a typed
/// zero.
///
/// **Narrowing from the C#.** `char.IsDigit` accepts any Unicode decimal digit,
/// so `char.IsDigit('٥')` is true. This accepts ASCII digits only. The value
/// arrives from a numeric input element, so non-ASCII digits are not reachable
/// in practice, and `int.Parse` on them would not round-trip anyway.
pub fn extract_digits(input: Option<&str>) -> Option<i32> {
    let input = input?;
    if input.is_empty() {
        return None;
    }

    let digits: String = input
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(MAX_DIGITS)
        .collect();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Whether to warn that changing an activity type's precision will reinterpret
/// its existing activities, since stored amounts are raw and never migrated.
pub fn should_warn_about_decimal_places(
    saved_decimal_places: i32,
    current_decimal_places: i32,
    activity_count: i32,
) -> bool {
    activity_count > 0 && current_decimal_places != saved_decimal_places
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Format_ZeroPlaces_ReturnsInteger ──────────────────────────────────

    #[test]
    fn zero_places_returns_the_integer_untouched() {
        for (amount, places, expected) in [(0, 0, "0"), (20, 0, "20"), (125, 0, "125")] {
            assert_eq!(format(amount, places), expected);
        }
    }

    #[test]
    fn negative_decimal_places_behave_like_zero() {
        assert_eq!(format(125, -1), "125");
        assert_eq!(format_display(125, -1), "125");
    }

    // ── Format_WithPlaces_InsertsDecimalPoint ─────────────────────────────

    #[test]
    fn places_insert_a_decimal_point_with_zero_padding() {
        for (amount, places, expected) in [
            (125, 2, "1.25"),
            (5, 2, "0.05"),
            (0, 2, "0.00"),
            (1250, 2, "12.50"),
            (20, 2, "0.20"),
            (5, 3, "0.005"),
            (1, 1, "0.1"),
        ] {
            assert_eq!(format(amount, places), expected, "{amount} @ {places}");
        }
    }

    #[test]
    fn negative_amount_keeps_its_sign() {
        assert_eq!(format(-125, 2), "-1.25");
    }

    #[test]
    fn extreme_amounts_do_not_overflow() {
        // i32::MIN cannot be negated in place; the C# widens to long first.
        assert_eq!(format(i32::MIN, 2), "-21474836.48");
        assert_eq!(format(i32::MAX, 2), "21474836.47");
    }

    // ── FormatDisplay_TrimsTrailingZeros ──────────────────────────────────

    #[test]
    fn display_trims_insignificant_trailing_zeros() {
        for (amount, places, expected) in [
            (125, 2, "1.25"),
            (120, 2, "1.2"),
            (100, 2, "1"),
            (50, 2, "0.5"),
            (5, 2, "0.05"),
            (0, 2, "0"),
            (200, 3, "0.2"),
            (20, 0, "20"),
            (-120, 2, "-1.2"),
        ] {
            assert_eq!(
                format_display(amount, places),
                expected,
                "{amount} @ {places}"
            );
        }
    }

    // ── ExtractDigits ─────────────────────────────────────────────────────

    #[test]
    fn no_digits_means_a_cleared_field() {
        assert_eq!(extract_digits(Some("")), None);
        assert_eq!(extract_digits(None), None);
        assert_eq!(extract_digits(Some(".")), None);
    }

    #[test]
    fn typed_zeros_are_a_real_value() {
        assert_eq!(extract_digits(Some("0.00")), Some(0));
    }

    #[test]
    fn digits_accumulate_as_keystrokes_arrive() {
        for (text, expected) in [("0.01", 1), ("0.12", 12), ("1.25", 125)] {
            assert_eq!(extract_digits(Some(text)), Some(expected));
        }
    }

    #[test]
    fn backspace_drops_the_last_digit() {
        assert_eq!(extract_digits(Some("1.2")), Some(12));
    }

    #[test]
    fn length_is_capped_to_avoid_overflow() {
        assert_eq!(extract_digits(Some("123456789012")), Some(123456789));
    }

    #[test]
    fn keystroke_then_format_matches_calculator_behavior() {
        let accumulated = extract_digits(Some("0.125")).expect("digits present");
        assert_eq!(accumulated, 125);
        assert_eq!(format(accumulated, 2), "1.25");
    }

    // ── DecimalPlacesWarning ──────────────────────────────────────────────

    #[test]
    fn warns_only_when_precision_changed_and_activities_exist() {
        assert!(should_warn_about_decimal_places(0, 2, 5));
        assert!(!should_warn_about_decimal_places(0, 2, 0));
        assert!(!should_warn_about_decimal_places(2, 2, 5));
        assert!(!should_warn_about_decimal_places(0, 0, 0));
    }
}
