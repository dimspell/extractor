//! 2D canvas rendering for the tile grid.
//!
//! Draws a 249×249 grid of axis-aligned colored squares, one per tile
//! descriptor. Color-coded by `object_id`:
//! - 0 = dark gray (empty)
//! - 1 = light gray (wall)
//! - 2+ = blue highlight
//!
//! Hovered/selected cells get a colored border.

use super::message::TileGridCanvasMessage;
use super::state::TileGridViewState;
use crate::message::{Message, MessageExt};
use dispel_core::references::tile_grid::{TileDescriptor, TileGrid};
use iced::widget::canvas::{self, Action, Frame, Geometry, Path, Stroke};
use iced::{Color, Event, Rectangle, mouse};

/// `iced::widget::canvas::Program` that renders the 2D tile grid.
pub struct TileGridCanvas<'a> {
    pub grid: &'a TileGrid,
    pub view: &'a TileGridViewState,
}

impl<'a> canvas::Program<Message> for TileGridCanvas<'a> {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        use mouse::Event as MouseEvent;
        match event {
            Event::Mouse(MouseEvent::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return Some(Action::publish(Message::tile_grid_canvas(
                        TileGridCanvasMessage::Click(pos.x, pos.y),
                    )));
                }
            }
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    return Some(Action::publish(Message::tile_grid_canvas(
                        TileGridCanvasMessage::Hover(pos.x, pos.y),
                    )));
                }
            }
            Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
                let factor = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => 1.0 + y * 0.1,
                    mouse::ScrollDelta::Pixels { y, .. } => 1.0 + y * 0.01,
                };
                return Some(Action::publish(Message::tile_grid_canvas(
                    TileGridCanvasMessage::Zoom(factor),
                )));
            }
            _ => {}
        }
        None
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let frame = Frame::new(renderer, bounds.size());
        // Draw directly into the frame; cache is handled at the view level
        // via iced's built-in canvas caching when the widget is used with
        // a stable program.
        let mut frame = frame;
        draw_grid(&mut frame, self.grid, self.view);
        vec![frame.into_geometry()]
    }
}

/// Draw the full 249×249 grid into a frame.
fn draw_grid(frame: &mut Frame, grid: &TileGrid, view: &TileGridViewState) {
    let cell_px = super::state::CELL_SIZE * view.zoom;
    let gs = grid.grid_size;

    // Clip to the visible area for performance
    let visible_x_start = ((-view.pan_x) / cell_px).max(0.0) as u32;
    let visible_y_start = ((-view.pan_y) / cell_px).max(0.0) as u32;
    let visible_x_end = ((frame.width() - view.pan_x) / cell_px + 1.0).min(gs as f32) as u32;
    let visible_y_end = ((frame.height() - view.pan_y) / cell_px + 1.0).min(gs as f32) as u32;

    for gy in visible_y_start..visible_y_end {
        for gx in visible_x_start..visible_x_end {
            let index = gy * gs + gx;
            if let Some(&word) = grid.data.get(index as usize) {
                let desc = TileDescriptor::from_word(word);
                let color = cell_color(desc.object_id);
                let x = gx as f32 * cell_px + view.pan_x;
                let y = gy as f32 * cell_px + view.pan_y;
                frame.fill_rectangle(
                    iced::Point::new(x, y),
                    iced::Size::new(cell_px, cell_px),
                    color,
                );
            }
        }
    }

    // Draw hover/selection borders
    for &index in [view.hovered_cell, view.selected_cell].iter().flatten() {
        let gx = (index % gs) as f32;
        let gy = (index / gs) as f32;
        let x = gx * cell_px + view.pan_x;
        let y = gy * cell_px + view.pan_y;
        let stroke_color = if Some(index) == view.selected_cell {
            Color::from_rgb(1.0, 0.8, 0.0) // gold for selected
        } else {
            Color::from_rgb(0.3, 0.7, 1.0) // light blue for hovered
        };
        let path = Path::rectangle(iced::Point::new(x, y), iced::Size::new(cell_px, cell_px));
        frame.stroke(
            &path,
            Stroke::default().with_color(stroke_color).with_width(2.0),
        );
    }
}

/// Color for a cell based on its `object_id`.
fn cell_color(object_id: u16) -> Color {
    match object_id {
        0 => Color::from_rgb(0.15, 0.15, 0.15), // empty: dark
        1 => Color::from_rgb(0.6, 0.6, 0.6),   // wall: light gray
        _ => Color::from_rgb(0.3, 0.5, 0.9),   // other: blue
    }
}
