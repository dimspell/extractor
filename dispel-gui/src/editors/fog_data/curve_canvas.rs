//! The fog-curve canvas: draws one level's 512-sample flicker curve over a
//! subtle grid and turns pointer input into pair selection / factor painting.
//!
//! Mapping: x → pixel-pair index (`0..512`), y → brightness factor
//! (`0..=31`, inverted so brighter is up).

use crate::editors::fog_data::message::FogDataMessage;
use crate::message::MessageExt;
use dispel_core::map::fogdata::{MAX_FACTOR, ROW_LEN};
use iced::widget::canvas::{self, Action, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Event, Point, Rectangle, Renderer, Size, mouse};

// ── Palette ───────────────────────────────────────────────────────────────────

/// Canvas backdrop — darker than the app background, consistent with the
/// map/hex canvases.
const BG: Color = Color::from_rgb(0.063, 0.063, 0.078);
/// Horizontal grid lines at factor levels.
const GRID_VALUE: Color = Color {
    a: 0.14,
    ..Color::from_rgb(0.85, 0.78, 0.60)
};
/// The 0 and MAX_FACTOR boundary lines are slightly stronger.
const GRID_EDGE: Color = Color {
    a: 0.28,
    ..Color::from_rgb(0.85, 0.78, 0.60)
};
/// Vertical grid lines every 64 pairs.
const GRID_PAIR: Color = Color {
    a: 0.08,
    ..Color::from_rgb(0.85, 0.78, 0.60)
};
/// The flicker curve itself — warm gold, echoing the Medieval accent.
const CURVE: Color = Color::from_rgb(0.891, 0.769, 0.412);
/// Translucent area under the curve.
const CURVE_FILL: Color = Color { a: 0.10, ..CURVE };
/// Selected-pair marker + hairline.
const SELECTION: Color = Color::from_rgb(0.918, 0.878, 0.784);
const SELECTION_HAIRLINE: Color = Color {
    a: 0.30,
    ..SELECTION
};
/// Hover crosshair (cursor position only — costs no messages).
const HOVER: Color = Color {
    a: 0.35,
    ..Color::from_rgb(1.0, 1.0, 1.0)
};
const LABEL: Color = Color {
    a: 0.45,
    ..Color::from_rgb(0.85, 0.78, 0.60)
};

/// Canvas program state: whether the left button is currently held inside
/// this widget (i.e. a paint stroke is in progress).
#[derive(Debug, Default, Clone, Copy)]
pub struct CurveCanvasState {
    painting: bool,
}

/// Borrowed view of the editor data, used as the canvas [`canvas::Program`].
pub struct FogCurveCanvas<'a> {
    pub tab_id: usize,
    /// The selected level's 512 samples.
    pub row: &'a [u8],
    pub selected_pair: usize,
}

impl<'a> FogCurveCanvas<'a> {
    pub fn into_element(self) -> Element<'a, crate::message::Message> {
        iced::widget::Canvas::new(self)
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
}

/// Pixel x → pair index (`0..ROW_LEN`).
fn pair_from_x(x: f32, width: f32) -> usize {
    if width <= 0.0 {
        return 0;
    }
    let fraction = (x / width).clamp(0.0, 1.0);
    ((fraction * (ROW_LEN - 1) as f32).round() as usize).min(ROW_LEN - 1)
}

/// Pixel y → quantized factor (`0..=MAX_FACTOR`, y grows downward).
fn value_from_y(y: f32, height: f32) -> u8 {
    if height <= 0.0 {
        return 0;
    }
    let fraction = 1.0 - (y / height).clamp(0.0, 1.0);
    ((fraction * MAX_FACTOR as f32).round() as u8).min(MAX_FACTOR)
}

/// Pair index → sample x coordinate.
fn x_of_pair(pair: usize, width: f32) -> f32 {
    (pair as f32 / (ROW_LEN - 1) as f32) * width
}

/// Factor value → sample y coordinate (brighter is up).
fn y_of_value(value: u8, height: f32) -> f32 {
    (1.0 - value as f32 / MAX_FACTOR as f32) * height
}

impl<'a> canvas::Program<crate::message::Message> for FogCurveCanvas<'a> {
    type State = CurveCanvasState;

    fn update(
        &self,
        interaction: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<crate::message::Message>> {
        use mouse::{Button, Event as MouseEvent};

        match event {
            // Stroke start: select the pair and paint it immediately.
            // (Painting implies selection — one message, no `and_publish`.)
            Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
                let pos = cursor.position_in(bounds)?;
                interaction.painting = true;
                let pair = pair_from_x(pos.x, bounds.width);
                let value = value_from_y(pos.y, bounds.height);
                Some(
                    Action::publish(crate::message::Message::fog_data(
                        FogDataMessage::FactorPainted(self.tab_id, pair, value),
                    ))
                    .and_capture(),
                )
            }
            // Dragging paints along the path.
            Event::Mouse(MouseEvent::CursorMoved { .. }) if interaction.painting => {
                let pos = cursor.position_in(bounds)?;
                let pair = pair_from_x(pos.x, bounds.width);
                let value = value_from_y(pos.y, bounds.height);
                Some(
                    Action::publish(crate::message::Message::fog_data(
                        FogDataMessage::FactorPainted(self.tab_id, pair, value),
                    ))
                    .and_capture(),
                )
            }
            // Stroke end: commit the undo snapshot.
            Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
                if !interaction.painting {
                    return None;
                }
                interaction.painting = false;
                Some(
                    Action::publish(crate::message::Message::fog_data(
                        FogDataMessage::StrokeEnded(self.tab_id),
                    ))
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let width = bounds.width;
        let height = bounds.height;
        let mut frame = Frame::new(renderer, bounds.size());

        // Backdrop
        frame.fill_rectangle(Point::ORIGIN, Size::new(width, height), BG);

        // Horizontal grid at meaningful factor levels (0/8/16/24/31).
        for &level in &[0u8, 8, 16, 24] {
            let y = y_of_value(level, height);
            stroke_line(
                &mut frame,
                Point::new(0.0, y),
                Point::new(width, y),
                GRID_VALUE,
                1.0,
            );
        }
        let top_y = y_of_value(MAX_FACTOR, height);
        stroke_line(
            &mut frame,
            Point::new(0.0, top_y),
            Point::new(width, top_y),
            GRID_EDGE,
            1.0,
        );
        let bottom_y = y_of_value(0, height);
        stroke_line(
            &mut frame,
            Point::new(0.0, bottom_y),
            Point::new(width, bottom_y),
            GRID_EDGE,
            1.0,
        );

        // Vertical grid every 64 pairs.
        for pair in (0..ROW_LEN).step_by(64) {
            let x = x_of_pair(pair, width);
            stroke_line(
                &mut frame,
                Point::new(x, 0.0),
                Point::new(x, height),
                GRID_PAIR,
                1.0,
            );
        }

        // Axis labels — tiny, unobtrusive.
        for &level in &[0u8, 8, 16, 24, 31] {
            frame.fill_text(Text {
                content: level.to_string(),
                position: Point::new(3.0, y_of_value(level, height) + 2.0),
                color: LABEL,
                size: 9.0.into(),
                ..Default::default()
            });
        }
        for &pair in &[0usize, 128, 256, 384, 511] {
            frame.fill_text(Text {
                content: pair.to_string(),
                position: Point::new((x_of_pair(pair, width) - 7.0).max(1.0), height - 12.0),
                color: LABEL,
                size: 9.0.into(),
                ..Default::default()
            });
        }

        if self.row.len() == ROW_LEN && width > 0.0 && height > 0.0 {
            // Area fill under the curve.
            let area = Path::new(|p| {
                p.move_to(Point::new(0.0, bottom_y));
                for (pair, &value) in self.row.iter().enumerate() {
                    p.line_to(Point::new(
                        x_of_pair(pair, width),
                        y_of_value(value, height),
                    ));
                }
                p.line_to(Point::new(width, bottom_y));
                p.close();
            });
            frame.fill(&area, CURVE_FILL);

            // The curve itself.
            let line = Path::new(|p| {
                let mut first = true;
                for (pair, &value) in self.row.iter().enumerate() {
                    let point = Point::new(x_of_pair(pair, width), y_of_value(value, height));
                    if first {
                        p.move_to(point);
                        first = false;
                    } else {
                        p.line_to(point);
                    }
                }
            });
            frame.stroke(&line, Stroke::default().with_color(CURVE).with_width(1.5));
        }

        // Selected pair: hairline + filled dot with ring.
        let sel_x = x_of_pair(self.selected_pair.min(ROW_LEN - 1), width);
        stroke_line(
            &mut frame,
            Point::new(sel_x, 0.0),
            Point::new(sel_x, height),
            SELECTION_HAIRLINE,
            1.0,
        );
        let sel_value = self
            .row
            .get(self.selected_pair.min(ROW_LEN - 1))
            .copied()
            .unwrap_or(0);
        let center = Point::new(sel_x, y_of_value(sel_value, height));
        frame.fill(&Path::circle(center, 4.0), SELECTION);
        frame.stroke(
            &Path::circle(center, 6.5),
            Stroke::default().with_color(SELECTION).with_width(1.0),
        );

        // Hover crosshair — derived straight from the cursor, no messages.
        if let Some(pos) = cursor.position_in(bounds) {
            let hover_pair = pair_from_x(pos.x, width);
            let hover_value = value_from_y(pos.y, height);
            let hx = x_of_pair(hover_pair, width);
            let hy = y_of_value(hover_value, height);
            stroke_line(
                &mut frame,
                Point::new(hx, 0.0),
                Point::new(hx, height),
                Color { a: 0.35, ..HOVER },
                1.0,
            );
            stroke_line(
                &mut frame,
                Point::new(0.0, hy),
                Point::new(width, hy),
                Color { a: 0.20, ..HOVER },
                1.0,
            );
            frame.stroke(
                &Path::circle(Point::new(hx, hy), 3.0),
                Stroke::default().with_color(HOVER).with_width(1.0),
            );
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::Idle
        }
    }
}

fn stroke_line(frame: &mut Frame, from: Point, to: Point, color: Color, width: f32) {
    let path = Path::line(from, to);
    frame.stroke(&path, Stroke::default().with_color(color).with_width(width));
}
