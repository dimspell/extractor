// ── Canvas input handling (MapCanvas Program) ────────────────────────────────

use super::hit_test::find_hovered_element;
use super::state::MapCanvasState;
use crate::editors::map_editor::message::{MapEditorMessage, SelectedEntity};
use crate::editors::map_editor::state::MapEditorState;
use crate::message::{Message, MessageExt};
use iced::widget::canvas::{self, Action, Frame, Geometry};
use iced::{mouse, Event, Rectangle};

/// Borrowed view of the map editor state, used as the canvas Program.
pub struct MapCanvas<'a> {
    pub state: &'a MapEditorState,
    pub tab_id: usize,
}

impl<'a> canvas::Program<Message> for MapCanvas<'a> {
    type State = MapCanvasState;

    fn update(
        &self,
        interaction: &mut MapCanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        use mouse::{Button, Event as MouseEvent, ScrollDelta};

        match event {
            Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
                // cursor.position_in(bounds) → canvas-local coords, None if outside canvas.
                // This is the critical fix: cursor.position() returns ABSOLUTE window
                // coordinates and is Some() even when the cursor is over the inspector
                // panel. Using position_in ensures we only start a drag/click when the
                // cursor is actually inside the canvas area.
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
                // Emit click only if released inside canvas and barely moved from press.
                if let Some(start) = interaction.drag_start.take() {
                    if let Some(pos) = cursor.position_in(bounds) {
                        let dx = pos.x - start.x;
                        let dy = pos.y - start.y;
                        if dx * dx + dy * dy < 25.0 {
                            return Some(
                                Action::publish(Message::map_editor(
                                    MapEditorMessage::CanvasClicked(self.tab_id, pos.x, pos.y),
                                ))
                                .and_capture(),
                            );
                        }
                    }
                }
            }
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if interaction.is_dragging {
                    // Use position_from for drag: gives canvas-local coords but works
                    // even when the cursor strays outside the canvas bounds.
                    if let Some(last) = interaction.drag_last {
                        if let Some(pos) = cursor.position_from(bounds.position()) {
                            let dx = pos.x - last.x;
                            let dy = pos.y - last.y;
                            interaction.drag_last = Some(pos);
                            return Some(
                                Action::publish(Message::map_editor(MapEditorMessage::PanChanged(
                                    self.tab_id,
                                    dx,
                                    dy,
                                )))
                                .and_capture(),
                            );
                        }
                    }
                } else {
                    // Update hover entity and cursor tile-coordinate overlay.
                    if let Some(pos) = cursor.position_in(bounds) {
                        // Recompute hovered entity each frame (cheap).
                        let hover = self.find_hovered_entity(pos.x, pos.y);
                        interaction.hovered_entity = hover;
                        return Some(Action::publish(Message::map_editor(
                            MapEditorMessage::MouseMoved(
                                self.tab_id,
                                pos.x,
                                pos.y,
                                bounds.width,
                                bounds.height,
                            ),
                        )));
                    } else {
                        interaction.hovered_entity = None;
                        return Some(Action::publish(Message::map_editor(
                            MapEditorMessage::MouseMoved(
                                self.tab_id,
                                f32::NAN,
                                f32::NAN,
                                0.0,
                                0.0,
                            ),
                        )));
                    }
                }
            }
            Event::Mouse(MouseEvent::CursorLeft) => {
                interaction.hovered_entity = None;
                return Some(Action::publish(Message::map_editor(
                    MapEditorMessage::MouseMoved(self.tab_id, f32::NAN, f32::NAN, 0.0, 0.0),
                )));
            }
            Event::Mouse(MouseEvent::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let scroll_y = match delta {
                    ScrollDelta::Lines { y, .. } => *y,
                    ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                if scroll_y.abs() > 0.001 {
                    // Multiplicative zoom: symmetric in/out, natural on trackpads.
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
                    return Some(
                        Action::publish(Message::map_editor(MapEditorMessage::ZoomChanged(
                            self.tab_id,
                            factor,
                            cx,
                            cy,
                        )))
                        .and_capture(),
                    );
                }
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _interaction: &MapCanvasState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let frame = Frame::new(renderer, bounds.size());
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        interaction: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if interaction.is_dragging {
            return mouse::Interaction::Grabbing;
        }
        if cursor.is_over(bounds) {
            if interaction.hovered_entity.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a> MapCanvas<'a> {
    /// Find the element (entity / collision tile / event tile) under the cursor.
    fn find_hovered_entity(&self, cx: f32, cy: f32) -> Option<SelectedEntity> {
        find_hovered_element(self.state, cx, cy)
    }
}
