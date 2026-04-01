use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_party_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.party_ref_editor;
        let header = row![
            text("Party Member Editor").size(20),
            text(format!(
                " - {} members",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![button(text("Load Catalog").size(13))
            .padding([8, 16])
            .on_press(Message::PartyRefOpLoadCatalog)
            .style(style::chip),]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_party
            .iter()
            .map(|(i, member)| {
                let is_selected = editor.selected_idx == Some(*i);
                let name = member.full_name.as_deref().unwrap_or("Unnamed");
                let job = member.job_name.as_deref().unwrap_or("-");
                let btn = button(
                    row![
                        text(format!("{:03}", member.id)).size(12).width(40),
                        text(name).size(12),
                        text(job).size(11).style(style::subtle_text),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::PartyRefOpSelectMember(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Party Member").size(16),
                row![
                    text("Name:").size(13).width(100),
                    text_input("", &editor.edit_full_name)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "full_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Job:").size(13).width(100),
                    text_input("", &editor.edit_job_name)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "job_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Root Map:").size(13).width(100),
                    text_input("", &editor.edit_root_map_id)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "root_map_id".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("NPC ID:").size(13).width(100),
                    text_input("", &editor.edit_npc_id)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(idx, "npc_id".into(), v))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Dlg Not In:").size(13).width(100),
                    text_input("", &editor.edit_dlg_not_in_party)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "dlg_when_not_in_party".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Dlg In:").size(13).width(100),
                    text_input("", &editor.edit_dlg_in_party)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "dlg_when_in_party".into(),
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
            text("Select a party member to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::PartyRefOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("Party Members").size(14),
                        text(format!("{} loaded", editor.filtered_party.len())).size(12),
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
