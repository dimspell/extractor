//! Drawing primitives shared by the matrix and diff views.
//!
//! Both views paint byte cells and scrollbars with the same geometry. Keeping
//! these helpers in one place ensures the two views stay pixel-identical.

use iced::advanced::Renderer as _;
use iced::advanced::graphics::text::Paragraph as GraphicsParagraph;
use iced::advanced::renderer;
use iced::advanced::text::{self, Paragraph as _};
use iced::{Background, Border, Color, Font, Pixels, Rectangle, Shadow, Size, alignment};

use gui_widgets::components::paragraph_cache::{ParagraphCache, ParagraphKey};

type Paragraph = GraphicsParagraph;

// Match the geometry and text sizes used by the matrix and diff views.
const ROW_HEIGHT: f32 = 16.0;
const TEXT_SIZE: f32 = 11.0;
const SCROLLBAR_THICKNESS: f32 = 10.0;

pub(crate) fn fill_cell(
    renderer: &mut iced::Renderer,
    x: f32,
    y: f32,
    width: f32,
    color: Color,
    clip: Rectangle,
) {
    let cell = Rectangle {
        x,
        y,
        width,
        height: ROW_HEIGHT,
    };
    let Some(rect) = clip.intersection(&cell) else {
        return;
    };
    renderer.fill_quad(
        renderer::Quad {
            bounds: rect,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(color),
    );
}

pub(crate) fn draw_glyph_string(
    renderer: &mut iced::Renderer,
    cache: &ParagraphCache,
    text_str: &str,
    font: Font,
    bounds: Rectangle,
    color: Color,
    clip: Rectangle,
) {
    let key = ParagraphKey::new(text_str, TEXT_SIZE, bounds.width, font);
    let para = cache.get_or_insert(key, || {
        Paragraph::with_text(text::Text {
            content: text_str,
            bounds: Size::new(bounds.width, bounds.height),
            size: Pixels(TEXT_SIZE),
            line_height: text::LineHeight::default(),
            font,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
            ellipsis: text::Ellipsis::None,
            hint_factor: None,
        })
    });
    let pos = bounds.anchor(
        para.min_bounds(),
        alignment::Horizontal::Left,
        alignment::Vertical::Center,
    );
    let Some(cell_clip) = clip.intersection(&bounds) else {
        return;
    };
    <iced::Renderer as text::Renderer>::fill_paragraph(renderer, &para, pos, color, cell_clip);
}

pub(crate) fn scrollbar_track(bounds: Rectangle, viewport_h: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - SCROLLBAR_THICKNESS,
        y: bounds.y,
        width: SCROLLBAR_THICKNESS,
        height: viewport_h,
    }
}

pub(crate) fn thumb_height(track: Rectangle, total_h: f32) -> f32 {
    (track.height / total_h * track.height).max(20.0)
}

pub(crate) fn scrollbar_thumb(track: Rectangle, scroll: f32, total_h: f32) -> Rectangle {
    let h = thumb_height(track, total_h);
    let max_off = (total_h - track.height).max(1.0);
    let y = track.y + (scroll / max_off) * (track.height - h);
    Rectangle {
        x: track.x + 1.0,
        y,
        width: track.width - 2.0,
        height: h,
    }
}

pub(crate) fn hthumb_len(track: Rectangle, content_w: f32, _avail_w: f32) -> f32 {
    (track.width / content_w * track.width).max(20.0)
}

pub(crate) fn hscrollbar_thumb(
    track: Rectangle,
    scroll_x: f32,
    content_w: f32,
    avail_w: f32,
) -> Rectangle {
    let w = hthumb_len(track, content_w, avail_w);
    let max_off = (content_w - avail_w).max(1.0);
    let x = track.x + (scroll_x / max_off) * (track.width - w);
    Rectangle {
        x,
        y: track.y + 1.0,
        width: w,
        height: track.height - 2.0,
    }
}
