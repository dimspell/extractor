use super::geometry;
use super::scrollbar;
use super::types::{Axis, HeaderRegion, ScrollbarDrag, State};
use super::widget::TableWidget;
use super::DOUBLE_CLICK_MS;

#[cfg(feature = "accessibility")]
use iced::advanced::graphics::core::accessibility::accesskit;
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::Shell;
use iced::keyboard::{self, key};
use iced::mouse;
use iced::{Element, Event, Length, Rectangle, Size};
use std::borrow::Cow;

// =========================================================================
// Widget trait implementation
// =========================================================================

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

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, limits: &Limits) -> Node {
        let max = limits.max();
        // No sync_external needed — we read self.table_state directly.
        Node::new(Size::new(max.width, max.height))
    }

    // ── Event handling ────────────────────────────────────────────────

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();
        let body = self.body_bounds(bounds);

        // Bump viewport-height in app state when the body resizes.
        let body_h = body.height;
        if state.last_body_height != Some(body_h) {
            state.last_body_height = Some(body_h);
            if let Some(cb) = &self.on_scroll {
                let off = self.scroll_offset();
                shell.publish(cb(off.x, off.y, body_h));
            }
        }

        match event {
            // ── Wheel scroll ──────────────────────────────────────────
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
                let cur_off = self.scroll_offset();
                if state.shift_pressed {
                    let horiz = if dx != 0.0 { dx } else { dy };
                    let new_x = cur_off.x + horiz;
                    if self.apply_scroll(state, bounds, new_x, cur_off.y, shell) {
                        shell.capture_event();
                    }
                } else {
                    let new_x = cur_off.x + dx;
                    let new_y = cur_off.y + dy;
                    if self.apply_scroll(state, bounds, new_x, new_y, shell) {
                        shell.capture_event();
                    }
                }
            }

            // ── Cursor move (hover tracking) ──────────────────────────
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(drag) = state.dragging {
                    let Some(cur) = cursor.position() else { return };
                    self.continue_drag(state, bounds, drag, cur, shell);
                    shell.capture_event();
                    return;
                }
                let cur_off = self.scroll_offset();
                let new_sb_hover = cursor
                    .position_over(bounds)
                    .and_then(|p| self.scrollbar_under(bounds, cur_off, p));
                if new_sb_hover != state.hovered_scrollbar {
                    state.hovered_scrollbar = new_sb_hover;
                    shell.request_redraw();
                }
                let new_hh = cursor
                    .position_over(bounds)
                    .and_then(|p| self.header_hit(bounds, cur_off.x, p));
                if new_hh != state.hovered_header {
                    state.hovered_header = new_hh;
                    shell.request_redraw();
                }
                let new_hover = cursor.position_over(bounds).and_then(|p| {
                    if self.over_scrollbar(bounds, cur_off, p) {
                        return None;
                    }
                    if !body.contains(p) {
                        return None;
                    }
                    let local_y = (p.y - body.y) + cur_off.y;
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

            // ── Left button press ─────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                let cur_off = self.scroll_offset();

                // Header hit (sort, filter, resize)
                if let Some((col, region)) = self.header_hit(bounds, cur_off.x, p) {
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

                // Vertical scrollbar
                if let Some((track, thumb)) = self.vertical_scrollbar(bounds, cur_off.y) {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Vertical,
                                start_cursor: p,
                                start_offset: cur_off,
                            });
                        } else {
                            let total_h = self.total_height();
                            let max_off = (total_h - body.height).max(1.0);
                            let travel = (body.height - thumb.height).max(1.0);
                            let target_thumb_y =
                                (p.y - thumb.height / 2.0).clamp(body.y, body.y + travel);
                            let frac = (target_thumb_y - body.y) / travel;
                            let new_y = frac * max_off;
                            self.apply_scroll(state, bounds, cur_off.x, new_y, shell);
                        }
                        shell.capture_event();
                        return;
                    }
                }

                // Horizontal scrollbar
                if let Some((track, thumb)) = self.horizontal_scrollbar(bounds, cur_off.x) {
                    if track.contains(p) {
                        if thumb.contains(p) {
                            state.dragging = Some(ScrollbarDrag {
                                axis: Axis::Horizontal,
                                start_cursor: p,
                                start_offset: cur_off,
                            });
                        } else {
                            let total_w = self.total_width();
                            let max_off = (total_w - body.width).max(1.0);
                            let travel = (body.width - thumb.width).max(1.0);
                            let target_thumb_x =
                                (p.x - thumb.width / 2.0).clamp(body.x, body.x + travel);
                            let frac = (target_thumb_x - body.x) / travel;
                            let new_x = frac * max_off;
                            self.apply_scroll(state, bounds, new_x, cur_off.y, shell);
                        }
                        shell.capture_event();
                        return;
                    }
                }

                // Body click → select row
                if !body.contains(p) {
                    return;
                }
                let local_y = (p.y - body.y) + cur_off.y;
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

            // ── Left button release (end drag) ────────────────────────
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging.is_some() =>
            {
                state.dragging = None;
                shell.capture_event();
                shell.request_redraw();
            }

            // ── Modifier keys ─────────────────────────────────────────
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                state.shift_pressed = modifiers.shift();
            }

            // ── Right button press (quick filter) ─────────────────────
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let Some(p) = cursor.position_over(bounds) else {
                    return;
                };
                let body = self.body_bounds(bounds);
                if !body.contains(p) {
                    return;
                }
                let cur_off = self.scroll_offset();
                let local_y = (p.y - body.y) + cur_off.y;
                if local_y < 0.0 {
                    return;
                }
                let row = (local_y / self.row_height) as usize;
                if row >= self.n_rows() {
                    return;
                }
                let local_x = (p.x - bounds.x) + cur_off.x;
                if local_x < 0.0 {
                    return;
                }
                let mut acc = 0.0_f32;
                for (col_idx, col_w) in (1..self.columns.len() + 1)
                    .map(|i| (i, geometry::col_width(self.id_col_width, &self.columns, i)))
                {
                    if local_x < acc + col_w {
                        let col = col_idx - 1;
                        if let Some(value) = self.cell_value(row, col_idx) {
                            if let Some(cb) = &self.on_quick_filter {
                                shell.publish(cb(col, value.into_owned()));
                                shell.capture_event();
                            }
                        }
                        return;
                    }
                    acc += col_w;
                }
            }

            // ── Keyboard navigation ───────────────────────────────────
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
                let cur_y = self.scroll_offset().y;
                let new_y = match key {
                    keyboard::Key::Named(key::Named::PageUp) => {
                        cur_y - (page_rows as f32 * self.row_height)
                    }
                    keyboard::Key::Named(key::Named::PageDown) => {
                        cur_y + (page_rows as f32 * self.row_height)
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
                if self.apply_scroll(state, bounds, self.scroll_offset().x, new_y, shell) {
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    // ── Mouse interaction ─────────────────────────────────────────────

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if let Some(p) = cursor.position_over(bounds) {
            if let Some((_col, region)) = self.header_hit(bounds, self.scroll_offset().x, p) {
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

    // ── Rendering ─────────────────────────────────────────────────────

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
        let off = self.scroll_offset();
        let body = self.body_bounds(bounds);

        if self.n_rows() == 0 || geometry::n_cols(&self.columns) == 0 {
            return;
        }

        // Visible row range
        let first_row = ((off.y / self.row_height).floor() as usize).min(self.n_rows());
        let last_row =
            (((off.y + body.height) / self.row_height).ceil() as usize).min(self.n_rows());

        let clip = body.intersection(viewport).unwrap_or(body);

        // Draw in z-order: data rows → frozen id column → header → scrollbars
        self.draw_rows(renderer, bounds, body, viewport, state);
        self.draw_frozen_column(renderer, bounds, body, viewport, first_row, last_row, clip);
        self.draw_header(renderer, bounds, viewport, state);

        let active_axis = state.dragging.map(|d| d.axis).or(state.hovered_scrollbar);
        let total_w = self.total_width();
        let total_h = self.total_height();
        scrollbar::draw_scrollbars(renderer, bounds, body, off, total_w, total_h, active_axis);
    }

    // ── Accessibility ─────────────────────────────────────────────────

    #[allow(clippy::needless_range_loop)]
    #[cfg(feature = "accessibility")]
    fn accessibility(
        &self,
        layout: Layout<'_>,
        tree: &Tree,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        id_counter: &mut u64,
    ) -> Option<accesskit::NodeId> {
        use accesskit::Role;

        let non_empty = |r: Rectangle| Rectangle {
            width: r.width.max(1.0),
            height: r.height.max(1.0),
            ..r
        };
        let to_ak_rect = |r: Rectangle| accesskit::Rect {
            x0: r.x as f64,
            y0: r.y as f64,
            x1: (r.x + r.width) as f64,
            y1: (r.y + r.height) as f64,
        };
        let union_rect = |a: Rectangle, b: Rectangle| Rectangle {
            x: a.x.min(b.x),
            y: a.y.min(b.y),
            width: (a.x + a.width).max(b.x + b.width) - a.x.min(b.x),
            height: (a.y + a.height).max(b.y + b.height) - a.y.min(b.y),
        };
        let set_bounds = |tree: &Tree, node: &mut accesskit::Node, bounds: Rectangle| {
            let b = non_empty(bounds);
            tree.set_accesskit_bounds(b);
            node.set_bounds(to_ak_rect(b));
        };

        let n_rows = self.n_rows();
        let n_cols = geometry::n_cols(&self.columns);
        if n_rows == 0 || n_cols == 0 {
            return None;
        }

        self.cell_node_map.borrow_mut().clear();

        let bounds = layout.bounds();
        let body = self.body_bounds(bounds);
        let off = self.scroll_offset();
        let col_pos = geometry::col_positions(self.id_col_width, &self.columns);

        let first_row = 0;
        let last_row = n_rows;

        // ---- Column-header row (ColumnHeader cells) ----
        let mut header_cell_ids: Vec<accesskit::NodeId> = Vec::with_capacity(n_cols);
        let mut header_row_bounds: Option<Rectangle> = None;
        let header_y = body.y - self.row_height;

        for col in 0..n_cols {
            let cell_id = accesskit::NodeId(*id_counter);
            *id_counter += 1;

            let cell_bounds = Rectangle {
                x: bounds.x + col_pos[col] - off.x,
                y: header_y,
                width: geometry::col_width(self.id_col_width, &self.columns, col),
                height: self.row_height,
            };

            let mut cell = accesskit::Node::new(Role::ColumnHeader);
            cell.add_action(accesskit::Action::Focus);
            cell.add_action(accesskit::Action::ScrollIntoView);
            cell.set_bounds(to_ak_rect(non_empty(cell_bounds)));
            cell.set_row_index(0);
            cell.set_column_index(col);
            cell.set_row_span(1_usize);
            cell.set_column_span(1_usize);

            if col == 0 {
                cell.set_label("#");
            } else if let Some(c) = self.columns.get(col - 1) {
                cell.set_label(c.label.as_str());
                if let Some(asc) = c.sort {
                    cell.set_description(if asc {
                        "sorted ascending"
                    } else {
                        "sorted descending"
                    });
                }
            }

            header_row_bounds =
                Some(header_row_bounds.map_or(cell_bounds, |r| union_rect(r, cell_bounds)));

            nodes.push((cell_id, cell));
            header_cell_ids.push(cell_id);
        }

        let header_row_id = accesskit::NodeId(*id_counter);
        *id_counter += 1;
        let mut header_row = accesskit::Node::new(Role::Row);
        header_row.set_row_index(0);
        header_row.set_column_index(0);
        header_row.set_column_span(n_cols);
        if let Some(bounds) = header_row_bounds {
            header_row.set_bounds(to_ak_rect(non_empty(bounds)));
        }
        for id in &header_cell_ids {
            header_row.push_child(*id);
        }
        nodes.push((header_row_id, header_row));

        // ---- Data rows (Cell cells) ----
        let mut row_ids: Vec<accesskit::NodeId> = Vec::new();

        for row_idx in first_row..last_row {
            let mut cell_ids: Vec<accesskit::NodeId> = Vec::with_capacity(n_cols);
            let mut row_bounds: Option<Rectangle> = None;
            let row_y = body.y + (row_idx as f32 * self.row_height) - off.y;

            for col in 0..n_cols {
                let cell_id = accesskit::NodeId(*id_counter);
                *id_counter += 1;

                let cell_bounds = Rectangle {
                    x: bounds.x + col_pos[col] - off.x,
                    y: row_y,
                    width: geometry::col_width(self.id_col_width, &self.columns, col),
                    height: self.row_height,
                };

                let mut cell = accesskit::Node::new(Role::Cell);
                cell.add_action(accesskit::Action::Focus);
                cell.add_action(accesskit::Action::ScrollIntoView);
                cell.set_bounds(to_ak_rect(non_empty(cell_bounds)));
                cell.set_row_index(row_idx + 1);
                cell.set_column_index(col);
                cell.set_row_span(1);
                cell.set_column_span(1);

                if let Some(val) = self.cell_value(row_idx, col) {
                    cell.set_value(&*val);
                    let label = if col == 0 {
                        Cow::Owned(format!("#: {}", val))
                    } else if let Some(c) = self.columns.get(col - 1) {
                        Cow::Owned(format!("{}: {}", c.label, val))
                    } else {
                        Cow::Borrowed(&*val)
                    };
                    cell.set_label(label.as_ref());
                }

                self.cell_node_map
                    .borrow_mut()
                    .insert(cell_id.0, (row_idx, col));

                cell.push_labelled_by(header_cell_ids[col]);

                row_bounds = Some(row_bounds.map_or(cell_bounds, |r| union_rect(r, cell_bounds)));

                nodes.push((cell_id, cell));
                cell_ids.push(cell_id);
            }

            let row_id = accesskit::NodeId(*id_counter);
            *id_counter += 1;
            let mut row = accesskit::Node::new(Role::Row);
            row.add_action(accesskit::Action::Focus);
            row.add_action(accesskit::Action::ScrollIntoView);
            row.set_row_index(row_idx + 1);
            row.set_column_index(0);
            row.set_column_span(n_cols);
            if let Some(bounds) = row_bounds {
                row.set_bounds(to_ak_rect(non_empty(bounds)));
            }
            let flags = (self.row_flags)(row_idx);
            if flags.selected {
                row.set_selected(true);
            }
            self.cell_node_map
                .borrow_mut()
                .insert(row_id.0, (row_idx, 0));
            for id in &cell_ids {
                row.push_child(*id);
            }
            nodes.push((row_id, row));
            row_ids.push(row_id);
        }

        // ---- Grid node ----
        let grid_id = accesskit::NodeId(*id_counter);
        *id_counter += 1;
        let mut grid = accesskit::Node::new(Role::Table);
        grid.set_scroll_y(off.y as f64);
        grid.set_scroll_x(off.x as f64);
        set_bounds(tree, &mut grid, bounds);
        grid.set_row_count(n_rows + 1);
        grid.set_column_count(n_cols);
        grid.set_multiselectable();
        if let Some(label) = &self.accessible_label {
            grid.set_label(label.as_str());
        }
        grid.push_child(header_row_id);
        for id in &row_ids {
            grid.push_child(*id);
        }
        nodes.push((grid_id, grid));

        let all_custom_ids: Vec<accesskit::NodeId> = nodes
            .iter()
            .map(|(id, _node)| *id)
            .filter(|id| *id != grid_id)
            .collect();
        tree.register_custom_accesskit_ids(&all_custom_ids);

        Some(grid_id)
    }

    #[cfg(feature = "accessibility")]
    fn accessibility_action(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        action: &accesskit::ActionRequest,
        shell: &mut Shell<'_, Message>,
    ) {
        eprintln!(
            "[VO DEBUG] accessibility_action called action={:?} target={}",
            action.action, action.target_node.0,
        );

        let row = match action.action {
            accesskit::Action::Focus | accesskit::Action::ScrollIntoView => {
                let found = self
                    .cell_node_map
                    .borrow()
                    .get(&action.target_node.0)
                    .copied();
                eprintln!(
                    "[VO DEBUG] accessibility_action lookup target={} -> {:?}",
                    action.target_node.0, found,
                );
                found.map(|(row, _col)| row)
            }
            _ => None,
        };

        if let Some(row) = row {
            // Scroll to the focused row.
            let bounds = layout.bounds();
            let body = self.body_bounds(bounds);
            let target_y = row as f32 * self.row_height;
            let clamped_y = target_y.clamp(0.0, (self.total_height() - body.height).max(0.0));
            let cur_x = self.scroll_offset().x;
            if (clamped_y - self.scroll_offset().y).abs() > f32::EPSILON {
                // Publish the new scroll offset — app state updates next frame.
                if let Some(cb) = &self.on_scroll {
                    shell.request_redraw();
                    shell.publish(cb(cur_x, clamped_y, body.height));
                }
            }

            // Select the row.
            if let Some(cb) = &self.on_select {
                shell.publish(cb(row));
            }
        }
    }
}

// ── Into<Element> ─────────────────────────────────────────────────────

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
