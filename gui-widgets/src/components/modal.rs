use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{self, Shell, overlay, renderer};
use iced::alignment::Alignment;
use iced::keyboard::key;
use iced::{Color, Element, Event, Length, Rectangle, Shadow, Size, Vector, keyboard, mouse};

/// Display a `modal` overlay on top of `base`, dimming it with `backdrop_alpha`.
///
/// Clicking outside the modal or pressing Escape calls `on_blur`.
pub fn modal<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    modal: impl Into<Element<'a, Message, Theme, Renderer>>,
    on_blur: impl Fn() -> Message + 'a,
    backdrop_alpha: f32,
) -> Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
    Message: 'a,
{
    Modal::new(base, modal, on_blur, backdrop_alpha).into()
}

pub struct Modal<'a, Message, Theme, Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    modal: Element<'a, Message, Theme, Renderer>,
    on_blur: Box<dyn Fn() -> Message + 'a>,
    backdrop: Color,
    shadow: Shadow,
}

impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer> {
    fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        modal: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_blur: impl Fn() -> Message + 'a,
        backdrop_alpha: f32,
    ) -> Self {
        Self {
            base: base.into(),
            modal: modal.into(),
            on_blur: Box::new(on_blur),
            backdrop: Color {
                a: backdrop_alpha.clamp(0.0, 1.0),
                ..Color::BLACK
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 24.0,
            },
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Modal<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        tree.diff_children(&mut [&mut self.base, &mut self.modal]);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let base = self
            .base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        let size = base.size();
        let modal_limits = layout::Limits::new(Size::ZERO, size)
            .width(Length::Fill)
            .height(Length::Fill);
        let modal = self
            .modal
            .as_widget_mut()
            .layout(&mut tree.children[1], renderer, &modal_limits)
            .align(Alignment::Center, Alignment::Center, modal_limits.max());

        layout::Node::with_children(size, vec![base, modal])
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
        let base_layout = children.next().unwrap();
        let modal_layout = children.next().unwrap();

        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !cursor.is_over(modal_layout.bounds()) =>
            {
                shell.publish((self.on_blur)());
                shell.capture_event();
                return;
            }
            Event::Mouse(_) | Event::Keyboard(_) => {
                self.modal.as_widget_mut().update(
                    &mut state.children[1],
                    event,
                    modal_layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );
                return;
            }
            _ => {}
        }

        self.base.as_widget_mut().update(
            &mut state.children[0],
            event,
            base_layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
        self.modal.as_widget_mut().update(
            &mut state.children[1],
            event,
            modal_layout,
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
        let mut children = layout.children();
        let base_layout = children.next().unwrap();
        let modal_layout = children.next().unwrap();

        self.base.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            base_layout,
            mouse::Cursor::Unavailable,
            viewport,
        );

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    ..renderer::Quad::default()
                },
                self.backdrop,
            );
            renderer.fill_quad(
                renderer::Quad {
                    bounds: modal_layout.bounds(),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..iced::Border::default()
                    },
                    shadow: self.shadow,
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );
            self.modal.as_widget().draw(
                &state.children[1],
                renderer,
                theme,
                style,
                modal_layout,
                cursor,
                viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.modal.as_widget_mut().overlay(
            &mut state.children[1],
            layout.children().nth(1).unwrap(),
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
        let modal_layout = layout.children().nth(1).unwrap();

        if cursor.is_over(modal_layout.bounds()) {
            self.modal.as_widget().mouse_interaction(
                &state.children[1],
                modal_layout,
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::default()
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
        let base_layout = children.next().unwrap();
        let modal_layout = children.next().unwrap();

        self.base
            .as_widget_mut()
            .operate(&mut state.children[0], base_layout, renderer, operation);
        self.modal.as_widget_mut().operate(
            &mut state.children[1],
            modal_layout,
            renderer,
            operation,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: 'a + advanced::Renderer,
    Message: 'a,
{
    fn from(m: Modal<'a, Message, Theme, Renderer>) -> Self {
        Element::new(m)
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::text;
    use iced_test::simulator;

    use super::*;

    #[test]
    fn test_modal_builds_element() {
        let base = text("Base content");
        let modal_content = text("Modal content");
        let result = modal(base, modal_content, || "blur", 0.5);
        let _: Element<'_, &str, iced::Theme, iced::Renderer> = result;
    }

    #[test]
    fn test_modal_backdrop_alpha_is_clamped() {
        let below = Modal::<&str, iced::Theme, iced::Renderer>::new(
            text("base"),
            text("modal"),
            || "blur",
            -0.5,
        );
        assert_eq!(below.backdrop.a, 0.0);

        let above = Modal::<&str, iced::Theme, iced::Renderer>::new(
            text("base"),
            text("modal"),
            || "blur",
            1.5,
        );
        assert_eq!(above.backdrop.a, 1.0);

        let inside = Modal::<&str, iced::Theme, iced::Renderer>::new(
            text("base"),
            text("modal"),
            || "blur",
            0.25,
        );
        assert_eq!(inside.backdrop.a, 0.25);
    }

    #[test]
    fn test_modal_backdrop_click_publishes_blur() {
        // The modal content is centered in the viewport, so a press near the
        // top-left corner lands on the backdrop and must dismiss the modal.
        let base = iced::widget::container(text("Base"))
            .width(Length::Fill)
            .height(Length::Fill);
        let mut ui = simulator::<&str, iced::Theme, iced::Renderer>(modal(
            base,
            text("Modal content"),
            || "blur",
            0.5,
        ));
        ui.point_at((8.0, 8.0));
        let _ = ui.simulate([Event::Mouse(mouse::Event::ButtonPressed(
            mouse::Button::Left,
        ))]);

        let messages: Vec<&str> = ui.into_messages().collect();
        assert_eq!(messages, vec!["blur"], "backdrop press must publish blur");
    }
}
