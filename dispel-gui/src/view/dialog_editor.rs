use crate::app::App;
use crate::message::{editor::dialog::DialogEditorMessage, Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{container, text};
use iced::{Element, Fill};

impl App {
    pub fn view_dialog_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let (Some(editor), Some(spreadsheet)) = (
            self.state.dialog_editors.get(&tab_id),
            self.state.dialog_spreadsheets.get(&tab_id),
        ) else {
            return container(
                text("Dialog file not loaded")
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
            Message::dialog(DialogEditorMessage::LoadCatalog),
            Message::dialog(DialogEditorMessage::Save),
            |idx| Message::dialog(DialogEditorMessage::Select(idx)),
            |idx, field, value| {
                Message::dialog(DialogEditorMessage::FieldChanged(idx, field, value))
            },
            |msg| Message::dialog(DialogEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::dialog(DialogEditorMessage::PaneResized(event)),
            |pane| Message::dialog(DialogEditorMessage::PaneClicked(pane)),
        )
    }
}
