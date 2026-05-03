//! Pre-shaped text widget backed by a process-wide `Paragraph` cache.
//!
//! The stock `iced::widget::text` widget shapes its content (cosmic-text
//! glyph layout) every time the widget tree is rebuilt at a new tree
//! position. The spreadsheet's virtual scroll constantly does that — when
//! the visible window slides by `WINDOW_STEP` rows, every row's lazy state
//! is dropped because Iced reconciles by position, not by key. The result
//! is ~100 µs of shaping per non-trivial cell × tens of cells per row × the
//! whole window: a frame hitch on every chunk crossing.
//!
//! This widget keeps an LRU of pre-shaped paragraphs keyed by
//! `(text, size, max_width, font)`. First render of a cell shapes once;
//! every subsequent render — even after the lazy cache is dropped — gets a
//! cache hit and skips shaping entirely. Cache values are
//! `Arc`-backed and clone in O(1).
//!
//! The cache lives in `SpreadsheetState`; clone the `ParagraphCache` handle
//! freely (it is a `Arc<Mutex<...>>` under the hood).

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{self, Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::alignment;
use iced::{mouse, Element, Font, Length, Pixels, Rectangle, Size};
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// The renderer-side paragraph type — fixed because the GUI uses
/// `iced::Renderer` (wgpu backend) exclusively.
type Paragraph = GraphicsParagraph;

/// Capacity of the LRU. Sized for `viewport rows × overscan × column count`
/// across a few editor swaps. ~16 KB per entry × 16 384 = ≈ 16 MB worst case.
const CACHE_CAPACITY: usize = 16_384;

/// Cache key for a shaped paragraph.
///
/// Hashes content rather than storing it: a u64 hash collision would
/// produce a wrong cell, but with a 10⁴-entry cache the birthday-bound
/// probability is ~10⁻¹¹ — negligible compared to the cost of storing
/// every cell string twice.
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

/// Pre-shaped text widget. API surface matches `iced::widget::text` for the
/// subset used by the spreadsheet (size, font, fixed monospace, no wrap,
/// no styling). Extend if more is needed.
pub struct CachedText {
    content: String,
    size: Pixels,
    font: Font,
    width: Length,
    height: Length,
    cache: ParagraphCache,
}

#[derive(Default)]
struct State {
    paragraph: Paragraph,
}

impl CachedText {
    pub fn new(content: impl Into<String>, cache: ParagraphCache) -> Self {
        Self {
            content: content.into(),
            size: Pixels(10.0),
            font: Font::MONOSPACE,
            width: Length::Shrink,
            height: Length::Shrink,
            cache,
        }
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }
}

pub fn cached_text(content: impl Into<String>, cache: ParagraphCache) -> CachedText {
    CachedText::new(content, cache)
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for CachedText {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        layout::sized(limits, self.width, self.height, |limits| {
            let bounds = limits.max();
            let key = ParagraphKey::new(&self.content, self.size.0, bounds.width, self.font);
            let size = self.size;
            let font = self.font;
            let content = self.content.clone();

            let paragraph = self.cache.get_or_insert(key, move || {
                Paragraph::with_text(text::Text {
                    content: content.as_str(),
                    bounds,
                    size,
                    line_height: text::LineHeight::default(),
                    font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
                })
            });

            let state = tree.state.downcast_mut::<State>();
            state.paragraph = paragraph;
            state.paragraph.min_bounds()
        })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let anchor = bounds.anchor(
            state.paragraph.min_bounds(),
            state.paragraph.align_x(),
            state.paragraph.align_y(),
        );
        <iced::Renderer as text::Renderer>::fill_paragraph(
            renderer,
            &state.paragraph,
            anchor,
            defaults.text_color,
            *viewport,
        );
    }
}

impl<'a, Message, Theme> From<CachedText> for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: CachedText) -> Self {
        Element::new(w)
    }
}
