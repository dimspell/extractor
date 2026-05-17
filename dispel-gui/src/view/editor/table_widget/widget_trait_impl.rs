use super::types::{Axis, HeaderRegion, ScrollbarDrag, State};
use super::widget::TableWidget;
use super::{
    DOUBLE_CLICK_MS, FILTER_BADGE_WIDTH, FILTER_ICON_WIDTH, RESIZE_HANDLE_WIDTH,
    SCROLLBAR_THICKNESS,
};
use crate::view::editor::paragraph_cache::{ParagraphCache, ParagraphKey};
use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key};
use iced::mouse;
use iced::{
    alignment, color, Background, Border, Color, Element, Event, Font, Length, Pixels, Rectangle,
    Shadow, Size, Vector,
};

type Paragraph = GraphicsParagraph;

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

        let body_h = self.body_bounds(bounds).height;
        if state.last_body_height != Some(body_h) {
            state.last_body_height = Some(body_h);
            if let Some(cb) = &self.on_scroll {
                shell.publish(cb(state.scroll_offset.x, state.scroll_offset.y, body_h));
            }
        }

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let (dx, dy) = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        (-x * self.row_height * 3.0, -y * self.row_height * 3.0)
                    }
                    mouse::ScrollDelta::Pixels { x, y } => (-x, -y),
                };
                if state.shift_pressed {
                    let new_x = state.scroll_offset.x + dy;
                    if self.apply_scroll(state, bounds, new_x, state.scroll_offset.y, shell) {
                        shell.capture_event();
                    }
                } else {
                    let new_x = state.scroll_offset.x + dx;
                    let new_y = state.scroll_offset.y + dy;
                    if self.apply_scroll(state, bounds, new_x, new_y, shell) {
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(drag) = state.dragging {
                    let Some(cur) = cursor.position() else { return };
                    self.continue_drag(state, bounds, drag, cur, shell);
                    shell.capture_event();
                    return;
                }
                let new_sb_hover = cursor
                    .position_over(bounds)
                    .and_then(|p| self.scrollbar_under(bounds, state.scroll_offset, p));
                if new_sb_hover != state.hovered_scrollbar {
                    state.hovered_scrollbar = new_sb_hover;
                    shell.request_redraw();
                }
                let new_hh = cursor
                    .position_over(bounds)
                    .and_then(|p| self.header_hit(bounds, state.scroll_offset.x, p));
                if new_hh != state.hovered_header {
                    state.hovered_header = new_hh;
                    shell.request_redraw();
                }
                let body = self.body_bounds(bounds);
                let new_hover = cursor.position_over(bounds).and_then(|p| {
                    if self.over_scrollbar(bounds, state.scroll_offset, p) {
                        return None;
                    }
                    if !body.contains(p) {
                        return None;
                    }
                    let local_y = (p.y - body.y) + state.scroll_offset.y;
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
                if let Some((col, region)) = self.header_hit(bounds, state.scroll_offset.x, p) {
                    match region {
                        HeaderRegion::Label => {
                            if let Some(cb) = &self.on_sort {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::FilterOpen => {
                            if let Some(cb) = &self.on_open_filter {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::FilterBadge => {
                            if let Some(cb) = &self.on_clear_filter {
                                shell.publish(cb(col));
                            }
                        }
                        HeaderRegion::Resize => {
                            let now = std::time::Instant::now();
                            let is_double = state.last_resize_click.is_some_and(|(c, t)| {
                                c == col && now.duration_since(t).as_millis() < DOUBLE_CLICK_MS
                            });
                            if is_double {
                                if let Some(cb) = &self.on_reset_column_width {
                                    shell.publish(cb(col));
                                }
                                state.last_resize_click = None;
                            } else {
                                if let Some(cb) = &self.on_start_resize {
                                    shell.publish(cb(col));
                                }
                                state.last_resize_click = Some((col, now));
                            }
                        }
                    }
                    shell.capture_event();
                    return;
                }
                if let Some((track, thumb)) = self.vertical_scrollbar(bounds, state.scroll_offset.y)
                {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Vertical,
                                start_cursor: p,
                                start_offset: state.scroll_offset,
                            });
                        } else {
                            let body = self.body_bounds(bounds);
                            let total_h = self.total_height();
                            let max_off = (total_h - body.height).max(1.0);
                            let travel = (body.height - thumb.height).max(1.0);
                            let target_thumb_y =
                                (p.y - thumb.height / 2.0).clamp(body.y, body.y + travel);
                            let frac = (target_thumb_y - body.y) / travel;
                            let new_y = frac * max_off;
                            self.apply_scroll(state, bounds, state.scroll_offset.x, new_y, shell);
                        }
                        shell.capture_event();
                        return;
                    }
                }
                if let Some((track, thumb)) =
                    self.horizontal_scrollbar(bounds, state.scroll_offset.x)
                {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Horizontal,
                                start_cursor: p,
                                start_offset: state.scroll_offset,
                            });
                        } else {
                            let body = self.body_bounds(bounds);
                            let total_w = self.total_width();
                            let max_off = (total_w - body.width).max(1.0);
                            let travel = (body.width - thumb.width).max(1.0);
                            let target_thumb_x =
                                (p.x - thumb.width / 2.0).clamp(body.x, body.x + travel);
                            let frac = (target_thumb_x - body.x) / travel;
                            let new_x = frac * max_off;
                            self.apply_scroll(state, bounds, new_x, state.scroll_offset.y, shell);
                        }
                        shell.capture_event();
                        return;
                    }
                }
                let body = self.body_bounds(bounds);
                if !body.contains(p) {
                    return;
                }
                let local_y = (p.y - body.y) + state.scroll_offset.y;
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
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging.is_some() =>
            {
                state.dragging = None;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift_pressed = modifiers.shift();
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                let body = self.body_bounds(bounds);
                if !body.contains(p) {
                    return;
                }
                let local_y = (p.y - body.y) + state.scroll_offset.y;
                if local_y < 0.0 {
                    return;
                }
                let row = (local_y / self.row_height) as usize;
                if row >= self.n_rows() {
                    return;
                }
                let local_x = (p.x - bounds.x) + state.scroll_offset.x;
                if local_x < 0.0 {
                    return;
                }
                let mut acc = 0.0_f32;
                for (col_idx, col_w) in (1..self.n_cols()).map(|i| (i, self.col_width(i))) {
                    if local_x < acc + col_w {
                        let col = col_idx - 1;
                        if let Some(value) = self.cell_value(row, col_idx) {
                            if let Some(cb) = &self.on_quick_filter {
                                shell.publish(cb(col, value));
                                shell.capture_event();
                            }
                        }
                        return;
                    }
                    acc += col_w;
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                if modifiers.control() {
                    if let keyboard::Key::Character(c) = key {
                        if c == "g" {
                            if modifiers.shift() {
                                if let Some(cb) = &self.on_prev_highlight {
                                    shell.publish(cb());
                                    shell.capture_event();
                                }
                            } else {
                                if let Some(cb) = &self.on_next_highlight {
                                    shell.publish(cb());
                                    shell.capture_event();
                                }
                            }
                            return;
                        }
                    }
                }
                let body = self.body_bounds(bounds);
                let page_rows = (body.height / self.row_height).floor() as i32;
                let new_y = match key {
                    keyboard::Key::Named(key::Named::PageUp) => {
                        state.scroll_offset.y - (page_rows as f32 * self.row_height)
                    }
                    keyboard::Key::Named(key::Named::PageDown) => {
                        state.scroll_offset.y + (page_rows as f32 * self.row_height)
                    }
                    keyboard::Key::Named(key::Named::Home) => 0.0,
                    keyboard::Key::Named(key::Named::End) => {
                        (self.total_height() - body.height).max(0.0)
                    }
                    keyboard::Key::Named(key::Named::ArrowRight) => {
                        if let Some(cb) = &self.on_next_highlight {
                            shell.publish(cb());
                            shell.capture_event();
                        }
                        return;
                    }
                    keyboard::Key::Named(key::Named::ArrowLeft) => {
                        if let Some(cb) = &self.on_prev_highlight {
                            shell.publish(cb());
                            shell.capture_event();
                        }
                        return;
                    }
                    keyboard::Key::Named(key::Named::Escape) => {
                        if let Some(cb) = &self.on_escape {
                            shell.publish(cb());
                            shell.capture_event();
                        }
                        return;
                    }
                    _ => return,
                };
                if self.apply_scroll(state, bounds, state.scroll_offset.x, new_y, shell) {
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        if let Some(p) = cursor.position_over(bounds) {
            if let Some((_col, region)) = self.header_hit(bounds, state.scroll_offset.x, p) {
                if region == HeaderRegion::Resize {
                    return mouse::Interaction::ResizingHorizontally;
                }
            }
        }

        if cursor.is_over(bounds) {
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
        let body = self.body_bounds(bounds);
        let off = state.scroll_offset;

        if self.n_rows() == 0 || self.n_cols() == 0 {
            return;
        }

        let clip = body.intersection(viewport).unwrap_or(body);
        let total_w = self.total_width();
        let content_visible_w = (total_w - off.x).clamp(0.0, body.width);

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

        let first_row = ((off.y / self.row_height).floor() as usize).min(self.n_rows());
        let last_row =
            (((off.y + body.height) / self.row_height).ceil() as usize).min(self.n_rows());

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
            .partition_point(|&x| x < off.x + body.width)
            .min(n_cols);

        let data_clip = clip
            .intersection(&self.data_area(body))
            .unwrap_or(self.data_area(body));

        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let flags = (self.row_flags)(row_idx);
            let is_hovered = state.hovered_row == Some(row_idx);

            let row_w = content_visible_w.min(clip.width);
            let row_y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let bg_y = row_y.max(body.y);
            let bg_height = (row_y + self.row_height).min(body.y + body.height) - bg_y;
            if bg_height > 0.0 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: clip.x,
                            y: bg_y,
                            width: row_w,
                            height: bg_height,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(crate::view::editor::table_widget::style::row_bg(
                        row_idx, flags, is_hovered,
                    )),
                );
            }

            for (col_idx, &cell_x_offset) in col_x
                .iter()
                .enumerate()
                .take(last_col)
                .skip(first_col.max(1))
            {
                let cell_x = bounds.x + cell_x_offset - off.x;
                let cell_w = self.col_width(col_idx);

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
                        align_y: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
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
                    crate::view::editor::table_widget::style::cell_text_color(flags),
                    cell_clip,
                );
            }

            if let Some((border_color, border_width)) =
                crate::view::editor::table_widget::style::row_border(flags)
            {
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

        let id_x = bounds.x;
        let id_w = self.id_col_width.min(bounds.width);
        for row_idx in first_row..last_row {
            let y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let id_y = body.y + (row_idx as f32 * self.row_height) - off.y;
            let id_bg_y = id_y.max(body.y);
            let id_bg_h = (id_y + self.row_height).min(body.y + body.height) - id_bg_y;
            let flags = (self.row_flags)(row_idx);
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
                Background::Color(crate::view::editor::table_widget::style::id_cell_bg(flags)),
            );

            let value = match self.cell_value(row_idx, 0) {
                Some(v) if !v.is_empty() => v,
                _ => continue,
            };
            let key = ParagraphKey::new(&value, self.text_size, id_w, self.font);
            let paragraph = self.cache.get_or_insert(key, || {
                Paragraph::with_text(text::Text {
                    content: value.as_str(),
                    bounds: Size::new(id_w, self.row_height),
                    size: Pixels(self.text_size),
                    line_height: text::LineHeight::default(),
                    font: self.font,
                    align_x: text::Alignment::Default,
                    align_y: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::None,
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
                crate::view::editor::table_widget::style::id_text_color(flags),
                id_clip,
            );

            if let Some((border_color, border_width)) =
                crate::view::editor::table_widget::style::row_border(flags)
            {
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

        let header = self.header_bounds(bounds);
        let header_clip = header.intersection(viewport).unwrap_or(header);
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

        for (col_idx, &col_x_offset) in col_x
            .iter()
            .enumerate()
            .take(last_col)
            .skip(first_col.max(1))
        {
            let col_l_screen = bounds.x + col_x_offset - off.x;
            let col_w = self.col_width(col_idx);
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
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
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
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::None,
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

        let active_axis = state.dragging.map(|d| d.axis).or(state.hovered_scrollbar);
        draw_scrollbars(
            renderer,
            bounds,
            body,
            off,
            self.total_width(),
            self.total_height(),
            active_axis,
        );
    }
}

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
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
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

/// Paint vertical and horizontal scrollbar thumbs along the right and bottom
/// edges of `bounds` to reflect `off` against the total content size.
///
/// When `active_axis` matches an axis, that scrollbar's thumb is drawn 1.5×
/// thicker and a few shades lighter so the user sees it's grabbable.
fn draw_scrollbars(
    renderer: &mut iced::Renderer,
    bounds: Rectangle,
    body: Rectangle,
    off: Vector,
    total_w: f32,
    total_h: f32,
    active_axis: Option<Axis>,
) {
    let track_color = color!(0x141210);
    let thumb_idle = color!(0x5d4037);
    let thumb_active = color!(0xB97024);
    let border_idle = color!(0x5d4037);
    let border_active = color!(0xB97024);

    if total_h > body.height {
        let track = Rectangle {
            x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
            y: body.y,
            width: SCROLLBAR_THICKNESS,
            height: body.height,
        };
        let thumb_h = (body.height / total_h * body.height).max(20.0);
        let max_off = (total_h - body.height).max(1.0);
        let thumb_y = body.y + (off.y / max_off) * (body.height - thumb_h);

        let active = active_axis == Some(Axis::Vertical);
        let extra = if active {
            SCROLLBAR_THICKNESS * 0.5
        } else {
            0.0
        };
        let thumb_w = SCROLLBAR_THICKNESS - 2.0 + extra;
        let thumb_x = track.x + 1.0 - extra;

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
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
        );
    }

    if total_w > body.width {
        let track = Rectangle {
            x: bounds.x,
            y: bounds.y + bounds.height - SCROLLBAR_THICKNESS,
            width: body.width,
            height: SCROLLBAR_THICKNESS,
        };
        let thumb_w = (body.width / total_w * body.width).max(20.0);
        let max_off = (total_w - body.width).max(1.0);
        let thumb_x = bounds.x + (off.x / max_off) * (body.width - thumb_w);

        let active = active_axis == Some(Axis::Horizontal);
        let extra = if active {
            SCROLLBAR_THICKNESS * 0.5
        } else {
            0.0
        };
        let thumb_h = SCROLLBAR_THICKNESS - 2.0 + extra;
        let thumb_y = track.y + 1.0 - extra;

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
                    y: thumb_y,
                    width: thumb_w,
                    height: thumb_h,
                },
                border: Border {
                    color: if active { border_active } else { border_idle },
                    width: if active { 1.0 } else { 0.5 },
                    radius: 0.into(),
                },
                shadow: Shadow::default(),
                snap: true,
            },
            Background::Color(if active { thumb_active } else { thumb_idle }),
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
