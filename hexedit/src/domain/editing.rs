//! Per-cell editing state for the hex matrix.
//!
//! When the user double-clicks a cell (or presses F2 / starts typing a hex
//! digit), the editor enters [`EditState`] for that address. Up to two hex
//! characters are buffered in `draft`; pressing Enter / Tab / typing the
//! second digit commits to the underlying provider and (optionally)
//! advances the cursor by one byte.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditState {
    pub addr: u64,
    /// 0..=2 hex characters, always uppercase.
    pub draft: String,
}

impl EditState {
    pub fn new(addr: u64) -> Self {
        Self {
            addr,
            draft: String::new(),
        }
    }

    /// Append `c` if it is a hex digit and the draft is not yet full.
    /// Returns true if the draft changed.
    pub fn push_char(&mut self, c: char) -> bool {
        if self.draft.len() >= 2 {
            return false;
        }
        if let Some(d) = hex_digit_to_upper(c) {
            self.draft.push(d);
            true
        } else {
            false
        }
    }

    /// Remove the last buffered character. Returns true if something was popped.
    pub fn pop_char(&mut self) -> bool {
        self.draft.pop().is_some()
    }

    /// True once two hex characters have been entered.
    pub fn is_complete(&self) -> bool {
        self.draft.len() == 2
    }

    /// The byte to write if the user committed right now. A single-digit draft
    /// is treated as the low nibble (`"A"` → `0x0A`); an empty draft commits
    /// nothing.
    pub fn staged_byte(&self) -> Option<u8> {
        if self.draft.is_empty() {
            return None;
        }
        u8::from_str_radix(&self.draft, 16).ok()
    }
}

/// Modal state for the data-inspector "Edit" button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorEditState {
    pub entry_idx: usize,
    pub addr: u64,
    pub draft: String,
    pub error: Option<String>,
}

impl InspectorEditState {
    pub fn new(entry_idx: usize, addr: u64, initial: String) -> Self {
        Self {
            entry_idx,
            addr,
            draft: initial,
            error: None,
        }
    }
}

pub fn hex_digit_to_upper(c: char) -> Option<char> {
    if c.is_ascii_hexdigit() {
        Some(c.to_ascii_uppercase())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_char_uppercases_and_caps_at_two() {
        let mut s = EditState::new(0);
        assert!(s.push_char('a'));
        assert!(s.push_char('F'));
        assert!(!s.push_char('1'));
        assert_eq!(s.draft, "AF");
        assert!(s.is_complete());
    }

    #[test]
    fn push_char_rejects_non_hex() {
        let mut s = EditState::new(0);
        assert!(!s.push_char('z'));
        assert!(!s.push_char(' '));
        assert_eq!(s.draft, "");
    }

    #[test]
    fn staged_byte_pads_single_digit_as_low_nibble() {
        let mut s = EditState::new(0);
        s.push_char('A');
        assert_eq!(s.staged_byte(), Some(0x0A));
        s.push_char('B');
        assert_eq!(s.staged_byte(), Some(0xAB));
    }

    #[test]
    fn empty_draft_stages_nothing() {
        let s = EditState::new(0);
        assert_eq!(s.staged_byte(), None);
        assert!(!s.is_complete());
    }

    #[test]
    fn pop_char_removes_last_then_returns_false() {
        let mut s = EditState::new(0);
        s.push_char('1');
        s.push_char('2');
        assert!(s.pop_char());
        assert_eq!(s.draft, "1");
        assert!(s.pop_char());
        assert!(!s.pop_char());
    }
}
