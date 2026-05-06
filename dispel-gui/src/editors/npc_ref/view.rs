use crate::app::App;
use crate::editors::npc_ref::NpcRefEditorMessage;
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
        app.state.npc_ref_editors.get(&tab_id),
        app.state.npc_ref_spreadsheets.get(&tab_id),
    ) else {
        return container(
            text("NPC ref file not loaded")
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
        Message::npc_ref(NpcRefEditorMessage::LoadCatalog(
            editor.current_file.clone().unwrap_or_default(),
        )),
        Message::npc_ref(NpcRefEditorMessage::Save),
        |idx| Message::npc_ref(NpcRefEditorMessage::Select(idx)),
        |idx, field, val| Message::npc_ref(NpcRefEditorMessage::FieldChanged(idx, field, val)),
        |msg| Message::npc_ref(NpcRefEditorMessage::Spreadsheet(msg)),
        &app.state.lookups,
        |event| Message::npc_ref(NpcRefEditorMessage::PaneResized(event)),
        |pane| Message::npc_ref(NpcRefEditorMessage::PaneClicked(pane)),
    )
}
