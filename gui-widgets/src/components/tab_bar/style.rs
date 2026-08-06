use iced::{Background, Color};

/// Visual status of a single tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The tab is the active (selected) tab.
    Active,
    /// The tab is currently hovered by the cursor.
    Hovered,
    /// This tab is being dragged (the drag source).
    DragSource,
    /// This tab is a valid drop target during a drag operation.
    DropTarget,
    /// Default — not active, hovered, or involved in a drag.
    Idle,
}

/// Visual style for a tab in a given [`Status`].
#[derive(Debug, Clone, Copy)]
pub struct Style {
    // ── Tab appearance ────────────────────────────────────────────
    pub background: Background,
    pub text_color: Color,
    pub close_button_color: Color,
    pub close_button_hovered_color: Color,
    /// Background circle color behind the close X when hovered (transparent when idle).
    pub close_button_background: Background,
    pub drag_handle_color: Color,
    pub pin_color: Color,
    pub modified_dot_color: Color,
    pub border_radius: f32,
    pub border_color: Color,
    pub border_width: f32,

    // ── Tab bar background ────────────────────────────────────────
    pub bar_background: Background,

    // ── Scroll buttons (overflow) ─────────────────────────────────
    pub scroll_button_color: Color,
    pub scroll_button_hovered_color: Color,

    // ── Separator lines between tabs ──────────────────────────────
    pub separator_color: Color,
    pub separator_width: f32,

    // ── Drop indicator (insertion marker line) ────────────────────
    pub drop_indicator_color: Color,
    pub drop_indicator_width: f32,

    // ── Ghost (dragged-tab overlay) ───────────────────────────────
    pub ghost_background: Background,
    pub ghost_text_color: Color,
    pub ghost_shadow_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::from_rgb(0.18, 0.18, 0.18)),
            text_color: Color::from_rgb(0.75, 0.75, 0.75),
            close_button_color: Color::from_rgb(0.5, 0.5, 0.5),
            close_button_hovered_color: Color::from_rgb(1.0, 0.3, 0.3),
            close_button_background: Background::Color(Color::from_rgba(0.8, 0.2, 0.2, 0.35)),
            drag_handle_color: Color::from_rgb(0.4, 0.4, 0.4),
            pin_color: Color::from_rgb(0.7, 0.5, 0.2),
            modified_dot_color: Color::from_rgb(1.0, 0.8, 0.0),
            border_radius: 4.0,
            border_color: Color::from_rgb(0.3, 0.3, 0.3),
            border_width: 1.0,

            bar_background: Background::Color(Color::from_rgb(0.08, 0.08, 0.08)),

            scroll_button_color: Color::from_rgb(0.5, 0.5, 0.5),
            scroll_button_hovered_color: Color::from_rgb(0.9, 0.9, 0.9),

            separator_color: Color::from_rgba(0.3, 0.3, 0.3, 0.4),
            separator_width: 1.0,

            drop_indicator_color: Color::from_rgb(0.8, 0.7, 0.2),
            drop_indicator_width: 2.0,

            ghost_background: Background::Color(Color::from_rgba(0.3, 0.25, 0.15, 0.85)),
            ghost_text_color: Color::from_rgba(1.0, 0.9, 0.7, 0.9),
            ghost_shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
        }
    }
}

impl Style {
    /// Active tab: brighter background, white text, gold border.
    pub fn active() -> Self {
        Self {
            background: Background::Color(Color::from_rgb(0.25, 0.22, 0.18)),
            text_color: Color::from_rgb(1.0, 0.9, 0.7),
            border_color: Color::from_rgb(0.6, 0.5, 0.2),
            ..Self::default()
        }
    }

    /// Hovered tab: noticeably lighter background, white text, brighter border.
    pub fn hovered() -> Self {
        Self {
            background: Background::Color(Color::from_rgb(0.32, 0.32, 0.32)),
            text_color: Color::from_rgb(1.0, 1.0, 1.0),
            border_color: Color::from_rgb(0.55, 0.55, 0.55),
            ..Self::default()
        }
    }

    /// Drag source: elevated look, gold border, brighter.
    pub fn drag_source() -> Self {
        Self {
            background: Background::Color(Color::from_rgb(0.3, 0.25, 0.15)),
            text_color: Color::from_rgb(1.0, 0.9, 0.7),
            border_color: Color::from_rgb(0.7, 0.6, 0.2),
            border_width: 2.0,
            ..Self::default()
        }
    }

    /// Drop target: highlight with a brass/gold inset border.
    pub fn drop_target() -> Self {
        Self {
            border_color: Color::from_rgb(0.7, 0.6, 0.2),
            border_width: 2.0,
            ..Self::default()
        }
    }
}

/// A style-category trait so consuming code can supply its own styling.
///
/// This follows the same pattern as Iced's built-in widgets (e.g.
/// [`button::Catalog`](iced::widget::button::Catalog)).
pub trait Catalog {
    /// The type of style-class identifier used by this theme.
    type Class<'a>;

    /// The default style class.
    fn default<'a>() -> Self::Class<'a>;

    /// Resolve a style class into a [`Style`] for the given [`Status`].
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// Default [`Catalog`] implementation for `iced::Theme`.
///
/// Uses a boxed closure as the class type. The default returns
/// [`Style::default()`] for all statuses.
impl Catalog for iced::Theme {
    type Class<'a> = Box<dyn Fn(&iced::Theme, Status) -> Style + 'a>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme, status| match status {
            Status::Active => Style::active(),
            Status::Hovered => Style::hovered(),
            Status::DragSource => Style::drag_source(),
            Status::DropTarget => Style::drop_target(),
            Status::Idle => Style::default(),
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}
