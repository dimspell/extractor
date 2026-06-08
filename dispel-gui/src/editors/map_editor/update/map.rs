use crate::app::App;
use crate::components::loading_state::LoadingState;
use crate::editors::map_editor::canvas::decode_tileset_file;
use crate::editors::map_editor::{
    DecodedEntitySprite, DecodedMapSprite, EntityBundle, EntitySpriteHandle,
    InternalSpriteHandle, MapDataHandle, MapEditorMessage, SpriteSequenceHandle,
    TilePixelData,
};
use crate::message::{Message, MessageExt};
use dispel_core::references::extractor::Extractor;
use iced::widget::image::Handle;
use iced::Task;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

// ── Message handlers ─────────────────────────────────────────────────────────

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
    {
        if let Ok(monsters) = dispel_core::references::monster_ini::MonsterIni::read_file(
            &std::path::PathBuf::from(&app.state.shared_game_path).join("Monster.ini"),
        ) {
            let names: Vec<(String, String)> = monsters
                .iter()
                .map(|m| (m.id.to_string(), m.name.clone().unwrap_or_default()))
                .collect();
            app.state
                .lookups
                .insert("monster_names".to_string(), names);
        }
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
                    seq_first.entry(sid).or_insert_with(|| {
                        (sprite.width, sprite.height, sprite.pixels.clone())
                    });
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
                    placement_count: seq_placements
                        .get(&sid)
                        .map(|v| v.len())
                        .unwrap_or(0),
                    placements: seq_placements.remove(&sid).unwrap_or_default(),
                })
                .collect();
            seq_handles.sort_by_key(|s| s.sequence_idx);
            state.data.sprite_sequence_handles = seq_handles;

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
                    let gtl_ids: HashSet<i32> =
                        arc_data.gtl_tiles.values().copied().collect();
                    let btl_ids: HashSet<i32> = arc_data
                        .btl_tiles
                        .values()
                        .copied()
                        .chain(arc_data.tiled_infos.iter().flat_map(|t| {
                            t.ids.iter().map(|&id| id.unsigned_abs() as i32)
                        }))
                        .filter(|&id| id > 0)
                        .collect();

                    let gtl =
                        decode_tileset_file(&gtl_path, &gtl_ids).unwrap_or_default();
                    let btl =
                        decode_tileset_file(&btl_path, &btl_ids).unwrap_or_default();

                    Ok(TilePixelData { gtl, btl })
                },
                move |result| {
                    Message::map_editor(MapEditorMessage::TilesDecoded(tab_id, result))
                },
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
        state.data.status_msg =
            Some("No game path set — entity files not loaded".to_string());
    }
    Task::perform(
        async move { load_entities(&map_path, game_path) },
        move |bundle| Message::map_editor(MapEditorMessage::EntitiesLoaded(tab_id, bundle)),
    )
}

pub fn entities_loaded(
    app: &mut App,
    tab_id: usize,
    bundle: EntityBundle,
) -> Task<Message> {
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
    let npc_id_to_sprite: HashMap<i32, String> =
        NpcIni::read_file(&game_path.join("Npc.ini"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
            .collect();
    let extra_id_to_sprite: HashMap<i32, String> =
        Extra::read_file(&game_path.join("Extra.ini"))
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
        |m: &MonsterRef| (m.mon_id, 3, false),
        &monster_id_to_sprite,
        &mut sprite_cache,
    );

    // ── NPCs ──────────────────────────────────────────────────────────────────
    let (npcs, npc_sprite_handles, npc_ref_path) = load_ref_file(
        map_ini.npc_filename,
        "NpcInGame",
        &resolve,
        |n: &NPC| {
            let dir = i32::from(n.looking_direction);
            let (seq, flip) = if dir > 4 {
                ((8 - dir) as usize, true)
            } else {
                (dir as usize, false)
            };
            (n.npc_id, seq, flip)
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
            let rotation = e.rotation as usize;
            let obj_type = u8::from(e.object_type) as usize;
            let seq = if obj_type == 0 {
                2 * e.closed as usize + rotation
            } else {
                rotation
            };
            (e.ext_id as i32, seq, false)
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
            all_draw_items.into_iter().filter(|d| d.map_id == id).collect()
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
        if p.exists() { Some(p.to_path_buf()) } else { None }
    }

    // ── resolve_map_id ──────────────────────────────────────────────────────────

    #[test]
    fn test_resolve_map_id_found() {
        let gp = match game_path() {
            Some(p) => p,
            None => { eprintln!("Skipping: fixtures not found"); return; }
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
            None => { eprintln!("Skipping: fixtures not found"); return; }
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
            None => { eprintln!("Skipping: fixtures not found"); return; }
        };
        assert_eq!(resolve_map_id("", &gp), None);
    }

    // ── load_entities ──────────────────────────────────────────────────────────

    #[test]
    fn test_load_entities_draw_items_and_map_id() {
        let gp = match game_path() {
            Some(p) => p,
            None => { eprintln!("Skipping: fixtures not found"); return; }
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
                d.item_type,
                dispel_core::references::enums::ItemTypeId::Event
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
            None => { eprintln!("Skipping: fixtures not found"); return; }
        };

        // cat1 = Palace of Aesh: NPCs only (24), no monsters, no extras
        let cat1_entities = load_entities(
            &gp.join("Map/cat1.map"),
            Some(gp.clone()),
        );
        assert_eq!(cat1_entities.monsters.len(), 0, "cat1 has no monsters");
        assert_eq!(cat1_entities.npcs.len(), 24, "cat1 has 24 NPCs");
        assert_eq!(
            cat1_entities.extra_refs.len(),
            0,
            "cat1 has no extra refs"
        );
        assert_eq!(
            cat1_entities.npc_sprites.len(),
            24,
            "each NPC has a sprite slot"
        );

        // map1 = Aesh overworld: monsters + NPCs + extras
        let map1_entities = load_entities(
            &gp.join("Map/map1.map"),
            Some(gp),
        );
        assert!(
            map1_entities.monsters.len() > 0,
            "map1 should have monsters"
        );
        assert!(map1_entities.npcs.len() > 0, "map1 should have NPCs");
        assert!(
            map1_entities.extra_refs.len() > 0,
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
        let bundle = load_entities(&Path::new("Map/cat1.map"), None);
        assert_eq!(bundle.draw_items.len(), 0);
        assert_eq!(bundle.all_map_id, None);
        assert_eq!(bundle.monsters.len(), 0);
    }
}
