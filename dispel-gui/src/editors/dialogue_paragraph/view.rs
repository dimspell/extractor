use crate::app::App;
use crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage;
use crate::message::{Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{container, text};
use iced::{Element, Fill};

pub fn view(app: &App) -> Element<'_, Message> {
    let tab_id = app
        .state
        .workspace
        .active()
        .map(|t| t.id)
        .unwrap_or(usize::MAX);

    let (Some(editor), Some(spreadsheet)) = (
        app.state.dialogue_paragraphs_editors.get(&tab_id),
        app.state.dialogue_paragraph_spreadsheets.get(&tab_id),
    ) else {
        return container(
            text("Dialogue Paragraph file not loaded")
                .size(14)
                .style(style::subtle_text),
        )
        .width(Fill)
        .height(Fill)
        .padding(16)
        .into();
    };

    let scan_msg = Message::dialogue_paragraph(DialogueParagraphEditorMessage::ScanCatalog);

    view_spreadsheet(
        &editor.editor,
        spreadsheet,
        scan_msg,
        Message::dialogue_paragraph(DialogueParagraphEditorMessage::Save),
        |idx| Message::dialogue_paragraph(DialogueParagraphEditorMessage::Select(idx)),
        |idx, field, value| {
            Message::dialogue_paragraph(DialogueParagraphEditorMessage::FieldChanged(
                idx, field, value,
            ))
        },
        |msg| Message::dialogue_paragraph(DialogueParagraphEditorMessage::Spreadsheet(msg)),
        &app.state.lookups,
        |event| Message::dialogue_paragraph(DialogueParagraphEditorMessage::PaneResized(event)),
        |pane| Message::dialogue_paragraph(DialogueParagraphEditorMessage::PaneClicked(pane)),
    )
}
