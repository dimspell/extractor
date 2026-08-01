//! Mouse and keyboard event handling for the dual-buffer diff view.

use iced::advanced::layout::Layout;
use iced::advanced::Shell;
use iced::keyboard::{self, key};
use iced::mouse;
use iced::{Event, Rectangle};

use crate::ui::view::minimap::{self, MINIMAP_WIDTH};

use super::draw;
use super::layout::{self, clamp_scroll, ROW_HEIGHT, SCROLLBAR_THICKNESS};
use super::state::State;
use super::DiffView;

pub type Cursor = mouse::Cursor;

/// Handle all events for the diff view widget.
pub fn handle_event<Message>(
    widget: &mut DiffView<'_, Message>,
    state: &mut State,
    event: &Event,
    layout: Layout<'_>,
    cursor: Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let bounds = layout.bounds();
    let bpr = widget.bytes_per_row as usize;
    let bpr64 = widget.bytes_per_row as u64;
    let content_top = bounds.y + super::layout::HEADER_HEIGHT;
    let viewport_h = widget.content_viewport_h(bounds.height, bounds.width);
    let total_bytes = widget
        .baseline_bytes
        .len()
        .max(widget.comparison_bytes.len());
    let total_h = (total_bytes.div_ceil(bpr) as f32) * ROW_HEIGHT;

    match event {
        Event::Mouse(me) => match me {
            mouse::Event::WheelScrolled { delta } => {
                if !cursor.is_over(bounds) {
                    return;
                }
                match delta {
                    mouse::ScrollDelta::Lines { y, x, .. } => {
                        if *x != 0.0 {
                            let content_w = layout::total_content_width(
                                bpr,
                                !widget.row_annotations.is_empty(),
                            );
                            let avail_w = bounds.width - widget.right_strip();
                            if content_w > avail_w {
                                let dx = x * layout::HEX_CELL_WIDTH * 3.0;
                                let new_scroll =
                                    clamp_scroll_h(state.scroll_x.get() + dx, content_w, avail_w);
                                if (new_scroll - state.scroll_x.get()).abs() > f32::EPSILON {
                                    state.scroll_x.set(new_scroll);
                                    shell.request_redraw();
                                }
                            }
                        } else {
                            let dy = -y * ROW_HEIGHT * 3.0;
                            let new =
                                clamp_scroll(state.scroll_offset.get() + dy, total_h, viewport_h);
                            if (new - state.scroll_offset.get()).abs() > f32::EPSILON {
                                state.scroll_offset.set(new);
                                shell.request_redraw();
                            }
                        }
                        shell.capture_event();
                    }
                    mouse::ScrollDelta::Pixels { y, x } => {
                        if *x != 0.0 {
                            let content_w = layout::total_content_width(
                                bpr,
                                !widget.row_annotations.is_empty(),
                            );
                            let avail_w = bounds.width - widget.right_strip();
                            if content_w > avail_w {
                                let new_scroll =
                                    clamp_scroll_h(state.scroll_x.get() + *x, content_w, avail_w);
                                if (new_scroll - state.scroll_x.get()).abs() > f32::EPSILON {
                                    state.scroll_x.set(new_scroll);
                                    shell.request_redraw();
                                }
                            }
                        } else {
                            let new =
                                clamp_scroll(state.scroll_offset.get() - *y, total_h, viewport_h);
                            if (new - state.scroll_offset.get()).abs() > f32::EPSILON {
                                state.scroll_offset.set(new);
                                shell.request_redraw();
                            }
                        }
                        shell.capture_event();
                    }
                }
            }

            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let pos = cursor.position().unwrap_or_default();
                let rel_x = pos.x - bounds.x;
                let rel_y = pos.y - content_top;

                // ── Vertical scrollbar drag ──
                if rel_x >= bounds.width - SCROLLBAR_THICKNESS
                    && rel_y >= 0.0
                    && rel_y <= viewport_h
                {
                    let track_h = viewport_h;
                    let thumb_h = (track_h / total_h * track_h).max(20.0);
                    let max_off = (total_h - viewport_h).max(1.0);
                    let frac = ((rel_y - thumb_h / 2.0) / (track_h - thumb_h)).clamp(0.0, 1.0);
                    state.scroll_offset.set(frac * max_off);
                    state.dragging_scrollbar = true;
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }

                // ── Horizontal scrollbar drag ──
                let content_w =
                    layout::total_content_width(bpr, !widget.row_annotations.is_empty());
                let avail_w = bounds.width - widget.right_strip();
                let _hscroll_y = bounds.y + bounds.height - SCROLLBAR_THICKNESS;
                if content_w > avail_w
                    && rel_y >= viewport_h
                    && rel_y <= viewport_h + SCROLLBAR_THICKNESS
                {
                    let track_w = bounds.width - SCROLLBAR_THICKNESS;
                    let thumb_w = (track_w / content_w * track_w).max(20.0);
                    let max_off = (content_w - avail_w).max(1.0);
                    let frac = ((rel_x - thumb_w / 2.0) / (track_w - thumb_w)).clamp(0.0, 1.0);
                    state.scroll_x.set(frac * max_off);
                    state.dragging_scrollbar_x = true;
                    shell.request_redraw();
                    shell.capture_event();
                    return;
                }

                // ── Minimap click / drag ──
                let has_vscroll = total_h > viewport_h;
                if widget.show_minimap && has_vscroll {
                    let cb = Rectangle {
                        x: bounds.x,
                        y: content_top,
                        width: bounds.width,
                        height: viewport_h,
                    };
                    let mm_rect =
                        minimap::minimap_rect(cb, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS);
                    if mm_rect.contains(pos) {
                        let thumb_r = minimap::minimap_thumb_rect(
                            mm_rect,
                            state.scroll_offset.get(),
                            total_h,
                            viewport_h,
                        );
                        if thumb_r.contains(pos) {
                            state.dragging_minimap = true;
                            state.drag_start_minimap_y = pos.y;
                            state.drag_start_minimap_scroll = state.scroll_offset.get();
                        } else {
                            let new_scroll =
                                minimap::minimap_scroll_from_y(pos.y, mm_rect, total_h, viewport_h);
                            let clamped = clamp_scroll(new_scroll, total_h, viewport_h);
                            state.scroll_offset.set(clamped);
                        }
                        shell.capture_event();
                        return;
                    }
                }

                // ── Click in data area → set cursor ──
                if rel_y >= 0.0 && rel_y <= viewport_h && rel_x >= 0.0 && rel_x <= bounds.width {
                    let scroll = state.scroll_offset.get();
                    let row = (scroll + rel_y) / ROW_HEIGHT;
                    let base_addr = (row as u64) * bpr64;
                    let col = draw::col_at_x(rel_x, bpr, state.scroll_x.get());
                    if let Some((byte_col, is_baseline)) = col {
                        let addr = base_addr + byte_col as u64;
                        if (addr as usize) < total_bytes {
                            state.last_clicked_baseline.set(is_baseline);
                            if let Some(cb) = &widget.on_select_at {
                                shell.publish(cb(addr, is_baseline));
                                shell.request_redraw();
                                shell.capture_event();
                            }
                            // Begin drag-extend tracking.
                            state.dragging_cursor = true;
                        }
                    }
                }
            }

            mouse::Event::ButtonPressed(mouse::Button::Right) => {
                if !cursor.is_over(bounds) {
                    return;
                }
                let pos = cursor.position().unwrap_or_default();
                let rel_x = pos.x - bounds.x;
                let rel_y = pos.y - content_top;
                // Right-click in data area → context menu.
                if rel_y >= 0.0 && rel_y <= viewport_h && rel_x >= 0.0 && rel_x <= bounds.width {
                    let scroll = state.scroll_offset.get();
                    let row = (scroll + rel_y) / ROW_HEIGHT;
                    let base_addr = (row as u64) * bpr64;
                    let col = draw::col_at_x(rel_x, bpr, state.scroll_x.get());
                    if let Some((byte_col, is_baseline)) = col {
                        let addr = base_addr + byte_col as u64;
                        if (addr as usize) < total_bytes {
                            state.last_clicked_baseline.set(is_baseline);
                            if let Some(cb) = &widget.on_right_click {
                                shell.publish(cb(addr, is_baseline));
                                shell.request_redraw();
                                // Note: intentionally NOT capturing the event here —
                                // the ContextMenu wrapper reads shell.is_event_captured()
                                // and skips opening if the event was captured.
                            }
                        }
                    }
                }
            }

            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                // Mirror the matrix view: only capture the release when a
                // drag interaction actually started inside this view.
                // Capturing unconditionally would poison the shared Shell for
                // sibling panes updated after this one (e.g. the inspector's
                // buttons never seeing their release → on_press never fires).
                let mut consumed = false;
                if state.dragging_scrollbar {
                    state.dragging_scrollbar = false;
                    consumed = true;
                }
                if state.dragging_scrollbar_x {
                    state.dragging_scrollbar_x = false;
                    consumed = true;
                }
                if state.dragging_cursor {
                    state.dragging_cursor = false;
                    consumed = true;
                }
                if state.dragging_minimap {
                    state.dragging_minimap = false;
                    consumed = true;
                }
                if consumed {
                    shell.capture_event();
                }
            }

            mouse::Event::CursorMoved { position } => {
                let pos = *position;
                let rel_x = pos.x - bounds.x;
                let rel_y = pos.y - content_top;

                // ── Scrollbar drag continuation ──
                if state.dragging_scrollbar {
                    if rel_y >= 0.0 && rel_y <= viewport_h {
                        let track_h = viewport_h;
                        let thumb_h = (track_h / total_h * track_h).max(20.0);
                        let max_off = (total_h - viewport_h).max(1.0);
                        let frac = ((rel_y - thumb_h / 2.0) / (track_h - thumb_h)).clamp(0.0, 1.0);
                        state.scroll_offset.set(frac * max_off);
                        shell.request_redraw();
                    }
                    return;
                }

                if state.dragging_scrollbar_x {
                    let content_w =
                        layout::total_content_width(bpr, !widget.row_annotations.is_empty());
                    let avail_w = bounds.width - widget.right_strip();
                    if content_w > avail_w {
                        let track_w = bounds.width - SCROLLBAR_THICKNESS;
                        let thumb_w = (track_w / content_w * track_w).max(20.0);
                        let max_off = (content_w - avail_w).max(1.0);
                        let frac = ((rel_x - thumb_w / 2.0) / (track_w - thumb_w)).clamp(0.0, 1.0);
                        state.scroll_x.set(frac * max_off);
                        shell.request_redraw();
                    }
                    return;
                }

                // ── Drag-extend selection ──
                if state.dragging_cursor
                    && rel_y >= 0.0
                    && rel_y <= viewport_h
                    && rel_x >= 0.0
                    && rel_x <= bounds.width
                {
                    let scroll = state.scroll_offset.get();
                    let row = (scroll + rel_y) / ROW_HEIGHT;
                    let base_addr = (row as u64) * bpr64;
                    let col = draw::col_at_x(rel_x, bpr, state.scroll_x.get());
                    if let Some((byte_col, is_baseline)) = col {
                        let addr = base_addr + byte_col as u64;
                        if (addr as usize) < total_bytes {
                            state.last_clicked_baseline.set(is_baseline);
                            if let Some(cb) = &widget.on_extend_to {
                                shell.publish(cb(addr, is_baseline));
                                shell.request_redraw();
                                shell.capture_event();
                                return;
                            }
                        }
                    }
                }

                // ── Minimap drag continuation ──
                if state.dragging_minimap {
                    if let Some(p) = cursor.position() {
                        let cb = Rectangle {
                            x: bounds.x,
                            y: content_top,
                            width: bounds.width,
                            height: viewport_h,
                        };
                        let mm_rect = minimap::minimap_rect(
                            cb,
                            viewport_h,
                            MINIMAP_WIDTH,
                            SCROLLBAR_THICKNESS,
                        );
                        let dy = p.y - state.drag_start_minimap_y;
                        let new = state.drag_start_minimap_scroll
                            + minimap::minimap_pixel_to_scroll(dy, mm_rect, total_h, viewport_h);
                        state
                            .scroll_offset
                            .set(clamp_scroll(new, total_h, viewport_h));
                        shell.request_redraw();
                        shell.capture_event();
                        return;
                    }
                }

                // ── Hover over minimap ──
                if widget.show_minimap && total_h > viewport_h {
                    let cb = Rectangle {
                        x: bounds.x,
                        y: content_top,
                        width: bounds.width,
                        height: viewport_h,
                    };
                    let mm_rect =
                        minimap::minimap_rect(cb, viewport_h, MINIMAP_WIDTH, SCROLLBAR_THICKNESS);
                    if let Some(p) = cursor.position() {
                        let now_mm = mm_rect.contains(p);
                        if now_mm != state.hovering_minimap.get() {
                            state.hovering_minimap.set(now_mm);
                            shell.request_redraw();
                        }
                    }
                }

                // ── Hover over scrollbar ──
                let now_hovering = rel_x >= bounds.width - SCROLLBAR_THICKNESS
                    && rel_y >= 0.0
                    && rel_y <= viewport_h;
                if now_hovering != state.hovering_scrollbar.get() {
                    state.hovering_scrollbar.set(now_hovering);
                    shell.request_redraw();
                }
            }

            _ => {}
        },

        Event::Keyboard(ke) => {
            handle_keyboard_event(widget, state, ke, layout, cursor, shell);
        }

        _ => {}
    }
}

/// Handle keyboard events for the diff view.
fn handle_keyboard_event<Message>(
    widget: &mut DiffView<'_, Message>,
    state: &mut State,
    event: &keyboard::Event,
    layout: Layout<'_>,
    cursor: Cursor,
    shell: &mut Shell<'_, Message>,
) {
    let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
        return;
    };

    if !cursor.is_over(layout.bounds()) {
        return;
    }

    let bpr = widget.bytes_per_row as u64;
    let total_bytes = widget
        .baseline_bytes
        .len()
        .max(widget.comparison_bytes.len()) as u64;
    if total_bytes == 0 {
        return;
    }
    let max_addr = total_bytes.saturating_sub(1);
    let viewport_h = widget.content_viewport_h(layout.bounds().height, layout.bounds().width);
    let page = layout::page_rows(viewport_h);
    let cursor_addr = widget.selection.cursor;

    // Ctrl/Cmd+Home/End → document start/end.
    if modifiers.command() || modifiers.control() {
        let dir = match key {
            keyboard::Key::Named(key::Named::Home) => {
                Some(crate::domain::selection::NavDir::DocumentStart)
            }
            keyboard::Key::Named(key::Named::End) => {
                Some(crate::domain::selection::NavDir::DocumentEnd)
            }
            _ => None,
        };
        if let Some(dir) = dir {
            let new_addr =
                crate::domain::selection::nav_target(cursor_addr, dir, bpr, page, max_addr);
            let extend = modifiers.shift();
            // Keyboard navigation has no side; keep the side of the last
            // mouse click so the inspector stays on the inspected file.
            let is_baseline = state.last_clicked_baseline.get();
            if extend {
                if let Some(cb) = &widget.on_extend_to {
                    shell.publish(cb(new_addr, is_baseline));
                }
            } else if let Some(cb) = &widget.on_select_at {
                shell.publish(cb(new_addr, is_baseline));
            }
            shell.request_redraw();
            // Scroll to new cursor.
            let total_h_f = (total_bytes.div_ceil(bpr) as f32) * ROW_HEIGHT;
            let new_scroll = layout::scroll_to_make_visible(
                state.scroll_offset.get(),
                new_addr,
                bpr,
                viewport_h,
                total_h_f,
            );
            if (new_scroll - state.scroll_offset.get()).abs() > f32::EPSILON {
                state.scroll_offset.set(new_scroll);
            }
            shell.capture_event();
            return;
        }

        // Ctrl+Down → next diff chunk.
        if matches!(key, keyboard::Key::Named(key::Named::ArrowDown)) {
            if let Some(cb) = &widget.on_diff_nav_next {
                shell.publish(cb());
                shell.request_redraw();
                shell.capture_event();
                return;
            }
        }
        // Ctrl+Up → previous diff chunk.
        if matches!(key, keyboard::Key::Named(key::Named::ArrowUp)) {
            if let Some(cb) = &widget.on_diff_nav_prev {
                shell.publish(cb());
                shell.request_redraw();
                shell.capture_event();
                return;
            }
        }
    }

    let dir = match key {
        keyboard::Key::Named(key::Named::ArrowLeft) => Some(crate::domain::selection::NavDir::Left),
        keyboard::Key::Named(key::Named::ArrowRight) => {
            Some(crate::domain::selection::NavDir::Right)
        }
        keyboard::Key::Named(key::Named::ArrowUp) => Some(crate::domain::selection::NavDir::Up),
        keyboard::Key::Named(key::Named::ArrowDown) => Some(crate::domain::selection::NavDir::Down),
        keyboard::Key::Named(key::Named::Home) => Some(crate::domain::selection::NavDir::LineStart),
        keyboard::Key::Named(key::Named::End) => Some(crate::domain::selection::NavDir::LineEnd),
        keyboard::Key::Named(key::Named::PageUp) => Some(crate::domain::selection::NavDir::PageUp),
        keyboard::Key::Named(key::Named::PageDown) => {
            Some(crate::domain::selection::NavDir::PageDown)
        }
        _ => None,
    };

    if let Some(dir) = dir {
        let extend = modifiers.shift();
        if extend {
            if let Some(cb) = &widget.on_nav {
                shell.publish(cb(dir, true));
                shell.request_redraw();
            }
            // Scroll to make new cursor visible, using nav_target as best estimate.
            let new_addr =
                crate::domain::selection::nav_target(cursor_addr, dir, bpr, page, max_addr);
            let total_h_f = (total_bytes.div_ceil(bpr) as f32) * ROW_HEIGHT;
            let new_scroll = layout::scroll_to_make_visible(
                state.scroll_offset.get(),
                new_addr,
                bpr,
                viewport_h,
                total_h_f,
            );
            if (new_scroll - state.scroll_offset.get()).abs() > f32::EPSILON {
                state.scroll_offset.set(new_scroll);
            }
            shell.capture_event();
        } else {
            let new_addr =
                crate::domain::selection::nav_target(cursor_addr, dir, bpr, page, max_addr);
            // Keyboard navigation has no side; keep the side of the last
            // mouse click so the inspector stays on the inspected file.
            let is_baseline = state.last_clicked_baseline.get();
            if let Some(cb) = &widget.on_select_at {
                shell.publish(cb(new_addr, is_baseline));
                shell.request_redraw();
            }
            // Scroll to make new cursor visible.
            let total_h_f = (total_bytes.div_ceil(bpr) as f32) * ROW_HEIGHT;
            let new_scroll = layout::scroll_to_make_visible(
                state.scroll_offset.get(),
                new_addr,
                bpr,
                viewport_h,
                total_h_f,
            );
            if (new_scroll - state.scroll_offset.get()).abs() > f32::EPSILON {
                state.scroll_offset.set(new_scroll);
            }
            shell.capture_event();
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn clamp_scroll_h(scroll: f32, content_w: f32, avail_w: f32) -> f32 {
    if content_w <= avail_w {
        0.0
    } else {
        scroll.clamp(0.0, content_w - avail_w)
    }
}
