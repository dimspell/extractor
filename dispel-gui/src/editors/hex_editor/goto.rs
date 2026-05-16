//! Goto-address state and parsing for the hex editor.
//!
//! Accepts hex (`0xFF`), decimal (`255`), or relative (`+10`, `-5`) expressions
//! and clamps to `[0, max_addr]`.

use iced::widget::Id;

#[derive(Debug, Clone, Default)]
pub struct GotoState {
    pub draft: String,
    pub error: Option<String>,
}

impl GotoState {
    pub fn input_id() -> Id {
        Id::new("hex_goto_input")
    }

    pub fn new() -> Self {
        Self {
            draft: String::new(),
            error: None,
        }
    }

    /// Parse `draft` as hex, decimal, or relative offset from `cursor`.
    /// Returns the clamped target address or an error string.
    pub fn parse(&self, cursor: u64, max_addr: u64) -> Result<u64, String> {
        let s = self.draft.trim();
        if s.is_empty() {
            return Err("Enter an address".to_string());
        }

        // Relative offset: +N or -N.
        if let Some(rest) = s.strip_prefix('+') {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err("Expected number after '+'".to_string());
            }
            let offset: u64 = parse_u64(rest).map_err(|_| format!("not a valid offset: {rest}"))?;
            return Ok((cursor + offset).min(max_addr));
        }
        if let Some(rest) = s.strip_prefix('-') {
            let rest = rest.trim();
            if rest.is_empty() {
                return Err("Expected number after '-'".to_string());
            }
            let offset: u64 = parse_u64(rest).map_err(|_| format!("not a valid offset: {rest}"))?;
            return Ok(cursor.saturating_sub(offset));
        }

        // 0x prefix → hex.
        if s.starts_with("0x") || s.starts_with("0X") {
            let body = s[2..].trim();
            if body.is_empty() {
                return Err("Expected hex digits after '0x'".to_string());
            }
            let addr = u64::from_str_radix(body, 16)
                .map_err(|_| format!("not a valid hex address: {body}"))?;
            return Ok(addr.min(max_addr));
        }

        // Contains hex letters (a-f, A-F) → hex.
        if s.chars()
            .any(|c| c.is_ascii_hexdigit() && !c.is_ascii_digit())
        {
            let addr =
                u64::from_str_radix(s, 16).map_err(|_| format!("not a valid hex address: {s}"))?;
            return Ok(addr.min(max_addr));
        }

        // Otherwise → decimal.
        if let Ok(addr) = s.parse::<u64>() {
            return Ok(addr.min(max_addr));
        }
        Err(format!("not a valid address: {s}"))
    }
}

/// Parse a string to u64 supporting hex (0x prefix) and decimal.
fn parse_u64(s: &str) -> Result<u64, ()> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(rest, 16).map_err(|_| ())
    } else {
        s.parse::<u64>().map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goto_hex_with_prefix() {
        let gs = GotoState {
            draft: "0xFF".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 1000), Ok(255));
    }

    #[test]
    fn goto_hex_without_prefix() {
        let gs = GotoState {
            draft: "FF".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 1000), Ok(255));
    }

    #[test]
    fn goto_hex_with_letters_parsed_as_hex() {
        let gs = GotoState {
            draft: "A0".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 1000), Ok(160));
    }

    #[test]
    fn goto_decimal() {
        let gs = GotoState {
            draft: "255".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 1000), Ok(255));
    }

    #[test]
    fn goto_relative_forward() {
        let gs = GotoState {
            draft: "+10".into(),
            error: None,
        };
        assert_eq!(gs.parse(100, 1000), Ok(110));
    }

    #[test]
    fn goto_relative_backward() {
        let gs = GotoState {
            draft: "-5".into(),
            error: None,
        };
        assert_eq!(gs.parse(100, 1000), Ok(95));
    }

    #[test]
    fn goto_relative_saturates_at_zero() {
        let gs = GotoState {
            draft: "-5".into(),
            error: None,
        };
        assert_eq!(gs.parse(3, 1000), Ok(0));
    }

    #[test]
    fn goto_clamps_to_max() {
        let gs = GotoState {
            draft: "0xFFFF".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 255), Ok(255));
    }

    #[test]
    fn goto_empty_returns_error() {
        let gs = GotoState {
            draft: "".into(),
            error: None,
        };
        assert!(gs.parse(0, 1000).is_err());
    }

    #[test]
    fn goto_garbage_returns_error() {
        let gs = GotoState {
            draft: "xyz".into(),
            error: None,
        };
        assert!(gs.parse(0, 1000).is_err());
    }

    #[test]
    fn goto_hex_plus_offset() {
        let gs = GotoState {
            draft: "0x10".into(),
            error: None,
        };
        assert_eq!(gs.parse(0, 1000), Ok(16));
    }
}
