use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_magic_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.magic_editor;

        // Header
        let header = row![
            text("Magic Spell Editor").size(20),
            text(format!(
                " - {} spells",
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
                .on_press(Message::MagicOpLoadCatalog)
                .style(style::chip),
            horizontal_space(),
            button(text("Save to File").size(13))
                .padding([8, 16])
                .on_press(Message::MagicOpSave)
                .style(style::run_button),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        // Spell list
        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_spells
            .iter()
            .map(|(i, spell)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", spell.id)).size(11).width(35),
                        text(format!("Dmg:{} Range:{}", spell.base_damage, spell.range))
                            .size(12)
                            .width(Length::Fill),
                        text(format!("Mana:{}", spell.mana_cost))
                            .size(10)
                            .style(style::subtle_text)
                            .width(70),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([8, 12])
                .on_press(Message::MagicOpSelectSpell(*i));
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
                text("Edit Spell").size(14),
                horizontal_rule(1),
                row![
                    text("Mana Cost").size(11).width(80),
                    text_input("", &editor.edit_mana_cost)
                        .on_input(move |v| Message::MagicOpFieldChanged(idx, "mana_cost".into(), v))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Success Rate").size(11).width(80),
                    text_input("", &editor.edit_success_rate)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "success_rate".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Base Damage").size(11).width(80),
                    text_input("", &editor.edit_base_damage)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "base_damage".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Range").size(11).width(80),
                    text_input("", &editor.edit_range)
                        .on_input(move |v| Message::MagicOpFieldChanged(idx, "range".into(), v))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Level Req").size(11).width(80),
                    text_input("", &editor.edit_level_required)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "level_required".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Effect Type").size(11).width(80),
                    text_input("", &editor.edit_effect_type)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "effect_type".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Magic School").size(11).width(80),
                    text_input("", &editor.edit_magic_school)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "magic_school".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Target Type").size(11).width(80),
                    text_input("", &editor.edit_target_type)
                        .on_input(move |v| Message::MagicOpFieldChanged(
                            idx,
                            "target_type".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(10)
            .padding(16)
            .into()
        } else {
            column![
                text("Spell Details").size(14),
                horizontal_rule(1),
                text("Select a spell to edit")
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
                        text("Spells").size(13),
                        horizontal_space(),
                        text(format!("{} loaded", editor.filtered_spells.len()))
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
