//! Custom Iced widget rendering the virtualized hex matrix.
//!
//! Layout (left → right): address gutter, hex bytes (grouped 8 with a small
//! gap), ASCII gutter, scrollbar. Only rows in the viewport are touched per
//! frame; everything else is virtual.

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::mouse;
use iced::{
    alignment, color, Background, Border, Color, Element, Event, Font, Length, Pixels, Rectangle,
    Shadow, Size,
};

use crate::view::editor::paragraph_cache::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

/// Default cell metrics. Tuned for 11px monospace.
const TEXT_SIZE: f32 = 11.0;
const ROW_HEIGHT: f32 = 16.0;
const HEX_CELL_WIDTH: f32 = 20.0;
const ASCII_CELL_WIDTH: f32 = 9.0;
const GROUP_GAP: f32 = 8.0;
const COLUMN_GAP: f32 = 12.0;
const ADDR_COL_WIDTH: f32 = 88.0;
const SCROLLBAR_THICKNESS: f32 = 8.0;

/// How many extra rows to render above/below the viewport so wheel scrolls
/// don't reveal blank bands during rapid scroll.
const OVERSCAN: u64 = 2;

/// Per-instance widget state — what the renderer keeps between frames.
#[derive(Default)]
pub struct State {
    pub scroll_offset: f32,
    pub dragging_scrollbar: bool,
    pub drag_start_cursor_y: f32,
    pub drag_start_offset: f32,
}

pub struct HexMatrix<'a, Message> {
    bytes: &'a [u8],
    bytes_per_row: u8,
    cache: ParagraphCache,
    width: Length,
    height: Length,
    _phantom: std::marker::PhantomData<Message>,
}

impl<'a, Message> HexMatrix<'a, Message> {
    pub fn new(bytes: &'a [u8], bytes_per_row: u8, cache: ParagraphCache) -> Self {
        Self {
            bytes,
            bytes_per_row: bytes_per_row.max(1),
            cache,
            width: Length::Fill,
            height: Length::Fill,
            _phantom: std::marker::PhantomData,
        }
    }

    fn total_rows(&self) -> u64 {
        let bpr = self.bytes_per_row as u64;
        if self.bytes.is_empty() {
            0
        } else {
            self.bytes.len().div_ceil(bpr as usize) as u64
        }
    }

    fn total_height(&self) -> f32 {
        self.total_rows() as f32 * ROW_HEIGHT
    }
}

/// Pure helper — clamp `scroll_offset` and compute `[first, last)` visible
/// rows including overscan. Extracted so it can be unit-tested.
pub fn visible_row_range(
    scroll: f32,
    viewport_height: f32,
    row_height: f32,
    total_rows: u64,
    overscan: u64,
) -> std::ops::Range<u64> {
    if total_rows == 0 || row_height <= 0.0 {
        return 0..0;
    }
    let scroll = scroll.max(0.0);
    let raw_first = ((scroll / row_height).floor() as i64 - overscan as i64).max(0) as u64;
    let first = raw_first.min(total_rows);
    let visible = (viewport_height / row_height).ceil() as u64 + overscan * 2 + 1;
    let last = first.saturating_add(visible).min(total_rows);
    first..last
}

/// Clamp `scroll` to `[0, max_scroll]`.
fn clamp_scroll(scroll: f32, total_height: f32, viewport_height: f32) -> f32 {
    let max_off = (total_height - viewport_height).max(0.0);
    scroll.clamp(0.0, max_off)
}

fn shape_glyph(cache: &ParagraphCache, glyph: &str, font: Font) -> Paragraph {
    let key = ParagraphKey::new(glyph, TEXT_SIZE, 64.0, font);
    cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: glyph,
            bounds: Size::new(64.0, ROW_HEIGHT),
            size: Pixels(TEXT_SIZE),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        })
    })
}

fn ascii_repr(b: u8) -> &'static str {
    // Printable ASCII window. Anything else collapses to a placeholder so
    // the column visually aligns with the hex side.
    const TABLE: [&str; 95] = [
        " ", "!", "\"", "#", "$", "%", "&", "'", "(", ")", "*", "+", ",", "-", ".", "/", "0", "1",
        "2", "3", "4", "5", "6", "7", "8", "9", ":", ";", "<", "=", ">", "?", "@", "A", "B", "C",
        "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U",
        "V", "W", "X", "Y", "Z", "[", "\\", "]", "^", "_", "`", "a", "b", "c", "d", "e", "f", "g",
        "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y",
        "z", "{", "|", "}", "~",
    ];
    if (0x20..0x7F).contains(&b) {
        TABLE[(b - 0x20) as usize]
    } else {
        "·"
    }
}

const HEX_DIGITS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F",
];

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for HexMatrix<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        let max = limits.max();
        Node::new(Size::new(max.width, max.height))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let total_h = self.total_height();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => -y * ROW_HEIGHT * 3.0,
                    mouse::ScrollDelta::Pixels { y, .. } => -y,
                };
                let new = clamp_scroll(state.scroll_offset + dy, total_h, bounds.height);
                if (new - state.scroll_offset).abs() > f32::EPSILON {
                    state.scroll_offset = new;
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                let scrollbar = scrollbar_track(bounds);
                if scrollbar.contains(p) && total_h > bounds.height {
                    let thumb = scrollbar_thumb(scrollbar, state.scroll_offset, total_h);
                    if thumb.contains(p) {
                        state.dragging_scrollbar = true;
                        state.drag_start_cursor_y = p.y;
                        state.drag_start_offset = state.scroll_offset;
                    } else {
                        // Click above/below thumb — page scroll toward cursor.
                        let dir = if p.y < thumb.y { -1.0 } else { 1.0 };
                        let new = clamp_scroll(
                            state.scroll_offset + dir * bounds.height,
                            total_h,
                            bounds.height,
                        );
                        state.scroll_offset = new;
                        shell.request_redraw();
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !state.dragging_scrollbar {
                    return;
                }
                let Some(p) = cursor.position() else { return };
                let scrollbar = scrollbar_track(bounds);
                let thumb_h = thumb_height(scrollbar, total_h);
                let travel = (scrollbar.height - thumb_h).max(1.0);
                let max_off = (total_h - bounds.height).max(1.0);
                let dy = p.y - state.drag_start_cursor_y;
                let new = state.drag_start_offset + dy * (max_off / travel);
                let new = clamp_scroll(new, total_h, bounds.height);
                state.scroll_offset = new;
                shell.request_redraw();
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging_scrollbar =>
            {
                state.dragging_scrollbar = false;
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let clip = bounds.intersection(viewport).unwrap_or(bounds);

        // Background.
        renderer.fill_quad(
            renderer::Quad {
                bounds: clip,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x14110f)),
        );

        let total_rows = self.total_rows();
        if total_rows == 0 {
            return;
        }

        let bpr = self.bytes_per_row as usize;
        let total_h = self.total_height();
        let scroll = clamp_scroll(state.scroll_offset, total_h, bounds.height);

        let visible = visible_row_range(scroll, bounds.height, ROW_HEIGHT, total_rows, OVERSCAN);

        let font = Font::MONOSPACE;
        let addr_color = color!(0x7a6f64);
        let hex_color = color!(0xd4cabd);
        let zero_color = color!(0x4a4339);
        let ascii_color = color!(0xb8a898);
        let group_separator_color = color!(0x251f1a);

        let hex_start_x = bounds.x + ADDR_COL_WIDTH;
        let ascii_start_x = hex_start_x
            + (bpr as f32) * HEX_CELL_WIDTH
            + group_count(bpr) as f32 * GROUP_GAP
            + COLUMN_GAP;

        for row_idx in visible {
            let base_addr = row_idx * bpr as u64;
            let y = bounds.y + (row_idx as f32 * ROW_HEIGHT) - scroll;

            // Address gutter.
            let addr_str = format!("{:08X}", base_addr);
            draw_glyph_string(
                renderer,
                &self.cache,
                &addr_str,
                font,
                Rectangle {
                    x: bounds.x + 8.0,
                    y,
                    width: ADDR_COL_WIDTH - 16.0,
                    height: ROW_HEIGHT,
                },
                addr_color,
                clip,
            );

            // Hex + ASCII columns.
            let row_end = (base_addr as usize + bpr).min(self.bytes.len());
            let row_bytes = &self.bytes[base_addr as usize..row_end];

            for (col, &b) in row_bytes.iter().enumerate() {
                let group = col / 8;
                let cell_x = hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
                let color = if b == 0 { zero_color } else { hex_color };

                // Two nibbles, painted via the shared paragraph cache.
                let hi = shape_glyph(&self.cache, HEX_DIGITS[(b >> 4) as usize], font);
                let lo = shape_glyph(&self.cache, HEX_DIGITS[(b & 0x0F) as usize], font);
                paint_glyph(renderer, &hi, cell_x, y, color, clip);
                paint_glyph(renderer, &lo, cell_x + 8.0, y, color, clip);

                let ascii = shape_glyph(&self.cache, ascii_repr(b), font);
                let ax = ascii_start_x + col as f32 * ASCII_CELL_WIDTH;
                paint_glyph(renderer, &ascii, ax, y, ascii_color, clip);
            }
        }

        // Subtle vertical separator between every 8-byte group (drawn over
        // the row content). Cheap because it's two fill_quads per group.
        for g in 1..group_count(bpr) {
            let x =
                hex_start_x + (g * 8) as f32 * HEX_CELL_WIDTH + (g - 1) as f32 * GROUP_GAP + 4.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: bounds.y,
                        width: 1.0,
                        height: bounds.height,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(group_separator_color),
            );
        }

        // Scrollbar (vertical only in v1).
        if total_h > bounds.height {
            draw_scrollbar(renderer, bounds, scroll, total_h, state.dragging_scrollbar);
        }
    }
}

fn group_count(bpr: usize) -> usize {
    bpr.div_ceil(8).saturating_sub(1)
}

fn paint_glyph(
    renderer: &mut iced::Renderer,
    paragraph: &Paragraph,
    x: f32,
    y: f32,
    color: Color,
    clip: Rectangle,
) {
    let cell = Rectangle {
        x,
        y,
        width: 16.0,
        height: ROW_HEIGHT,
    };
    let pos = cell.anchor(
        paragraph.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    let cell_clip = clip.intersection(&cell).unwrap_or(Rectangle {
        x,
        y,
        width: 0.0,
        height: 0.0,
    });
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, paragraph, pos, color, cell_clip);
}

fn draw_glyph_string(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    text_str: &str,
    font: Font,
    bounds: Rectangle,
    color: Color,
    clip: Rectangle,
) {
    let key = ParagraphKey::new(text_str, TEXT_SIZE, bounds.width, font);
    let para = cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: text_str,
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(TEXT_SIZE),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    let cell_clip = clip.intersection(&bounds).unwrap_or(bounds);
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}

fn scrollbar_track(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
        y: bounds.y,
        width: SCROLLBAR_THICKNESS,
        height: bounds.height,
    }
}

fn thumb_height(track: Rectangle, total_h: f32) -> f32 {
    (track.height / total_h * track.height).max(20.0)
}

fn scrollbar_thumb(track: Rectangle, scroll: f32, total_h: f32) -> Rectangle {
    let h = thumb_height(track, total_h);
    let max_off = (total_h - track.height).max(1.0);
    let y = track.y + (scroll / max_off) * (track.height - h);
    Rectangle {
        x: track.x + 1.0,
        y,
        width: track.width - 2.0,
        height: h,
    }
}

fn draw_scrollbar(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    active: bool,
) {
    let track = scrollbar_track(bounds);
    let thumb = scrollbar_thumb(track, scroll, total_h);
    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0x141210)),
    );
    let thumb_color = if active {
        color!(0xB97024)
    } else {
        color!(0x5d4037)
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: thumb,
            border: Border {
                color: thumb_color,
                width: 0.5,
                radius: 0.into(),
            },
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(thumb_color),
    );
}

impl<'a, Message, Theme> From<HexMatrix<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: HexMatrix<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_provider_yields_empty_range() {
        assert_eq!(visible_row_range(0.0, 200.0, 16.0, 0, 2), 0..0);
    }

    #[test]
    fn visible_range_with_zero_scroll() {
        // 200/16 = 12.5 → ceil = 13; +overscan*2+1 = 18.
        let r = visible_row_range(0.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 18);
    }

    #[test]
    fn visible_range_scrolled_into_middle() {
        // scroll = 320 → row 20; first = 20-2 = 18; last = 18+18 = 36.
        let r = visible_row_range(320.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 18);
        assert_eq!(r.end, 36);
    }

    #[test]
    fn visible_range_clamps_at_total() {
        // Near-end scroll truncates to total_rows.
        let r = visible_row_range(2_000.0, 200.0, 16.0, 100, 2);
        assert!(r.end <= 100);
        assert!(r.start <= r.end);
    }

    #[test]
    fn visible_range_negative_scroll_clamped_to_zero() {
        let r = visible_row_range(-50.0, 200.0, 16.0, 100, 2);
        assert_eq!(r.start, 0);
    }

    #[test]
    fn clamp_scroll_keeps_within_bounds() {
        assert_eq!(clamp_scroll(-10.0, 1000.0, 200.0), 0.0);
        assert_eq!(clamp_scroll(2_000.0, 1000.0, 200.0), 800.0);
        assert_eq!(clamp_scroll(500.0, 1000.0, 200.0), 500.0);
    }

    #[test]
    fn clamp_scroll_when_content_smaller_than_viewport() {
        // total_h < viewport_height → max_off should be 0.
        assert_eq!(clamp_scroll(500.0, 100.0, 1000.0), 0.0);
    }

    #[test]
    fn ascii_repr_handles_printable_and_non_printable() {
        assert_eq!(ascii_repr(b'A'), "A");
        assert_eq!(ascii_repr(b' '), " ");
        assert_eq!(ascii_repr(0x00), "·");
        assert_eq!(ascii_repr(0xFF), "·");
        assert_eq!(ascii_repr(0x7F), "·");
    }

    #[test]
    fn group_count_handles_typical_widths() {
        assert_eq!(group_count(8), 0);
        assert_eq!(group_count(16), 1);
        assert_eq!(group_count(32), 3);
    }
}
