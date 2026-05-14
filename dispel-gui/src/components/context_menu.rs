use std::slice;

use iced::advanced::widget::tree;
use iced::advanced::{layout, overlay, renderer, widget, Clipboard, Layout, Shell, Widget};
use iced::widget::{button, column, container, row, text};
use iced::{mouse, Element, Event, Fill, Point, Rectangle, Size, Vector};

use crate::style;

/// A context menu entry.
#[derive(Debug, Clone)]
pub enum Entry<Message> {
    Item {
        label: String,
        icon: Option<String>,
        action: Message,
    },
    Separator,
    Disabled {
        label: String,
        icon: Option<String>,
    },
}

impl<Message: Clone> Entry<Message> {
    pub fn item<S: Into<String>>(label: S, action: Message) -> Self {
        Entry::Item {
            label: label.into(),
            icon: None,
            action,
        }
    }

    #[allow(dead_code)]
    pub fn item_with_icon<S: Into<String>, I: Into<String>>(
        label: S,
        icon: I,
        action: Message,
    ) -> Self {
        Entry::Item {
            label: label.into(),
            icon: Some(icon.into()),
            action,
        }
    }

    pub fn separator() -> Self {
        Entry::Separator
    }

    pub fn disabled<S: Into<String>>(label: S) -> Self {
        Entry::Disabled {
            label: label.into(),
            icon: None,
        }
    }

    #[allow(dead_code)]
    pub fn disabled_with_icon<S: Into<String>, I: Into<String>>(label: S, icon: I) -> Self {
        Entry::Disabled {
            label: label.into(),
            icon: Some(icon.into()),
        }
    }
}

/// A widget that adds a right-click context menu overlay to any base element.
///
/// Menu state is widget-local — no app messages needed for open/close.
pub struct ContextMenu<'a, Message> {
    base: Element<'a, Message>,
    entries: Vec<Entry<Message>>,
    offset: Point,
}

impl<'a, Message> ContextMenu<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(base: impl Into<Element<'a, Message>>, entries: Vec<Entry<Message>>) -> Self {
        Self {
            base: base.into(),
            entries,
            offset: Point::new(2.0, 2.0),
        }
    }

    /// Convenience constructor that wraps each `(String, Message)` pair as
    /// `Entry::item`.
    pub fn from_simple(
        base: impl Into<Element<'a, Message>>,
        entries: Vec<(String, Message)>,
    ) -> Self {
        let entries = entries
            .into_iter()
            .map(|(label, action)| Entry::item(label, action))
            .collect();
        Self {
            base: base.into(),
            entries,
            offset: Point::new(2.0, 2.0),
        }
    }

    /// Set the offset from the cursor position where the menu appears.
    #[allow(dead_code)]
    pub fn offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }
}

// ── Widget-local state ────────────────────────────────────────────────────────

impl Status {
    #[allow(dead_code)]
    fn position(self) -> Option<Point> {
        match self {
            Status::Closed => None,
            Status::Open { position } => Some(position),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Status {
    #[default]
    Closed,
    Open {
        position: Point,
    },
}

pub struct State {
    status: Status,
    menu_tree: widget::Tree,
    hovered_idx: Option<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: Status::Closed,
            menu_tree: widget::Tree::empty(),
            hovered_idx: None,
        }
    }
}

// ── Menu construction ─────────────────────────────────────────────────────────

fn build_menu<Message>(
    entries: &[Entry<Message>],
    hovered_idx: Option<usize>,
) -> Element<'static, Message>
where
    Message: Clone + 'static,
{
    let mut items: Vec<Element<'static, Message>> = Vec::with_capacity(entries.len());

    for (idx, entry) in entries.iter().enumerate() {
        match entry {
            Entry::Item {
                label,
                icon,
                action: _,
            } => {
                let content: Element<'static, Message> = if let Some(icon) = icon {
                    let i = icon.clone();
                    let l = label.clone();
                    row![text(i).size(14), text(l).size(13)].spacing(6).into()
                } else {
                    text(label.clone()).size(13).into()
                };
                let hovered = hovered_idx == Some(idx);
                items.push(
                    button(content)
                        .style(move |theme, _status| {
                            let s = if hovered {
                                button::Status::Hovered
                            } else {
                                button::Status::Active
                            };
                            style::menu_item(theme, s)
                        })
                        .width(Fill)
                        .padding([6, 10])
                        .into(),
                );
            }
            Entry::Separator => {
                items.push(
                    container(row![].height(1))
                        .padding([2, 4])
                        .style(style::menu_separator)
                        .into(),
                );
            }
            Entry::Disabled { label, icon } => {
                let content: Element<'static, Message> = if let Some(icon) = icon {
                    row![
                        text(icon.clone()).size(14).style(style::menu_disabled_text),
                        text(label.clone())
                            .size(13)
                            .style(style::menu_disabled_text),
                    ]
                    .spacing(6)
                    .into()
                } else {
                    text(label.clone())
                        .size(13)
                        .style(style::menu_disabled_text)
                        .into()
                };
                items.push(
                    button(content)
                        .width(Fill)
                        .padding([6, 10])
                        .style(style::menu_disabled_item)
                        .into(),
                );
            }
        }
    }

    container(column(items).spacing(1).padding(4))
        .style(style::context_menu)
        .width(220)
        .into()
}

// ── Widget implementation ─────────────────────────────────────────────────────

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ContextMenu<'a, Message>
where
    Message: Clone + 'a + 'static,
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
                    position: Point::new(position.x + self.offset.x, position.y + self.offset.y),
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

        // Destructure to borrow disjoint fields for MenuOverlay.
        let State {
            ref mut status,
            ref mut menu_tree,
            ref mut hovered_idx,
        } = state;

        let context_overlay = match *status {
            Status::Open { position } => {
                let menu = build_menu(&self.entries, *hovered_idx);
                menu_tree.diff(&menu);
                Some(overlay::Element::new(Box::new(MenuOverlay {
                    menu,
                    menu_tree,
                    status,
                    hovered_idx,
                    entries: &self.entries[..],
                    position: position + translation,
                })))
            }
            Status::Closed => {
                *hovered_idx = None;
                None
            }
        };

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
    Message: Clone + 'a + 'static,
{
    fn from(cm: ContextMenu<'a, Message>) -> Self {
        Element::new(cm)
    }
}

/// Walk the layout tree to find which menu entry the cursor is over.
///
/// The menu layout is: Container → Column → [items].
fn find_hovered_entry(layout: Layout<'_>, cursor_pos: Point, entry_count: usize) -> Option<usize> {
    // First child of the container is the column.
    let column = layout.children().next()?;
    for (idx, child) in column.children().enumerate() {
        if idx >= entry_count {
            break;
        }
        if child.bounds().contains(cursor_pos) {
            return Some(idx);
        }
    }
    None
}

// ── Overlay ───────────────────────────────────────────────────────────────────

struct MenuOverlay<'a, Message> {
    menu: Element<'static, Message>,
    menu_tree: &'a mut widget::Tree,
    status: &'a mut Status,
    hovered_idx: &'a mut Option<usize>,
    entries: &'a [Entry<Message>],
    position: Point,
}

impl<Message> overlay::Overlay<Message, iced::Theme, iced::Renderer> for MenuOverlay<'_, Message>
where
    Message: Clone + 'static,
{
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let node = self
            .menu
            .as_widget_mut()
            .layout(self.menu_tree, renderer, &limits);

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
            self.menu_tree,
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
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    let new = find_hovered_entry(layout, pos, self.entries.len());
                    if *self.hovered_idx != new {
                        *self.hovered_idx = new;
                        shell.request_redraw();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(idx) = *self.hovered_idx {
                    if let Some(Entry::Item { action, .. }) = self.entries.get(idx) {
                        shell.publish(action.clone());
                        shell.capture_event();
                    }
                }
                *self.status = Status::Closed;
                shell.request_redraw();
                return;
            }
            _ => {}
        }

        self.menu.as_widget_mut().update(
            self.menu_tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.menu.as_widget().mouse_interaction(
            self.menu_tree,
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
            .operate(self.menu_tree, layout, renderer, operation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::button;

    type TestMessage = String;

    #[test]
    fn test_entry_item() {
        let entry: Entry<TestMessage> = Entry::item("Test", "action".into());
        assert!(matches!(entry, Entry::Item { label, .. } if label == "Test"));
    }

    #[test]
    fn test_entry_separator() {
        let entry: Entry<TestMessage> = Entry::separator();
        assert!(matches!(entry, Entry::Separator));
    }

    #[test]
    fn test_entry_disabled() {
        let entry: Entry<TestMessage> = Entry::disabled("Unavailable");
        assert!(matches!(entry, Entry::Disabled { label, .. } if label == "Unavailable"));
    }

    #[test]
    fn test_status_default_closed() {
        let status = Status::default();
        assert!(matches!(status, Status::Closed));
    }

    #[test]
    fn test_status_position() {
        let status = Status::Open {
            position: Point::new(100.0, 200.0),
        };
        assert_eq!(status.position(), Some(Point::new(100.0, 200.0)));
    }

    #[test]
    fn test_context_menu_new() {
        let entries = vec![Entry::item("Option", "msg".into())];
        let cm = ContextMenu::new(button("Test"), entries);
        let _: Element<'static, TestMessage> = cm.into();
    }

    #[test]
    fn test_context_menu_from_simple() {
        let entries = vec![("Option".to_string(), "msg".into())];
        let cm = ContextMenu::from_simple(button("Test"), entries);
        let _: Element<'static, TestMessage> = cm.into();
    }

    #[test]
    fn test_context_menu_with_separator() {
        let entries = vec![
            Entry::item("Copy", "copy".into()),
            Entry::separator(),
            Entry::item("Paste", "paste".into()),
        ];
        let cm = ContextMenu::new(button("Test"), entries);
        let _: Element<'static, TestMessage> = cm.into();
    }

    #[test]
    fn test_context_menu_with_disabled() {
        let entries = vec![
            Entry::item("Enabled", "enabled".into()),
            Entry::separator(),
            Entry::disabled("Not available"),
        ];
        let cm = ContextMenu::new(button("Test"), entries);
        let _: Element<'static, TestMessage> = cm.into();
    }

    #[test]
    fn test_context_menu_empty_entries() {
        let entries: Vec<Entry<TestMessage>> = vec![];
        let cm = ContextMenu::new(button("Test"), entries);
        let _: Element<'static, TestMessage> = cm.into();
    }

    #[test]
    fn test_context_menu_tag() {
        let entries: Vec<Entry<TestMessage>> = vec![];
        let cm = ContextMenu::new(button("Test"), entries);
        assert_eq!(cm.tag(), tree::Tag::of::<State>());
    }
}
