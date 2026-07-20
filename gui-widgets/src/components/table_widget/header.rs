//! Column-header strip rendering and hit-testing.
//!
//! Contains the `impl TableWidget` methods for drawing the frozen column
//! headers and the interactive resize / sort / filter regions.

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::Renderer as _;
use iced::Point;
use iced::{alignment, color, Background, Border, Color, Font, Pixels, Rectangle, Shadow, Size};

use super::geometry;
use super::types::{HeaderRegion, State};
use super::widget::TableWidget;
use super::{FILTER_BADGE_WIDTH, FILTER_ICON_WIDTH, RESIZE_HANDLE_WIDTH};
use crate::components::paragraph_cache::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

// ── Hit-test ─────────────────────────────────────────────────────────

impl<'a, Message> TableWidget<'a, Message> {
    /// Hit-test a cursor position against the column-header strip.
    ///
    /// Returns `(data_column_index, region)` when the cursor lands on an
    /// interactive part of a header cell, or `None` when it hits the id
    /// column, the gap between columns, or outside the header strip.
    pub(crate) fn header_hit(
        &self,
        bounds: Rectangle,
        off_x: f32,
        p: Point,
    ) -> Option<(usize, HeaderRegion)> {
        let header_bounds = geometry::header_bounds(bounds, self.row_height);
        if !header_bounds.contains(p) {
            return None;
        }
        let id_w = self.id_col_width.min(bounds.width);
        let id_r = bounds.x + id_w;
        if p.x < id_r {
            return None;
        }
        let local_x = (p.x - id_r) + off_x;
        if local_x < 0.0 {
            return None;
        }
        let mut acc = 0.0_f32;
        for (col, c) in self.columns.iter().enumerate() {
            let col_l = acc;
            let col_r = col_l + c.width_px;
            if local_x < col_r {
                let rel = local_x - col_l;
                let resize_l = c.width_px - RESIZE_HANDLE_WIDTH;
                let filter_btn_l = resize_l - FILTER_ICON_WIDTH;
                let filter_badge_l = if c.has_filter {
                    filter_btn_l - FILTER_BADGE_WIDTH
                } else {
                    filter_btn_l
                };
                let region = if rel >= resize_l {
                    HeaderRegion::Resize
                } else if rel >= filter_btn_l {
                    HeaderRegion::FilterOpen
                } else if c.has_filter && rel >= filter_badge_l {
                    HeaderRegion::FilterBadge
                } else {
                    HeaderRegion::Label
                };
                return Some((col, region));
            }
            acc = col_r;
        }
        None
    }
}

// ── Rendering ────────────────────────────────────────────────────────

impl<'a, Message> TableWidget<'a, Message> {
    /// Draw the frozen column-header strip (data column headers + id `#`
    /// header), including sort indicators, filter badges, and resize handles.
    pub(crate) fn draw_header(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        viewport: &Rectangle,
        state: &State,
    ) {
        let n_cols = geometry::n_cols(&self.columns);
        if n_cols == 0 {
            return;
        }
        let col_x = geometry::col_positions(self.id_col_width, &self.columns);
        let off_x = self.scroll_offset().x;

        let header = geometry::header_bounds(bounds, self.row_height);
        let header_clip = header.intersection(viewport).unwrap_or(header);

        // Header background
        renderer.fill_quad(
            renderer::Quad {
                bounds: header,
                border: Border {
                    color: color!(0x4a3728),
                    width: 1.0,
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x1c1813)),
        );

        let id_w = self.id_col_width.min(bounds.width);
        let header_data_rect = Rectangle {
            x: bounds.x + id_w,
            y: header.y,
            width: (bounds.width - id_w).max(0.0),
            height: header.height,
        };
        let header_data_clip = header_clip
            .intersection(&header_data_rect)
            .unwrap_or(header_data_rect);

        // Visible column range (data columns only, skip id column at index 0)
        let first_col = col_x
            .partition_point(|&x| x <= off_x)
            .saturating_sub(1)
            .min(n_cols.saturating_sub(1));
        let last_col = col_x
            .partition_point(|&x| x < off_x + header_data_rect.width)
            .min(n_cols);

        for (col_idx, &col_x_offset) in col_x
            .iter()
            .enumerate()
            .take(last_col)
            .skip(first_col.max(1))
        {
            let col_l_screen = bounds.x + col_x_offset - off_x;
            let col_w = geometry::col_width(self.id_col_width, &self.columns, col_idx);
            let data_col = col_idx - 1;
            let column = &self.columns[data_col];

            let resize_l = col_l_screen + col_w - RESIZE_HANDLE_WIDTH;
            let filter_btn_l = resize_l - FILTER_ICON_WIDTH;
            let filter_badge_l = if column.has_filter {
                filter_btn_l - FILTER_BADGE_WIDTH
            } else {
                filter_btn_l
            };
            let label_r = filter_badge_l;

            // ── Label hover background ────────────────────────────────
            let label_hovered = state
                .hovered_header
                .is_some_and(|(c, r)| c == data_col && r == HeaderRegion::Label);
            if label_hovered {
                if let Some(r) = header_data_clip.intersection(&Rectangle {
                    x: col_l_screen,
                    y: header.y,
                    width: (label_r - col_l_screen).max(0.0),
                    height: header.height,
                }) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: r,
                            border: Border::default(),
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(color!(0x2d2218)),
                    );
                }
            }

            // ── Column label ──────────────────────────────────────────
            let sort_suffix = match column.sort {
                Some(true) => " ▲",
                Some(false) => " ▼",
                None => "",
            };
            let label = if sort_suffix.is_empty() {
                column.label.clone()
            } else {
                format!("{}{}", column.label, sort_suffix)
            };
            let avail_label_w = (label_r - col_l_screen - self.cell_padding_x * 2.0).max(0.0);
            if avail_label_w > 0.0 {
                let key = ParagraphKey::new(&label, self.text_size, avail_label_w, self.font);
                let para = self.cache.get_or_insert(key, || {
                    Paragraph::with_text(text::Text {
                        content: label.as_str(),
                        bounds: Size::new(avail_label_w, header.height),
                        size: Pixels(self.text_size),
                        line_height: text::LineHeight::default(),
                        font: self.font,
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Basic,
                        wrapping: text::Wrapping::None,
                        ellipsis: text::Ellipsis::None,
                        hint_factor: None,
                    })
                });
                let inner = Rectangle {
                    x: col_l_screen + self.cell_padding_x,
                    y: header.y,
                    width: avail_label_w,
                    height: header.height,
                };
                let pos = inner.anchor(
                    para.min_bounds(),
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                );
                let cell_clip = header_data_clip
                    .intersection(&Rectangle {
                        x: col_l_screen,
                        y: header.y,
                        width: (label_r - col_l_screen).max(0.0),
                        height: header.height,
                    })
                    .unwrap_or(Rectangle {
                        x: col_l_screen,
                        y: header.y,
                        width: 0.0,
                        height: 0.0,
                    });
                <iced::Renderer as text::Renderer>::fill_paragraph(
                    renderer,
                    &para,
                    pos,
                    color!(0xb8a898),
                    cell_clip,
                );
            }

            // ── Filter badge ──────────────────────────────────────────
            if column.has_filter {
                draw_centered_glyph(
                    renderer,
                    &self.cache,
                    "◼",
                    8.0,
                    self.font,
                    Rectangle {
                        x: filter_badge_l,
                        y: header.y,
                        width: FILTER_BADGE_WIDTH,
                        height: header.height,
                    },
                    color!(0xffd700),
                    header_data_clip,
                );
            }

            // ── Filter dropdown icon ──────────────────────────────────
            draw_centered_glyph(
                renderer,
                &self.cache,
                "▾",
                8.0,
                self.font,
                Rectangle {
                    x: filter_btn_l,
                    y: header.y,
                    width: FILTER_ICON_WIDTH,
                    height: header.height,
                },
                color!(0xb8a898),
                header_data_clip,
            );

            // ── Resize handle ─────────────────────────────────────────
            let resize_hovered = state
                .hovered_header
                .is_some_and(|(c, r)| c == data_col && r == HeaderRegion::Resize);
            let handle_color = if resize_hovered {
                color!(0x6a5238)
            } else {
                color!(0x4a3728)
            };
            if let Some(r) = header_data_clip.intersection(&Rectangle {
                x: resize_l,
                y: header.y,
                width: RESIZE_HANDLE_WIDTH,
                height: header.height,
            }) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: r,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(handle_color),
                );
            }
        }

        // ── Id column header ──────────────────────────────────────────
        let id_header = Rectangle {
            x: bounds.x,
            y: header.y,
            width: id_w,
            height: header.height,
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: id_header,
                border: Border {
                    color: color!(0x3d2b1f),
                    width: 1.0,
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x171411)),
        );
        let key = ParagraphKey::new("#", self.text_size, id_w, self.font);
        let para = self.cache.get_or_insert(key, || {
            Paragraph::with_text(text::Text {
                content: "#",
                bounds: Size::new(id_w, header.height),
                size: Pixels(self.text_size),
                line_height: text::LineHeight::default(),
                font: self.font,
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: None,
            })
        });
        let id_inner = Rectangle {
            x: bounds.x + self.cell_padding_x,
            y: header.y,
            width: (id_w - self.cell_padding_x * 2.0).max(0.0),
            height: header.height,
        };
        let pos = id_inner.anchor(
            para.min_bounds(),
            alignment::Horizontal::Left,
            alignment::Vertical::Center,
        );
        <iced::Renderer as text::Renderer>::fill_paragraph(
            renderer,
            &para,
            pos,
            color!(0x6a5e54),
            id_header.intersection(viewport).unwrap_or(id_header),
        );
    }
}

// ── Glyph drawing helper ─────────────────────────────────────────────

/// Draw a single glyph centered inside `bounds` using `cache` to avoid
/// re-shaping. Used for the small filter icons (`◼`, `▾`) in column headers.
#[allow(clippy::too_many_arguments)]
fn draw_centered_glyph(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    glyph: &str,
    size: f32,
    font: Font,
    bounds: Rectangle,
    color: Color,
    clip: Rectangle,
) {
    let key = ParagraphKey::new(glyph, size, bounds.width, font);
    let para = cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: glyph,
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(size),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Center,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
            ellipsis: text::Ellipsis::None,
            hint_factor: None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Center,
        alignment::Vertical::Center,
    );
    let cell_clip = clip.intersection(&bounds).unwrap_or(bounds);
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}
