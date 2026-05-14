//! Selection model for the hex matrix.
//!
//! A selection is an `(anchor, cursor)` pair; the *range* is the inclusive
//! span between them. Nav helpers are pure functions so they're trivial to
//! unit-test without a running widget.

use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: u64,
    pub cursor: u64,
}

impl Selection {
    pub fn single(addr: u64) -> Self {
        Self {
            anchor: addr,
            cursor: addr,
        }
    }

    /// Inclusive byte range covered by the selection.
    pub fn range(&self) -> RangeInclusive<u64> {
        let lo = self.anchor.min(self.cursor);
        let hi = self.anchor.max(self.cursor);
        lo..=hi
    }

    pub fn start(&self) -> u64 {
        self.anchor.min(self.cursor)
    }

    pub fn end(&self) -> u64 {
        self.anchor.max(self.cursor)
    }

    /// Number of bytes covered by the selection. Always ≥ 1 (a single-byte
    /// selection has length 1); `is_empty` doesn't apply.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.end() - self.start() + 1
    }

    pub fn is_single(&self) -> bool {
        self.anchor == self.cursor
    }

    pub fn contains(&self, addr: u64) -> bool {
        self.range().contains(&addr)
    }

    /// Move both anchor and cursor to `addr` (single-byte selection).
    pub fn select(&mut self, addr: u64, max_addr: u64) {
        let a = addr.min(max_addr);
        self.anchor = a;
        self.cursor = a;
    }

    /// Move only the cursor — extends the selection.
    pub fn extend(&mut self, addr: u64, max_addr: u64) {
        self.cursor = addr.min(max_addr);
    }
}

/// Direction for keyboard navigation.
#[derive(Debug, Clone, Copy)]
pub enum NavDir {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
}

/// Compute the new cursor address after a navigation step.
///
/// `bytes_per_row` and `page_rows` come from the renderer; `max_addr` is the
/// last valid byte address (file_len - 1, or 0 for empty files).
pub fn nav_target(
    cursor: u64,
    dir: NavDir,
    bytes_per_row: u64,
    page_rows: u64,
    max_addr: u64,
) -> u64 {
    let bpr = bytes_per_row.max(1);
    let next = match dir {
        NavDir::Left => cursor.saturating_sub(1),
        NavDir::Right => cursor.saturating_add(1),
        NavDir::Up => cursor.saturating_sub(bpr),
        NavDir::Down => cursor.saturating_add(bpr),
        NavDir::LineStart => (cursor / bpr) * bpr,
        NavDir::LineEnd => ((cursor / bpr) * bpr).saturating_add(bpr - 1),
        NavDir::PageUp => cursor.saturating_sub(bpr.saturating_mul(page_rows.max(1))),
        NavDir::PageDown => cursor.saturating_add(bpr.saturating_mul(page_rows.max(1))),
        NavDir::DocumentStart => 0,
        NavDir::DocumentEnd => max_addr,
    };
    next.min(max_addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_byte_selection_has_length_one() {
        let s = Selection::single(42);
        assert_eq!(s.len(), 1);
        assert_eq!(s.start(), 42);
        assert_eq!(s.end(), 42);
        assert!(s.is_single());
        assert!(s.contains(42));
        assert!(!s.contains(43));
    }

    #[test]
    fn extend_grows_selection_in_either_direction() {
        let mut s = Selection::single(10);
        s.extend(15, 100);
        assert_eq!(s.range(), 10..=15);
        assert_eq!(s.len(), 6);

        let mut s = Selection::single(10);
        s.extend(5, 100);
        assert_eq!(s.range(), 5..=10);
        assert_eq!(s.len(), 6);
    }

    #[test]
    fn select_and_extend_clamp_to_max_addr() {
        let mut s = Selection::default();
        s.select(999, 50);
        assert_eq!(s.cursor, 50);
        s.extend(999, 50);
        assert_eq!(s.cursor, 50);
    }

    #[test]
    fn nav_horizontal() {
        assert_eq!(nav_target(5, NavDir::Left, 16, 10, 100), 4);
        assert_eq!(nav_target(5, NavDir::Right, 16, 10, 100), 6);
        // saturating at zero
        assert_eq!(nav_target(0, NavDir::Left, 16, 10, 100), 0);
        // clamped to max
        assert_eq!(nav_target(100, NavDir::Right, 16, 10, 100), 100);
    }

    #[test]
    fn nav_vertical() {
        assert_eq!(nav_target(20, NavDir::Up, 16, 10, 1000), 4);
        assert_eq!(nav_target(20, NavDir::Down, 16, 10, 1000), 36);
        assert_eq!(nav_target(5, NavDir::Up, 16, 10, 1000), 0);
    }

    #[test]
    fn nav_line_endpoints() {
        // bytes_per_row=16, cursor=20 → row starts at 16, ends at 31
        assert_eq!(nav_target(20, NavDir::LineStart, 16, 10, 1000), 16);
        assert_eq!(nav_target(20, NavDir::LineEnd, 16, 10, 1000), 31);
    }

    #[test]
    fn nav_page() {
        assert_eq!(nav_target(500, NavDir::PageUp, 16, 10, 10_000), 340);
        assert_eq!(nav_target(500, NavDir::PageDown, 16, 10, 10_000), 660);
    }

    #[test]
    fn nav_document() {
        assert_eq!(nav_target(500, NavDir::DocumentStart, 16, 10, 10_000), 0);
        assert_eq!(nav_target(500, NavDir::DocumentEnd, 16, 10, 10_000), 10_000);
    }
}
