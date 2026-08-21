use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Shell, renderer};
use iced::keyboard::key;
use iced::{
    Color, Element, Event, Length, Point, Rectangle, Shadow, Size, Vector, keyboard, mouse,
};

/// Display an anchored popover panel above sibling content.
///
/// The `trigger` element is laid out normally. When `open` is `true`, the
/// `panel` element floats directly below the trigger (clamped to the window,
/// flipped above if there is no room) via the overlay layer, so it renders
/// above any sibling widgets. Clicking the trigger calls `on_toggle`; clicking
/// outside the panel or pressing Escape calls `on_blur`.
///
/// Open/close state is caller-owned (mirroring how consumers drive [`crate::components::modal`]):
/// render this widget unconditionally and pass the current `open` flag.
pub fn popover<'a, Message, Theme, Renderer>(
    trigger: impl Into<Element<'a, Message, Theme, Renderer>>,
    panel: impl Into<Element<'a, Message, Theme, Renderer>>,
    open: bool,
    on_toggle: impl Fn() -> Message + 'a,
    on_blur: impl Fn() -> Message + 'a,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
    Message: 'a,
{
    Popover::new(trigger, panel, open, on_toggle, on_blur).into()
}

struct Popover<'a, Message, Theme, Renderer> {
    trigger: Element<'a, Message, Theme, Renderer>,
    panel: Element<'a, Message, Theme, Renderer>,
    open: bool,
    on_toggle: Box<dyn Fn() -> Message + 'a>,
    on_blur: Box<dyn Fn() -> Message + 'a>,
    shadow: Shadow,
}

impl<'a, Message, Theme, Renderer> Popover<'a, Message, Theme, Renderer> {
    fn new(
        trigger: impl Into<Element<'a, Message, Theme, Renderer>>,
        panel: impl Into<Element<'a, Message, Theme, Renderer>>,
        open: bool,
        on_toggle: impl Fn() -> Message + 'a,
        on_blur: impl Fn() -> Message + 'a,
    ) -> Self {
        Self {
            trigger: trigger.into(),
            panel: panel.into(),
            open,
            on_toggle: Box::new(on_toggle),
            on_blur: Box::new(on_blur),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 6.0),
                blur_radius: 16.0,
            },
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Popover<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        tree.diff_children(&mut [&mut self.trigger, &mut self.panel]);
    }

    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // Only the trigger participates in layout; the panel is rendered by
        // the overlay layer (see `overlay`).
        let trigger = self
            .trigger
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        layout::Node::with_children(trigger.size(), vec![trigger, layout::Node::new(Size::ZERO)])
    }

    fn update(
        &mut self,
        state: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let trigger_layout = children.next().unwrap();

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            // Toggle only when closed. While open, the overlay owns all
            // clicks (outside-panel press closes it), so handling the
            // trigger here too would immediately re-open after a blur.
            if !self.open && cursor.is_over(trigger_layout.bounds()) {
                shell.publish((self.on_toggle)());
                shell.capture_event();
                return;
            }
        }

        self.trigger.as_widget_mut().update(
            &mut state.children[0],
            event,
            trigger_layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        state: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let trigger_layout = layout.children().next().unwrap();

        self.trigger.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            trigger_layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let trigger_layout = children.next().unwrap();

        if !self.open {
            return self.trigger.as_widget_mut().overlay(
                &mut state.children[0],
                trigger_layout,
                renderer,
                viewport,
                translation,
            );
        }

        let bounds = trigger_layout.bounds();
        Some(advanced::overlay::Element::new(Box::new(PopoverOverlay {
            position: bounds.position() + translation,
            target_height: bounds.height,
            viewport: *viewport,
            tree: &mut state.children[1],
            panel: &mut self.panel,
            on_blur: &self.on_blur,
            shadow: self.shadow,
        })))
    }

    fn mouse_interaction(
        &self,
        state: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let trigger_layout = layout.children().next().unwrap();

        self.trigger.as_widget().mouse_interaction(
            &state.children[0],
            trigger_layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        state: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let trigger_layout = layout.children().next().unwrap();

        self.trigger.as_widget_mut().operate(
            &mut state.children[0],
            trigger_layout,
            renderer,
            operation,
        );
    }
}

/// The floating panel of an open [`Popover`], rendered above all siblings.
struct PopoverOverlay<'a, 'b, Message, Theme, Renderer> {
    position: Point,
    target_height: f32,
    viewport: Rectangle,
    tree: &'a mut widget::Tree,
    panel: &'a mut Element<'b, Message, Theme, Renderer>,
    on_blur: &'a dyn Fn() -> Message,
    shadow: Shadow,
}

impl<Message, Theme, Renderer> advanced::overlay::Overlay<Message, Theme, Renderer>
    for PopoverOverlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let space_below = bounds.height - (self.position.y + self.target_height);
        let space_above = self.position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                (bounds.width - self.position.x).max(0.0),
                if space_below > space_above {
                    space_below.max(0.0)
                } else {
                    space_above.max(0.0)
                },
            ),
        )
        .width(Length::Shrink)
        .height(Length::Shrink);

        let node = self
            .panel
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();

        // Clamp horizontally so the panel stays inside the window.
        let x = self.position.x.min((bounds.width - size.width).max(0.0));

        // Prefer below the trigger; flip above when there is no room.
        let y = if space_below >= space_above || space_below + self.target_height >= size.height {
            self.position.y + self.target_height
        } else {
            (self.position.y - size.height).max(0.0)
        };

        node.move_to(Point::new(x.max(0.0), y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            // Click anywhere outside the panel (including the trigger)
            // dismisses it.
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !cursor.is_over(bounds) =>
            {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            _ => {}
        }

        self.panel
            .as_widget_mut()
            .update(self.tree, event, layout, cursor, renderer, shell, &bounds);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();

        // Drop shadow behind the panel.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    radius: 4.0.into(),
                    ..iced::Border::default()
                },
                shadow: self.shadow,
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );

        self.panel
            .as_widget()
            .draw(self.tree, renderer, theme, style, layout, cursor, &bounds);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.panel.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &self.viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.panel
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }
}

impl<'a, Message, Theme, Renderer> From<Popover<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
    Message: 'a,
{
    fn from(p: Popover<'a, Message, Theme, Renderer>) -> Self {
        Element::new(p)
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::{button, container};

    use super::*;

    #[test]
    fn test_popover_creation_closed() {
        let trigger = button("Layers");
        let panel = container("Panel content");
        let result = popover(trigger, panel, false, || "toggle", || "blur");
        let _: Element<'_, &str, iced::Theme, iced::Renderer> = result;
    }

    #[test]
    fn test_popover_creation_open() {
        let trigger = button("Layers");
        let panel = container("Panel content");
        let result = popover(trigger, panel, true, || "toggle", || "blur");
        let _: Element<'_, &str, iced::Theme, iced::Renderer> = result;
    }

    #[test]
    fn test_popover_size_follows_trigger() {
        // Compile-level check: the widget's `size()` delegates to the trigger;
        // constructing with differently sized triggers must be valid.
        let result = popover("tiny", container(column_of_text()), true, || "t", || "b");
        let _: Element<'_, &str, iced::Theme, iced::Renderer> = result;
    }

    fn column_of_text() -> iced::widget::Column<'static, &'static str, iced::Theme> {
        iced::widget::column!["a", "b", "c"]
    }

    #[derive(Debug, Clone)]
    enum TestMsg {
        Toggle,
        Blur,
        ItemSelected,
    }

    #[test]
    fn test_popover_different_message_types() {
        let trigger = button("Trigger").on_press(TestMsg::Toggle);
        let panel = button("Item").on_press(TestMsg::ItemSelected);
        let result = popover(trigger, panel, true, || TestMsg::Toggle, || TestMsg::Blur);
        let _: Element<'_, TestMsg, iced::Theme, iced::Renderer> = result;
    }

    #[test]
    fn test_popover_into_element_helper() {
        let result = popover("trigger", "panel", false, || "toggle", || "blur");
        let _: Element<'_, &str, iced::Theme, iced::Renderer> = result;
    }
}
