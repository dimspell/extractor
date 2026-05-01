//! Custom virtualised table widget — Approach B from
//! `docs/spreadsheet_scroll_perf.md`.
//!
//! Replaces the `column(of lazy(of button(of row(of cells))))` tree in
//! `build_table_content` with a single widget that owns viewport state and
//! draws cells directly. Only rows that intersect the visible bounds are
//! shaped; everything else is skipped. Reuses [`ParagraphCache`] from
//! `cached_text` so the shaped paragraphs survive viewport changes.
//!
//! Lives behind the `table_widget` cargo feature so the existing lazy/column
//! path remains the default until step 8 deletes it.
//!
//! ## Data layout
//!
//! The widget borrows two slices: `display_cache[orig_idx][col_idx]` (full
//! row data, parallel to the catalog) and `filtered_indices[visible_idx] =
//! orig_idx` (which rows to show, in order).
//!
//! ## External vs internal scroll
//!
//! Scroll offset is owned by widget state, but each `view()` rebuild passes
//! an `external_offset` from `SpreadsheetState`. When that differs from the
//! last value the widget saw, the widget snaps to it. This is the path
//! programmatic navigation (`NavigateUp`, `NavigateNextHighlight`, …) takes
//! to move the viewport: the message handler sets `vertical_scroll_offset`
//! on the state, and the widget reads it next frame.

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{
    alignment, color, mouse, Background, Border, Color, Element, Event, Font, Length, Pixels,
    Point, Rectangle, Shadow, Size, Vector,
};

use crate::view::editor::cached_text::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

/// Width in pixels of the painted scrollbar thumbs along the right and
/// bottom edges of the table.
const SCROLLBAR_THICKNESS: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
pub struct TableColumn {
    pub width_px: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RowFlags {
    pub selected: bool,
    pub highlighted: bool,
    pub current_highlight: bool,
}

pub struct TableWidget<'a, Message> {
    /// Full display cache — `display_cache[orig_idx][col_idx]`.
    display_cache: &'a [Vec<String>],
    /// Visible rows in display order: `filtered_indices[visible_idx] = orig_idx`.
    filtered_indices: &'a [usize],
    /// Owned column widths. Rebuilt per view; cheap because N ≈ 20.
    columns: Vec<TableColumn>,
    /// Width of the leading id column (rendered as `format!("{}", orig_idx + 1)`).
    id_col_width: f32,
    /// Owned closure producing flags for a given visible-row index.
    row_flags: Box<dyn Fn(usize) -> RowFlags + 'a>,
    row_height: f32,
    cache: ParagraphCache,
    text_size: f32,
    font: Font,
    cell_padding_x: f32,
    width: Length,
    height: Length,
    /// Scroll offset the parent state currently holds. The widget snaps its
    /// internal offset to this value whenever it changes (allowing
    /// programmatic scroll-to-row from the message layer).
    external_offset: Vector,
    on_select: Option<Box<dyn Fn(usize) -> Message + 'a>>,
    on_scroll: Option<Box<dyn Fn(f32, f32) -> Message + 'a>>,
}

#[derive(Default)]
struct State {
    scroll_offset: Vector,
    /// Most recent `external_offset` we synced to. `None` until the first
    /// frame.
    last_external: Option<Vector>,
    hovered_row: Option<usize>,
}

impl<'a, Message> TableWidget<'a, Message> {
    pub fn new(
        display_cache: &'a [Vec<String>],
        filtered_indices: &'a [usize],
        columns: Vec<TableColumn>,
        id_col_width: f32,
        row_flags: impl Fn(usize) -> RowFlags + 'a,
        row_height: f32,
        cache: ParagraphCache,
    ) -> Self {
        Self {
            display_cache,
            filtered_indices,
            columns,
            id_col_width,
            row_flags: Box::new(row_flags),
            row_height,
            cache,
            text_size: 10.0,
            font: Font::MONOSPACE,
            cell_padding_x: 8.0,
            width: Length::Fill,
            height: Length::Fill,
            external_offset: Vector::new(0.0, 0.0),
            on_select: None,
            on_scroll: None,
        }
    }

    pub fn on_select(mut self, f: impl Fn(usize) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn on_scroll(mut self, f: impl Fn(f32, f32) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    pub fn external_offset(mut self, x: f32, y: f32) -> Self {
        self.external_offset = Vector::new(x, y);
        self
    }

    pub fn text_size(mut self, size: f32) -> Self {
        self.text_size = size;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn cell_padding_x(mut self, px: f32) -> Self {
        self.cell_padding_x = px;
        self
    }

    fn n_rows(&self) -> usize {
        self.filtered_indices.len()
    }

    fn n_cols(&self) -> usize {
        // +1 for the leading id column.
        self.columns.len() + 1
    }

    fn col_width(&self, col_idx: usize) -> f32 {
        if col_idx == 0 {
            self.id_col_width
        } else {
            self.columns[col_idx - 1].width_px
        }
    }

    fn total_width(&self) -> f32 {
        self.id_col_width + self.columns.iter().map(|c| c.width_px).sum::<f32>()
    }

    fn total_height(&self) -> f32 {
        self.n_rows() as f32 * self.row_height
    }

    fn cell_value(&self, visible_idx: usize, col_idx: usize) -> Option<String> {
        let orig_idx = *self.filtered_indices.get(visible_idx)?;
        if col_idx == 0 {
            Some(format!("{}", orig_idx + 1))
        } else {
            self.display_cache
                .get(orig_idx)
                .and_then(|row| row.get(col_idx - 1))
                .cloned()
        }
    }

    /// Sync `state.scroll_offset` to `external_offset` when the parent state
    /// has moved (programmatic scroll). Idempotent — only triggers when the
    /// external value differs from what we last observed, so the user's own
    /// wheel-scroll updates aren't clobbered.
    fn sync_external(&self, state: &mut State, bounds: Size) {
        let need_sync = state
            .last_external
            .is_none_or(|last| last != self.external_offset);
        if !need_sync {
            return;
        }
        let total_w = self.total_width();
        let total_h = self.total_height();
        let max_x = (total_w - bounds.width).max(0.0);
        let max_y = (total_h - bounds.height).max(0.0);
        state.scroll_offset.x = self.external_offset.x.clamp(0.0, max_x);
        state.scroll_offset.y = self.external_offset.y.clamp(0.0, max_y);
        state.last_external = Some(self.external_offset);
    }
}

// ── Style palette (pulled from style::spreadsheet_row) ────────────────────
fn cell_text_color(flags: RowFlags) -> Color {
    if flags.current_highlight {
        color!(0xffffff)
    } else if flags.highlighted {
        color!(0xfff2c0)
    } else if flags.selected {
        color!(0xffd700)
    } else {
        color!(0xd4c5a9)
    }
}

fn id_text_color(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0xffd700)
    } else {
        color!(0x6a5e54)
    }
}

fn id_cell_bg(flags: RowFlags) -> Color {
    if flags.selected || flags.current_highlight {
        color!(0x2a2218)
    } else {
        color!(0x171411)
    }
}

fn row_bg(visible_idx: usize, flags: RowFlags, hovered: bool) -> Color {
    if flags.current_highlight {
        color!(0x7a6a2a)
    } else if flags.highlighted {
        color!(0x5a4e1a)
    } else if flags.selected {
        color!(0x3a2e1a)
    } else if hovered {
        color!(0x2d2820)
    } else if visible_idx.is_multiple_of(2) {
        color!(0x1e1b17)
    } else {
        color!(0x232019)
    }
}

fn row_border(flags: RowFlags) -> Option<(Color, f32)> {
    if flags.current_highlight {
        Some((color!(0xffd700, 0.85), 2.0))
    } else if flags.highlighted {
        Some((color!(0xdaa520, 0.7), 1.0))
    } else if flags.selected {
        Some((color!(0xdaa520, 0.5), 1.0))
    } else {
        None
    }
}

impl<Message, Theme> Widget<Message, Theme, iced::Renderer> for TableWidget<'_, Message> {
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
        let max = limits.max();
        // Sync external scroll into widget state on every layout so
        // programmatic moves take effect even if no events fire.
        let state = tree.state.downcast_mut::<State>();
        self.sync_external(state, max);
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

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let (dx, dy) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        (-x * self.row_height, -y * self.row_height)
                    }
                    mouse::ScrollDelta::Pixels { x, y } => (-x, -y),
                };
                let total_w = self.total_width();
                let total_h = self.total_height();
                let new_x = (state.scroll_offset.x + dx)
                    .clamp(0.0, (total_w - bounds.width).max(0.0));
                let new_y = (state.scroll_offset.y + dy)
                    .clamp(0.0, (total_h - bounds.height).max(0.0));
                if (new_x - state.scroll_offset.x).abs() > f32::EPSILON
                    || (new_y - state.scroll_offset.y).abs() > f32::EPSILON
                {
                    state.scroll_offset.x = new_x;
                    state.scroll_offset.y = new_y;
                    // Reflect the wheel-driven offset back as the new
                    // "external" baseline so the next view rebuild doesn't
                    // think the parent has moved us.
                    state.last_external = Some(state.scroll_offset);
                    shell.request_redraw();
                    if let Some(cb) = &self.on_scroll {
                        shell.publish(cb(new_x, new_y));
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let new_hover = cursor.position_over(bounds).and_then(|p| {
                    let local_y = (p.y - bounds.y) + state.scroll_offset.y;
                    if local_y < 0.0 {
                        return None;
                    }
                    let row = (local_y / self.row_height) as usize;
                    if row >= self.n_rows() {
                        None
                    } else {
                        Some(row)
                    }
                });
                if new_hover != state.hovered_row {
                    state.hovered_row = new_hover;
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                let local_y = (p.y - bounds.y) + state.scroll_offset.y;
                if local_y < 0.0 {
                    return;
                }
                let row = (local_y / self.row_height) as usize;
                if row >= self.n_rows() {
                    return;
                }
                if let Some(cb) = &self.on_select {
                    shell.publish(cb(row));
                    shell.capture_event();
                }
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
            mouse::Interaction::Pointer
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
        let off = state.scroll_offset;

        if self.n_rows() == 0 || self.n_cols() == 0 {
            return;
        }

        let clip = bounds.intersection(viewport).unwrap_or(bounds);

        // Backdrop — paint the table body with the default zebra-base colour
        // so columns that don't fully fill the row leave a clean background.
        renderer.fill_quad(
            renderer::Quad {
                bounds: clip,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x1e1b17)),
        );

        // ── Visible row range ──────────────────────────────────────────────
        let first_row = ((off.y / self.row_height).floor() as usize).min(self.n_rows());
        let last_row = (((off.y + bounds.height) / self.row_height).ceil() as usize)
            .min(self.n_rows());

        // ── Visible column range (cumulative x prefix sum) ─────────────────
        let n_cols = self.n_cols();
        let mut col_x: Vec<f32> = Vec::with_capacity(n_cols + 1);
        let mut acc = 0.0f32;
        col_x.push(0.0);
        for c in 0..n_cols {
            acc += self.col_width(c);
            col_x.push(acc);
        }
        let first_col = col_x
            .partition_point(|&x| x <= off.x)
            .saturating_sub(1)
            .min(n_cols.saturating_sub(1));
        let last_col = col_x
            .partition_point(|&x| x < off.x + bounds.width)
            .min(n_cols);

        for row_idx in first_row..last_row {
            let y = bounds.y + (row_idx as f32 * self.row_height) - off.y;
            let flags = (self.row_flags)(row_idx);
            let is_hovered = state.hovered_row == Some(row_idx);

            // Row background (full-width band).
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: clip.x,
                        y,
                        width: clip.width,
                        height: self.row_height,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(row_bg(row_idx, flags, is_hovered)),
            );

            // ── Cells ──────────────────────────────────────────────────────
            for col_idx in first_col..last_col {
                let cell_x = bounds.x + col_x[col_idx] - off.x;
                let cell_w = self.col_width(col_idx);
                let is_id_col = col_idx == 0;

                // The id column gets its own darker frozen-style background
                // painted on top of the row band.
                if is_id_col {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: cell_x.max(clip.x),
                                y,
                                width: cell_w.min(clip.x + clip.width - cell_x).max(0.0),
                                height: self.row_height,
                            },
                            border: Border {
                                color: color!(0x3d2b1f),
                                width: 0.5,
                                radius: 0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(id_cell_bg(flags)),
                    );
                }

                let value = match self.cell_value(row_idx, col_idx) {
                    Some(v) if !v.is_empty() => v,
                    _ => continue,
                };

                let key = ParagraphKey::new(&value, self.text_size, cell_w, self.font);
                let paragraph = self.cache.get_or_insert(key, || {
                    Paragraph::with_text(text::Text {
                        content: value.as_str(),
                        bounds: Size::new(cell_w, self.row_height),
                        size: Pixels(self.text_size),
                        line_height: text::LineHeight::default(),
                        font: self.font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Center,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    })
                });

                let position = Point::new(
                    cell_x + self.cell_padding_x,
                    y + self.row_height / 2.0,
                );
                let text_color = if is_id_col {
                    id_text_color(flags)
                } else {
                    cell_text_color(flags)
                };
                <iced::Renderer as text::Renderer>::fill_paragraph(
                    renderer,
                    &paragraph,
                    position,
                    text_color,
                    clip,
                );
            }

            // Row border (selection / highlight outline). Drawn last so it
            // overlays the cell backgrounds.
            if let Some((color, width)) = row_border(flags) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: clip.x,
                            y,
                            width: clip.width,
                            height: self.row_height,
                        },
                        border: Border {
                            color,
                            width,
                            radius: 0.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
        }

        // ── Scrollbars ─────────────────────────────────────────────────────
        draw_scrollbars(renderer, clip, off, self.total_width(), self.total_height());
    }
}

/// Paint vertical and horizontal scrollbar thumbs along the right and bottom
/// edges of `bounds` to reflect `off` against the total content size.
fn draw_scrollbars(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    off: Vector,
    total_w: f32,
    total_h: f32,
) {
    let track_color = color!(0x141210);
    let thumb_color = color!(0x5d4037);

    if total_h > bounds.height {
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: bounds.y,
            width: SCROLLBAR_THICKNESS,
            height: bounds.height,
        };
        let thumb_h = (bounds.height / total_h * bounds.height).max(20.0);
        let max_off = (total_h - bounds.height).max(1.0);
        let thumb_y = bounds.y + (off.y / max_off) * (bounds.height - thumb_h);
        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: track.x + 1.0,
                    y: thumb_y,
                    width: SCROLLBAR_THICKNESS - 2.0,
                    height: thumb_h,
                },
                border: Border {
                    color: color!(0x8b5a2b),
                    width: 0.5,
                    radius: (SCROLLBAR_THICKNESS / 2.0).into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(thumb_color),
        );
    }

    if total_w > bounds.width {
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: bounds.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (bounds.width / total_w * bounds.width).max(20.0);
        let max_off = (total_w - bounds.width).max(1.0);
        let thumb_x = bounds.x + (off.x / max_off) * (bounds.width - thumb_w);
        renderer.fill_quad(
            renderer::Quad {
                bounds: track,
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(track_color),
        );
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: thumb_x,
                    y: track.y + 1.0,
                    width: thumb_w,
                    height: SCROLLBAR_THICKNESS - 2.0,
                },
                border: Border {
                    color: color!(0x8b5a2b),
                    width: 0.5,
                    radius: (SCROLLBAR_THICKNESS / 2.0).into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(thumb_color),
        );
    }
}

impl<'a, Message, Theme> From<TableWidget<'a, Message>>
    for Element<'a, Message, Theme, iced::Renderer>
where
    Theme: 'a,
    Message: 'a,
{
    fn from(w: TableWidget<'a, Message>) -> Self {
        Element::new(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_flags(_: usize) -> RowFlags {
        RowFlags::default()
    }

    #[test]
    fn empty_table_does_not_panic() {
        let cache = ParagraphCache::default();
        let _w: TableWidget<'_, ()> =
            TableWidget::new(&[], &[], vec![], 42.0, no_flags, 24.0, cache);
    }

    #[test]
    fn total_dimensions_include_id_column() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into(), "b".into()]; 5];
        let filtered: Vec<usize> = (0..5).collect();
        let cols = vec![
            TableColumn { width_px: 100.0 },
            TableColumn { width_px: 200.0 },
        ];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
        assert_eq!(w.total_width(), 42.0 + 100.0 + 200.0);
        assert_eq!(w.total_height(), 5.0 * 24.0);
    }

    #[test]
    fn cell_value_id_column_uses_orig_idx() {
        let display = vec![vec!["a".into()]; 3];
        let filtered = vec![2, 0, 1];
        let cols = vec![TableColumn { width_px: 100.0 }];
        let cache = ParagraphCache::default();
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache);
        assert_eq!(w.cell_value(0, 0).as_deref(), Some("3"));
        assert_eq!(w.cell_value(1, 0).as_deref(), Some("1"));
        assert_eq!(w.cell_value(0, 1).as_deref(), Some("a"));
    }

    #[test]
    fn sync_external_clamps_to_content() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into()]; 100];
        let filtered: Vec<usize> = (0..100).collect();
        let cols = vec![TableColumn { width_px: 100.0 }];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
                .external_offset(0.0, 100_000.0);
        let mut state = State::default();
        let bounds = Size::new(200.0, 240.0);
        w.sync_external(&mut state, bounds);
        // total_h = 100 * 24 = 2400; max_y = 2400 - 240 = 2160.
        assert_eq!(state.scroll_offset.y, 2160.0);
        assert_eq!(state.last_external, Some(Vector::new(0.0, 100_000.0)));
    }

    #[test]
    fn sync_external_idempotent() {
        let cache = ParagraphCache::default();
        let display: Vec<Vec<String>> = vec![vec!["a".into()]; 50];
        let filtered: Vec<usize> = (0..50).collect();
        let cols = vec![TableColumn { width_px: 100.0 }];
        let w: TableWidget<'_, ()> =
            TableWidget::new(&display, &filtered, cols, 42.0, no_flags, 24.0, cache)
                .external_offset(10.0, 20.0);
        let mut state = State::default();
        let bounds = Size::new(200.0, 240.0);
        w.sync_external(&mut state, bounds);
        // Now mutate scroll_offset (simulating wheel scroll) and call again
        // with the same external — the wheel value must stick.
        state.scroll_offset.y = 50.0;
        w.sync_external(&mut state, bounds);
        assert_eq!(state.scroll_offset.y, 50.0);
    }

    #[test]
    fn cell_text_color_priority() {
        // current_highlight beats highlighted beats selected.
        let f = RowFlags {
            current_highlight: true,
            highlighted: true,
            selected: true,
        };
        assert_eq!(cell_text_color(f), color!(0xffffff));
    }
}
