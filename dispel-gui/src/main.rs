use iced::color;
use iced::Theme;

pub mod app;
pub mod chest_editor;
pub mod db;
pub mod db_viewer_state;
pub mod edit_item_editor;
pub mod event_item_editor;
pub mod heal_item_editor;
pub mod magic_editor;
pub mod message;
pub mod misc_item_editor;
pub mod monster_editor;
pub mod monster_ref_editor;
pub mod npc_ini_editor;
pub mod party_ini_editor;
pub mod party_ref_editor;
pub mod store_editor;
pub mod style;
pub mod sprite_browser;
pub mod types;
pub mod utils;
pub mod view;
pub mod weapon_editor;

use crate::app::App;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(|_: &App| {
            Theme::custom("Dispel Medieval",
                iced::theme::Palette {
                    background: color!(0x2a2a2a),
                    text: color!(0xeae0c8),
                    primary: color!(0x8b5a2b),
                    success: color!(0x2d5a27),
                    danger: color!(0x800000),
                    warning: color!(0x8b8b00),
                },
            )
        })
        .window_size((1100.0, 800.0))
        .run()
}
