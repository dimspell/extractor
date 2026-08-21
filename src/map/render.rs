use image::{ImageBuffer, Rgb, Rgba};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::map::tileset::{TILE_HEIGHT, Tile, plot_tile};
use crate::sprite::{ImageInfo, SequenceInfo, rgb16_565_produce_color};
use byteorder::{LittleEndian, ReadBytesExt};

use super::types::{
    Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo, convert_map_coords_to_image_coords,
    internal_sprite_sort_key, tiled_object_sort_key,
};

use super::model::MapModel;

// --------------------------------------------------------------------------
// Layer visibility toggles
// --------------------------------------------------------------------------

/// Global visibility toggles for each layer and overlay type.
///
/// All layer toggles default to `true` (visible). Overlay toggles default to
/// `false` (hidden) and must be explicitly enabled.
#[derive(Debug, Clone, Copy)]
pub struct LayerToggles {
    pub show_ground: bool,
    pub show_buildings: bool,
    pub show_roofs: bool,
    pub show_internal_sprites: bool,
    pub show_monsters: bool,
    pub show_npcs: bool,
    pub show_objects: bool,
    /// If true, render the entire map canvas instead of the occluded viewport.
    pub full_map: bool,
    /// If true, output RGBA PNG where black (0,0,0) background pixels are
    /// transparent (alpha=0). When false, output standard RGB PNG.
    pub transparent: bool,
    pub show_collisions: bool,
    pub show_events: bool,
    pub show_draw_items: bool,
    pub show_npc_waypoints: bool,
}

impl Default for LayerToggles {
    fn default() -> Self {
        Self {
            show_ground: true,
            show_buildings: true,
            show_roofs: true,
            show_internal_sprites: true,
            show_monsters: true,
            show_npcs: true,
            show_objects: true,
            full_map: false,
            transparent: false,
            show_collisions: false,
            show_events: false,
            show_draw_items: false,
            show_npc_waypoints: false,
        }
    }
}

// --------------------------------------------------------------------------
// Pre-loaded external entity data for interleaved rendering
// --------------------------------------------------------------------------

/// Pre-loaded render info for a single external entity.
#[derive(Debug, Clone)]
pub struct EntityRenderInfo {
    pub x: i32,
    pub y: i32,
    pub fallback_color: [u8; 3],
    pub sprite_path: Option<PathBuf>,
    pub sequence: usize,
    pub flip: bool,
}

/// All external entity data collected from game files.
pub struct ExternalEntities {
    pub monsters: Vec<EntityRenderInfo>,
    pub npcs: Vec<EntityRenderInfo>,
    pub extras: Vec<EntityRenderInfo>,
    /// Full NPC records kept for waypoint overlay rendering.
    pub npc_records: Vec<crate::references::npc_ref::NPC>,
    /// Draw items placed on this map.
    pub draw_items: Vec<crate::references::draw_item::DrawItem>,
}

// --------------------------------------------------------------------------
// Top-level render entry point
// --------------------------------------------------------------------------

/// Configuration for rendering a map
pub struct MapRenderConfig<'a> {
    pub reader: &'a mut BufReader<File>,
    pub output_path: &'a Path,
    pub data: &'a super::MapData,
    pub occlusion: bool,
    pub gtl_tileset: &'a [Tile],
    pub btl_tileset: &'a [Tile],
    pub map_id: &'a str,
    pub game_path: Option<&'a Path>,
    pub toggles: LayerToggles,
}

/// Renders the full map to a PNG file.
///
/// Rendering order:
/// 1. Ground tiles (if `show_ground`)
/// 2. Interleaved objects + entities sorted by Y-depth (if their toggles are on)
/// 3. Roof tiles (if `show_roofs`)
/// 4. Overlays: collisions, events, draw items, NPC waypoints (if their toggles are on)
pub fn render_map(config: MapRenderConfig) -> Result<()> {
    let MapRenderConfig {
        reader,
        output_path,
        data,
        occlusion,
        gtl_tileset,
        btl_tileset,
        map_id,
        game_path,
        toggles,
    } = config;

    let image_width = if occlusion {
        data.model.occluded_map_in_pixels_width
    } else {
        data.model.map_width_in_pixels
    };
    let image_height = if occlusion {
        data.model.occluded_map_in_pixels_height
    } else {
        data.model.map_height_in_pixels
    };

    println!("{:?}", data.model);
    println!(
        "{}, {}",
        image_width.unsigned_abs(),
        image_height.unsigned_abs()
    );

    let offset_x = if !occlusion {
        data.model.map_non_occluded_start_x
    } else {
        0
    };
    let offset_y = if !occlusion {
        data.model.map_non_occluded_start_y
    } else {
        0
    };

    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::new(image_width.unsigned_abs(), image_height.unsigned_abs());

    // ── Pass 1: Ground tiles ──────────────────────────────────────────────
    if toggles.show_ground {
        plot_base(
            &mut imgbuf,
            &data.model,
            occlusion,
            &data.gtl_tiles,
            gtl_tileset,
        );
    }

    // ── Pre-load external entities (if game path given) ──────────────────
    let external = game_path.and_then(|gp| collect_external_entities(map_id, gp, &data.model).ok());

    // ── Pass 2: Interleaved objects + entities ───────────────────────────
    // All depth-relevant items (buildings, internal sprites, monsters, NPCs,
    // extras) are collected into one list, sorted by Y-depth with type
    // tiebreaker, then rendered together — matching the DispelTools
    // IInterlacedOrderObject / IInterlacedOrderObjectComparer approach.
    {
        let mut sprite_cache: HashMap<
            PathBuf,
            Option<Vec<super::sprite_loader::LoadedSpriteFrame>>,
        > = HashMap::new();

        let diagonal = data.model.tiled_map_width + data.model.tiled_map_height;
        // All interlaced sort keys are compared in map-local pixel space
        // (world Y minus the non-occluded origin), independent of output mode.
        let noy = data.model.map_non_occluded_start_y;

        // Helper: entity sort position (matches GUI's entity_pos)
        let entity_pos = |tx: i32, ty: i32| -> i32 {
            convert_map_coords_to_image_coords(tx, ty, diagonal).1 + 32 - noy
        };

        enum ItemKind {
            TiledObject(usize, usize),
            Sprite(usize),
            Monster(usize),
            Npc(usize),
            Extra(usize),
        }

        let mut items: Vec<(i32, i32, i32, ItemKind)> = Vec::new();

        // Buildings (type_order=0) — one item per stack tile so entities
        // interleave at per-tile depth (see tiled_object_sort_key). Keys are
        // map-local, matching entity_pos below.
        if toggles.show_buildings {
            for (i, info) in data.tiled_infos.iter().enumerate() {
                for level in 0..info.ids.len() {
                    let pos = tiled_object_sort_key(info.y, level);
                    items.push((pos, 0, info.x, ItemKind::TiledObject(i, level)));
                }
            }
        }

        // Internal sprites (type_order=1) — key is map-local
        // (sprite_bottom_right_y == sprite_y + height), minus the half-tile
        // window so characters sitting on props draw over them.
        if toggles.show_internal_sprites {
            for (i, block) in data.sprite_blocks.iter().enumerate() {
                if block.sprite_id < data.internal_sprites.len() {
                    let h = data.internal_sprites[block.sprite_id].frame_infos[0].height;
                    let pos = internal_sprite_sort_key(block.sprite_y + h);
                    items.push((pos, 1, block.sprite_x, ItemKind::Sprite(i)));
                }
            }
        }

        // External entities (type_order rungs shared with the GUI interlaced
        // pass: Extra=2 < DrawItem=3 < Monster=4 < Npc=5).
        if let Some(ref ext) = external {
            if toggles.show_monsters {
                for (i, m) in ext.monsters.iter().enumerate() {
                    items.push((entity_pos(m.x, m.y), 4, m.x, ItemKind::Monster(i)));
                }
            }
            if toggles.show_npcs {
                for (i, n) in ext.npcs.iter().enumerate() {
                    items.push((entity_pos(n.x, n.y), 5, n.x, ItemKind::Npc(i)));
                }
            }
            if toggles.show_objects {
                for (i, e) in ext.extras.iter().enumerate() {
                    items.push((entity_pos(e.x, e.y), 2, e.x, ItemKind::Extra(i)));
                }
            }
        }

        items.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

        for (_, _, _, item) in &items {
            match item {
                ItemKind::TiledObject(i, level) => {
                    let info = &data.tiled_infos[*i];
                    let Some(&btl_id) = info.ids.get(*level) else {
                        continue;
                    };
                    if btl_id <= 0 {
                        continue;
                    }
                    let btl_tile_idx = btl_id.unsigned_abs() as usize;
                    if let Some(tile) = btl_tileset.get(btl_tile_idx) {
                        let x = info.x + offset_x;
                        let y = info.y + (*level as i32 * TILE_HEIGHT as i32) + offset_y;
                        plot_tile(&mut imgbuf, tile.colors, x, y);
                    }
                }
                ItemKind::Sprite(i) => {
                    let block = &data.sprite_blocks[*i];
                    let sequence = &data.internal_sprites[block.sprite_id];
                    let sprite = &sequence.frame_infos[0];
                    let dest_x = block.sprite_x + offset_x;
                    let dest_y = block.sprite_y + offset_y;
                    plot_sprite_on_bitmap(&mut imgbuf, reader, sprite, dest_x, dest_y)?;
                }
                ItemKind::Monster(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.monsters[*i],
                            &mut sprite_cache,
                            diagonal,
                            offset_x,
                            offset_y,
                        );
                    }
                }
                ItemKind::Npc(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.npcs[*i],
                            &mut sprite_cache,
                            diagonal,
                            offset_x,
                            offset_y,
                        );
                    }
                }
                ItemKind::Extra(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.extras[*i],
                            &mut sprite_cache,
                            diagonal,
                            offset_x,
                            offset_y,
                        );
                    }
                }
            }
        }
    }

    // ── Pass 3: Roof tiles ───────────────────────────────────────────────
    if toggles.show_roofs {
        plot_roofs(
            &mut imgbuf,
            &data.model,
            occlusion,
            &data.btl_tiles,
            btl_tileset,
        );
    }

    // ── Pass 4: Overlays ─────────────────────────────────────────────────
    let diagonal = data.model.tiled_map_width + data.model.tiled_map_height;

    if toggles.show_collisions {
        plot_collisions_overlay(
            &mut imgbuf,
            &data.model,
            &data.collisions,
            occlusion,
            diagonal,
        );
    }

    if toggles.show_events {
        plot_events_overlay(&mut imgbuf, &data.model, &data.events, occlusion, diagonal);
    }

    if toggles.show_draw_items
        && let Some(ref ext) = external
    {
        plot_draw_items_overlay(
            &mut imgbuf,
            &ext.draw_items,
            &data.model,
            occlusion,
            diagonal,
        );
    }

    if toggles.show_npc_waypoints
        && let Some(ref ext) = external
    {
        plot_npc_waypoints_overlay(
            &mut imgbuf,
            &ext.npc_records,
            &data.model,
            occlusion,
            diagonal,
        );
    }

    // ── Save: RGBA PNG (transparent) or RGB PNG (solid black) ───────────
    if toggles.transparent {
        let (w, h) = imgbuf.dimensions();
        let mut rgba: ImageBuffer<Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(w, h);
        for (x, y, pixel) in imgbuf.enumerate_pixels() {
            if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
                rgba.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            } else {
                rgba.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], 255]));
            }
        }
        rgba.save(output_path)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    } else {
        imgbuf
            .save(output_path)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Ground layer
// --------------------------------------------------------------------------

/// Renders ground (GTL) tiles. Pure tiles, no event/collision tinting (those
/// are now rendered as separate overlays).
pub fn plot_base(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    occlusion: bool,
    gtl_tiles: &HashMap<Coords, i32>,
    gtl_tileset: &[Tile],
) {
    let map_diagonal_tiles = model.tiled_map_width + model.tiled_map_height;
    let width = model.tiled_map_width;
    let height = model.tiled_map_height;

    for diff in -(width - 1)..height {
        let start_x = 0.max(-diff);
        let end_x = (width - 1).min(height - 1 - diff);
        for x in start_x..=end_x {
            let y = x + diff;
            let coords: Coords = (x, y);
            if let Some(&gtl_tile_id) = gtl_tiles.get(&coords) {
                let gtl_tile_idx = gtl_tile_id.unsigned_abs() as usize;
                let Some(gtl_tile) = gtl_tileset.get(gtl_tile_idx) else {
                    continue;
                };

                let (mut sx, mut sy) = convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
                if occlusion {
                    sx -= model.map_non_occluded_start_x;
                    sy -= model.map_non_occluded_start_y;
                }

                plot_tile(image, gtl_tile.colors, sx, sy);
            }
        }
    }
}

// --------------------------------------------------------------------------
// Object layer (sprites + tiled objects, sorted by ground-y)
// --------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct PlotObjectsParams<'a> {
    pub btl_tileset: &'a [Tile],
    pub tiled_info: &'a [TiledObjectInfo],
    pub internal_sprites: &'a [SequenceInfo],
    pub sprite_blocks: &'a [SpriteInfoBlock],
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Legacy object renderer — renders only sprites + tiled objects sorted by Y.
/// Used by the database renderer; new code should use `render_map` with toggles.
pub fn plot_objects(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    reader: &mut BufReader<File>,
    _model: &MapModel,
    _occlusion: bool,
    _btl_tiles: &HashMap<Coords, i32>,
    params: PlotObjectsParams,
) -> Result<()> {
    enum Kind {
        Sprite(usize),
        TiledObject { obj: usize, level: usize },
    }
    struct Item {
        ground_y: i32,
        kind: Kind,
    }

    let mut items = Vec::new();

    for (i, block) in params.sprite_blocks.iter().enumerate() {
        let sequence = &params.internal_sprites[block.sprite_id];
        let sprite = &sequence.frame_infos[0];
        items.push(Item {
            ground_y: block.sprite_y + sprite.height,
            kind: Kind::Sprite(i),
        });
    }
    // Per-tile depth (see tiled_object_sort_key) — consistent with render_map.
    for (i, info) in params.tiled_info.iter().enumerate() {
        for level in 0..info.ids.len() {
            items.push(Item {
                ground_y: tiled_object_sort_key(info.y, level),
                kind: Kind::TiledObject { obj: i, level },
            });
        }
    }
    items.sort_by_key(|it| it.ground_y);

    for item in items {
        match item.kind {
            Kind::Sprite(i) => plot_single_sprite(
                imgbuf,
                reader,
                &params.sprite_blocks[i],
                params.internal_sprites,
                params.offset_x,
                params.offset_y,
            )?,
            Kind::TiledObject { obj, level } => {
                let tiled_info = &params.tiled_info[obj];
                let Some(&btl_id) = tiled_info.ids.get(level) else {
                    continue;
                };
                if btl_id <= 0 {
                    continue;
                }
                let btl_tile_idx = btl_id.unsigned_abs() as usize;
                if let Some(tile) = params.btl_tileset.get(btl_tile_idx) {
                    let x = tiled_info.x + params.offset_x;
                    let y = tiled_info.y + (level as i32 * TILE_HEIGHT as i32) + params.offset_y;
                    plot_tile(imgbuf, tile.colors, x, y);
                }
            }
        }
    }
    Ok(())
}

fn plot_single_sprite(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    reader: &mut BufReader<File>,
    sprite_block: &SpriteInfoBlock,
    internal_sprites: &[SequenceInfo],
    offset_x: i32,
    offset_y: i32,
) -> Result<()> {
    let sequence = &internal_sprites[sprite_block.sprite_id];
    let sprite = &sequence.frame_infos[0];
    let dest_x = sprite_block.sprite_x + offset_x;
    let dest_y = sprite_block.sprite_y + offset_y;
    plot_sprite_on_bitmap(imgbuf, reader, sprite, dest_x, dest_y)
}

pub fn plot_sprite_on_bitmap(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    reader: &mut BufReader<File>,
    sprite: &ImageInfo,
    dest_x: i32,
    dest_y: i32,
) -> Result<()> {
    if dest_x + sprite.width <= imgbuf.width() as i32
        && dest_x >= 0
        && dest_y >= 0
        && dest_y + sprite.height <= imgbuf.height() as i32
    {
        reader.seek(SeekFrom::Start(sprite.image_start_position))?;
        for y in 0..sprite.height {
            for x in 0..sprite.width {
                let pixel = reader.read_u16::<LittleEndian>()?;
                let color = rgb16_565_produce_color(pixel);
                if pixel > 0 {
                    imgbuf.put_pixel(
                        (dest_x + x) as u32,
                        (dest_y + y) as u32,
                        Rgb([color.r, color.g, color.b]),
                    );
                }
            }
        }
    }
    Ok(())
}

// --------------------------------------------------------------------------
// Roof layer
// --------------------------------------------------------------------------

pub fn plot_roofs(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    occlusion: bool,
    btl_tiles: &HashMap<Coords, i32>,
    btl_tileset: &[Tile],
) {
    let map_diagonal_tiles = model.tiled_map_width + model.tiled_map_height;
    let width = model.tiled_map_width;
    let height = model.tiled_map_height;

    for diff in -(width - 1)..height {
        let start_x = 0.max(-diff);
        let end_x = (width - 1).min(height - 1 - diff);
        for x in start_x..=end_x {
            let y = x + diff;
            let coords: Coords = (x, y);
            let btl_tile_id = btl_tiles.get(&coords).copied().unwrap_or(0);
            if btl_tile_id > 0 {
                let btl_tile_idx = btl_tile_id as usize;
                if let Some(btl_tile) = btl_tileset.get(btl_tile_idx) {
                    let (mut sx, mut sy) =
                        convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
                    if occlusion {
                        sx -= model.map_non_occluded_start_x;
                        sy -= model.map_non_occluded_start_y;
                    }
                    plot_tile(image, btl_tile.colors, sx, sy);
                }
            }
        }
    }
}

// --------------------------------------------------------------------------
// External entity collection and rendering
// --------------------------------------------------------------------------

/// Loads all external entity data (monsters, NPCs, extras, draw items) for the
/// given map from the game files.
pub fn collect_external_entities(
    map_id: &str,
    game_path: &Path,
    model: &MapModel,
) -> Result<ExternalEntities> {
    use crate::references::{
        draw_item::DrawItem, extra_ini::Extra, extra_ref::ExtraRef, extractor::Extractor,
        map_ini::read_map_ini, monster_ini::MonsterIni, monster_ref::MonsterRef, npc_ini::NpcIni,
        npc_ref::NPC,
    };

    let map_ini_path = game_path.join("Ref").join("Map.ini");
    if !map_ini_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Map.ini not found at {:?}", map_ini_path),
        ));
    }

    let map_base_name = map_id.split('.').next().unwrap_or(map_id);

    let map_inis = read_map_ini(&map_ini_path)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let map_ini = map_inis.into_iter().find(|ini| {
        ini.monsters_filename
            .as_ref()
            .map(|m| m.contains(map_base_name))
            .unwrap_or(false)
            || ini
                .npc_filename
                .as_ref()
                .map(|n| n.contains(map_base_name))
                .unwrap_or(false)
            || ini
                .extra_filename
                .as_ref()
                .map(|e| e.contains(map_base_name))
                .unwrap_or(false)
    });

    let Some(map_ini) = map_ini else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No Map.ini entry found for map '{}'", map_base_name),
        ));
    };

    let diagonal = model.tiled_map_width + model.tiled_map_height;
    let _ = diagonal; // used for resolving offsets, kept for consistency

    let resolve = |dir: &str, filename: &str| -> PathBuf {
        let upper = filename.to_ascii_uppercase();
        let p_upper = game_path.join(dir).join(&upper);
        if p_upper.exists() {
            return p_upper;
        }
        let p = game_path.join(dir).join(filename);
        if p.exists() {
            return p;
        }
        let lower = filename.to_ascii_lowercase();
        let p_lower = game_path.join(dir).join(&lower);
        if p_lower.exists() {
            return p_lower;
        }
        let mut capitalized = filename.to_string();
        if let Some(c) = capitalized.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        let p_cap = game_path.join(dir).join(&capitalized);
        if p_cap.exists() {
            return p_cap;
        }
        p
    };

    let monster_sprite_map: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();

    let npc_sprite_map: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();

    let extra_sprite_map: HashMap<i32, String> = Extra::read_file(&game_path.join("Extra.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
        .collect();

    let mut monsters = Vec::new();
    let mut npcs = Vec::new();
    let mut extras = Vec::new();
    let mut npc_records = Vec::new();

    if let Some(f) = map_ini.monsters_filename {
        let p = resolve("MonsterInGame", &f);
        if let Ok(data) = MonsterRef::read_file(&p) {
            for m in data {
                let sprite_path = monster_sprite_map
                    .get(&m.monster_db_id)
                    .map(|s| resolve("MonsterInGame", s));
                monsters.push(EntityRenderInfo {
                    x: m.map_x,
                    y: m.map_y,
                    fallback_color: [220, 50, 50],
                    sprite_path,
                    sequence: 3,
                    flip: false,
                });
            }
        }
    }

    if let Some(f) = map_ini.npc_filename {
        let p = resolve("NpcInGame", &f);
        if let Ok(data) = NPC::read_file(&p) {
            for n in data {
                let waypoints = [
                    (n.goto1_filled, n.goto1_x, n.goto1_y),
                    (n.goto2_filled, n.goto2_x, n.goto2_y),
                    (n.goto3_filled, n.goto3_x, n.goto3_y),
                    (n.goto4_filled, n.goto4_x, n.goto4_y),
                ];

                let (x, y) = waypoints
                    .iter()
                    .find(|(filled, _, _)| i32::from(*filled) != 0)
                    .map(|(_, x, y)| (*x, *y))
                    .unwrap_or((n.goto1_x, n.goto1_y));

                let dir = i32::from(n.waypoint1_facing_direction);
                let (seq, flip) = if dir > 4 {
                    ((8 - dir) as usize, true)
                } else {
                    (dir as usize, false)
                };
                let sprite_path = npc_sprite_map
                    .get(&n.npc_ini_id)
                    .map(|s| resolve("NpcInGame", s));

                npcs.push(EntityRenderInfo {
                    x,
                    y,
                    fallback_color: [50, 100, 220],
                    sprite_path,
                    sequence: seq,
                    flip,
                });
                npc_records.push(n.clone());
            }
        }
    }

    if let Some(f) = map_ini.extra_filename {
        let p = resolve("ExtraInGame", &f);
        if let Ok(data) = ExtraRef::read_file(&p) {
            for e in data {
                let rotation = e.direction as i32;
                let obj_type = u8::from(e.object_type) as i32;
                let seq = if obj_type == 0 {
                    // Chests use a separate sprite sequence after opening.
                    (2 * e.interaction_state + rotation) as usize
                } else {
                    rotation as usize
                };
                let sprite_path = extra_sprite_map
                    .get(&(e.extra_definition_id as i32))
                    .map(|s| resolve("ExtraInGame", s));
                extras.push(EntityRenderInfo {
                    x: e.map_x,
                    y: e.map_y,
                    fallback_color: [200, 180, 30],
                    sprite_path,
                    sequence: seq,
                    flip: false,
                });
            }
        }
    }

    // Load draw items for this map
    let map_draw_id: i32 = map_base_name
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .parse()
        .unwrap_or(0);
    let draw_items: Vec<DrawItem> =
        DrawItem::read_file(&game_path.join("Ref").join("DRAWITEM.ref"))
            .unwrap_or_default()
            .into_iter()
            .filter(|d| d.map_id == map_draw_id)
            .collect();

    Ok(ExternalEntities {
        monsters,
        npcs,
        extras,
        npc_records,
        draw_items,
    })
}

/// Render a single external entity sprite (or fallback marker) onto the image.
fn render_entity_sprite(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    entity: &EntityRenderInfo,
    sprite_cache: &mut HashMap<PathBuf, Option<Vec<super::sprite_loader::LoadedSpriteFrame>>>,
    diagonal: i32,
    offset_px_x: i32,
    offset_px_y: i32,
) {
    let (px, py) = convert_map_coords_to_image_coords(entity.x, entity.y, diagonal);
    let cx = px - offset_px_x + super::tileset::TILE_WIDTH as i32 / 2;
    let cy = py - offset_px_y + TILE_HEIGHT as i32 / 2;

    let mut rendered = false;
    if let Some(ref sp) = entity.sprite_path
        && sp.exists()
    {
        let frames = sprite_cache
            .entry(sp.clone())
            .or_insert_with(|| super::sprite_loader::load_sprite_frames(sp));

        if let Some(Some(frames)) = Some(frames)
            && !frames.is_empty()
        {
            let idx = entity.sequence.min(frames.len() - 1);
            let frame = &frames[idx];
            let dest_x = if entity.flip {
                cx - (frame.image.width() as i32 - frame.origin_x)
            } else {
                cx - frame.origin_x
            };
            let dest_y = cy - frame.origin_y;
            plot_rgba_sprite_on_rgb(imgbuf, &frame.image, dest_x, dest_y, entity.flip);
            rendered = true;
        }
    }

    if !rendered {
        plot_rgb_marker(imgbuf, cx, cy, entity.fallback_color);
    }
}

/// Blit an RGBA sprite onto an RGB destination image. Transparent pixels are skipped.
fn plot_rgba_sprite_on_rgb(
    dest: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    sprite: &image::RgbaImage,
    dest_x: i32,
    dest_y: i32,
    flip: bool,
) {
    let sw = sprite.width() as i32;
    let sh = sprite.height() as i32;
    let dw = dest.width() as i32;
    let dh = dest.height() as i32;
    for sy in 0..sh {
        let py = dest_y + sy;
        if py < 0 || py >= dh {
            continue;
        }
        for sx in 0..sw {
            let src_x = if flip {
                (sw - 1 - sx) as u32
            } else {
                sx as u32
            };
            let pixel = *sprite.get_pixel(src_x, sy as u32);
            if pixel[3] == 0 {
                continue;
            }
            let px = dest_x + sx;
            if px >= 0 && px < dw {
                dest.put_pixel(px as u32, py as u32, Rgb([pixel[0], pixel[1], pixel[2]]));
            }
        }
    }
}

/// Draw a 7×7 colored diamond marker centered at (cx, cy) on an RGB image.
fn plot_rgb_marker(imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, cx: i32, cy: i32, color: [u8; 3]) {
    let r = 4i32;
    let iw = imgbuf.width() as i32;
    let ih = imgbuf.height() as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx.abs() + dy.abs() <= r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < iw && py >= 0 && py < ih {
                    imgbuf.put_pixel(px as u32, py as u32, Rgb(color));
                }
            }
        }
    }
}

// --------------------------------------------------------------------------
// Overlay rendering
// --------------------------------------------------------------------------

/// Blend two RGB colors with a given alpha (0–255, where 255 = full overlay).
fn blend_rgb(base: [u8; 3], overlay: [u8; 3], alpha: u8) -> [u8; 3] {
    let a = alpha as u32;
    let inv = 255u32.wrapping_sub(a);
    [
        ((overlay[0] as u32 * a + base[0] as u32 * inv) / 255) as u8,
        ((overlay[1] as u32 * a + base[1] as u32 * inv) / 255) as u8,
        ((overlay[2] as u32 * a + base[2] as u32 * inv) / 255) as u8,
    ]
}

/// Draw a filled diamond centered at (cx, cy) with half-size `r` and given
/// color, blended with the existing image pixels at `alpha`.
fn fill_diamond_blended(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    cx: i32,
    cy: i32,
    r: i32,
    color: [u8; 3],
    alpha: u8,
) {
    let iw = imgbuf.width() as i32;
    let ih = imgbuf.height() as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx.abs() + dy.abs() <= r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < iw && py >= 0 && py < ih {
                    let existing = imgbuf.get_pixel(px as u32, py as u32);
                    let blended = blend_rgb([existing[0], existing[1], existing[2]], color, alpha);
                    imgbuf.put_pixel(px as u32, py as u32, Rgb(blended));
                }
            }
        }
    }
}

/// Draw a filled circle blended onto the image.
fn fill_circle_blended(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    cx: i32,
    cy: i32,
    r: i32,
    color: [u8; 3],
    alpha: u8,
) {
    let iw = imgbuf.width() as i32;
    let ih = imgbuf.height() as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                let px = cx + dx;
                let py = cy + dy;
                if px >= 0 && px < iw && py >= 0 && py < ih {
                    let existing = imgbuf.get_pixel(px as u32, py as u32);
                    let blended = blend_rgb([existing[0], existing[1], existing[2]], color, alpha);
                    imgbuf.put_pixel(px as u32, py as u32, Rgb(blended));
                }
            }
        }
    }
}

/// Draw a line between two points using Bresenham's algorithm, blended.
fn draw_line_blended(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: [u8; 3],
    alpha: u8,
) {
    let iw = imgbuf.width() as i32;
    let ih = imgbuf.height() as i32;
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x >= 0 && x < iw && y >= 0 && y < ih {
            let existing = imgbuf.get_pixel(x as u32, y as u32);
            let blended = blend_rgb([existing[0], existing[1], existing[2]], color, alpha);
            imgbuf.put_pixel(x as u32, y as u32, Rgb(blended));
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// --------------------------------------------------------------------------
// Bitmap font — digits 0-9 in a 3×5 grid
// --------------------------------------------------------------------------

/// 3×5 pixel patterns for digits 0-9 (row-major, 1=on).
const DIGIT_PATTERNS: [[u8; 15]; 10] = [
    [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1], // 0
    [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0], // 1
    [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1], // 2
    [1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1], // 3
    [1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1], // 4
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 5
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 6
    [1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1], // 7
    [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 8
    [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 9
];

const DIGIT_W: i32 = 3;
const DIGIT_H: i32 = 5;
const DIGIT_SPACING: i32 = 1;

/// Render a decimal number using a built-in 3×5 bitmap font. Only renders the
/// last `max_digits` digits to avoid overflowing.
fn draw_number(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    number: i32,
    color: [u8; 3],
    max_digits: usize,
) {
    let iw = imgbuf.width() as i32;
    let ih = imgbuf.height() as i32;
    // Get digits from least significant, cap at max_digits
    let mut n = number.unsigned_abs();
    let mut digits = Vec::with_capacity(max_digits);
    if n == 0 {
        digits.push(0usize);
    } else {
        while n > 0 && digits.len() < max_digits {
            digits.push((n % 10) as usize);
            n /= 10;
        }
    }
    digits.reverse();

    let total_w = digits.len() as i32 * (DIGIT_W + DIGIT_SPACING) - DIGIT_SPACING;
    let start_x = x - total_w / 2;

    for (d_idx, &digit) in digits.iter().enumerate() {
        let dx = start_x + d_idx as i32 * (DIGIT_W + DIGIT_SPACING);
        let pattern = &DIGIT_PATTERNS[digit];
        for row in 0..DIGIT_H {
            for col in 0..DIGIT_W {
                if pattern[(row * DIGIT_W + col) as usize] == 1 {
                    let px = dx + col;
                    let py = y + row;
                    if px >= 0 && px < iw && py >= 0 && py < ih {
                        imgbuf.put_pixel(px as u32, py as u32, Rgb(color));
                    }
                }
            }
        }
    }
}

// --------------------------------------------------------------------------
// Collision overlay — red blended diamonds on blocked tiles
// --------------------------------------------------------------------------

fn plot_collisions_overlay(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    collisions: &HashMap<Coords, bool>,
    occlusion: bool,
    diagonal: i32,
) {
    let width = model.tiled_map_width;
    let height = model.tiled_map_height;

    for diff in -(width - 1)..height {
        let start_x = 0.max(-diff);
        let end_x = (width - 1).min(height - 1 - diff);
        for x in start_x..=end_x {
            let y = x + diff;
            let coords: Coords = (x, y);
            let blocked = collisions.get(&coords).copied().unwrap_or(false);
            if !blocked {
                continue;
            }
            let (mut px, mut py) = convert_map_coords_to_image_coords(x, y, diagonal);
            if occlusion {
                px -= model.map_non_occluded_start_x;
                py -= model.map_non_occluded_start_y;
            }
            // Diamond center
            let cx = px + super::tileset::TILE_WIDTH as i32 / 2;
            let cy = py + TILE_HEIGHT as i32 / 2;
            let r = super::tileset::TILE_WIDTH as i32 / 4;
            fill_diamond_blended(imgbuf, cx, cy, r, [200, 25, 25], 80);
        }
    }
}

// --------------------------------------------------------------------------
// Event overlay — magenta dots with event ID labels
// --------------------------------------------------------------------------

fn plot_events_overlay(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    events: &HashMap<Coords, EventBlock>,
    occlusion: bool,
    diagonal: i32,
) {
    let width = model.tiled_map_width;
    let height = model.tiled_map_height;

    for diff in -(width - 1)..height {
        let start_x = 0.max(-diff);
        let end_x = (width - 1).min(height - 1 - diff);
        for x in start_x..=end_x {
            let y = x + diff;
            let coords: Coords = (x, y);
            let event = events.get(&coords).copied().unwrap_or(EventBlock {
                x,
                y,
                _unknown_value: 0,
                event_id: 0,
            });
            if event.event_id == 0 {
                continue;
            }
            let (mut px, mut py) = convert_map_coords_to_image_coords(x, y, diagonal);
            if occlusion {
                px -= model.map_non_occluded_start_x;
                py -= model.map_non_occluded_start_y;
            }
            let cx = px + super::tileset::TILE_WIDTH as i32 / 2;
            let cy = py + TILE_HEIGHT as i32 / 2;
            // Magenta dot
            fill_circle_blended(imgbuf, cx, cy, 3, [200, 25, 200], 180);
            // Event ID label above the dot
            draw_number(
                imgbuf,
                cx,
                cy - 8,
                event.event_id as i32,
                [255, 255, 255],
                3,
            );
        }
    }
}

// --------------------------------------------------------------------------
// Draw items overlay — coloured diamonds by item type + ID labels
// --------------------------------------------------------------------------

fn draw_item_color(item_type: Option<crate::references::enums::ItemTypeId>) -> [u8; 3] {
    use crate::references::enums::ItemTypeId;
    let item_type = match item_type {
        Some(t) => t,
        None => return [155, 155, 155],
    };
    match item_type {
        ItemTypeId::Weapon => [230, 40, 40],
        ItemTypeId::Healing => [40, 230, 40],
        ItemTypeId::Edit => [40, 115, 230],
        ItemTypeId::Event => [200, 40, 200],
        ItemTypeId::Misc => [240, 215, 25],
        ItemTypeId::Other => [155, 155, 155],
    }
}

fn plot_draw_items_overlay(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    draw_items: &[crate::references::draw_item::DrawItem],
    model: &MapModel,
    occlusion: bool,
    diagonal: i32,
) {
    for di in draw_items {
        let (mut px, mut py) = convert_map_coords_to_image_coords(di.x_coord, di.y_coord, diagonal);
        if occlusion {
            px -= model.map_non_occluded_start_x;
            py -= model.map_non_occluded_start_y;
        }
        let cx = px + super::tileset::TILE_WIDTH as i32 / 2;
        let cy = py + TILE_HEIGHT as i32 / 2;
        let color = draw_item_color(di.item.item_type());
        let r = 6;
        // Coloured diamond
        fill_diamond_blended(imgbuf, cx, cy, r, color, 200);
        // Item ID label above diamond
        draw_number(
            imgbuf,
            cx,
            cy - r - 3,
            di.item.item_id() as i32,
            [255, 255, 255],
            3,
        );
    }
}

// --------------------------------------------------------------------------
// NPC waypoint overlay — coloured arrow connectors
// --------------------------------------------------------------------------

fn plot_npc_waypoints_overlay(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    npc_records: &[crate::references::npc_ref::NPC],
    model: &MapModel,
    occlusion: bool,
    diagonal: i32,
) {
    let waypoint_colors: [[u8; 3]; 4] = [
        [50, 200, 50],  // green
        [50, 50, 200],  // blue
        [200, 50, 50],  // red
        [200, 200, 50], // yellow
    ];

    for npc in npc_records {
        let waypoints: Vec<(i32, i32)> = [
            (npc.goto1_x, npc.goto1_y),
            (npc.goto2_x, npc.goto2_y),
            (npc.goto3_x, npc.goto3_y),
            (npc.goto4_x, npc.goto4_y),
        ]
        .into_iter()
        .filter(|&(wx, wy)| wx != 0 || wy != 0)
        .collect();

        if waypoints.len() < 2 {
            continue;
        }

        for j in 0..waypoints.len() {
            let (sx, sy) =
                convert_map_coords_to_image_coords(waypoints[j].0, waypoints[j].1, diagonal);
            let (ex, ey) = convert_map_coords_to_image_coords(
                waypoints[(j + 1) % waypoints.len()].0,
                waypoints[(j + 1) % waypoints.len()].1,
                diagonal,
            );

            let mut sx = sx;
            let mut sy = sy;
            let mut ex = ex;
            let mut ey = ey;
            if occlusion {
                sx -= model.map_non_occluded_start_x;
                sy -= model.map_non_occluded_start_y;
                ex -= model.map_non_occluded_start_x;
                ey -= model.map_non_occluded_start_y;
            }

            // Center on tile
            let sx = sx + super::tileset::TILE_WIDTH as i32 / 2;
            let sy = sy + TILE_HEIGHT as i32 / 2;
            let ex = ex + super::tileset::TILE_WIDTH as i32 / 2;
            let ey = ey + TILE_HEIGHT as i32 / 2;

            let color = waypoint_colors[j % waypoint_colors.len()];

            // Draw arrow line
            let dx = ex - sx;
            let dy = ey - sy;
            let length = ((dx * dx + dy * dy) as f64).sqrt() as i32;
            if length < 1 {
                continue;
            }
            let nx = dx as f64 / length as f64;
            let ny = dy as f64 / length as f64;

            // Line stops short of the arrowhead
            let head_len = 8.0;
            let line_end_x = (ex as f64 - head_len * nx) as i32;
            let line_end_y = (ey as f64 - head_len * ny) as i32;

            draw_line_blended(imgbuf, sx, sy, line_end_x, line_end_y, color, 220);

            // Arrowhead
            let head_w = 4.0;
            let hx1 = (line_end_x as f64 + head_w * ny) as i32;
            let hy1 = (line_end_y as f64 - head_w * nx) as i32;
            let hx2 = (line_end_x as f64 - head_w * ny) as i32;
            let hy2 = (line_end_y as f64 + head_w * nx) as i32;

            draw_line_blended(imgbuf, ex, ey, hx1, hy1, color, 220);
            draw_line_blended(imgbuf, ex, ey, hx2, hy2, color, 220);

            // Waypoint index label
            let label_cx = sx + super::tileset::TILE_WIDTH as i32 / 2;
            let label_cy = sy - 8;
            draw_number(imgbuf, label_cx, label_cy, (j + 1) as i32, [255; 3], 1);
        }
    }
}

// --------------------------------------------------------------------------
// Atlas tile blitter (used by render_from_database)
// --------------------------------------------------------------------------

pub struct AtlasTileParams<'a> {
    pub dest: &'a mut image::RgbaImage,
    pub atlas: &'a image::DynamicImage,
    pub src_x: u32,
    pub src_y: u32,
    pub tile_w: u32,
    pub tile_h: u32,
    pub dest_x: i32,
    pub dest_y: i32,
}

/// Copies a tile from a pre-built atlas image onto the destination buffer,
/// with per-pixel alpha blending support.
pub fn plot_atlas_tile(params: AtlasTileParams) {
    use image::GenericImageView;

    let dest_x = if params.dest_x < 0 || params.dest_y < 0 {
        return;
    } else {
        params.dest_x as u32
    };
    let dest_y = params.dest_y as u32;

    if dest_x + params.tile_w > params.dest.width()
        || dest_y + params.tile_h > params.dest.height()
        || params.src_x + params.tile_w > params.atlas.width()
        || params.src_y + params.tile_h > params.atlas.height()
    {
        return;
    }

    for py in 0..params.tile_h {
        for px in 0..params.tile_w {
            let pixel = params.atlas.get_pixel(params.src_x + px, params.src_y + py);
            let alpha = pixel[3];
            if alpha == 0 {
                continue;
            }
            if alpha == 255 {
                params.dest.put_pixel(dest_x + px, dest_y + py, pixel);
            } else {
                let existing = *params.dest.get_pixel(dest_x + px, dest_y + py);
                let blend = |src: u8, dst: u8, a: u8| -> u8 {
                    ((src as u32 * a as u32 + dst as u32 * (255 - a as u32)) / 255) as u8
                };
                params.dest.put_pixel(
                    dest_x + px,
                    dest_y + py,
                    image::Rgba([
                        blend(pixel[0], existing[0], alpha),
                        blend(pixel[1], existing[1], alpha),
                        blend(pixel[2], existing[2], alpha),
                        255,
                    ]),
                );
            }
        }
    }
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[test]
fn rgb16_565_produce_color_test() {
    let color = rgb16_565_produce_color(0);
    assert_eq!(color.r as i16 + color.g as i16 + color.b as i16, 0);
}

#[test]
fn rgb16_565_red_max() {
    let color = rgb16_565_produce_color(0xF800);
    assert_eq!(color.r, 248);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
}

#[test]
fn rgb16_565_green_max() {
    let color = rgb16_565_produce_color(0x07E0);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 252);
    assert_eq!(color.b, 0);
}

#[test]
fn rgb16_565_blue_max() {
    let color = rgb16_565_produce_color(0x001F);
    assert_eq!(color.r, 0);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 248);
}

#[test]
fn rgb16_565_white() {
    let color = rgb16_565_produce_color(0xFFFF);
    assert_eq!(color.r, 248);
    assert_eq!(color.g, 252);
    assert_eq!(color.b, 248);
}

#[test]
fn plot_atlas_tile_params() {
    use image::{ImageBuffer, Rgba, RgbaImage};

    let mut dest: RgbaImage = ImageBuffer::new(100, 100);
    let atlas: image::DynamicImage =
        image::DynamicImage::ImageRgba8(ImageBuffer::from_pixel(64, 64, Rgba([255, 0, 0, 255])));

    plot_atlas_tile(AtlasTileParams {
        dest: &mut dest,
        atlas: &atlas,
        src_x: 0,
        src_y: 0,
        tile_w: 32,
        tile_h: 32,
        dest_x: 10,
        dest_y: 10,
    });

    let pixel = dest.get_pixel(10, 10);
    assert_eq!(pixel[0], 255);
    assert_eq!(pixel[3], 255);
}

#[test]
fn test_blend_rgb() {
    let result = blend_rgb([255, 255, 255], [0, 0, 0], 128);
    assert_eq!(result, [127, 127, 127]);
}

#[test]
fn test_blend_rgb_full_alpha() {
    let result = blend_rgb([100, 100, 100], [200, 50, 150], 255);
    assert_eq!(result, [200, 50, 150]);
}

#[test]
fn test_blend_rgb_zero_alpha() {
    let result = blend_rgb([100, 100, 100], [200, 50, 150], 0);
    assert_eq!(result, [100, 100, 100]);
}

#[test]
fn test_draw_number_zero() {
    let mut img = ImageBuffer::new(50, 20);
    draw_number(&mut img, 25, 10, 0, [255; 3], 3);
    // Verify at least one white pixel was drawn (the zero pattern has center pixel lit)
    let has_white = img.pixels().any(|p| *p == Rgb([255, 255, 255]));
    assert!(has_white, "draw_number(0) should produce some white pixels");
}

#[test]
fn test_draw_number_negative() {
    let mut img = ImageBuffer::new(50, 20);
    draw_number(&mut img, 25, 10, -42, [255; 3], 3);
    let has_white = img.pixels().any(|p| *p == Rgb([255, 255, 255]));
    assert!(
        has_white,
        "draw_number(-42) should produce some white pixels"
    );
}

#[test]
fn test_draw_number_clips_at_max_digits() {
    let mut img = ImageBuffer::new(100, 20);
    draw_number(&mut img, 50, 10, 12345, [255; 3], 2);
    // With max_digits=2, only "45" should be visible (last 2 digits)
    let has_white = img.pixels().any(|p| *p == Rgb([255, 255, 255]));
    assert!(has_white, "draw_number with max_digits should still render");
}
