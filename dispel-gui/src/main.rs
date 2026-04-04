use iced::color;
use iced::Theme;

pub mod all_map_ini_editor;
pub mod app;
pub mod auto_save;
pub mod chdata_editor;
pub mod chest_editor;
pub mod command_palette;
pub mod db;
pub mod db_viewer_state;
pub mod dialog_editor;
pub mod dialogue_text_editor;
pub mod draw_item_editor;
pub mod edit_item_editor;
pub mod edit_history;
pub mod event_ini_editor;
pub mod event_item_editor;
pub mod event_npc_ref_editor;
pub mod extra_ini_editor;
pub mod extra_ref_editor;
pub mod file_tree;
pub mod generic_editor;
pub mod global_search;
pub mod heal_item_editor;
pub mod magic_editor;
pub mod map_ini_editor;
pub mod message;
pub mod message_scr_editor;
pub mod misc_item_editor;
pub mod monster_editor;
pub mod monster_ref_editor;
pub mod npc_ini_editor;
pub mod npc_ref_editor;
pub mod party_ini_editor;
pub mod party_level_db_editor;
pub mod party_ref_editor;
pub mod quest_scr_editor;
pub mod sprite_browser;
pub mod state;
pub mod store_editor;
pub mod style;
pub mod tab_bar;
pub mod types;
pub mod utils;
pub mod view;
pub mod wave_ini_editor;
pub mod weapon_editor;
pub mod workspace;

use crate::app::App;

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(|_: &App| {
            Theme::custom(
                "Medieval",
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
