use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.magic_editor;
        let header = row![
            text("Magic Spell Editor").size(20),
            text(format!(
                " - {} spells",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![button(text("Load Catalog").size(13))
            .padding([8, 16])
            .on_press(Message::MagicOpLoadCatalog)
            .style(style::chip),]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_spells
            .iter()
            .map(|(i, spell)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", spell.id)).size(12).width(40),
                        text(format!(
                            "Mana:{} Dmg:{} Range:{}",
                            spell.mana_cost, spell.base_damage, spell.range
                        ))
                        .size(12),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::MagicOpSelectSpell(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Spell").size(16),
                row![
                    text("Mana Cost:").size(13).width(100),
                    text_input("", &editor.edit_mana_cost)
                        .on_input(move |v| Message::MagicOpFieldChanged(idx, "mana_cost".into(), v))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Success Rate:").size(13).width(100),
                    text_input("", &editor.edit_success_rate)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "success_rate".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Base Damage:").size(13).width(100),
                    text_input("", &editor.edit_base_damage)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "base_damage".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Range:").size(13).width(100),
                    text_input("", &editor.edit_range)
                        .on_input(move |v| Message::MagicOpFieldChanged(idx, "range".into(), v))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Level Req:").size(13).width(100),
                    text_input("", &editor.edit_level_required)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "level_required".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Effect Type:").size(13).width(100),
                    text_input("", &editor.edit_effect_type)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "effect_type".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Magic School:").size(13).width(100),
                    text_input("", &editor.edit_magic_school)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "magic_school".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Target Type:").size(13).width(100),
                    text_input("", &editor.edit_target_type)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "target_type".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(8)
            .into()
        } else {
            text("Select a spell to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::MagicOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("Spells").size(14),
                        text(format!("{} loaded", editor.filtered_spells.len())).size(12),
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
