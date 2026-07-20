//! Scrollable data-row body rendering.
//!
//! Draws the visible record rows (data columns) including alternating
//! backgrounds, cell text, selection/highlight/hover styling, and row
//! borders.  The frozen id column *is not* drawn here — that belongs to
//! [`frozen_column`].

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::Renderer as _;
use iced::{alignment, color, Background, Border, Color, Pixels, Rectangle, Shadow, Size};

use super::geometry;
use super::types::State;
use super::widget::TableWidget;
use crate::components::paragraph_cache::ParagraphKey;

type Paragraph = GraphicsParagraph;

impl<'a, Message> TableWidget<'a, Message> {
    /// Draw the visible data rows (all columns except the frozen id column).
    ///
    /// Skips rows and cells outside the viewport.  Uses `self.cache`
    /// (a [`ParagraphCache`]) so shaped paragraphs survive viewport changes.
    pub(crate) fn draw_rows(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        body: Rectangle,
        viewport: &Rectangle,
        state: &State,
    ) {
        let off = self.scroll_offset();
        let n_cols = geometry::n_cols(&self.columns);
        let n_rows = self.n_rows();
        if n_rows == 0 || n_cols == 0 {
            return;
        }

        let clip = body.intersection(viewport).unwrap_or(body);
        let total_w = geometry::total_width(self.id_col_width, &self.columns);
        let content_visible_w = (total_w - off.x).clamp(0.0, body.width);

        // Body background fill
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: clip.x,
                    y: clip.y,
                    width: content_visible_w.min(clip.width),
                    height: clip.height,
                },
                border: Border::default(),
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(color!(0x1e1b17)),
        );

        // Visible row range
        let first_row = ((off.y / self.row_height).floor() as usize).min(n_rows);
        let last_row = (((off.y + body.height) / self.row_height).ceil() as usize).min(n_rows);

        let col_x = geometry::col_positions(self.id_col_width, &self.columns);
        let first_col = col_x
            .partition_point(|&x| x <= off.x)
            .saturating_sub(1)
            .min(n_cols.saturating_sub(1));
        let last_col = col_x
            .partition_point(|&x| x < off.x + body.width)
            .min(n_cols);

        let data_clip = clip
            .intersection(&geometry::data_area(body, self.id_col_width))
            .unwrap_or(geometry::data_area(body, self.id_col_width));

        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let flags = (self.row_flags)(row_idx);
            let is_hovered = state.hovered_row == Some(row_idx);

            // Row background
            let row_y = y;
            let bg_y = row_y.max(body.y);
            let bg_height = (row_y + self.row_height).min(body.y + body.height) - bg_y;
            if bg_height > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: clip.x,
                            y: bg_y,
                            width: content_visible_w.min(clip.width),
                            height: bg_height,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(super::style::row_bg(row_idx, flags, is_hovered)),
                );
            }

            // Data cells
            for (col_idx, &cell_x_offset) in col_x
                .iter()
                .enumerate()
                .take(last_col)
                .skip(first_col.max(1))
            {
                let cell_x = bounds.x + cell_x_offset - off.x;
                let cell_w = geometry::col_width(self.id_col_width, &self.columns, col_idx);

                let value = match self.cell_value(row_idx, col_idx) {
                    Some(v) if !v.is_empty() => v,
                    _ => continue,
                };

                let key = ParagraphKey::new(&value, self.text_size, cell_w, self.font);
                let paragraph = self.cache.get_or_insert(key, || {
                    Paragraph::with_text(text::Text {
                        content: &*value,
                        bounds: Size::new(cell_w, self.row_height),
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

                let cell_inner = Rectangle {
                    x: cell_x + self.cell_padding_x,
                    y,
                    width: (cell_w - self.cell_padding_x * 2.0).max(0.0),
                    height: self.row_height,
                };
                let position = cell_inner.anchor(
                    paragraph.min_bounds(),
                    alignment::Horizontal::Left,
                    alignment::Vertical::Center,
                );
                let cell_clip = data_clip
                    .intersection(&Rectangle {
                        x: cell_x,
                        y,
                        width: cell_w,
                        height: self.row_height,
                    })
                    .unwrap_or(Rectangle {
                        x: cell_x,
                        y,
                        width: 0.0,
                        height: 0.0,
                    });
                <iced::Renderer as text::Renderer>::fill_paragraph(
                    renderer,
                    &paragraph,
                    position,
                    super::style::cell_text_color(flags),
                    cell_clip,
                );
            }

            // Row border (selection / highlight indicator)
            if let Some((border_color, border_width)) = super::style::row_border(flags) {
                let border_y = y.max(body.y);
                let border_h = (y + self.row_height).min(body.y + body.height) - border_y;
                if border_h > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: clip.x,
                                y: border_y,
                                width: content_visible_w.min(clip.width),
                                height: border_h,
                            },
                            border: Border {
                                color: border_color,
                                width: border_width,
                                radius: 0.into(),
                            },
                            shadow: Shadow::default(),
                            snap: true,
                        },
                        Background::Color(Color::TRANSPARENT),
                    );
                }
            }
        }
    }
}
