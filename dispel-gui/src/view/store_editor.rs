use crate::app::App;
use crate::message::Message;
use crate::style;

use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input,
};
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

        let product_type_label = |t: i16| -> String {
            match t {
                1 => "Weapon".to_string(),
                2 => "HealItem".to_string(),
                3 => "EditItem".to_string(),
                4 => "MiscItem".to_string(),
                _ => format!("Type{}", t),
            }
        };

        let product_list: Vec<Element<'_, Message>> = editor
            .edit_products
            .iter()
            .enumerate()
            .map(|(i, prod)| {
                let is_selected = editor.selected_product_idx == Some(i);
                let type_name = product_type_label(prod.product_type);
                let item_id = prod.item_id;
                let item_name = editor.get_product_item_name(
                    prod.product_type,
                    item_id,
                    &self.weapon_editor.catalog,
                    &self.heal_item_editor.catalog,
                    &self.misc_item_editor.catalog,
                    &self.edit_item_editor.catalog,
                );
                let btn = button(
                    row![
                        text(format!("{:02}", i)).size(11).width(24),
                        text(type_name).size(11).width(60),
                        text(format!("#{}", item_id)).size(11).width(30),
                        text(item_name).size(11),
                    ]
                    .spacing(4)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([4, 8])
                .on_press(Message::StoreOpSelectProduct(i));
                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let products_panel: Element<'_, Message> = if !editor.is_inn() {
            let product_editor_content: Element<'_, Message> =
                if let Some(prod_idx) = editor.selected_product_idx {
                    if let Some(prod) = editor.edit_products.get(prod_idx) {
                        let item_name = editor.get_product_item_name(
                            prod.product_type,
                            prod.item_id,
                            &self.weapon_editor.catalog,
                            &self.heal_item_editor.catalog,
                            &self.misc_item_editor.catalog,
                            &self.edit_item_editor.catalog,
                        );
                        column![
                            text("Product Details").size(14),
                            text(item_name).size(13).style(style::subtle_text),
                            container(
                                row![
                                    text("Type:").size(12).width(60),
                                    text_input("", &product_type_label(prod.product_type))
                                        .on_input(move |v| {
                                            let t = match v.to_lowercase().as_str() {
                                                "weapon" | "1" => 1,
                                                "healitem" | "heal" | "healing" | "2" => 2,
                                                "edititem" | "edit" | "3" => 3,
                                                "miscitem" | "misc" | "4" => 4,
                                                _ => 1,
                                            };
                                            Message::StoreOpProductFieldChanged(
                                                prod_idx,
                                                "product_type".into(),
                                                t.to_string(),
                                            )
                                        })
                                        .padding(6)
                                        .size(12)
                                ]
                                .spacing(8)
                            ),
                            container(
                                row![
                                    text("Item ID:").size(12).width(60),
                                    text_input("", &prod.item_id.to_string())
                                        .on_input(move |v| {
                                            Message::StoreOpProductFieldChanged(
                                                prod_idx,
                                                "item_id".into(),
                                                v,
                                            )
                                        })
                                        .padding(6)
                                        .size(12)
                                ]
                                .spacing(8)
                            ),
                        ]
                        .spacing(12)
                        .padding(16)
                        .into()
                    } else {
                        Element::from(text("Select a product").size(12).style(style::subtle_text))
                    }
                } else {
                    Element::from(
                        text("Select a product to edit")
                            .size(12)
                            .style(style::subtle_text),
                    )
                };

            column![
                row![
                    text("Products").size(13),
                    horizontal_space(),
                    button(text("+ Add").size(12))
                        .on_press(Message::StoreOpAddProduct)
                        .padding([4, 12])
                        .style(style::browse_button),
                    if editor.selected_product_idx.is_some() {
                        button(text("- Remove").size(12))
                            .on_press(Message::StoreOpRemoveProduct(
                                editor.selected_product_idx.unwrap(),
                            ))
                            .padding([4, 12])
                            .style(style::browse_button)
                            .into()
                    } else {
                        Element::from(text(""))
                    }
                ]
                .padding([8, 12])
                .align_y(iced::Alignment::Center),
                scrollable(column(product_list).spacing(2)).height(150),
                container(product_editor_content)
                    .style(style::info_card)
                    .height(Fill),
            ]
            .into()
        } else {
            text("Inns have no products")
                .size(12)
                .style(style::subtle_text)
                .into()
        };

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

        let detail_panel = container(column![
            scrollable(editor_panel).height(Length::FillPortion(1)),
            container(products_panel)
                .style(style::info_card)
                .height(Length::FillPortion(1)),
        ])
        .padding(16)
        .width(400);

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
