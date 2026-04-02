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

/// Renders the full map to a PNG file, compositing ground → objects → roofs.
pub fn render_map(
    reader: &mut BufReader<File>,
    output_path: &std::path::Path,
    data: &super::MapData,
    occlusion: bool,
    gtl_tileset: &[Tile],
    btl_tileset: &[Tile],
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
