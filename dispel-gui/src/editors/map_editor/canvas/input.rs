// ── Canvas input handling (MapCanvas Program) ────────────────────────────────

use super::hit_test::find_hovered_element;
use crate::components::map_render::{handle_input, MapCanvasState};
use crate::editors::map_editor::message::MapEditorMessage;
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
        // 1. Delegate to shared pan/zoom/click handler.
        if let Some(action) = handle_input(
            interaction,
            event,
            bounds,
            cursor,
            |cx, cy| Message::map_editor(MapEditorMessage::CanvasClicked(self.tab_id, cx, cy)),
            |dx, dy| Message::map_editor(MapEditorMessage::PanChanged(self.tab_id, dx, dy)),
            |f, cx, cy| Message::map_editor(MapEditorMessage::ZoomChanged(self.tab_id, f, cx, cy)),
        ) {
            return Some(action);
        }

        // 2. Editor-specific hover handling (only when NOT dragging).
        use mouse::Event as MouseEvent;
        match event {
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
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
                    return Some(Action::publish(Message::map_editor(
                        MapEditorMessage::MouseMoved(self.tab_id, f32::NAN, f32::NAN, 0.0, 0.0),
                    )));
                }
            }
            Event::Mouse(MouseEvent::CursorLeft) => {
                return Some(Action::publish(Message::map_editor(
                    MapEditorMessage::MouseMoved(self.tab_id, f32::NAN, f32::NAN, 0.0, 0.0),
                )));
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
        _state: &MapCanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            if cursor
                .position_in(bounds)
                .is_some_and(|pos| find_hovered_element(self.state, pos.x, pos.y).is_some())
            {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::Idle
        }
    }
}
