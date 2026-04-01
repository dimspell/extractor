use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_misc_item_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.misc_item_editor;

        // Header
        let header = row![
            text("Misc Item Editor").size(20),
            text(format!(
                " - {} items",
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
                .on_press(Message::MiscItemOpLoadCatalog)
                .style(style::chip),
            horizontal_space(),
            button(text("Save to File").size(13))
                .padding([8, 16])
                .on_press(Message::MiscItemOpSave)
                .style(style::run_button),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        // Item list
        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_items
            .iter()
            .map(|(i, item)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(11).width(35),
                        text(&item.name).size(12).width(Length::Fill),
                        text(format!("{}g", item.base_price))
                            .size(10)
                            .style(style::subtle_text)
                            .width(60),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([8, 12])
                .on_press(Message::MiscItemOpSelectItem(*i));
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
                text("Edit Item").size(14),
                horizontal_rule(1),
                row![
                    text("Name").size(11).width(80),
                    text_input("", &editor.edit_name)
                        .on_input(move |v| Message::MiscItemOpFieldChanged(idx, "name".into(), v))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Description").size(11).width(80),
                    text_input("", &editor.edit_description)
                        .on_input(move |v| Message::MiscItemOpFieldChanged(
                            idx,
                            "description".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Base Price").size(11).width(80),
                    text_input("", &editor.edit_base_price)
                        .on_input(move |v| Message::MiscItemOpFieldChanged(
                            idx,
                            "base_price".into(),
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
                text("Item Details").size(14),
                horizontal_rule(1),
                text("Select an item to edit")
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
                        text("Items").size(13),
                        horizontal_space(),
                        text(format!("{} loaded", editor.filtered_items.len()))
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
