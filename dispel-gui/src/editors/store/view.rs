use crate::app::App;
use crate::components::modal::modal;
use crate::components::textarea;
use crate::message::editor::store::StoreEditorMessage;
use crate::message::{Message, MessageExt};
use crate::state::store_editor::{EditableProduct, StoreEditorState, StorePaneContent};
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space};
use iced::widget::pane_grid::{self};
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
            ProductTypeOption::Weapon => write!(f, "Weapon"),
            ProductTypeOption::HealItem => write!(f, "HealItem"),
            ProductTypeOption::EditItem => write!(f, "EditItem"),
            ProductTypeOption::MiscItem => write!(f, "MiscItem"),
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

fn type_color_weapon(_theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(iced::Color::from_rgb(0.30, 0.78, 0.30)),
    }
}

fn type_color_heal(_theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(iced::Color::from_rgb(0.85, 0.65, 0.10)),
    }
}

fn type_color_edit(_theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(iced::Color::from_rgb(0.40, 0.60, 0.90)),
    }
}

fn type_color_misc(_theme: &iced::Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(iced::Color::from_rgb(0.70, 0.50, 0.85)),
    }
}

fn type_text_style(product_type: i16) -> fn(&iced::Theme) -> iced::widget::text::Style {
    match product_type {
        1 => type_color_weapon,
        2 => type_color_heal,
        3 => type_color_edit,
        _ => type_color_misc,
    }
}

// ── Pane builders ────────────────────────────────────────────────────────────

fn store_list_pane<'a>(editor: &'a StoreEditorState) -> Element<'a, Message> {
    let store_rows: Vec<Element<'a, Message>> = editor
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
                    text(format!("{:03}", i))
                        .size(11)
                        .style(style::subtle_text)
                        .width(32),
                    text(&store.store_name).size(12).width(Fill),
                    text(store_type)
                        .size(10)
                        .style(style::subtle_text)
                        .width(36),
                    text(format!("({})", product_count))
                        .size(10)
                        .style(style::subtle_text)
                        .width(28),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            )
            .width(Fill)
            .padding([8, 12])
            .on_press(Message::store(StoreEditorMessage::SelectStore(*i)));

            if is_selected {
                btn.style(style::active_tab_button).into()
            } else {
                btn.style(style::tab_button).into()
            }
        })
        .collect();

    let header = container(
        row![
            text("Stores").size(13),
            horizontal_space(),
            text(format!("{} loaded", editor.filtered_stores.len()))
                .size(11)
                .style(style::subtle_text),
        ]
        .padding([8, 12])
        .align_y(iced::Alignment::Center),
    )
    .style(style::grid_header_cell)
    .width(Fill);

    column![
        header,
        scrollable(column(store_rows).spacing(2)).height(Fill)
    ]
    .height(Fill)
    .into()
}

fn store_details_pane<'a>(editor: &'a StoreEditorState) -> Element<'a, Message> {
    let header = container(
        row![
            text("Store Details").size(13),
            horizontal_space(),
            button(text("Save to File").size(11))
                .on_press(Message::store(StoreEditorMessage::Save))
                .padding([4, 12])
                .style(style::commit_button),
        ]
        .padding([8, 12])
        .align_y(iced::Alignment::Center),
    )
    .style(style::grid_header_cell)
    .width(Fill);

    let body: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
        scrollable(
            column![
                field_row(
                    "Name",
                    text_input("", &editor.edit_store_name)
                        .on_input(move |v| Message::store(StoreEditorMessage::FieldChanged(
                            idx,
                            "store_name".to_string(),
                            v
                        )))
                        .padding(6)
                        .size(12)
                        .into()
                ),
                field_row(
                    "Inn Cost",
                    text_input("", &editor.edit_inn_night_cost)
                        .on_input(move |v| Message::store(StoreEditorMessage::FieldChanged(
                            idx,
                            "inn_night_cost".to_string(),
                            v
                        )))
                        .padding(6)
                        .size(12)
                        .into()
                ),
                field_row(
                    "Unknown №",
                    text_input("", &editor.edit_some_unknown_number)
                        .on_input(move |v| Message::store(StoreEditorMessage::FieldChanged(
                            idx,
                            "some_unknown_number".to_string(),
                            v
                        )))
                        .padding(6)
                        .size(12)
                        .into()
                ),
                multiline_field(
                    "Invitation",
                    textarea(&editor.edit_invitation_content, |a| Message::store(
                        StoreEditorMessage::InvitationChanged(a)
                    ),),
                ),
                multiline_field(
                    "Haggle OK",
                    textarea(&editor.edit_haggle_success_content, |a| Message::store(
                        StoreEditorMessage::HaggleSuccessChanged(a)
                    ),),
                ),
                multiline_field(
                    "Haggle Fail",
                    textarea(&editor.edit_haggle_fail_content, |a| Message::store(
                        StoreEditorMessage::HaggleFailChanged(a)
                    ),),
                ),
            ]
            .spacing(10)
            .padding(12),
        )
        .height(Fill)
        .into()
    } else {
        container(
            text("Select a store to view details")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(16)
        .into()
    };

    column![header, body].height(Fill).into()
}

fn product_list_pane<'a>(
    editor: &'a StoreEditorState,
    weapons: &'a Option<Vec<dispel_core::WeaponItem>>,
    heals: &'a Option<Vec<dispel_core::HealItem>>,
    misc: &'a Option<Vec<dispel_core::MiscItem>>,
    edit_items: &'a Option<Vec<dispel_core::EditItem>>,
) -> Element<'a, Message> {
    let can_add = editor.selected_idx.is_some() && !editor.is_inn();

    let header = container(
        row![
            text("Products").size(13),
            horizontal_space(),
            button(text("+ Add").size(11))
                .padding([4, 10])
                .style(style::browse_button)
                .on_press_maybe(
                    can_add.then_some(Message::store(StoreEditorMessage::OpenProductModal(None)))
                ),
        ]
        .padding([8, 12])
        .align_y(iced::Alignment::Center),
    )
    .style(style::grid_header_cell)
    .width(Fill);

    let body: Element<'_, Message> = if editor.selected_idx.is_none() {
        container(
            text("Select a store to view products")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(16)
        .into()
    } else if editor.is_inn() {
        container(
            text("Inns have no products")
                .size(12)
                .style(style::subtle_text),
        )
        .padding(16)
        .into()
    } else {
        let cards: Vec<Element<'a, Message>> = editor
            .edit_products
            .iter()
            .enumerate()
            .map(|(i, prod)| product_card(i, prod, editor, weapons, heals, misc, edit_items))
            .collect();

        scrollable(column(cards).spacing(4).padding([8, 8]))
            .height(Fill)
            .into()
    };

    column![header, body].height(Fill).into()
}

fn product_card<'a>(
    i: usize,
    prod: &'a EditableProduct,
    editor: &'a StoreEditorState,
    weapons: &'a Option<Vec<dispel_core::WeaponItem>>,
    heals: &'a Option<Vec<dispel_core::HealItem>>,
    misc: &'a Option<Vec<dispel_core::MiscItem>>,
    edit_items: &'a Option<Vec<dispel_core::EditItem>>,
) -> Element<'a, Message> {
    let is_selected = editor.selected_product_idx == Some(i);
    let type_opt = ProductTypeOption::from_id(prod.product_type);
    let item_name = editor.get_product_item_name(
        prod.product_type,
        prod.item_id,
        weapons,
        heals,
        misc,
        edit_items,
    );

    let card_info = row![
        text(format!("{:02}", i))
            .size(10)
            .style(style::subtle_text)
            .width(20),
        text(type_opt.to_string())
            .size(10)
            .style(type_text_style(prod.product_type))
            .width(60),
        text(format!("#{}", prod.item_id))
            .size(10)
            .style(style::subtle_text)
            .width(32),
        text(item_name).size(11).width(Fill),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    row![
        button(card_info)
            .width(Fill)
            .padding([8, 10])
            .on_press(Message::store(StoreEditorMessage::SelectProduct(i)))
            .style(if is_selected {
                style::active_chip
            } else {
                style::chip
            }),
        button(text("Edit").size(10))
            .on_press(Message::store(StoreEditorMessage::OpenProductModal(Some(
                i
            ))))
            .padding([4, 8])
            .style(style::browse_button),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

fn product_modal<'a>(
    editor: &'a StoreEditorState,
    weapons: &'a Option<Vec<dispel_core::WeaponItem>>,
    heals: &'a Option<Vec<dispel_core::HealItem>>,
    misc: &'a Option<Vec<dispel_core::MiscItem>>,
    edit_items: &'a Option<Vec<dispel_core::EditItem>>,
) -> Element<'a, Message> {
    let title = if editor.modal_product_idx.is_some() {
        "Edit Product"
    } else {
        "Add Product"
    };

    let resolved_name = editor.get_product_item_name(
        editor.modal_edit_type,
        editor.modal_edit_item_id.parse().unwrap_or(0),
        weapons,
        heals,
        misc,
        edit_items,
    );

    let content = column![
        text(title).size(15).style(style::section_header),
        horizontal_rule(1),
        row![
            text("Type").size(12).width(70),
            pick_list(
                ProductTypeOption::all(),
                Some(ProductTypeOption::from_id(editor.modal_edit_type)),
                |selected| Message::store(StoreEditorMessage::ModalTypeChanged(selected.to_id()))
            )
            .padding(6)
            .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        row![
            text("Item ID").size(12).width(70),
            text_input("", &editor.modal_edit_item_id)
                .on_input(|v| Message::store(StoreEditorMessage::ModalItemIdChanged(v)))
                .padding(6)
                .size(12)
                .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text(resolved_name).size(11).style(style::subtle_text),
        horizontal_rule(1),
        row![
            button(text("Save").size(12))
                .on_press(Message::store(StoreEditorMessage::SaveModalProduct))
                .padding([6, 20])
                .style(style::commit_button),
            button(text("Cancel").size(12))
                .on_press(Message::store(StoreEditorMessage::CloseProductModal))
                .padding([6, 20])
                .style(style::browse_button),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .padding(24)
    .width(320);

    container(content).style(style::modal_container).into()
}

// ── Layout helpers ────────────────────────────────────────────────────────────

fn field_row<'a>(label: &'a str, input: Element<'a, Message>) -> Element<'a, Message> {
    row![text(label).size(11).width(90), input]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into()
}

fn multiline_field<'a>(label: &'a str, input: Element<'a, Message>) -> Element<'a, Message> {
    column![text(label).size(11).style(style::subtle_text), input]
        .spacing(4)
        .into()
}

// ── Main view ────────────────────────────────────────────────────────────────

impl App {
    pub fn view_store_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.store_editor;
        let weapons = &self.state.weapon_editor.catalog;
        let heals = &self.state.heal_item_editor.catalog;
        let misc = &self.state.misc_item_editor.catalog;
        let edit_items = &self.state.edit_item_editor.catalog;

        let header = row![
            text("Store Editor").size(20),
            text(format!(
                " — {} stores",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
            horizontal_space(),
            button(text("Load Catalog").size(13))
                .padding([8, 16])
                .on_press(Message::store(StoreEditorMessage::LoadCatalog))
                .style(style::chip),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let panes =
            pane_grid::PaneGrid::new(&editor.pane_state, |_pane, content, _is_maximized| {
                let body: Element<'_, Message> = match content {
                    StorePaneContent::StoreList => store_list_pane(editor),
                    StorePaneContent::StoreDetails => store_details_pane(editor),
                    StorePaneContent::ProductList => {
                        product_list_pane(editor, weapons, heals, misc, edit_items)
                    }
                };
                pane_grid::Content::new(body)
            })
            .on_resize(4, |e| Message::store(StoreEditorMessage::PaneResized(e)))
            .height(Fill)
            .width(Fill);

        let base: Element<'_, Message> = column![header, horizontal_rule(1), status, panes,]
            .spacing(10)
            .padding(16)
            .height(Length::Fill)
            .into();

        if editor.show_product_modal {
            let modal_elem = product_modal(editor, weapons, heals, misc, edit_items);
            modal(
                base,
                modal_elem,
                || Message::store(StoreEditorMessage::CloseProductModal),
                0.5,
            )
        } else {
            base
        }
    }
}
