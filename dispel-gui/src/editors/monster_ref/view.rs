use crate::app::App;
use crate::editors::monster_ref::MonsterRefEditorMessage;
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
        app.state.monster_ref_editors.get(&tab_id),
        app.state.monster_ref_spreadsheets.get(&tab_id),
    ) else {
        return container(
            text("Monster ref file not loaded")
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
        Message::monster_ref(MonsterRefEditorMessage::LoadCatalog(
            editor.current_file.clone().unwrap_or_default(),
        )),
        Message::monster_ref(MonsterRefEditorMessage::Save),
        |idx| Message::monster_ref(MonsterRefEditorMessage::SelectEntry(idx)),
        |idx, field, val| {
            Message::monster_ref(MonsterRefEditorMessage::FieldChanged(idx, field, val))
        },
        |msg| Message::monster_ref(MonsterRefEditorMessage::Spreadsheet(msg)),
        &app.state.lookups,
        |event| Message::monster_ref(MonsterRefEditorMessage::PaneResized(event)),
        |pane| Message::monster_ref(MonsterRefEditorMessage::PaneClicked(pane)),
    )
}
