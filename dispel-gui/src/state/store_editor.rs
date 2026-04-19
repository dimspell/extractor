use crate::edit_history::EditHistory;
use crate::generic_editor::UndoRedo;
use dispel_core::{Extractor, ProductType, Store};
use iced::widget::{pane_grid, text_editor};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum StorePaneContent {
    StoreList,
    StoreDetails,
    ProductList,
}

#[derive(Debug, Clone)]
pub struct EditableProduct {
    pub order: i16,
    pub product_type: i16,
    pub item_id: i16,
}

#[derive(Debug, Clone)]
pub struct StoreEditorState {
    pub catalog: Option<Vec<Store>>,
    pub filtered_stores: Vec<(usize, Store)>,
    pub selected_idx: Option<usize>,

    pub edit_store_name: String,
    pub edit_inn_night_cost: String,
    pub edit_some_unknown_number: String,
    pub edit_invitation: String,
    pub edit_haggle_success: String,
    pub edit_haggle_fail: String,

    pub edit_invitation_content: text_editor::Content,
    pub edit_haggle_success_content: text_editor::Content,
    pub edit_haggle_fail_content: text_editor::Content,

    pub edit_products: Vec<EditableProduct>,
    pub selected_product_idx: Option<usize>,

    pub status_msg: String,
    pub loading_state: crate::loading_state::LoadingState<()>,
    pub edit_history: EditHistory,

    pub pane_state: pane_grid::State<StorePaneContent>,
    pub show_product_modal: bool,
    pub modal_product_idx: Option<usize>,
    pub modal_edit_type: i16,
    pub modal_edit_item_id: String,
}

impl Default for StoreEditorState {
    fn default() -> Self {
        let config = pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.30,
            a: Box::new(pane_grid::Configuration::Pane(StorePaneContent::StoreList)),
            b: Box::new(pane_grid::Configuration::Split {
                axis: pane_grid::Axis::Vertical,
                ratio: 0.571,
                a: Box::new(pane_grid::Configuration::Pane(
                    StorePaneContent::StoreDetails,
                )),
                b: Box::new(pane_grid::Configuration::Pane(
                    StorePaneContent::ProductList,
                )),
            }),
        };
        Self {
            catalog: None,
            filtered_stores: Vec::new(),
            selected_idx: None,
            edit_store_name: String::new(),
            edit_inn_night_cost: String::new(),
            edit_some_unknown_number: String::new(),
            edit_invitation: String::new(),
            edit_haggle_success: String::new(),
            edit_haggle_fail: String::new(),
            edit_invitation_content: text_editor::Content::new(),
            edit_haggle_success_content: text_editor::Content::new(),
            edit_haggle_fail_content: text_editor::Content::new(),
            edit_products: Vec::new(),
            selected_product_idx: None,
            status_msg: String::new(),
            loading_state: crate::loading_state::LoadingState::default(),
            edit_history: EditHistory::default(),
            pane_state: pane_grid::State::with_configuration(config),
            show_product_modal: false,
            modal_product_idx: None,
            modal_edit_type: 1,
            modal_edit_item_id: String::from("1"),
        }
    }
}

impl UndoRedo for StoreEditorState {
    fn undo(&mut self) -> Option<String> {
        self.edit_history
            .undo()
            .map(|action| format!("Undid: {:?}", action))
    }

    fn redo(&mut self) -> Option<String> {
        self.edit_history
            .redo()
            .map(|action| format!("Redid: {:?}", action))
    }

    fn can_undo(&self) -> bool {
        self.edit_history.can_undo()
    }

    fn can_redo(&self) -> bool {
        self.edit_history.can_redo()
    }

    fn edit_history(&self) -> &EditHistory {
        &self.edit_history
    }
}

impl StoreEditorState {
    pub fn refresh_stores(&mut self) {
        if let Some(catalog) = &self.catalog {
            self.filtered_stores = catalog
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.clone()))
                .collect::<Vec<_>>();
        }
    }

    pub fn select_store(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        self.selected_product_idx = None;
        if let Some((_, record)) = self.filtered_stores.get(idx) {
            self.edit_store_name = record.store_name.clone();
            self.edit_inn_night_cost = record.inn_night_cost.to_string();
            self.edit_some_unknown_number = record.some_unknown_number.to_string();
            self.edit_invitation = record.invitation.clone();
            self.edit_haggle_success = record.haggle_success.clone();
            self.edit_haggle_fail = record.haggle_fail.clone();
            self.edit_invitation_content = text_editor::Content::with_text(&record.invitation);
            self.edit_haggle_success_content =
                text_editor::Content::with_text(&record.haggle_success);
            self.edit_haggle_fail_content = text_editor::Content::with_text(&record.haggle_fail);
            self.edit_products = record
                .products
                .iter()
                .map(|(order, ptype, item_id)| EditableProduct {
                    order: *order,
                    product_type: i32::from(*ptype) as i16,
                    item_id: *item_id,
                })
                .collect();
        }
    }

    pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
        if let Some(record) = self.filtered_stores.get_mut(idx).map(|(_, r)| r) {
            match field {
                "store_name" => {
                    self.edit_store_name = value.clone();
                    record.store_name = value;
                }
                "inn_night_cost" => {
                    self.edit_inn_night_cost = value.clone();
                    if let Ok(v) = value.parse() {
                        record.inn_night_cost = v
                    }
                }
                "some_unknown_number" => {
                    self.edit_some_unknown_number = value.clone();
                    if let Ok(v) = value.parse() {
                        record.some_unknown_number = v
                    }
                }
                "invitation" => {
                    self.edit_invitation = value.clone();
                    record.invitation = value;
                }
                "haggle_success" => {
                    self.edit_haggle_success = value.clone();
                    record.haggle_success = value;
                }
                "haggle_fail" => {
                    self.edit_haggle_fail = value.clone();
                    record.haggle_fail = value;
                }
                _ => {}
            }
            self.refresh_stores();
        }
    }

    pub fn add_product(&mut self) {
        let new_order = self.edit_products.len() as i16;
        self.edit_products.push(EditableProduct {
            order: new_order,
            product_type: 1,
            item_id: 0,
        });
        self.sync_products_to_record();
    }

    pub fn remove_product(&mut self, prod_idx: usize) {
        if prod_idx < self.edit_products.len() {
            self.edit_products.remove(prod_idx);
            for (i, p) in self.edit_products.iter_mut().enumerate() {
                p.order = i as i16;
            }
            if self.selected_product_idx == Some(prod_idx) {
                self.selected_product_idx = None;
            } else if self.selected_product_idx > Some(prod_idx) {
                self.selected_product_idx = self.selected_product_idx.map(|v| v - 1);
            }
            self.sync_products_to_record();
        }
    }

    pub fn update_product(&mut self, prod_idx: usize, field: &str, value: String) {
        if let Some(product) = self.edit_products.get_mut(prod_idx) {
            match field {
                "product_type" => {
                    if let Ok(v) = value.parse() {
                        product.product_type = v;
                    }
                }
                "item_id" => {
                    if let Ok(v) = value.parse() {
                        product.item_id = v;
                    }
                }
                _ => {}
            }
            self.sync_products_to_record();
        }
    }

    fn sync_products_to_record(&mut self) {
        if let Some(selected_idx) = self.selected_idx {
            if let Some((_, record)) = self.filtered_stores.get_mut(selected_idx) {
                record.products = self
                    .edit_products
                    .iter()
                    .map(|p| {
                        let ptype = ProductType::from_i32(p.product_type as i32)
                            .unwrap_or(ProductType::MiscItem);
                        (p.order, ptype, p.item_id)
                    })
                    .collect();
            }
        }
    }

    pub fn select_product(&mut self, idx: usize) {
        self.selected_product_idx = Some(idx);
    }

    pub fn is_inn(&self) -> bool {
        self.edit_inn_night_cost.parse::<i32>().unwrap_or(0) > 0
    }

    pub fn get_product_item_name(
        &self,
        product_type: i16,
        item_id: i16,
        weapons: &Option<Vec<dispel_core::WeaponItem>>,
        heals: &Option<Vec<dispel_core::HealItem>>,
        misc: &Option<Vec<dispel_core::MiscItem>>,
        edit: &Option<Vec<dispel_core::EditItem>>,
    ) -> String {
        let idx = item_id as usize;
        match product_type {
            1 => weapons
                .as_ref()
                .and_then(|w| w.get(idx))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("Weapon #{}", item_id)),
            2 => heals
                .as_ref()
                .and_then(|h| h.get(idx))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("HealItem #{}", item_id)),
            3 => edit
                .as_ref()
                .and_then(|e| e.get(idx))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("EditItem #{}", item_id)),
            4 => misc
                .as_ref()
                .and_then(|m| m.get(idx))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("MiscItem #{}", item_id)),
            _ => format!("Unknown #{}", item_id),
        }
    }

    pub fn save_stores(&self, game_path: &str) -> Result<(), String> {
        let path = PathBuf::from(game_path)
            .join("CharacterInGame")
            .join("STORE.DB");
        if let Some(catalog) = &self.catalog {
            Store::save_file(catalog, &path).map_err(|e| format!("Failed to save stores: {}", e))
        } else {
            Err("No catalog loaded".to_string())
        }
    }
}
