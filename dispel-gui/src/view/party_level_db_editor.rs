use crate::app::App;
use crate::message::editor::party_level_db::PartyLevelDbEditorMessage;
use crate::message::{Message, MessageExt};
use crate::style;
use crate::view::editor::view_spreadsheet;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_party_level_db_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.party_level_db_editor;

        let npc_buttons: Vec<Element<Message>> = match &editor.catalog {
            None => vec![text("No catalog loaded")
                .size(12)
                .style(style::subtle_text)
                .into()],
            Some(npcs) => npcs
                .iter()
                .enumerate()
                .map(|(idx, npc)| {
                    let label = editor.npc_label(npc.npc_index);
                    let is_selected = editor.selected_npc_idx == Some(idx);
                    let btn = button(
                        text(label)
                            .size(12)
                            .font(Font::MONOSPACE),
                    )
                    .width(Fill)
                    .on_press(Message::party_level_db(PartyLevelDbEditorMessage::SelectNpc(
                        idx,
                    )));
                    if is_selected {
                        btn.style(style::active_chip).into()
                    } else {
                        btn.style(style::chip).into()
                    }
                })
                .collect(),
        };

        let npc_nav = column![
            container(
                text("Party Members")
                    .size(13)
                    .font(Font::MONOSPACE)
            )
            .padding([8, 10])
            .width(Fill)
            .style(style::grid_header_cell),
            scrollable(column(npc_buttons).spacing(2).padding([4, 6])).height(Fill),
        ]
        .width(170);

        let spreadsheet_area: Element<Message> = if editor.selected_npc_idx.is_some() {
            view_spreadsheet(
                &self.state.party_level_db_level_editor,
                &self.state.party_level_db_spreadsheet,
                Message::party_level_db(PartyLevelDbEditorMessage::LoadCatalog),
                Message::party_level_db(PartyLevelDbEditorMessage::Save),
                |_idx| Message::party_level_db(PartyLevelDbEditorMessage::LoadCatalog),
                |idx, field, val| {
                    Message::party_level_db(PartyLevelDbEditorMessage::FieldChanged(
                        idx, field, val,
                    ))
                },
                |msg| Message::party_level_db(PartyLevelDbEditorMessage::Spreadsheet(msg)),
                &self.state.lookups,
                |msg| Message::party_level_db(PartyLevelDbEditorMessage::PaneResized(msg)),
                |pane| Message::party_level_db(PartyLevelDbEditorMessage::PaneClicked(pane)),
            )
        } else {
            container(
                text(if editor.catalog.is_some() {
                    "Select a party member to view their level stats"
                } else {
                    "Click Scan to load PrtLevel.db"
                })
                .size(13)
                .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .padding(20)
            .into()
        };

        row![npc_nav, spreadsheet_area]
            .spacing(0)
            .height(Fill)
            .into()
    }
}
