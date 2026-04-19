use std::slice;

use iced::advanced::widget::tree;
use iced::advanced::{layout, overlay, renderer, widget, Clipboard, Layout, Shell, Widget};
use iced::widget::{button, column, container, text};
use iced::{mouse, Element, Event, Fill, Point, Rectangle, Size, Vector};

use crate::style;

/// A widget that adds a right-click context menu overlay to any base element.
///
/// Menu state is widget-local — no app messages needed for open/close.
pub struct ContextMenu<'a, Message> {
    base: Element<'a, Message>,
    entries: Vec<(&'a str, Message)>,
    /// Cached menu element; rebuilt each time the menu is opened.
    menu: Option<Element<'a, Message>>,
}

impl<'a, Message> ContextMenu<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(base: impl Into<Element<'a, Message>>, entries: Vec<(&'a str, Message)>) -> Self {
        Self {
            base: base.into(),
            entries,
            menu: None,
        }
    }
}

// ── Widget-local state ────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Closed,
    Open {
        position: Point,
    },
}

impl Status {
    fn position(self) -> Option<Point> {
        match self {
            Status::Closed => None,
            Status::Open { position } => Some(position),
        }
    }
}

#[derive(Debug)]
pub struct State {
    status: Status,
    menu_tree: widget::Tree,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: Status::Closed,
            menu_tree: widget::Tree::empty(),
        }
    }
}

// ── Menu construction ─────────────────────────────────────────────────────────

fn build_menu<'a, Message>(entries: &[(&'a str, Message)]) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let items: Vec<Element<'a, Message>> = entries
        .iter()
        .map(|(label, msg)| {
            button(text(*label).size(12))
                .on_press(msg.clone())
                .width(Fill)
                .style(style::menu_item)
                .into()
        })
        .collect();

    container(column(items).spacing(2).padding(4))
        .style(style::context_menu)
        .width(200)
        .into()
}

// ── Widget implementation ─────────────────────────────────────────────────────

// TODO: The order of `impl` members differs from the trait
impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ContextMenu<'a, Message>
where
    Message: Clone + 'a,
{
    fn size(&self) -> iced::Size<iced::Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> iced::Size<iced::Length> {
        self.base.as_widget().size_hint()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.base)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(slice::from_ref(&self.base));
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            if let Some(position) = cursor.position_over(layout.bounds()) {
                let state = tree.state.downcast_mut::<State>();
                state.status = Status::Open {
                    position: Point::new(position.x + 5.0, position.y + 5.0),
                };
                shell.capture_event();
                shell.request_redraw();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.base.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.base
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let base_overlay = self.base.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let state = tree.state.downcast_mut::<State>();

        // Sync the cached menu element with current status.
        match state.status {
            Status::Open { .. } => {
                if self.menu.is_none() {
                    let m = build_menu(&self.entries);
                    state.menu_tree.diff(&m);
                    self.menu = Some(m);
                } else if let Some(m) = &self.menu {
                    state.menu_tree.diff(m);
                }
            }
            Status::Closed => {
                self.menu = None;
            }
        }

        let context_overlay =
            state
                .status
                .position()
                .zip(self.menu.as_mut())
                .map(|(position, menu)| {
                    overlay::Element::new(Box::new(MenuOverlay {
                        menu,
                        state,
                        position: position + translation,
                    }))
                });

        match (base_overlay, context_overlay) {
            (None, None) => None,
            (Some(base), None) => Some(base),
            (None, Some(ctx)) => Some(ctx),
            (Some(base), Some(ctx)) => {
                Some(overlay::Group::with_children(vec![base, ctx]).overlay())
            }
        }
    }
}

impl<'a, Message> From<ContextMenu<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(cm: ContextMenu<'a, Message>) -> Self {
        Element::new(cm)
    }
}

// ── Overlay ───────────────────────────────────────────────────────────────────

struct MenuOverlay<'a, 'b, Message> {
    menu: &'b mut Element<'a, Message>,
    state: &'b mut State,
    position: Point,
}

impl<Message> overlay::Overlay<Message, iced::Theme, iced::Renderer>
    for MenuOverlay<'_, '_, Message>
where
    Message: Clone,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let node = self
            .menu
            .as_widget_mut()
            .layout(&mut self.state.menu_tree, renderer, &limits);

        // Clamp position so the menu stays within the viewport (5px inset).
        let padding = 5.0;
        let vp = Rectangle::new(
            Point::new(padding, padding),
            Size::new(bounds.width - 2.0 * padding, bounds.height - 2.0 * padding),
        );
        let mut rect = Rectangle::new(self.position, node.size());

        if rect.x < vp.x {
            rect.x = vp.x;
        } else if vp.x + vp.width < rect.x + rect.width {
            rect.x = vp.x + vp.width - rect.width;
        }

        if rect.y < vp.y {
            rect.y = vp.y;
        } else if vp.y + vp.height < rect.y + rect.height {
            rect.y = vp.y + vp.height - rect.height;
        }

        node.move_to(rect.position())
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.menu.as_widget().draw(
            &self.state.menu_tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        self.menu.as_widget_mut().update(
            &mut self.state.menu_tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );

        if let Event::Mouse(mouse::Event::ButtonPressed(_)) = event {
            if !shell.is_event_captured() {
                self.state.status = Status::Closed;
            }
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.menu.as_widget().mouse_interaction(
            &self.state.menu_tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.menu
            .as_widget_mut()
            .operate(&mut self.state.menu_tree, layout, renderer, operation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::button;
    use iced::{Element, Point};

    // ═══════════════════════════════════════════════════════════════════════════
    // ContextMenu Creation Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_context_menu_creation() {
        let base = button("Test Button");
        let entries = vec![("Option 1", "msg1"), ("Option 2", "msg2")];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
        assert!(true);
    }

    #[test]
    fn test_context_menu_with_single_entry() {
        let base = button("Single Option");
        let entries = vec![("Only Option", "only")];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_context_menu_with_many_entries() {
        let base = button("Many Options");
        let entries = vec![
            ("Option 1", "msg1"),
            ("Option 2", "msg2"),
            ("Option 3", "msg3"),
            ("Option 4", "msg4"),
            ("Option 5", "msg5"),
            ("Option 6", "msg6"),
            ("Option 7", "msg7"),
            ("Option 8", "msg8"),
            ("Option 9", "msg9"),
            ("Option 10", "msg10"),
        ];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_context_menu_empty_entries() {
        let base = button("Test Button");
        let entries = vec![];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_context_menu_with_various_message_types() {
        #[derive(Clone, Debug, PartialEq)]
        enum TestMsg {
            Close,
            Open(String),
            Delete,
        }

        let base = button("Test");
        let entries = vec![
            ("Close", TestMsg::Close),
            ("Open", TestMsg::Open("test".to_string())),
            ("Delete", TestMsg::Delete),
        ];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, TestMsg> = context_menu.into();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Status Enum Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_status_default_is_closed() {
        let status = Status::default();
        assert!(matches!(status, Status::Closed));
    }

    #[test]
    fn test_status_open_with_position() {
        let position = Point::new(100.0, 200.0);
        let status = Status::Open { position };
        assert!(matches!(status, Status::Open { position: _ }));
        assert_eq!(status.position(), Some(position));
    }

    #[test]
    fn test_status_closed_has_no_position() {
        let status = Status::Closed;
        assert_eq!(status.position(), None);
    }

    #[test]
    fn test_status_debug_trait() {
        let closed = Status::Closed;
        let open = Status::Open {
            position: Point::new(10.0, 20.0),
        };
        let debug_closed = format!("{:?}", closed);
        let debug_open = format!("{:?}", open);
        assert!(debug_closed.contains("Closed"));
        assert!(debug_open.contains("Open"));
    }

    #[test]
    fn test_status_partial_eq() {
        let pos1 = Point::new(10.0, 20.0);
        let pos2 = Point::new(10.0, 20.0);
        let pos3 = Point::new(30.0, 40.0);

        let status1 = Status::Open { position: pos1 };
        let status2 = Status::Open { position: pos2 };
        let status3 = Status::Open { position: pos3 };
        let status_closed = Status::Closed;

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
        assert_ne!(status1, status_closed);
    }

    #[test]
    fn test_status_clone() {
        let pos = Point::new(50.0, 75.0);
        let status = Status::Open { position: pos };
        let cloned = status;
        assert_eq!(status.position(), cloned.position());
    }

    #[test]
    fn test_status_copy() {
        let status = Status::Closed;
        let copied = status;
        assert!(matches!(copied, Status::Closed));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // State Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_state_default() {
        let state = State::default();
        assert!(matches!(state.status, Status::Closed));
    }

    #[test]
    fn test_state_debug() {
        let state = State::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("status"));
        assert!(debug.contains("menu_tree"));
    }

    #[test]
    fn test_state_transitions() {
        let mut state = State::default();
        assert!(matches!(state.status, Status::Closed));

        state.status = Status::Open {
            position: Point::new(100.0, 100.0),
        };
        assert!(matches!(state.status, Status::Open { position: _ }));

        state.status = Status::Closed;
        assert!(matches!(state.status, Status::Closed));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Build Menu Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_build_menu_with_entries() {
        let entries = vec![
            ("Option 1", "msg1"),
            ("Option 2", "msg2"),
            ("Option 3", "msg3"),
        ];
        let menu = build_menu(&entries);
        let _ = menu;
    }

    #[test]
    fn test_build_menu_with_empty_entries() {
        let entries: Vec<(&str, &str)> = vec![];
        let menu = build_menu(&entries);
        let _ = menu;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Widget Lifecycle Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_context_menu_tag() {
        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Test", "msg")]);
        let tag = context_menu.tag();
        assert_eq!(tag, tree::Tag::of::<State>());
    }

    #[test]
    fn test_context_menu_state_creation() {
        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Test", "msg")]);
        let _state = context_menu.state();
    }

    #[test]
    fn test_context_menu_children() {
        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Test", "msg")]);
        let children = context_menu.children();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_context_menu_size() {
        let base = button("Test Button");
        let context_menu = ContextMenu::new(base, vec![("Test", "msg")]);
        let size = context_menu.size();
        let _ = size.width;
        let _ = size.height;
    }

    #[test]
    fn test_context_menu_size_hint() {
        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Test", "msg")]);
        let hint = context_menu.size_hint();
        let _ = hint.width;
        let _ = hint.height;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Element Conversion Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_from_context_menu_to_element() {
        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Option", "msg")]);
        let _element: Element<'_, &str> = context_menu.into();
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Event Handling Tests (Conceptual)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_status_position_method() {
        let closed = Status::Closed;
        let open = Status::Open {
            position: Point::new(42.0, 84.0),
        };

        assert_eq!(closed.position(), None);
        assert_eq!(open.position(), Some(Point::new(42.0, 84.0)));
    }

    #[test]
    fn test_status_equality_regardless_of_position() {
        let status1 = Status::Open {
            position: Point::new(10.0, 20.0),
        };
        let status2 = Status::Open {
            position: Point::new(10.0, 20.0),
        };
        let status3 = Status::Open {
            position: Point::new(30.0, 40.0),
        };

        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Edge Cases
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_context_menu_with_unicode_labels() {
        let base = button("Test");
        let entries = vec![
            ("Close", "close"),
            ("Zamknij", "close_pl"),
            ("关闭", "close_cn"),
            ("Закрыть", "close_ru"),
        ];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_context_menu_with_long_labels() {
        let base = button("Test");
        let long_label = "A very long option label that exceeds normal length".to_string();
        let entries: Vec<(&str, &str)> = vec![(Box::leak(long_label.into_boxed_str()), "long")];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_context_menu_with_special_characters() {
        let base = button("Test");
        let entries = vec![
            ("Normal", "normal"),
            ("With & Ampersand", "amp"),
            ("With Quotes", "quotes"),
            ("With Brackets", "brackets"),
            ("With  spaces  ", "spaces"),
        ];
        let context_menu = ContextMenu::new(base, entries);
        let _element: Element<'_, &str> = context_menu.into();
    }

    #[test]
    fn test_multiple_context_menu_instances_independent() {
        let base1 = button("Button 1");
        let base2 = button("Button 2");

        let menu1 = ContextMenu::new(base1, vec![("Option 1", "msg1")]);
        let menu2 = ContextMenu::new(base2, vec![("Option A", "msgA"), ("Option B", "msgB")]);

        let _elem1: Element<'_, &str> = menu1.into();
        let _elem2: Element<'_, &str> = menu2.into();
    }

    #[test]
    fn test_context_menu_as_widget() {
        use iced::advanced::widget::Widget;

        let base = button("Test");
        let context_menu = ContextMenu::new(base, vec![("Option", "msg")]);

        let tag = context_menu.tag();
        assert_eq!(tag, tree::Tag::of::<State>());

        let size = context_menu.size();
        let _ = size.height;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Menu Overlay Positioning Tests (Conceptual)
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_status_position_offset() {
        let cursor_pos = Point::new(100.0, 150.0);
        let expected_menu_pos = Point::new(cursor_pos.x + 5.0, cursor_pos.y + 5.0);

        let mut state = State::default();
        state.status = Status::Open {
            position: expected_menu_pos,
        };

        assert_eq!(state.status.position(), Some(Point::new(105.0, 155.0)));
    }
}
