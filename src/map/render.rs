use image::{ImageBuffer, Rgb, Rgba};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::map::tileset::{TILE_HEIGHT, Tile, plot_tile, plot_tile_opaque};
use crate::sprite::{ImageInfo, rgb16_565_produce_color};
use byteorder::{LittleEndian, ReadBytesExt};

use super::types::{
    Coords, EventBlock, convert_map_coords_to_image_coords, internal_sprite_sort_key,
    tiled_object_sort_key,
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
    /// Darken tiles according to their shadow/fog-of-war level from the
    /// access-ref grid.
    pub show_shadows: bool,
    pub show_internal_sprites: bool,
    pub show_monsters: bool,
    pub show_npcs: bool,
    pub show_objects: bool,
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
            show_shadows: true,
            show_internal_sprites: true,
            show_monsters: true,
            show_npcs: true,
            show_objects: true,
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
        gtl_tileset,
        btl_tileset,
        map_id,
        game_path,
        toggles,
    } = config;

    // The renderer reproduces the observed occluded viewport — the region
    // the player actually sees — not the full map canvas.
    let image_width = data.model.occluded_map_in_pixels_width;
    let image_height = data.model.occluded_map_in_pixels_height;

    println!("{:?}", data.model);
    println!(
        "{}, {}",
        image_width.unsigned_abs(),
        image_height.unsigned_abs()
    );

    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::new(image_width.unsigned_abs(), image_height.unsigned_abs());

    // ── Pass 1: Ground tiles ──────────────────────────────────────────────
    if toggles.show_ground {
        plot_base(&mut imgbuf, &data.model, &data.gtl_tiles, gtl_tileset);
    }

    // ── Pre-load external entities (if game path given) ──────────────────
    let external = game_path.and_then(|gp| collect_external_entities(map_id, gp, &data.model).ok());

    // ── Pass 2: Interleaved objects + entities ───────────────────────────
    // All depth-relevant items (buildings, internal sprites, monsters, NPCs,
    // extras) are collected into one list, sorted by Y-depth with type
    // tiebreaker, then rendered together.
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
            TiledObject(usize),
            Sprite(usize),
            Monster(usize),
            Npc(usize),
            Extra(usize),
        }

        let mut items: Vec<(i32, i32, i32, ItemKind)> = Vec::new();

        // Buildings (type_order=0) — single units ordered by their stack bottom.
        // Keys are map-local, matching entity_pos below.
        if toggles.show_buildings {
            for (i, info) in data.tiled_infos.iter().enumerate() {
                let pos = tiled_object_sort_key(info.y, info.ids.len());
                items.push((pos, 0, info.x, ItemKind::TiledObject(i)));
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
                ItemKind::TiledObject(i) => {
                    let info = &data.tiled_infos[*i];
                    for (j, &btl_id) in info.ids.iter().enumerate() {
                        if btl_id <= 0 {
                            continue;
                        }
                        let btl_tile_idx = btl_id.unsigned_abs() as usize;
                        if let Some(tile) = btl_tileset.get(btl_tile_idx) {
                            let x = info.x;
                            let y = info.y + (j as i32 * TILE_HEIGHT as i32);
                            plot_tile(&mut imgbuf, tile.colors, x, y);
                        }
                    }
                }
                ItemKind::Sprite(i) => {
                    let block = &data.sprite_blocks[*i];
                    let sequence = &data.internal_sprites[block.sprite_id];
                    let sprite = &sequence.frame_infos[0];
                    let dest_x = block.sprite_x;
                    let dest_y = block.sprite_y;
                    plot_sprite_on_bitmap(&mut imgbuf, reader, sprite, dest_x, dest_y)?;
                }
                ItemKind::Monster(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.monsters[*i],
                            &mut sprite_cache,
                            &data.model,
                            diagonal,
                        );
                    }
                }
                ItemKind::Npc(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.npcs[*i],
                            &mut sprite_cache,
                            &data.model,
                            diagonal,
                        );
                    }
                }
                ItemKind::Extra(i) => {
                    if let Some(ref ext) = external {
                        render_entity_sprite(
                            &mut imgbuf,
                            &ext.extras[*i],
                            &mut sprite_cache,
                            &data.model,
                            diagonal,
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
            &data.btl_tiles,
            btl_tileset,
            &data.overlay_modes,
        );
    }

    // ── Pass 3b: Shadows ─────────────────────────────────────────────────
    // Reproduction of the observed lighting pass: on maps flagged
    // Dark in AllMap.ini, level-0 tiles are blacked out and tiles with a
    // light level are faded through the fogdata.dat tables. Drawn after
    // world pixels so annotations added later stay fully visible.
    if toggles.show_shadows
        && let Some(fog) = prepare_shadow_pass(game_path, map_id)
    {
        plot_shadows(&mut imgbuf, &data.model, &data.access_ref_words, &fog);
    }

    // ── Pass 4: Overlays ─────────────────────────────────────────────────
    let diagonal = data.model.tiled_map_width + data.model.tiled_map_height;

    if toggles.show_collisions {
        plot_collisions_overlay(&mut imgbuf, &data.model, &data.collisions, diagonal);
    }

    if toggles.show_events {
        plot_events_overlay(&mut imgbuf, &data.model, &data.events, diagonal);
    }

    if toggles.show_draw_items
        && let Some(ref ext) = external
    {
        plot_draw_items_overlay(&mut imgbuf, &ext.draw_items, &data.model, diagonal);
    }

    if toggles.show_npc_waypoints
        && let Some(ref ext) = external
    {
        plot_npc_waypoints_overlay(&mut imgbuf, &ext.npc_records, &data.model, diagonal);
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
                sx -= model.map_non_occluded_start_x;
                sy -= model.map_non_occluded_start_y;

                plot_tile(image, gtl_tile.colors, sx, sy);
            }
        }
    }
}

// --------------------------------------------------------------------------
// Object layer (sprites + tiled objects, sorted by ground-y)
// --------------------------------------------------------------------------

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
    btl_tiles: &HashMap<Coords, i32>,
    btl_tileset: &[Tile],
    overlay_modes: &[u16],
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
                    // Per-id mode from the map's overlay table: high byte is
                    // the draw-enable flag (0 hides the overlay), low byte
                    // selects the blit mode — 0 draws opaquely, any other
                    // value skips black pixels.
                    let entry = overlay_modes.get(btl_tile_idx).copied().unwrap_or(0);
                    if entry >> 8 == 0 {
                        continue;
                    }
                    let (mut sx, mut sy) =
                        convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
                    sx -= model.map_non_occluded_start_x;
                    sy -= model.map_non_occluded_start_y;
                    if entry & 0xFF == 0 {
                        plot_tile_opaque(image, btl_tile.colors, sx, sy);
                    } else {
                        plot_tile(image, btl_tile.colors, sx, sy);
                    }
                }
            }
        }
    }
}

// --------------------------------------------------------------------------
// Shadow overlay – reproduction of the observed lighting pass
//
// Applied near the end of the world render sequence. The pass only runs on
// maps flagged Dark in AllMap.ini — an observed per-map-catalog flag,
// toggleable at runtime via the "Turn off the light" / "Honey, It's too
// dark" cheat codes.
//
// Per tile, with level = (access_ref_word >> 15) & 0x7FFF:
//   level ≥ 124    → untouched by the pass (124..=199 fit the level field
//                    but read past the fade tables = undefined; levels
//                    ≥ 200 are simply not darkened)
//   level == 0     → span diamond filled solid black (fog of war)
//   level 1..=123  → per pixel pair, red and green are scaled by f/32 where
//                    f = fogdata.dat[(level-1)*512 + pair]; blue is never
//                    modified. Flag bits 30–31 mark tiles whose factor is
//                    blended with dynamic entity lights via max(); statically
//                    we render the tile's own pattern.
// --------------------------------------------------------------------------

/// Per-row `(x_start, width)` spans of the shadow mask (32 rows). The rows
/// trace the 64×32 isometric tile diamond; each row is processed as
/// `width/2` pixel pairs sharing one fade factor.
pub const SHADOW_SPANS: [(i32, i32); 32] = [
    (30, 2),
    (28, 6),
    (26, 10),
    (24, 14),
    (22, 18),
    (20, 22),
    (18, 26),
    (16, 30),
    (14, 34),
    (12, 38),
    (10, 42),
    (8, 46),
    (6, 50),
    (4, 54),
    (2, 58),
    (0, 62),
    (0, 62),
    (2, 58),
    (4, 54),
    (6, 50),
    (8, 46),
    (10, 42),
    (12, 38),
    (14, 34),
    (16, 30),
    (18, 26),
    (20, 22),
    (22, 18),
    (24, 14),
    (26, 10),
    (28, 6),
    (30, 2),
];

pub use super::fogdata::FogData;

/// Loads the fade tables when the map `map_stem` is flagged Dark in
/// `<game_path>/AllMap.ini`. Returns `None` for Light maps (the game skips
/// its lighting pass there) and reports IO problems on stderr.
pub fn load_fog_if_dark(game_path: &Path, map_stem: &str) -> Option<FogData> {
    use crate::references::all_map_ini::read_all_map_ini;
    use crate::references::enums::MapLighting;

    let all_map_path = game_path.join("AllMap.ini");
    let maps = match read_all_map_ini(&all_map_path) {
        Ok(maps) => maps,
        Err(e) => {
            eprintln!("Warning: could not read {all_map_path:?} ({e}); rendering without shadows");
            return None;
        }
    };

    let is_dark = maps
        .iter()
        .any(|m| m.map_filename.eq_ignore_ascii_case(map_stem) && m.lighting == MapLighting::Dark);
    if !is_dark {
        return None;
    }

    match FogData::load(game_path) {
        Ok(fog) => Some(fog),
        Err(e) => {
            eprintln!("Warning: could not load fogdata.dat ({e}); rendering without shadows");
            None
        }
    }
}

/// Loads the fade tables when the rendered map is flagged Dark in AllMap.ini.
///
/// Returns `None` when shadows must not be drawn: no game path or a Light
/// map (see [`load_fog_if_dark`]).
fn prepare_shadow_pass(game_path: Option<&Path>, map_id: &str) -> Option<FogData> {
    let game_path = game_path?;
    let base_name = map_id.split('.').next().unwrap_or(map_id);
    load_fog_if_dark(game_path, base_name)
}

/// Applies the observed per-tile lighting pass over the rendered world pixels.
///
/// Levels `1..=[super::fogdata::ROWS]` index the fade tables; levels above
/// `ROWS` (including the accepted-but-undefined `124..=199` range,
/// which would read past the 123-row file) are skipped — the tile is left
/// untouched. Reading out of bounds there is undefined behavior;
/// tooling must not. Level 0 tiles are blacked out outright.
fn plot_shadows(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    words: &HashMap<Coords, u32>,
    fog: &FogData,
) {
    let map_diagonal_tiles = model.tiled_map_width + model.tiled_map_height;

    for (&(x, y), &word) in words {
        let level = (word >> 15) & 0x7FFF;
        if level as usize > super::fogdata::ROWS {
            continue;
        }

        let (mut px, mut py) = convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
        px -= model.map_non_occluded_start_x;
        py -= model.map_non_occluded_start_y;

        let mut pair = 0usize;
        for (row, &(start, width)) in SHADOW_SPANS.iter().enumerate() {
            let py_row = py + row as i32;
            if py_row < 0 {
                continue;
            }
            for p in 0..(width as usize / 2) {
                let x0 = px + start + (p * 2) as i32;
                if x0 < 0 {
                    continue;
                }
                if level == 0 {
                    for dx in [0, 1] {
                        if let Some(pixel) =
                            image.get_pixel_mut_checked((x0 + dx) as u32, py_row as u32)
                        {
                            *pixel = Rgb([0, 0, 0]);
                        }
                    }
                } else {
                    // Both pixels of the pair share one fade factor byte.
                    let f = u32::from(fog.factor(level, pair));
                    for dx in [0, 1] {
                        if let Some(pixel) =
                            image.get_pixel_mut_checked((x0 + dx) as u32, py_row as u32)
                        {
                            pixel[0] = (u32::from(pixel[0]) * f / 32) as u8;
                            pixel[1] = (u32::from(pixel[1]) * f / 32) as u8;
                            // Blue stays untouched, as observed.
                        }
                    }
                }
                pair += 1;
            }
        }
    }
}

// --------------------------------------------------------------------------
// External entity collection and rendering
// --------------------------------------------------------------------------

/// Loads all external entity data (monsters, NPCs, extras, draw items) for the
/// given map from the game data files.
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
                let (x, y) = npc_placement(&n);

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

    // Load draw items for this map. A draw item's map reference is the
    // numeric ID from AllMap.ini — resolve it by filename (parsing digits
    // from the stem gives wrong IDs: "cat1" → 1 instead of 3).
    let map_draw_id = resolve_all_map_id(map_base_name, game_path);
    let draw_items: Vec<DrawItem> = match map_draw_id {
        Some(id) => DrawItem::read_file(&game_path.join("Ref").join("DRAWITEM.ref"))
            .unwrap_or_default()
            .into_iter()
            .filter(|d| d.map_id == id)
            .collect(),
        None => Vec::new(),
    };

    Ok(ExternalEntities {
        monsters,
        npcs,
        extras,
        npc_records,
        draw_items,
    })
}

/// Resolves a map file stem (e.g. `"cat1"`) to its AllMap.ini numeric ID.
///
/// Mirrors the GUI's `resolve_map_id`; returns `None` when the map is not
/// listed in AllMap.ini.
fn resolve_all_map_id(map_base_name: &str, game_path: &Path) -> Option<i32> {
    use crate::references::all_map_ini::read_all_map_ini;
    read_all_map_ini(&game_path.join("AllMap.ini"))
        .ok()?
        .into_iter()
        .find(|m| m.map_filename.eq_ignore_ascii_case(map_base_name))
        .map(|m| m.id)
}

/// Where an NPC stands: its first active waypoint, falling back to waypoint 1.
///
/// Shared by the renderer and the placement tests; the GUI's
/// `hit_test::npc_pos` implements the same rule.
pub fn npc_placement(n: &crate::references::npc_ref::NPC) -> (i32, i32) {
    [
        (i32::from(n.goto1_filled), n.goto1_x, n.goto1_y),
        (i32::from(n.goto2_filled), n.goto2_x, n.goto2_y),
        (i32::from(n.goto3_filled), n.goto3_x, n.goto3_y),
        (i32::from(n.goto4_filled), n.goto4_x, n.goto4_y),
    ]
    .iter()
    .find(|(filled, _, _)| *filled != 0)
    .map(|&(_, x, y)| (x, y))
    .unwrap_or((n.goto1_x, n.goto1_y))
}

/// Render a single external entity sprite (or fallback marker) onto the image.
fn render_entity_sprite(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    entity: &EntityRenderInfo,
    sprite_cache: &mut HashMap<PathBuf, Option<Vec<super::sprite_loader::LoadedSpriteFrame>>>,
    model: &MapModel,
    diagonal: i32,
) {
    // Entities live in absolute canvas space; shift into the occluded
    // viewport like every other layer by subtracting the map origin,
    // matching the observed on-screen placement.
    let (px, py) = convert_map_coords_to_image_coords(entity.x, entity.y, diagonal);
    let cx = px - model.map_non_occluded_start_x + super::tileset::TILE_WIDTH as i32 / 2;
    let cy = py - model.map_non_occluded_start_y + TILE_HEIGHT as i32 / 2;

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
            px -= model.map_non_occluded_start_x;
            py -= model.map_non_occluded_start_y;
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
            let event = events
                .get(&coords)
                .copied()
                .unwrap_or(EventBlock { x, y, word: 0 });
            if event.event_id() == 0 {
                continue;
            }
            let (mut px, mut py) = convert_map_coords_to_image_coords(x, y, diagonal);
            px -= model.map_non_occluded_start_x;
            py -= model.map_non_occluded_start_y;
            let cx = px + super::tileset::TILE_WIDTH as i32 / 2;
            let cy = py + TILE_HEIGHT as i32 / 2;
            // Magenta dot
            fill_circle_blended(imgbuf, cx, cy, 3, [200, 25, 200], 180);
            // Event ID label above the dot
            draw_number(
                imgbuf,
                cx,
                cy - 8,
                event.event_id() as i32,
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
    diagonal: i32,
) {
    for di in draw_items {
        let (mut px, mut py) = convert_map_coords_to_image_coords(di.x_coord, di.y_coord, diagonal);
        px -= model.map_non_occluded_start_x;
        py -= model.map_non_occluded_start_y;
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

            let sx = sx - model.map_non_occluded_start_x;
            let sy = sy - model.map_non_occluded_start_y;
            let ex = ex - model.map_non_occluded_start_x;
            let ey = ey - model.map_non_occluded_start_y;

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

#[test]
fn shadow_spans_trace_tile_diamond() {
    assert_eq!(SHADOW_SPANS.len(), 32, "one span row per tile pixel row");
    let mut pairs = 0usize;
    for (i, &(start, width)) in SHADOW_SPANS.iter().enumerate() {
        assert_eq!(width % 2, 0, "rows are processed as pixel pairs");
        assert!(
            start >= 0 && start + width <= 64,
            "span must fit tile width"
        );
        assert_eq!(
            (start, width),
            SHADOW_SPANS[31 - i],
            "mask must be vertically symmetric"
        );
        pairs += width as usize / 2;
    }
    assert_eq!(
        pairs, 512,
        "diamond covers exactly one fogdata row of pairs"
    );
}

/// Builds a canvas filled with a known color and returns it together with
/// the image position of map tile (0, 0).
#[cfg(test)]
fn shadow_test_canvas() -> (ImageBuffer<Rgb<u8>, Vec<u8>>, i32, i32) {
    let model = test_model();
    let diagonal = model.tiled_map_width + model.tiled_map_height;
    let (px, py) = convert_map_coords_to_image_coords(0, 0, diagonal);
    let img = image::ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(128, 128, Rgb([200, 100, 50]));
    (img, px, py)
}

#[cfg(test)]
fn test_model() -> MapModel {
    MapModel {
        tiled_map_width: 1,
        tiled_map_height: 1,
        ..MapModel::default()
    }
}

#[test]
fn level_zero_tile_is_blacked_out_inside_spans_only() {
    let (mut img, px, py) = shadow_test_canvas();
    let fog = super::fogdata::FogData::from_raw(vec![31; super::fogdata::EXPECTED_LEN]);
    let mut words = HashMap::new();
    words.insert((0, 0), 0u32);
    plot_shadows(&mut img, &test_model(), &words, &fog);

    // Center of the diamond: solid black.
    assert_eq!(img.get_pixel((px + 32) as u32, (py + 16) as u32)[0], 0);
    // Corner outside the diamond: untouched.
    assert_eq!(img.get_pixel(px as u32, py as u32)[0], 200);
}

#[test]
fn lit_tiles_scale_red_green_pairwise_and_keep_blue() {
    // Level 1 → fogdata row 0. Give each pair its own factor to prove the
    // pair index walks the spans and both pixels share it.
    let mut data = vec![0u8; super::fogdata::EXPECTED_LEN];
    for (pair, slot) in data.iter_mut().enumerate().take(512) {
        *slot = (pair % 32) as u8;
    }
    let fog = super::fogdata::FogData::from_raw(data);

    let (mut img, px, py) = shadow_test_canvas();
    let mut words = HashMap::new();
    words.insert((0, 0), 1u32 << 15);
    plot_shadows(&mut img, &test_model(), &words, &fog);

    // First span row: start=30, width=2 → one pair at x=px+30.
    let f0 = u32::from(fog.factor(1, 0));
    let left = img.get_pixel((px + 30) as u32, py as u32);
    assert_eq!(left[0], (200 * f0 / 32) as u8, "red scaled by f/32");
    assert_eq!(left[1], (100 * f0 / 32) as u8, "green scaled by f/32");
    assert_eq!(left[2], 50, "blue must stay untouched");

    // A later pair (row 15's first pair) uses its own factor byte.
    let &(start, _) = &SHADOW_SPANS[15];
    let pair_index: usize = SHADOW_SPANS[..15]
        .iter()
        .map(|&(_, w)| w as usize / 2)
        .sum();
    let f = u32::from(fog.factor(1, pair_index));
    let p = img.get_pixel((px + start) as u32, (py + 15) as u32);
    assert_eq!(p[0], (200 * f / 32) as u8);
    assert_eq!(p[1], (100 * f / 32) as u8);
    assert_eq!(p[2], 50);
}

#[test]
fn levels_at_or_above_200_are_untouched() {
    // 124..=199 fit the level field but have no data rows
    // (the table covers 1..=123); they must be skipped, not panic.
    for level in [124u32, 150, 199, 200, 250, 0x7FFF] {
        let (mut img, _px, _py) = shadow_test_canvas();
        let fog = super::fogdata::FogData::from_raw(vec![0; super::fogdata::EXPECTED_LEN]);
        let mut words = HashMap::new();
        words.insert((0, 0), level << 15);
        plot_shadows(&mut img, &test_model(), &words, &fog);
        assert!(
            img.pixels().all(|p| *p == Rgb([200, 100, 50])),
            "level {level} tiles must not be modified"
        );
    }
}

#[test]
fn fogdata_loads_from_fixture_game_dir() {
    let fog = FogData::load(Path::new("fixtures/Dispel")).expect("fixture fogdata.dat");
    // Known values from the shipped file: level 1 is uniformly dark (f=2),
    // level 65 is fully bright (f=31).
    assert_eq!(fog.factor(1, 0), 2);
    assert_eq!(fog.factor(65, 256), 31);
}

#[test]
fn entity_marker_is_placed_in_viewport_space() {
    // Regression: external entities must be shifted into the occluded
    // viewport like every other layer — subtracting the map origin
    // when placing NPCs/monsters/extras, as observed.
    let model = MapModel {
        tiled_map_width: 1,
        tiled_map_height: 1,
        map_non_occluded_start_x: 100,
        map_non_occluded_start_y: 20,
        ..MapModel::default()
    };
    let diagonal = model.tiled_map_width + model.tiled_map_height;

    let mut img = ImageBuffer::from_pixel(512, 512, Rgb([0, 0, 0]));
    let entity = EntityRenderInfo {
        x: 3,
        y: 5,
        fallback_color: [255, 0, 0],
        sprite_path: None,
        sequence: 0,
        flip: false,
    };
    let mut cache = HashMap::new();
    render_entity_sprite(&mut img, &entity, &mut cache, &model, diagonal);

    let (px, py) = convert_map_coords_to_image_coords(3, 5, diagonal);
    let cx = (px - model.map_non_occluded_start_x) + super::tileset::TILE_WIDTH as i32 / 2;
    let cy = (py - model.map_non_occluded_start_y) + TILE_HEIGHT as i32 / 2;
    assert_eq!(
        img.get_pixel(cx as u32, cy as u32)[0],
        255,
        "fallback marker must sit at the viewport-space tile center"
    );
}

#[test]
fn draw_item_map_id_resolves_via_all_map_ini() {
    // Regression: the draw-item map reference is the AllMap.ini numeric ID.
    // Parsing digits from the file stem gave wrong IDs ("cat1" → 1 instead
    // of 3), rendering another map's item drops.
    let gp = std::path::Path::new("fixtures/Dispel");
    if !gp.exists() {
        eprintln!("Skipping: fixtures not found");
        return;
    }
    assert_eq!(resolve_all_map_id("cat1", gp), Some(3));
    assert_eq!(resolve_all_map_id("dun01", gp), Some(7));
    assert_eq!(resolve_all_map_id("notamap", gp), None);
}

#[test]
fn roof_overlay_modes_gate_and_pick_blit_style() {
    let model = MapModel {
        tiled_map_width: 1,
        tiled_map_height: 1,
        ..MapModel::default()
    };
    let diagonal = model.tiled_map_width + model.tiled_map_height;
    let (px, py) = convert_map_coords_to_image_coords(0, 0, diagonal);
    let (sx, sy) = (
        px - model.map_non_occluded_start_x,
        py - model.map_non_occluded_start_y,
    );

    // Tile id 1: half black, half red. Ids 2/3: fully red.
    let mut colors = [crate::sprite::Color { r: 255, g: 0, b: 0 }; 1024];
    for c in colors.iter_mut().take(512) {
        *c = crate::sprite::Color { r: 0, g: 0, b: 0 };
    }
    let solid_red = [crate::sprite::Color { r: 255, g: 0, b: 0 }; 1024];
    let tileset = vec![
        Tile { colors: solid_red },
        Tile { colors },
        Tile { colors: solid_red },
    ];

    let mut btl_tiles = HashMap::new();
    btl_tiles.insert((0, 0), 1i32);

    let mut img = image::ImageBuffer::from_pixel(128, 128, Rgb([10, 20, 30]));

    // Draw-enable high byte 0 → tile must not be drawn at all.
    let modes = |hi: u16, lo: u16| vec![0u16, (hi << 8) | lo, 0, 0];
    plot_roofs(&mut img, &model, &btl_tiles, &tileset, &modes(0, 1));
    assert!(
        img.get_pixel(sx as u32 + 32, sy as u32 + 16)[0] == 10,
        "disabled overlay must be skipped"
    );

    // Transparent mode (low byte nonzero) → black pixels skipped, red drawn.
    plot_roofs(&mut img, &model, &btl_tiles, &tileset, &modes(1, 1));
    assert_eq!(
        img.get_pixel(sx as u32 + 32, sy as u32 + 16)[0],
        255,
        "red pixel drawn"
    );
    assert_eq!(
        img.get_pixel(sx as u32 + 30, sy as u32)[0],
        10,
        "black pixel (top rows) skipped in transparent mode"
    );

    // Opaque mode (low byte 0) → black pixels written.
    let mut img2 = image::ImageBuffer::from_pixel(128, 128, Rgb([10, 20, 30]));
    plot_roofs(&mut img2, &model, &btl_tiles, &tileset, &modes(1, 0));
    assert_eq!(
        img2.get_pixel(sx as u32 + 30, sy as u32)[0],
        0,
        "black pixel (top rows) written in opaque mode"
    );
}
