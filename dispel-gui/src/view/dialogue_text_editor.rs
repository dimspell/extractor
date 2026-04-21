use crate::app::App;
use crate::message::{editor::dialoguetext::DialogueTextEditorMessage, Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{container, text};
use iced::{Element, Fill};

impl App {
    pub fn view_dialogue_text_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let (Some(editor), Some(spreadsheet)) = (
            self.state.dialogue_text_editors.get(&tab_id),
            self.state.dialogue_text_spreadsheets.get(&tab_id),
        ) else {
            return container(
                text("Dialogue text file not loaded")
                    .size(14)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
        };

        let scan_msg = Message::dialogue_text(DialogueTextEditorMessage::ScanDialogueTexts);

        view_spreadsheet(
            &editor.editor,
            spreadsheet,
            scan_msg,
            Message::dialogue_text(DialogueTextEditorMessage::Save),
            |idx| Message::dialogue_text(DialogueTextEditorMessage::SelectText(idx)),
            |idx, field, value| {
                Message::dialogue_text(DialogueTextEditorMessage::FieldChanged(idx, field, value))
            },
            |msg| Message::dialogue_text(DialogueTextEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::dialogue_text(DialogueTextEditorMessage::PaneResized(event)),
            |pane| Message::dialogue_text(DialogueTextEditorMessage::PaneClicked(pane)),
        )
    }
}
