use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::labeled_input;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.monster_editor;
        let header = row![
            text("Monster Editor").size(20),
            text(format!(
                " - {} monsters",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![button(text("Load Catalog").size(13))
            .padding([8, 16])
            .on_press(Message::MonsterOpLoadCatalog)
            .style(style::chip),]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_monsters
            .iter()
            .map(|(i, m)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(12).width(40),
                        text(&m.name).size(12),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::MonsterOpSelectMonster(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Monster").size(16),
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
            .spacing(8)
            .into()
        } else {
            text("Select a monster to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::MonsterOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("Monsters").size(14),
                        text(format!("{} loaded", editor.filtered_monsters.len())).size(12),
                    ]
                    .padding(10)
                    .align_y(iced::Alignment::Center)
                )
                .style(style::grid_header_cell),
                scrollable(column(item_list).spacing(2)).height(Fill),
            ]
            .width(Length::Fill),
            detail_panel,
        ]
        .height(Fill);

        column![header, controls, status, main_content, save_btn]
            .spacing(12)
            .padding(16)
            .into()
    }
}
