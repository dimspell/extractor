use crate::components::map_render::MapCanvasState;
use iced::mouse::{Button, Event as MouseEvent, ScrollDelta};
use iced::widget::canvas::Action;
use iced::{mouse, Event, Rectangle};

pub fn handle_input<M, Click, Pan, Zoom>(
    interaction: &mut MapCanvasState,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    on_click: Click,
    on_pan: Pan,
    on_zoom: Zoom,
) -> Option<Action<M>>
where
    Click: Fn(f32, f32) -> M,
    Pan: Fn(f32, f32) -> M,
    Zoom: Fn(f32, f32, f32) -> M,
{
    match event {
        Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
            if let Some(pos) = cursor.position_in(bounds) {
                interaction.is_dragging = true;
                interaction.drag_last = Some(pos);
                interaction.drag_start = Some(pos);
                return Some(Action::capture());
            }
        }
        Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
            interaction.is_dragging = false;
            interaction.drag_last = None;
            if let Some(start) = interaction.drag_start.take() {
                if let Some(pos) = cursor.position_in(bounds) {
                    let dx = pos.x - start.x;
                    let dy = pos.y - start.y;
                    if dx * dx + dy * dy < 25.0 {
                        return Some(Action::publish(on_click(pos.x, pos.y)).and_capture());
                    }
                }
            }
        }
        Event::Mouse(MouseEvent::CursorMoved { .. }) if interaction.is_dragging => {
            if let Some(last) = interaction.drag_last {
                if let Some(pos) = cursor.position_from(bounds.position()) {
                    let dx = pos.x - last.x;
                    let dy = pos.y - last.y;
                    interaction.drag_last = Some(pos);
                    return Some(Action::publish(on_pan(dx, dy)).and_capture());
                }
            }
        }
        Event::Mouse(MouseEvent::WheelScrolled { delta }) if cursor.is_over(bounds) => {
            let scroll_y = match delta {
                ScrollDelta::Lines { y, .. } => *y,
                ScrollDelta::Pixels { y, .. } => *y / 20.0,
            };
            if scroll_y.abs() > 0.001 {
                let magnitude = scroll_y.abs().min(3.0) * 0.12;
                let factor = if scroll_y > 0.0 {
                    1.0 + magnitude
                } else {
                    1.0 / (1.0 + magnitude)
                };
                let (cx, cy) = cursor
                    .position_in(bounds)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0.0, 0.0));
                return Some(Action::publish(on_zoom(factor, cx, cy)).and_capture());
            }
        }
        _ => {}
    }
    None
}
