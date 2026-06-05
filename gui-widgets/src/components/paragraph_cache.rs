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

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Font;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_paragraph_cache_insert_and_retrieve() {
        let cache = ParagraphCache::default();
        let key = ParagraphKey::new("Hello", 12.0, 100.0, Font::default());
        let _p = cache.get_or_insert(key.clone(), Paragraph::new);
        // Second call should return cached value without rebuilding
        let _p2 = cache.get_or_insert(key, Paragraph::new);
    }

    #[test]
    fn test_paragraph_cache_clear() {
        let cache = ParagraphCache::default();
        let key = ParagraphKey::new("Hello", 12.0, 100.0, Font::default());
        cache.get_or_insert(key.clone(), Paragraph::new);
        cache.clear();
        // After clear, a new value should be built
        let _ = cache.get_or_insert(key, Paragraph::new);
    }

    #[test]
    fn test_paragraph_key_equality() {
        let a = ParagraphKey::new("abc", 12.0, 100.0, Font::default());
        let b = ParagraphKey::new("abc", 12.0, 100.0, Font::default());
        assert_eq!(a, b);
    }

    #[test]
    fn test_paragraph_key_inequality() {
        let a = ParagraphKey::new("abc", 12.0, 100.0, Font::default());
        let b = ParagraphKey::new("xyz", 12.0, 100.0, Font::default());
        assert_ne!(a, b);
    }

    #[test]
    fn test_paragraph_cache_default_is_empty() {
        let cache = ParagraphCache::default();
        let key = ParagraphKey::new("test", 12.0, 100.0, Font::default());
        // Should not panic — will build a new paragraph
        let _ = cache.get_or_insert(key, Paragraph::new);
    }

    #[test]
    fn test_paragraph_key_size_differentiation() {
        let a = ParagraphKey::new("same text", 12.0, 100.0, Font::default());
        let b = ParagraphKey::new("same text", 14.0, 100.0, Font::default());
        assert_ne!(a, b);
    }

    #[test]
    fn test_paragraph_key_max_width_differentiation() {
        let a = ParagraphKey::new("text", 12.0, 100.0, Font::default());
        let b = ParagraphKey::new("text", 12.0, 200.0, Font::default());
        assert_ne!(a, b);
    }

    #[test]
    fn test_paragraph_key_max_width_clamp() {
        let a = ParagraphKey::new("text", 12.0, 99999.0, Font::default());
        let b = ParagraphKey::new("text", 12.0, 65535.0, Font::default());
        // Both are > u16::MAX (when cast as u16), so both clamp to u16::MAX
        assert_eq!(a, b);
    }

    #[test]
    fn test_paragraph_key_debug() {
        let key = ParagraphKey::new("debug", 12.0, 100.0, Font::default());
        let debug = format!("{:?}", key);
        assert!(debug.contains("ParagraphKey"));
    }

    #[test]
    fn test_paragraph_cache_debug() {
        let cache = ParagraphCache::default();
        let debug = format!("{:?}", cache);
        assert!(debug.contains("ParagraphCache"));
        assert!(debug.contains("len"));
    }

    #[test]
    fn test_paragraph_cache_build_only_when_missing() {
        let cache = ParagraphCache::default();
        let key = ParagraphKey::new("only once", 12.0, 100.0, Font::default());
        let build_count = Arc::new(AtomicI32::new(0));

        let _p1 = cache.get_or_insert(key.clone(), {
            let count = Arc::clone(&build_count);
            move || {
                count.fetch_add(1, Ordering::SeqCst);
                Paragraph::new()
            }
        });
        assert_eq!(build_count.load(Ordering::SeqCst), 1, "first call should build");

        let _p2 = cache.get_or_insert(key.clone(), {
            let count = Arc::clone(&build_count);
            move || {
                count.fetch_add(1, Ordering::SeqCst);
                Paragraph::new()
            }
        });
        assert_eq!(build_count.load(Ordering::SeqCst), 1, "second call should use cache, not build");
    }

    #[test]
    fn test_paragraph_cache_clone() {
        let cache = ParagraphCache::default();
        let key = ParagraphKey::new("clone", 12.0, 100.0, Font::default());
        let _p = cache.get_or_insert(key.clone(), Paragraph::new);

        let cloned = cache.clone();
        // Cloned cache should return the same cached paragraph
        let _p2 = cloned.get_or_insert(key, Paragraph::new);
    }
}
