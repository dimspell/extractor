//! Event handling — the mouse and keyboard logic extracted from
//! `Widget::update`.
//!
//! The function here is a pure (stateless) dispatch: given the current widget,
//! state snapshot, and incoming event, it advances the state and/or publishes
//! messages through `shell`.

use std::time::Instant;

use iced::advanced::layout::Layout;
use iced::advanced::Shell;
use iced::keyboard::{self, key};
use iced::mouse;
use iced::{Event, Point, Rectangle};

use crate::domain::write_mode::WriteMode;
use crate::selection::NavDir;
use crate::ui::view::minimap::{self, MINIMAP_WIDTH};

use super::draw::{
    first_hex_char, first_printable_char, hscrollbar_thumb, hscrollbar_track, hthumb_len,
    scrollbar_thumb, scrollbar_track, thumb_height,
};
use super::layout::{
    addr_at, clamp_scroll, clamp_scroll_x, HEADER_HEIGHT, ROW_HEIGHT, SCROLLBAR_THICKNESS,
};
use super::state::{State, DOUBLE_CLICK_WINDOW};
use super::HexMatrix;

/// Dispatch a single `Event` to the hex matrix widget.
///
/// Called by the `Widget::update` method — extracted to keep the Widget
/// impl focused on the trait bridge.
pub fn handle_event<'a, Message>(
    widget: &HexMatrix<'a, Message>,
    state: &mut State,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let bounds = layout.bounds();
    let total_h = widget.total_height();
    let total_len = widget.bytes.len() as u64;
    let viewport_h = widget.content_viewport_h(bounds.height, bounds.width);
    let content_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y + HEADER_HEIGHT,
        width: bounds.width,
        height: viewport_h,
    };

    match event {
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            if !cursor.is_over(bounds) {
                return;
            }
            let shift = state.shift_pressed.get();
            match delta {
                mouse::ScrollDelta::Lines { y, x, .. } => {
                    if shift {
                        let content_w = widget.total_content_width();
                        let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                        let sx = state.scroll_x.get();
                        let horiz = if *x != 0.0 { *x } else { *y };
                        let nsx =
                            clamp_scroll_x(sx - horiz * ROW_HEIGHT * 3.0, content_w, avail_w);
                        if (nsx - sx).abs() > f32::EPSILON {
                            state.scroll_x.set(nsx);
                            shell.request_redraw();
                        }
                    } else {
                        let dy = -y * ROW_HEIGHT * 3.0;
                        let so = state.scroll_offset.get();
                        let new = clamp_scroll(so + dy, total_h, viewport_h);
                        if (new - so).abs() > f32::EPSILON {
                            state.scroll_offset.set(new);
                            shell.request_redraw();
                        }
                    }
                    if !shift && *x != 0.0 {
                        let content_w = widget.total_content_width();
                        let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                        let sx = state.scroll_x.get();
                        let nsx =
                            clamp_scroll_x(sx - x * ROW_HEIGHT * 3.0, content_w, avail_w);
                        if (nsx - sx).abs() > f32::EPSILON {
                            state.scroll_x.set(nsx);
                            shell.request_redraw();
                        }
                    }
                    shell.capture_event();
                }
                mouse::ScrollDelta::Pixels { y, x } => {
                    if shift {
                        let content_w = widget.total_content_width();
                        let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                        let sx = state.scroll_x.get();
                        let horiz = if *x != 0.0 { *x } else { *y };
                        let nsx = clamp_scroll_x(sx - horiz, content_w, avail_w);
                        if (nsx - sx).abs() > f32::EPSILON {
                            state.scroll_x.set(nsx);
                            shell.request_redraw();
                        }
                    } else {
                        let so = state.scroll_offset.get();
                        let new = clamp_scroll(so - y, total_h, viewport_h);
                        if (new - so).abs() > f32::EPSILON {
                            state.scroll_offset.set(new);
                            shell.request_redraw();
                        }
                    }
                    if !shift && *x != 0.0 {
                        let content_w = widget.total_content_width();
                        let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                        let sx = state.scroll_x.get();
                        let nsx = clamp_scroll_x(sx - x, content_w, avail_w);
                        if (nsx - sx).abs() > f32::EPSILON {
                            state.scroll_x.set(nsx);
                            shell.request_redraw();
                        }
                    }
                    shell.capture_event();
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            let Some(p) = cursor.position_over(bounds) else {
                return;
            };

            // Horizontal scrollbar takes precedence.
            let htrack = hscrollbar_track(bounds);
            let content_w = widget.total_content_width();
            let avail_w = bounds.width - SCROLLBAR_THICKNESS;
            let needs_hscroll = content_w > avail_w;
            if needs_hscroll && htrack.contains(p) {
                let hthumb =
                    hscrollbar_thumb(htrack, state.scroll_x.get(), content_w, avail_w);
                if hthumb.contains(p) {
                    state.dragging_scrollbar_x = true;
                    state.drag_start_cursor_x = p.x;
                    state.drag_start_offset_x = state.scroll_x.get();
                } else {
                    let dir = if p.x < hthumb.x { -1.0 } else { 1.0 };
                    let nsx = clamp_scroll_x(
                        state.scroll_x.get() + dir * avail_w,
                        content_w,
                        avail_w,
                    );
                    if (nsx - state.scroll_x.get()).abs() > f32::EPSILON {
                        state.scroll_x.set(nsx);
                        shell.request_redraw();
                    }
                }
                shell.capture_event();
                return;
            }

            // Vertical scrollbar (sits below the column header).
            let scrollbar = scrollbar_track(content_bounds, viewport_h);
            if scrollbar.contains(p) && total_h > viewport_h {
                let thumb = scrollbar_thumb(scrollbar, state.scroll_offset.get(), total_h);
                if thumb.contains(p) {
                    state.dragging_scrollbar = true;
                    state.drag_start_cursor_y = p.y;
                    state.drag_start_offset = state.scroll_offset.get();
                } else {
                    let dir = if p.y < thumb.y { -1.0 } else { 1.0 };
                    let new = clamp_scroll(
                        state.scroll_offset.get() + dir * viewport_h,
                        total_h,
                        viewport_h,
                    );
                    state.scroll_offset.set(new);
                    shell.request_redraw();
                }
                shell.capture_event();
                return;
            }

            // Minimap hit-test (when enabled, sits between content and scrollbar).
            if widget.show_minimap && total_h > viewport_h {
                let mm_rect = minimap::minimap_rect(
                    content_bounds,
                    viewport_h,
                    MINIMAP_WIDTH,
                    SCROLLBAR_THICKNESS,
                );
                if mm_rect.contains(p) {
                    let thumb_r = minimap::minimap_thumb_rect(
                        mm_rect,
                        state.scroll_offset.get(),
                        total_h,
                        viewport_h,
                    );
                    if thumb_r.contains(p) {
                        state.dragging_minimap = true;
                        state.drag_start_minimap_y = p.y;
                        state.drag_start_minimap_scroll = state.scroll_offset.get();
                    } else {
                        let new_scroll = minimap::minimap_scroll_from_y(
                            p.y,
                            mm_rect,
                            total_h,
                            viewport_h,
                        );
                        let clamped = clamp_scroll(new_scroll, total_h, viewport_h);
                        state.scroll_offset.set(clamped);
                    }
                    shell.capture_event();
                    return;
                }
            }

            // Gutter click → toggle address format.
            if p.x >= bounds.x && p.x < bounds.x + widget.addr_col_width() {
                if let Some(cb) = &widget.on_toggle_addr_format {
                    shell.publish(cb());
                }
                shell.request_redraw();
                shell.capture_event();
                return;
            }

            // Header area (above content rows) -> ignore for byte selection.
            if p.y < bounds.y + HEADER_HEIGHT {
                return;
            }

            // Cell click → selection (and maybe edit on double-click).
            if let Some(addr) = addr_at(
                p,
                content_bounds,
                state.scroll_offset.get(),
                state.scroll_x.get(),
                widget.bytes_per_row,
                total_len,
                widget.addr_col_width(),
            ) {
                let now = Instant::now();
                let is_double = matches!(
                    (state.last_click_addr, state.last_click_at),
                    (Some(prev), Some(at))
                        if prev == addr && now.duration_since(at) <= DOUBLE_CLICK_WINDOW
                );
                state.last_click_addr = Some(addr);
                state.last_click_at = Some(now);

                if is_double {
                    if let Some(cb) = &widget.on_begin_edit {
                        shell.publish(cb(addr));
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }
                }

                state.selecting = true;
                if let Some(cb) = &widget.on_select_at {
                    shell.publish(cb(addr));
                }
                shell.request_redraw();
                shell.capture_event();
            }
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
            let Some(p) = cursor.position_over(bounds) else {
                return;
            };
            if p.y < bounds.y + HEADER_HEIGHT {
                return;
            }
            if let Some(addr) = addr_at(
                p,
                content_bounds,
                state.scroll_offset.get(),
                state.scroll_x.get(),
                widget.bytes_per_row,
                total_len,
                widget.addr_col_width(),
            ) {
                if let Some(cb) = &widget.on_right_click {
                    shell.publish(cb(addr));
                }
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { .. }) => {
            let content_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + HEADER_HEIGHT,
                width: bounds.width,
                height: viewport_h,
            };

            if cursor.is_over(bounds) {
                if let Some(p) = cursor.position() {
                    let vtrack = scrollbar_track(content_bounds, viewport_h);
                    let htrack = hscrollbar_track(bounds);
                    let now_hovering = vtrack.contains(p) || htrack.contains(p);
                    if now_hovering != state.hovering_scrollbar.get() {
                        state.hovering_scrollbar.set(now_hovering);
                        shell.request_redraw();
                    }
                    if widget.show_minimap && total_h > viewport_h {
                        let mm_rect = minimap::minimap_rect(
                            content_bounds,
                            viewport_h,
                            MINIMAP_WIDTH,
                            SCROLLBAR_THICKNESS,
                        );
                        let now_mm_hover = mm_rect.contains(p);
                        if now_mm_hover != state.hovering_minimap.get() {
                            state.hovering_minimap.set(now_mm_hover);
                            shell.request_redraw();
                        }
                    }
                }
            }
            if state.dragging_scrollbar_x {
                let Some(p) = cursor.position() else { return };
                let htrack = hscrollbar_track(bounds);
                let content_w = widget.total_content_width();
                let avail_w = bounds.width - SCROLLBAR_THICKNESS;
                let thumb_w = hthumb_len(htrack, content_w, avail_w);
                let travel = (htrack.width - thumb_w).max(1.0);
                let max_off = (content_w - avail_w).max(1.0);
                let dx = p.x - state.drag_start_cursor_x;
                let nsx = state.drag_start_offset_x + dx * (max_off / travel);
                state.scroll_x.set(clamp_scroll_x(nsx, content_w, avail_w));
                shell.request_redraw();
                shell.capture_event();
                return;
            }
            if state.dragging_scrollbar {
                let Some(p) = cursor.position() else { return };
                let scrollbar = scrollbar_track(content_bounds, viewport_h);
                let thumb_h = thumb_height(scrollbar, total_h);
                let travel = (scrollbar.height - thumb_h).max(1.0);
                let max_off = (total_h - viewport_h).max(1.0);
                let dy = p.y - state.drag_start_cursor_y;
                let new = state.drag_start_offset + dy * (max_off / travel);
                state.scroll_offset.set(clamp_scroll(new, total_h, content_bounds.height));
                shell.request_redraw();
                shell.capture_event();
                return;
            }
            if state.dragging_minimap {
                let Some(p) = cursor.position() else { return };
                let mm_rect = minimap::minimap_rect(
                    content_bounds, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS,
                );
                let dy = p.y - state.drag_start_minimap_y;
                let new = state.drag_start_minimap_scroll
                    + minimap::minimap_pixel_to_scroll(dy, mm_rect, total_h, viewport_h);
                state.scroll_offset.set(clamp_scroll(new, total_h, content_bounds.height));
                shell.request_redraw();
                shell.capture_event();
                return;
            }
            if state.selecting {
                let Some(p) = cursor.position() else { return };
                if p.y < bounds.y + HEADER_HEIGHT {
                    return;
                }
                // Clamp the y-coordinate to the content area so the selection
                // still extends when the mouse is dragged below the canvas.
                let clamped_y = p.y.clamp(
                    content_bounds.y,
                    content_bounds.y + content_bounds.height - 1.0,
                );
                let clamped = Point::new(p.x, clamped_y);
                if let Some(addr) = addr_at(
                    clamped, content_bounds,
                    state.scroll_offset.get(), state.scroll_x.get(),
                    widget.bytes_per_row, total_len,
                    widget.addr_col_width(),
                ) {
                    if let Some(cb) = &widget.on_extend_to {
                        shell.publish(cb(addr));
                    }
                    shell.request_redraw();
                    shell.capture_event();
                }
            }
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            let mut consumed = false;
            if state.dragging_scrollbar {
                state.dragging_scrollbar = false;
                consumed = true;
            }
            if state.dragging_scrollbar_x {
                state.dragging_scrollbar_x = false;
                consumed = true;
            }
            if state.dragging_minimap {
                state.dragging_minimap = false;
                consumed = true;
            }
            if state.selecting {
                state.selecting = false;
                consumed = true;
            }
            if consumed {
                shell.capture_event();
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            state.shift_pressed.set(modifiers.shift());
        }
        Event::Keyboard(keyboard::Event::KeyPressed {
            key, modifiers, text, ..
        }) => {
            if !cursor.is_over(bounds) {
                return;
            }

            // ── Edit-mode keys take priority ─────────────────────────
            if widget.edit.is_some() {
                match key {
                    keyboard::Key::Named(key::Named::Escape) => {
                        if let Some(cb) = &widget.on_edit_cancel {
                            shell.publish(cb());
                            shell.capture_event();
                            return;
                        }
                    }
                    keyboard::Key::Named(key::Named::Enter | key::Named::Tab) => {
                        if let Some(cb) = &widget.on_edit_commit {
                            shell.publish(cb(true));
                            shell.capture_event();
                            return;
                        }
                    }
                    keyboard::Key::Named(key::Named::Backspace) => {
                        if let Some(cb) = &widget.on_edit_backspace {
                            shell.publish(cb());
                            shell.capture_event();
                            return;
                        }
                    }
                    _ => {}
                }
            }

            // F2 starts a hex-digit edit at the current cursor.
            if widget.write_mode == WriteMode::Hex
                && matches!(key, keyboard::Key::Named(key::Named::F2))
                && widget.edit.is_none()
            {
                if let Some(cb) = &widget.on_begin_edit {
                    shell.publish(cb(widget.selection.cursor));
                    shell.capture_event();
                    return;
                }
            }

            // CTRL+E creates a pattern from the current selection.
            if (modifiers.control() || modifiers.command())
                && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "e")
            {
                if let Some(cb) = &widget.on_create_pattern {
                    shell.publish(cb());
                    shell.capture_event();
                    return;
                }
            }

            // Ctrl+G opens the goto dialog.
            if (modifiers.control() || modifiers.command())
                && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "g")
            {
                if let Some(cb) = &widget.on_open_goto {
                    shell.publish(cb());
                    shell.capture_event();
                    return;
                }
            }

            // Ctrl+F opens the search overlay.
            if (modifiers.control() || modifiers.command())
                && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "f")
            {
                if let Some(cb) = &widget.on_open_search {
                    shell.publish(cb());
                    shell.capture_event();
                    return;
                }
            }

            // Ctrl+C copies the selected byte range as hex text.
            if (modifiers.control() || modifiers.command())
                && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "c")
            {
                if let Some(cb) = &widget.on_copy_selection {
                    shell.publish(cb());
                    shell.capture_event();
                    return;
                }
            }

            // Ctrl+V pastes hex bytes from the clipboard.
            if (modifiers.control() || modifiers.command())
                && matches!(key, keyboard::Key::Character(c) if c.to_lowercase() == "v")
            {
                if let Some(cb) = &widget.on_paste {
                    shell.publish(cb());
                    shell.capture_event();
                    return;
                }
            }

            // Character typing.
            let mods_blocked = if widget.write_mode == WriteMode::Hex {
                modifiers.control() || modifiers.command() || modifiers.alt()
            } else {
                modifiers.control() || modifiers.command()
            };
            if !mods_blocked {
                if let Some(t) = text {
                    let c = if widget.write_mode == WriteMode::Hex {
                        first_hex_char(t)
                    } else {
                        first_printable_char(t)
                    };
                    if let Some(c) = c {
                        if widget.write_mode == WriteMode::Hex {
                            if widget.edit.is_some() {
                                if let Some(cb) = &widget.on_edit_type {
                                    shell.publish(cb(c));
                                    shell.capture_event();
                                    return;
                                }
                            } else if !widget.bytes.is_empty() {
                                if let Some(begin) = &widget.on_begin_edit {
                                    shell.publish(begin(widget.selection.cursor));
                                }
                                if let Some(typ) = &widget.on_edit_type {
                                    shell.publish(typ(c));
                                }
                                shell.capture_event();
                                return;
                            }
                        } else {
                            if !widget.bytes.is_empty() {
                                if let Some(cb) = &widget.on_edit_type {
                                    shell.publish(cb(c));
                                    shell.capture_event();
                                    return;
                                }
                            }
                        }
                    }
                }
            }

            // Text-mode Backspace / Delete.
            if widget.write_mode != WriteMode::Hex {
                if matches!(key, keyboard::Key::Named(key::Named::Backspace)) {
                    if let Some(cb) = &widget.on_edit_backspace {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                } else if matches!(key, keyboard::Key::Named(key::Named::Delete)) {
                    if let Some(cb) = &widget.on_delete_byte {
                        shell.publish(cb());
                        shell.capture_event();
                        return;
                    }
                }
            }

            // Navigation.
            if modifiers.control() || modifiers.command() {
                let dir = match key {
                    keyboard::Key::Named(key::Named::Home) => Some(NavDir::DocumentStart),
                    keyboard::Key::Named(key::Named::End) => Some(NavDir::DocumentEnd),
                    _ => None,
                };
                if let Some(dir) = dir {
                    widget.publish_nav(state, dir, modifiers.shift(), bounds, shell);
                    shell.capture_event();
                }
                return;
            }
            let dir = match key {
                keyboard::Key::Named(key::Named::ArrowLeft) => Some(NavDir::Left),
                keyboard::Key::Named(key::Named::ArrowRight) => Some(NavDir::Right),
                keyboard::Key::Named(key::Named::ArrowUp) => Some(NavDir::Up),
                keyboard::Key::Named(key::Named::ArrowDown) => Some(NavDir::Down),
                keyboard::Key::Named(key::Named::Home) => Some(NavDir::LineStart),
                keyboard::Key::Named(key::Named::End) => Some(NavDir::LineEnd),
                keyboard::Key::Named(key::Named::PageUp) => Some(NavDir::PageUp),
                keyboard::Key::Named(key::Named::PageDown) => Some(NavDir::PageDown),
                _ => None,
            };
            if let Some(dir) = dir {
                widget.publish_nav(state, dir, modifiers.shift(), bounds, shell);
                shell.capture_event();
            }
        }
        _ => {}
    }
}
