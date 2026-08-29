//! String helpers, ported from `Trainer/Extensions/StringExtensions.cs`.

/// Returns `None` when the value is absent, empty, or entirely whitespace;
/// otherwise the value unchanged.
///
/// Used when saving to the model so a blank field is stored as null rather than
/// an empty string. Note that this is what produces the `None` state of `notes`;
/// a `Some("")` in stored data predates it or arrived by another path, and the
/// two remain distinct.
pub fn null_if_empty_or_whitespace(value: Option<&str>) -> Option<&str> {
    match value {
        Some(v) if !v.trim().is_empty() => Some(v),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_values_become_none() {
        assert_eq!(null_if_empty_or_whitespace(None), None);
        assert_eq!(null_if_empty_or_whitespace(Some("")), None);
        assert_eq!(null_if_empty_or_whitespace(Some("   ")), None);
        assert_eq!(null_if_empty_or_whitespace(Some("\t\n ")), None);
    }

    #[test]
    fn non_blank_values_pass_through_unchanged() {
        assert_eq!(null_if_empty_or_whitespace(Some("note")), Some("note"));
        // Surrounding whitespace is not trimmed, only used for the blank test.
        assert_eq!(
            null_if_empty_or_whitespace(Some("  note  ")),
            Some("  note  ")
        );
    }
}
