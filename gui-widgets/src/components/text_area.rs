use iced::advanced::text::Wrapping;
use iced::widget::text_editor;
use iced::{Background, Border, Color, Element, Length, color};

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

/// Styled multiline text area that grows with its wrapped content.
///
/// This is best for fields inside a parent scrollable, such as an inspector.
/// Use [`textarea_fill`] when the editor should take its container's remaining
/// vertical space instead.
pub fn textarea<'a, Message>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a,
{
    textarea_with_height(content, on_action, Length::Fit)
}

/// Styled multiline text area that fills its container's remaining height.
///
/// Use this for a primary editor pane whose surrounding layout already
/// constrains its height.
pub fn textarea_fill<'a, Message>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message>
where
    Message: 'a,
{
    textarea_with_height(content, on_action, Length::Fill)
}

fn textarea_with_height<'a, Message>(
    content: &'a text_editor::Content,
    on_action: impl Fn(text_editor::Action) -> Message + 'a,
    height: Length,
) -> Element<'a, Message>
where
    Message: 'a,
{
    text_editor::TextEditor::new(content)
        .on_action(on_action)
        .padding([8, 10])
        .size(12)
        .height(height)
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
