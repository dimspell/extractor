use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_party_level_db_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.party_level_db_editor;

        let npc_list: Vec<Element<Message>> = if let Some(catalog) = &editor.catalog {
            catalog
                .iter()
                .enumerate()
                .map(|(idx, npc)| {
                    let is_selected = editor.selected_npc_idx == Some(idx);
                    let label = format!("NPC {}", npc.npc_index);

                    let btn = button(text(label).size(11).font(Font::MONOSPACE))
                        .width(Fill)
                        .on_press(Message::PartyLevelDbOpSelectNpc(idx));

                    if is_selected {
                        btn.style(style::active_chip).into()
                    } else {
                        btn.style(style::chip).into()
                    }
                })
                .collect()
        } else {
            vec![text("No data loaded")
                .size(13)
                .style(style::subtle_text)
                .into()]
        };

        let npc_scroll = scrollable(column(npc_list).spacing(4)).height(Length::Fill);

        let level_list: Vec<Element<Message>> = if let Some(npc_idx) = editor.selected_npc_idx {
            if let Some(catalog) = &editor.catalog {
                if let Some(npc) = catalog.get(npc_idx) {
                    npc.records
                        .iter()
                        .enumerate()
                        .map(|(idx, record)| {
                            let is_selected = editor.selected_record_idx == Some(idx);
                            let label = format!(
                                "Lv.{} STR:{} CON:{} WIS:{}",
                                record.level, record.strength, record.constitution, record.wisdom
                            );

                            let btn = button(text(label).size(10).font(Font::MONOSPACE))
                                .width(Fill)
                                .on_press(Message::PartyLevelDbOpSelectRecord(idx));

                            if is_selected {
                                btn.style(style::active_chip).into()
                            } else {
                                btn.style(style::chip).into()
                            }
                        })
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![text("Select an NPC")
                .size(13)
                .style(style::subtle_text)
                .into()]
        };

        let level_scroll = scrollable(column(level_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Level Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if editor.selected_record_idx.is_some() {
            detail_content.push(labeled_input("Level:", &editor.edit_level, |v| {
                Message::PartyLevelDbOpFieldChanged("level".into(), v)
            }));
            detail_content.push(labeled_input("Strength:", &editor.edit_strength, |v| {
                Message::PartyLevelDbOpFieldChanged("strength".into(), v)
            }));
            detail_content.push(labeled_input(
                "Constitution:",
                &editor.edit_constitution,
                |v| Message::PartyLevelDbOpFieldChanged("constitution".into(), v),
            ));
            detail_content.push(labeled_input("Wisdom:", &editor.edit_wisdom, |v| {
                Message::PartyLevelDbOpFieldChanged("wisdom".into(), v)
            }));
            detail_content.push(labeled_input(
                "Health Points:",
                &editor.edit_health_points,
                |v| Message::PartyLevelDbOpFieldChanged("health_points".into(), v),
            ));
            detail_content.push(labeled_input(
                "Mana Points:",
                &editor.edit_mana_points,
                |v| Message::PartyLevelDbOpFieldChanged("mana_points".into(), v),
            ));
            detail_content.push(labeled_input("Agility:", &editor.edit_agility, |v| {
                Message::PartyLevelDbOpFieldChanged("agility".into(), v)
            }));
            detail_content.push(labeled_input("Attack:", &editor.edit_attack, |v| {
                Message::PartyLevelDbOpFieldChanged("attack".into(), v)
            }));
            detail_content.push(labeled_input(
                "Mana Recharge:",
                &editor.edit_mana_recharge,
                |v| Message::PartyLevelDbOpFieldChanged("mana_recharge".into(), v),
            ));
            detail_content.push(labeled_input("Defense:", &editor.edit_defense, |v| {
                Message::PartyLevelDbOpFieldChanged("defense".into(), v)
            }));
        } else {
            detail_content.push(
                text("No level selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(300)
            .style(style::info_card);

        let npc_header = row![
            text("NPCs").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::PartyLevelDbOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let level_header = row![text("Levels").size(14), horizontal_space(),]
            .padding(10)
            .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(npc_header).style(style::grid_header_cell),
            npc_scroll,
        ];

        let middle_panel = column![
            container(level_header).style(style::grid_header_cell),
            level_scroll,
        ];

        let main_content = row![
            left_panel.width(Length::FillPortion(1)),
            middle_panel.width(Length::FillPortion(1)),
            detail_panel,
        ]
        .spacing(0)
        .height(Length::Fill);

        column![
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if editor.is_loading {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save Party Levels"))
                        .on_press(Message::PartyLevelDbOpSave)
                        .style(style::commit_button),
                ]
                .padding([10, 20])
                .align_y(iced::Alignment::Center),
            )
            .width(Fill)
            .style(style::status_bar),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}
