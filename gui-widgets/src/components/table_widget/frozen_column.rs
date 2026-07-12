//! Frozen id-column rendering.
//!
//! The id column (index 0, rendered as `orig_idx + 1`) stays pinned to the
//! left edge while the user scrolls the rest of the table horizontally.

use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::Renderer as _;
use iced::{alignment, color, Background, Border, Color, Pixels, Rectangle, Shadow, Size};

use super::widget::TableWidget;
use crate::components::paragraph_cache::ParagraphKey;

type Paragraph = GraphicsParagraph;

impl<'a, Message> TableWidget<'a, Message> {
    /// Draw the frozen id column cells and their borders for the visible
    /// row range `[first_row, last_row)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_frozen_column(
        &self,
        renderer: &mut iced::Renderer,
        bounds: Rectangle,
        body: Rectangle,
        viewport: &Rectangle,
        first_row: usize,
        last_row: usize,
        clip: Rectangle,
    ) {
        let _ = viewport;
        let off_y = self.scroll_offset().y;
        let id_w = self.id_col_width.min(bounds.width);
        let id_x = bounds.x;

        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off_y;
            let id_y = y;
            let id_bg_y = id_y.max(body.y);
            let id_bg_h = (id_y + self.row_height).min(body.y + body.height) - id_bg_y;
            let flags = (self.row_flags)(row_idx);

            // Cell background
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: id_x,
                        y: id_bg_y,
                        width: id_w,
                        height: id_bg_h,
                    },
                    border: Border {
                        color: color!(0x3d2b1f),
                        width: 0.5,
                        radius: 0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(super::style::id_cell_bg(flags)),
            );

            // Cell value
            let value = match self.cell_value(row_idx, 0) {
                Some(v) if !v.is_empty() => v,
                _ => continue,
            };
            let key = ParagraphKey::new(&value, self.text_size, id_w, self.font);
            let paragraph = self.cache.get_or_insert(key, || {
                Paragraph::with_text(text::Text {
                    content: &*value,
                    bounds: Size::new(id_w, self.row_height),
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
                x: id_x + self.cell_padding_x,
                y,
                width: (id_w - self.cell_padding_x * 2.0).max(0.0),
                height: self.row_height,
            };
            let position = id_inner.anchor(
                paragraph.min_bounds(),
                alignment::Horizontal::Left,
                alignment::Vertical::Center,
            );
            let id_clip = clip
                .intersection(&Rectangle {
                    x: id_x,
                    y: body.y,
                    width: id_w,
                    height: body.height,
                })
                .unwrap_or(Rectangle {
                    x: id_x,
                    y: body.y,
                    width: id_w,
                    height: body.height,
                });
            <iced::Renderer as text::Renderer>::fill_paragraph(
                renderer,
                &paragraph,
                position,
                super::style::id_text_color(flags),
                id_clip,
            );

            // Border
            if let Some((border_color, border_width)) = super::style::row_border(flags) {
                let border_y = y.max(body.y);
                let border_h = (y + self.row_height).min(body.y + body.height) - border_y;
                if border_h > 0.0 {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: id_x,
                                y: border_y,
                                width: id_w,
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
