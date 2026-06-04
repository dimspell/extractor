use crate::app::App;
use crate::editors::map_editor::MapEditorMessage;
use crate::message::{Message, MessageExt};
use dispel_core::map::writer::write_map_to_path;
use dispel_core::references::extractor::Extractor;
use iced::Task;

pub fn save_entities(app: &mut App, tab_id: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    if state.data.is_saving {
        return Task::none();
    }
    state.data.is_saving = true;
    let monsters = state.data.monsters.clone();
    let npcs = state.data.npcs.clone();
    let extra_refs = state.data.extra_refs.clone();
    let monster_path = state.data.monster_ref_path.clone();
    let npc_path = state.data.npc_ref_path.clone();
    let extra_path = state.data.extra_ref_path.clone();

    Task::perform(
        async move {
            let mut saved: Vec<String> = Vec::new();
            let mut errors: Vec<String> = Vec::new();

            macro_rules! save_type {
                ($T:ty, $records:expr, $path:expr) => {
                    if let Some(p) = $path {
                        match <$T>::save_file($records, &p) {
                            Ok(()) => saved.push(
                                p.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| p.display().to_string()),
                            ),
                            Err(e) => errors.push(format!(
                                "{}: {}",
                                p.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                                e
                            )),
                        }
                    }
                };
            }
            save_type!(dispel_core::MonsterRef, &monsters, monster_path);
            save_type!(dispel_core::NPC, &npcs, npc_path);
            save_type!(dispel_core::ExtraRef, &extra_refs, extra_path);

            if !errors.is_empty() {
                Err(errors.join("; "))
            } else if saved.is_empty() {
                Err("No entity files found to save".to_string())
            } else {
                Ok(format!("Saved: {}", saved.join(", ")))
            }
        },
        move |result| Message::map_editor(MapEditorMessage::SaveComplete(tab_id, result)),
    )
}

pub fn save_map(app: &mut App, tab_id: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    if state.data.is_saving {
        return Task::none();
    }
    state.data.is_saving = true;

    // Capture everything needed for save.
    let map_path = match state.data.map_path.clone() {
        Some(p) => p,
        None => return Task::none(),
    };
    let map_handle = match state.map_data() {
        Some(h) => h.clone(),
        None => return Task::none(),
    };
    let monsters = state.data.monsters.clone();
    let npcs = state.data.npcs.clone();
    let extra_refs = state.data.extra_refs.clone();
    let monster_path = state.data.monster_ref_path.clone();
    let npc_path = state.data.npc_ref_path.clone();
    let extra_path = state.data.extra_ref_path.clone();

    Task::perform(
        async move {
            let mut saved: Vec<String> = Vec::new();
            let mut errors: Vec<String> = Vec::new();

            // Save entity .ref files.
            macro_rules! save_type {
                ($T:ty, $records:expr, $path:expr) => {
                    if let Some(p) = $path {
                        match <$T>::save_file($records, &p) {
                            Ok(()) => saved.push(
                                p.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| p.display().to_string()),
                            ),
                            Err(e) => errors.push(format!(
                                "{}: {}",
                                p.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_default(),
                                e
                            )),
                        }
                    }
                };
            }
            save_type!(dispel_core::MonsterRef, &monsters, monster_path);
            save_type!(dispel_core::NPC, &npcs, npc_path);
            save_type!(dispel_core::ExtraRef, &extra_refs, extra_path);

            // Save .map binary (collisions + events).
            match write_map_to_path(&map_path, &map_handle.0) {
                Ok(()) => saved.push(
                    map_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "map".to_string()),
                ),
                Err(e) => errors.push(format!(
                    "{}: {}",
                    map_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    e
                )),
            }

            if !errors.is_empty() {
                Err(errors.join("; "))
            } else if saved.is_empty() {
                Err("Nothing to save".to_string())
            } else {
                Ok(format!("Saved: {}", saved.join(", ")))
            }
        },
        move |result| Message::map_editor(MapEditorMessage::MapSaved(tab_id, result)),
    )
}

pub fn save_complete(
    app: &mut App,
    tab_id: usize,
    result: Result<String, String>,
) -> Task<Message> {
    let success = result.is_ok();
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.is_saving = false;
        match result {
            Ok(msg) => {
                state.data.dirty = false;
                state.data.status_msg = Some(msg);
            }
            Err(e) => {
                state.data.status_msg = Some(format!("Save failed: {e}"));
            }
        }
    }
    if success {
        super::set_tab_modified(app, tab_id, false);
        super::dismiss_status_after(tab_id)
    } else {
        Task::none()
    }
}

pub fn map_saved(
    app: &mut App,
    tab_id: usize,
    result: Result<String, String>,
) -> Task<Message> {
    let success = result.is_ok();
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.is_saving = false;
        match result {
            Ok(msg) => {
                state.data.dirty = false;
                state.data.status_msg = Some(msg);
            }
            Err(e) => {
                state.data.status_msg = Some(format!("Save failed: {e}"));
            }
        }
    }
    if success {
        super::set_tab_modified(app, tab_id, false);
        super::dismiss_status_after(tab_id)
    } else {
        Task::none()
    }
}

pub fn export_image(app: &mut App, tab_id: usize) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };
    if state.data.is_exporting {
        return Task::none();
    }
    state.data.is_exporting = true;
    let state = &*state;
    let map_path = match &state.data.map_path {
        Some(p) => p.clone(),
        None => return Task::none(),
    };
    let gtl_path = match &state.data.gtl_path {
        Some(p) => p.clone(),
        None => map_path.with_extension("gtl"),
    };
    let btl_path = match &state.data.btl_path {
        Some(p) => p.clone(),
        None => map_path.with_extension("btl"),
    };
    let Some(map_handle) = state.map_data() else {
        return Task::none();
    };
    let map_data = map_handle.0.clone();
    let game_path = app.state.workspace.game_path.clone();

    Task::perform(
        async move {
            let file_handle = rfd::AsyncFileDialog::new()
                .set_title("Export map as PNG")
                .add_filter("PNG Image", &["png"])
                .set_file_name(
                    map_path
                        .file_stem()
                        .map(|s| format!("{}.png", s.to_string_lossy()))
                        .unwrap_or_else(|| "map.png".to_string())
                        .as_str(),
                )
                .save_file()
                .await;

            let Some(file_handle) = file_handle else {
                return Ok("Export cancelled".to_string());
            };
            let output_path = file_handle.path().to_path_buf();

            let gtl_tiles = dispel_core::map::tileset::extract(&gtl_path)
                .map_err(|e| format!("GTL read failed: {e}"))?;
            let btl_tiles = dispel_core::map::tileset::extract(&btl_path)
                .map_err(|e| format!("BTL read failed: {e}"))?;

            let file = std::fs::File::open(&map_path)
                .map_err(|e| format!("Map open failed: {e}"))?;
            let mut reader = std::io::BufReader::new(file);

            let map_id = map_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            dispel_core::map::render::render_map(
                dispel_core::map::render::MapRenderConfig {
                    reader: &mut reader,
                    output_path: &output_path,
                    data: &map_data,
                    occlusion: false,
                    gtl_tileset: &gtl_tiles,
                    btl_tileset: &btl_tiles,
                    map_id: &map_id,
                    game_path: game_path.as_deref(),
                },
            )
            .map_err(|e| format!("Render failed: {e}"))?;

            Ok(format!(
                "Exported to {}",
                output_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| output_path.display().to_string())
            ))
        },
        move |result| {
            Message::map_editor(MapEditorMessage::ExportComplete(tab_id, result))
        },
    )
}

pub fn export_complete(
    app: &mut App,
    tab_id: usize,
    result: Result<String, String>,
) -> Task<Message> {
    let success = result.is_ok();
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.is_exporting = false;
        state.data.status_msg = Some(match result {
            Ok(msg) => msg,
            Err(e) => format!("Export failed: {e}"),
        });
    }
    if success {
        super::dismiss_status_after(tab_id)
    } else {
        Task::none()
    }
}
