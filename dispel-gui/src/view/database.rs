use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::types::DbOp;
use crate::utils::vertical_space;
use iced::widget::{button, column, row, text};
use iced::Element;

impl App {
    pub fn view_database_tab(&self) -> Element<'_, Message> {
        let op_buttons: Vec<Element<Message>> = DbOp::ALL
            .iter()
            .map(|op| {
                let is_active = self.state.db_op == Some(*op);
                let btn = button(text(op.label()).size(12))
                    .padding([6, 14])
                    .on_press(Message::DbOpSelected(*op));
                if is_active {
                    btn.style(style::active_chip)
                } else {
                    btn.style(style::chip)
                }
                .into()
            })
            .collect();
        let desc = match self.state.db_op {
            Some(DbOp::Import) => {
                "Imports everything: maps → refs → rest → dialog-texts → databases"
            }
            Some(DbOp::DialogTexts) => "Imports .dlg and .pgp dialogue files",
            Some(DbOp::Maps) => "Imports .map files + AllMap.ini + Map.ini",
            Some(DbOp::Databases) => {
                "Imports .db files (weapons, stores, monsters, items, spells, quests, messages)"
            }
            Some(DbOp::Refs) => "Imports INI config files (Extra, Event, Monster, Npc, Wave)",
            Some(DbOp::Rest) => "Imports .ref and .pgp reference files",
            None => "Select an operation above.",
        };
        column![
            row(op_buttons).spacing(6).wrap(),
            vertical_space().height(16),
            text(desc).size(13).style(style::subtle_text),
        ]
        .spacing(4)
        .into()
    }
}
