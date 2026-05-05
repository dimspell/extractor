//! Process-wide cache of pre-shaped `Paragraph`s, keyed by content + width
//! + size + font.
//!
//! Cosmic-text shaping (≈100 µs per non-trivial cell) used to dominate the
//! spreadsheet's scroll cost. The custom [`TableWidget`](super::table_widget)
//! looks each cell up here on every layout: a hit clones an `Arc`-backed
//! glyph buffer and skips shaping entirely, a miss shapes once and stores
//! the result for reuse on the next viewport tick.
//!
//! The cache lives in `SpreadsheetState`; clone the [`ParagraphCache`]
//! handle freely (it is `Arc<Mutex<…>>` under the hood).

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::Font;
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// The renderer-side paragraph type — fixed because the GUI uses
/// `iced::Renderer` (wgpu backend) exclusively.
pub type Paragraph = GraphicsParagraph;

/// Capacity of the LRU. Sized for `viewport rows × overscan × column count`
/// across a few editor swaps. ~1 KB per entry × 16 384 ≈ 16 MB worst case.
const CACHE_CAPACITY: usize = 16_384;

/// Cache key for a shaped paragraph.
///
/// Hashes content rather than storing it: a u64 hash collision would
/// produce a wrong cell, but with a 10⁴-entry cache the birthday-bound
/// probability is ~10⁻¹¹ — negligible compared to storing every cell
/// string twice.
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParagraphKey {
    text_hash: u64,
    size_x10: u16,
    max_width_px: u16,
    font_hash: u64,
}

impl ParagraphKey {
    pub fn new(text: &str, size: f32, max_width: f32, font: Font) -> Self {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut h);
        let text_hash = h.finish();

        let mut h = std::collections::hash_map::DefaultHasher::new();
        font.hash(&mut h);
        let font_hash = h.finish();

        Self {
            text_hash,
            size_x10: (size * 10.0) as u16,
            max_width_px: max_width.clamp(0.0, u16::MAX as f32) as u16,
            font_hash,
        }
    }
}

#[derive(Clone)]
pub struct ParagraphCache {
    inner: Arc<Mutex<LruCache<ParagraphKey, Paragraph>>>,
}

impl Default for ParagraphCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CAPACITY).expect("non-zero"),
            ))),
        }
    }
}

impl ParagraphCache {
    pub fn get_or_insert<F>(&self, key: ParagraphKey, build: F) -> Paragraph
    where
        F: FnOnce() -> Paragraph,
    {
        let mut g = self.inner.lock().expect("paragraph cache poisoned");
        if let Some(p) = g.get(&key) {
            return p.clone();
        }
        let p = build();
        g.put(key, p.clone());
        p
    }

    /// Drop every cached paragraph. Call after a font / theme change that
    /// invalidates shaped glyphs.
    pub fn clear(&self) {
        if let Ok(mut g) = self.inner.lock() {
            g.clear();
        }
    }
}

impl std::fmt::Debug for ParagraphCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.inner.lock().map(|g| g.len()).unwrap_or(0);
        f.debug_struct("ParagraphCache")
            .field("len", &len)
            .field("capacity", &CACHE_CAPACITY)
            .finish()
    }
}
