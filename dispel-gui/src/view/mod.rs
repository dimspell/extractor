use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::types::Tab;
use crate::utils::{horizontal_rule, horizontal_space, truncate_path, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

pub mod all_map_ini_editor;
pub mod chdata_editor;
pub mod chest_editor;
pub mod database;
pub mod db_viewer;
pub mod dialog_editor;
pub mod dialogue_text_editor;
pub mod draw_item_editor;
pub mod edit_item_editor;
pub mod event_ini_editor;
pub mod event_item_editor;
pub mod event_npc_ref_editor;
pub mod extra_ini_editor;
pub mod extra_ref_editor;
pub mod heal_item_editor;
pub mod magic_editor;
pub mod map;
pub mod map_ini_editor;
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
pub mod ref_tab;
pub mod sound;
pub mod sprite;
pub mod sprite_browser;
pub mod store_editor;
pub mod wave_ini_editor;
pub mod weapon_editor;

impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let game_path_toolbar = self.view_shared_game_path_toolbar();

        let content = if self.active_tab == Tab::DbViewer {
            self.view_db_viewer()
        } else if self.active_tab == Tab::ChestEditor {
            self.view_chest_editor_tab()
        } else if self.active_tab == Tab::WeaponEditor {
            self.view_weapon_editor_tab()
        } else if self.active_tab == Tab::SpriteBrowser {
            self.view_sprite_browser_tab()
        } else if self.active_tab == Tab::HealItemEditor {
            self.view_heal_item_editor_tab()
        } else if self.active_tab == Tab::MiscItemEditor {
            self.view_misc_item_editor_tab()
        } else if self.active_tab == Tab::EditItemEditor {
            self.view_edit_item_editor_tab()
        } else if self.active_tab == Tab::EventItemEditor {
            self.view_event_item_editor_tab()
        } else if self.active_tab == Tab::MonsterEditor {
            self.view_monster_editor_tab()
        } else if self.active_tab == Tab::NpcIniEditor {
            self.view_npc_ini_editor_tab()
        } else if self.active_tab == Tab::MagicEditor {
            self.view_magic_editor_tab()
        } else if self.active_tab == Tab::StoreEditor {
            self.view_store_editor_tab()
        } else if self.active_tab == Tab::PartyRefEditor {
            self.view_party_ref_editor_tab()
        } else if self.active_tab == Tab::PartyIniEditor {
            self.view_party_ini_editor_tab()
        } else if self.active_tab == Tab::MonsterRefEditor {
            self.view_monster_ref_editor_tab()
        } else if self.active_tab == Tab::AllMapIniEditor {
            self.view_all_map_ini_editor_tab()
        } else if self.active_tab == Tab::DialogEditor {
            self.view_dialog_editor_tab()
        } else if self.active_tab == Tab::DialogueTextEditor {
            self.view_dialogue_text_editor_tab()
        } else if self.active_tab == Tab::DrawItemEditor {
            self.view_draw_item_editor_tab()
        } else if self.active_tab == Tab::EventIniEditor {
            self.view_event_ini_editor_tab()
        } else if self.active_tab == Tab::EventNpcRefEditor {
            self.view_event_npc_ref_editor_tab()
        } else if self.active_tab == Tab::ExtraIniEditor {
            self.view_extra_ini_editor_tab()
        } else if self.active_tab == Tab::ExtraRefEditor {
            self.view_extra_ref_editor_tab()
        } else if self.active_tab == Tab::MapIniEditor {
            self.view_map_ini_editor_tab()
        } else if self.active_tab == Tab::MessageScrEditor {
            self.view_message_scr_editor_tab()
        } else if self.active_tab == Tab::NpcRefEditor {
            self.view_npc_ref_editor_tab()
        } else if self.active_tab == Tab::PartyLevelDbEditor {
            self.view_party_level_db_editor_tab()
        } else if self.active_tab == Tab::QuestScrEditor {
            self.view_quest_scr_editor_tab()
        } else if self.active_tab == Tab::WaveIniEditor {
            self.view_wave_ini_editor_tab()
        } else if self.active_tab == Tab::ChDataEditor {
            self.view_chdata_editor_tab()
        } else {
            let tab_content = self.view_tab_content();
            let log_panel = self.view_log();
            column![tab_content, horizontal_rule(1), log_panel]
                .spacing(0)
                .width(Fill)
                .height(Fill)
                .into()
        };

        let main_content = column![game_path_toolbar, content].spacing(0).height(Fill);
        let layout = row![sidebar, main_content].height(Fill).width(Fill);
        container(layout)
            .width(Fill)
            .height(Fill)
            .style(style::root_container)
            .into()
    }

    fn view_shared_game_path_toolbar(&self) -> Element<'_, Message> {
        let path_display = if self.shared_game_path.is_empty() {
            "No game path set - click Browse to select"
        } else {
            &self.shared_game_path
        };

        let path_text = container(
            text(truncate_path(path_display, 80))
                .size(12)
                .font(Font::MONOSPACE),
        )
        .padding([4, 12])
        .width(Fill)
        .style(style::sql_editor_container);

        container(
            row![
                text("Game Path:")
                    .size(12)
                    .width(80)
                    .style(style::subtle_text),
                path_text,
                button(text("Browse").size(12))
                    .on_press(Message::BrowseSharedGamePath)
                    .padding([4, 12])
                    .style(style::browse_button),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([8, 16]),
        )
        .width(Fill)
        .style(style::toolbar_container)
        .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let title = text("Dispel Extractor").size(18).font(Font::MONOSPACE);
        let tabs: Vec<Element<Message>> = Tab::ALL
            .iter()
            .map(|tab| {
                let is_active = *tab == self.active_tab;
                let btn = button(text(tab.label()).size(14))
                    .width(Fill)
                    .padding([10, 16])
                    .on_press(Message::TabSelected(*tab));
                if is_active {
                    btn.style(style::active_tab_button)
                } else {
                    btn.style(style::tab_button)
                }
                .into()
            })
            .collect();
        let sidebar_content = column![
            vertical_space().height(12),
            container(title).padding([0, 16]),
            vertical_space().height(16),
            scrollable(column(tabs).spacing(2).padding([0, 8])),
            vertical_space().height(Length::Fill),
            vertical_space().height(8),
        ]
        .spacing(0)
        .width(220);
        container(sidebar_content)
            .height(Fill)
            .style(style::sidebar_container)
            .into()
    }

    fn view_tab_content(&self) -> Element<'_, Message> {
        let inner = match self.active_tab {
            Tab::AllMapIniEditor => self.view_all_map_ini_editor_tab(),
            Tab::ChDataEditor => self.view_chdata_editor_tab(),
            Tab::ChestEditor => self.view_chest_editor_tab(),
            Tab::Database => self.view_database_tab(),
            Tab::DbViewer => text("").into(),
            Tab::DialogEditor => self.view_dialog_editor_tab(),
            Tab::DialogueTextEditor => self.view_dialogue_text_editor_tab(),
            Tab::DrawItemEditor => self.view_draw_item_editor_tab(),
            Tab::EditItemEditor => self.view_edit_item_editor_tab(),
            Tab::EventIniEditor => self.view_event_ini_editor_tab(),
            Tab::EventItemEditor => self.view_event_item_editor_tab(),
            Tab::EventNpcRefEditor => self.view_event_npc_ref_editor_tab(),
            Tab::ExtraIniEditor => self.view_extra_ini_editor_tab(),
            Tab::ExtraRefEditor => self.view_extra_ref_editor_tab(),
            Tab::HealItemEditor => self.view_heal_item_editor_tab(),
            Tab::MagicEditor => self.view_magic_editor_tab(),
            Tab::Map => self.view_map_tab(),
            Tab::MapIniEditor => self.view_map_ini_editor_tab(),
            Tab::MessageScrEditor => self.view_message_scr_editor_tab(),
            Tab::MiscItemEditor => self.view_misc_item_editor_tab(),
            Tab::MonsterEditor => self.view_monster_editor_tab(),
            Tab::MonsterRefEditor => self.view_monster_ref_editor_tab(),
            Tab::NpcIniEditor => self.view_npc_ini_editor_tab(),
            Tab::NpcRefEditor => self.view_npc_ref_editor_tab(),
            Tab::PartyIniEditor => self.view_party_ini_editor_tab(),
            Tab::PartyLevelDbEditor => self.view_party_level_db_editor_tab(),
            Tab::PartyRefEditor => self.view_party_ref_editor_tab(),
            Tab::QuestScrEditor => self.view_quest_scr_editor_tab(),
            Tab::Ref => self.view_ref_tab(),
            Tab::Sound => self.view_sound_tab(),
            Tab::Sprite => self.view_sprite_tab(),
            Tab::SpriteBrowser => self.view_sprite_browser_tab(),
            Tab::StoreEditor => self.view_store_editor_tab(),
            Tab::WaveIniEditor => self.view_wave_ini_editor_tab(),
            Tab::WeaponEditor => self.view_weapon_editor_tab(),
        };
        let run_btn: Element<'_, Message> = if self.is_running {
            button(text("⏳ Running…").size(14))
                .padding([10, 28])
                .style(style::run_button_disabled)
                .into()
        } else if self.active_tab == Tab::WeaponEditor || self.active_tab == Tab::SpriteBrowser {
            text("").into()
        } else {
            button(text("▶  Run Command").size(14))
                .padding([10, 28])
                .on_press(Message::Run)
                .style(style::run_button)
                .into()
        };
        let header = text(match self.active_tab {
            Tab::Map => "Map Operations",
            Tab::Ref => "Reference Data Extraction",
            Tab::Database => "Database Import Pipeline",
            Tab::Sprite => "Sprite / Animation Extraction",
            Tab::Sound => "Audio Conversion (SNF → WAV)",
            _ => "",
        })
        .size(22);
        let subtitle = text(match self.active_tab {
            Tab::Map => "Extract tiles, render maps, and manage map assets",
            Tab::Ref => "Read game DB/INI/REF files and output as JSON",
            Tab::Database => "Populate a local SQLite database from game fixtures",
            Tab::Sprite => "Parse .SPR / .SPX files to extract frames or sequences",
            Tab::Sound => "Convert .SNF game audio to standard .WAV format",
            _ => "",
        })
        .size(13)
        .style(style::subtle_text);
        let content = column![
            header,
            subtitle,
            vertical_space().height(16),
            inner,
            vertical_space().height(16),
            row![horizontal_space(), run_btn].width(Fill)
        ]
        .spacing(4)
        .padding(24)
        .width(Fill);
        content.into()
    }

    fn view_log(&self) -> Element<'_, Message> {
        let title = row![
            text("Output Log").size(14).font(Font::MONOSPACE),
            horizontal_space(),
            button(text("Clear").size(11))
                .padding([4, 12])
                .on_press(Message::ClearLog)
                .style(style::chip)
        ]
        .align_y(iced::Alignment::Center);

        let content =
            container(text(&self.log).size(12).font(Font::MONOSPACE).width(Fill)).padding(12);

        container(column![title, content].spacing(8))
            .padding(16)
            .height(Length::FillPortion(1))
            .style(style::log_container)
            .into()
    }
}
