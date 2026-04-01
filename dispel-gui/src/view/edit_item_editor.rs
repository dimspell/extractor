use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_edit_item_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.edit_item_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_items
            .iter()
            .enumerate()
            .map(|(idx, (_, item))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!("[{}] {} - {}g", item.index, item.name, item.base_price);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::EditItemOpSelectItem(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Edit Item Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _item)) = editor.filtered_items.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::EditItemOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Description:",
                    &editor.edit_description,
                    move |v| Message::EditItemOpFieldChanged(orig, "description".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Base Price (gold):",
                    &editor.edit_base_price,
                    move |v| Message::EditItemOpFieldChanged(orig, "base_price".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Health Points:",
                    &editor.edit_health_points,
                    move |v| Message::EditItemOpFieldChanged(orig, "health_points".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Mana Points:",
                    &editor.edit_mana_points,
                    move |v| Message::EditItemOpFieldChanged(orig, "mana_points".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Strength:",
                    &editor.edit_strength,
                    move |v| Message::EditItemOpFieldChanged(orig, "strength".into(), v),
                ));
                detail_content.push(labeled_input("Agility:", &editor.edit_agility, move |v| {
                    Message::EditItemOpFieldChanged(orig, "agility".into(), v)
                }));
                detail_content.push(labeled_input("Wisdom:", &editor.edit_wisdom, move |v| {
                    Message::EditItemOpFieldChanged(orig, "wisdom".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Constitution:",
                    &editor.edit_constitution,
                    move |v| Message::EditItemOpFieldChanged(orig, "constitution".into(), v),
                ));
                detail_content.push(labeled_input(
                    "To Dodge:",
                    &editor.edit_to_dodge,
                    move |v| Message::EditItemOpFieldChanged(orig, "to_dodge".into(), v),
                ));
                detail_content.push(labeled_input("To Hit:", &editor.edit_to_hit, move |v| {
                    Message::EditItemOpFieldChanged(orig, "to_hit".into(), v)
                }));
                detail_content.push(labeled_input("Offense:", &editor.edit_offense, move |v| {
                    Message::EditItemOpFieldChanged(orig, "offense".into(), v)
                }));
                detail_content.push(labeled_input("Defense:", &editor.edit_defense, move |v| {
                    Message::EditItemOpFieldChanged(orig, "defense".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Magical Power:",
                    &editor.edit_magical_power,
                    move |v| Message::EditItemOpFieldChanged(orig, "magical_power".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Destroy Power:",
                    &editor.edit_item_destroying_power,
                    move |v| {
                        Message::EditItemOpFieldChanged(orig, "item_destroying_power".into(), v)
                    },
                ));
                detail_content.push(labeled_input(
                    "Modifies Item:",
                    &editor.edit_modifies_item,
                    move |v| Message::EditItemOpFieldChanged(orig, "modifies_item".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Effect:",
                    &editor.edit_additional_effect,
                    move |v| Message::EditItemOpFieldChanged(orig, "additional_effect".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No edit item selected")
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
            text("Edit Items").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::EditItemOpScanItems)
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
                    button(text("Save Edit Items"))
                        .on_press(Message::EditItemOpSave)
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
