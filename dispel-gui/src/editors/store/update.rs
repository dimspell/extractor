use crate::app::App;
use crate::components::editable::EditableRecord;
use crate::components::loading_state::LoadingState;
use crate::editors::store::StoreEditorMessage;
use crate::message::MessageExt;
use dispel_core::{EditItem, Extractor, HealItem, MiscItem, Store, WeaponItem};
use iced::Task;
use std::path::{Path, PathBuf};

/// Type alias to simplify the complex return type of store scanning. The
/// trailing strings are non-fatal warnings from optional item catalogs.
type StoreScanResult = Result<
    (
        Option<Vec<WeaponItem>>,
        Option<Vec<HealItem>>,
        Option<Vec<MiscItem>>,
        Option<Vec<EditItem>>,
        Vec<Store>,
        Vec<String>,
    ),
    String,
>;

/// Read a catalog that Store only cross-references. A missing or unreadable
/// optional catalog is reported as a warning instead of failing the scan.
fn read_optional_catalog<T: Extractor>(path: &Path, warnings: &mut Vec<String>) -> Option<Vec<T>> {
    match T::read_file(path) {
        Ok(records) => Some(records),
        Err(e) => {
            warnings.push(format!("{}: {}", path.display(), e));
            None
        }
    }
}

/// Read STORE.DB plus the four item catalogs it cross-references. Only STORE.DB
/// is required: a partial install may lack one of the item catalogs, which is
/// reported as a non-fatal warning so the store editor can still be opened.
async fn scan_store_catalog(
    weapons_path: PathBuf,
    heals_path: PathBuf,
    misc_path: PathBuf,
    edit_path: PathBuf,
    store_path: PathBuf,
) -> StoreScanResult {
    let mut warnings = Vec::new();
    let weapons = read_optional_catalog(&weapons_path, &mut warnings);
    let heals = read_optional_catalog(&heals_path, &mut warnings);
    let misc = read_optional_catalog(&misc_path, &mut warnings);
    let edit = read_optional_catalog(&edit_path, &mut warnings);
    let stores =
        Store::read_file(&store_path).map_err(|e| format!("{}: {}", store_path.display(), e))?;
    Ok((weapons, heals, misc, edit, stores, warnings))
}

pub fn handle(message: StoreEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        StoreEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.store_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            app.state.editors.store_editor.loading_state = LoadingState::Loading;
            app.state.editors.store_editor.status_msg = "Loading store catalog...".into();

            let path = PathBuf::from(&app.state.shared_game_path);
            let char_path = path.join("CharacterInGame");
            let weapons_path = char_path.join("weaponItem.db");
            let heals_path = char_path.join("HealItem.db");
            let misc_path = char_path.join("MiscItem.db");
            let edit_path = char_path.join("EditItem.db");
            let store_path = char_path.join("STORE.DB");

            Task::perform(
                scan_store_catalog(weapons_path, heals_path, misc_path, edit_path, store_path),
                |result: StoreScanResult| {
                    crate::message::Message::store(StoreEditorMessage::Scanned(result))
                },
            )
        }
        StoreEditorMessage::Scanned(result) => {
            match result {
                Ok((weapons, heals, misc, edit, stores, warnings)) => {
                    // These four item catalogs are shared with their own editors.
                    // A scan must never discard edits that are still in the
                    // editor's undo history, so keep a loaded, modified catalog.
                    macro_rules! apply_scanned {
                        ($editor:expr, $scanned:expr) => {
                            if let Some(records) = $scanned.as_ref() {
                                let editor = &mut $editor;
                                let has_unsaved_edits =
                                    editor.catalog.is_some() && editor.edit_history.can_undo();
                                if !has_unsaved_edits {
                                    editor.catalog = Some(records.clone());
                                    editor.refresh();
                                    editor.spreadsheet.apply_filter(records);
                                }
                            }
                        };
                    }
                    apply_scanned!(app.state.editors.weapon_editor, weapons);
                    apply_scanned!(app.state.editors.heal_item_editor, heals);
                    apply_scanned!(app.state.editors.misc_item_editor, misc);
                    apply_scanned!(app.state.editors.edit_item_editor, edit);
                    app.state.editors.store_editor.catalog = Some(stores.clone());

                    let weapons_count = weapons.as_ref().map(|w| w.len()).unwrap_or(0);
                    let heals_count = heals.as_ref().map(|h| h.len()).unwrap_or(0);
                    let misc_count = misc.as_ref().map(|m| m.len()).unwrap_or(0);
                    let edit_count = edit.as_ref().map(|e| e.len()).unwrap_or(0);

                    let mut status = format!(
                        "Loaded: {} stores, {} weapons, {} heals, {} misc, {} edit items",
                        stores.len(),
                        weapons_count,
                        heals_count,
                        misc_count,
                        edit_count
                    );
                    for warning in warnings {
                        status.push_str(" · Warning: ");
                        status.push_str(&warning);
                    }
                    app.state.editors.store_editor.status_msg = status;
                    app.state.editors.store_editor.refresh_stores();
                }
                Err(e) => {
                    app.state.editors.store_editor.status_msg =
                        format!("Error loading store catalog: {}", e)
                }
            }
            app.state.editors.store_editor.loading_state =
                crate::components::loading_state::LoadingState::Loaded(());
            Task::none()
        }
        StoreEditorMessage::SelectStore(index) => {
            app.state.editors.store_editor.select_store(index);
            if let Some(catalog) = &app.state.editors.store_editor.catalog
                && index < catalog.len()
            {
                app.state.editors.store_editor.status_msg =
                    format!("Selected store: {}", catalog[index]);
            }
            Task::none()
        }
        StoreEditorMessage::FieldChanged(index, field, value) => {
            // Capture (orig_idx, old) BEFORE the edit. `index` from the
            // message is the position in the *filtered* view; the patcher
            // addresses by the original record id, so use `i` from the
            // (orig_idx, store) tuple. Bail out if the row is missing.
            let captured = app
                .state
                .editors
                .store_editor
                .filtered_stores
                .get(index)
                .map(|(i, r)| (*i as u32, r.get_field(&field)));
            let new_value = value.clone();
            app.state
                .editors
                .store_editor
                .update_field(index, &field, value);
            match captured {
                Some((orig_idx, old_value)) if old_value != new_value => {
                    crate::editors::mod_packager::recording::observe_field_change(
                        app,
                        "CharacterInGame/STORE.DB",
                        orig_idx,
                        &field,
                        old_value,
                        new_value,
                    )
                }
                _ => Task::none(),
            }
        }
        StoreEditorMessage::SelectProduct(index) => {
            app.state.editors.store_editor.selected_product_idx = Some(index);
            app.state.editors.store_editor.status_msg = format!("Selected product {}", index);
            Task::none()
        }
        StoreEditorMessage::AddProduct => {
            app.state.editors.store_editor.add_product();
            Task::none()
        }
        StoreEditorMessage::RemoveProduct(index) => {
            app.state.editors.store_editor.remove_product(index);
            Task::none()
        }
        StoreEditorMessage::ProductFieldChanged(product_idx, field, value) => {
            app.state
                .editors
                .store_editor
                .update_product(product_idx, &field, value);
            Task::none()
        }
        StoreEditorMessage::Save => {
            if app.state.shared_game_path.is_empty() {
                app.state.editors.store_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }

            if let Some(ref _catalog) = app.state.editors.store_editor.catalog {
                app.state.editors.store_editor.status_msg = "Saving store changes...".into();
                app.state.editors.store_editor.loading_state = LoadingState::Loading;

                let result = app
                    .state
                    .editors
                    .store_editor
                    .save_stores(&app.state.shared_game_path);

                return Task::perform(async { result }, |result: Result<(), String>| {
                    crate::message::Message::store(StoreEditorMessage::Saved(result))
                });
            }

            app.state.editors.store_editor.status_msg = "No catalog to save".into();
            Task::none()
        }
        StoreEditorMessage::Saved(result) => {
            app.state.editors.store_editor.loading_state =
                crate::components::loading_state::LoadingState::Loaded(());

            match result {
                Ok(_) => {
                    if let Some(ref catalog) = app.state.editors.store_editor.catalog {
                        app.state.editors.store_editor.status_msg =
                            format!("Successfully saved {} stores", catalog.len());
                    } else {
                        app.state.editors.store_editor.status_msg = "Save completed".into();
                    }
                }
                Err(e) => {
                    app.state.editors.store_editor.status_msg =
                        format!("Error saving stores: {}", e);
                }
            }
            Task::none()
        }
        StoreEditorMessage::InvitationChanged(action) => {
            let editor = &mut app.state.editors.store_editor;
            editor.edit_invitation_content.perform(action);
            let text = editor.edit_invitation_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "invitation", text);
            }
            Task::none()
        }
        StoreEditorMessage::HaggleSuccessChanged(action) => {
            let editor = &mut app.state.editors.store_editor;
            editor.edit_haggle_success_content.perform(action);
            let text = editor.edit_haggle_success_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "haggle_success", text);
            }
            Task::none()
        }
        StoreEditorMessage::HaggleFailChanged(action) => {
            let editor = &mut app.state.editors.store_editor;
            editor.edit_haggle_fail_content.perform(action);
            let text = editor.edit_haggle_fail_content.text();
            if let Some(idx) = editor.selected_idx {
                editor.update_field(idx, "haggle_fail", text);
            }
            Task::none()
        }
        StoreEditorMessage::PaneResized(event) => {
            app.state
                .editors
                .store_editor
                .pane_state
                .resize(event.split, event.ratio);
            Task::none()
        }
        StoreEditorMessage::OpenProductModal(opt_idx) => {
            let editor = &mut app.state.editors.store_editor;
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
            app.state.editors.store_editor.show_product_modal = false;
            Task::none()
        }
        StoreEditorMessage::ModalTypeChanged(t) => {
            app.state.editors.store_editor.modal_edit_type = t;
            Task::none()
        }
        StoreEditorMessage::ModalItemIdChanged(s) => {
            app.state.editors.store_editor.modal_edit_item_id = s;
            Task::none()
        }
        StoreEditorMessage::SaveModalProduct => {
            let editor = &mut app.state.editors.store_editor;
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

#[cfg(test)]
mod tests {
    use super::*;
    use dispel_core::Store;

    fn scanned_with_weapons(weapons: Vec<WeaponItem>) -> StoreEditorMessage {
        StoreEditorMessage::Scanned(Ok((
            Some(weapons),
            Some(vec![]),
            Some(vec![]),
            Some(vec![]),
            vec![Store::default()],
            vec![],
        )))
    }

    #[test]
    fn test_store_scan_initializes_filtered_indices() {
        let mut app = App::test_new(crate::workspace::Workspace::new());
        let weapons = vec![
            WeaponItem {
                name: "Invented Blade".to_string(),
                ..Default::default()
            },
            WeaponItem {
                name: "Invented Shield".to_string(),
                ..Default::default()
            },
        ];

        let _ = handle(scanned_with_weapons(weapons), &mut app);

        let editor = &app.state.editors.weapon_editor;
        assert_eq!(editor.catalog.as_ref().unwrap().len(), 2);
        assert_eq!(
            editor.spreadsheet.filtered_indices,
            vec![0, 1],
            "scan must initialize filtered_indices for every loaded record"
        );
    }

    #[cfg(feature = "iced_test")]
    #[test]
    fn test_store_scan_table_is_not_empty() {
        use crate::view::editor::spreadsheet::view::table::build_table_content;
        use iced_test::simulator;

        let mut app = App::test_new(crate::workspace::Workspace::new());
        let weapons = vec![WeaponItem {
            name: "Invented Blade".to_string(),
            ..Default::default()
        }];
        let _ = handle(scanned_with_weapons(weapons), &mut app);

        let editor = &app.state.editors.weapon_editor;
        let element: iced::Element<'_, crate::message::Message> = build_table_content::<WeaponItem>(
            WeaponItem::field_descriptors(),
            editor.catalog.as_ref(),
            &editor.spreadsheet,
            |_| crate::message::Message::System(crate::message::SystemMessage::DismissError),
        );
        let mut ui = simulator(element);
        ui.find("No matching records")
            .expect_err("scanned records must be visible instead of the empty-filter message");
    }

    #[test]
    fn test_store_scan_preserves_unsaved_item_edits() {
        let mut app = App::test_new(crate::workspace::Workspace::new());
        let weapon = WeaponItem {
            name: "Edited In Editor".to_string(),
            ..Default::default()
        };
        app.state.editors.weapon_editor.catalog = Some(vec![weapon]);
        app.state.editors.weapon_editor.refresh();
        app.state.editors.weapon_editor.select(0);
        assert!(app.state.editors.weapon_editor.update_field(
            0,
            "name",
            "Unsaved Name".to_string()
        ));
        assert!(app.state.editors.weapon_editor.edit_history.can_undo());

        let from_disk = WeaponItem {
            name: "From Disk".to_string(),
            ..Default::default()
        };
        let _ = handle(
            StoreEditorMessage::Scanned(Ok((
                Some(vec![from_disk]),
                Some(vec![]),
                Some(vec![]),
                Some(vec![]),
                vec![Store::default()],
                vec![],
            ))),
            &mut app,
        );

        let catalog = app.state.editors.weapon_editor.catalog.as_ref().unwrap();
        assert_eq!(
            catalog[0].name, "Unsaved Name",
            "scan must not discard an unsaved edit"
        );
    }

    #[test]
    fn test_store_scan_tolerates_missing_item_file_with_warning() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let char_dir = tmp.path().join("CharacterInGame");
        std::fs::create_dir_all(&char_dir).expect("create CharacterInGame");

        // STORE.DB is valid and the other three item catalogs are readable;
        // only weaponItem.db is missing.
        Store::save_file(&[Store::default()], &char_dir.join("STORE.DB")).expect("write STORE.DB");
        dispel_core::HealItem::save_file(&[], &char_dir.join("HealItem.db"))
            .expect("write HealItem.db");
        dispel_core::MiscItem::save_file(&[], &char_dir.join("MiscItem.db"))
            .expect("write MiscItem.db");
        dispel_core::EditItem::save_file(&[], &char_dir.join("EditItem.db"))
            .expect("write EditItem.db");

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime");
        let (weapons, heals, misc, edit, stores, warnings) = runtime
            .block_on(scan_store_catalog(
                char_dir.join("weaponItem.db"),
                char_dir.join("HealItem.db"),
                char_dir.join("MiscItem.db"),
                char_dir.join("EditItem.db"),
                char_dir.join("STORE.DB"),
            ))
            .expect("a missing optional item catalog must not be fatal");

        assert!(weapons.is_none());
        assert!(heals.is_some() && misc.is_some() && edit.is_some());
        assert_eq!(stores.len(), 1);
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("weaponItem.db") && !w.trim().is_empty()),
            "warning must name the missing file and a reason, got: {warnings:?}"
        );

        let mut app = App::test_new(crate::workspace::Workspace::new());
        let _ = handle(
            StoreEditorMessage::Scanned(Ok((weapons, heals, misc, edit, stores, warnings))),
            &mut app,
        );
        let status = &app.state.editors.store_editor.status_msg;
        assert!(
            status.starts_with("Loaded:") && status.contains("weaponItem.db"),
            "status must report a successful load with the missing-file warning, got: {status}"
        );
    }
}
