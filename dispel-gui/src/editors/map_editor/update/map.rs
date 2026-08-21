use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::map_editor::canvas::decode_tileset_file;
use crate::editors::map_editor::{
    DecodedEntitySprite, DecodedMapSprite, EntityBundle, EntitySpriteHandle, InternalSpriteHandle,
    MapDataHandle, MapEditorMessage, SpriteSequenceHandle, TilePixelData,
};
use crate::message::{Message, MessageExt};
use dispel_core::references::extractor::Extractor;
use iced::Task;
use iced::widget::image::Handle;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

// ── Message handlers ─────────────────────────────────────────────────────────

use crate::components::map_render::view_state::MapViewState;
use crate::editors::map_editor::message::{MapLayer, MapTool, ObjectBrushMode, SelectedEntity};
use crate::editors::map_editor::state::MapEditAction;

/// Whether a display layer is currently visible.
pub fn layer_visible(view: &MapViewState, layer: MapLayer) -> bool {
    match layer {
        MapLayer::Ground => view.show_ground,
        MapLayer::Buildings => view.show_buildings,
        MapLayer::Roofs => view.show_roofs,
        MapLayer::InternalSprites => view.show_internal_sprites,
        MapLayer::Collisions => view.show_collisions,
        MapLayer::Events => view.show_events,
        MapLayer::Monsters => view.show_monsters,
        MapLayer::Npcs => view.show_npcs,
        MapLayer::NpcWaypoints => view.show_npc_waypoints,
        MapLayer::Objects => view.show_objects,
        MapLayer::DrawItems => view.show_draw_items,
        MapLayer::ObjectIds => view.show_object_ids,
    }
}

fn set_layer_visible(view: &mut MapViewState, layer: MapLayer, visible: bool) {
    match layer {
        MapLayer::Ground => view.show_ground = visible,
        MapLayer::Buildings => view.show_buildings = visible,
        MapLayer::Roofs => view.show_roofs = visible,
        MapLayer::InternalSprites => view.show_internal_sprites = visible,
        MapLayer::Collisions => view.show_collisions = visible,
        MapLayer::Events => view.show_events = visible,
        MapLayer::Monsters => view.show_monsters = visible,
        MapLayer::Npcs => view.show_npcs = visible,
        MapLayer::NpcWaypoints => view.show_npc_waypoints = visible,
        MapLayer::Objects => view.show_objects = visible,
        MapLayer::DrawItems => view.show_draw_items = visible,
        MapLayer::ObjectIds => view.show_object_ids = visible,
    }
}

/// Select the active canvas tool. If the tool's owning layer is hidden,
/// force-enable it (visibility ≠ editability: selecting an editing tool
/// implies you need to see what you edit).
pub fn select_tool(app: &mut App, tab_id: usize, tool: MapTool) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        if let Some(layer) = tool.owning_layer()
            && !layer_visible(&state.view, layer)
        {
            set_layer_visible(&mut state.view, layer, true);
            state.view.overlay_cache.clear();
            state.view.tile_layer_cache.clear();
        }
        state.view.active_tool = tool;
    }
    Task::none()
}

/// Toggle the collision flag on a tile. Returns `true` when the map changed.
pub fn toggle_collision_at(
    state: &mut crate::editors::map_editor::state::MapEditorState,
    tx: i32,
    ty: i32,
) -> bool {
    if !state.data.can_mutate_map_data() {
        state.data.notify(
            gui_widgets::components::toast::Status::Warning,
            "Collision",
            "Cannot edit while save/export is in progress",
        );
        return false;
    }
    let LoadingState::Loaded(ref mut handle) = state.data.loading_state else {
        return false;
    };
    let map_data =
        Arc::get_mut(&mut handle.0).expect("MapData Arc has unexpected shared reference");
    let old = map_data.collisions.get(&(tx, ty)).copied().unwrap_or(false);
    map_data.collisions.insert((tx, ty), !old);
    state.push_undo(MapEditAction {
        entity: SelectedEntity::CollisionTile(tx, ty),
        field: "collision".into(),
        old_value: old.to_string(),
        new_value: (!old).to_string(),
    });
    state.view.selected_entity = None;
    state.view.overlay_cache.clear();
    // Replace-in-place: painting a stroke updates one toast, not many.
    state.data.notify_replace(
        "Collision",
        gui_widgets::components::toast::Status::Primary,
        "Collision",
        format!(
            "{} ({},{})",
            if old { "Unblocked" } else { "Blocked" },
            tx,
            ty
        ),
    );
    true
}

/// Apply the object-id brush to a tile according to the current brush mode.
///
/// Paint writes the brush value (overwriting whatever was there); Erase removes
/// the entry regardless of its current value. Returns `true` when changed.
pub fn apply_object_id_edit(
    state: &mut crate::editors::map_editor::state::MapEditorState,
    tx: i32,
    ty: i32,
) -> bool {
    if !state.data.can_mutate_map_data() {
        state.data.notify(
            gui_widgets::components::toast::Status::Warning,
            "Object ID",
            "Cannot edit while save/export is in progress",
        );
        return false;
    }
    let LoadingState::Loaded(ref mut handle) = state.data.loading_state else {
        return false;
    };
    let erase = state.view.object_brush_mode == ObjectBrushMode::Erase;
    let brush = state.data.object_brush.clamp(1, 511);
    let map_data =
        Arc::get_mut(&mut handle.0).expect("MapData Arc has unexpected shared reference");
    let old = map_data.object_ids.get(&(tx, ty)).copied().unwrap_or(0);
    let new = if erase { 0 } else { brush };
    if new == 0 {
        map_data.object_ids.remove(&(tx, ty));
    } else {
        map_data.object_ids.insert((tx, ty), new);
    }
    state.push_undo(MapEditAction {
        entity: SelectedEntity::ObjectIdTile(tx, ty),
        field: "object_id".into(),
        old_value: old.to_string(),
        new_value: new.to_string(),
    });
    state.view.selected_entity = None;
    state.view.overlay_cache.clear();
    state.view.tile_layer_cache.clear();
    // Replace-in-place: painting a stroke updates one toast, not many.
    let body = if erase || new == 0 {
        format!("Erased obj @ ({},{})", tx, ty)
    } else {
        format!("Obj {} → ({},{})", new, tx, ty)
    };
    state.data.notify_replace(
        "Object ID",
        gui_widgets::components::toast::Status::Primary,
        "Object ID",
        body,
    );
    true
}

pub fn open(app: &mut App, tab_id: usize, path: PathBuf) -> Task<Message> {
    // Ensure item lookups are loaded for composite-item pickers in the
    // entity-inspector panels (MonsterRef, NPC, ExtraRef).
    crate::components::item_catalog::ensure_item_lookups(
        &app.state.shared_game_path,
        &mut app.state.lookups,
    );
    // Ensure monster-name lookups are loaded for the `mon_id` field's
    // pick_list in the Monster inspector.
    if !app.state.shared_game_path.is_empty()
        && !app.state.lookups.contains_key("monster_names")
        && let Ok(monsters) = dispel_core::references::monster_ini::MonsterIni::read_file(
            &std::path::PathBuf::from(&app.state.shared_game_path).join("Monster.ini"),
        )
    {
        let names: Vec<(String, String)> = monsters
            .iter()
            .map(|m| (m.id.to_string(), m.name.clone().unwrap_or_default()))
            .collect();
        app.state.lookups.insert("monster_names".to_string(), names);
    }

    let state = app.state.editors.map_editors.entry(tab_id).or_default();
    state.data.map_path = Some(path.clone());
    state.data.loading_state = LoadingState::Loading;
    state.data.tiles_ready = false;
    state.data.internal_sprite_handles.clear();
    state.data.sprite_sequence_handles.clear();
    state.view.view_mode = crate::editors::map_editor::MapViewMode::Map;
    state.view.selected_sprite_sequence = None;
    state.view.tile_layer_cache.clear();
    state.view.overlay_cache.clear();

    Task::perform(
        async move {
            let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
            let mut reader = std::io::BufReader::new(file);
            let map_data =
                dispel_core::map::read_map_data(&mut reader).map_err(|e| e.to_string())?;

            // While the file is still open, decode internal sprites (thrones, etc.)
            let sprites = decode_internal_map_sprites(&mut reader, &map_data);

            Ok((MapDataHandle(Arc::new(map_data)), sprites))
        },
        move |result| Message::map_editor(MapEditorMessage::MapLoaded(tab_id, result)),
    )
}

pub fn map_loaded(
    app: &mut App,
    tab_id: usize,
    result: Result<(MapDataHandle, Vec<DecodedMapSprite>), String>,
) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };

    match result {
        Ok((handle, decoded_sprites)) => {
            let arc_data = handle.0.clone();
            let nox = arc_data.model.map_non_occluded_start_x;
            let noy = arc_data.model.map_non_occluded_start_y;

            // Build per-sequence thumbnail data for the Sprites browser.
            // decoded_sprites is parallel to arc_data.sprite_blocks.
            let mut seq_first: std::collections::HashMap<usize, (u32, u32, Vec<u8>)> =
                std::collections::HashMap::new();
            let mut seq_placements: std::collections::HashMap<usize, Vec<(i32, i32)>> =
                std::collections::HashMap::new();
            for (i, sprite) in decoded_sprites.iter().enumerate() {
                if let Some(block) = arc_data.sprite_blocks.get(i) {
                    let sid = block.sprite_id;
                    seq_first
                        .entry(sid)
                        .or_insert_with(|| (sprite.width, sprite.height, sprite.pixels.clone()));
                    seq_placements
                        .entry(sid)
                        .or_default()
                        .push((block.sprite_x, block.sprite_y));
                }
            }
            let mut seq_handles: Vec<SpriteSequenceHandle> = seq_first
                .into_iter()
                .map(|(sid, (w, h, pixels))| SpriteSequenceHandle {
                    sequence_idx: sid,
                    handle: Handle::from_rgba(w, h, pixels),
                    width: w,
                    height: h,
                    placement_count: seq_placements.get(&sid).map(|v| v.len()).unwrap_or(0),
                    placements: seq_placements.remove(&sid).unwrap_or_default(),
                })
                .collect();
            seq_handles.sort_by_key(|s| s.sequence_idx);
            state.data.sprite_sequence_handles = seq_handles;

            let loaded_name = state
                .data
                .map_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            state.data.notify(
                gui_widgets::components::toast::Status::Success,
                "Loaded",
                loaded_name,
            );

            // Convert decoded internal sprites → Iced Handles.
            state.data.internal_sprite_handles = decoded_sprites
                .into_iter()
                .map(|s| InternalSpriteHandle {
                    x: s.x + nox,
                    y: s.y + noy,
                    sort_y: s.bottom_right_y,
                    handle: Handle::from_rgba(s.width, s.height, s.pixels),
                    width: s.width,
                    height: s.height,
                })
                .collect();

            state.data.loading_state = LoadingState::Loaded(handle);
            state.view.tile_layer_cache.clear();
            state.view.overlay_cache.clear();

            let map_path = match &state.data.map_path {
                Some(p) => p.clone(),
                None => return Task::none(),
            };
            let gtl_path = map_path.with_extension("gtl");
            let btl_path = map_path.with_extension("btl");
            state.data.gtl_path = Some(gtl_path.clone());
            state.data.btl_path = Some(btl_path.clone());

            Task::perform(
                async move {
                    let gtl_ids: HashSet<i32> = arc_data.gtl_tiles.values().copied().collect();
                    let btl_ids: HashSet<i32> = arc_data
                        .btl_tiles
                        .values()
                        .copied()
                        .chain(
                            arc_data
                                .tiled_infos
                                .iter()
                                .flat_map(|t| t.ids.iter().map(|&id| id.unsigned_abs() as i32)),
                        )
                        .filter(|&id| id > 0)
                        .collect();

                    let gtl = decode_tileset_file(&gtl_path, &gtl_ids).unwrap_or_default();
                    let btl = decode_tileset_file(&btl_path, &btl_ids).unwrap_or_default();

                    Ok(TilePixelData { gtl, btl })
                },
                move |result| Message::map_editor(MapEditorMessage::TilesDecoded(tab_id, result)),
            )
        }
        Err(e) => {
            state.data.loading_state = LoadingState::Failed(e.clone());
            Task::done(Message::System(crate::message::SystemMessage::ShowError(
                format!("Failed to load map: {}", e),
            )))
        }
    }
}

pub fn tiles_decoded(
    app: &mut App,
    tab_id: usize,
    result: Result<TilePixelData, String>,
) -> Task<Message> {
    let state = match app.state.editors.map_editors.get_mut(&tab_id) {
        Some(s) => s,
        None => return Task::none(),
    };

    match result {
        Ok(pixel_data) => {
            state.data.gtl_handles = pixel_data
                .gtl
                .into_iter()
                .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                .collect();
            state.data.btl_handles = pixel_data
                .btl
                .into_iter()
                .map(|(id, px)| (id, Handle::from_rgba(62, 32, px)))
                .collect();
            state.data.tiles_ready = true;
            state.view.tile_layer_cache.clear();
            state.view.overlay_cache.clear();
        }
        Err(e) => {
            eprintln!("Tile decode failed for tab {}: {}", tab_id, e);
        }
    }

    // Centre the map at 100% zoom using the last known canvas size (defaults
    // to 1200×800 until the user moves the mouse over the canvas).
    let center = state.map_data().map(|h| {
        let model = &h.0.model;
        let diagonal = model.tiled_map_width + model.tiled_map_height;
        let (cx, cy) = dispel_core::map::types::convert_map_coords_to_image_coords(
            model.tiled_map_width / 2,
            model.tiled_map_height / 2,
            diagonal,
        );
        (cx as f32, cy as f32)
    });
    if let Some((center_px, center_py)) = center {
        let vp_w = state.view.last_canvas_w;
        let vp_h = state.view.last_canvas_h;
        state.view.zoom = 1.0;
        state.view.pan_x = vp_w / 2.0 - center_px;
        state.view.pan_y = vp_h / 2.0 - center_py;
        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
    }

    let map_path = match &state.data.map_path {
        Some(p) => p.clone(),
        None => return Task::none(),
    };
    let game_path = app.state.workspace.game_path.clone();
    if game_path.is_none() {
        state.data.notify(
            gui_widgets::components::toast::Status::Warning,
            "Load",
            "No game path set — entity files not loaded",
        );
    }
    Task::perform(
        async move { load_entities(&map_path, game_path) },
        move |bundle| Message::map_editor(MapEditorMessage::EntitiesLoaded(tab_id, bundle)),
    )
}

pub fn entities_loaded(app: &mut App, tab_id: usize, bundle: EntityBundle) -> Task<Message> {
    if let Some(state) = app.state.editors.map_editors.get_mut(&tab_id) {
        state.data.monsters = bundle.monsters;
        state.data.npcs = bundle.npcs;
        state.data.extra_refs = bundle.extra_refs;
        state.data.draw_items = bundle.draw_items;
        state.data.all_map_id = bundle.all_map_id;
        state.data.monster_ref_path = bundle.monster_ref_path;
        state.data.npc_ref_path = bundle.npc_ref_path;
        state.data.extra_ref_path = bundle.extra_ref_path;
        state.data.npc_id_to_sprite = bundle.npc_id_to_sprite;

        state.data.monster_sprites = bundle
            .monster_sprites
            .into_iter()
            .map(|opt| {
                opt.map(|s| EntitySpriteHandle {
                    handle: Handle::from_rgba(s.width, s.height, s.pixels),
                    width: s.width,
                    height: s.height,
                    origin_x: s.origin_x,
                    origin_y: s.origin_y,
                    flip: s.flip,
                })
            })
            .collect();
        state.data.npc_sprites = bundle
            .npc_sprites
            .into_iter()
            .map(|opt| {
                opt.map(|s| EntitySpriteHandle {
                    handle: Handle::from_rgba(s.width, s.height, s.pixels),
                    width: s.width,
                    height: s.height,
                    origin_x: s.origin_x,
                    origin_y: s.origin_y,
                    flip: s.flip,
                })
            })
            .collect();
        state.data.extra_sprites = bundle
            .extra_sprites
            .into_iter()
            .map(|opt| {
                opt.map(|s| EntitySpriteHandle {
                    handle: Handle::from_rgba(s.width, s.height, s.pixels),
                    width: s.width,
                    height: s.height,
                    origin_x: s.origin_x,
                    origin_y: s.origin_y,
                    flip: s.flip,
                })
            })
            .collect();

        state.view.tile_layer_cache.clear();
        state.view.overlay_cache.clear();
    }
    Task::none()
}

// ── Internal sprite decoding ──────────────────────────────────────────────────

fn decode_internal_map_sprites(
    reader: &mut std::io::BufReader<std::fs::File>,
    map_data: &dispel_core::map::MapData,
) -> Vec<DecodedMapSprite> {
    use std::io::{Read, Seek, SeekFrom};

    let mut result = Vec::new();

    for block in &map_data.sprite_blocks {
        let Some(sequence) = map_data.internal_sprites.get(block.sprite_id) else {
            continue;
        };
        let Some(frame) = sequence.frame_infos.first() else {
            continue;
        };
        if frame.width <= 0 || frame.height <= 0 {
            continue;
        }
        if reader
            .seek(SeekFrom::Start(frame.image_start_position))
            .is_err()
        {
            continue;
        }

        let w = frame.width as u32;
        let h = frame.height as u32;
        let pixel_count = (w * h) as usize;
        let mut raw = vec![0u8; pixel_count * 2];
        if reader.read_exact(&mut raw).is_err() {
            continue;
        }

        let mut pixels = vec![0u8; pixel_count * 4];
        for i in 0..pixel_count {
            let lo = raw[i * 2] as u16;
            let hi = raw[i * 2 + 1] as u16;
            let pixel = lo | (hi << 8);
            if pixel > 0 {
                let r5 = ((pixel >> 11) & 0x1F) as u32;
                let g6 = ((pixel >> 5) & 0x3F) as u32;
                let b5 = (pixel & 0x1F) as u32;
                let idx = i * 4;
                pixels[idx] = (r5 * 255 / 31) as u8;
                pixels[idx + 1] = (g6 * 255 / 63) as u8;
                pixels[idx + 2] = (b5 * 255 / 31) as u8;
                pixels[idx + 3] = 255;
            }
        }

        result.push(DecodedMapSprite {
            x: block.sprite_x,
            y: block.sprite_y,
            bottom_right_y: block.sprite_bottom_right_y,
            pixels,
            width: w,
            height: h,
        });
    }

    result
}

// ── Entity loading ────────────────────────────────────────────────────────────

/// Load entity .ref files for the given map using `Ref/Map.ini` for discovery.
///
/// Matches the map_ini entry by checking whether each entry's entity filenames
/// *contain* the map stem — e.g. "npccat1.ref" contains "cat1".  This mirrors
/// the strategy used by `render.rs::plot_external_entities`.
pub fn load_entities(
    map_path: &std::path::Path,
    game_path: Option<std::path::PathBuf>,
) -> EntityBundle {
    use dispel_core::references::extra_ini::Extra;
    use dispel_core::references::monster_ini::MonsterIni;
    use dispel_core::references::npc_ini::NpcIni;
    use dispel_core::{ExtraRef, MonsterRef, NPC};

    let stem = map_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let game_path = match game_path {
        Some(p) => p,
        None => return EntityBundle::default(),
    };

    let map_ini_path = game_path.join("Ref").join("Map.ini");
    if !map_ini_path.exists() {
        return EntityBundle::default();
    }

    let map_inis = match dispel_core::references::map_ini::read_map_ini(&map_ini_path) {
        Ok(v) => v,
        Err(_) => return EntityBundle::default(),
    };

    let map_ini = map_inis.into_iter().find(|ini| {
        ini.monsters_filename
            .as_ref()
            .is_some_and(|m| m.to_lowercase().contains(stem.as_str()))
            || ini
                .npc_filename
                .as_ref()
                .is_some_and(|n| n.to_lowercase().contains(stem.as_str()))
            || ini
                .extra_filename
                .as_ref()
                .is_some_and(|e| e.to_lowercase().contains(stem.as_str()))
    });

    let Some(map_ini) = map_ini else {
        return EntityBundle::default();
    };

    // Case-insensitive file resolution: try original → uppercase → lowercase.
    let resolve = |sub_dir: &str, filename: &str| -> Option<PathBuf> {
        for name in &[
            filename.to_string(),
            filename.to_ascii_uppercase(),
            filename.to_ascii_lowercase(),
        ] {
            let p = game_path.join(sub_dir).join(name);
            if p.exists() {
                return Some(p);
            }
        }
        None
    };

    // Build id→sprite_filename lookups from .ini files.
    let monster_id_to_sprite: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();
    let npc_id_to_sprite: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();
    let extra_id_to_sprite: HashMap<i32, String> = Extra::read_file(&game_path.join("Extra.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
        .collect();

    // Frame cache: avoid re-reading the same sprite file.
    type FrameCache =
        HashMap<PathBuf, Option<Vec<dispel_core::map::sprite_loader::LoadedSpriteFrame>>>;
    let mut sprite_cache: FrameCache = HashMap::new();

    // ── Monsters ──────────────────────────────────────────────────────────────
    let (monsters, monster_sprite_handles, monster_ref_path) = load_ref_file(
        map_ini.monsters_filename,
        "MonsterInGame",
        &resolve,
        |m: &MonsterRef| (m.monster_db_id, 3, false),
        &monster_id_to_sprite,
        &mut sprite_cache,
    );

    // ── NPCs ──────────────────────────────────────────────────────────────────
    let (npcs, npc_sprite_handles, npc_ref_path) = load_ref_file(
        map_ini.npc_filename,
        "NpcInGame",
        &resolve,
        |n: &NPC| {
            let dir = i32::from(n.waypoint1_facing_direction);
            let (seq, flip) = if dir > 4 {
                ((8 - dir) as usize, true)
            } else {
                (dir as usize, false)
            };
            (n.npc_ini_id, seq, flip)
        },
        &npc_id_to_sprite,
        &mut sprite_cache,
    );

    // ── Extra refs ────────────────────────────────────────────────────────────
    let (extra_refs, extra_sprite_handles, extra_ref_path) = load_ref_file(
        map_ini.extra_filename,
        "ExtraInGame",
        &resolve,
        |e: &ExtraRef| {
            let direction = e.direction as usize;
            let obj_type = u8::from(e.object_type) as usize;
            let seq = if obj_type == 0 {
                2 * e.interaction_state as usize + direction
            } else {
                direction
            };
            (e.extra_definition_id as i32, seq, false)
        },
        &extra_id_to_sprite,
        &mut sprite_cache,
    );

    // ── Draw items & map ID ────────────────────────────────────────
    // Resolve the current map's AllMap.ini ID and load its draw items.
    let all_map_id = resolve_map_id(&stem, &game_path);
    let draw_items = match all_map_id {
        Some(id) => {
            let all_draw_items: Vec<dispel_core::DrawItem> =
                dispel_core::DrawItem::read_file(&game_path.join("Ref").join("DRAWITEM.ref"))
                    .unwrap_or_default();
            all_draw_items
                .into_iter()
                .filter(|d| d.map_id == id)
                .collect()
        }
        None => Vec::new(),
    };

    EntityBundle {
        monsters,
        npcs,
        extra_refs,
        draw_items,
        all_map_id,
        monster_sprites: monster_sprite_handles,
        npc_sprites: npc_sprite_handles,
        extra_sprites: extra_sprite_handles,
        monster_ref_path,
        npc_ref_path,
        extra_ref_path,
        npc_id_to_sprite,
    }
}

/// Shared loader for one entity type: reads the .ref file, resolves per-entity
/// sprites, and returns (entities, sprite_handles, ref_path).
///
/// `get_id_seq_flip` derives `(sprite_lookup_id, frame_seq, flip)` for each entity.
fn load_ref_file<T: dispel_core::references::extractor::Extractor>(
    filename: Option<String>,
    subdir: &str,
    resolve: &impl Fn(&str, &str) -> Option<std::path::PathBuf>,
    get_id_seq_flip: impl Fn(&T) -> (i32, usize, bool),
    id_to_sprite: &std::collections::HashMap<i32, String>,
    sprite_cache: &mut std::collections::HashMap<
        std::path::PathBuf,
        Option<Vec<dispel_core::map::sprite_loader::LoadedSpriteFrame>>,
    >,
) -> (
    Vec<T>,
    Vec<Option<DecodedEntitySprite>>,
    Option<std::path::PathBuf>,
) {
    use dispel_core::map::sprite_loader::load_sprite_frames;

    let Some(f) = filename else {
        return (Vec::new(), Vec::new(), None);
    };
    let Some(p) = resolve(subdir, &f) else {
        return (Vec::new(), Vec::new(), None);
    };
    let Ok(data) = T::read_file(&p) else {
        return (Vec::new(), Vec::new(), None);
    };

    let ref_path = Some(p.clone());
    let sprites: Vec<Option<DecodedEntitySprite>> = data
        .iter()
        .map(|entity| {
            let (id, seq, flip) = get_id_seq_flip(entity);
            id_to_sprite
                .get(&id)
                .and_then(|spr_name| resolve(subdir, spr_name))
                .and_then(|spr_path| {
                    let frames = sprite_cache
                        .entry(spr_path.clone())
                        .or_insert_with(|| load_sprite_frames(&spr_path));
                    frames
                        .as_ref()
                        .and_then(|fs| fs.get(seq).or_else(|| fs.first()))
                        .map(|frame| decoded_from_frame(frame, flip))
                })
        })
        .collect();

    (data, sprites, ref_path)
}

/// Resolve a map file stem (e.g. "cat1") to its `AllMap.ini` numeric ID.
pub fn resolve_map_id(stem: &str, game_path: &std::path::Path) -> Option<i32> {
    use dispel_core::references::all_map_ini::Map as AllMapI;
    let all_maps = AllMapI::read_file(&game_path.join("AllMap.ini")).ok()?;
    all_maps
        .into_iter()
        .find(|m| m.map_filename.to_lowercase() == stem)
        .map(|m| m.id)
}

/// Resolve an `AllMap.ini` numeric ID to its map filename stem (e.g. `3` → `"cat1"`).
///
/// Used by the save-file map preview to locate `.map`/`.gtl`/`.btl` files
/// from a save file's `MapSectionData.map_id`.
pub fn resolve_map_filename(map_id: i32, game_path: &std::path::Path) -> Option<String> {
    use dispel_core::references::all_map_ini::Map as AllMapI;
    let all_maps = AllMapI::read_file(&game_path.join("AllMap.ini")).ok()?;
    all_maps
        .into_iter()
        .find(|m| m.id == map_id)
        .map(|m| m.map_filename)
}

/// Convert a `LoadedSpriteFrame` to `DecodedEntitySprite`.
fn decoded_from_frame(
    frame: &dispel_core::map::sprite_loader::LoadedSpriteFrame,
    flip: bool,
) -> DecodedEntitySprite {
    let w = frame.image.width();
    let h = frame.image.height();
    DecodedEntitySprite {
        pixels: frame.image.as_raw().to_vec(),
        width: w,
        height: h,
        origin_x: frame.origin_x,
        origin_y: frame.origin_y,
        flip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn game_path() -> Option<std::path::PathBuf> {
        let p = Path::new("../fixtures/Dispel");
        if p.exists() {
            Some(p.to_path_buf())
        } else {
            None
        }
    }

    // ── resolve_map_id ──────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_map_id_found() {
        let gp = match game_path() {
            Some(p) => p,
            None => {
                eprintln!("Skipping: fixtures not found");
                return;
            }
        };
        assert_eq!(resolve_map_id("cat1", &gp), Some(3));
        assert_eq!(resolve_map_id("map1", &gp), Some(0));
        assert_eq!(resolve_map_id("dun04", &gp), Some(10));
        assert_eq!(resolve_map_id("dun22", &gp), Some(28));
    }

    #[test]
    fn test_resolve_map_id_not_found() {
        let gp = match game_path() {
            Some(p) => p,
            None => {
                eprintln!("Skipping: fixtures not found");
                return;
            }
        };
        assert_eq!(resolve_map_id("nonexistent", &gp), None);
    }

    #[test]
    fn test_resolve_map_id_missing_file() {
        let tmp = std::env::temp_dir().join("dispel_test_no_allmap");
        let _ = std::fs::create_dir_all(&tmp);
        // No AllMap.ini in this dir → should return None
        assert_eq!(resolve_map_id("map1", &tmp), None);
    }

    #[test]
    fn test_resolve_map_id_empty_stem() {
        let gp = match game_path() {
            Some(p) => p,
            None => {
                eprintln!("Skipping: fixtures not found");
                return;
            }
        };
        assert_eq!(resolve_map_id("", &gp), None);
    }

    // ── load_entities ──────────────────────────────────────────────────────────

    #[test]
    fn test_load_entities_draw_items_and_map_id() {
        let gp = match game_path() {
            Some(p) => p,
            None => {
                eprintln!("Skipping: fixtures not found");
                return;
            }
        };

        // cat1 = Palace of Aesh: AllMap.ini id=3, has no draw items
        let cat1 = load_entities(&gp.join("Map/cat1.map"), Some(gp.clone()));
        assert_eq!(cat1.all_map_id, Some(3), "cat1 AllMap.ini ID");
        assert_eq!(cat1.draw_items.len(), 0, "cat1 has no draw items");

        // map1 = Aesh overworld: AllMap.ini id=0, has 2 draw items
        let map1 = load_entities(&gp.join("Map/map1.map"), Some(gp.clone()));
        assert_eq!(map1.all_map_id, Some(0), "map1 AllMap.ini ID");
        assert_eq!(map1.draw_items.len(), 2, "map1 draw items");
        // Both map1 items are event items (type 4) on the event map
        for d in &map1.draw_items {
            assert_eq!(d.map_id, 0, "all draw items belong to map1");
            assert_eq!(
                d.item.item_type(),
                Some(dispel_core::references::enums::ItemTypeId::Event)
            );
        }

        // dun04 (dungeon): AllMap.ini id=10, has 3 draw items
        let dun04 = load_entities(&gp.join("Map/dun04.map"), Some(gp.clone()));
        assert_eq!(dun04.all_map_id, Some(10), "dun04 AllMap.ini ID");
        assert_eq!(dun04.draw_items.len(), 3, "dun04 draw items");

        // dun22: AllMap.ini id=28, has 6 draw items
        let dun22 = load_entities(&gp.join("Map/dun22.map"), Some(gp.clone()));
        assert_eq!(dun22.all_map_id, Some(28), "dun22 AllMap.ini ID");
        assert_eq!(dun22.draw_items.len(), 6, "dun22 draw items");

        // non-existent map → no draw items, all_map_id = None
        let missing = load_entities(&gp.join("Map/nope.map"), Some(gp));
        assert_eq!(missing.all_map_id, None, "missing map has no ID");
        assert_eq!(missing.draw_items.len(), 0);
    }

    #[test]
    fn test_load_entities() {
        let gp = match game_path() {
            Some(p) => p,
            None => {
                eprintln!("Skipping: fixtures not found");
                return;
            }
        };

        // cat1 = Palace of Aesh: NPCs only (24), no monsters, no extras
        let cat1_entities = load_entities(&gp.join("Map/cat1.map"), Some(gp.clone()));
        assert_eq!(cat1_entities.monsters.len(), 0, "cat1 has no monsters");
        assert_eq!(cat1_entities.npcs.len(), 24, "cat1 has 24 NPCs");
        assert_eq!(cat1_entities.extra_refs.len(), 0, "cat1 has no extra refs");
        assert_eq!(
            cat1_entities.npc_sprites.len(),
            24,
            "each NPC has a sprite slot"
        );

        // map1 = Aesh overworld: monsters + NPCs + extras
        let map1_entities = load_entities(&gp.join("Map/map1.map"), Some(gp));
        assert!(
            !map1_entities.monsters.is_empty(),
            "map1 should have monsters"
        );
        assert!(!map1_entities.npcs.is_empty(), "map1 should have NPCs");
        assert!(
            !map1_entities.extra_refs.is_empty(),
            "map1 should have extra refs"
        );
        assert_eq!(
            map1_entities.monster_sprites.len(),
            map1_entities.monsters.len(),
            "sprite vec parallel to monsters"
        );
    }

    #[test]
    fn test_load_entities_no_game_path() {
        let bundle = load_entities(Path::new("Map/cat1.map"), None);
        assert_eq!(bundle.draw_items.len(), 0);
        assert_eq!(bundle.all_map_id, None);
        assert_eq!(bundle.monsters.len(), 0);
    }

    // ── Tool selection & object-id brush ──────────────────────────────────────

    use crate::app::App;
    use crate::editors::map_editor::state::MapEditorState;
    use dispel_core::map::{MapData, MapModel};

    const TAB: usize = 7;

    /// App with a map editor tab whose map data is loaded (8×8 tiles).
    fn app_with_loaded_map(object_ids: HashMap<(i32, i32), i32>) -> App {
        let mut app = App::new().0;
        let model = MapModel {
            border_count: 2,
            tiled_map_width: 8,
            tiled_map_height: 8,
            map_width_in_pixels: 16 * 32,
            map_height_in_pixels: 16 * 16,
            map_non_occluded_start_x: 0,
            map_non_occluded_start_y: 0,
            occluded_map_in_pixels_width: 8 * 64,
            occluded_map_in_pixels_height: 8 * 32,
        };
        let map_data = MapData {
            model,
            gtl_tiles: HashMap::new(),
            btl_tiles: HashMap::new(),
            access_ref_words: HashMap::new(),
            collisions: HashMap::new(),
            events: HashMap::new(),
            object_ids,
            tiled_infos: vec![],
            internal_sprites: vec![],
            sprite_blocks: vec![],
        };
        let state = app.state.editors.map_editors.entry(TAB).or_default();
        state.data.loading_state = LoadingState::Loaded(MapDataHandle(Arc::new(map_data)));
        app
    }

    #[test]
    fn test_select_tool_collision_enables_layer() {
        let mut app = app_with_loaded_map(HashMap::new());
        assert!(!layer_visible(
            &app.state.editors.map_editors[&TAB].view,
            MapLayer::Collisions
        ));

        let _ = select_tool(&mut app, TAB, MapTool::Collision);

        let state = &app.state.editors.map_editors[&TAB];
        assert_eq!(state.view.active_tool, MapTool::Collision);
        assert!(
            layer_visible(&state.view, MapLayer::Collisions),
            "selecting the Collision tool must force-enable the Collisions layer"
        );
    }

    #[test]
    fn test_hide_layer_resets_tool_to_pan() {
        let mut app = app_with_loaded_map(HashMap::new());
        let _ = select_tool(&mut app, TAB, MapTool::ObjectId);
        assert_eq!(
            app.state.editors.map_editors[&TAB].view.active_tool,
            MapTool::ObjectId
        );

        // Hide the layer that owns the active tool via LayerToggled.
        let msg = MapEditorMessage::LayerToggled(TAB, MapLayer::ObjectIds);
        let _ = super::super::handle(msg, &mut app);

        assert_eq!(
            app.state.editors.map_editors[&TAB].view.active_tool,
            MapTool::Pan,
            "hiding the owning layer must reset the tool to Pan"
        );
    }

    #[test]
    fn test_object_id_paint_writes_brush() {
        let mut app = app_with_loaded_map(HashMap::from([((5, 5), 7)]));
        let state = app.state.editors.map_editors.get_mut(&TAB).unwrap();
        state.data.object_brush = 3;
        state.view.object_brush_mode = ObjectBrushMode::Paint;

        assert!(apply_object_id_edit(state, 5, 5));
        let state = app.state.editors.map_editors.get_mut(&TAB).unwrap();
        if let LoadingState::Loaded(ref handle) = state.data.loading_state {
            assert_eq!(
                handle.0.object_ids.get(&(5, 5)),
                Some(&3),
                "paint overwrites existing value"
            );
        } else {
            panic!("map not loaded");
        }
        assert_eq!(state.data.undo_stack.len(), 1);
    }

    #[test]
    fn test_object_id_erase_removes_any_value() {
        let mut app = app_with_loaded_map(HashMap::from([((5, 5), 7)]));
        let state = app.state.editors.map_editors.get_mut(&TAB).unwrap();
        state.data.object_brush = 7; // same value as existing entry — erase must still remove
        state.view.object_brush_mode = ObjectBrushMode::Erase;

        assert!(apply_object_id_edit(state, 5, 5));
        let state = app.state.editors.map_editors.get_mut(&TAB).unwrap();
        if let LoadingState::Loaded(ref handle) = state.data.loading_state {
            assert!(
                !handle.0.object_ids.contains_key(&(5, 5)),
                "erase removes the entry regardless of its value"
            );
        } else {
            panic!("map not loaded");
        }
    }

    #[test]
    fn test_brush_stepper_clamps_at_bounds() {
        let mut app = app_with_loaded_map(HashMap::new());

        for v in [999, 600] {
            let msg = MapEditorMessage::SetObjectBrush(TAB, v);
            let _ = super::super::handle(msg, &mut app);
            assert_eq!(app.state.editors.map_editors[&TAB].data.object_brush, 511);
        }
        for v in [0, -5] {
            let msg = MapEditorMessage::SetObjectBrush(TAB, v);
            let _ = super::super::handle(msg, &mut app);
            assert_eq!(app.state.editors.map_editors[&TAB].data.object_brush, 1);
        }
    }

    #[allow(dead_code)]
    fn _assert_state_type(s: &MapEditorState) -> &MapEditorState {
        s
    }
}
