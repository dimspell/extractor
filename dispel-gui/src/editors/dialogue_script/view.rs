use crate::app::App;
use crate::message::{editor::dialogue_script::DialogueScriptEditorMessage, Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{container, text};
use iced::{Element, Fill};

impl App {
    pub fn view_dialogue_script_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let (Some(editor), Some(spreadsheet)) = (
            self.state.dialogue_script_editors.get(&tab_id),
            self.state.dialogue_script_spreadsheets.get(&tab_id),
        ) else {
            return container(
                text("DialogueScript file not loaded")
                    .size(14)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
        };

        view_spreadsheet(
            &editor.editor,
            spreadsheet,
            Message::dialogue_script(DialogueScriptEditorMessage::LoadCatalog),
            Message::dialogue_script(DialogueScriptEditorMessage::Save),
            |idx| Message::dialogue_script(DialogueScriptEditorMessage::Select(idx)),
            |idx, field, value| {
                Message::dialogue_script(DialogueScriptEditorMessage::FieldChanged(
                    idx, field, value,
                ))
            },
            |msg| Message::dialogue_script(DialogueScriptEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::dialogue_script(DialogueScriptEditorMessage::PaneResized(event)),
            |pane| Message::dialogue_script(DialogueScriptEditorMessage::PaneClicked(pane)),
        )
    }
}
