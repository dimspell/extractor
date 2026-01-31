use crate::sprite::{rgb16_565_produce_color, save_sequence, Color, ImageInfo, SequenceInfo};
use crate::{ tileset};

use super::sprite;
use crate::tileset::{mix_color, plot_tile, Tile, TILE_HEIGHT};
use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageBuffer, Rgb};
use std::collections::HashMap;
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::{fs::File, path::Path};

pub fn extract(input_map_file: &Path,
               input_btl_file: &Path,
               input_gtl_file: &Path,
               output_path: &Path,
               save_map_sprites: &bool) -> Result<()> {
    let file = File::open(input_map_file)?;

    let metadata = file.metadata()?;
    println!("File len: {:?}", metadata.len());

    let mut reader = BufReader::new(file);

    let map_model = read_map_model(&mut reader)?;
    let tiled_map_width = map_model.tiled_map_width;
    let tiled_map_height = map_model.tiled_map_height;

    // first block
    first_block(&mut reader)?;

    // second block
    second_block(&mut reader)?;

    // sprites block
    let internal_sprites = sprite_block(&mut reader)?;

    // sprite info block
    let sprite_blocks = sprite_info_block(&mut reader, &internal_sprites)?;
    println!("{sprite_blocks:?} {}", sprite_blocks.len());

    // tiled objects block
    let tiled_infos = tiled_objects_block(&mut reader)?;

    // change read position
    println!("Before: {:?}", reader.seek(SeekFrom::Current(0))?);
    let skip = -(tiled_map_height * tiled_map_width * 4 * 3);
    let skip = skip.try_into().unwrap();
    reader.seek(SeekFrom::End(skip))?;
    println!("After: {:?}", reader.seek(SeekFrom::Current(0))?);

    // read event block
    let events = read_events_block(&mut reader, tiled_map_width, tiled_map_height)?;

    // read tiles and access block
    let (gtl_tiles, collisions) =
        read_tiles_and_access_block(&mut reader, tiled_map_width, tiled_map_height)?;

    let mut btl_tiles = HashMap::new();
    let pos = reader.seek(SeekFrom::Current(0))?;
    let skip = (tiled_map_height * tiled_map_width * 4 * 3)
        .try_into()
        .unwrap();
    if pos > skip {
        btl_tiles = read_roof_tiles(&mut reader, tiled_map_width, tiled_map_height)?;
    }

    println!("Finished at: {:?}", reader.seek(SeekFrom::Current(0))?);

    // Save sprites
    if *save_map_sprites {
        for i in 0..internal_sprites.len() {
            let frames = &internal_sprites[i].frame_infos;
            save_sequence(&mut reader, frames, i.try_into().unwrap(), &"cat1".to_string())?;
        }
    }

    let btl_tileset = tileset::extract(input_btl_file)?;
    let gtl_tileset = tileset::extract(input_gtl_file)?;

    // TODO: Read ini files

    // read_extra_ini(&Path::new("sample-data/Extra.ini"))?;
    // read_extra_ref(&Path::new("sample-data/ExtraInGame/Extdun01.ref"))?;

    // read_ini(&Path::new("sample-data/Monster.ini"))?;
    // read_monster_db(&Path::new("sample-data/MonsterInGame/Monster.db"))?;
    // read_monster_ref(&Path::new("sample-data/MonsterInGame/Mondun01.ref"))?;

    // read_ini(&Path::new("sample-data/Npc.ini"))?;
    // read_npc_ref(&Path::new("sample-data/NpcInGame/Npccat1.ref"))?;

    let map_id = input_map_file.file_stem().unwrap().to_str().unwrap();

    // TODO: Generate map
    generate_map(
        &mut reader,
        output_path,
        &map_model,
        true,
        &gtl_tiles,
        &gtl_tileset,
        &btl_tiles,
        &btl_tileset,
        &tiled_infos,
        &internal_sprites,
        &sprite_blocks,
        &collisions,
        &events,
        map_id,
    )?;

    Ok(())
}

fn generate_map(
    reader: &mut BufReader<File>,
    output_path: &Path,
    model: &MapModel,
    occlusion: bool,
    gtl_tiles: &HashMap<Coords, i32>,
    gtl_tileset: &Vec<Tile>,
    btl_tiles: &HashMap<Coords, i32>,
    btl_tileset: &Vec<Tile>,
    tiled_infos: &Vec<TiledObjectInfo>,
    internal_sprites: &Vec<SequenceInfo>,
    sprite_blocks: &Vec<SpriteInfoBlock>,
    collisions: &HashMap<Coords, bool>,
    events: &HashMap<Coords, EventBlock>,
    map_id: &str,
) -> Result<()> {
    // Save to database
    println!("Saving map_tiles...");
    let conn_res = rusqlite::Connection::open("database.sqlite");
    if let Ok(conn) = conn_res {
        let _ = crate::database::save_map_tiles(
            &conn,
            map_id,
            gtl_tiles,
            btl_tiles,
            collisions,
            events,
            model.tiled_map_width,
            model.tiled_map_height,
        );
    }
    let image_width = if occlusion {
        model.occluded_map_in_pixels_width
    } else {
        model.map_width_in_pixels
    };
    let image_height = if occlusion {
        model.occluded_map_in_pixels_height
    } else {
        model.map_height_in_pixels
    };

    println!("{:?}", model);

    println!(
        "{}, {}",
        image_width.unsigned_abs(),
        image_height.unsigned_abs(),
    );

    let offset_x = if !occlusion {
        model.map_non_occluded_start_x
    } else {
        0
    };
    let offset_y = if !occlusion {
        model.map_non_occluded_start_y
    } else {
        0
    };

    let mut imgbuf: ImageBuffer<Rgb<u8>, Vec<u8>> =
        image::ImageBuffer::new(image_width.unsigned_abs(), image_height.unsigned_abs());

    plot_base(
        &mut imgbuf,
        model,
        occlusion,
        gtl_tiles,
        gtl_tileset,
        collisions,
        events,
    );

    plot_objects(
        &mut imgbuf,
        reader,
        model,
        occlusion,
        btl_tiles,
        btl_tileset,
        tiled_infos,
        internal_sprites,
        sprite_blocks,
        offset_x,
        offset_y,
    )?;

    plot_roofs(&mut imgbuf, model, occlusion, btl_tiles, btl_tileset);

    imgbuf.save(output_path).unwrap();

    Ok(())
}

pub type Coords = (i32, i32);

pub const TILE_WIDTH_HALF: i32 = 62 / 2;
pub const TILE_HEIGHT_HALF: i32 = 32 / 2;
pub const TILE_HORIZONTAL_OFFSET_HALF: i32 = 32;
pub const TILE_PIXEL_NUMBER: i32 = 32 * 32;

fn convert_map_coords_to_image_coords(x: i32, y: i32, map_diagonal_tiles: i32) -> (i32, i32) {
    let start_x = (x + y) * TILE_HORIZONTAL_OFFSET_HALF;
    let start_y = (-x + y) * TILE_HEIGHT_HALF + (map_diagonal_tiles / 2 * TILE_HEIGHT_HALF);
    (start_x, start_y)
}

fn plot_base(
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
            let gtl_tile_id = gtl_tiles.get(&coords);

            match gtl_tile_id {
                Some(gtl_tile_id) => {
                    let gtl_tile_id_usize: usize = gtl_tile_id.unsigned_abs().try_into().unwrap();
                    let gtl_tile = &gtl_tileset[gtl_tile_id_usize];

                    let event_block = events.get(&coords);
                    let collision = collisions.get(&coords);

                    let (mut start_x, mut start_y) =
                        convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);

                    if occlusion {
                        start_x -= model.map_non_occluded_start_x;
                        start_y -= model.map_non_occluded_start_y;
                    }

                    let event_id = match event_block {
                        Some(event) => event.event_id,
                        None => 0,
                    };
                    let is_collision = match collision {
                        Some(col) => *col,
                        None => false,
                    };
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
                        // mix_color(gtl_tile.colors, Color { r: 10, b: 255, g: 200 }, 50)
                        gtl_tile.colors
                    } else {
                        gtl_tile.colors
                    };

                    plot_tile(image, tile_colors, start_x, start_y);
                    // textGenerator.PlotIdOnMap(image, eventId, mapCoords.X + TileSet.TILE_WIDTH_HALF, mapCoords.Y + TextGenerator.DigitHeight);
                }
                _ => {} // noop
            }
        }
    }
}

fn plot_objects(
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
        let sequence = &internal_sprites[block.sprite_id as usize];
        let sprite = &sequence.frame_infos[0];
        let ground_y = block.sprite_y + sprite.height as i32;
        items.push(Item {
            ground_y,
            kind: Kind::Sprite(i),
        });
    }

    for i in 0..tiled_info.len() {
        let info = &tiled_info[i];
        let ground_y = info.y + (info.ids.len() as i32 * TILE_HEIGHT as i32);
        items.push(Item {
            ground_y,
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
    for i in 0..tiled_info.ids.len() {
        let btl_id = &tiled_info.ids[i];
        let tile = &btl_tileset[btl_id.abs() as usize];
        let x = tiled_info.x + offset_x;
        let y = tiled_info.y + (i as i32 * TILE_HEIGHT as i32) + offset_y;
        plot_tile(imgbuf, tile.colors, x, y);
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
    let sequence: &SequenceInfo = &internal_sprites[sprite_block.sprite_id as usize];
    let sprite = &sequence.frame_infos[0];
    let dest_x = sprite_block.sprite_x + offset_x;
    let dest_y = sprite_block.sprite_y + offset_y;
    plot_sprite_on_bitmap(imgbuf, reader, &sprite, dest_x, dest_y)?;
    Ok(())
}

fn plot_sprite_on_bitmap(
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
                    let final_x = (dest_x + x) as u32;
                    let final_y = (dest_y + y) as u32;
                    imgbuf.put_pixel(final_x, final_y, Rgb([color.r, color.g, color.b]));
                }
            }
        }
    }

    Ok(())
}

#[test]
fn rgb16_565_produce_color_test() {
    let color = rgb16_565_produce_color(0);
    assert_eq!(color.r as i16 + color.g as i16 + color.b as i16, 0);
}


fn plot_roofs(
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
            let btl_tile_id = match btl_tiles.get(&coords) {
                None => 0,
                Some(id) => *id,
            };

            if btl_tile_id > 0 {
                let btl_tile = &btl_tileset[btl_tile_id as usize];

                let (mut start_x, mut start_y) =
                    convert_map_coords_to_image_coords(x, y, map_diagonal_tiles);

                if occlusion {
                    start_x -= model.map_non_occluded_start_x;
                    start_y -= model.map_non_occluded_start_y;
                }

                // println!("{start_x} {start_y}, {btl_tile_id}");
                plot_tile(image, btl_tile.colors, start_x, start_y);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct MapModel {
    tiled_map_width: i32,
    tiled_map_height: i32,
    map_width_in_pixels: i32,
    map_height_in_pixels: i32,
    map_non_occluded_start_x: i32,
    map_non_occluded_start_y: i32,
    occluded_map_in_pixels_width: i32,
    occluded_map_in_pixels_height: i32,
}

fn read_map_model(reader: &mut BufReader<File>) -> Result<MapModel> {
    // map size
    let width = reader.read_i32::<LittleEndian>()?;
    let height = reader.read_i32::<LittleEndian>()?;
    let diagonal = width + height;

    // tiled map size
    const MAP_CHUNK_SIZE: i32 = 25;
    let tiled_map_width = width * MAP_CHUNK_SIZE - 1;
    let tiled_map_height = height * MAP_CHUNK_SIZE - 1;

    let map_width_in_pixels = diagonal * MAP_CHUNK_SIZE * TILE_HORIZONTAL_OFFSET_HALF;
    let map_height_in_pixels = diagonal * MAP_CHUNK_SIZE * TILE_HEIGHT_HALF;

    let x_aspect: f64 = 0.3;
    let y_aspect: f64 = 0.2;

    let compensate_x: f64 = TILE_HORIZONTAL_OFFSET_HALF.try_into().unwrap();
    let compensate_y: f64 = 0.0;

    let map_non_occluded_start_x: f64 = map_width_in_pixels.into();
    let map_non_occluded_start_x: f64 = x_aspect * map_non_occluded_start_x - compensate_x;
    let map_non_occluded_start_x: f64 = map_non_occluded_start_x.round();
    let map_non_occluded_start_x: i32 = map_non_occluded_start_x as i32; // todo fixme

    let map_non_occluded_start_y: f64 = map_height_in_pixels.into();
    let map_non_occluded_start_y: f64 = y_aspect * map_non_occluded_start_y - compensate_y;
    let map_non_occluded_start_y: f64 = map_non_occluded_start_y.round();
    let map_non_occluded_start_y: i32 = map_non_occluded_start_y as i32; // todo fixme

    let occluded_map_in_pixels_width = map_width_in_pixels - (map_non_occluded_start_x * 2);
    let occluded_map_in_pixels_height = map_height_in_pixels - (map_non_occluded_start_y * 2);

    let model = MapModel {
        tiled_map_width,
        tiled_map_height,
        map_width_in_pixels,
        map_height_in_pixels,
        map_non_occluded_start_x,
        map_non_occluded_start_y,
        occluded_map_in_pixels_width,
        occluded_map_in_pixels_height,
    };

    Ok(model)
}

fn first_block(reader: &mut BufReader<File>) -> Result<()> {
    let multiplier = reader.read_i32::<LittleEndian>()?;
    let size = reader.read_i32::<LittleEndian>()?;
    reader.seek(SeekFrom::Start(8))?;
    let skip = multiplier * size * 4;
    let skip: i64 = skip.try_into().unwrap();
    reader.seek(SeekFrom::Current(skip))?;

    Ok(())
}

fn second_block(reader: &mut BufReader<File>) -> Result<()> {
    let size = reader.read_i32::<LittleEndian>()?;
    let skip = size * 2;
    let skip: i64 = skip.try_into().unwrap();
    reader.seek(SeekFrom::Current(skip))?;

    Ok(())
}

fn sprite_block(reader: &mut BufReader<File>) -> Result<Vec<SequenceInfo>> {
    let sprite_count = reader.read_i32::<LittleEndian>()?;
    let mut sprites = vec![];
    for _ in 0..sprite_count {
        let image_stamp = reader.read_i32::<LittleEndian>()?;
        let image_offset: i32 = if image_stamp == 6 {
            1904
        } else if image_stamp == 9 {
            2996
        } else {
            unimplemented!("Unexpected image-stamp {image_stamp}");
        };

        reader.seek(SeekFrom::Current(264))?;

        let info = sprite::get_sequence_info(reader)?;
        let info_offset = info.sequence_end_position;
        sprites.push(info);
        reader.seek(SeekFrom::Start(info_offset))?;

        let image_offset: i64 = image_offset.try_into().unwrap();
        reader.seek(SeekFrom::Current(image_offset))?;
    }

    Ok(sprites)
}

#[derive(Copy, Clone, Debug)]
struct SpriteInfoBlock {
    sprite_id: usize,
    sprite_x: i32,
    sprite_y: i32,
    // sprite_bottom_right_x: i32,
    // sprite_bottom_right_y: i32,
}

fn sprite_info_block(
    reader: &mut BufReader<File>,
    sprites: &Vec<SequenceInfo>,
) -> Result<Vec<SpriteInfoBlock>> {
    let count = reader.read_i32::<LittleEndian>()?;

    let info_size: usize = count.try_into().unwrap();
    let mut info = Vec::with_capacity(info_size);

    for _i in 0..count {
        let sprite_id = reader.read_i32::<LittleEndian>()?;
        reader.read_i32::<LittleEndian>()?; // what is it?
        reader.read_i32::<LittleEndian>()?; // what is it?
        let _sprite_bottom_right_x = reader.read_i32::<LittleEndian>()?;
        let _sprite_bottom_right_y = reader.read_i32::<LittleEndian>()?;
        let sprite_x = reader.read_i32::<LittleEndian>()?;
        let sprite_y = reader.read_i32::<LittleEndian>()?;

        let sprite_id: usize = sprite_id.try_into().unwrap();
        let skip = sprites[sprite_id].frame_count;
        let skip = skip - 1;
        let skip = skip * 6 * 4;
        let skip = skip.try_into().unwrap();
        reader.seek(SeekFrom::Current(skip))?;

        info.push(SpriteInfoBlock {
            sprite_id,
            sprite_x,
            sprite_y,
            // sprite_bottom_right_x,
            // sprite_bottom_right_y,
        });
    }

    Ok(info)
}

struct TiledObjectInfo {
    ids: Vec<i16>,
    x: i32,
    y: i32,
}

fn tiled_objects_block(reader: &mut BufReader<File>) -> Result<Vec<TiledObjectInfo>> {
    let bundles_count = reader.read_i32::<LittleEndian>()?;
    let _number1 = reader.read_i32::<LittleEndian>()?;

    let mut infos: Vec<TiledObjectInfo> = Vec::with_capacity(bundles_count.unsigned_abs() as usize);
    for _i in 0..bundles_count {
        reader.seek(SeekFrom::Current(264))?;

        let _s8 = reader.read_i32::<LittleEndian>()?;
        let _s0_1 = reader.read_i32::<LittleEndian>()?;
        let _s1 = reader.read_i32::<LittleEndian>()?;
        let _s0_2 = reader.read_i32::<LittleEndian>()?;

        let _v1 = reader.read_i32::<LittleEndian>()?;
        let _v2 = reader.read_i32::<LittleEndian>()?;
        let _v3 = reader.read_i32::<LittleEndian>()?;
        let _v4 = reader.read_i32::<LittleEndian>()?;
        let x = reader.read_i32::<LittleEndian>()?;
        let y = reader.read_i32::<LittleEndian>()?;
        let _v7 = reader.read_i32::<LittleEndian>()?;
        let _v8 = reader.read_i32::<LittleEndian>()?;

        let c1 = reader.read_i32::<LittleEndian>()?;
        let c2 = reader.read_i32::<LittleEndian>()?;
        let c3 = reader.read_i32::<LittleEndian>()?;

        let mut ids: Vec<i16> = vec![];
        for _i in 0..c3 {
            ids.push(reader.read_i16::<LittleEndian>()?);
        }

        infos.push(TiledObjectInfo { ids, x, y });

        reader.seek(SeekFrom::Current(84))?;

        let skip = (c1 + c2 + c3) * 4;
        let skip = skip.try_into().unwrap();
        reader.seek(SeekFrom::Current(skip))?;
    }

    let back_pos = 20;
    reader.seek(SeekFrom::Current(-back_pos))?;
    let mut last_pos = 0;
    for _i in 0..back_pos {
        let v: u8 = reader.read_u8()?;
        if v == 1 {
            last_pos = v;
        }
    }

    let to_undo: i64 = back_pos.try_into().unwrap();
    reader.seek(SeekFrom::Current(to_undo))?;

    let to_undo: i64 = last_pos.try_into().unwrap();
    reader.seek(SeekFrom::Current(-to_undo - 4))?;

    Ok(infos)
}

#[derive(Copy, Clone, Debug)]
pub struct EventBlock {
    pub x: i32,
    pub y: i32,
    unknown: i16,
    pub event_id: i16,
}

fn read_events_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<HashMap<Coords, EventBlock>> {
    let mut blocks = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let event_id = reader.read_i16::<LittleEndian>()?;
            let what_is_it = reader.read_i16::<LittleEndian>()?;
            if what_is_it != 0 {
                // Not implemented in Dispel Tools
                // println!("{event_id}: {what_is_it}: (x: {x} y: {y})");
            }

            let coords = (x, y);
            blocks.insert(
                coords,
                EventBlock {
                    x,
                    y,
                    event_id,
                    unknown: 0,
                },
            );
        }
    }

    Ok(blocks)
}

fn read_tiles_and_access_block(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<(HashMap<Coords, i32>, HashMap<Coords, bool>)> {
    let mut gtl_tiles = HashMap::new();
    let mut collisions = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let coords: Coords = (x, y);

            let value = reader.read_i32::<LittleEndian>()?;
            let gtl_tile_id = value >> 10;
            let collision = (gtl_tile_id & 0x1) == 1;

            gtl_tiles.insert(coords, gtl_tile_id);
            collisions.insert(coords, collision);
        }
    }

    Ok((gtl_tiles, collisions))
}

fn read_roof_tiles(
    reader: &mut BufReader<File>,
    tiled_map_width: i32,
    tiled_map_height: i32,
) -> Result<HashMap<Coords, i32>> {
    let mut btl_tiles = HashMap::new();

    for y in 0..tiled_map_height {
        for x in 0..tiled_map_width {
            let btl_tile_id = reader.read_i16::<LittleEndian>()?;
            let some_flag = reader.read_i16::<LittleEndian>()?;
            let coords: Coords = (x, y);

            if btl_tile_id > 0 {
                if some_flag > 0 {
                    println!("ReadRoofTiles TODO: {btl_tile_id:?} {some_flag}");
                }
                btl_tiles.insert(coords, btl_tile_id.into());
            }
        }
    }

    Ok(btl_tiles)
}
