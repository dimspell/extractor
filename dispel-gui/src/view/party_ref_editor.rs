use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_party_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.party_ref_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_party
            .iter()
            .enumerate()
            .map(|(idx, (_, member))| {
                let is_selected = editor.selected_idx == Some(idx);
                let name = member.full_name.as_deref().unwrap_or("Unnamed");
                let job = member.job_name.as_deref().unwrap_or("-");
                let label = format!("[{:03}] {} - {}", member.id, name, job);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::PartyRefOpSelectMember(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Party Member Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _member)) = editor.filtered_party.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input(
                    "Full Name:",
                    &editor.edit_full_name,
                    move |v| Message::PartyRefOpFieldChanged(orig, "full_name".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Job Name:",
                    &editor.edit_job_name,
                    move |v| Message::PartyRefOpFieldChanged(orig, "job_name".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Root Map ID:",
                    &editor.edit_root_map_id,
                    move |v| Message::PartyRefOpFieldChanged(orig, "root_map_id".into(), v),
                ));
                detail_content.push(labeled_input("NPC ID:", &editor.edit_npc_id, move |v| {
                    Message::PartyRefOpFieldChanged(orig, "npc_id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Dlg Not In Party:",
                    &editor.edit_dlg_not_in_party,
                    move |v| {
                        Message::PartyRefOpFieldChanged(orig, "dlg_when_not_in_party".into(), v)
                    },
                ));
                detail_content.push(labeled_input(
                    "Dlg In Party:",
                    &editor.edit_dlg_in_party,
                    move |v| Message::PartyRefOpFieldChanged(orig, "dlg_when_in_party".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No party member selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(380)
            .style(style::info_card);

        let item_list_header = row![
            text("Party Members").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::PartyRefOpScanParty)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2)),]
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
                    button(text("Save Party Members"))
                        .on_press(Message::PartyRefOpSave)
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
