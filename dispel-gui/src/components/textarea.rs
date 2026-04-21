use iced::advanced::text::Wrapping;
use iced::widget::text_editor;
use iced::{color, Background, Border, Color, Element};

/// Owned `text_editor::Content` that can be stored in app state.
///
/// `text_editor::Content` doesn't implement `Clone`/`Debug` directly because
/// it wraps a renderer-backed buffer. This newtype provides those impls so
/// it can live inside `#[derive(Clone, Debug)]` state structs. Cloning
/// recreates the content from its text (cursor position is lost, which is
/// acceptable for undo/redo or row-switch resets).
#[derive(Debug)]
pub struct TextAreaContent(pub text_editor::Content);

impl TextAreaContent {
    pub fn with_text(s: &str) -> Self {
        TextAreaContent(text_editor::Content::with_text(s))
    }
}

impl Clone for TextAreaContent {
    fn clone(&self) -> Self {
        TextAreaContent(text_editor::Content::with_text(&self.0.text()))
    }
}

/// Styled multiline text area using the project's leather/medieval theme.
///
/// Matches the color palette of other input fields in the project.
pub fn textarea<'a, Message>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a,
{
    text_editor::TextEditor::new(content)
        .on_action(on_action)
        .padding([8, 10])
        .size(12)
        .min_height(80.0)
        .wrapping(Wrapping::WordOrGlyph)
        .style(textarea_style)
        .into()
}

pub fn textarea_style(_theme: &iced::Theme, status: text_editor::Status) -> text_editor::Style {
    let border_color = match status {
        text_editor::Status::Focused { .. } => color!(0xdaa520),
        text_editor::Status::Hovered => color!(0x8d6e63),
        _ => color!(0x5d4037),
    };
    text_editor::Style {
        background: Background::Color(color!(0x1a1510)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        placeholder: color!(0x666666),
        value: color!(0xeae0c8),
        selection: Color {
            a: 0.3,
            ..color!(0xdaa520)
        },
    }
}
