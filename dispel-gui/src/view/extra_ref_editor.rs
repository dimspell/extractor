use crate::app::App;
use crate::message::{editor::extraref::ExtraRefEditorMessage, Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{container, text};
use iced::{Element, Fill};

impl App {
    pub fn view_extra_ref_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let (Some(editor), Some(spreadsheet)) = (
            self.state.extra_ref_editors.get(&tab_id),
            self.state.extra_ref_spreadsheets.get(&tab_id),
        ) else {
            return container(
                text("Extra ref file not loaded")
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
            Message::extra_ref(ExtraRefEditorMessage::Save),
            Message::extra_ref(ExtraRefEditorMessage::Save),
            |idx| Message::extra_ref(ExtraRefEditorMessage::Select(idx)),
            |idx, field, val| {
                Message::extra_ref(ExtraRefEditorMessage::FieldChanged(idx, field, val))
            },
            |msg| Message::extra_ref(ExtraRefEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::extra_ref(ExtraRefEditorMessage::PaneResized(event)),
            |pane| Message::extra_ref(ExtraRefEditorMessage::PaneClicked(pane)),
        )
    }
}
