//! Byte-source abstraction for the hex editor.
//!
//! The matrix widget reads through a [`HexProvider`] so that vanilla
//! snapshots (read-only mmap, future commit) and the live editing buffer
//! can coexist behind the same interface.

use std::collections::BTreeSet;
use std::ops::Range;

/// A random-access source of bytes consumed by the hex editor.
pub trait HexProvider {
    /// Read the byte range. Returned slice may be shorter than `range.len()`
    /// if `range.end > self.len()`.
    fn read(&self, range: Range<u64>) -> &[u8];

    /// Overwrite bytes starting at `addr`. Out-of-range writes are ignored.
    ///
    /// This is an overwrite-only operation: it never changes the buffer
    /// length. Length-changing edits (e.g. [`BufferProvider::insert`]) are
    /// provided by concrete implementations.
    fn write(&mut self, addr: u64, bytes: &[u8]);

    /// Total byte count.
    fn len(&self) -> u64;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// True if `write` has any effect.
    fn is_writable(&self) -> bool;
}

/// In-memory editing buffer. Tracks which addresses have been overwritten
/// since load so the diff overlay (commit 4) can highlight them.
#[derive(Debug, Clone, Default)]
pub struct BufferProvider {
    data: Vec<u8>,
    dirty: BTreeSet<u64>,
}

impl BufferProvider {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            dirty: BTreeSet::new(),
        }
    }

    pub fn dirty(&self) -> &BTreeSet<u64> {
        &self.dirty
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Insert `bytes` at `addr`, growing the buffer. Bytes previously at
    /// `addr` and beyond shift forward. `addr` beyond the end of the buffer
    /// clamps to the end (append). Inserted offsets are marked dirty, and
    /// existing dirty marks at/after `addr` are rebased forward by
    /// `bytes.len()` so they keep pointing at the same (shifted) bytes.
    pub fn insert(&mut self, addr: u64, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let Ok(addr) = usize::try_from(addr) else {
            return;
        };
        let addr = addr.min(self.data.len());
        let count = bytes.len();
        // Rebase existing dirty marks: a mark at/after the insert point shifts
        // forward by `count`; marks before it keep their keys.
        let old_dirty = std::mem::take(&mut self.dirty);
        for a in old_dirty {
            let a = a as usize;
            self.dirty.insert(if a >= addr {
                (a + count) as u64
            } else {
                a as u64
            });
        }
        self.data.splice(addr..addr, bytes.iter().copied());
        for (i, _) in bytes.iter().enumerate() {
            self.dirty.insert((addr + i) as u64);
        }
    }
}

impl HexProvider for BufferProvider {
    fn read(&self, range: Range<u64>) -> &[u8] {
        let start = (range.start as usize).min(self.data.len());
        let end = (range.end as usize).min(self.data.len());
        &self.data[start..end]
    }

    fn write(&mut self, addr: u64, bytes: &[u8]) {
        let Ok(start) = usize::try_from(addr) else {
            return;
        };
        if start >= self.data.len() {
            return;
        }
        let end = (start + bytes.len()).min(self.data.len());
        let n = end - start;
        if n == 0 {
            return;
        }
        for (i, b) in bytes[..n].iter().enumerate() {
            let a = start + i;
            if self.data[a] != *b {
                self.data[a] = *b;
                self.dirty.insert(a as u64);
            }
        }
    }

    fn len(&self) -> u64 {
        self.data.len() as u64
    }

    fn is_writable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_slice_within_bounds() {
        let p = BufferProvider::from_bytes(vec![1, 2, 3, 4, 5]);
        assert_eq!(p.read(0..3), &[1, 2, 3]);
        assert_eq!(p.read(2..5), &[3, 4, 5]);
    }

    #[test]
    fn read_past_eof_returns_short_slice() {
        let p = BufferProvider::from_bytes(vec![1, 2, 3]);
        assert_eq!(p.read(1..10), &[2, 3]);
        assert_eq!(p.read(5..10), b"");
    }

    #[test]
    fn write_marks_changed_bytes_dirty() {
        let mut p = BufferProvider::from_bytes(vec![0, 0, 0, 0]);
        p.write(1, &[0xAA, 0xBB]);
        assert_eq!(p.as_slice(), &[0, 0xAA, 0xBB, 0]);
        assert_eq!(p.dirty_count(), 2);
        assert!(p.dirty().contains(&1));
        assert!(p.dirty().contains(&2));
    }

    #[test]
    fn write_with_unchanged_value_does_not_mark_dirty() {
        let mut p = BufferProvider::from_bytes(vec![0xAA, 0xBB]);
        p.write(0, &[0xAA]);
        assert_eq!(p.dirty_count(), 0);
    }

    #[test]
    fn write_past_eof_truncates() {
        let mut p = BufferProvider::from_bytes(vec![0, 0, 0]);
        p.write(2, &[1, 2, 3, 4]);
        assert_eq!(p.as_slice(), &[0, 0, 1]);
        assert_eq!(p.dirty_count(), 1);
    }

    #[test]
    fn write_past_end_is_noop() {
        let mut p = BufferProvider::from_bytes(vec![0, 0]);
        p.write(10, &[1, 2]);
        assert_eq!(p.as_slice(), &[0, 0]);
        assert_eq!(p.dirty_count(), 0);
    }

    #[test]
    fn clear_dirty_resets_set() {
        let mut p = BufferProvider::from_bytes(vec![0]);
        p.write(0, &[1]);
        assert_eq!(p.dirty_count(), 1);
        p.clear_dirty();
        assert_eq!(p.dirty_count(), 0);
    }

    #[test]
    fn insert_mid_buffer_shifts_tail() {
        let mut p = BufferProvider::from_bytes(vec![1, 2, 3, 4, 5]);
        p.insert(2, &[0xFF, 0xFF]);
        assert_eq!(p.as_slice(), &[1, 2, 0xFF, 0xFF, 3, 4, 5]);
        assert_eq!(p.len(), 7);
    }

    #[test]
    fn insert_at_zero_prepends() {
        let mut p = BufferProvider::from_bytes(vec![1, 2]);
        p.insert(0, &[0xAA]);
        assert_eq!(p.as_slice(), &[0xAA, 1, 2]);
    }

    #[test]
    fn insert_at_eof_appends() {
        let mut p = BufferProvider::from_bytes(vec![1, 2, 3]);
        p.insert(3, &[0xBB, 0xCC]);
        assert_eq!(p.as_slice(), &[1, 2, 3, 0xBB, 0xCC]);
    }

    #[test]
    fn insert_past_eof_clamps_to_append() {
        let mut p = BufferProvider::from_bytes(vec![1, 2]);
        p.insert(10, &[0xDD]);
        assert_eq!(p.as_slice(), &[1, 2, 0xDD]);
    }

    #[test]
    fn insert_empty_is_noop() {
        let mut p = BufferProvider::from_bytes(vec![1, 2]);
        p.insert(1, &[]);
        assert_eq!(p.as_slice(), &[1, 2]);
        assert_eq!(p.dirty_count(), 0);
    }

    #[test]
    fn insert_marks_inserted_offsets_dirty() {
        let mut p = BufferProvider::from_bytes(vec![0, 0, 0, 0]);
        p.insert(1, &[9, 9]);
        assert_eq!(p.dirty_count(), 2);
        assert!(p.dirty().contains(&1));
        assert!(p.dirty().contains(&2));
        assert!(!p.dirty().contains(&0));
        assert!(!p.dirty().contains(&3));
        assert!(!p.dirty().contains(&4));
    }

    #[test]
    fn insert_rebases_existing_dirty_marks() {
        let mut p = BufferProvider::from_bytes(vec![0, 0, 0, 0, 0, 0]);
        p.write(1, &[0xFF]); // below insert point — key stays at 1
        p.write(3, &[0xFF]); // at insert point — shifts to 3 + 2 = 5
        p.write(4, &[0xFF]); // above insert point — shifts to 4 + 2 = 6
        p.insert(3, &[0xAA, 0xBB]);
        // Rebased marks {1, 5, 6} + inserted offsets {3, 4}.
        assert_eq!(p.dirty_count(), 5);
        assert!(p.dirty().contains(&1));
        assert!(p.dirty().contains(&3));
        assert!(p.dirty().contains(&4));
        assert!(p.dirty().contains(&5));
        assert!(p.dirty().contains(&6));
        assert!(!p.dirty().contains(&7), "old addr 4 shifted to 6, not 7");
    }
}
