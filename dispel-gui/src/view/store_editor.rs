use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProductTypeOption {
    Weapon,
    HealItem,
    EditItem,
    MiscItem,
}

impl std::fmt::Display for ProductTypeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductTypeOption::Weapon => write!(f, "1 - Weapon"),
            ProductTypeOption::HealItem => write!(f, "2 - HealItem"),
            ProductTypeOption::EditItem => write!(f, "3 - EditItem"),
            ProductTypeOption::MiscItem => write!(f, "4 - MiscItem"),
        }
    }
}

impl ProductTypeOption {
    fn to_id(self) -> i16 {
        match self {
            ProductTypeOption::Weapon => 1,
            ProductTypeOption::HealItem => 2,
            ProductTypeOption::EditItem => 3,
            ProductTypeOption::MiscItem => 4,
        }
    }

    fn from_id(id: i16) -> Self {
        match id {
            2 => ProductTypeOption::HealItem,
            3 => ProductTypeOption::EditItem,
            4 => ProductTypeOption::MiscItem,
            _ => ProductTypeOption::Weapon,
        }
    }

    fn all() -> &'static [ProductTypeOption] {
        &[
            ProductTypeOption::Weapon,
            ProductTypeOption::HealItem,
            ProductTypeOption::EditItem,
            ProductTypeOption::MiscItem,
        ]
    }
}

impl App {
    pub fn view_store_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.store_editor;

        // Header
        let header = row![
            text("Store Editor").size(20),
            text(format!(
                " - {} stores",
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
                .on_press(Message::StoreOpLoadCatalog)
                .style(style::chip),
            horizontal_space(),
            button(text("Save to File").size(13))
                .padding([8, 16])
                .on_press(Message::StoreOpSave)
                .style(style::run_button),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        // Status
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        // Store list
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
                let product_count = store.products.len();
                let btn = button(
                    row![
                        text(format!("{:03}", i)).size(11).width(35),
                        text(&store.store_name).size(12).width(Length::Fill),
                        text(store_type)
                            .size(10)
                            .style(style::subtle_text)
                            .width(40),
                        text(format!("({})", product_count))
                            .size(10)
                            .style(style::subtle_text)
                            .width(30),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([8, 12])
                .on_press(Message::StoreOpSelectStore(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        // Product list
        let product_list: Vec<Element<'_, Message>> = editor
            .edit_products
            .iter()
            .enumerate()
            .map(|(i, prod)| {
                let is_selected = editor.selected_product_idx == Some(i);
                let type_name = ProductTypeOption::from_id(prod.product_type);
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
                        text(format!("{:02}", i)).size(10).width(24),
                        text(type_name.to_string()).size(10).width(80),
                        text(format!("#{}", item_id)).size(10).width(30),
                        text(item_name).size(11),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 10])
                .on_press(Message::StoreOpSelectProduct(i));
                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        // Product editor panel
        let product_editor_panel: Element<'_, Message> = if !editor.is_inn() {
            let product_content: Element<'_, Message> =
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
                        let item_name_owned = item_name.clone();
                        column![
                            text("Product Details").size(14),
                            horizontal_rule(1),
                            text(item_name_owned)
                                .size(13)
                                .style(style::subtle_text)
                                .width(Fill),
                            container(column![
                                row![
                                    text("Type").size(11).width(60),
                                    pick_list(
                                        ProductTypeOption::all(),
                                        Some(ProductTypeOption::from_id(prod.product_type)),
                                        move |selected| Message::StoreOpProductFieldChanged(
                                            prod_idx,
                                            "product_type".into(),
                                            selected.to_id().to_string()
                                        )
                                    )
                                    .padding(6)
                                    .width(Fill)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                                row![
                                    text("Item ID").size(11).width(60),
                                    text_input("", &prod.item_id.to_string())
                                        .on_input(move |v| Message::StoreOpProductFieldChanged(
                                            prod_idx,
                                            "item_id".into(),
                                            v
                                        ))
                                        .padding(6)
                                        .size(12)
                                        .width(Fill)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                            ])
                            .padding(12)
                            .style(style::info_card)
                            .width(Fill),
                        ]
                        .spacing(10)
                        .padding(16)
                        .into()
                    } else {
                        text("Product not found")
                            .size(12)
                            .style(style::subtle_text)
                            .into()
                    }
                } else {
                    column![
                        text("Products").size(14),
                        horizontal_rule(1),
                        text("Select a product to edit")
                            .size(12)
                            .style(style::subtle_text),
                    ]
                    .spacing(10)
                    .padding(16)
                    .into()
                };

            column![
                row![
                    text("Products").size(13),
                    horizontal_space(),
                    button(text("+ Add").size(11))
                        .on_press(Message::StoreOpAddProduct)
                        .padding([4, 10])
                        .style(style::browse_button),
                    if editor.selected_product_idx.is_some() {
                        button(text("- Remove").size(11))
                            .on_press(Message::StoreOpRemoveProduct(
                                editor.selected_product_idx.unwrap(),
                            ))
                            .padding([4, 10])
                            .style(style::browse_button)
                            .into()
                    } else {
                        Element::from(text(""))
                    }
                ]
                .padding([8, 12])
                .align_y(iced::Alignment::Center),
                scrollable(column(product_list).spacing(2)).height(180),
                product_content,
            ]
            .into()
        } else {
            column![
                text("Products").size(13),
                horizontal_rule(1),
                text("Inns have no products")
                    .size(12)
                    .style(style::subtle_text),
            ]
            .spacing(10)
            .padding(16)
            .into()
        };

        // Store editor panel
        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit Store").size(14),
                horizontal_rule(1),
                row![
                    text("Name").size(11).width(80),
                    text_input("", &editor.edit_store_name)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "store_name".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Night Cost").size(11).width(80),
                    text_input("", &editor.edit_inn_night_cost)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "inn_night_cost".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Invitation").size(11).width(80),
                    text_input("", &editor.edit_invitation)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "invitation".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Haggle OK").size(11).width(80),
                    text_input("", &editor.edit_haggle_success)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "haggle_success".into(),
                            v
                        ))
                        .padding(6)
                        .size(12)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Haggle Fail").size(11).width(80),
                    text_input("", &editor.edit_haggle_fail)
                        .on_input(move |v| Message::StoreOpFieldChanged(
                            idx,
                            "haggle_fail".into(),
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
                text("Store Details").size(14),
                horizontal_rule(1),
                text("Select a store to edit")
                    .size(12)
                    .style(style::subtle_text),
            ]
            .spacing(10)
            .padding(16)
            .into()
        };

        // Detail panel
        let detail_panel = container(
            scrollable(column![editor_panel, product_editor_panel].spacing(16))
                .height(Length::Fill),
        )
        .padding(16)
        .width(400)
        .style(style::info_card);

        // Main content
        let main_content = row![
            column![
                container(
                    row![
                        text("Stores").size(13),
                        horizontal_space(),
                        text(format!("{} loaded", editor.filtered_stores.len()))
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
