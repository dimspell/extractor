use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::store::StoreEditorMessage;
use crate::message::MessageExt;
use dispel_core::{EditItem, Extractor, HealItem, MiscItem, Store, WeaponItem};
use iced::Task;
use std::path::PathBuf;

/// Type alias to simplify the complex return type of store scanning
type StoreScanResult = Result<
    (
        Option<Vec<WeaponItem>>,
        Option<Vec<HealItem>>,
        Option<Vec<MiscItem>>,
        Option<Vec<EditItem>>,
        Vec<Store>,
    ),
    String,
>;

pub fn handle(message: StoreEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        StoreEditorMessage::LoadCatalog | StoreEditorMessage::ScanStores => {
            if app.state.shared_game_path.is_empty() {
                app.state.store_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.store_editor.loading_state = LoadingState::Loading;
            app.state.store_editor.status_msg = "Loading store catalog...".into();

            let path = PathBuf::from(&app.state.shared_game_path);
            let char_path = path.join("CharacterInGame");
            let weapons_path = char_path.join("weaponItem.db");
            let heals_path = char_path.join("HealItem.db");
            let misc_path = char_path.join("MiscItem.db");
            let edit_path = char_path.join("EditItem.db");
            let store_path = char_path.join("STORE.DB");

            Task::perform(
                async move {
                    let weapons = WeaponItem::read_file(&weapons_path).ok();
                    let heals = HealItem::read_file(&heals_path).ok();
                    let misc = MiscItem::read_file(&misc_path).ok();
                    let edit = EditItem::read_file(&edit_path).ok();
                    let stores =
                        Store::read_file(&store_path).map_err(|e: std::io::Error| e.to_string())?;
                    Ok((weapons, heals, misc, edit, stores))
                },
                |result: StoreScanResult| {
                    crate::message::Message::store(StoreEditorMessage::Scanned(result))
                },
            )
        }
        StoreEditorMessage::Scanned(result) => {
            match result {
                Ok((weapons, heals, misc, edit, stores)) => {
                    app.state.weapon_editor.catalog = weapons.clone();
                    app.state.weapon_editor.refresh();
                    app.state.heal_item_editor.catalog = heals.clone();
                    app.state.heal_item_editor.refresh();
                    app.state.misc_item_editor.catalog = misc.clone();
                    app.state.misc_item_editor.refresh();
                    app.state.edit_item_editor.catalog = edit.clone();
                    app.state.edit_item_editor.refresh();
                    app.state.store_editor.catalog = Some(stores.clone());

                    let weapons_count = weapons.as_ref().map(|w| w.len()).unwrap_or(0);
                    let heals_count = heals.as_ref().map(|h| h.len()).unwrap_or(0);
                    let misc_count = misc.as_ref().map(|m| m.len()).unwrap_or(0);
                    let edit_count = edit.as_ref().map(|e| e.len()).unwrap_or(0);

                    app.state.store_editor.status_msg = format!(
                        "Loaded: {} stores, {} weapons, {} heals, {} misc, {} edit items",
                        stores.len(),
                        weapons_count,
                        heals_count,
                        misc_count,
                        edit_count
                    );
                    app.state.store_editor.refresh_stores();
                }
                Err(e) => {
                    app.state.store_editor.status_msg =
                        format!("Error loading store catalog: {}", e)
                }
            }
            app.state.store_editor.loading_state = crate::loading_state::LoadingState::Loaded(());
            Task::none()
        }
        StoreEditorMessage::SelectStore(index) => {
            app.state.store_editor.select_store(index);
            if let Some(catalog) = &app.state.store_editor.catalog {
                if index < catalog.len() {
                    app.state.store_editor.status_msg =
                        format!("Selected store: {}", catalog[index]);
                }
            }
            Task::none()
        }
        StoreEditorMessage::FieldChanged(index, field, value) => {
            app.state.store_editor.update_field(index, &field, value);
            Task::none()
        }
        StoreEditorMessage::SelectProduct(index) => {
            app.state.store_editor.selected_product_idx = Some(index);
            app.state.store_editor.status_msg = format!("Selected product {}", index);
            Task::none()
        }
        StoreEditorMessage::AddProduct => {
            app.state.store_editor.add_product();
            Task::none()
        }
        StoreEditorMessage::RemoveProduct(index) => {
            app.state.store_editor.remove_product(index);
            Task::none()
        }
        StoreEditorMessage::ProductFieldChanged(product_idx, field, value) => {
            app.state
                .store_editor
                .update_product(product_idx, &field, value);
            Task::none()
        }
        StoreEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.store_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            if let Some(ref _catalog) = app.state.store_editor.catalog {
                app.state.store_editor.status_msg = "Saving store changes...".into();
                app.state.store_editor.loading_state = LoadingState::Loading;

                let result = app
                    .state
                    .store_editor
                    .save_stores(&app.state.shared_game_path);

                return Task::perform(async { result }, |result: Result<(), String>| {
                    crate::message::Message::store(StoreEditorMessage::Saved(result))
                });
            }

            app.state.store_editor.status_msg = "No catalog to save".into();
            Task::none()
        }
        StoreEditorMessage::Saved(result) => {
            app.state.store_editor.loading_state = crate::loading_state::LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    if let Some(ref catalog) = app.state.store_editor.catalog {
                        app.state.store_editor.status_msg =
                            format!("Successfully saved {} stores", catalog.len());
                    } else {
                        app.state.store_editor.status_msg = "Save completed".into();
                    }
                }
                Err(e) => {
                    app.state.store_editor.status_msg = format!("Error saving stores: {}", e);
                }
            }
            Task::none()
        }
        StoreEditorMessage::InvitationChanged(action) => {
            let editor = &mut app.state.store_editor;
            editor.edit_invitation_content.perform(action);
            let text = editor.edit_invitation_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "invitation", text);
            }
            Task::none()
        }
        StoreEditorMessage::HaggleSuccessChanged(action) => {
            let editor = &mut app.state.store_editor;
            editor.edit_haggle_success_content.perform(action);
            let text = editor.edit_haggle_success_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "haggle_success", text);
            }
            Task::none()
        }
        StoreEditorMessage::HaggleFailChanged(action) => {
            let editor = &mut app.state.store_editor;
            editor.edit_haggle_fail_content.perform(action);
            let text = editor.edit_haggle_fail_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "haggle_fail", text);
            }
            Task::none()
        }
        StoreEditorMessage::PaneResized(event) => {
            app.state
                .store_editor
                .pane_state
                .resize(event.split, event.ratio);
            Task::none()
        }
        StoreEditorMessage::OpenProductModal(opt_idx) => {
            let editor = &mut app.state.store_editor;
            editor.show_product_modal = true;
            editor.modal_product_idx = opt_idx;
            if let Some(idx) = opt_idx {
                if let Some(prod) = editor.edit_products.get(idx) {
                    editor.modal_edit_type = prod.product_type;
                    editor.modal_edit_item_id = prod.item_id.to_string();
                }
            } else {
                editor.modal_edit_type = 1;
                editor.modal_edit_item_id = "1".to_string();
            }
            Task::none()
        }
        StoreEditorMessage::CloseProductModal => {
            app.state.store_editor.show_product_modal = false;
            Task::none()
        }
        StoreEditorMessage::ModalTypeChanged(t) => {
            app.state.store_editor.modal_edit_type = t;
            Task::none()
        }
        StoreEditorMessage::ModalItemIdChanged(s) => {
            app.state.store_editor.modal_edit_item_id = s;
            Task::none()
        }
        StoreEditorMessage::SaveModalProduct => {
            let editor = &mut app.state.store_editor;
            let type_val = editor.modal_edit_type.to_string();
            let id_val = editor.modal_edit_item_id.clone();

            if let Some(prod_idx) = editor.modal_product_idx {
                editor.update_product(prod_idx, "product_type", type_val);
                editor.update_product(prod_idx, "item_id", id_val);
            } else {
                editor.add_product();
                let new_idx = editor.edit_products.len().saturating_sub(1);
                editor.update_product(new_idx, "product_type", type_val);
                editor.update_product(new_idx, "item_id", id_val);
            }
            editor.show_product_modal = false;
            Task::none()
        }
    }
}
