use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_party_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.party_ref_editor;

        // Header
        let header = row![
            text("Party Member Editor").size(20),
            text(format!(
                " - {} members",
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
                .on_press(Message::PartyRefOpLoadCatalog)
                .style(style::chip),
            horizontal_space(),
            button(text("Save to File").size(13))
                .padding([8, 16])
                .on_press(Message::PartyRefOpSave)
                .style(style::run_button),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        // Party member list
        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_party
            .iter()
            .map(|(i, member)| {
                let is_selected = editor.selected_idx == Some(*i);
                let name = member.full_name.as_deref().unwrap_or("Unnamed");
                let job = member.job_name.as_deref().unwrap_or("-");
                let btn = button(
                    row![
                        text(format!("{:03}", member.id)).size(11).width(35),
                        text(name).size(12).width(Length::Fill),
                        text(job).size(10).style(style::subtle_text),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([8, 12])
                .on_press(Message::PartyRefOpSelectMember(*i));
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
                text("Edit Party Member").size(14),
                horizontal_rule(1),
                row![
                    text("Name").size(11).width(80),
                    text_input("", &editor.edit_full_name)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "full_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Job").size(11).width(80),
                    text_input("", &editor.edit_job_name)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "job_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Root Map").size(11).width(80),
                    text_input("", &editor.edit_root_map_id)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "root_map_id".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("NPC ID").size(11).width(80),
                    text_input("", &editor.edit_npc_id)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(idx, "npc_id".into(), v))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Dlg Not In").size(11).width(80),
                    text_input("", &editor.edit_dlg_not_in_party)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "dlg_when_not_in_party".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Dlg In").size(11).width(80),
                    text_input("", &editor.edit_dlg_in_party)
                        .on_input(move |v| Message::PartyRefOpFieldChanged(
                            idx,
                            "dlg_when_in_party".into(),
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
                text("Member Details").size(14),
                horizontal_rule(1),
                text("Select a party member to edit")
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
                        text("Party Members").size(13),
                        horizontal_space(),
                        text(format!("{} loaded", editor.filtered_party.len()))
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
