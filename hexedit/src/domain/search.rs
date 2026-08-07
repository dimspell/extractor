//! Search state and algorithms for the hex editor.
//!
//! Supports hex byte-sequence search and ASCII substring search.
//! Results are a sorted `Vec<u64>` of match start addresses for fast
//! zero-allocation iteration and rendering.

use iced::Color;
use std::collections::BTreeSet;

use crate::coloring::CellColorProvider;

/// Search mode toggled between hex bytes, ASCII text, and decimal integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    Hex,
    Ascii,
    Decimal,
}

impl SearchMode {
    pub fn toggle(self) -> Self {
        match self {
            SearchMode::Hex => SearchMode::Ascii,
            SearchMode::Ascii => SearchMode::Decimal,
            SearchMode::Decimal => SearchMode::Hex,
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
    /// Per-match length in ORIGINAL bytes, parallel to `results`. For
    /// whitespace-collapsed ASCII matches this may exceed `query_len` (the
    /// normalized needle length) because a collapsed run of whitespace maps
    /// back to more than one byte in the file.
    pub extents: Vec<u64>,
    pub query_len: u64,
    pub current_match: Option<usize>,
    /// Decimal byte width (1/2/4/8).
    pub width: u8,
    /// Decimal search endianness: `true` = little-endian.
    pub little_endian: bool,
    /// Pre-computed set of all addresses covered by matches (for rendering).
    pub match_set: BTreeSet<u64>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            visible: false,
            query: String::new(),
            mode: SearchMode::Hex,
            results: Vec::new(),
            extents: Vec::new(),
            query_len: 0,
            current_match: None,
            width: 4,
            little_endian: true,
            match_set: BTreeSet::new(),
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
        self.extents.clear();
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
            SearchMode::Decimal => self.search_decimal(data, &q),
        }

        // Build match_set for the renderer, covering each match's FULL original
        // extent (which may be longer than `query_len` for collapsed ASCII).
        for (j, &start) in self.results.iter().enumerate() {
            let len = self.extents.get(j).copied().unwrap_or(self.query_len);
            for a in start..start + len {
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
                self.extents.push(bytes.len() as u64);
            }
        }
    }

    fn search_ascii(&mut self, data: &[u8], query: &str) {
        let (norm_data, starts, ends) = normalize_whitespace(data);
        let needle = collapse_whitespace(query.as_bytes());
        if needle.is_empty() {
            return;
        }
        self.query_len = needle.len() as u64;
        if needle.len() > norm_data.len() {
            return;
        }
        // Sliding window over the normalized data; map each match back to the
        // original byte offset and full original extent.
        for i in 0..=norm_data.len() - needle.len() {
            if norm_data[i..i + needle.len()] == needle[..] {
                self.results.push(starts[i]);
                let end = ends[i + needle.len() - 1] + 1;
                self.extents.push(end - starts[i]);
            }
        }
    }

    fn search_decimal(&mut self, data: &[u8], query: &str) {
        let value: i128 = match query.parse() {
            Ok(v) => v,
            Err(_) => return,
        };
        let width = self.width.clamp(1, 8) as usize;
        let bits = width * 8;
        // Reject values that don't fit the signed range of `width` bytes
        // rather than silently wrapping (least-surprising behaviour).
        let max = (1i128 << (bits - 1)) - 1;
        let min = -(1i128 << (bits - 1));
        if value < min || value > max {
            return;
        }
        let bytes = int_to_bytes(value, width, self.little_endian);
        self.query_len = bytes.len() as u64;
        if bytes.len() > data.len() {
            return;
        }
        for i in 0..=data.len() - bytes.len() {
            if data[i..i + bytes.len()] == bytes[..] {
                self.results.push(i as u64);
                self.extents.push(bytes.len() as u64);
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

    /// Length in ORIGINAL bytes of the current match's extent, falling back
    /// to `query_len` when there is no current match.
    pub fn current_len(&self) -> u64 {
        match self.current_match {
            Some(i) => self.extents.get(i).copied().unwrap_or(self.query_len),
            None => self.query_len,
        }
    }

    /// Range of bytes covered by the current match.
    pub fn current_range(&self) -> Option<(u64, u64)> {
        self.current_addr()
            .map(|start| (start, start + self.current_len().saturating_sub(1)))
    }

    /// True if the state has an active query with results.
    pub fn has_results(&self) -> bool {
        !self.results.is_empty()
    }

    pub fn clear(&mut self) {
        self.visible = false;
        self.query.clear();
        self.results.clear();
        self.extents.clear();
        self.match_set.clear();
        self.current_match = None;
        self.query_len = 0;
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

/// Normalize a byte slice by collapsing every run of ASCII whitespace bytes
/// into a single `b' '` placeholder. Returns the normalized bytes and two
/// parallel maps that translate each normalized index back to the ORIGINAL
/// byte offsets:
///
/// - `starts[i]` = index of the first byte of the whitespace run / its own index.
/// - `ends[i]` = index of the LAST byte of the whitespace run / its own index.
fn normalize_whitespace(bytes: &[u8]) -> (Vec<u8>, Vec<u64>, Vec<u64>) {
    let mut norm = Vec::with_capacity(bytes.len());
    let mut starts: Vec<u64> = Vec::with_capacity(bytes.len());
    let mut ends: Vec<u64> = Vec::with_capacity(bytes.len());
    let mut in_ws = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b.is_ascii_whitespace() {
            if !in_ws {
                norm.push(b' ');
                starts.push(i as u64);
                ends.push(i as u64);
                in_ws = true;
            } else {
                // Extend the run's end to the current (last seen) whitespace byte.
                if let Some(last) = ends.last_mut() {
                    *last = i as u64;
                }
            }
        } else {
            norm.push(b);
            starts.push(i as u64);
            ends.push(i as u64);
            in_ws = false;
        }
    }
    (norm, starts, ends)
}

/// Collapse every run of ASCII whitespace bytes in `bytes` into a single
/// `b' '` placeholder, returning only the normalized bytes (no offset maps).
/// Used for the query side where the original-offset mapping is not needed.
fn collapse_whitespace(bytes: &[u8]) -> Vec<u8> {
    let mut norm = Vec::with_capacity(bytes.len());
    let mut in_ws = false;
    for &b in bytes {
        if b.is_ascii_whitespace() {
            if !in_ws {
                norm.push(b' ');
                in_ws = true;
            }
        } else {
            norm.push(b);
            in_ws = false;
        }
    }
    norm
}

/// Encode `value` as `width` two's-complement bytes in the requested endianness.
/// The value is masked to `width * 8` bits so negative values wrap correctly.
fn int_to_bytes(value: i128, width: usize, little_endian: bool) -> Vec<u8> {
    let nbytes = width.max(1);
    let bits = nbytes * 8;
    let mask = if bits >= 128 {
        u128::MAX
    } else {
        (1u128 << bits) - 1
    };
    let masked = (value as u128) & mask;
    let mut bytes = vec![0u8; nbytes];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let shift = if little_endian {
            i * 8
        } else {
            (nbytes - 1 - i) * 8
        };
        *byte = (masked >> shift) as u8;
    }
    bytes
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
        if let Some(cur) = self.current_addr
            && addr >= cur
            && addr < cur + self.query_len
        {
            return (Some(self.current_fg), Some(self.current_bg));
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

    #[test]
    fn toggle_cycles_through_all_modes() {
        assert_eq!(SearchMode::Hex.toggle(), SearchMode::Ascii);
        assert_eq!(SearchMode::Ascii.toggle(), SearchMode::Decimal);
        assert_eq!(SearchMode::Decimal.toggle(), SearchMode::Hex);
    }

    #[test]
    fn new_defaults_decimal_width_and_endian() {
        let s = SearchState::new();
        assert_eq!(s.width, 4);
        assert!(s.little_endian);
    }

    #[test]
    fn decimal_search_le_width_2() {
        // 1000 = 0x03E8 → LE bytes E8 03 at offset 3.
        let data = b"\x00\x01\x02\xE8\x03\x00";
        let mut s = SearchState::new();
        s.query = "1000".into();
        s.mode = SearchMode::Decimal;
        s.width = 2;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 3);
    }

    #[test]
    fn decimal_search_be_width_4() {
        // 0x00010203 → BE bytes 00 01 02 03 at offset 0.
        let data = b"\x00\x01\x02\x03\xFF";
        let mut s = SearchState::new();
        s.query = "66051".into();
        s.mode = SearchMode::Decimal;
        s.width = 4;
        s.little_endian = false;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn decimal_search_negative_two_complement() {
        // -1 as width-1 byte → 0xFF.
        let data = b"\xAA\xFF\xBB";
        let mut s = SearchState::new();
        s.query = "-1".into();
        s.mode = SearchMode::Decimal;
        s.width = 1;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 1);
    }

    #[test]
    fn decimal_search_negative_wraps_via_masking() {
        // -256 as width-2 LE → 00 FF.
        let data = b"\x00\xFF\x00";
        let mut s = SearchState::new();
        s.query = "-256".into();
        s.mode = SearchMode::Decimal;
        s.width = 2;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn decimal_search_parse_failure_returns_zero() {
        let data = b"\x00\x01\x02\x03";
        let mut s = SearchState::new();
        s.query = "not-a-number".into();
        s.mode = SearchMode::Decimal;
        s.execute(data);
        assert_eq!(s.count(), 0);
        assert!(s.results.is_empty());
    }

    #[test]
    fn decimal_search_width_zero_clamps() {
        // Width 0 should clamp to 1 and match the low byte.
        let data = b"\x64\x00";
        let mut s = SearchState::new();
        s.query = "100".into();
        s.mode = SearchMode::Decimal;
        s.width = 0;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn decimal_search_positive_overflow_rejected() {
        // 256 doesn't fit signed 8-bit (max 127); must NOT wrap to 0x00.
        let data = b"\x00\xFF\x00";
        let mut s = SearchState::new();
        s.query = "256".into();
        s.mode = SearchMode::Decimal;
        s.width = 1;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 0, "positive overflow should yield no matches");
        assert!(s.results.is_empty());
    }

    #[test]
    fn decimal_search_signed_boundary() {
        // 127 fits signed 8-bit and matches 0x7F.
        let data = b"\x7F\x80";
        let mut s = SearchState::new();
        s.query = "127".into();
        s.mode = SearchMode::Decimal;
        s.width = 1;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 1, "127 should fit signed 8-bit");
        assert_eq!(s.results[0], 0);

        // 128 is out of signed 8-bit range; must be rejected.
        let mut s = SearchState::new();
        s.query = "128".into();
        s.mode = SearchMode::Decimal;
        s.width = 1;
        s.little_endian = true;
        s.execute(data);
        assert_eq!(s.count(), 0, "128 should NOT fit signed 8-bit");
    }

    #[test]
    fn ascii_search_whitespace_collapse_mapping() {
        // "hello\nworld" contains a newline; query "hello world" should match.
        let data = b"hello\nworld";
        let mut s = SearchState::new();
        s.query = "hello world".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 1);
        // The normalized needle "hello world" matches normalized data
        // "hello world"; the space maps to the newline offset 5, and the
        // match start is the "h" at offset 0.
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn ascii_search_multi_whitespace_run_collapses() {
        // Two spaces collapse with the newline into a single normalized space.
        let data = b"hello\n world";
        let mut s = SearchState::new();
        s.query = "hello world".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }

    #[test]
    fn normalize_whitespace_maps_offsets() {
        let (norm, starts, ends) = normalize_whitespace(b"a b");
        assert_eq!(norm, b"a b");
        assert_eq!(starts, vec![0, 1, 2]);
        assert_eq!(ends, vec![0, 1, 2]);

        let (norm, starts, ends) = normalize_whitespace(b"a  \t\n  b");
        assert_eq!(norm, b"a b");
        // Non-whitespace map to themselves; the run starts at index 1 and its
        // last byte (the second space) is at index 6.
        assert_eq!(starts, vec![0, 1, 7]);
        assert_eq!(ends, vec![0, 6, 7]);
    }

    #[test]
    fn ascii_whitespace_extent_covers_full_original_run() {
        // Data "ab  \ncd" — indices: 0='a',1='b',2=' ',3=' ',4='\n',5='c',6='d'.
        // Query "b cd" collapses to "b cd" (3 normalized bytes: b, space, cd).
        // The space in the query collapses the run "  \n" (indices 2..=4).
        // Match start = index 1; full original extent = indices 1..=6 (6 bytes).
        let data = b"ab  \ncd";
        let mut s = SearchState::new();
        s.query = "b cd".into();
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 1, "should find one match");
        assert_eq!(s.results[0], 1, "match should start at index 1");
        assert_eq!(s.extents[0], 6, "extent should cover all 6 original bytes");
        // Navigate to the match first; current_len then reflects its extent.
        s.next_match();
        assert_eq!(s.current_len(), 6, "current match length should be 6");
        for a in 1..=6 {
            assert!(
                s.match_set.contains(&a),
                "match_set should contain index {a} (covered by extent)"
            );
        }
        assert!(
            !s.match_set.contains(&0),
            "match_set should NOT contain index 0"
        );
        assert!(
            !s.match_set.contains(&7),
            "match_set should NOT contain index 7"
        );
        // current_range reflects the full extent, not the normalized length.
        assert_eq!(s.current_range(), Some((1, 6)));
    }

    #[test]
    fn ascii_search_query_whitespace_normalized() {
        // Query itself has runs of whitespace that collapse to single spaces.
        let data = b"hello  world"; // two spaces in data
        let mut s = SearchState::new();
        s.query = "hello   world".into(); // three spaces in query
        s.mode = SearchMode::Ascii;
        s.execute(data);
        assert_eq!(s.count(), 1);
        assert_eq!(s.results[0], 0);
    }
}
