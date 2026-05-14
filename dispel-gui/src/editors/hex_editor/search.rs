//! Search state and algorithms for the hex editor.
//!
//! Supports hex byte-sequence search and ASCII substring search.
//! Results are a sorted `Vec<u64>` of match start addresses for fast
//! zero-allocation iteration and rendering.

use iced::Color;
use std::collections::BTreeSet;

use super::coloring::CellColorProvider;

/// Search mode toggled between hex bytes and ASCII text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    Hex,
    Ascii,
}

impl SearchMode {
    pub fn toggle(self) -> Self {
        match self {
            SearchMode::Hex => SearchMode::Ascii,
            SearchMode::Ascii => SearchMode::Hex,
        }
    }
}

/// State for the hex-editor search overlay.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    pub visible: bool,
    pub query: String,
    pub mode: SearchMode,
    pub results: Vec<u64>,
    pub query_len: u64,
    pub current_match: Option<usize>,
    /// Pre-computed set of all addresses covered by matches (for rendering).
    pub match_set: BTreeSet<u64>,
    pub replace_query: String,
    /// Whether the replace-all confirmation is showing.
    pub show_replace_confirm: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            mode: SearchMode::Hex,
            results: Vec::new(),
            query_len: 0,
            current_match: None,
            match_set: BTreeSet::new(),
            replace_query: String::new(),
            show_replace_confirm: false,
        }
    }

    /// Open the search overlay.
    pub fn open(&mut self) {
        self.visible = true;
    }

    /// Returns true if the overlay should be visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Run the search against `data` and populate results.
    pub fn execute(&mut self, data: &[u8]) {
        self.results.clear();
        self.match_set.clear();
        self.current_match = None;
        self.query_len = 0;

        let q = self.query.trim().to_string();
        if q.is_empty() || data.is_empty() {
            return;
        }

        match self.mode {
            SearchMode::Hex => self.search_hex(data, &q),
            SearchMode::Ascii => self.search_ascii(data, &q),
        }

        // Build match_set for the renderer.
        for &start in &self.results {
            for a in start..start + self.query_len {
                self.match_set.insert(a);
            }
        }
    }

    fn search_hex(&mut self, data: &[u8], query: &str) {
        let bytes = match parse_hex_query(query) {
            Some(b) => b,
            None => return,
        };
        if bytes.is_empty() {
            return;
        }
        self.query_len = bytes.len() as u64;
        if bytes.len() > data.len() {
            return;
        }
        for i in 0..=data.len() - bytes.len() {
            if data[i..i + bytes.len()] == bytes[..] {
                self.results.push(i as u64);
            }
        }
    }

    fn search_ascii(&mut self, data: &[u8], query: &str) {
        let needle = query.as_bytes();
        if needle.is_empty() {
            return;
        }
        self.query_len = needle.len() as u64;
        if needle.len() > data.len() {
            return;
        }
        // Simple sliding window.
        for i in 0..=data.len() - needle.len() {
            if data[i..i + needle.len()] == needle[..] {
                self.results.push(i as u64);
            }
        }
    }

    /// Number of matches.
    pub fn count(&self) -> usize {
        self.results.len()
    }

    /// Index of the current match (0-based), or None.
    pub fn current_idx(&self) -> Option<usize> {
        self.current_match
    }

    /// Navigate to the next match (wrapping).
    pub fn next_match(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let next = match self.current_match {
            Some(i) => (i + 1) % self.results.len(),
            None => 0,
        };
        self.current_match = Some(next);
    }

    /// Navigate to the previous match (wrapping).
    pub fn prev_match(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let prev = match self.current_match {
            Some(i) => {
                if i == 0 {
                    self.results.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.results.len() - 1,
        };
        self.current_match = Some(prev);
    }

    /// Current match address, if any.
    pub fn current_addr(&self) -> Option<u64> {
        self.current_match
            .and_then(|i| self.results.get(i).copied())
    }

    /// Range of bytes covered by the current match.
    pub fn current_range(&self) -> Option<(u64, u64)> {
        self.current_addr()
            .map(|start| (start, start + self.query_len.saturating_sub(1)))
    }

    /// True if the state has an active query with results.
    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn clear(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
        self.match_set.clear();
        self.current_match = None;
        self.query_len = 0;
        self.replace_query.clear();
        self.show_replace_confirm = false;
    }
}

/// Parse a hex query string (space-separated or continuous) into a byte vec.
///
/// Examples:
/// - `"DE AD BE EF"` → `[0xDE, 0xAD, 0xBE, 0xEF]`
/// - `"DEADBEEF"` → `[0xDE, 0xAD, 0xBE, 0xEF]`
/// - `"FF"` → `[0xFF]`
pub fn parse_hex_query(s: &str) -> Option<Vec<u8>> {
    let compact: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() || !compact.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(compact.len() / 2);
    for chunk in compact.as_bytes().chunks(2) {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk.get(1).copied().unwrap_or(b'0'))?;
        bytes.push((hi << 4) | lo);
    }
    Some(bytes)
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Returns true if `s` contains at least one valid hex digit character.
pub fn looks_like_hex(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_hexdigit())
}

// ── Coloring provider for search matches ────────────────────────────────

/// Highlights bytes that match a search query.
pub struct SearchMatchProvider<'a> {
    pub results: &'a BTreeSet<u64>,
    pub query_len: u64,
    pub current_addr: Option<u64>,
    pub fg: Color,
    pub bg: Color,
    pub current_fg: Color,
    pub current_bg: Color,
}

impl CellColorProvider for SearchMatchProvider<'_> {
    fn color(&self, addr: u64, _byte: u8) -> (Option<Color>, Option<Color>) {
        // Check if addr is within the current-match highlight.
        if let Some(cur) = self.current_addr {
            if addr >= cur && addr < cur + self.query_len {
                return (Some(self.current_fg), Some(self.current_bg));
            }
        }
        // Check if addr is within any match range.
        if self.results.contains(&addr) {
            return (Some(self.fg), Some(self.bg));
        }
        (None, None)
    }
}

/// Build a BTreeSet from all match addresses covered by `results` + `query_len`.
pub fn build_search_set(results: &[u64], query_len: u64) -> BTreeSet<u64> {
    let mut set = BTreeSet::new();
    for &start in results {
        for a in start..start + query_len {
            set.insert(a);
        }
    }
    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_search_finds_single_occurrence() {
        let data = b"hello\xDE\xAD\xBE\xEFworld";
        let mut s = SearchState::new();
        s.query = "DE AD BE EF".into();
        s.mode = SearchMode::Hex;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 5);
    }

    #[test]
    fn hex_search_finds_multiple_occurrences() {
        let data = b"\xAA\xBB\xAA\xBB\xAA\xBB";
        let mut s = SearchState::new();
        s.query = "AA BB".into();
        s.mode = SearchMode::Hex;
        s.execute(data);
        assert_eq!(s.count(), 3);
    }

    #[test]
    fn hex_search_continuous_no_spaces() {
        let data = b"\xDE\xAD\xBE\xEF";
        let mut s = SearchState::new();
        s.query = "DEADBEEF".into();
        s.mode = SearchMode::Hex;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn hex_search_skips_invalid_queries() {
        let data = b"\xDE\xAD";
        let mut s = SearchState::new();
        s.query = "XYZ".into();
        s.mode = SearchMode::Hex;
        s.execute(data);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn hex_search_odd_length_ignored() {
        let data = b"\xDE\xAD";
        let mut s = SearchState::new();
        s.query = "DEA".into();
        s.mode = SearchMode::Hex;
        s.execute(data);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn ascii_search_finds_substring() {
        let data = b"hello world hello";
        let mut s = SearchState::new();
        s.query = "hello".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 2);
        assert_eq!(s.results[0], 0);
        assert_eq!(s.results[1], 12);
    }

    #[test]
    fn ascii_search_no_match() {
        let data = b"hello world";
        let mut s = SearchState::new();
        s.query = "xyzzy".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn search_empty_query_clears() {
        let data = b"hello";
        let mut s = SearchState::new();
        s.query = "".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn next_match_wraps() {
        let mut s = SearchState::new();
        s.results = vec![10, 20, 30];
        s.current_match = Some(2);
        s.next_match();
        assert_eq!(s.current_match, Some(0));
    }

    #[test]
    fn prev_match_wraps() {
        let mut s = SearchState::new();
        s.results = vec![10, 20, 30];
        s.current_match = Some(0);
        s.prev_match();
        assert_eq!(s.current_match, Some(2));
    }

    #[test]
    fn prev_match_on_empty_does_nothing() {
        let mut s = SearchState::new();
        s.results = vec![];
        s.prev_match();
        assert!(s.current_match.is_none());
    }

    #[test]
    fn parse_valid_hex() {
        let bytes = parse_hex_query("DE AD BE EF").unwrap();
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_hex_continuous() {
        let bytes = parse_hex_query("DEADBEEF").unwrap();
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_hex_mixed_whitespace() {
        let bytes = parse_hex_query(" DE AD  BE EF ").unwrap();
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn parse_hex_odd_length_returns_none() {
        assert!(parse_hex_query("DEA").is_none());
    }

    #[test]
    fn parse_hex_empty_returns_none() {
        assert!(parse_hex_query("").is_none());
    }

    #[test]
    fn build_search_set_covers_all_match_addresses() {
        let results = vec![0, 4];
        let set = build_search_set(&results, 3);
        assert!(set.contains(&0));
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&4));
        assert!(set.contains(&5));
        assert!(set.contains(&6));
        assert_eq!(set.len(), 6);
    }
}
