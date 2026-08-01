//! Rendering for the dual-buffer diff view.
//!
//! Inspired by the matrix's `draw.rs` but adapted for two byte sources with
//! a shared address column and diff-coloured cells on both sides.

use std::collections::BTreeSet;

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::Layout;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::Renderer as _;
use iced::mouse;
use iced::{alignment, Background, Border, Color, Font, Pixels, Rectangle, Shadow, Size};

use gui_widgets::components::paragraph_cache::{ParagraphCache, ParagraphKey};

use crate::coloring::{default_byte_colors, ColorScheme};
use crate::ui::theme::HexEditorTheme;
use crate::ui::view::minimap;

use super::layout::{
    self, visible_row_range, ADDR_COL_WIDTH, ANN_COL_GAP, ASCII_CELL_WIDTH, GROUP_GAP,
    HEADER_HEIGHT, HEX_CELL_WIDTH, MAX_ANN_COL_WIDTH, ROW_HEIGHT, SCROLLBAR_THICKNESS, TEXT_SIZE,
};
use super::state::State;
use super::DiffView;

type Paragraph = GraphicsParagraph;

/// Render the entire diff view into the given viewport.
pub fn draw_diff_view<'a, Message>(
    widget: &DiffView<'a, Message>,
    state: &State,
    renderer: &mut iced::Renderer,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
) {
    let bounds = layout.bounds();
    let full_clip = bounds.intersection(viewport).unwrap_or(bounds);

    // ── Background ──────────────────────────────────────────────────────
    renderer.fill_quad(
        renderer::Quad {
            bounds: full_clip,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(widget.theme.matrix_bg),
    );

    // ── Geometry ────────────────────────────────────────────────────────
    let bpr = widget.bytes_per_row as usize;
    let bpr64 = widget.bytes_per_row as u64;
    let total_rows = widget.total_rows();

    // Use the longer of the two buffers for row count.
    let total_bytes = widget
        .baseline_bytes
        .len()
        .max(widget.comparison_bytes.len());
    let total_h = (total_bytes.div_ceil(bpr) as f32) * ROW_HEIGHT;

    let content_top = bounds.y + HEADER_HEIGHT;
    let viewport_h = widget.content_viewport_h(bounds.height, bounds.width);
    let content_bounds = Rectangle {
        x: bounds.x,
        y: content_top,
        width: bounds.width,
        height: viewport_h,
    };

    let content_clip_y = content_top.max(full_clip.y);
    let content_clip_bottom = (content_top + viewport_h).min(full_clip.y + full_clip.height);
    let content_clip = Rectangle {
        x: full_clip.x,
        y: content_clip_y,
        width: (full_clip.width - widget.right_strip()).max(0.0),
        height: (content_clip_bottom - content_clip_y).max(0.0),
    };

    // Further clip hex/ASCII cells to exclude the address gutter, so
    // horizontally-scrolled cells don't paint over the address column.
    let cell_clip = Rectangle {
        x: content_clip.x.max(bounds.x + ADDR_COL_WIDTH),
        y: content_clip.y,
        width: (content_clip.x + content_clip.width
            - content_clip.x.max(bounds.x + ADDR_COL_WIDTH))
        .max(0.0),
        height: content_clip.height,
    };

    let font = Font::MONOSPACE;
    let header_color = widget.theme.header_fg;
    let header_separator = widget.theme.header_separator;
    let addr_color = widget.theme.address_fg;
    let hex_color = widget.theme.hex_fg;
    let ascii_color = widget.theme.ascii_fg;
    let selection_bg = widget.theme.selection_bg;
    let selection_fg = widget.theme.selection_fg;
    let cursor_bg = widget.theme.cursor_bg;

    // Diff colours: baseline side gets a warm (reddish) tint, comparison
    // side gets a cool (greenish) tint — like most diff tools.
    // Derived from the theme's generic diff_bg/diff_fg.
    let diff_bg = widget.theme.diff_bg;
    let (dr, dg, db) = (diff_bg.r, diff_bg.g, diff_bg.b);
    // Baseline tint: shift toward red-orange
    let diff_bg_baseline = Color::from_rgb(
        (dr + 0.25).min(1.0),
        (dg * 0.4).max(0.0),
        (db * 0.3).max(0.0),
    );
    // Comparison tint: shift toward green
    let diff_bg_comparison = Color::from_rgb(
        (dr * 0.4).max(0.0),
        (dg + 0.25).min(1.0),
        (db * 0.4).max(0.0),
    );
    let diff_text_baseline = Color::from_rgb(1.0, 0.7, 0.7);
    let diff_text_comparison = Color::from_rgb(0.7, 1.0, 0.7);

    let sel_range = widget.selection.range();
    let cursor_addr = widget.selection.cursor;

    // ── Center-on-scroll request (one-shot, set by ◀▶ nav buttons) ──
    if let Some(addr) = widget.pending_center_on.take() {
        let scroll = layout::center_scroll_on(
            state.scroll_offset.get(),
            addr,
            bpr64,
            viewport_h.max(1.0),
            total_h,
        );
        state.scroll_offset.set(scroll);
    }

    let scroll = state.scroll_offset.get();
    let scroll_x = state.scroll_x.get();
    let adj_addr_col_w = ADDR_COL_WIDTH;

    // ── Column headers ─────────────────────────────────────────────────
    let header_y = bounds.y;
    draw_column_headers(
        renderer,
        &widget.cache,
        bounds,
        header_y,
        header_color,
        full_clip,
        bpr,
        font,
        scroll_x,
    );

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: bounds.x,
                y: header_y + HEADER_HEIGHT - 1.0,
                width: bounds.width,
                height: 1.0,
            },
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(header_separator),
    );

    // ── Visible row range ─────────────────────────────────────────────
    let visible = visible_row_range(scroll, viewport_h, ROW_HEIGHT, total_rows, layout::OVERSCAN);

    // ── Data rows ──────────────────────────────────────────────────────
    for row_idx in visible.clone() {
        let base_addr = row_idx as u64 * bpr64;

        // "Show Diffs Only" — skip rows with zero differing bytes.
        if widget.diff_review && !widget.row_has_diff(base_addr) {
            continue;
        }

        let y = content_bounds.y + (row_idx as f32 * ROW_HEIGHT) - scroll;

        // ── Address gutter ─────────────────────────────────────────────
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
                x: bounds.x + adj_addr_col_w - 8.0 - text_w,
                y,
                width: text_w,
                height: ROW_HEIGHT,
            },
            addr_color,
            content_clip,
        );

        // ── Baseline side (A) ──────────────────────────────────────────
        let row_bytes_a = widget
            .baseline_bytes
            .get(base_addr as usize..)
            .and_then(|s| s.get(..bpr))
            .unwrap_or(&[]);

        let hex_a_start = layout::baseline_hex_start(adj_addr_col_w) - scroll_x;
        let ascii_a_start = layout::baseline_ascii_start(adj_addr_col_w, bpr) - scroll_x;

        for (col, &b) in row_bytes_a.iter().enumerate() {
            let addr = base_addr + col as u64;
            render_byte_cell(
                renderer,
                widget,
                font,
                addr,
                b,
                col,
                y,
                bpr,
                hex_a_start,
                ascii_a_start,
                sel_range.clone(),
                cursor_addr,
                diff_bg_baseline,
                diff_text_baseline,
                selection_bg,
                selection_fg,
                cursor_bg,
                hex_color,
                ascii_color,
                cell_clip,
            );
        }

        // ── Comparison side (B) ────────────────────────────────────────
        let row_bytes_b = widget
            .comparison_bytes
            .get(base_addr as usize..)
            .and_then(|s| s.get(..bpr))
            .unwrap_or(&[]);

        let hex_b_start = layout::comparison_hex_start(adj_addr_col_w, bpr) - scroll_x;
        let ascii_b_start = layout::comparison_ascii_start(adj_addr_col_w, bpr) - scroll_x;

        for (col, &b) in row_bytes_b.iter().enumerate() {
            let addr = base_addr + col as u64;
            render_byte_cell(
                renderer,
                widget,
                font,
                addr,
                b,
                col,
                y,
                bpr,
                hex_b_start,
                ascii_b_start,
                sel_range.clone(),
                cursor_addr,
                diff_bg_comparison,
                diff_text_comparison,
                selection_bg,
                selection_fg,
                cursor_bg,
                hex_color,
                ascii_color,
                cell_clip,
            );
        }

        // ── Annotation column (right side, same as matrix) ─────────────
        if !widget.row_annotations.is_empty() {
            if let Some(segments) = widget.row_annotations.get(&base_addr) {
                let ann_start_x = layout::comparison_ascii_start(adj_addr_col_w, bpr)
                    + bpr as f32 * ASCII_CELL_WIDTH
                    + ANN_COL_GAP
                    - scroll_x;
                let mut ann_x = ann_start_x;
                for (pid, text) in segments {
                    let is_active = widget.active_patterns.contains(pid);
                    let ann_color = if is_active {
                        widget.theme.annotation_fg
                    } else {
                        widget.theme.annotation_inactive
                    };
                    let prefix = if is_active { "▸" } else { " " };
                    let label = format!("{prefix}{text}");
                    draw_glyph_string(
                        renderer,
                        &widget.cache,
                        &label,
                        font,
                        Rectangle {
                            x: ann_x,
                            y,
                            width: MAX_ANN_COL_WIDTH,
                            height: ROW_HEIGHT,
                        },
                        ann_color,
                        content_clip,
                    );
                    ann_x += label.len() as f32 * ASCII_CELL_WIDTH + 4.0;
                }
            }
        }
    }

    // ── Minimap overview strip (between content and scrollbar) ──────────
    let needs_vscroll = total_h > viewport_h;
    if needs_vscroll && widget.show_minimap {
        let total_len = total_bytes as u64;
        let h_px = viewport_h.max(1.0) as u32;
        let empty_dirty = BTreeSet::new();
        let ctx = minimap::BlockContext {
            bytes: widget.baseline_bytes,
            total_len,
            pattern_by_addr: widget.patterns,
            alternate_patterns: &widget.alternate_patterns,
            dirty: &empty_dirty,
            vanilla_diff: widget.diff,
            color_scheme: widget.color_scheme,
            dim_nulls: widget.dim_nulls,
            theme: widget.theme,
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
                dirty_fingerprint: minimap::set_fingerprint(&empty_dirty),
                diff_fingerprint: minimap::set_fingerprint(widget.diff),
                content_ptr: widget.baseline_bytes.as_ptr() as usize,
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
            widget.theme,
        );
    }

    // ── Scrollbars ──────────────────────────────────────────────────────
    // Bucket diff addresses to row-level to avoid rendering one marker per
    // differing byte (which can be tens of thousands for large diffs).
    let mut diff_rows: Vec<u64> = Vec::new();
    if !widget.diff.is_empty() {
        let mut prev_row = u64::MAX;
        for &addr in widget.diff {
            let row = addr / bpr as u64;
            if row != prev_row {
                prev_row = row;
                diff_rows.push(row * bpr as u64);
            }
        }
    }
    draw_vscrollbar(
        renderer,
        content_bounds,
        scroll,
        total_h,
        viewport_h,
        state.dragging_scrollbar || state.hovering_scrollbar.get(),
        widget.search_match_starts,
        &diff_rows,
        cursor_addr,
        total_bytes as u64,
        widget.theme,
    );
    let content_w = layout::total_content_width(bpr, !widget.row_annotations.is_empty());
    let avail_w = bounds.width - widget.right_strip();
    if content_w > avail_w {
        let htrack = hscrollbar_track(bounds);
        let hovering = cursor
            .position_over(bounds)
            .map(|p| htrack.contains(p))
            .unwrap_or(false);
        draw_hscrollbar(
            renderer,
            htrack,
            state.scroll_x.get(),
            content_w,
            avail_w,
            state.dragging_scrollbar_x || hovering,
            widget.theme,
        );
    }
}

// ── Per-byte cell rendering ─────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_byte_cell<Message>(
    renderer: &mut iced::Renderer,
    widget: &DiffView<'_, Message>,
    font: Font,
    addr: u64,
    byte: u8,
    col: usize,
    y: f32,
    _bpr: usize,
    hex_start_x: f32,
    ascii_start_x: f32,
    sel_range: std::ops::RangeInclusive<u64>,
    cursor_addr: u64,
    diff_bg: Color,
    diff_text: Color,
    selection_bg: Color,
    selection_fg: Color,
    cursor_bg: Color,
    hex_color: Color,
    ascii_color: Color,
    clip: Rectangle,
) {
    let group = col / 8;
    let cell_x = hex_start_x + col as f32 * HEX_CELL_WIDTH + group as f32 * GROUP_GAP;
    let ax = ascii_start_x + col as f32 * ASCII_CELL_WIDTH;

    let in_sel = sel_range.contains(&addr);
    let is_diff = widget.diff.contains(&addr);
    let pat_entry = widget.patterns.get(&addr).copied();

    // Background priority: selection > cursor > pattern > diff > none.
    let base_bg = if in_sel {
        let bg = if addr == cursor_addr {
            cursor_bg
        } else {
            selection_bg
        };
        Some(bg)
    } else if let Some((pid, color_idx)) = pat_entry {
        let mut bg = widget.theme.pattern_bg_palette[color_idx as usize % 16];
        if widget.alternate_patterns.contains(&pid) {
            bg.r *= 0.5;
            bg.g *= 0.5;
            bg.b *= 0.5;
        }
        Some(bg)
    } else if is_diff {
        Some(diff_bg)
    } else {
        None
    };

    // Foreground via provider chain.
    let (default_fg, _) = default_byte_colors(widget.color_scheme, byte, widget.dim_nulls);
    let default_fg = default_fg.unwrap_or(hex_color);

    let text_color = if in_sel {
        selection_fg
    } else if let Some((_, color_idx)) = pat_entry {
        widget.theme.pattern_fg_palette[color_idx as usize % 16]
    } else if is_diff {
        diff_text
    } else {
        default_fg
    };

    let ascii_col = if in_sel {
        selection_fg
    } else if let Some((_, color_idx)) = pat_entry {
        widget.theme.pattern_fg_palette[color_idx as usize % 16]
    } else if is_diff {
        diff_text
    } else if widget.color_scheme != ColorScheme::Monochrome {
        default_fg
    } else {
        ascii_color
    };

    // Search-match overlay.
    let in_search = widget.search_match_set.contains(&addr);
    let in_current_match = widget
        .search_current_addr
        .map(|cur| addr >= cur && addr < cur + widget.search_query_len)
        .unwrap_or(false);
    let bg = if in_current_match {
        Some(widget.theme.search_current_bg)
    } else if in_search {
        Some(widget.theme.search_match_bg)
    } else {
        base_bg
    };
    let text_color = if in_current_match {
        widget.theme.search_current_fg
    } else if in_search {
        widget.theme.search_match_fg
    } else {
        text_color
    };

    if let Some(c) = bg {
        fill_cell(renderer, cell_x, y, HEX_CELL_WIDTH, c, clip);
        fill_cell(renderer, ax, y, ASCII_CELL_WIDTH, c, clip);
    }

    // Hex glyph.
    let hex_str = format!("{:02X}", byte);
    draw_glyph_string(
        renderer,
        &widget.cache,
        &hex_str,
        font,
        Rectangle {
            x: cell_x + 1.0,
            y,
            width: HEX_CELL_WIDTH - 2.0,
            height: ROW_HEIGHT,
        },
        text_color,
        clip,
    );

    // ASCII glyph.
    let ascii_char = if byte.is_ascii_graphic() || byte == b' ' {
        char::from(byte).to_string()
    } else {
        "·".to_string()
    };
    draw_glyph_string(
        renderer,
        &widget.cache,
        &ascii_char,
        font,
        Rectangle {
            x: ax,
            y,
            width: ASCII_CELL_WIDTH,
            height: ROW_HEIGHT,
        },
        ascii_col,
        clip,
    );
}

// ── Column headers ───────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_column_headers(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    bounds: Rectangle,
    y: f32,
    color: Color,
    clip: Rectangle,
    bpr: usize,
    font: Font,
    scroll_x: f32,
) {
    let labels = [
        ("Address", bounds.x, ADDR_COL_WIDTH),
        (
            "Hex (A)",
            layout::baseline_hex_start(ADDR_COL_WIDTH) - scroll_x,
            bpr as f32 * HEX_CELL_WIDTH + layout::group_count(bpr) as f32 * GROUP_GAP,
        ),
        (
            "ASCII",
            layout::baseline_ascii_start(ADDR_COL_WIDTH, bpr) - scroll_x,
            bpr as f32 * ASCII_CELL_WIDTH,
        ),
        (
            "Hex (B)",
            layout::comparison_hex_start(ADDR_COL_WIDTH, bpr) - scroll_x,
            bpr as f32 * HEX_CELL_WIDTH + layout::group_count(bpr) as f32 * GROUP_GAP,
        ),
        (
            "ASCII",
            layout::comparison_ascii_start(ADDR_COL_WIDTH, bpr) - scroll_x,
            bpr as f32 * ASCII_CELL_WIDTH,
        ),
    ];
    for (label, x, w) in &labels {
        draw_glyph_string(
            renderer,
            cache,
            label,
            font,
            Rectangle {
                x: *x + 2.0,
                y,
                width: *w - 4.0,
                height: HEADER_HEIGHT,
            },
            color,
            clip,
        );
    }
}

// ── Drawing helpers ──────────────────────────────────────────────────────

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
    if let Some(rect) = clip.intersection(&cell) {
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
            ellipsis: text::Ellipsis::None,
            hint_factor: None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    if let Some(cell_clip) = clip.intersection(&bounds) {
        <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
    }
}

// ── Scrollbar rendering ──────────────────────────────────────────────────

const MARKER_SIZE: f32 = 4.0;

fn scrollbar_track(bounds: Rectangle, viewport_h: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
        y: bounds.y,
        width: SCROLLBAR_THICKNESS,
        height: viewport_h,
    }
}

fn thumb_height(track: Rectangle, total_h: f32) -> f32 {
    (track.height / total_h * track.height).max(20.0)
}

fn scrollbar_y_frac(addr: u64, total_len: u64, track: Rectangle) -> f32 {
    if total_len <= 1 {
        return track.y;
    }
    track.y + (addr as f32 / (total_len - 1) as f32) * track.height
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

#[allow(clippy::too_many_arguments)]
fn draw_vscrollbar(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    scroll: f32,
    total_h: f32,
    viewport_h: f32,
    active: bool,
    search_match_starts: &[u64],
    diff_markers: &[u64],
    cursor_addr: u64,
    total_len: u64,
    theme: &HexEditorTheme,
) {
    let track = scrollbar_track(bounds, viewport_h);
    let thumb = scrollbar_thumb(track, scroll, total_h);

    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(theme.scrollbar_bg),
    );
    let thumb_color = if active {
        theme.scrollbar_thumb_hover
    } else {
        theme.scrollbar_thumb
    };

    // ── Diff markers (skip overlapping ones) ──
    let mut last_y: Option<f32> = None;
    for &diff_addr in diff_markers {
        let my = scrollbar_y_frac(diff_addr, total_len, track);
        if last_y.map_or(true, |ly| (my - ly).abs() >= MARKER_SIZE) {
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
                Background::Color(theme.diff_bg),
            );
            last_y = Some(my);
        }
    }

    // ── Search result markers ──
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
            Background::Color(theme.scrollbar_search_dot),
        );
    }

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
        Background::Color(theme.scrollbar_cursor_dot),
    );

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

fn hscrollbar_track(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x,
        y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
        width: bounds.width - SCROLLBAR_THICKNESS,
        height: SCROLLBAR_THICKNESS,
    }
}

fn hthumb_len(track: Rectangle, content_w: f32, _avail_w: f32) -> f32 {
    (track.width / content_w * track.width).max(20.0)
}

fn hscrollbar_thumb(track: Rectangle, scroll_x: f32, content_w: f32, avail_w: f32) -> Rectangle {
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

fn draw_hscrollbar(
    renderer: &mut iced::Renderer,
    track: Rectangle,
    scroll_x: f32,
    content_w: f32,
    avail_w: f32,
    active: bool,
    theme: &HexEditorTheme,
) {
    let thumb = hscrollbar_thumb(track, scroll_x, content_w, avail_w);
    renderer.fill_quad(
        renderer::Quad {
            bounds: track,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(theme.scrollbar_bg),
    );
    let thumb_color = if active {
        theme.scrollbar_thumb_hover
    } else {
        theme.scrollbar_thumb
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

// ── Helpers for event.rs ─────────────────────────────────────────────────

/// Map a pixel x-coordinate to the byte column index (0..bpr) on the
/// baseline or comparison side. Returns `None` if the click is in the
/// address gutter, mid-gap, or annotation area.
pub fn col_at_x(x: f32, bpr: usize, scroll_x: f32) -> Option<(usize, bool)> {
    let hex_a_start = layout::baseline_hex_start(ADDR_COL_WIDTH) - scroll_x;
    let ascii_a_start = layout::baseline_ascii_start(ADDR_COL_WIDTH, bpr) - scroll_x;
    let comp_hex_start = layout::comparison_hex_start(ADDR_COL_WIDTH, bpr) - scroll_x;
    let comp_ascii_start = layout::comparison_ascii_start(ADDR_COL_WIDTH, bpr) - scroll_x;

    // Baseline hex
    if x >= hex_a_start
        && x < hex_a_start
            + bpr as f32 * HEX_CELL_WIDTH
            + layout::group_count(bpr) as f32 * GROUP_GAP
    {
        let local = x - hex_a_start;
        let effective_col_w = HEX_CELL_WIDTH;
        let mut pos = 0.0;
        for col in 0..bpr {
            if col > 0 && col % 8 == 0 {
                pos += GROUP_GAP;
            }
            if local >= pos && local < pos + effective_col_w {
                return Some((col, true));
            }
            pos += effective_col_w;
        }
        return Some((
            ((local / effective_col_w) as usize).min(bpr.saturating_sub(1)),
            true,
        ));
    }

    // Baseline ASCII
    if x >= ascii_a_start && x < ascii_a_start + bpr as f32 * ASCII_CELL_WIDTH {
        let local = (x - ascii_a_start) / ASCII_CELL_WIDTH;
        return Some((local as usize, true));
    }

    // Comparison hex
    if x >= comp_hex_start
        && x < comp_hex_start
            + bpr as f32 * HEX_CELL_WIDTH
            + layout::group_count(bpr) as f32 * GROUP_GAP
    {
        let local = x - comp_hex_start;
        let effective_col_w = HEX_CELL_WIDTH;
        let mut pos = 0.0;
        for col in 0..bpr {
            if col > 0 && col % 8 == 0 {
                pos += GROUP_GAP;
            }
            if local >= pos && local < pos + effective_col_w {
                return Some((col, false));
            }
            pos += effective_col_w;
        }
        return Some((
            ((local / effective_col_w) as usize).min(bpr.saturating_sub(1)),
            false,
        ));
    }

    // Comparison ASCII
    if x >= comp_ascii_start && x < comp_ascii_start + bpr as f32 * ASCII_CELL_WIDTH {
        let local = (x - comp_ascii_start) / ASCII_CELL_WIDTH;
        return Some((local as usize, false));
    }

    None
}
