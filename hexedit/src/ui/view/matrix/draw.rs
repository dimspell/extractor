//! Rendering: the `Widget::draw` method and all private paint helpers.
//!
//! Everything here is a pure function of the widget state snapshot — no
//! mutations, no side effects beyond writing to the renderer.

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::Layout;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::Renderer as _;
use iced::mouse;
use iced::{alignment, color, Background, Border, Color, Font, Pixels, Rectangle, Shadow, Size};

use gui_widgets::components::paragraph_cache::{ParagraphCache, ParagraphKey};

use crate::coloring::default_byte_colors;
use crate::domain::byte_stats::entropy_to_color;
use crate::pattern::{pattern_bg, pattern_fg};
use crate::ui::view::minimap::{self, MINIMAP_WIDTH};

use super::layout::{
    clamp_scroll, group_count, visible_row_range, ASCII_CELL_WIDTH, GROUP_GAP, HEADER_HEIGHT,
    HEX_CELL_WIDTH, OVERSCAN, ROW_HEIGHT, SCROLLBAR_THICKNESS, TEXT_SIZE,
};
use super::state::State;
use super::HexMatrix;

type Paragraph = GraphicsParagraph;

// ── Top-level draw entry-point ────────────────────────────────────────

/// Render the hex matrix into the given viewport.
///
/// Called by `Widget::draw` — extracted so the `matrix` module stays
/// focused on the public API and Widget trait bridge.
#[allow(clippy::too_many_arguments)]
pub fn draw_matrix<'a, Message>(
    widget: &HexMatrix<'a, Message>,
    state: &State,
    renderer: &mut iced::Renderer,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
) {
    let bounds = layout.bounds();
    let full_clip = bounds.intersection(viewport).unwrap_or(bounds);

    // Background (full bounds — fill everything).
    renderer.fill_quad(
        renderer::Quad {
            bounds: full_clip,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0x14110f)),
    );

    // ── Geometry for the content area below the column header ──────
    let viewport_h = widget.content_viewport_h(bounds.height, bounds.width);
    let content_top = bounds.y + HEADER_HEIGHT;
    let content_bounds = Rectangle {
        x: bounds.x,
        y: content_top,
        width: bounds.width,
        height: viewport_h,
    };

    // Clip the content rows to the visible portion of the content area
    // (below header, excluding the vertical scrollbar and minimap strip).
    let content_clip_y = content_top.max(full_clip.y);
    let content_clip_bottom = (content_top + viewport_h).min(full_clip.y + full_clip.height);
    let content_clip = Rectangle {
        x: full_clip.x,
        y: content_clip_y,
        width: (full_clip.width - widget.right_strip()).max(0.0),
        height: (content_clip_bottom - content_clip_y).max(0.0),
    };

    // Further clip hex/ASCII cells to exclude the address gutter.
    let cell_clip = Rectangle {
        x: content_clip.x.max(bounds.x + widget.addr_col_width()),
        y: content_clip.y,
        width: (content_clip.x + content_clip.width
            - content_clip.x.max(bounds.x + widget.addr_col_width()))
        .max(0.0),
        height: content_clip.height,
    };

    let total_rows = widget.total_rows();
    let bpr = widget.bytes_per_row as usize;
    let total_h = widget.total_height();
    let bpr64 = bpr as u64;

    let scroll = if total_h <= viewport_h || total_rows == 0 {
        0.0
    } else {
        let cursor = widget.selection.cursor;
        let cursor_row = cursor / bpr64;
        let last = state.last_cursor_row.get();

        if last != Some(cursor_row) {
            state.last_cursor_row.set(Some(cursor_row));
            ensure_visible(
                state.scroll_offset.get(),
                cursor,
                bpr64,
                viewport_h,
                total_h,
            )
        } else {
            state.scroll_offset.get()
        }
    };
    state.scroll_offset.set(scroll);

    let scroll_x = state.scroll_x.get();

    let visible = visible_row_range(scroll, viewport_h, ROW_HEIGHT, total_rows, OVERSCAN);

    let font = Font::MONOSPACE;
    let addr_color = color!(0x7a6f64);
    let hex_color = color!(0xd4cabd);
    let ascii_color = color!(0xb8a898);
    let header_color = color!(0x8a7a6a);
    let header_bg = color!(0x1a1614);
    let header_separator = color!(0x2a2218);
    let group_separator_color = color!(0x251f1a);
    let selection_bg = color!(0x3b2a18);
    let cursor_bg = color!(0x6a4a26);
    let selection_text = color!(0xfff4e0);
    let dirty_bg = color!(0x4a1f1a);
    let dirty_text = color!(0xff9d6e);
    let diff_bg = color!(0x232f1f);
    let diff_text = color!(0x9bd07a);
    let edit_bg = color!(0xc25e1c);
    let edit_text = color!(0xfff8ee);
    let caret_color = color!(0xfff4e0);

    let hex_start_x = content_bounds.x + widget.addr_col_width() - scroll_x;
    let ascii_start_x = widget.ascii_start_x(content_bounds.x) - scroll_x;
    let sel_range = widget.selection.range();
    let cursor_addr = widget.selection.cursor;
    let edit_addr = widget.edit.map(|e| e.addr);

    // ── Address-gutter background (covers header + content rows) ────
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: widget.addr_col_width(),
                height: HEADER_HEIGHT + viewport_h,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0x14110f)),
    );

    // ── Column header ─────────────────────────────────────────────────
    // Background for the header row (right of the address gutter).
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: bounds.x + widget.addr_col_width(),
                y: bounds.y,
                width: (bounds.width - widget.addr_col_width()).max(0.0),
                height: HEADER_HEIGHT,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(header_bg),
    );

    // Hex column numbers (e.g. "0 1 2 3 4 5 6 7  8 9 A B C D E F").
    for col in 0..widget.bytes_per_row as usize {
        let group = col / 8;
        let cell_x = hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
        let label = if widget.show_decimal {
            format!("{}", col)
        } else {
            let c = match col {
                0..=9 => (b'0' + col as u8) as char,
                10..=15 => (b'A' + col as u8 - 10) as char,
                _ => '?',
            };
            c.to_string()
        };
        let label_w = label.len() as f32 * 9.0;
        let text_x = cell_x + (HEX_CELL_WIDTH - label_w) / 2.0;
        draw_glyph_string(
            renderer,
            &widget.cache,
            &label,
            font,
            Rectangle {
                x: text_x,
                y: bounds.y,
                width: label_w,
                height: HEADER_HEIGHT,
            },
            header_color,
            full_clip,
        );
    }

    // Thin separator line below the header.
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: bounds.x,
                y: bounds.y + HEADER_HEIGHT - 1.0,
                width: bounds.width,
                height: 1.0,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(header_separator),
    );

    // ── Data rows ─────────────────────────────────────────────────────
    for row_idx in visible {
        let base_addr = row_idx * bpr as u64;
        let y = content_bounds.y + (row_idx as f32 * ROW_HEIGHT) - scroll;

        // ── Entropy colour band in the address gutter ────────────────
        if let Some(bands) = widget.entropy_bands {
            if let Some(&(_, entropy)) = bands.get(row_idx as usize) {
                let (r, g, b) = entropy_to_color(entropy);
                let band_color = Color::from_rgb(r, g, b);
                let band_rect = Rectangle {
                    x: bounds.x,
                    y,
                    width: 4.0,
                    height: ROW_HEIGHT,
                };
                if let Some(clipped) = content_clip.intersection(&band_rect) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: clipped,
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(band_color),
                    );
                }
            }
        }

        // Address gutter — right-aligned.
        let addr_str = if widget.show_decimal {
            format!("{}", base_addr)
        } else {
            format!("{:08X}", base_addr)
        };
        let text_w = addr_str.len() as f32 * 9.0;
        draw_glyph_string(
            renderer,
            &widget.cache,
            &addr_str,
            font,
            Rectangle {
                x: bounds.x + widget.addr_col_width() - 8.0 - text_w,
                y,
                width: text_w,
                height: ROW_HEIGHT,
            },
            addr_color,
            content_clip,
        );

        // Hex + ASCII columns.
        let row_end = (base_addr as usize + bpr).min(widget.bytes.len());
        let row_bytes = &widget.bytes[base_addr as usize..row_end];

        for (col, &b) in row_bytes.iter().enumerate() {
            let addr = base_addr + col as u64;
            let group = col / 8;
            let cell_x =
                hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
            let ax = ascii_start_x + col as f32 * ASCII_CELL_WIDTH;

            let in_sel = sel_range.contains(&addr);
            let is_dirty = widget.dirty.contains(&addr);
            let is_diff = widget.vanilla_diff.contains(&addr);
            let pat_entry = widget.patterns.get(&addr).copied();
            let is_editing = edit_addr == Some(addr);

            // Background priority: edit > selection-cursor > selection >
            // pattern > dirty (this session) > diff (cumulative vs vanilla).
            let base_bg = if is_editing {
                Some(edit_bg)
            } else if in_sel {
                Some(if addr == cursor_addr {
                    cursor_bg
                } else {
                    selection_bg
                })
            } else if let Some((pid, color_idx)) = pat_entry {
                let mut bg = pattern_bg(color_idx);
                if widget.alternate_patterns.contains(&pid) {
                    bg.r *= 0.5;
                    bg.g *= 0.5;
                    bg.b *= 0.5;
                }
                Some(bg)
            } else if is_dirty {
                Some(dirty_bg)
            } else if is_diff {
                Some(diff_bg)
            } else {
                None
            };

            // Default foreground via the shared provider chain.
            let (default_fg, _) =
                default_byte_colors(widget.color_scheme, b, widget.dim_nulls);
            let default_fg = default_fg.unwrap_or(hex_color);

            let text_color = if is_editing {
                edit_text
            } else if in_sel {
                selection_text
            } else if let Some((_, color_idx)) = pat_entry {
                pattern_fg(color_idx)
            } else if is_dirty {
                dirty_text
            } else if is_diff {
                diff_text
            } else {
                default_fg
            };
            let ascii_col = if is_editing {
                edit_text
            } else if in_sel {
                selection_text
            } else if let Some((_, color_idx)) = pat_entry {
                pattern_fg(color_idx)
            } else if is_dirty {
                dirty_text
            } else if is_diff {
                diff_text
            } else if widget.color_scheme != crate::coloring::ColorScheme::Monochrome {
                default_fg
            } else {
                ascii_color
            };

            // Search-match overlay (overrides bg/fg when applicable).
            let in_search = widget.search_match_set.contains(&addr);
            let in_current_match = widget
                .search_current_addr
                .map(|cur| addr >= cur && addr < cur + widget.search_query_len)
                .unwrap_or(false);
            let bg = if in_current_match {
                Some(color!(0x4a6a2a))
            } else if in_search {
                Some(color!(0x2a4a2a))
            } else {
                base_bg
            };
            let text_color = if in_current_match {
                color!(0xfff8ee)
            } else if in_search {
                color!(0xfff4e0)
            } else {
                text_color
            };

            if let Some(c) = bg {
                fill_cell(renderer, cell_x, y, HEX_CELL_WIDTH, c, cell_clip);
                fill_cell(renderer, ax, y, ASCII_CELL_WIDTH, c, cell_clip);
            }

            if is_editing {
                // Render the in-flight draft instead of the underlying byte.
                // Empty draft → show a thin caret block where the first nibble
                // would land.
                let draft = widget.edit.map(|e| e.draft).unwrap_or("");
                let chars: Vec<char> = draft.chars().collect();
                let hi = chars
                    .first()
                    .map(|c| char_to_glyph(*c))
                    .unwrap_or(HEX_DIGITS[(b >> 4) as usize]);
                let lo = chars
                    .get(1)
                    .map(|c| char_to_glyph(*c))
                    .unwrap_or(HEX_DIGITS[(b & 0x0F) as usize]);
                let hi_p = shape_glyph(&widget.cache, hi, font);
                let lo_p = shape_glyph(&widget.cache, lo, font);
                paint_glyph(renderer, &hi_p, cell_x, y, text_color, cell_clip);
                paint_glyph(renderer, &lo_p, cell_x + 8.0, y, text_color, cell_clip);

                // Caret over the next nibble slot.
                let caret_off = match chars.len() {
                    0 => 0.0,
                    1 => 8.0,
                    _ => 16.0,
                };
                fill_cell(
                    renderer,
                    cell_x + caret_off,
                    y + ROW_HEIGHT - 2.0,
                    7.0,
                    caret_color,
                    cell_clip,
                );

                // ASCII column shows the would-be byte.
                let ascii_glyph = match chars.len() {
                    2 => {
                        let v = u8::from_str_radix(draft, 16).unwrap_or(b);
                        ascii_repr(v)
                    }
                    _ => "·",
                };
                let ascii = shape_glyph(&widget.cache, ascii_glyph, font);
                paint_glyph(renderer, &ascii, ax, y, ascii_col, cell_clip);
            } else {
                let hi = shape_glyph(&widget.cache, HEX_DIGITS[(b >> 4) as usize], font);
                let lo =
                    shape_glyph(&widget.cache, HEX_DIGITS[(b & 0x0F) as usize], font);
                paint_glyph(renderer, &hi, cell_x, y, text_color, cell_clip);
                paint_glyph(renderer, &lo, cell_x + 8.0, y, text_color, cell_clip);

                let ascii = shape_glyph(&widget.cache, ascii_repr(b), font);
                paint_glyph(renderer, &ascii, ax, y, ascii_col, cell_clip);
            }
        }

        // ── Annotation column (per-segment colour) ──────────────────
        if let Some(segments) = widget.row_annotations.get(&base_addr) {
            let ann_x0 = widget.annotation_start_x(bounds.x) - scroll_x;
            let mut seg_x = ann_x0;
            // Shared separator paragraph (shaped once).
            let sep_para = shape_glyph(&widget.cache, " │ ", font);
            let sep_w = sep_para.min_bounds().width;
            for (i, (pat_id, text)) in segments.iter().enumerate() {
                let is_active = widget.active_patterns.contains(pat_id);
                let color = if is_active {
                    color!(0xd4cabd)
                } else {
                    color!(0x6a6050)
                };
                if i > 0 {
                    let cell = Rectangle {
                        x: seg_x,
                        y,
                        width: sep_w,
                        height: ROW_HEIGHT,
                    };
                    let pos = cell.anchor(
                        sep_para.min_bounds(),
                        alignment::Horizontal::Left,
                        alignment::Vertical::Center,
                    );
                    if let Some(cell_clip) = content_clip.intersection(&cell) {
                        <iced::Renderer as text::Renderer>::fill_paragraph(
                            renderer,
                            &sep_para,
                            pos,
                            color!(0x6a6050),
                            cell_clip,
                        );
                    }
                    seg_x += sep_w;
                }
                let para = shape_glyph(&widget.cache, text, font);
                let text_w = para.min_bounds().width;
                let cell = Rectangle {
                    x: seg_x,
                    y,
                    width: text_w,
                    height: ROW_HEIGHT,
                };
                let pos = cell.anchor(
                    para.min_bounds(),
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                );
                if let Some(cell_clip) = content_clip.intersection(&cell) {
                    <iced::Renderer as text::Renderer>::fill_paragraph(
                        renderer, &para, pos, color, cell_clip,
                    );
                }
                seg_x += text_w;
            }
        }
    }

    // Subtle vertical separator between every 8-byte group.
    for g in 1..group_count(bpr) {
        let x = hex_start_x + (g * 8) as f32 * HEX_CELL_WIDTH + (g - 1) as f32 * GROUP_GAP + 4.0;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x,
                    y: content_bounds.y,
                    width: 1.0,
                    height: viewport_h,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(group_separator_color),
        );
    }

    // Minimap overview strip (between content and scrollbar).
    let total_len = widget.bytes.len() as u64;
    let needs_vscroll = total_h > viewport_h;
    if needs_vscroll && widget.show_minimap {
        let hovering = cursor
            .position_over(content_bounds)
            .map(|p| {
                minimap::minimap_rect(content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS)
                    .contains(p)
            })
            .unwrap_or(false);

        // Compute or reuse the minimap pixel cache.
        let h_px = viewport_h.max(1.0) as u32;
        let ctx = minimap::BlockContext {
            bytes: widget.bytes,
            total_len,
            pattern_by_addr: widget.patterns,
            alternate_patterns: &widget.alternate_patterns,
            dirty: widget.dirty,
            vanilla_diff: widget.vanilla_diff,
            color_scheme: widget.color_scheme,
            dim_nulls: widget.dim_nulls,
        };
        let mut cache = state.minimap_cache.borrow_mut();
        let needs_recompute = match &*cache {
            Some(c) => !minimap::minimap_cache_valid(c, h_px, &ctx),
            None => true,
        };
        if needs_recompute {
            let cols = minimap::compute_block_pixels(h_px, &ctx);
            *cache = Some(minimap::MinimapCache {
                columns: cols,
                total_len,
                h_px,
                color_scheme: widget.color_scheme,
                dim_nulls: widget.dim_nulls,
                pattern_hash: minimap::pattern_hash(widget.patterns),
                dirty_count: widget.dirty.len(),
                diff_count: widget.vanilla_diff.len(),
            });
        }
        let columns = &cache.as_ref().unwrap().columns;

        minimap::draw_minimap(
            renderer,
            content_bounds,
            scroll,
            total_h,
            viewport_h,
            SCROLLBAR_THICKNESS,
            columns,
            widget.selection.start(),
            widget.selection.end(),
            widget.selection.cursor,
            total_len,
            state.dragging_minimap || hovering,
        );
    }

    // Scrollbar with search-match and cursor-position markers.
    if needs_vscroll {
        let hovering = cursor
            .position_over(content_bounds)
            .map(|p| scrollbar_track(content_bounds, viewport_h).contains(p))
            .unwrap_or(false);
        draw_vscrollbar(
            renderer,
            content_bounds,
            scroll,
            total_h,
            viewport_h,
            state.dragging_scrollbar || hovering,
            widget.search_match_starts,
            widget.selection.cursor,
            total_len,
        );
    }

    // Horizontal scrollbar at the bottom.
    let content_w = widget.total_content_width();
    let avail_w = bounds.width - SCROLLBAR_THICKNESS;
    let needs_hscroll = content_w > avail_w;
    if needs_hscroll {
        let htrack = hscrollbar_track(bounds);
        let hovering = cursor
            .position_over(bounds)
            .map(|p| htrack.contains(p))
            .unwrap_or(false);
        draw_hscrollbar(
            renderer,
            htrack,
            scroll_x,
            content_w,
            avail_w,
            state.dragging_scrollbar_x || hovering,
        );
    }
}

// ── Private helper functions (unchanged from original) ────────────────

/// Ensure `cursor` is centered in the viewport, adjusting scroll if needed.
fn ensure_visible(
    scroll: f32,
    addr: u64,
    bytes_per_row: u64,
    viewport_height: f32,
    total_height: f32,
) -> f32 {
    let bpr = bytes_per_row.max(1);
    let row = addr / bpr;
    let row_top = row as f32 * ROW_HEIGHT;
    let row_bot = row_top + ROW_HEIGHT;
    if row_top >= scroll && row_bot <= scroll + viewport_height {
        return clamp_scroll(scroll, total_height, viewport_height);
    }
    let center = row_top - (viewport_height - ROW_HEIGHT) / 2.0;
    clamp_scroll(center, total_height, viewport_height)
}

/// Lift a hex character to its rendered glyph. Falls back to a blank for
/// non-hex input (which the message handler also rejects).
pub(crate) fn char_to_glyph(c: char) -> &'static str {
    match c.to_ascii_uppercase() {
        '0' => "0",
        '1' => "1",
        '2' => "2",
        '3' => "3",
        '4' => "4",
        '5' => "5",
        '6' => "6",
        '7' => "7",
        '8' => "8",
        '9' => "9",
        'A' => "A",
        'B' => "B",
        'C' => "C",
        'D' => "D",
        'E' => "E",
        'F' => "F",
        _ => " ",
    }
}

/// First hex character in a typed `text` field, if any.
pub fn first_hex_char(t: &str) -> Option<char> {
    t.chars().find(|c| c.is_ascii_hexdigit())
}

/// First printable (non-control) character in a typed `text` field, if any.
pub fn first_printable_char(t: &str) -> Option<char> {
    t.chars().find(|c| !c.is_control())
}

/// Shape a glyph into a pre-rendered paragraph using the cache.
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

/// ASCII representation of a byte (printable or `·`).
pub(super) fn ascii_repr(b: u8) -> &'static str {
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

fn fill_cell(
    renderer: &mut iced::Renderer,
    x: f32,
    y: f32,
    width: f32,
    color: Color,
    clip: Rectangle,
) {
    let cell = Rectangle {
        x,
        y,
        width,
        height: ROW_HEIGHT,
    };
    let Some(rect) = clip.intersection(&cell) else {
        return;
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: rect,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color),
    );
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
    let Some(cell_clip) = clip.intersection(&bounds) else {
        return;
    };
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}

pub(super) fn scrollbar_track(bounds: Rectangle, viewport_h: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
        y: bounds.y,
        width: SCROLLBAR_THICKNESS,
        height: viewport_h,
    }
}

pub(super) fn hscrollbar_track(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x,
        y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
        width: bounds.width - SCROLLBAR_THICKNESS,
        height: SCROLLBAR_THICKNESS,
    }
}

pub(super) fn thumb_height(track: Rectangle, total_h: f32) -> f32 {
    (track.height / total_h * track.height).max(20.0)
}

/// Y position of a file address on the scrollbar track, as a fraction 0..1.
fn scrollbar_y_frac(addr: u64, total_len: u64, track: Rectangle) -> f32 {
    if total_len <= 1 {
        return track.y;
    }
    track.y + (addr as f32 / (total_len - 1) as f32) * track.height
}

pub(super) fn scrollbar_thumb(track: Rectangle, scroll: f32, total_h: f32) -> Rectangle {
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

pub(super) fn hthumb_len(track: Rectangle, content_w: f32, _avail_w: f32) -> f32 {
    (track.width / content_w * track.width).max(20.0)
}

pub(super) fn hscrollbar_thumb(track: Rectangle, scroll_x: f32, content_w: f32, avail_w: f32) -> Rectangle {
    let w = hthumb_len(track, content_w, avail_w);
    let max_off = (content_w - avail_w).max(1.0);
    let x = track.x + (scroll_x / max_off) * (track.width - w);
    Rectangle {
        x,
        y: track.y + 1.0,
        width: w,
        height: track.height - 2.0,
    }
}

/// Marker dot size in pixels.
const MARKER_SIZE: f32 = 4.0;

/// Draw the vertical scrollbar (no fattening — simple color change on hover).
#[allow(clippy::too_many_arguments)]
fn draw_vscrollbar(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    active: bool,
    search_match_starts: &[u64],
    cursor_addr: u64,
    total_len: u64,
) {
    let track = scrollbar_track(bounds, viewport_h);
    let thumb = scrollbar_thumb(track, scroll, total_h);

    // Track background.
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

    // Search-match markers (small green dots).
    for &match_start in search_match_starts {
        let my = scrollbar_y_frac(match_start, total_len, track);
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: track.x + (track.width - MARKER_SIZE) / 2.0,
                    y: my - MARKER_SIZE / 2.0,
                    width: MARKER_SIZE,
                    height: MARKER_SIZE,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x4a7a2a)),
        );
    }

    // Cursor-position marker (amber dot).
    let cy = scrollbar_y_frac(cursor_addr, total_len, track);
    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: track.x + (track.width - MARKER_SIZE) / 2.0,
                y: cy - MARKER_SIZE / 2.0,
                width: MARKER_SIZE,
                height: MARKER_SIZE,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color!(0xB97024)),
    );

    // Thumb.
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

/// Draw the horizontal scrollbar.
fn draw_hscrollbar(
    renderer: &mut iced::Renderer,
    track: Rectangle,
    scroll_x: f32,
    content_w: f32,
    avail_w: f32,
    active: bool,
) {
    let thumb = hscrollbar_thumb(track, scroll_x, content_w, avail_w);

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
