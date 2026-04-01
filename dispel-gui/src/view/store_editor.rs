use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_store_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.store_editor;
        let header = row![
            text("Store Editor").size(20),
            text(format!(
                " - {} stores",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![button(text("Load Catalog").size(13))
            .padding([8, 16])
            .on_press(Message::StoreOpLoadCatalog)
            .style(style::chip),]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_stores
            .iter()
            .map(|(i, store)| {
                let is_selected = editor.selected_idx == Some(*i);
                let store_type = if store.inn_night_cost > 0 {
                    "Inn"
                } else {
                    "Shop"
                };
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(12).width(40),
                        text(&store.store_name).size(12),
                        text(store_type).size(11).style(style::subtle_text),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::StoreOpSelectStore(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Store").size(16),
                row![
                    text("Name:").size(13).width(100),
                    text_input("", &editor.edit_store_name)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "store_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Night Cost:").size(13).width(100),
                    text_input("", &editor.edit_inn_night_cost)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "inn_night_cost".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Invitation:").size(13).width(100),
                    text_input("", &editor.edit_invitation)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "invitation".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Haggle OK:").size(13).width(100),
                    text_input("", &editor.edit_haggle_success)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "haggle_success".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Haggle Fail:").size(13).width(100),
                    text_input("", &editor.edit_haggle_fail)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "haggle_fail".into(),
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
            text("Select a store to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::StoreOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("Stores").size(14),
                        text(format!("{} loaded", editor.filtered_stores.len())).size(12),
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
