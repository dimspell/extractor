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
/// `panel` element floats directly below the trigger (clamped to the available
/// bounds). Clicking the trigger calls `on_toggle`; clicking outside the panel
/// or pressing Escape calls `on_blur`.
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
        let trigger = self
            .trigger
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = trigger.size();

        // Lay out the panel; when open, anchor it below the trigger
        // (left-aligned, clamped to the available bounds).
        let panel_limits = layout::Limits::new(Size::ZERO, limits.max())
            .width(Length::Shrink)
            .height(Length::Shrink);
        let panel = if self.open {
            let mut panel =
                self.panel
                    .as_widget_mut()
                    .layout(&mut tree.children[1], renderer, &panel_limits);
            let panel_size = panel.size();
            let max = limits.max();
            let x = trigger
                .bounds()
                .x
                .min((max.width - panel_size.width).max(0.0));
            panel.move_to_mut(Point::new(x.max(0.0), size.height));
            panel
        } else {
            // Keep the child slot present for state consistency, but empty.
            layout::Node::new(Size::ZERO)
        };

        layout::Node::with_children(size, vec![trigger, panel])
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
        let panel_layout = children.next().unwrap();

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) if self.open => {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(trigger_layout.bounds()) {
                    shell.publish((self.on_toggle)());
                    shell.capture_event();
                    return;
                }
                if self.open && !cursor.is_over(panel_layout.bounds()) {
                    shell.publish((self.on_blur)());
                    shell.capture_event();
                    return;
                }
            }
            _ => {}
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
        if self.open {
            self.panel.as_widget_mut().update(
                &mut state.children[1],
                event,
                panel_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );
        }
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
        let mut children = layout.children();
        let trigger_layout = children.next().unwrap();
        let panel_layout = children.next().unwrap();

        self.trigger.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            trigger_layout,
            cursor,
            viewport,
        );

        if self.open {
            renderer.with_layer(layout.bounds(), |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: panel_layout.bounds(),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..iced::Border::default()
                        },
                        shadow: self.shadow,
                        ..renderer::Quad::default()
                    },
                    Color::TRANSPARENT,
                );
                self.panel.as_widget().draw(
                    &state.children[1],
                    renderer,
                    theme,
                    style,
                    panel_layout,
                    cursor,
                    viewport,
                );
            });
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        self.trigger.as_widget_mut().overlay(
            &mut state.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        state: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let trigger_layout = children.next().unwrap();
        let panel_layout = children.next().unwrap();

        if self.open && cursor.is_over(panel_layout.bounds()) {
            self.panel.as_widget().mouse_interaction(
                &state.children[1],
                panel_layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            self.trigger.as_widget().mouse_interaction(
                &state.children[0],
                trigger_layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn operate(
        &mut self,
        state: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let mut children = layout.children();
        let trigger_layout = children.next().unwrap();
        let panel_layout = children.next().unwrap();

        self.trigger.as_widget_mut().operate(
            &mut state.children[0],
            trigger_layout,
            renderer,
            operation,
        );
        if self.open {
            self.panel.as_widget_mut().operate(
                &mut state.children[1],
                panel_layout,
                renderer,
                operation,
            );
        }
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
