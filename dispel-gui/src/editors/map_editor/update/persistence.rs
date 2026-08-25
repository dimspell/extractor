use crate::app::App;
use crate::editors::map_editor::MapEditorMessage;
use crate::editors::mod_packager::recording::record_file_replace;
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
    let draw_items = state.data.draw_items.clone();
    let all_map_id = state.data.all_map_id;
    let game_path = app.state.workspace.game_path.clone();
    let monster_path = state.data.monster_ref_path.clone();
    let npc_path = state.data.npc_ref_path.clone();
    let extra_path = state.data.extra_ref_path.clone();

    // Capture recording session info for recording integration.
    let recording_info: Option<(std::path::PathBuf, String)> = app
        .state
        .recording
        .as_ref()
        .map(|s| (s.workspace_root.clone(), s.mod_slug.clone()));

    Task::perform(
        async move {
            let mut saved: Vec<String> = Vec::new();
            let mut errors: Vec<String> = Vec::new();

            // ── Pre-commit validation ────────────────────────────────────────
            if !map_path.exists() {
                errors.push(format!("Map file not found: {}", map_path.display()));
            }

            // Validate map file size is compatible with expected end-block size.
            let w = map_handle.0.model.tiled_map_width;
            let h = map_handle.0.model.tiled_map_height;
            let expected_blocks = (w * h * 4) as u64 * 3;
            match std::fs::metadata(&map_path) {
                Ok(meta) => {
                    let file_len = meta.len();
                    if file_len < expected_blocks {
                        errors.push(format!(
                            "Map file too small: {} bytes, need at least {} for {w}×{h} map",
                            file_len, expected_blocks
                        ));
                    }
                }
                Err(e) => {
                    errors.push(format!("Cannot read map file metadata: {e}"));
                }
            }

            // Note: event_id is i16 so range is guaranteed by the type. No
            // explicit range validation needed here.
            // Other potential checks: verify vanilla file size hasn't changed
            // since load (would require storing original_len on MapDataState).

            // ── Save entity .ref files ───────────────────────────────────────
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

            // ── Save DrawItems ──────────────────────────────────────────────
            if let Some(map_id) = all_map_id
                && let Some(ref gp) = game_path
            {
                let draw_item_path = gp.join("Ref").join("DRAWITEM.ref");
                match dispel_core::DrawItem::read_file(&draw_item_path) {
                    Ok(mut all) => {
                        all.retain(|d| d.map_id != map_id);
                        all.extend(draw_items.iter().cloned());
                        match dispel_core::DrawItem::save_file(&all, &draw_item_path) {
                            Ok(()) => saved.push("DRAWITEM.ref".into()),
                            Err(e) => errors.push(format!("DRAWITEM.ref: {e}")),
                        }
                    }
                    Err(e) => errors.push(format!("DRAWITEM.ref read: {e}")),
                }
            }

            // ── Save .map binary (collisions + events) with recording ────────
            if errors.is_empty() {
                // Read old bytes before write (for recording delta).
                let old_map_bytes = std::fs::read(&map_path).ok();

                match write_map_to_path(&map_path, &map_handle.0) {
                    Ok(()) => {
                        let map_name = map_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "map".to_string());
                        saved.push(map_name);

                        // Recording integration — append ChangeAction
                        // to the active mod workspace when recording is active.
                        if let Some((ref ws_root, ref mod_slug)) = recording_info
                            && let Some(ref game_dir) = game_path
                            && let Some(old) = old_map_bytes
                            && let Ok(new_bytes) = std::fs::read(&map_path)
                            && old != new_bytes
                        {
                            let relative = map_path
                                .strip_prefix(game_dir)
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_else(|_| {
                                    map_path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_default()
                                });
                            if let Err(e) = record_file_replace(
                                ws_root, game_dir, mod_slug, &relative, &new_bytes,
                            ) {
                                errors.push(format!("Recording: {e}"));
                            } else {
                                saved.push(format!("→ recorded in `{mod_slug}`"));
                            }
                        }
                    }
                    Err(e) => errors.push(format!(
                        "{}: {}",
                        map_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                        e
                    )),
                }
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
                state.data.notify(
                    gui_widgets::components::toast::Status::Success,
                    "Saved",
                    msg,
                );
            }
            Err(e) => {
                state
                    .data
                    .notify(gui_widgets::components::toast::Status::Danger, "Error", e);
            }
        }
    }
    if success {
        super::set_tab_modified(app, tab_id, false);
    }
    Task::none()
}

pub fn map_saved(app: &mut App, tab_id: usize, result: Result<String, String>) -> Task<Message> {
    let success = result.is_ok();
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.is_saving = false;
        match result {
            Ok(msg) => {
                state.data.dirty = false;
                state.data.notify(
                    gui_widgets::components::toast::Status::Success,
                    "Saved",
                    msg,
                );
            }
            Err(e) => {
                state
                    .data
                    .notify(gui_widgets::components::toast::Status::Danger, "Error", e);
            }
        }
    }
    if success {
        super::set_tab_modified(app, tab_id, false);
    }
    Task::none()
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

            let file =
                std::fs::File::open(&map_path).map_err(|e| format!("Map open failed: {e}"))?;
            let mut reader = std::io::BufReader::new(file);

            let map_id = map_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            dispel_core::map::render::render_map(dispel_core::map::render::MapRenderConfig {
                reader: &mut reader,
                output_path: &output_path,
                data: &map_data,
                gtl_tileset: &gtl_tiles,
                btl_tileset: &btl_tiles,
                map_id: &map_id,
                game_path: game_path.as_deref(),
                toggles: Default::default(),
                lights: &[],
            })
            .map_err(|e| format!("Render failed: {e}"))?;

            Ok(format!(
                "Exported to {}",
                output_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| output_path.display().to_string())
            ))
        },
        move |result| Message::map_editor(MapEditorMessage::ExportComplete(tab_id, result)),
    )
}

pub fn export_complete(
    app: &mut App,
    tab_id: usize,
    result: Result<String, String>,
) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.is_exporting = false;
        match result {
            Ok(msg) => {
                state.data.notify(
                    gui_widgets::components::toast::Status::Success,
                    "Export",
                    msg,
                );
            }
            Err(e) => {
                state
                    .data
                    .notify(gui_widgets::components::toast::Status::Danger, "Error", e);
            }
        }
    }
    Task::none()
}

#[cfg(test)]
mod tests {
    use dispel_core::DrawItem;
    use dispel_core::references::extractor::Extractor;

    /// Helper: create a temp DRAWITEM.ref with known items for two maps.
    fn create_fixture(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("DRAWITEM.ref");
        let items = vec![
            DrawItem {
                map_id: 0,
                x_coord: 10,
                y_coord: 20,
                item: dispel_core::InventoryItem::new(
                    dispel_core::references::enums::ItemTypeId::Event,
                    1,
                ),
            },
            DrawItem {
                map_id: 0,
                x_coord: 30,
                y_coord: 40,
                item: dispel_core::InventoryItem::new(
                    dispel_core::references::enums::ItemTypeId::Event,
                    2,
                ),
            },
            DrawItem {
                map_id: 10,
                x_coord: 50,
                y_coord: 60,
                item: dispel_core::InventoryItem::new(
                    dispel_core::references::enums::ItemTypeId::Weapon,
                    3,
                ),
            },
        ];
        DrawItem::save_file(&items, &path).expect("save fixture");
        path
    }

    fn read_all(path: &std::path::Path) -> Vec<DrawItem> {
        DrawItem::read_file(path).expect("read draw items")
    }

    #[test]
    fn test_draw_item_save_merge_replaces_map_items() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = create_fixture(dir.path());

        // Simulate the merge-save logic for map 0
        let mut all = read_all(&path);
        all.retain(|d| d.map_id != 0);
        all.push(DrawItem {
            map_id: 0,
            x_coord: 99,
            y_coord: 20,
            item: dispel_core::InventoryItem::new(
                dispel_core::references::enums::ItemTypeId::Event,
                1,
            ),
        });
        all.push(DrawItem {
            map_id: 0,
            x_coord: 30,
            y_coord: 88,
            item: dispel_core::InventoryItem::new(
                dispel_core::references::enums::ItemTypeId::Event,
                2,
            ),
        });
        DrawItem::save_file(&all, &path).expect("save merged");

        let result = read_all(&path);
        assert_eq!(result.len(), 3, "merged file has 3 items");

        let map0: Vec<&DrawItem> = result.iter().filter(|d| d.map_id == 0).collect();
        assert_eq!(map0.len(), 2, "map 0 has 2 items");
        assert_eq!(map0[0].x_coord, 99, "first item x_coord updated");
        assert_eq!(map0[1].y_coord, 88, "second item y_coord updated");

        let map10: Vec<&DrawItem> = result.iter().filter(|d| d.map_id == 10).collect();
        assert_eq!(map10.len(), 1, "map 10 has 1 item");
        assert_eq!(map10[0].x_coord, 50, "map 10 x_coord unchanged");
    }

    #[test]
    fn test_draw_item_save_merge_removes_all_entries_for_map() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = create_fixture(dir.path());

        let mut all = read_all(&path);
        all.retain(|d| d.map_id != 0);
        DrawItem::save_file(&all, &path).expect("save merged");

        let result = read_all(&path);
        assert_eq!(result.len(), 1, "only map 10 item remains");
        assert_eq!(result[0].map_id, 10);
    }

    #[test]
    fn test_draw_item_save_merge_adds_entries_for_new_map() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = create_fixture(dir.path());

        let mut all = read_all(&path);
        all.retain(|d| d.map_id != 5);
        all.push(DrawItem {
            map_id: 5,
            x_coord: 1,
            y_coord: 2,
            item: dispel_core::InventoryItem::new(
                dispel_core::references::enums::ItemTypeId::Healing,
                99,
            ),
        });
        DrawItem::save_file(&all, &path).expect("save merged");

        let result = read_all(&path);
        assert_eq!(result.len(), 4, "original 3 + new 1 = 4");
        assert_eq!(result.iter().filter(|d| d.map_id == 5).count(), 1);
    }

    #[test]
    fn test_draw_item_save_merge_round_trip_preserves_encoding() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let path = create_fixture(dir.path());

        let original = read_all(&path);
        DrawItem::save_file(&original, &path).expect("save unmodified");
        let after = read_all(&path);

        assert_eq!(original.len(), after.len());
        for (a, b) in original.iter().zip(after.iter()) {
            assert_eq!(a.map_id, b.map_id);
            assert_eq!(a.x_coord, b.x_coord);
            assert_eq!(a.y_coord, b.y_coord);
            assert_eq!(a.item.raw(), b.item.raw());
        }
    }
}
