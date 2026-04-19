use image::{ImageBuffer, Rgb};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, Seek, SeekFrom};

use crate::map::tileset::{mix_color, plot_tile, Tile, TILE_HEIGHT};
use crate::sprite::{rgb16_565_produce_color, Color, ImageInfo, SequenceInfo};
use byteorder::{LittleEndian, ReadBytesExt};

use super::types::{
    convert_map_coords_to_image_coords, Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo,
};

use super::model::MapModel;

// --------------------------------------------------------------------------
// Top-level render entry point
// --------------------------------------------------------------------------

/// Configuration for rendering a map
pub struct MapRenderConfig<'a> {
    pub reader: &'a mut BufReader<File>,
    pub output_path: &'a std::path::Path,
    pub data: &'a super::MapData,
    pub occlusion: bool,
    pub gtl_tileset: &'a [Tile],
    pub btl_tileset: &'a [Tile],
    pub map_id: &'a str,
    pub game_path: Option<&'a std::path::Path>,
}

/// Renders the full map to a PNG file, compositing ground → objects → roofs → entities.
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

    plot_base(
        &mut imgbuf,
        &data.model,
        occlusion,
        &data.gtl_tiles,
        gtl_tileset,
        &data.collisions,
        &data.events,
    );
    plot_objects(
        &mut imgbuf,
        reader,
        &data.model,
        occlusion,
        &data.btl_tiles,
        PlotObjectsParams {
            btl_tileset,
            tiled_info: &data.tiled_infos,
            internal_sprites: &data.internal_sprites,
            sprite_blocks: &data.sprite_blocks,
            offset_x,
            offset_y,
        },
    )?;
    plot_roofs(
        &mut imgbuf,
        &data.model,
        occlusion,
        &data.btl_tiles,
        btl_tileset,
    );

    // Pass 4: external entities (monsters, NPCs, extras) — markers or sprites
    if let Some(gp) = game_path {
        plot_external_entities(&mut imgbuf, &data.model, occlusion, map_id, gp);
    }

    imgbuf
        .save(output_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}

// --------------------------------------------------------------------------
// Ground layer
// --------------------------------------------------------------------------

pub fn plot_base(
    image: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    occlusion: bool,
    gtl_tiles: &HashMap<Coords, i32>,
    gtl_tileset: &[Tile],
    collisions: &HashMap<Coords, bool>,
    events: &HashMap<Coords, EventBlock>,
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

                let event_block = events.get(&coords);
                let collision = collisions.get(&coords);

                let (mut sx, mut sy) = convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
                if occlusion {
                    sx -= model.map_non_occluded_start_x;
                    sy -= model.map_non_occluded_start_y;
                }

                let event_id = event_block.map(|e| e.event_id).unwrap_or(0);
                let _is_collision = collision.copied().unwrap_or(false);

                let tile_colors = if event_id > 0 {
                    mix_color(
                        gtl_tile.colors,
                        Color {
                            r: 200,
                            b: 10,
                            g: 255,
                        },
                        50,
                    )
                } else {
                    gtl_tile.colors
                };

                plot_tile(image, tile_colors, sx, sy);
            }
        }
    }
}

// --------------------------------------------------------------------------
// Object layer (sprites + tiled objects, sorted by ground-y)
// --------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct PlotObjectsParams<'a> {
    btl_tileset: &'a [Tile],
    tiled_info: &'a [TiledObjectInfo],
    internal_sprites: &'a [SequenceInfo],
    sprite_blocks: &'a [SpriteInfoBlock],
    offset_x: i32,
    offset_y: i32,
}

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
        TiledObject(usize),
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
    for (i, info) in params.tiled_info.iter().enumerate() {
        items.push(Item {
            ground_y: info.y + (info.ids.len() as i32 * TILE_HEIGHT as i32),
            kind: Kind::TiledObject(i),
        });
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
            Kind::TiledObject(i) => plot_single_tiled_object(
                imgbuf,
                &params.tiled_info[i],
                params.btl_tileset,
                params.offset_x,
                params.offset_y,
            ),
        }
    }
    Ok(())
}

fn plot_single_tiled_object(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    tiled_info: &TiledObjectInfo,
    btl_tileset: &[Tile],
    offset_x: i32,
    offset_y: i32,
) {
    for (i, btl_id) in tiled_info.ids.iter().enumerate() {
        let btl_tile_idx = btl_id.unsigned_abs() as usize;
        if let Some(tile) = btl_tileset.get(btl_tile_idx) {
            let x = tiled_info.x + offset_x;
            let y = tiled_info.y + (i as i32 * TILE_HEIGHT as i32) + offset_y;
            plot_tile(imgbuf, tile.colors, x, y);
        }
    }
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
// External entity rendering (monsters, NPCs, extras)
// --------------------------------------------------------------------------

fn plot_external_entities(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    model: &MapModel,
    occlusion: bool,
    map_id: &str,
    game_path: &std::path::Path,
) {
    use crate::references::{
        extra_ini::Extra, extra_ref::ExtraRef, extractor::Extractor, map_ini::read_map_ini,
        monster_ini::MonsterIni, monster_ref::MonsterRef, npc_ini::NpcIni, npc_ref::NPC,
    };
    use std::collections::HashMap;

    let map_ini_path = game_path.join("Ref").join("Map.ini");
    if !map_ini_path.exists() {
        return;
    }

    // Extract base name without extension (e.g., "cat1" from "cat1.map")
    let map_base_name = map_id.split('.').next().unwrap_or(map_id);

    let map_inis = match read_map_ini(&map_ini_path) {
        Ok(v) => v,
        Err(_) => return,
    };

    // Try to find matching map ini by filename pattern
    let map_ini = map_inis.into_iter().find(|ini| {
        // Check all possible filename fields for a match
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
        println!(
            "  Warning: No Map.ini entry found for map '{}'",
            map_base_name
        );
        return;
    };

    let diagonal = model.tiled_map_width + model.tiled_map_height;
    // When rendering in occluded mode the viewport is cropped: subtract the
    // non-occluded pixel offset so entity screen coords match the tile layer.
    let offset_px_x = if occlusion {
        model.map_non_occluded_start_x
    } else {
        0
    };
    let offset_px_y = if occlusion {
        model.map_non_occluded_start_y
    } else {
        0
    };

    // Try filename with different casings (case-insensitive lookup)
    // Always prefer uppercase for sprite files
    let resolve = |dir: &str, filename: &str| -> std::path::PathBuf {
        // Try uppercase version first (preferred)
        let upper = filename.to_ascii_uppercase();
        let p_upper = game_path.join(dir).join(&upper);
        if p_upper.exists() {
            println!(
                "  Resolve: '{}' -> '{}' (uppercase)",
                filename,
                p_upper.display()
            );
            return p_upper;
        }

        // Try original case
        let p = game_path.join(dir).join(filename);
        if p.exists() {
            println!("  Resolve: '{}' -> '{}' (original)", filename, p.display());
            return p;
        }

        // Try lowercase version
        let lower = filename.to_ascii_lowercase();
        let p_lower = game_path.join(dir).join(&lower);
        if p_lower.exists() {
            println!(
                "  Resolve: '{}' -> '{}' (lowercase)",
                filename,
                p_lower.display()
            );
            return p_lower;
        }

        // Try capitalized version (first letter uppercase)
        let mut capitalized = filename.to_string();
        if let Some(c) = capitalized.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        let p_cap = game_path.join(dir).join(&capitalized);
        if p_cap.exists() {
            println!(
                "  Resolve: '{}' -> '{}' (capitalized)",
                filename,
                p_cap.display()
            );
            return p_cap;
        }

        // Return original path even if it doesn't exist (will show warning later)
        println!(
            "  Resolve: '{}' -> '{}' (not found, using original)",
            filename,
            p.display()
        );
        p
    };

    // Build id→sprite_filename maps from the ini files (best-effort)
    let monster_sprites: HashMap<i32, String> =
        MonsterIni::read_file(&game_path.join("Monster.ini"))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|m| m.sprite_filename.map(|s| (m.id, s)))
            .collect();
    let npc_sprites: HashMap<i32, String> = NpcIni::read_file(&game_path.join("Npc.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|n| n.sprite_filename.map(|s| (n.id, s)))
        .collect();
    let extra_sprites: HashMap<i32, String> = Extra::read_file(&game_path.join("Extra.ini"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.sprite_filename.map(|s| (e.id, s)))
        .collect();

    struct Entity {
        x: i32,
        y: i32,
        fallback_color: [u8; 3],
        sprite_path: Option<std::path::PathBuf>,
        sequence: usize,
        flip: bool,
    }
    let mut entities: Vec<Entity> = Vec::new();

    if let Some(f) = map_ini.monsters_filename {
        let p = resolve("MonsterInGame", &f);
        if let Ok(data) = MonsterRef::read_file(&p) {
            for m in data {
                let sprite_path = monster_sprites
                    .get(&m.mon_id)
                    .map(|s| resolve("MonsterInGame", s));
                entities.push(Entity {
                    x: m.pos_x,
                    y: m.pos_y,
                    fallback_color: [220, 50, 50],
                    sprite_path,
                    sequence: 3, // C# uses sequence=3 for standing monsters
                    flip: false,
                });
            }
        }
    } else {
        println!(
            "  Warning: no MonsterInGame file found: {:?}",
            map_ini.monsters_filename
        );
    }
    if let Some(f) = map_ini.npc_filename {
        let p = resolve("NpcInGame", &f);
        if let Ok(data) = NPC::read_file(&p) {
            for n in data {
                // Find first active waypoint
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
                    .unwrap_or((n.goto1_x, n.goto1_y)); // Fallback to goto1 if none active

                let dir = i32::from(n.looking_direction);
                let (seq, flip) = if dir > 4 {
                    ((8 - dir) as usize, true)
                } else {
                    (dir as usize, false)
                };
                let sprite_path = npc_sprites.get(&n.npc_id).map(|s| resolve("NpcInGame", s));
                entities.push(Entity {
                    x,
                    y,
                    fallback_color: [50, 100, 220],
                    sprite_path,
                    sequence: seq,
                    flip,
                });
            }
        }
    } else {
        println!(
            "  Warning: no NpcInGame file found: {:?}",
            map_ini.npc_filename
        );
    }
    if let Some(f) = map_ini.extra_filename {
        let p = resolve("ExtraInGame", &f);
        if let Ok(data) = ExtraRef::read_file(&p) {
            for e in data {
                let rotation = e.rotation as i32;
                let obj_type = u8::from(e.object_type) as i32;
                let seq = if obj_type == 0 {
                    (2 * i32::from(e.closed) + rotation) as usize
                } else {
                    rotation as usize
                };
                let sprite_path = extra_sprites
                    .get(&(e.ext_id as i32))
                    .map(|s| resolve("ExtraInGame", s));
                entities.push(Entity {
                    x: e.x_pos,
                    y: e.y_pos,
                    fallback_color: [200, 180, 30],
                    sprite_path,
                    sequence: seq,
                    flip: false,
                });
            }
        }
    } else {
        println!(
            "  Warning: no ExtraInGame file found: {:?}",
            map_ini.extra_filename
        );
    }

    println!("  Rendering {} external entities", entities.len());

    let mut sprite_cache: HashMap<
        std::path::PathBuf,
        Option<Vec<super::sprite_loader::LoadedSpriteFrame>>,
    > = HashMap::new();

    let mut rendered_count = 0;
    let mut fallback_count = 0;

    for entity in &entities {
        let (px, py) = convert_map_coords_to_image_coords(entity.x, entity.y, diagonal);
        // Center on tile and apply viewport offset
        let cx = px - offset_px_x + super::tileset::TILE_WIDTH as i32 / 2;
        let cy = py - offset_px_y + TILE_HEIGHT as i32 / 2;

        let mut rendered = false;
        if let Some(ref sp) = entity.sprite_path {
            // Check if sprite file exists before trying to load
            if !sp.exists() {
                println!("  Warning: sprite file not found: {:?}", sp);
            } else {
                println!("  Rendering sprite: {:?}", sp);
                if !sprite_cache.contains_key(sp) {
                    let frames = super::sprite_loader::load_sprite_frames(sp);
                    println!("    Loaded frames: {:?}", frames.as_ref().map(|v| v.len()));
                    sprite_cache.insert(sp.clone(), frames);
                }
                match sprite_cache.get(sp) {
                    Some(Some(frames)) => {
                        if !frames.is_empty() {
                            let idx = entity.sequence.min(frames.len() - 1);
                            let frame = &frames[idx];
                            let dest_x = if entity.flip {
                                cx - (frame.image.width() as i32 - frame.origin_x)
                            } else {
                                cx - frame.origin_x
                            };
                            let dest_y = cy - frame.origin_y;
                            plot_rgba_sprite_on_rgb(
                                imgbuf,
                                &frame.image,
                                dest_x,
                                dest_y,
                                entity.flip,
                            );
                            rendered = true;
                        }
                    }
                    Some(None) => {
                        println!("    Warning: Failed to load sprite frames for {:?}", sp);
                    }
                    None => {
                        println!("    Warning: Sprite not in cache for {:?}", sp);
                    }
                }
            }
        } else {
            println!(
                "  Warning: no sprite filename found for NPC at ({}, {})",
                entity.x, entity.y
            );
        }
        if !rendered {
            println!(
                "  Debug: Entity at ({}, {}) fell back to marker",
                entity.x, entity.y
            );
            if let Some(ref sp) = entity.sprite_path {
                println!("    Sprite path: {:?}", sp);
                println!("    Exists: {}", sp.exists());
            } else {
                println!("    No sprite path available");
            }
            plot_rgb_marker(imgbuf, cx, cy, entity.fallback_color);
            fallback_count += 1;
        } else {
            rendered_count += 1;
        }
    }

    println!(
        "  Successfully rendered {}/{} external entities",
        rendered_count,
        entities.len()
    );
    if fallback_count > 0 {
        println!(
            "  Warning: {}/{} entities fell back to markers (missing sprites)",
            fallback_count,
            entities.len()
        );
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
