use iced::advanced::{layout, overlay, renderer, widget, Clipboard, Layout, Shell};
use iced::widget::{button, column, container, row, text};
use iced::{mouse, Element, Event, Fill, Point, Rectangle, Size};

use crate::style;

use super::entry::Entry;
use super::state::Status;

/// Build the Iced element tree for the menu popup.
pub(super) fn build_menu<Message>(
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

/// Walk the layout tree to find which menu entry the cursor is over.
///
/// The menu layout is: Container → Column → [items].
fn find_hovered_entry(layout: Layout<'_>, cursor_pos: Point, entry_count: usize) -> Option<usize> {
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

pub(crate) struct MenuOverlay<'a, Message> {
    pub(crate) menu: Element<'static, Message>,
    pub(crate) menu_tree: &'a mut widget::Tree,
    pub(crate) status: &'a mut Status,
    pub(crate) hovered_idx: &'a mut Option<usize>,
    pub(crate) entries: &'a [Entry<Message>],
    pub(crate) position: Point,
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
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if let Some(pos) = cursor.position() {
            if let Some(idx) = find_hovered_entry(layout, pos, self.entries.len()) {
                if matches!(self.entries.get(idx), Some(Entry::Item { .. })) {
                    return mouse::Interaction::Pointer;
                }
            }
        }
        mouse::Interaction::Idle
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
