use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::path::Path;

use crate::sprite::{self, rgb16_565_produce_color};

// --------------------------------------------------------------------------
// Types
// --------------------------------------------------------------------------

/// A single decoded frame from a game sprite file (`.SPR` / `.SPX`),
/// with its anchor offsets so it can be placed correctly on the map.
pub struct LoadedSpriteFrame {
    pub image: image::RgbaImage,
    pub origin_x: i32,
    pub origin_y: i32,
}

// --------------------------------------------------------------------------
// Sprite loading
// --------------------------------------------------------------------------

/// Loads the first frame of every sequence from a sprite file.
///
/// Returns `None` if the file cannot be opened or contains no valid frames.
pub fn load_sprite_frames(sprite_path: &Path) -> Option<Vec<LoadedSpriteFrame>> {
    let file = File::open(sprite_path).ok()?;
    let file_len = file.metadata().ok()?.len();
    let mut reader = BufReader::new(file);
    let mut frames: Vec<LoadedSpriteFrame> = Vec::new();

    loop {
        let pos = reader.seek(SeekFrom::Current(0)).unwrap_or(file_len);
        match sprite::seek_next_sequence(&mut reader, pos, file_len) {
            Ok(true) => {}
            _ => break,
        }
        let info = match sprite::get_sequence_info(&mut reader) {
            Ok(i) => i,
            Err(_) => break,
        };

        let frame_data = if info.frame_count > 0 && !info.frame_infos.is_empty() {
            let f = &info.frame_infos[0];
            if f.width > 0 && f.height > 0 {
                reader
                    .seek(SeekFrom::Start(f.image_start_position))
                    .ok()
                    .and_then(|_| {
                        let mut img = image::RgbaImage::new(f.width as u32, f.height as u32);
                        for y in 0..f.height {
                            for x in 0..f.width {
                                let pix = reader.read_u16::<LittleEndian>().ok()?;
                                if pix > 0 {
                                    let c = rgb16_565_produce_color(pix);
                                    img.put_pixel(
                                        x as u32,
                                        y as u32,
                                        image::Rgba([c.r, c.g, c.b, 255]),
                                    );
                                }
                            }
                        }
                        Some(LoadedSpriteFrame {
                            image: img,
                            origin_x: f.origin_x,
                            origin_y: f.origin_y,
                        })
                    })
            } else {
                None
            }
        } else {
            None
        };

        frames.push(frame_data.unwrap_or_else(|| LoadedSpriteFrame {
            image: image::RgbaImage::new(1, 1),
            origin_x: 0,
            origin_y: 0,
        }));

        if reader
            .seek(SeekFrom::Start(info.sequence_end_position))
            .is_err()
        {
            break;
        }
    }

    if frames.is_empty() {
        None
    } else {
        Some(frames)
    }
}

// --------------------------------------------------------------------------
// Sprite plotting
// --------------------------------------------------------------------------

/// Plots a sprite frame onto a destination RGBA image, optionally flipped horizontally.
pub fn plot_entity_sprite(
    dest: &mut image::RgbaImage,
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
                dest.put_pixel(px as u32, py as u32, pixel);
            }
        }
    }
}
