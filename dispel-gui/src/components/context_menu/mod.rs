pub mod platform;

mod entry;
mod popup;
mod state;

#[cfg(test)]
mod tests;

use std::slice;

use iced::advanced::widget::tree;
use iced::advanced::{layout, renderer, widget, Clipboard, Layout, Shell, Widget};
use iced::advanced::overlay as iced_overlay;
use iced::{mouse, Element, Event, Point, Rectangle, Vector};

pub use entry::Entry;
pub use state::{State, Status};

use popup::MenuOverlay;

/// The offset from the cursor where the custom overlay menu appears.
const MENU_OFFSET: Point = Point::new(2.0, 2.0);

/// A widget that adds a right-click context menu overlay to any base element.
///
/// Menu state is widget-local — no app messages needed for open/close.
///
/// On macOS and Windows a native OS menu is shown. On all other platforms the
/// Iced-rendered overlay is used (set `DISPEL_FORCE_CUSTOM_CONTEXT_MENU=1` to
/// force the overlay on macOS/Windows for debugging).
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
            offset: MENU_OFFSET,
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
            offset: MENU_OFFSET,
        }
    }

    /// Set the offset from the cursor position where the menu appears.
    #[allow(dead_code)]
    pub fn offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }
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

                match platform::try_show_native_menu(&self.entries) {
                    Some(platform::NativeResult::Selected(idx)) => {
                        if let Some(Entry::Item { action, .. }) = self.entries.get(idx) {
                            shell.publish(action.clone());
                        }
                        shell.capture_event();
                        return;
                    }
                    Some(platform::NativeResult::Cancelled) => {
                        shell.capture_event();
                        return;
                    }
                    None => {}
                }

                state.status = Status::Open {
                    position: Point::new(
                        position.x + self.offset.x,
                        position.y + self.offset.y,
                    ),
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
    ) -> Option<iced_overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let base_overlay = self.base.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let state = tree.state.downcast_mut::<State>();

        let context_overlay = match state.status {
            Status::Open { position } => {
                let menu = popup::build_menu(&self.entries, state.hovered_idx);
                state.menu_tree.diff(&menu);
                Some(iced_overlay::Element::new(Box::new(MenuOverlay {
                    menu,
                    menu_tree: &mut state.menu_tree,
                    status: &mut state.status,
                    hovered_idx: &mut state.hovered_idx,
                    entries: &self.entries[..],
                    position: position + translation,
                })))
            }
            Status::Closed => {
                state.hovered_idx = None;
                None
            }
        };

        match (base_overlay, context_overlay) {
            (None, None) => None,
            (Some(base), None) => Some(base),
            (None, Some(ctx)) => Some(ctx),
            (Some(base), Some(ctx)) => {
                Some(iced_overlay::Group::with_children(vec![base, ctx]).overlay())
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
