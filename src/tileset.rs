use byteorder::{LittleEndian, ReadBytesExt};
use image::{Rgb, RgbaImage, RgbImage};
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::{fs::File, path::Path};

use crate::sprite::{rgb16_565_produce_color, Color};

pub const TILE_PIXEL_NUMBER: i32 = 32 * 32;
pub const TILE_WIDTH: u32 = 62;
pub const TILE_HEIGHT: u32 = 32;

pub fn extract(source_path: &Path) -> Result<Vec<Tile>> {
    let file = File::open(source_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    let tile_number = file_len / (32 * 32 * 2);
    let mut tiles = Vec::<Tile>::with_capacity(tile_number as usize);

    for _ in 0..tile_number {
        let pos = reader.seek(SeekFrom::Current(0))?;
        if pos > file_len {
            break;
        }

        let mut pixels: [Color; 1024] = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_NUMBER as usize];
        for p in 0..TILE_PIXEL_NUMBER {
            let pixel = reader.read_u16::<LittleEndian>()?;
            let color = rgb16_565_produce_color(pixel);
            pixels[p as usize] = color;
        }

        let tile = Tile { colors: pixels };
        tiles.push(tile);
    }

    Ok(tiles)
}

fn create_mask() -> [[i32; 32]; 2] {
    let mut mask = [[0i32; TILE_HEIGHT as usize]; 2];
    let mut pixels_x: i32 = 1;
    let step: i32 = 2;
    let mut direction: i32 = 1;
    let limit = 31;

    for y in 0..TILE_HEIGHT as usize {
        mask[0][y] = (TILE_WIDTH as i32) / 2 - pixels_x;
        mask[1][y] = pixels_x * 2;
        pixels_x += step * direction;
        if pixels_x > limit {
            direction = -1;
            pixels_x = limit;
        }
    }

    mask
}

pub fn plot_all_tiles(tiles: &Vec<Tile>) {
    for tile_index in 0..TILE_PIXEL_NUMBER {
        let tile = &tiles[tile_index as usize];

        let mut imgbuf = image::ImageBuffer::new(TILE_WIDTH, TILE_HEIGHT);
        let dest_x = 0;
        let dest_y: i32 = 0;
        plot_tile(&mut imgbuf, tile.colors, dest_x, dest_y);
        imgbuf.save(format!("image_{:?}.png", tile_index)).unwrap();
    }
}

pub fn plot_tile(imgbuf: &mut RgbImage, colors: [Color; 1024], dest_x: i32, dest_y: i32) {
    if dest_x + TILE_WIDTH as i32 <= imgbuf.width() as i32
        && dest_x >= 0
        && dest_y >= 0
        && dest_y + TILE_HEIGHT as i32 <= imgbuf.height() as i32
    {
        // Todo: calculate it only once
        let mask = create_mask();

        let mut i = 0;
        for y in 0..TILE_HEIGHT as usize {
            for x in 0..mask[1][y] {
                let pixel: Color = colors[i];
                i += 1;

                let final_x = dest_x + x + mask[0][y];
                let final_y = dest_y + y as i32;

                if pixel.r != 0 && pixel.g != 0 && pixel.b != 0 {
                    imgbuf.put_pixel(
                        final_x.try_into().unwrap(),
                        final_y.try_into().unwrap(),
                        Rgb([pixel.r, pixel.g, pixel.b]),
                    );
                }
            }
        }
    }
}

pub fn plot_tileset_map(tiles: &Vec<Tile>, out_path: &str) {
    let w = 10;

    let h = (tiles.len() as f64 / 10.0).ceil();
    let h: i32 = h as i32;

    println!("{}, {}", tiles.len(), h * w);
    let width: u32 = TILE_WIDTH * w as u32;
    let height: u32 = TILE_HEIGHT * h as u32;
    let mut bitmap = image::ImageBuffer::new(width, height);

    let mut tile_index = 0;
    for y in 0..h as u32 {
        for x in 0..w as u32 {
            if tiles.len() == tile_index {
                break;
            }

            let tile = &tiles[tile_index];
            tile_index += 1;

            let offset_x = 0;
            let offset_y = 0;
            // if y % 2 != 0 {
            //     offset_x = TILE_WIDTH / 2 + 1;
            //     offset_y = TILE_HEIGHT / 2;
            // }

            let dest_x: u32 = x * TILE_WIDTH + offset_x;
            let dest_x: i32 = dest_x.try_into().unwrap();
            let dest_y: u32 = y * TILE_HEIGHT + offset_y;
            let dest_y: i32 = dest_y.try_into().unwrap();

            plot_tile_rgba(&mut bitmap, tile.colors, dest_x, dest_y)
        }
    }

    bitmap.save(out_path).unwrap();
}

pub fn plot_tile_rgba(imgbuf: &mut RgbaImage, colors: [Color; 1024], dest_x: i32, dest_y: i32) {
    if dest_x + TILE_WIDTH as i32 <= imgbuf.width() as i32
        && dest_x >= 0
        && dest_y >= 0
        && dest_y + TILE_HEIGHT as i32 <= imgbuf.height() as i32
    {
        // Todo: calculate it only once
        let mask = create_mask();

        let mut i = 0;
        for y in 0..TILE_HEIGHT as usize {
            for x in 0..mask[1][y] {
                let pixel: Color = colors[i];
                i += 1;

                let final_x = dest_x + x + mask[0][y];
                let final_y = dest_y + y as i32;

                if pixel.r != 0 && pixel.g != 0 && pixel.b != 0 {
                    imgbuf.put_pixel(
                        final_x.try_into().unwrap(),
                        final_y.try_into().unwrap(),
                        image::Rgba([pixel.r, pixel.g, pixel.b, 255]),
                    );
                }
            }
        }
    }
}


pub struct Tile {
    pub colors: [Color; 1024],
}

pub fn mix_color(canvas: [Color; 1024], color: Color, alpha: u8) -> [Color; 1024] {
    const TILE_PIXEL_NUMBER: i32 = 32 * 32; // 1024
    let mut pixels = [Color { r: 0, g: 0, b: 0 }; TILE_PIXEL_NUMBER as usize];
    let amount: f64 = alpha as f64 / 255.0;

    for i in 0..TILE_PIXEL_NUMBER as usize {
        let base = canvas[i];
        let r = ((color.r as f64 * amount) + base.r as f64 * (1.0 - amount)) as u8;
        let g: u8 = ((color.g as f64 * amount) + base.g as f64 * (1.0 - amount)) as u8;
        let b: u8 = ((color.r as f64 * amount) + base.b as f64 * (1.0 - amount)) as u8;
        pixels[i] = Color { r, g, b };
    }
    pixels
}
