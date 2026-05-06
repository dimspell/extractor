use iced::widget::{canvas, Canvas};
use iced::{Color, Element, Point, Rectangle, Renderer, Size};

pub fn waveform_canvas(points: &[(f32, f32)]) -> Element<'static, crate::message::Message> {
    Canvas::new(WaveformProgram {
        points: points.to_vec(),
    })
    .width(iced::Length::Fill)
    .height(iced::Length::Fixed(120.0))
    .into()
}

struct WaveformProgram {
    points: Vec<(f32, f32)>,
}

impl canvas::Program<crate::message::Message> for WaveformProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let n = self.points.len();
        if n == 0 {
            return vec![];
        }

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let bar_w = bounds.width / n as f32;
        let mid_y = bounds.height / 2.0;
        let bar_color = Color::from_rgb(0.4, 0.7, 0.4);

        for (i, &(lo, hi)) in self.points.iter().enumerate() {
            let x = i as f32 * bar_w;
            let y_top = mid_y - hi * mid_y;
            let y_bot = mid_y - lo * mid_y;
            let bar_h = (y_bot - y_top).max(1.0);

            frame.fill_rectangle(
                Point::new(x, y_top),
                Size::new(bar_w.max(1.0), bar_h),
                bar_color,
            );
        }

        vec![frame.into_geometry()]
    }
}
