use iced::advanced::text::Wrapping;
use iced::widget::text_editor;
use iced::{color, Background, Border, Color, Element};

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

fn textarea_style(_theme: &iced::Theme, status: text_editor::Status) -> text_editor::Style {
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
