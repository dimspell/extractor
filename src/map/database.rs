use std::collections::HashMap;
use std::io::Result;
use std::path::Path;

use crate::map::tileset;

use super::render::{plot_atlas_tile, AtlasTileParams};
use super::sprite_loader::{load_sprite_frames, plot_entity_sprite};
use super::types::{
    convert_map_coords_to_image_coords, TiledObjectInfo, TILE_HEIGHT_HALF,
    TILE_HORIZONTAL_OFFSET_HALF, TILE_WIDTH_HALF,
};

/// Configuration for rendering a map from database
pub struct RenderConfig<'a> {
    pub database_path: &'a Path,
    pub map_id: &'a str,
    pub gtl_atlas_path: &'a Path,
    pub btl_atlas_path: &'a Path,
    pub atlas_columns: u32,
    pub output_path: &'a Path,
    pub game_path: Option<&'a Path>,
}

/// Renders a map image from data stored in the SQLite database together with
/// pre-built tileset atlas PNG files.
///
/// # Arguments
/// * `config` - Render configuration
pub fn render_from_database(config: RenderConfig) -> Result<()> {
    let RenderConfig {
        database_path,
        map_id,
        gtl_atlas_path,
        btl_atlas_path,
        atlas_columns,
        output_path,
        game_path,
    } = config;
    use image::RgbaImage;
    use rusqlite::Connection;

    println!("Loading atlases...");
    let gtl_atlas =
        image::open(gtl_atlas_path).map_err(|e| std::io::Error::other(e.to_string()))?;
    let btl_atlas =
        image::open(btl_atlas_path).map_err(|e| std::io::Error::other(e.to_string()))?;

    println!("Opening database...");
    let conn = Connection::open(database_path).map_err(|e| std::io::Error::other(e.to_string()))?;

    // ── Map tile bounds ────────────────────────────────────────────────────
    let bounds: (Option<i32>, Option<i32>, Option<i32>, Option<i32>) = conn
        .query_row(
            "SELECT MIN(x), MAX(x), MIN(y), MAX(y) FROM map_tiles WHERE map_id = ?",
            [map_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let (min_x, max_x, min_y, max_y) = match bounds {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => {
            return Err(std::io::Error::other(format!(
                "Map '{}' not found in database or has no tiles",
                map_id
            )));
        }
    };

    let map_width = max_x - min_x + 1;
    let map_height = max_y - min_y + 1;

    println!(
        "Map bounds: x=[{}, {}], y=[{}, {}], size={}x{}",
        min_x, max_x, min_y, max_y, map_width, map_height
    );

    // ── Tile data ──────────────────────────────────────────────────────────
    println!("Fetching tiles and objects...");
    let mut gtl_tiles = HashMap::new();
    let mut btl_tiles = HashMap::new();

    {
        let mut stmt = conn
            .prepare("SELECT x, y, gtl_tile_id, btl_tile_id FROM map_tiles WHERE map_id = ?")
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let iter = stmt
            .query_map([map_id], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                ))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        for row in iter {
            let (x, y, gtl, btl) = row.map_err(|e| std::io::Error::other(e.to_string()))?;
            if gtl > 0 {
                gtl_tiles.insert((x, y), gtl);
            }
            if btl > 0 {
                btl_tiles.insert((x, y), btl);
            }
        }
    }

    // ── Tiled objects ──────────────────────────────────────────────────────
    // Preserve object_index as tiebreaker: equal PositionOrder (y + size*TILE_HEIGHT)
    // must resolve in object_index order, matching C# IInterlacedOrderObjectComparer.
    let mut objects_map: HashMap<i32, (i32, TiledObjectInfo)> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT x, y, btl_tile_id, object_index FROM map_objects WHERE map_id = ? ORDER BY object_index, stack_order")
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let iter = stmt
            .query_map([map_id], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                ))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        for row in iter {
            let (x, y, tile_id, idx) = row.map_err(|e| std::io::Error::other(e.to_string()))?;
            let entry = objects_map.entry(idx).or_insert((
                idx,
                TiledObjectInfo {
                    ids: Vec::new(),
                    x,
                    y,
                },
            ));
            entry.1.ids.push(tile_id as i16);
        }
    }
    let mut objects: Vec<(i32, TiledObjectInfo)> = objects_map.into_values().collect();

    // ── Map metadata (original dimensions + offsets) ───────────────────────
    let metadata: (Option<i32>, Option<i32>, Option<i32>, Option<i32>) = conn
        .query_row(
            "SELECT tiled_width, tiled_height, non_occluded_x, non_occluded_y FROM map_metadata WHERE map_id = ?",
            [map_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap_or((None, None, None, None));

    let (width, height, non_occluded_x, non_occluded_y) = match metadata {
        (Some(w), Some(h), nox, noy) if w > 0 && h > 0 => {
            (w, h, nox.unwrap_or(0), noy.unwrap_or(0))
        }
        _ => {
            println!("WARNING: Map dimensions not found in map_metadata, falling back to bounds");
            (map_width, map_height, 0, 0)
        }
    };

    let diagonal = width + height;
    let offset_x_tiles = width / 2;
    let offset_y_tiles = height / 2;

    let image_width = (diagonal * TILE_HORIZONTAL_OFFSET_HALF) as u32;
    let image_height = (diagonal * TILE_HEIGHT_HALF) as u32;

    println!("Creating image: {}x{} pixels", image_width, image_height);
    let mut imgbuf: RgbaImage = image::ImageBuffer::new(image_width, image_height);

    // ── Pass 1: Ground tiles ───────────────────────────────────────────────
    println!("Rendering pass 1: Ground...");
    for ((real_x, real_y), gtl_id) in &gtl_tiles {
        let x = real_x + offset_x_tiles;
        let y = real_y + offset_y_tiles;
        let (dest_x, dest_y) = convert_map_coords_to_image_coords(x, y, diagonal);
        let atlas_x = (*gtl_id as u32 % atlas_columns) * tileset::TILE_WIDTH;
        let atlas_y = (*gtl_id as u32 / atlas_columns) * tileset::TILE_HEIGHT;
        plot_atlas_tile(AtlasTileParams {
            dest: &mut imgbuf,
            atlas: &gtl_atlas,
            src_x: atlas_x,
            src_y: atlas_y,
            tile_w: tileset::TILE_WIDTH,
            tile_h: tileset::TILE_HEIGHT,
            dest_x,
            dest_y,
        });
    }

    // ── Pass 2: Objects ────────────────────────────────────────────────────
    println!("Rendering pass 2: Objects...");
    objects.sort_by_key(|(idx, o)| {
        (
            o.y + (o.ids.len() as i32 * tileset::TILE_HEIGHT as i32),
            *idx,
        )
    });
    for (_, obj) in &objects {
        for (i, &btl_id) in obj.ids.iter().enumerate() {
            if btl_id <= 0 {
                continue;
            }
            let atlas_x = (btl_id as u32 % atlas_columns) * tileset::TILE_WIDTH;
            let atlas_y = (btl_id as u32 / atlas_columns) * tileset::TILE_HEIGHT;
            let x = obj.x + non_occluded_x;
            let y = obj.y + (i as i32 * tileset::TILE_HEIGHT as i32) + non_occluded_y;
            plot_atlas_tile(AtlasTileParams {
                dest: &mut imgbuf,
                atlas: &btl_atlas,
                src_x: atlas_x,
                src_y: atlas_y,
                tile_w: tileset::TILE_WIDTH,
                tile_h: tileset::TILE_HEIGHT,
                dest_x: x,
                dest_y: y,
            });
        }
    }

    // ── Pass 3: Roofs ──────────────────────────────────────────────────────
    println!("Rendering pass 3: Roofs...");
    for ((real_x, real_y), btl_id) in &btl_tiles {
        let x = real_x + offset_x_tiles;
        let y = real_y + offset_y_tiles;
        let (dest_x, dest_y) = convert_map_coords_to_image_coords(x, y, diagonal);
        let atlas_x = (*btl_id as u32 % atlas_columns) * tileset::TILE_WIDTH;
        let atlas_y = (*btl_id as u32 / atlas_columns) * tileset::TILE_HEIGHT;
        plot_atlas_tile(AtlasTileParams {
            dest: &mut imgbuf,
            atlas: &btl_atlas,
            src_x: atlas_x,
            src_y: atlas_y,
            tile_w: tileset::TILE_WIDTH,
            tile_h: tileset::TILE_HEIGHT,
            dest_x,
            dest_y,
        });
    }

    // ── Pass 3.5: Internal sprites from database ───────────────────────
    println!("Rendering pass 3.5: Internal sprites (from DB)...");
    {
        let mut stmt = conn
            .prepare(
                "SELECT sf.png_blob, sf.width, sf.height, sf.origin_x, sf.origin_y,
                        s.x AS placement_x, s.y AS placement_y
                 FROM map_sprite_frames sf
                 JOIN map_sprites s ON s.map_id = sf.map_id
                    AND s.internal_sprite_id = sf.internal_sprite_id
                 WHERE sf.map_id = ?1 AND sf.frame_index = 0",
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let iter = stmt
            .query_map([map_id], |row| {
                Ok((
                    row.get::<_, Vec<u8>>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i32>(6)?,
                ))
            })
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        for row in iter {
            let (png_blob, _width, _height, origin_x, origin_y, sx, sy) =
                row.map_err(|e| std::io::Error::other(e.to_string()))?;

            // Decode PNG from DB blob
            let img = image::load_from_memory(&png_blob)
                .map_err(|e| std::io::Error::other(e.to_string()))?
                .into_rgba8();

            // Convert placement tile coords to image pixel coords
            let x = sx + offset_x_tiles;
            let y = sy + offset_y_tiles;
            let (dest_x, dest_y) = convert_map_coords_to_image_coords(x, y, diagonal);

            // Render using existing sprite_loader function
            plot_entity_sprite(&mut imgbuf, &img, dest_x - origin_x, dest_y - origin_y, false);
        }
    }

    // ── Pass 4: External entities (NPCs, monsters, extras) ─────────────────
    println!("Rendering pass 4: External entities...");
    render_external_entities(
        &conn,
        &mut imgbuf,
        map_id,
        game_path,
        diagonal,
        offset_x_tiles,
        offset_y_tiles,
    );

    println!("Saving to {:?}...", output_path);
    imgbuf
        .save(output_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(())
}

// --------------------------------------------------------------------------
// External entity (NPC / monster / extra) rendering helpers
// --------------------------------------------------------------------------

struct ExternalEntity {
    x: i32,
    y: i32,
    color: image::Rgba<u8>,
    sprite_filename: Option<String>,
    sprite_dir: &'static str,
    sprite_sequence: usize,
    flip: bool,
}

fn render_external_entities(
    conn: &rusqlite::Connection,
    imgbuf: &mut image::RgbaImage,
    map_id: &str,
    game_path: Option<&Path>,
    diagonal: i32,
    _offset_x_tiles: i32,
    _offset_y_tiles: i32,
) {
    let refs_query = "
        SELECT i.monsters_filename, i.npc_filename, i.extra_filename
        FROM map_inis i
        JOIN maps m ON m.id = i.map_id
        WHERE m.map_filename = ? COLLATE NOCASE
        LIMIT 1
    ";

    let refs = conn
        .query_row(refs_query, [map_id], |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .ok();

    let mut entities: Vec<ExternalEntity> = Vec::new();

    if let Some((monsters_file, npc_file, extra_file)) = refs {
        collect_monsters(conn, monsters_file, &mut entities);
        collect_npcs(conn, npc_file, &mut entities);
        collect_extras(conn, extra_file, &mut entities);
    }

    println!("  Found {} external entities to render", entities.len());

    let mut sprite_cache: HashMap<String, Option<Vec<super::sprite_loader::LoadedSpriteFrame>>> =
        HashMap::new();

    for entity in &entities {
        // Entity tile coordinates are raw game tile coords (same system as the
        // .ref files store them). Unlike DB tile coords they are NOT shifted by
        // offset_x_tiles, so pass directly to convert_map_coords_to_image_coords.
        let (dest_x, dest_y) = convert_map_coords_to_image_coords(entity.x, entity.y, diagonal);
        let cx = dest_x + TILE_WIDTH_HALF;
        let cy = dest_y + TILE_HEIGHT_HALF;

        let mut rendered = false;

        if let Some(ref sf) = entity.sprite_filename {
            let key = format!("{}/{}", entity.sprite_dir, sf);

            if !sprite_cache.contains_key(&key) {
                let loaded = load_entity_sprite(
                    conn,
                    &key,
                    entity.sprite_dir,
                    sf,
                    game_path,
                );
                sprite_cache.insert(key.clone(), loaded);
            }

            if let Some(Some(frames)) = sprite_cache.get(&key) {
                if !frames.is_empty() {
                    let idx = entity.sprite_sequence.min(frames.len() - 1);
                    let frame = &frames[idx];
                    let sx = if entity.flip {
                        cx - (frame.image.width() as i32 - frame.origin_x)
                    } else {
                        cx - frame.origin_x
                    };
                    let sy = cy - frame.origin_y;
                    plot_entity_sprite(imgbuf, &frame.image, sx, sy, entity.flip);
                    rendered = true;
                }
            }
        }

        if !rendered {
            draw_entity_marker(imgbuf, cx, cy, entity.color);
        }
    }
}

/// Tries to load entity sprite frames. First attempts filesystem (if `game_path`
/// is provided), then falls back to the `sprite_file_blobs` database table.
fn load_entity_sprite(
    conn: &rusqlite::Connection,
    key: &str,
    sprite_dir: &str,
    sf: &str,
    game_path: Option<&Path>,
) -> Option<Vec<super::sprite_loader::LoadedSpriteFrame>> {
    // 1. Try filesystem if game_path is available
    if let Some(gp) = game_path {
        let try_paths = vec![
            gp.join(sprite_dir).join(sf),
            gp.join(sprite_dir).join(sf.to_ascii_uppercase()),
            gp.join(sprite_dir).join(sf.to_ascii_lowercase()),
        ];

        for p in &try_paths {
            if p.exists() {
                if let Some(frames) = load_sprite_frames(p) {
                    return Some(frames);
                }
            }
        }
    }

    // 2. Fallback: try database sprite_file_blobs table
    if let Ok(mut stmt) = conn
        .prepare("SELECT data FROM sprite_file_blobs WHERE normalized_path = ?1")
    {
        // Try original key, then lowercase, then uppercase
        let try_keys = vec![
            key.to_string(),
            key.to_ascii_lowercase(),
            key.to_ascii_uppercase(),
        ];
        for k in &try_keys {
            if let Ok(row) = stmt.query_row([k.as_str()], |row| row.get::<_, Vec<u8>>(0)) {
                if let Some(frames) = crate::map::sprite_loader::load_sprite_frames_from_bytes(&row)
                {
                    return Some(frames);
                }
            }
        }
    }

    None
}

fn collect_monsters(
    conn: &rusqlite::Connection,
    file: Option<String>,
    entities: &mut Vec<ExternalEntity>,
) {
    if let Some(mut f) = file {
        f = f.replace('\\', "/");
        if let Some(name) = f.split('/').next_back() {
            let q = format!(
                "SELECT mr.pos_x, mr.pos_y, mi.sprite_filename \
                 FROM monster_refs mr \
                 LEFT JOIN monster_inis mi ON mi.id = mr.mon_id \
                 WHERE mr.file_path LIKE '%{}'",
                name
            );
            if let Ok(mut stmt) = conn.prepare(&q) {
                if let Ok(iter) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, i32>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                }) {
                    for row in iter.filter_map(|r| r.ok()) {
                        entities.push(ExternalEntity {
                            x: row.0,
                            y: row.1,
                            color: image::Rgba([255, 60, 60, 255]),
                            sprite_filename: row.2,
                            sprite_dir: "MonsterInGame",
                            sprite_sequence: 3,
                            flip: false,
                        });
                    }
                }
            }
        }
    }
}

fn collect_npcs(
    conn: &rusqlite::Connection,
    file: Option<String>,
    entities: &mut Vec<ExternalEntity>,
) {
    if let Some(mut f) = file {
        f = f.replace('\\', "/");
        if let Some(name) = f.split('/').next_back() {
            let q = format!(
                "SELECT nr.show_on_event, nr.goto1_filled, nr.goto1_x, nr.goto1_y,
                        nr.goto2_filled, nr.goto2_x, nr.goto2_y,
                        nr.goto3_filled, nr.goto3_x, nr.goto3_y,
                        nr.goto4_filled, nr.goto4_x, nr.goto4_y,
                        ni.sprite_filename, nr.looking_direction \
                 FROM npc_refs nr \
                 LEFT JOIN npc_inis ni ON ni.id = nr.npc_id \
                 WHERE nr.file_path LIKE '%{}'",
                name
            );
            if let Ok(mut stmt) = conn.prepare(&q) {
                if let Ok(iter) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, i32>(0)?,             // show_on_event
                        row.get::<_, i32>(1)?,             // goto1_filled
                        row.get::<_, i32>(2)?,             // goto1_x
                        row.get::<_, i32>(3)?,             // goto1_y
                        row.get::<_, i32>(4)?,             // goto2_filled
                        row.get::<_, i32>(5)?,             // goto2_x
                        row.get::<_, i32>(6)?,             // goto2_y
                        row.get::<_, i32>(7)?,             // goto3_filled
                        row.get::<_, i32>(8)?,             // goto3_x
                        row.get::<_, i32>(9)?,             // goto3_y
                        row.get::<_, i32>(10)?,            // goto4_filled
                        row.get::<_, i32>(11)?,            // goto4_x
                        row.get::<_, i32>(12)?,            // goto4_y
                        row.get::<_, Option<String>>(13)?, // sprite_filename
                        row.get::<_, i32>(14)?,            // looking_direction
                    ))
                }) {
                    for row in iter.filter_map(|r| r.ok()) {
                        // Find first active waypoint
                        let waypoints = [
                            (row.1, row.2, row.3),    // goto1
                            (row.4, row.5, row.6),    // goto2
                            (row.7, row.8, row.9),    // goto3
                            (row.10, row.11, row.12), // goto4
                        ];

                        let (x, y) = waypoints
                            .iter()
                            .find(|(filled, _, _)| *filled != 0)
                            .map(|(_, x, y)| (*x, *y))
                            .unwrap_or((row.2, row.3)); // Fallback to goto1

                        let direction = row.14;
                        let seq = if direction > 4 {
                            (8 - direction) as usize
                        } else {
                            direction as usize
                        };
                        entities.push(ExternalEntity {
                            x,
                            y,
                            color: image::Rgba([60, 255, 60, 255]),
                            sprite_filename: row.13,
                            sprite_dir: "NpcInGame",
                            sprite_sequence: seq,
                            flip: direction > 4,
                        });
                    }
                }
            }
        }
    }
}

fn collect_extras(
    conn: &rusqlite::Connection,
    file: Option<String>,
    entities: &mut Vec<ExternalEntity>,
) {
    if let Some(mut f) = file {
        f = f.replace('\\', "/");
        if let Some(name) = f.split('/').next_back() {
            let q = format!(
                "SELECT er.x_pos, er.y_pos, e.sprite_filename, er.rotation, er.object_type, er.closed \
                 FROM extra_refs er \
                 LEFT JOIN extras e ON e.id = er.ext_id \
                 WHERE er.file_path LIKE '%{}'",
                name
            );
            if let Ok(mut stmt) = conn.prepare(&q) {
                if let Ok(iter) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, i32>(0)?,
                        row.get::<_, i32>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, i32>(3)?,
                        row.get::<_, i32>(4)?,
                        row.get::<_, i32>(5)?,
                    ))
                }) {
                    for row in iter.filter_map(|r| r.ok()) {
                        let (rotation, obj_type, closed) = (row.3, row.4, row.5);
                        let seq = if obj_type == 0 {
                            (2 * closed + rotation) as usize
                        } else {
                            rotation as usize
                        };
                        entities.push(ExternalEntity {
                            x: row.0,
                            y: row.1,
                            color: image::Rgba([80, 120, 255, 255]),
                            sprite_filename: row.2,
                            sprite_dir: "ExtraInGame",
                            sprite_sequence: seq,
                            flip: false,
                        });
                    }
                }
            }
        }
    }
}

/// Draws a diamond-shaped marker with a 1px dark outline, centred at (cx, cy).
fn draw_entity_marker(dest: &mut image::RgbaImage, cx: i32, cy: i32, fill: image::Rgba<u8>) {
    let r: i32 = 5;
    let outline = image::Rgba([0u8, 0, 0, 255]);
    let w = dest.width() as i32;
    let h = dest.height() as i32;

    let put = |dest: &mut image::RgbaImage, px: i32, py: i32, color: image::Rgba<u8>| {
        if px >= 0 && px < w && py >= 0 && py < h {
            dest.put_pixel(px as u32, py as u32, color);
        }
    };

    for d in 0..r {
        for &(px, py) in &[
            (cx + d, cy + (r - d)),
            (cx + d, cy - (r - d)),
            (cx - d, cy + (r - d)),
            (cx - d, cy - (r - d)),
            (cx + (r - d), cy + d),
            (cx + (r - d), cy - d),
            (cx - (r - d), cy + d),
            (cx - (r - d), cy - d),
        ] {
            put(dest, px, py, outline);
        }
    }
    for &(px, py) in &[(cx, cy + r), (cx, cy - r), (cx + r, cy), (cx - r, cy)] {
        put(dest, px, py, outline);
    }
    for dx in -(r - 1)..=r - 1 {
        for dy in -(r - 1)..=r - 1 {
            if dx.abs() + dy.abs() < r {
                put(dest, cx + dx, cy + dy, fill);
            }
        }
    }
}
