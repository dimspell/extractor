// Chest editor message handlers
use crate::app::App;
use crate::loading_state::LoadingState;
use crate::message::editor::chest::ChestEditorMessage;
use dispel_core::Extractor;
use iced::Task;
use std::path::PathBuf;

pub fn handle(message: ChestEditorMessage, app: &mut App) -> Task<crate::message::Message> {
    match message {
        ChestEditorMessage::ScanMaps => {
            if app.state.shared_game_path.is_empty() {
                app.state.chest_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.chest_editor.loading_state = LoadingState::Loading;
            let path = PathBuf::from(&app.state.shared_game_path).join("ExtraInGame");
            Task::perform(
                async move {
                    let mut files = vec![];
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.flatten() {
                            let p = entry.path();
                            if p.is_file()
                                && p.extension().map(|e| e == "ref").unwrap_or(false)
                                && p.file_name()
                                    .map(|n| n.to_string_lossy().starts_with("Ext"))
                                    .unwrap_or(false)
                            {
                                files.push(p.to_string_lossy().to_string());
                            }
                        }
                    }
                    files.sort();
                    Ok(files)
                },
                |res| {
                    crate::message::Message::Editor(crate::message::editor::EditorMessage::Chest(
                        ChestEditorMessage::MapsScanned(res),
                    ))
                },
            )
        }
        ChestEditorMessage::MapsScanned(res) => {
            app.state.chest_editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(files) => {
                    app.state.chest_editor.map_files =
                        files.into_iter().map(PathBuf::from).collect();
                    app.state.chest_editor.status_msg = format!(
                        "Found {} map files.",
                        app.state.chest_editor.map_files.len()
                    );
                }
                Err(e) => app.state.chest_editor.status_msg = format!("Error scanning maps: {}", e),
            }
            // Also load the catalog for human-friendly item names
            if app.state.shared_game_path.is_empty() {
                Task::none()
            } else {
                app.state.chest_editor.loading_state = LoadingState::Loading;
                let path = PathBuf::from(&app.state.shared_game_path);
                Task::perform(
                    async move { crate::state::chest_editor::ItemCatalog::load_from_folder(&path) },
                    |res| {
                        crate::message::Message::Editor(
                            crate::message::editor::EditorMessage::Chest(
                                ChestEditorMessage::CatalogLoaded(res.map_err(|e| e.to_string())),
                            ),
                        )
                    },
                )
            }
        }
        ChestEditorMessage::LoadCatalog => {
            if app.state.shared_game_path.is_empty() {
                app.state.chest_editor.status_msg = "Please select game path first.".into();
                return Task::none();
            }
            app.state.chest_editor.loading_state = LoadingState::Loading;
            let path = PathBuf::from(&app.state.shared_game_path);
            Task::perform(
                async move { crate::state::chest_editor::ItemCatalog::load_from_folder(&path) },
                |res| {
                    crate::message::Message::Editor(crate::message::editor::EditorMessage::Chest(
                        ChestEditorMessage::CatalogLoaded(res.map_err(|e| e.to_string())),
                    ))
                },
            )
        }
        ChestEditorMessage::CatalogLoaded(res) => {
            app.state.chest_editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(catalog) => {
                    app.state.chest_editor.catalog = Some(catalog);
                    app.state.chest_editor.status_msg = "Catalog loaded successfully.".into();
                }
                Err(e) => {
                    app.state.chest_editor.status_msg = format!("Error loading catalog: {}", e)
                }
            }
            Task::none()
        }
        ChestEditorMessage::SelectMap => {
            if app.state.chest_editor.current_map_file.is_empty() {
                app.state.chest_editor.status_msg = "No map file selected.".into();
                return Task::none();
            }
            app.load_map_file(PathBuf::from(&app.state.chest_editor.current_map_file))
        }
        ChestEditorMessage::SelectMapFromFile(path) => {
            app.state.chest_editor.current_map_file = path.clone();
            app.load_map_file(PathBuf::from(path))
        }
        ChestEditorMessage::MapLoaded(res) => {
            app.state.chest_editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(records) => {
                    app.state.chest_editor.all_records =
                        records.into_iter().map(|(_, record)| record).collect();
                    app.state.chest_editor.status_msg = "Map loaded successfully.".into();
                    app.refresh_chests();
                }
                Err(e) => app.state.chest_editor.status_msg = format!("Error loading map: {}", e),
            }
            Task::none()
        }
        ChestEditorMessage::SelectChest(index) => {
            app.state.chest_editor.selected_idx = Some(index);
            if let Some((_, record)) = app.state.chest_editor.filtered_chests.get(index) {
                app.state.chest_editor.edit_name = record.name.clone();
                app.state.chest_editor.edit_x = record.x_pos.to_string();
                app.state.chest_editor.edit_y = record.y_pos.to_string();
                app.state.chest_editor.edit_gold = record.gold_amount.to_string();
                app.state.chest_editor.edit_item_count = record.item_count.to_string();
                app.state.chest_editor.edit_item_id = record.item_id.to_string();
                app.state.chest_editor.edit_item_type = (u8::from(record.item_type_id)).to_string();
                app.state.chest_editor.edit_closed = record.closed.to_string();
            }
            Task::none()
        }
        ChestEditorMessage::FieldChanged(orig_idx, field, val) => {
            match field.as_str() {
                "name" => app.state.chest_editor.edit_name = val.clone(),
                "x" => app.state.chest_editor.edit_x = val.clone(),
                "y" => app.state.chest_editor.edit_y = val.clone(),
                "gold" => app.state.chest_editor.edit_gold = val.clone(),
                "item_count" => app.state.chest_editor.edit_item_count = val.clone(),
                "item_id" => app.state.chest_editor.edit_item_id = val.clone(),
                "item_type" => app.state.chest_editor.edit_item_type = val.clone(),
                "closed" => app.state.chest_editor.edit_closed = val.clone(),
                _ => {}
            }
            if let Some(record) = app.state.chest_editor.all_records.get_mut(orig_idx) {
                match field.as_str() {
                    "name" => record.name = val,
                    "x" => {
                        if let Ok(v) = val.parse() {
                            record.x_pos = v
                        }
                    }
                    "y" => {
                        if let Ok(v) = val.parse() {
                            record.y_pos = v
                        }
                    }
                    "gold" => {
                        if let Ok(v) = val.parse() {
                            record.gold_amount = v
                        }
                    }
                    "item_count" => {
                        if let Ok(v) = val.parse() {
                            record.item_count = v
                        }
                    }
                    "item_id" => {
                        if let Ok(v) = val.parse() {
                            record.item_id = v
                        }
                    }
                    "item_type" => {
                        if let Some(t) = dispel_core::ItemTypeId::from_name(&val) {
                            record.item_type_id = t;
                        }
                    }
                    "closed" => {
                        if let Ok(v) = val.parse() {
                            record.closed = v
                        }
                    }
                    _ => {}
                }
                app.refresh_chests();
            }
            Task::none()
        }
        ChestEditorMessage::Save => {
            if app.state.chest_editor.current_map_file.is_empty()
                || app.state.chest_editor.all_records.is_empty()
            {
                return Task::none();
            }
            app.state.chest_editor.loading_state = LoadingState::Loading;

            let path = PathBuf::from(&app.state.chest_editor.current_map_file);

            // Copy the original file with a timestamp (before file extension) as a backup
            if path.exists() {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let ext = path.extension().unwrap_or_default().to_string_lossy();

                let mut backup_path = path.clone();
                backup_path.set_file_name(format!("{}_{}.{}", stem, timestamp, ext));

                if let Err(e) = std::fs::copy(&path, &backup_path) {
                    return Task::perform(
                        async move { Err(format!("Failed to backup: {}", e)) },
                        |res| {
                            crate::message::Message::Editor(
                                crate::message::editor::EditorMessage::Chest(
                                    ChestEditorMessage::Saved(res),
                                ),
                            )
                        },
                    );
                }
            }

            let records = app.state.chest_editor.all_records.clone();
            Task::perform(
                async move { dispel_core::ExtraRef::save_file(&records, &path) },
                |res: Result<(), std::io::Error>| {
                    crate::message::Message::Editor(crate::message::editor::EditorMessage::Chest(
                        ChestEditorMessage::Saved(res.map_err(|e| e.to_string())),
                    ))
                },
            )
        }
        ChestEditorMessage::Saved(res) => {
            app.state.chest_editor.loading_state = LoadingState::Loaded(());
            match res {
                Ok(_) => app.state.chest_editor.status_msg = "Map saved successfully.".into(),
                Err(e) => app.state.chest_editor.status_msg = format!("Error saving map: {}", e),
            }
            Task::none()
        }
        ChestEditorMessage::Add => Task::none(),
        ChestEditorMessage::Delete(_) => Task::none(),
    }
}
