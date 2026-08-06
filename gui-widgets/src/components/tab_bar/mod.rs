//! A custom [`TabBar`] widget with real drag-and-drop reordering.
//!
//! The widget handles layout, rendering, and mouse events internally using
//! a press-deadband-drag-release cycle (like `pane_grid`). Drag state lives
//! in the widget [`Tree`](iced::advanced::widget::Tree), not the app.

mod event;
mod style;
mod tab;

pub use event::TabBarEvent;
pub use style::{Catalog, Status, Style};
pub use tab::Tab;

use std::marker::PhantomData;

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text;
use iced::advanced::widget::Widget;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{mouse, overlay as iced_overlay};
use iced::alignment;
use iced::{
    Background, Border, Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Vector,
};
use lucide_icons::Icon;

use crate::lucide::LUCIDE_FONT;
use tab::{CLOSE_BUTTON_WIDTH, LABEL_SIZE, TAB_PADDING};

/// Deadband distance in logical pixels — the cursor must move this far
/// from the press origin to transition from "click" to "drag".
const DRAG_DEADBAND: f32 = 8.0;

/// Fixed height of each tab in logical pixels.
pub(crate) const TAB_HEIGHT: f32 = 30.0;

/// Width of each scroll button (left/right arrows when tabs overflow).
const SCROLL_BUTTON_WIDTH: f32 = 20.0;

/// Pixels scrolled per mouse-wheel tick or button click.
const SCROLL_STEP: f32 = 100.0;

// ── Drag-action state (stored in widget Tree) ─────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
enum Action {
    /// No drag in progress.
    #[default]
    Idle,
    /// Left button pressed on a tab, waiting to exceed the deadband.
    PressPending { tab_idx: usize, origin: Point },
    /// Dragging a tab — cursor moved past the deadband.
    Dragging {
        tab_idx: usize,
        origin: Point,
        /// The cursor X since last layout (for overlay position).
        current_x: f32,
        /// The cursor Y since last layout (for overlay position).
        current_y: f32,
    },
}

/// Per-widget state stored in the widget [`Tree`].
struct TabBarState {
    action: Action,
    hovered_tab: Option<usize>,
    hovered_close: bool,
    drop_target: Option<usize>,
    tab_widths: Vec<f32>,
    total_width: f32,
    /// Horizontal scroll offset when tabs overflow the bar width.
    scroll_offset: f32,
    /// Maximum valid scroll_offset (0 when all tabs fit).
    max_scroll: f32,
    /// Whether the left scroll button is hovered.
    hovered_scroll_left: bool,
    /// Whether the right scroll button is hovered.
    hovered_scroll_right: bool,
}

impl Default for TabBarState {
    fn default() -> Self {
        Self {
            action: Action::Idle,
            hovered_tab: None,
            hovered_close: false,
            drop_target: None,
            tab_widths: Vec::new(),
            total_width: 0.0,
            scroll_offset: 0.0,
            max_scroll: 0.0,
            hovered_scroll_left: false,
            hovered_scroll_right: false,
        }
    }
}

// ── The TabBar widget ─────────────────────────────────────────────────────────

/// A horizontally scrolling tab bar with real drag-and-drop reordering.
pub struct TabBar<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: style::Catalog,
    Renderer: text::Renderer<Font = iced::Font>,
{
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    spacing: f32,
    padding: f32,
    class: Theme::Class<'a>,
    on_event: Option<Box<dyn Fn(TabBarEvent) -> Message + 'a>>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> TabBar<'a, Message, Theme, Renderer>
where
    Theme: style::Catalog,
    Renderer: text::Renderer<Font = iced::Font>,
{
    /// Create a new [`TabBar`].
    pub fn new(tabs: Vec<Tab>, active_tab: Option<usize>) -> Self {
        Self {
            tabs,
            active_tab,
            spacing: 4.0,
            padding: 8.0,
            class: Theme::default(),
            on_event: None,
            _renderer: PhantomData,
        }
    }

    /// Set the spacing between tabs.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Set the padding around the tab bar.
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }

    /// Set the style class.
    pub fn class(mut self, class: Theme::Class<'a>) -> Self {
        self.class = class;
        self
    }

    /// Set the event handler closure.
    pub fn on_event<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(TabBarEvent) -> Message,
    {
        self.on_event = Some(Box::new(f));
        self
    }

    /// Compute the insertion gap index (0..=tabs.len()) for a given x position
    /// relative to the tab content area (excluding bar padding).
    /// Returns where a tab would land if dropped at this x.
    fn insertion_index_at_x(x: f32, tab_widths: &[f32], spacing: f32) -> usize {
        if x <= 0.0 {
            return 0;
        }
        let mut cx = 0.0f32;
        for (i, &tw) in tab_widths.iter().enumerate() {
            let mid = cx + tw * 0.5;
            if x <= mid {
                return i; // insert before this tab
            }
            cx += tw + spacing;
        }
        tab_widths.len() // insert after the last tab
    }

    fn publish(&self, shell: &mut iced::advanced::Shell<'_, Message>, event: TabBarEvent) {
        if let Some(on_event) = &self.on_event {
            shell.publish(on_event(event));
        }
    }
}

// ── Widget trait implementation ───────────────────────────────────────────────

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TabBar<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: style::Catalog + 'a,
    Renderer: text::Renderer<Font = iced::Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TabBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TabBarState::default())
    }

    fn diff(&mut self, _tree: &mut Tree) {}

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let tab_widths: Vec<f32> = self.tabs.iter().map(|tab| tab.content_width()).collect();

        let total_width: f32 = tab_widths.iter().copied().sum::<f32>()
            + (self.tabs.len().saturating_sub(1) as f32) * self.spacing;

        let state = tree.state.downcast_mut::<TabBarState>();
        state.tab_widths = tab_widths;
        state.total_width = total_width;

        // Available content width (excluding bar padding on each side)
        let max_bounds = limits.max();
        let available_content = max_bounds.width - self.padding * 2.0;
        let overflow = total_width > available_content;

        if overflow {
            // Scroll buttons occupy SCROLL_BUTTON_WIDTH on each side. The
            // visible content area width for tabs is therefore the widget
            // width minus both scroll button widths.  Padding is NOT
            // subtracted here because content_left and content_right in
            // draw() are derived from bounds.x/bounds.width which already
            // include the node's padding offset.  Subtracting it again
            // would over-shrink the visible area and leave a gap on the
            // right at max scroll.
            let content_width = max_bounds.width - SCROLL_BUTTON_WIDTH * 2.0;
            state.max_scroll = (total_width - content_width).max(0.0);
            state.scroll_offset = state.scroll_offset.clamp(0.0, state.max_scroll);
        } else {
            state.max_scroll = 0.0;
            state.scroll_offset = 0.0;
        }

        let total_height = TAB_HEIGHT + self.padding * 2.0;
        let children: Vec<layout::Node> = state
            .tab_widths
            .iter()
            .copied()
            .scan(0.0f32, |x, w| {
                let node = layout::Node::new(Size::new(w, TAB_HEIGHT)).move_to(Point::new(*x, 0.0));
                *x += w + self.spacing;
                Some(node)
            })
            .collect();

        // Widget width: fill available width when overflow, otherwise fit content
        let width = if overflow {
            max_bounds.width
        } else {
            total_width + self.padding * 2.0
        };

        layout::Node::with_children(Size::new(width, total_height), children)
            .move_to(Point::new(self.padding, self.padding))
    }

    #[allow(clippy::too_many_arguments)]
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TabBarState>();
        let has_overflow = state.max_scroll > 0.0;

        let tab_at_x = |x: f32| -> Option<usize> {
            let mut cx = 0.0f32;
            for (i, &tw) in state.tab_widths.iter().enumerate() {
                if x >= cx && x < cx + tw {
                    return Some(i);
                }
                cx += tw + self.spacing;
            }
            None
        };

        let tab_at_cursor = |bounds: Rectangle, cursor_pos: Point| -> Option<usize> {
            // Convert screen x to content-relative x via scroll_offset.
            // Content starts after the left scroll button when overflow.
            let content_left = if has_overflow {
                bounds.x + SCROLL_BUTTON_WIDTH
            } else {
                bounds.x
            };
            let x = cursor_pos.x - content_left + state.scroll_offset;
            let y = cursor_pos.y - bounds.y;
            if !(0.0..=TAB_HEIGHT).contains(&y) {
                return None;
            }
            // Skip hit when over scroll button slots (always reserved space)
            if has_overflow {
                let rel_x = cursor_pos.x - bounds.x;
                if rel_x < SCROLL_BUTTON_WIDTH || rel_x > bounds.width - SCROLL_BUTTON_WIDTH {
                    return None;
                }
            }
            tab_at_x(x)
        };

        let close_at_cursor = |tab_idx: usize, bounds: Rectangle, cursor_pos: Point| -> bool {
            let content_left = if has_overflow {
                bounds.x + SCROLL_BUTTON_WIDTH
            } else {
                bounds.x
            };
            let x = cursor_pos.x - content_left + state.scroll_offset;
            let y = cursor_pos.y - bounds.y;
            if !(0.0..=TAB_HEIGHT).contains(&y) {
                return false;
            }
            let mut cx = 0.0f32;
            for i in 0..=tab_idx {
                let tw = state.tab_widths.get(i).copied().unwrap_or(0.0);
                if i == tab_idx {
                    let close_x = cx + tw - CLOSE_BUTTON_WIDTH - TAB_PADDING;
                    return x >= close_x && x < cx + tw - TAB_PADDING;
                }
                cx += tw + self.spacing;
            }
            false
        };

        let content_rel_x = |bounds: Rectangle, cursor_pos: Point| -> f32 {
            let content_left = if has_overflow {
                bounds.x + SCROLL_BUTTON_WIDTH
            } else {
                bounds.x
            };
            cursor_pos.x - content_left + state.scroll_offset
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = layout.bounds();
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    shell.capture_event();

                    // Check scroll buttons first (only respond when visible)
                    if has_overflow {
                        let rel_x = cursor_pos.x - bounds.x;
                        if state.scroll_offset > 0.0 && rel_x < SCROLL_BUTTON_WIDTH {
                            // Left scroll
                            state.scroll_offset = (state.scroll_offset - SCROLL_STEP).max(0.0);
                            shell.request_redraw();
                            return;
                        }
                        if state.scroll_offset < state.max_scroll
                            && rel_x > bounds.width - SCROLL_BUTTON_WIDTH
                        {
                            // Right scroll
                            state.scroll_offset =
                                (state.scroll_offset + SCROLL_STEP).min(state.max_scroll);
                            shell.request_redraw();
                            return;
                        }
                    }

                    if let Some(tab_idx) = tab_at_cursor(bounds, cursor_pos) {
                        let tab = &self.tabs[tab_idx];

                        if !tab.pinned && close_at_cursor(tab_idx, bounds, cursor_pos) {
                            self.publish(shell, TabBarEvent::Closed(tab_idx));
                            return;
                        }

                        if tab.pinned {
                            self.publish(shell, TabBarEvent::Selected(tab_idx));
                        } else {
                            state.action = Action::PressPending {
                                tab_idx,
                                origin: cursor_pos,
                            };
                        }
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                let bounds = layout.bounds();
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    shell.capture_event();
                    if let Some(tab_idx) = tab_at_cursor(bounds, cursor_pos) {
                        self.publish(shell, TabBarEvent::Closed(tab_idx));
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let bounds = layout.bounds();
                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    shell.capture_event();
                    if let Some(tab_idx) = tab_at_cursor(bounds, cursor_pos) {
                        self.publish(shell, TabBarEvent::RightClicked(tab_idx));
                    }
                }
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                match state.action {
                    Action::PressPending { tab_idx, origin } => {
                        if let Some(cursor_pos) = cursor.position() {
                            if cursor_pos.distance(origin) <= DRAG_DEADBAND {
                                self.publish(shell, TabBarEvent::Selected(tab_idx));
                            } else {
                                self.publish(shell, TabBarEvent::Dragged(tab_idx, tab_idx));
                            }
                        } else {
                            self.publish(shell, TabBarEvent::DragCanceled(tab_idx));
                        }
                    }
                    Action::Dragging {
                        tab_idx, origin, ..
                    } => {
                        if let Some(cursor_pos) = cursor.position() {
                            if cursor_pos.distance(origin) > DRAG_DEADBAND {
                                let bounds = layout.bounds();
                                let rel_x = content_rel_x(bounds, cursor_pos);
                                let gap = Self::insertion_index_at_x(
                                    rel_x,
                                    &state.tab_widths,
                                    self.spacing,
                                );
                                self.publish(shell, TabBarEvent::Dragged(tab_idx, gap));
                            } else {
                                self.publish(shell, TabBarEvent::DragCanceled(tab_idx));
                            }
                        } else {
                            self.publish(shell, TabBarEvent::DragCanceled(tab_idx));
                        }
                    }
                    Action::Idle => {}
                }
                state.action = Action::Idle;
                state.drop_target = None;
            }

            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let bounds = layout.bounds();

                if let Some(cursor_pos) = cursor.position_over(bounds) {
                    let rel_x = cursor_pos.x - bounds.x;

                    // Scroll button hover (only when button is visible)
                    if has_overflow {
                        state.hovered_scroll_left =
                            state.scroll_offset > 0.0 && rel_x < SCROLL_BUTTON_WIDTH;
                        state.hovered_scroll_right = state.scroll_offset < state.max_scroll
                            && rel_x > bounds.width - SCROLL_BUTTON_WIDTH;
                        if state.hovered_scroll_left || state.hovered_scroll_right {
                            state.hovered_tab = None;
                            state.hovered_close = false;
                            shell.request_redraw();
                            return;
                        }
                    } else {
                        state.hovered_scroll_left = false;
                        state.hovered_scroll_right = false;
                    }

                    let tab_idx = tab_at_cursor(bounds, cursor_pos);
                    state.hovered_tab = tab_idx;

                    if let Some(t) = tab_idx {
                        state.hovered_close = close_at_cursor(t, bounds, cursor_pos);
                    } else {
                        state.hovered_close = false;
                    }

                    match state.action {
                        Action::PressPending { tab_idx, origin } => {
                            if cursor_pos.distance(origin) > DRAG_DEADBAND {
                                state.action = Action::Dragging {
                                    tab_idx,
                                    origin,
                                    current_x: cursor_pos.x,
                                    current_y: cursor_pos.y,
                                };
                                shell.request_redraw();
                            }
                        }
                        Action::Dragging {
                            tab_idx,
                            origin,
                            current_x: _,
                            current_y: _,
                        } => {
                            state.action = Action::Dragging {
                                tab_idx,
                                origin,
                                current_x: cursor_pos.x,
                                current_y: cursor_pos.y,
                            };
                            // Compute drop-target gap from cursor x (content-relative)
                            let content_x = content_rel_x(bounds, cursor_pos);
                            state.drop_target = Some(Self::insertion_index_at_x(
                                content_x,
                                &state.tab_widths,
                                self.spacing,
                            ));
                            shell.request_redraw();
                        }
                        Action::Idle => {
                            shell.request_redraw();
                        }
                    }
                } else {
                    state.hovered_tab = None;
                    state.hovered_close = false;
                    state.hovered_scroll_left = false;
                    state.hovered_scroll_right = false;
                    shell.request_redraw();
                    if matches!(state.action, Action::Dragging { .. }) {
                        state.drop_target = None;
                    }
                }
            }

            // ── Mouse wheel scrolling ────────────────────────────────
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let bounds = layout.bounds();
                if cursor.is_over(bounds) {
                    shell.capture_event();
                    let pixels = match delta {
                        iced::mouse::ScrollDelta::Lines { x, y } => {
                            // Treat each line as ~SCROLL_STEP pixels
                            let h = x + y;
                            if h == 0.0 {
                                return;
                            }
                            if h > 0.0 { -SCROLL_STEP } else { SCROLL_STEP }
                        }
                        iced::mouse::ScrollDelta::Pixels { x, y } => {
                            let h = x + y;
                            if h == 0.0 {
                                return;
                            }
                            -h
                        }
                    };
                    if has_overflow {
                        state.scroll_offset =
                            (state.scroll_offset + pixels).clamp(0.0, state.max_scroll);
                        shell.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TabBarState>();
        let bounds = layout.bounds();

        // Resolve a baseline style once (bar background, separator, drop indicator)
        let idle_style = theme.style(&self.class, Status::Idle);

        let _default_font = renderer.default_font();

        // Determine whether a drag is active (used for source de-emphasis and drop line)
        let drag_info = match state.action {
            Action::Dragging { tab_idx, .. } => Some(tab_idx),
            _ => None,
        };
        let is_dragging = drag_info.is_some();
        // The gap (0..=len) where the indicator line should appear.
        let drop_gap = if is_dragging { state.drop_target } else { None };

        // ── Scroll buttons: only show when there's room to scroll ────
        let has_overflow = state.max_scroll > 0.0;
        let scroll_left_x = bounds.x;
        let scroll_right_x = bounds.x + bounds.width - SCROLL_BUTTON_WIDTH;

        let show_left = has_overflow && state.scroll_offset > 0.0;
        let show_right = has_overflow && state.scroll_offset < state.max_scroll;

        if show_left {
            let btn_bounds = Rectangle {
                x: scroll_left_x,
                y: bounds.y,
                width: SCROLL_BUTTON_WIDTH,
                height: TAB_HEIGHT,
            };
            let btn_color = if state.hovered_scroll_left {
                idle_style.scroll_button_hovered_color
            } else {
                idle_style.scroll_button_color
            };
            renderer.fill_text(
                text::Text {
                    content: String::from(char::from(Icon::ChevronLeft)),
                    bounds: Size::new(SCROLL_BUTTON_WIDTH, TAB_HEIGHT),
                    size: LABEL_SIZE,
                    line_height: text::LineHeight::Relative(1.0),
                    font: LUCIDE_FONT,
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::None,
                    hint_factor: None,
                },
                btn_bounds.center(),
                btn_color,
                btn_bounds,
            );
        }
        if show_right {
            let btn_bounds = Rectangle {
                x: scroll_right_x,
                y: bounds.y,
                width: SCROLL_BUTTON_WIDTH,
                height: TAB_HEIGHT,
            };
            let btn_color = if state.hovered_scroll_right {
                idle_style.scroll_button_hovered_color
            } else {
                idle_style.scroll_button_color
            };
            renderer.fill_text(
                text::Text {
                    content: String::from(char::from(Icon::ChevronRight)),
                    bounds: Size::new(SCROLL_BUTTON_WIDTH, TAB_HEIGHT),
                    size: LABEL_SIZE,
                    line_height: text::LineHeight::Relative(1.0),
                    font: LUCIDE_FONT,
                    align_x: text::Alignment::Center,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                    ellipsis: text::Ellipsis::None,
                    hint_factor: None,
                },
                btn_bounds.center(),
                btn_color,
                btn_bounds,
            );
        }

        // ── Scissor clip: prevent tabs from spilling past visible area ──
        // The scroll-button slots are always reserved so tabs never shift.
        let content_left = if has_overflow {
            bounds.x + SCROLL_BUTTON_WIDTH
        } else {
            bounds.x
        };
        let content_right = if has_overflow {
            bounds.x + bounds.width - SCROLL_BUTTON_WIDTH
        } else {
            bounds.x + bounds.width
        };
        let content_bounds = Rectangle {
            x: content_left,
            y: bounds.y,
            width: (content_right - content_left).max(0.0),
            height: TAB_HEIGHT,
        };

        renderer.with_layer(content_bounds, |renderer| {
            let scroll_offset = state.scroll_offset;
            let mut x = content_left - scroll_offset;
            for (i, tab) in self.tabs.iter().enumerate() {
                let tab_w = state.tab_widths.get(i).copied().unwrap_or(100.0);
                let tab_bounds = Rectangle {
                    x,
                    y: bounds.y,
                    width: tab_w,
                    height: TAB_HEIGHT,
                };

                // Skip tabs entirely outside the visible area
                if x + tab_w < content_left || x > content_right {
                    x += tab_w + self.spacing;
                    continue;
                }

                let is_active = self.active_tab == Some(i);
                let is_hovered = state.hovered_tab == Some(i);
                let is_drag_source = drag_info == Some(i);

                // De-emphasize the source tab slot during drag
                if is_drag_source && is_dragging {
                    renderer.fill_quad(
                        Quad {
                            bounds: tab_bounds,
                            border: Border::default().rounded(
                                iced::border::Radius::default()
                                    .top_left(idle_style.border_radius)
                                    .top_right(idle_style.border_radius),
                            ),
                            ..Quad::default()
                        },
                        Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.2)),
                    );
                } else {
                    let status = if is_active {
                        Status::Active
                    } else if is_hovered {
                        Status::Hovered
                    } else {
                        Status::Idle
                    };

                    let tab_style = theme.style(&self.class, status);

                    // Tab background with top-only rounded corners
                    renderer.fill_quad(
                        Quad {
                            bounds: tab_bounds,
                            border: Border::default()
                                .color(tab_style.border_color)
                                .width(tab_style.border_width)
                                .rounded(
                                    iced::border::Radius::default()
                                        .top_left(tab_style.border_radius)
                                        .top_right(tab_style.border_radius),
                                ),
                            ..Quad::default()
                        },
                        tab_style.background,
                    );

                    tab.draw(
                        renderer,
                        tab_bounds,
                        status,
                        &tab_style,
                        is_hovered && state.hovered_close,
                    );
                }

                x += tab_w + self.spacing;
            }

            // ── Draw separator lines between tabs (offset by scroll) ──
            if !is_dragging {
                let mut sx = content_left - scroll_offset;
                for &tw in &state.tab_widths {
                    sx += tw; // right edge of this tab
                    let sep_x = sx;
                    // Only draw separators within the visible content area
                    if sep_x >= content_left && sep_x <= content_right {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle {
                                    x: sep_x,
                                    y: bounds.y + 6.0,
                                    width: idle_style.separator_width,
                                    height: TAB_HEIGHT - 12.0,
                                },
                                border: Border::default(),
                                ..Quad::default()
                            },
                            Background::Color(idle_style.separator_color),
                        );
                    }
                    sx += self.spacing;
                }
            }

            // ── Draw drop indicator line (offset by scroll) ───────────
            if let Some(gap) = drop_gap {
                let mut ix = content_left - scroll_offset;
                let mut remaining = gap;
                for &tw in &state.tab_widths {
                    if remaining == 0 {
                        break;
                    }
                    ix += tw + self.spacing;
                    remaining -= 1;
                }
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: ix - idle_style.drop_indicator_width * 0.5,
                            y: bounds.y + 4.0,
                            width: idle_style.drop_indicator_width,
                            height: TAB_HEIGHT - 8.0,
                        },
                        border: Border::default().rounded(1.0),
                        ..Quad::default()
                    },
                    Background::Color(idle_style.drop_indicator_color),
                );
            }
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: iced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> iced::mouse::Interaction {
        let state = tree.state.downcast_ref::<TabBarState>();
        if matches!(state.action, Action::Dragging { .. }) {
            iced::mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            iced::mouse::Interaction::Pointer
        } else {
            iced::mouse::Interaction::default()
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<iced_overlay::Element<'b, Message, Theme, Renderer>> {
        let state_ref = tree.state.downcast_ref::<TabBarState>();

        match state_ref.action {
            Action::Dragging {
                tab_idx,
                current_x,
                current_y,
                ..
            } if tab_idx < self.tabs.len() => {
                let tab = &self.tabs[tab_idx];
                let tab_width = state_ref.tab_widths.get(tab_idx).copied().unwrap_or(100.0);
                // Note: overlay() has no theme access, so ghost colors use
                // hardcoded defaults. The Style struct has ghost_* fields for
                // when Iced adds theme access to overlay() in the future.
                Some(iced_overlay::Element::new(Box::new(PickedTab {
                    label: tab.label.clone(),
                    tab_width,
                    current_x,
                    current_y,
                    ghost_background: Background::Color(Color::from_rgba(0.3, 0.25, 0.15, 0.85)),
                    ghost_text_color: Color::from_rgba(1.0, 0.9, 0.7, 0.9),
                    ghost_shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                })))
            }
            _ => None,
        }
    }
}

// ── Overlay: ghost tab following cursor ───────────────────────────────────────

struct PickedTab {
    label: String,
    tab_width: f32,
    current_x: f32,
    /// Absolute cursor Y in the viewport.
    current_y: f32,
    // Ghost styling — resolved from theme
    ghost_background: Background,
    ghost_text_color: Color,
    ghost_shadow_color: Color,
}

/// Y-offset applied to the ghost overlay so it appears "lifted" above the bar.
const GHOST_Y_OFFSET: f32 = -10.0;

/// Shadow offset from the ghost position.
const GHOST_SHADOW_OFFSET: f32 = 4.0;

impl<Message, Theme, Renderer> iced::advanced::Overlay<Message, Theme, Renderer> for PickedTab
where
    Renderer: text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        // Position the ghost at the absolute cursor position (viewport
        // coordinates), lifted slightly above the cursor so it appears
        // "peeled off" the tab bar.
        layout::Node::new(Size::new(self.tab_width, TAB_HEIGHT))
            .move_to(Point::new(self.current_x, self.current_y + GHOST_Y_OFFSET))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let default_font = renderer.default_font();

        // Drop shadow quad (offset behind ghost)
        renderer.fill_quad(
            Quad {
                bounds: Rectangle {
                    x: bounds.x + GHOST_SHADOW_OFFSET,
                    y: bounds.y + GHOST_SHADOW_OFFSET,
                    width: bounds.width,
                    height: bounds.height,
                },
                border: Border::default().rounded(4.0),
                ..Quad::default()
            },
            Background::Color(self.ghost_shadow_color),
        );

        // Ghost body
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border::default().rounded(4.0),
                ..Quad::default()
            },
            self.ghost_background,
        );

        // Ghost label — follow the same positioning as normal tabs:
        // left-aligned with horizontal padding, vertically centered.
        let label_pad = TAB_PADDING;
        renderer.fill_text(
            text::Text {
                content: self.label.clone(),
                bounds: Size::new(bounds.width - label_pad * 2.0, bounds.height),
                size: LABEL_SIZE,
                line_height: text::LineHeight::Relative(1.0),
                font: default_font,
                align_x: text::Alignment::Left,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: None,
            },
            Point::new(bounds.x + label_pad, bounds.center_y()),
            self.ghost_text_color,
            bounds,
        );
    }
}

// ── From impl ─────────────────────────────────────────────────────────────────

impl<'a, Message, Theme, Renderer> From<TabBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: style::Catalog + 'a,
    Renderer: text::Renderer<Font = iced::Font> + 'a,
{
    fn from(tab_bar: TabBar<'a, Message, Theme, Renderer>) -> Self {
        Element::new(tab_bar)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tab(id: usize, label: &str, modified: bool, pinned: bool) -> Tab {
        Tab::new(id, label.to_string())
            .modified(modified)
            .pinned(pinned)
    }

    #[test]
    fn test_tab_data_creation() {
        let tab = make_tab(0, "Test", true, false);
        assert_eq!(tab.id, 0);
        assert_eq!(tab.label, "Test");
        assert!(tab.modified);
        assert!(!tab.pinned);
    }

    #[test]
    fn test_tab_content_width() {
        let tab = make_tab(0, "Hello", false, false);
        let w = tab.content_width();
        assert!(w > 0.0);
    }

    #[test]
    fn test_tab_content_width_pinned() {
        let tab = make_tab(1, "Pinned", false, true);
        let w = tab.content_width();
        assert!(w > 0.0);
    }

    #[test]
    fn test_tab_content_width_modified() {
        let tab_mod = make_tab(2, "Mod", true, false);
        let tab_norm = make_tab(3, "Mod", false, false);
        assert!(tab_mod.content_width() > tab_norm.content_width());
    }

    #[test]
    fn test_state_default_is_idle() {
        let state = TabBarState::default();
        assert!(matches!(state.action, Action::Idle));
        assert!(state.hovered_tab.is_none());
        assert!(!state.hovered_close);
        assert!(state.drop_target.is_none());
    }

    #[test]
    fn test_tab_bar_event_types() {
        let e1 = TabBarEvent::Selected(0);
        let e2 = TabBarEvent::Closed(1);
        let e3 = TabBarEvent::Dragged(2, 3);
        let e4 = TabBarEvent::DragCanceled(4);
        let e5 = TabBarEvent::RightClicked(5);

        match e1 {
            TabBarEvent::Selected(i) => assert_eq!(i, 0),
            _ => panic!("wrong variant"),
        }
        match e2 {
            TabBarEvent::Closed(i) => assert_eq!(i, 1),
            _ => panic!("wrong variant"),
        }
        match e3 {
            TabBarEvent::Dragged(from, to) => {
                assert_eq!(from, 2);
                assert_eq!(to, 3);
            }
            _ => panic!("wrong variant"),
        }
        match e4 {
            TabBarEvent::DragCanceled(i) => assert_eq!(i, 4),
            _ => panic!("wrong variant"),
        }
        match e5 {
            TabBarEvent::RightClicked(i) => assert_eq!(i, 5),
            _ => panic!("wrong variant"),
        }
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
    fn test_style_status_eq() {
        assert_eq!(Status::Active, Status::Active);
        assert_ne!(Status::Active, Status::Idle);
    }
}
