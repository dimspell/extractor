use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_weapon_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.state.weapon_editor,
            Message::WeaponOpScanWeapons,
            Message::WeaponOpSave,
            Message::WeaponOpSelectWeapon,
            Message::WeaponOpFieldChanged,
            &self.state.lookups,
        )
    }
}
