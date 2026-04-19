use crate::app::App;
use crate::message::{editor::weapon::WeaponEditorMessage, Message, MessageExt};
use crate::view::editor::view_spreadsheet;
use iced::Element;

impl App {
    pub fn view_weapon_editor_tab(&self) -> Element<'_, Message> {
        view_spreadsheet(
            &self.state.weapon_editor,
            &self.state.weapon_spreadsheet,
            Message::weapon(WeaponEditorMessage::ScanWeapons),
            Message::weapon(WeaponEditorMessage::Save),
            |idx| Message::weapon(WeaponEditorMessage::SelectWeapon(idx)),
            |idx, field, value| {
                Message::weapon(WeaponEditorMessage::FieldChanged(idx, field, value))
            },
            |msg| Message::weapon(WeaponEditorMessage::Spreadsheet(msg)),
            &self.state.lookups,
            |event| Message::weapon(WeaponEditorMessage::PaneResized(event)),
            |pane| Message::weapon(WeaponEditorMessage::PaneClicked(pane)),
        )
    }
}
