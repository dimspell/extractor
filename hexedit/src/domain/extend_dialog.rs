//! Dialog state for the "Extend File" hex editor operation.
//!
//! Prompted from the context menu when the user right-clicks a byte. The
//! user enters a byte count and a hex fill pattern (e.g. `"00"` or
//! `"DE AD BE EF"`); committing inserts that many bytes at the cursor
//! address, filled with the repeated pattern, shifting the rest of the
//! file forward.

use iced::widget::Id;

use super::search::parse_hex_query;

/// Upper bound on how many bytes a single extend operation may insert.
/// Guards against accidental OOM from a fat-fingered byte count.
pub const MAX_EXTEND_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct ExtendDialog {
    /// Number of bytes to insert (as typed text).
    pub count_draft: String,
    /// Hex fill pattern (as typed text).
    pub pattern_draft: String,
    /// Last parse/validation error, shown in the modal.
    pub error: Option<String>,
}

impl ExtendDialog {
    pub fn count_input_id() -> Id {
        Id::new("hex_extend_count_input")
    }

    pub fn pattern_input_id() -> Id {
        Id::new("hex_extend_pattern_input")
    }

    pub fn new() -> Self {
        Self {
            count_draft: String::new(),
            pattern_draft: "00".to_string(),
            error: None,
        }
    }

    /// Parse the fill-pattern draft as a sequence of hex bytes (space-separated
    /// or contiguous).
    ///
    /// Reuses [`parse_hex_query`] which accepts both styles:
    /// - `"00 FF AA BB"` → `[0x00, 0xFF, 0xAA, 0xBB]`
    /// - `"00FFAABB"` → `[0x00, 0xFF, 0xAA, 0xBB]` (requires even number of hex digits)
    ///
    /// Returns the pattern bytes or an error message suitable for display.
    pub fn parse_pattern(&self) -> Result<Vec<u8>, String> {
        let s = self.pattern_draft.trim();
        if s.is_empty() {
            return Err("Enter hex byte(s) to fill with".to_string());
        }

        match parse_hex_query(s) {
            // parse_hex_query never returns Some(empty), so any Some is valid.
            Some(bytes) => Ok(bytes),
            None => Err(format!("Invalid hex input: \"{s}\"")),
        }
    }

    /// Parse both drafts into `(byte_count, pattern_bytes)`.
    ///
    /// The count must be a non-zero integer within [`MAX_EXTEND_BYTES`]; the
    /// pattern must be a non-empty hex byte sequence. On success the returned
    /// pattern is guaranteed non-empty, so callers can safely repeat it across
    /// the requested count.
    pub fn parse(&self) -> Result<(u64, Vec<u8>), String> {
        let s = self.count_draft.trim();
        if s.is_empty() {
            return Err("Enter the number of bytes to add".to_string());
        }
        let count = match s.parse::<u64>() {
            Ok(n) => n,
            Err(_) => return Err(format!("Invalid byte count: \"{s}\"")),
        };
        if count == 0 {
            return Err("Count must be at least 1 byte".to_string());
        }
        if count > MAX_EXTEND_BYTES {
            return Err("Count too large (max 16777216)".to_string());
        }
        let pattern = self.parse_pattern()?;
        Ok((count, pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_count_and_spaced_pattern() {
        let dlg = ExtendDialog {
            count_draft: "4".into(),
            pattern_draft: "DE AD BE EF".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Ok((4, vec![0xDE, 0xAD, 0xBE, 0xEF])));
    }

    #[test]
    fn valid_count_and_contiguous_pattern() {
        let dlg = ExtendDialog {
            count_draft: " 2 ".into(),
            pattern_draft: "0102AABB".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Ok((2, vec![0x01, 0x02, 0xAA, 0xBB])));
    }

    #[test]
    fn empty_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(
            dlg.parse(),
            Err("Enter the number of bytes to add".to_string())
        );
    }

    #[test]
    fn whitespace_only_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "   ".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert!(dlg.parse().is_err());
    }

    #[test]
    fn non_numeric_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "12ab".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Err("Invalid byte count: \"12ab\"".to_string()));
    }

    #[test]
    fn negative_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "-5".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Err("Invalid byte count: \"-5\"".to_string()));
    }

    #[test]
    fn zero_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "0".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(
            dlg.parse(),
            Err("Count must be at least 1 byte".to_string())
        );
    }

    #[test]
    fn overflowing_count_returns_error() {
        // u64::MAX + 1 doesn't fit in u64 → parse failure.
        let dlg = ExtendDialog {
            count_draft: "18446744073709551616".into(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert!(dlg.parse().is_err());
    }

    #[test]
    fn too_large_count_returns_error() {
        let dlg = ExtendDialog {
            count_draft: (MAX_EXTEND_BYTES + 1).to_string(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(
            dlg.parse(),
            Err("Count too large (max 16777216)".to_string())
        );
    }

    #[test]
    fn max_count_is_accepted() {
        let dlg = ExtendDialog {
            count_draft: MAX_EXTEND_BYTES.to_string(),
            pattern_draft: "FF".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Ok((MAX_EXTEND_BYTES, vec![0xFF])));
    }

    #[test]
    fn empty_pattern_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "4".into(),
            pattern_draft: "".into(),
            error: None,
        };
        assert_eq!(
            dlg.parse(),
            Err("Enter hex byte(s) to fill with".to_string())
        );
    }

    #[test]
    fn whitespace_only_pattern_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "4".into(),
            pattern_draft: "   ".into(),
            error: None,
        };
        assert!(dlg.parse().is_err());
    }

    #[test]
    fn invalid_pattern_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "4".into(),
            pattern_draft: "XYZ".into(),
            error: None,
        };
        assert_eq!(dlg.parse(), Err("Invalid hex input: \"XYZ\"".to_string()));
    }

    #[test]
    fn odd_hex_digits_pattern_returns_error() {
        let dlg = ExtendDialog {
            count_draft: "4".into(),
            pattern_draft: "A".into(),
            error: None,
        };
        // parse_hex_query requires even number of contiguous hex digits
        assert!(dlg.parse().is_err());
    }

    #[test]
    fn new_defaults_to_single_zero_fill() {
        let dlg = ExtendDialog::new();
        assert_eq!(dlg.count_draft, "");
        assert_eq!(dlg.pattern_draft, "00");
        assert!(dlg.error.is_none());
    }
}
