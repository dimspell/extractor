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
        let action = self.edit_history.undo()?;
        if let crate::edit_history::EditAction::FieldChange {
            record_idx,
            ref field,
            ref old_value,
            ..
        } = action
        {
            self.apply_field_to_store(record_idx, field, old_value.clone());
        }
        Some(format!("Undo: {}", action.display_text()))
    }

    fn redo(&mut self) -> Option<String> {
        let action = self.edit_history.redo()?;
        // The redo action is the inverted undo action: old_value holds the value to re-apply.
        if let crate::edit_history::EditAction::FieldChange {
            record_idx,
            ref field,
            ref old_value,
            ..
        } = action
        {
            self.apply_field_to_store(record_idx, field, old_value.clone());
        }
        Some(format!("Redo: {}", action.display_text()))
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
        // Borrow 1: read orig_idx and old_value, then drop the borrow
        let (orig_idx, old_value) = match self.filtered_stores.get(idx) {
            Some((i, record)) => {
                let old = match field {
                    "store_name" => record.store_name.clone(),
                    "inn_night_cost" => record.inn_night_cost.to_string(),
                    "some_unknown_number" => record.some_unknown_number.to_string(),
                    "invitation" => record.invitation.clone(),
                    "haggle_success" => record.haggle_success.clone(),
                    "haggle_fail" => record.haggle_fail.clone(),
                    _ => return,
                };
                (*i, old)
            }
            None => return,
        };
        if old_value == value {
            return;
        }

        // Borrow 2: apply mutation, sync catalog, and refresh buffers
        self.apply_field_to_store(orig_idx, field, value.clone());

        self.edit_history
            .push(crate::edit_history::EditAction::FieldChange {
                record_idx: orig_idx,
                field: field.to_string(),
                old_value,
                new_value: value,
            });
    }

    fn apply_field_to_store(&mut self, orig_idx: usize, field: &str, value: String) {
        // Update filtered_stores entry
        if let Some((_, record)) = self
            .filtered_stores
            .iter_mut()
            .find(|(i, _)| *i == orig_idx)
        {
            match field {
                "store_name" => record.store_name = value.clone(),
                "inn_night_cost" => {
                    if let Ok(v) = value.parse() {
                        record.inn_night_cost = v
                    }
                }
                "some_unknown_number" => {
                    if let Ok(v) = value.parse() {
                        record.some_unknown_number = v
                    }
                }
                "invitation" => record.invitation = value.clone(),
                "haggle_success" => record.haggle_success = value.clone(),
                "haggle_fail" => record.haggle_fail = value.clone(),
                _ => {}
            }
        }
        // Sync to catalog so saves are consistent
        if let Some(catalog) = &mut self.catalog {
            if let Some(cat_record) = catalog.get_mut(orig_idx) {
                match field {
                    "store_name" => cat_record.store_name = value.clone(),
                    "inn_night_cost" => {
                        if let Ok(v) = value.parse() {
                            cat_record.inn_night_cost = v
                        }
                    }
                    "some_unknown_number" => {
                        if let Ok(v) = value.parse() {
                            cat_record.some_unknown_number = v
                        }
                    }
                    "invitation" => cat_record.invitation = value.clone(),
                    "haggle_success" => cat_record.haggle_success = value.clone(),
                    "haggle_fail" => cat_record.haggle_fail = value.clone(),
                    _ => {}
                }
            }
        }
        // Re-populate edit buffers if this store is currently selected
        if let Some(sel) = self.selected_idx {
            if self.filtered_stores.get(sel).map(|(i, _)| *i) == Some(orig_idx) {
                self.select_store(sel);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generic_editor::UndoRedo;

    fn make_store(name: &str) -> Store {
        Store {
            index: 0,
            store_name: name.to_string(),
            inn_night_cost: 0,
            some_unknown_number: 0,
            invitation: String::new(),
            haggle_success: String::new(),
            haggle_fail: String::new(),
            products: Vec::new(),
        }
    }

    fn editor_with_one_store(name: &str) -> StoreEditorState {
        let mut editor = StoreEditorState::default();
        editor.catalog = Some(vec![make_store(name)]);
        editor.refresh_stores();
        editor.select_store(0);
        editor
    }

    #[test]
    fn test_update_field_records_history() {
        let mut editor = editor_with_one_store("Old Name");
        editor.update_field(0, "store_name", "New Name".to_string());
        assert!(editor.edit_history.can_undo());
    }

    #[test]
    fn test_update_field_no_change_skips_history() {
        let mut editor = editor_with_one_store("Same");
        editor.update_field(0, "store_name", "Same".to_string());
        assert!(!editor.edit_history.can_undo());
    }

    #[test]
    fn test_undo_reverts_field() {
        let mut editor = editor_with_one_store("Before");
        editor.update_field(0, "store_name", "After".to_string());
        let msg = editor.undo();
        assert!(msg.is_some());
        assert_eq!(editor.filtered_stores[0].1.store_name, "Before");
        assert_eq!(editor.catalog.as_ref().unwrap()[0].store_name, "Before");
    }

    #[test]
    fn test_undo_syncs_edit_buffers() {
        let mut editor = editor_with_one_store("Original");
        editor.update_field(0, "store_name", "Changed".to_string());
        editor.undo();
        assert_eq!(editor.edit_store_name, "Original");
    }

    #[test]
    fn test_redo_reapplies_field() {
        let mut editor = editor_with_one_store("A");
        editor.update_field(0, "store_name", "B".to_string());
        editor.undo();
        let msg = editor.redo();
        assert!(msg.is_some());
        assert_eq!(editor.filtered_stores[0].1.store_name, "B");
        assert_eq!(editor.edit_store_name, "B");
    }

    #[test]
    fn test_multiple_undo_levels() {
        let mut editor = editor_with_one_store("v0");
        editor.update_field(0, "store_name", "v1".to_string());
        editor.update_field(0, "store_name", "v2".to_string());
        editor.update_field(0, "store_name", "v3".to_string());
        editor.undo();
        assert_eq!(editor.filtered_stores[0].1.store_name, "v2");
        editor.undo();
        assert_eq!(editor.filtered_stores[0].1.store_name, "v1");
        editor.undo();
        assert_eq!(editor.filtered_stores[0].1.store_name, "v0");
        assert!(!editor.edit_history.can_undo());
    }
}
