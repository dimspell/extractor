use std::path::Path;

use crate::sprite;

// --------------------------------------------------------------------------
// Types
// --------------------------------------------------------------------------

/// A single decoded frame from a game sprite file (`.SPR`),
/// with its anchor offsets so it can be placed correctly on the map.
pub struct LoadedSpriteFrame {
    pub image: image::RgbaImage,
    pub origin_x: i32,
    pub origin_y: i32,
}

// --------------------------------------------------------------------------
// Sprite loading
// --------------------------------------------------------------------------

/// Decodes frame 0 of a single parsed sprite sequence into a `LoadedSpriteFrame`.
fn decode_first_frame(seq: &sprite::SpriteSequence) -> LoadedSpriteFrame {
    match seq.frames.first() {
        Some(f) if f.width > 0 && f.height > 0 => {
            let rgba = f.decode_to_rgba();
            let img = match image::RgbaImage::from_raw(f.width as u32, f.height as u32, rgba) {
                Some(img) => img,
                None => image::RgbaImage::new(1, 1),
            };
            LoadedSpriteFrame {
                image: img,
                origin_x: f.origin_x,
                origin_y: f.origin_y,
            }
        }
        _ => LoadedSpriteFrame {
            image: image::RgbaImage::new(1, 1),
            origin_x: 0,
            origin_y: 0,
        },
    }
}

/// Loads the first frame of every sequence from a sprite file.
///
/// Returns `None` if the file cannot be opened or contains no valid frames.
pub fn load_sprite_frames(sprite_path: &Path) -> Option<Vec<LoadedSpriteFrame>> {
    let sf = sprite::read_sprite_file(sprite_path).ok()?;
    if sf.sequences.is_empty() {
        return None;
    }
    Some(sf.sequences.iter().map(decode_first_frame).collect())
}

/// Loads the first frame of every sequence from an in-memory sprite buffer.
///
/// Same as `load_sprite_frames` but reads from a byte slice instead of a file.
/// Useful when the sprite data comes from a database blob rather than the filesystem.
pub fn load_sprite_frames_from_bytes(data: &[u8]) -> Option<Vec<LoadedSpriteFrame>> {
    let sf = sprite::parse_sprite_bytes(data).ok()?;
    if sf.sequences.is_empty() {
        return None;
    }
    Some(sf.sequences.iter().map(decode_first_frame).collect())
}

/// Decodes the last frame of a single parsed sprite sequence into a
/// `LoadedSpriteFrame`. Used for dead monsters, where the final frame of the
/// death sequence is the "corpse" pose.
fn decode_last_frame(seq: &sprite::SpriteSequence) -> LoadedSpriteFrame {
    match seq.frames.last() {
        Some(f) if f.width > 0 && f.height > 0 => {
            let rgba = f.decode_to_rgba();
            let img = match image::RgbaImage::from_raw(f.width as u32, f.height as u32, rgba) {
                Some(img) => img,
                None => image::RgbaImage::new(1, 1),
            };
            LoadedSpriteFrame {
                image: img,
                origin_x: f.origin_x,
                origin_y: f.origin_y,
            }
        }
        _ => LoadedSpriteFrame {
            image: image::RgbaImage::new(1, 1),
            origin_x: 0,
            origin_y: 0,
        },
    }
}

/// Loads the last frame of a specific sequence from a sprite file.
///
/// Returns `None` if the file cannot be opened, has no sequences, or `seq_idx`
/// is out of range. The returned frame is the final frame of that sequence
/// (e.g. the dead pose for a monster's death animation).
pub fn load_last_frame_of_sequence(
    sprite_path: &Path,
    seq_idx: usize,
) -> Option<LoadedSpriteFrame> {
    let sf = sprite::read_sprite_file(sprite_path).ok()?;
    let seq = sf.sequences.get(seq_idx)?;
    Some(decode_last_frame(seq))
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
