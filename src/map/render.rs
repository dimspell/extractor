use image::{ImageBuffer, Rgb};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Result, Seek, SeekFrom};

use crate::sprite::{rgb16_565_produce_color, Color, ImageInfo, SequenceInfo};
use crate::map::tileset::{mix_color, plot_tile, Tile, TILE_HEIGHT};
use byteorder::{LittleEndian, ReadBytesExt};

use super::types::{
    convert_map_coords_to_image_coords, Coords, EventBlock, SpriteInfoBlock, TiledObjectInfo,
};

use super::model::MapModel;

// --------------------------------------------------------------------------
// Top-level render entry point
// --------------------------------------------------------------------------

/// Renders the full map to a PNG file, compositing ground → objects → roofs.
pub fn render_map(
    reader: &mut BufReader<File>,
    output_path: &std::path::Path,
    data: &super::MapData,
    occlusion: bool,
    gtl_tileset: &Vec<Tile>,
    btl_tileset: &Vec<Tile>,
    _map_id: &str,
) -> Result<()> {
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
        btl_tileset,
        &data.tiled_infos,
        &data.internal_sprites,
        &data.sprite_blocks,
        offset_x,
        offset_y,
    )?;
    plot_roofs(
        &mut imgbuf,
        &data.model,
        occlusion,
        &data.btl_tiles,
        btl_tileset,
    );

    imgbuf.save(output_path).unwrap();
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
    gtl_tileset: &Vec<Tile>,
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
                let is_collision = collision.copied().unwrap_or(false);

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
                } else if is_collision {
                    gtl_tile.colors
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

pub fn plot_objects(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    reader: &mut BufReader<File>,
    _model: &MapModel,
    _occlusion: bool,
    _btl_tiles: &HashMap<Coords, i32>,
    btl_tileset: &Vec<Tile>,
    tiled_info: &Vec<TiledObjectInfo>,
    internal_sprites: &Vec<SequenceInfo>,
    sprite_blocks: &Vec<SpriteInfoBlock>,
    offset_x: i32,
    offset_y: i32,
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

    for i in 0..sprite_blocks.len() {
        let block = &sprite_blocks[i];
        let sequence = &internal_sprites[block.sprite_id];
        let sprite = &sequence.frame_infos[0];
        items.push(Item {
            ground_y: block.sprite_y + sprite.height,
            kind: Kind::Sprite(i),
        });
    }
    for i in 0..tiled_info.len() {
        let info = &tiled_info[i];
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
                &sprite_blocks[i],
                internal_sprites,
                offset_x,
                offset_y,
            )?,
            Kind::TiledObject(i) => {
                plot_single_tiled_object(imgbuf, &tiled_info[i], btl_tileset, offset_x, offset_y)
            }
        }
    }
    Ok(())
}

fn plot_single_tiled_object(
    imgbuf: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    tiled_info: &TiledObjectInfo,
    btl_tileset: &Vec<Tile>,
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
    internal_sprites: &Vec<SequenceInfo>,
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
    btl_tileset: &Vec<Tile>,
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
                    let (mut sx, mut sy) = convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);
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
// Atlas tile blitter (used by render_from_database)
// --------------------------------------------------------------------------

/// Copies a tile from a pre-built atlas image onto the destination buffer,
/// with per-pixel alpha blending support.
pub fn plot_atlas_tile(
    dest: &mut image::RgbaImage,
    atlas: &image::DynamicImage,
    src_x: u32,
    src_y: u32,
    tile_w: u32,
    tile_h: u32,
    dest_x: i32,
    dest_y: i32,
) {
    use image::GenericImageView;

    if dest_x < 0 || dest_y < 0 {
        return;
    }
    let dest_x = dest_x as u32;
    let dest_y = dest_y as u32;

    if dest_x + tile_w > dest.width()
        || dest_y + tile_h > dest.height()
        || src_x + tile_w > atlas.width()
        || src_y + tile_h > atlas.height()
    {
        return;
    }

    for py in 0..tile_h {
        for px in 0..tile_w {
            let pixel = atlas.get_pixel(src_x + px, src_y + py);
            let alpha = pixel[3];
            if alpha == 0 {
                continue;
            }
            if alpha == 255 {
                dest.put_pixel(dest_x + px, dest_y + py, pixel);
            } else {
                let existing = *dest.get_pixel(dest_x + px, dest_y + py);
                let blend = |src: u8, dst: u8, a: u8| -> u8 {
                    ((src as u32 * a as u32 + dst as u32 * (255 - a as u32)) / 255) as u8
                };
                dest.put_pixel(
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
