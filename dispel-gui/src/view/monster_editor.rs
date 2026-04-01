use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input};

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.monster_editor;

        // Header
        let header = row![
            text("Monster Editor").size(20),
            text(format!(
                " - {} monsters",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // Controls
        let controls = row![
            button(text("Load Catalog").size(13))
                .padding([8, 16])
                .on_press(Message::MonsterOpLoadCatalog)
                .style(style::chip),
            horizontal_space(),
            button(text("Save to File").size(13))
                .padding([8, 16])
                .on_press(Message::MonsterOpSave)
                .style(style::run_button),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        // Monster list
        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_monsters
            .iter()
            .map(|(i, m)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(11).width(35),
                        text(&m.name).size(12).width(Length::Fill),
                        text(format!("HP:{}", m.health_points_max))
                            .size(10)
                            .style(style::subtle_text)
                            .width(60),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([8, 12])
                .on_press(Message::MonsterOpSelectMonster(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        // Editor panel
        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Monster").size(14),
                horizontal_rule(1),
                labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::MonsterOpFieldChanged(idx, "name".into(), v)
                }),
                labeled_input("HP Max:", &editor.edit_hp_max, move |v| {
                    Message::MonsterOpFieldChanged(idx, "health_points_max".into(), v)
                }),
                labeled_input("HP Min:", &editor.edit_hp_min, move |v| {
                    Message::MonsterOpFieldChanged(idx, "health_points_min".into(), v)
                }),
                labeled_input("MP Max:", &editor.edit_mp_max, move |v| {
                    Message::MonsterOpFieldChanged(idx, "mana_points_max".into(), v)
                }),
                labeled_input("MP Min:", &editor.edit_mp_min, move |v| {
                    Message::MonsterOpFieldChanged(idx, "mana_points_min".into(), v)
                }),
                labeled_input("AI Type:", &editor.edit_ai_type, move |v| {
                    Message::MonsterOpFieldChanged(idx, "ai_type".into(), v)
                }),
                labeled_input("Offense Max:", &editor.edit_offense_max, move |v| {
                    Message::MonsterOpFieldChanged(idx, "offense_max".into(), v)
                }),
                labeled_input("Defense Max:", &editor.edit_defense_max, move |v| {
                    Message::MonsterOpFieldChanged(idx, "defense_max".into(), v)
                }),
            ]
            .spacing(10)
            .padding(16)
            .into()
        } else {
            column![
                text("Monster Details").size(14),
                horizontal_rule(1),
                text("Select a monster to edit")
                    .size(12)
                    .style(style::subtle_text),
            ]
            .spacing(10)
            .padding(16)
            .into()
        };

        // Detail panel
        let detail_panel = container(scrollable(editor_panel).height(Length::Fill))
            .padding(0)
            .width(400)
            .style(style::info_card);

        // Main content
        let main_content = row![
            column![
                container(
                    row![
                        text("Monsters").size(13),
                        horizontal_space(),
                        text(format!("{} loaded", editor.filtered_monsters.len()))
                            .size(11)
                            .style(style::subtle_text),
                    ]
                    .padding([8, 12])
                    .align_y(iced::Alignment::Center)
                )
                .style(style::grid_header_cell),
                scrollable(column(item_list).spacing(2)).height(Fill),
            ]
            .width(Length::Fill),
            detail_panel,
        ]
        .spacing(12)
        .height(Fill);

        column![header, controls, horizontal_rule(1), status, main_content,]
            .spacing(10)
            .padding(16)
            .height(Length::Fill)
            .into()
    }
}
