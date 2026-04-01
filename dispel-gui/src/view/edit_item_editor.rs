use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{labeled_file_row, labeled_input};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_edit_item_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.edit_item_editor;
        let header = row![
            text("Edit Item Editor").size(20),
            text(format!(
                " - {} items",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![
            labeled_file_row(
                "Game Path:",
                &editor.game_path,
                |_| Message::FileSelected {
                    field: "edit_item_game_path".into(),
                    path: None
                },
                Message::EditItemOpBrowseGamePath,
            ),
            button(text("Load Catalog").size(13))
                .padding([8, 16])
                .on_press(Message::EditItemOpLoadCatalog)
                .style(style::chip),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_items
            .iter()
            .map(|(i, item)| {
                let is_selected = editor.selected_idx == Some(*i);
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(12).width(40),
                        text(&item.name).size(12),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::EditItemOpSelectItem(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Item").size(16),
                labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::EditItemOpFieldChanged(idx, "name".into(), v)
                }),
                labeled_input("Description:", &editor.edit_description, move |v| {
                    Message::EditItemOpFieldChanged(idx, "description".into(), v)
                }),
                labeled_input("Base Price:", &editor.edit_base_price, move |v| {
                    Message::EditItemOpFieldChanged(idx, "base_price".into(), v)
                }),
                labeled_input("HP:", &editor.edit_health_points, move |v| {
                    Message::EditItemOpFieldChanged(idx, "health_points".into(), v)
                }),
                labeled_input("MP:", &editor.edit_mana_points, move |v| {
                    Message::EditItemOpFieldChanged(idx, "mana_points".into(), v)
                }),
                labeled_input("Strength:", &editor.edit_strength, move |v| {
                    Message::EditItemOpFieldChanged(idx, "strength".into(), v)
                }),
                labeled_input("Agility:", &editor.edit_agility, move |v| {
                    Message::EditItemOpFieldChanged(idx, "agility".into(), v)
                }),
                labeled_input("Wisdom:", &editor.edit_wisdom, move |v| {
                    Message::EditItemOpFieldChanged(idx, "wisdom".into(), v)
                }),
                labeled_input("Constitution:", &editor.edit_constitution, move |v| {
                    Message::EditItemOpFieldChanged(idx, "constitution".into(), v)
                }),
                labeled_input("To Dodge:", &editor.edit_to_dodge, move |v| {
                    Message::EditItemOpFieldChanged(idx, "to_dodge".into(), v)
                }),
                labeled_input("To Hit:", &editor.edit_to_hit, move |v| {
                    Message::EditItemOpFieldChanged(idx, "to_hit".into(), v)
                }),
                labeled_input("Offense:", &editor.edit_offense, move |v| {
                    Message::EditItemOpFieldChanged(idx, "offense".into(), v)
                }),
                labeled_input("Defense:", &editor.edit_defense, move |v| {
                    Message::EditItemOpFieldChanged(idx, "defense".into(), v)
                }),
                labeled_input("Magical Power:", &editor.edit_magical_power, move |v| {
                    Message::EditItemOpFieldChanged(idx, "magical_power".into(), v)
                }),
                labeled_input(
                    "Destroy Power:",
                    &editor.edit_item_destroying_power,
                    move |v| Message::EditItemOpFieldChanged(
                        idx,
                        "item_destroying_power".into(),
                        v
                    )
                ),
                labeled_input("Modifies Item:", &editor.edit_modifies_item, move |v| {
                    Message::EditItemOpFieldChanged(idx, "modifies_item".into(), v)
                }),
                labeled_input("Effect:", &editor.edit_additional_effect, move |v| {
                    Message::EditItemOpFieldChanged(idx, "additional_effect".into(), v)
                }),
            ]
            .spacing(8)
            .into()
        } else {
            text("Select an item to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::EditItemOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("Items").size(14),
                        text(format!("{} loaded", editor.filtered_items.len())).size(12),
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
