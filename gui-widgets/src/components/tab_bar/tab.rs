//! A single tab data + draw encapsulation for the [`TabBar`](super::TabBar) widget.
//!
//! [`Tab`] holds the identity and visual state of one tab. Drawing is delegated
//! to [`Tab::draw`] so positioning logic is encapsulated in one place, preventing
//! the off-by-one bugs that plagued the original monolithic draw loop.

use iced::advanced::renderer::Quad;
use iced::advanced::text;
use iced::alignment;
use iced::{
    Background, Border, Color, Font, Pixels, Point, Rectangle, Size,
};

use lucide_icons::Icon;

use super::style::{Status, Style};
use super::TAB_HEIGHT;

/// Font name for Lucide icon glyphs.
const LUCIDE_FONT: Font = Font::new("lucide");

// ── Layout constants ──────────────────────────────────────────────────────────

/// Width reserved for the drag handle area (⋮⋮).
pub const DRAG_HANDLE_WIDTH: f32 = 14.0;

/// Width reserved for the close button (✕).
pub const CLOSE_BUTTON_WIDTH: f32 = 16.0;

/// Horizontal spacing between elements inside a tab.
pub const INNER_SPACING: f32 = 4.0;

/// Horizontal padding on each side of a tab.
pub const TAB_PADDING: f32 = 6.0;

/// The font size used for tab labels.
pub const LABEL_SIZE: Pixels = Pixels(11.0);

/// Approximate average character width (px) for the label font size.
pub const CHAR_WIDTH_ESTIMATE: f32 = 7.0;

// ── The Tab struct ────────────────────────────────────────────────────────────

/// Describes a single tab's identity and visual state.
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub label: String,
    pub modified: bool,
    pub pinned: bool,
}

impl Tab {
    /// Create a new tab with the given id and label.
    pub fn new(id: usize, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            modified: false,
            pinned: false,
        }
    }

    /// Builder: mark this tab as modified (shows a dot indicator).
    pub fn modified(mut self, v: bool) -> Self {
        self.modified = v;
        self
    }

    /// Builder: pin this tab (no drag handle, no close button).
    pub fn pinned(mut self, v: bool) -> Self {
        self.pinned = v;
        self
    }

    // ── Width computation ─────────────────────────────────────────

    /// Compute the full width this tab would occupy in the bar,
    /// including internal padding.
    pub fn content_width(&self) -> f32 {
        let label_width = self.label.len() as f32 * CHAR_WIDTH_ESTIMATE;
        let mut w = 0.0;

        if !self.pinned {
            w += DRAG_HANDLE_WIDTH + INNER_SPACING;
        } else {
            w += 14.0 + INNER_SPACING;
        }

        w += label_width;

        if self.modified {
            w += 8.0 + INNER_SPACING;
        }

        w += INNER_SPACING;

        if !self.pinned {
            w += CLOSE_BUTTON_WIDTH;
        }

        w + TAB_PADDING * 2.0
    }

    // ── Drawing ───────────────────────────────────────────────────

    /// Draw this tab within the given `bounds`.
    ///
    /// `status` determines the visual treatment: idle, active, hovered,
    /// drag-source (placeholder), or drop-target.
    /// `style` is the resolved [`Style`] for the given `status`.
    /// `hovered_close` indicates whether the cursor is over the close button.
    pub fn draw<Renderer: text::Renderer<Font = iced::Font>>(
        &self,
        renderer: &mut Renderer,
        bounds: Rectangle,
        status: Status,
        style: &Style,
        hovered_close: bool,
    ) {
        let font = renderer.default_font();
        let pad = TAB_PADDING;

        match status {
            Status::DragSource => {
                // Drag source: draw a transparent placeholder slot
                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default()
                            .rounded(
                                iced::border::Radius::default()
                                    .top_left(style.border_radius)
                                    .top_right(style.border_radius),
                            ),
                        ..Quad::default()
                    },
                    Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.2)),
                );
            }
            _ => {
                // Tab background with top-only rounded corners
                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default()
                            .color(style.border_color)
                            .width(style.border_width)
                            .rounded(
                                iced::border::Radius::default()
                                    .top_left(style.border_radius)
                                    .top_right(style.border_radius),
                            ),
                        ..Quad::default()
                    },
                    style.background,
                );

                if !self.pinned {
                    // ── Drag handle (left) ────────────────────────────
                    renderer.fill_text(
                        make_text(String::from(char::from(Icon::GripVertical)), LUCIDE_FONT, DRAG_HANDLE_WIDTH, text::Alignment::Center),
                        Point::new(bounds.x + pad, bounds.center_y()),
                        style.drag_handle_color,
                        bounds,
                    );

                    // ── Close button (right) ──────────────────────────
                    let close_x = bounds.x + bounds.width - pad - CLOSE_BUTTON_WIDTH / 2.0;
                    let close_color = if hovered_close {
                        style.close_button_hovered_color
                    } else {
                        style.close_button_color
                    };

                    // Red circle background on hover
                    if hovered_close {
                        let circle_size = CLOSE_BUTTON_WIDTH + 4.0;
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle {
                                    x: close_x - circle_size * 0.5,
                                    y: bounds.center_y() - circle_size * 0.5,
                                    width: circle_size,
                                    height: circle_size,
                                },
                                border: Border::default().rounded(circle_size * 0.5),
                                ..Quad::default()
                            },
                            style.close_button_background,
                        );
                    }

                    renderer.fill_text(
                        make_text(String::from(char::from(Icon::X)), LUCIDE_FONT, CLOSE_BUTTON_WIDTH, text::Alignment::Center),
                        Point::new(close_x, bounds.center_y()),
                        close_color,
                        bounds,
                    );

                    // ── Label (centered between handle and close) ─────
                    let label_x = bounds.x + pad + DRAG_HANDLE_WIDTH + INNER_SPACING;
                    let label_max_w = bounds.width
                        - 2.0 * pad
                        - DRAG_HANDLE_WIDTH
                        - CLOSE_BUTTON_WIDTH
                        - 2.0 * INNER_SPACING;
                    renderer.fill_text(
                        make_text(self.label.clone(), font, label_max_w, text::Alignment::Left),
                        Point::new(label_x, bounds.center_y()),
                        style.text_color,
                        bounds,
                    );

                    // ── Modified dot ─────────────────────────────────
                    if self.modified {
                        let label_w = self.label.len() as f32 * CHAR_WIDTH_ESTIMATE;
                        let dot_x = label_x + label_w + INNER_SPACING;
                        let dot_y = bounds.y + TAB_HEIGHT * 0.5 - 3.0;
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle {
                                    x: dot_x,
                                    y: dot_y,
                                    width: 6.0,
                                    height: 6.0,
                                },
                                border: Border::default().rounded(3.0),
                                ..Quad::default()
                            },
                            Background::Color(style.modified_dot_color),
                        );
                    }
                } else {
                    // ── Pinned tab: pin indicator + label ─────────────
                    renderer.fill_text(
                        make_text(String::from(char::from(Icon::Pin)), LUCIDE_FONT, 16.0, text::Alignment::Center),
                        Point::new(bounds.x + pad, bounds.center_y()),
                        style.pin_color,
                        bounds,
                    );
                    let label_x = bounds.x + pad + 14.0 + INNER_SPACING;
                    let label_max_w = bounds.width - pad - 14.0 - INNER_SPACING - pad;
                    renderer.fill_text(
                        make_text(self.label.clone(), font, label_max_w, text::Alignment::Left),
                        Point::new(label_x, bounds.center_y()),
                        style.text_color,
                        bounds,
                    );
                    if self.modified {
                        let label_w = self.label.len() as f32 * CHAR_WIDTH_ESTIMATE;
                        let dot_x = label_x + label_w + INNER_SPACING;
                        let dot_y = bounds.y + TAB_HEIGHT * 0.5 - 3.0;
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle {
                                    x: dot_x,
                                    y: dot_y,
                                    width: 6.0,
                                    height: 6.0,
                                },
                                border: Border::default().rounded(3.0),
                                ..Quad::default()
                            },
                            Background::Color(style.modified_dot_color),
                        );
                    }
                }
            }
        }
    }
}

// ── Text helper ───────────────────────────────────────────────────────────────

/// Build a [`text::Text`] value with sensible defaults for tab labels.
fn make_text<Font>(
    content: String,
    font: Font,
    width: f32,
    align_x: text::Alignment,
) -> text::Text<String, Font> {
    text::Text {
        content,
        bounds: Size::new(width, TAB_HEIGHT),
        size: LABEL_SIZE,
        line_height: text::LineHeight::Relative(1.0),
        font,
        align_x,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Basic,
        wrapping: text::Wrapping::None,
        ellipsis: text::Ellipsis::None,
        hint_factor: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::tab_bar::style::Style;

    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(0, "Test");
        assert_eq!(tab.id, 0);
        assert_eq!(tab.label, "Test");
        assert!(!tab.modified);
        assert!(!tab.pinned);
    }

    #[test]
    fn test_tab_builder_modified() {
        let tab = Tab::new(1, "Modified").modified(true);
        assert!(tab.modified);
    }

    #[test]
    fn test_tab_builder_pinned() {
        let tab = Tab::new(2, "Pinned").pinned(true);
        assert!(tab.pinned);
    }

    #[test]
    fn test_tab_builder_both() {
        let tab = Tab::new(3, "Both").modified(true).pinned(true);
        assert!(tab.modified);
        assert!(tab.pinned);
    }

    #[test]
    fn test_content_width_nonpinned() {
        let tab = Tab::new(0, "Hello");
        let w = tab.content_width();
        assert!(w > 0.0);
        // Verify it's at least the sum of fixed elements
        assert!(w >= TAB_PADDING * 2.0 + DRAG_HANDLE_WIDTH + CLOSE_BUTTON_WIDTH + 2.0 * INNER_SPACING);
    }

    #[test]
    fn test_content_width_pinned() {
        let tab = Tab::new(1, "Pinned").pinned(true);
        let w = tab.content_width();
        assert!(w > 0.0);
    }

    #[test]
    fn test_content_width_modified() {
        let tab_mod = Tab::new(2, "Mod").modified(true);
        let tab_norm = Tab::new(3, "Mod");
        assert!(tab_mod.content_width() > tab_norm.content_width());
    }

    #[test]
    fn test_style_default() {
        let style = Style::default();
        assert!(style.border_radius > 0.0);
    }

    #[test]
    fn test_style_active() {
        let style = Style::active();
        assert!(style.border_radius > 0.0);
    }

    #[test]
    fn test_status_eq() {
        assert_eq!(Status::Active, Status::Active);
        assert_ne!(Status::Active, Status::Idle);
    }
}
