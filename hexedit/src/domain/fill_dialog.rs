//! Dialog state for the "Fill Selection" hex editor operation.
//!
//! Prompted from the context menu when a multi-byte selection exists.
//! The user enters hex bytes (e.g. `"00"` or `"DE AD BE EF"`) which are
//! then repeated across the selected range.

use iced::widget::Id;

use super::search::parse_hex_query;

#[derive(Debug, Clone, Default)]
pub struct FillDialog {
    pub draft: String,
    pub error: Option<String>,
}

impl FillDialog {
    pub fn input_id() -> Id {
        Id::new("hex_fill_input")
    }

    pub fn new() -> Self {
        Self {
            draft: String::new(),
            error: None,
        }
    }

    /// Parse `draft` as a sequence of hex bytes (space-separated or contiguous).
    ///
    /// Reuses [`parse_hex_query`] which accepts both styles:
    /// - `"00 FF AA BB"` → `[0x00, 0xFF, 0xAA, 0xBB]`
    /// - `"00FFAABB"` → `[0x00, 0xFF, 0xAA, 0xBB]` (requires even number of hex digits)
    ///
    /// Returns the pattern bytes or an error message suitable for display.
    pub fn parse_pattern(&self) -> Result<Vec<u8>, String> {
        let s = self.draft.trim();
        if s.is_empty() {
            return Err("Enter hex byte(s) to fill with".to_string());
        }

        match parse_hex_query(s) {
            Some(bytes) if bytes.is_empty() => Err(format!("Could not parse \"{s}\" as hex bytes")),
            Some(bytes) => Ok(bytes),
            None => Err(format!("Invalid hex input: \"{s}\"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte() {
        let dlg = FillDialog {
            draft: "00".into(),
            error: None,
        };
        assert_eq!(dlg.parse_pattern(), Ok(vec![0x00]));
    }

    #[test]
    fn single_byte_nonzero() {
        let dlg = FillDialog {
            draft: "FF".into(),
            error: None,
        };
        assert_eq!(dlg.parse_pattern(), Ok(vec![0xFF]));
    }

    #[test]
    fn multi_byte_spaces() {
        let dlg = FillDialog {
            draft: "DE AD BE EF".into(),
            error: None,
        };
        assert_eq!(dlg.parse_pattern(), Ok(vec![0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[test]
    fn contiguous_hex() {
        let dlg = FillDialog {
            draft: "0102AABB".into(),
            error: None,
        };
        assert_eq!(dlg.parse_pattern(), Ok(vec![0x01, 0x02, 0xAA, 0xBB]));
    }

    #[test]
    fn empty_returns_error() {
        let dlg = FillDialog {
            draft: "".into(),
            error: None,
        };
        assert!(dlg.parse_pattern().is_err());
    }

    #[test]
    fn whitespace_only_returns_error() {
        let dlg = FillDialog {
            draft: "   ".into(),
            error: None,
        };
        assert!(dlg.parse_pattern().is_err());
    }

    #[test]
    fn invalid_hex_returns_error() {
        let dlg = FillDialog {
            draft: "XYZ".into(),
            error: None,
        };
        assert!(dlg.parse_pattern().is_err());
    }

    #[test]
    fn odd_hex_digits_no_spaces_returns_error() {
        let dlg = FillDialog {
            draft: "A".into(),
            error: None,
        };
        // parse_hex_query requires even number of contiguous hex digits
        assert!(dlg.parse_pattern().is_err());
    }
}
