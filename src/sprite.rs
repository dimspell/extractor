use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageEncoder, RgbaImage};
use std::io::{BufReader, Cursor, Result, Seek, SeekFrom};
use std::{fs::File, path::Path};

// ===========================================================================
// DISPEL SPRITE FILE FORMAT (.SPR)
// ===========================================================================
//
// Sprite files store character sprites, animations, and visual effects used
// for rendering NPCs, monsters, party members, and special effects in the
// isometric game world. Each file contains one or more animation sequences,
// where each sequence contains one or more frames of pixel data.
//
// Full documentation: docs/files/Map/Sprites.spr.md
//
// Quick reference:
//   - 268-byte unknown header, then variable-length sequences
//   - Sequences found by scanning for valid header patterns (15×i32)
//   - Each sequence: header → frame metadata blocks → RGB565 pixel data
//   - RGB565: 5R/6G/5B, 0x0000=transparent, little-endian
//   - Frames have origin_x/origin_y for alignment in a bounding rect
//
// Reading flow:
//   1. seek(268)
//   2. seek_next_sequence() → find header or EOF
//   3. get_sequence_info() → parse header + frame metadata
//   4. seek(sequence_start_position) → render frames
//   5. seek(sequence_end_position) → continue to next sequence
//
// ===========================================================================

// ===========================================================================
// Types
// ===========================================================================

/// Metadata for a single frame within a sprite sequence.
///
/// The `origin_x` and `origin_y` fields define the anchor point relative to
/// the frame's top-left corner. Frames within a sequence may have different
/// sizes and origins, so a bounding rectangle must be computed to align them.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ImageInfo {
    /// X offset from the frame's top-left to its anchor point.
    pub origin_x: i32,
    /// Y offset from the frame's top-left to its anchor point.
    pub origin_y: i32,
    /// Frame width in pixels.
    pub width: i32,
    /// Frame height in pixels.
    pub height: i32,
    /// Size of the pixel data in bytes (width × height × 2).
    pub size_bytes: i64,
    /// File offset where this frame's RGB565 pixel data begins.
    pub image_start_position: u64,
}

/// Parsed information for a single animation sequence.
///
/// Contains the file offsets needed to navigate between sequences and
/// the metadata for all frames within this sequence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SequenceInfo {
    /// File offset where this sequence's frame metadata begins.
    /// Seek here before reading pixel data for rendering.
    pub sequence_start_position: u64,
    /// File offset after the last frame's pixel data.
    /// Seek here to continue scanning for the next sequence.
    pub sequence_end_position: u64,
    /// Number of frames in this sequence.
    pub frame_count: i32,
    /// Metadata for each frame in this sequence.
    pub frame_infos: Vec<ImageInfo>,
}

/// An RGB color decoded from a 16-bit RGB565 pixel value.
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameInfoJson {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SequenceInfoJson {
    pub sequence_index: usize,
    pub frame_count: usize,
    pub frames: Vec<FrameInfoJson>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpriteInfoJson {
    pub file_path: String,
    pub file_size: u64,
    pub sequence_count: usize,
    pub total_frames: usize,
    pub sequences: Vec<SequenceInfoJson>,
}

// ===========================================================================
// Low-level parsing
// ===========================================================================

/// Decodes a 16-bit RGB565 pixel value into an 8-bit RGB `Color`.
///
/// The RGB565 format uses 5 bits for red, 6 for green, and 5 for blue.
/// Values are expanded to 8-bit by left-shifting: R<<3, G<<2, B<<3.
///
/// A pixel value of `0` represents transparency and should be skipped
/// during rendering.
pub fn rgb16_565_produce_color(pixel: u16) -> Color {
    let red_mask: u16 = 0xF800;
    let green_mask: u16 = 0x7E0;
    let blue_mask: u16 = 0x1F;

    let red_value = (pixel & red_mask) >> 11;
    let green_value = (pixel & green_mask) >> 5;
    let blue_value = pixel & blue_mask;

    Color {
        r: (red_value << 3) as u8,
        g: (green_value << 2) as u8,
        b: (blue_value << 3) as u8,
    }
}

fn get_image_info(reader: &mut BufReader<File>) -> Result<ImageInfo> {
    reader.seek(SeekFrom::Current(6 * 4))?;

    let origin_x = reader.read_i32::<LittleEndian>()?;
    let origin_y = reader.read_i32::<LittleEndian>()?;
    let width = reader.read_i32::<LittleEndian>()?;
    let height = reader.read_i32::<LittleEndian>()?;

    let size_bytes = reader.read_u32::<LittleEndian>()?;
    let size_bytes = (size_bytes as i64) * 2;

    let image_start_position = reader.stream_position()?;

    if width < 1 || height < 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame width or height is zero",
        ));
    }

    Ok(ImageInfo {
        origin_x,
        origin_y,
        width,
        height,
        size_bytes,
        image_start_position,
    })
}

/// Parses a single sequence header and all its frame metadata.
///
/// The reader must be positioned at the start of a valid sequence header
/// (as found by `seek_next_sequence`). After this function returns, the
/// reader is positioned at `sequence_end_position` (after all pixel data),
/// ready for the next `seek_next_sequence` call.
///
/// To render the frames, seek back to `sequence_start_position` before
/// reading pixel data.
pub fn get_sequence_info(reader: &mut BufReader<File>) -> Result<SequenceInfo> {
    let mut stamp = reader.read_i32::<LittleEndian>()?;
    if stamp == 8 {
        stamp = reader.read_i32::<LittleEndian>()?;
    }

    let mut frame_count = 0;
    if stamp == 0 {
        frame_count = reader.read_i32::<LittleEndian>()?;
        _ = reader.read_i32::<LittleEndian>()?;
    }

    let start_position = reader.stream_position()?;
    let mut frame_infos = Vec::with_capacity(frame_count as usize);
    for _ in 0..frame_count {
        let image_info = get_image_info(reader)?;
        frame_infos.push(image_info);
        reader.seek(SeekFrom::Current(image_info.size_bytes))?;
    }
    let end_position = reader.stream_position()?;

    Ok(SequenceInfo {
        sequence_start_position: start_position,
        sequence_end_position: end_position,
        frame_count,
        frame_infos,
    })
}

/// Scans forward from `start_pos` to find the next valid sequence header.
///
/// Reads 15 consecutive i32 values (60 bytes) and checks for known sequence
/// patterns. If no match, advances by 4 bytes and retries.
///
/// Returns `true` if a valid sequence header was found (reader positioned
/// at the header). Returns `false` if no more sequences exist in the file.
pub fn seek_next_sequence(
    reader: &mut BufReader<File>,
    start_pos: u64,
    file_len: u64,
) -> Result<bool> {
    let mut number_of_skips = 0;

    loop {
        let pos = reader.stream_position()?;
        if pos + 60 >= file_len {
            break;
        }

        let mut ints = [0; 15];
        for int in &mut ints {
            *int = reader.read_i32::<LittleEndian>()?;
        }

        let valid = (ints[0] == 0
            && ints[1] > 0
            && ints[1] < 255
            && ints[2] == 0
            && ints[11] > 0
            && ints[12] > 0
            && i64::from(ints[11]) * i64::from(ints[12]) == i64::from(ints[13]))
            || (ints[0] == 8
                && ints[1] == 0
                && ints[2] > 0
                && ints[2] < 255
                && ints[3] == 0
                && ints[12] > 0
                && ints[13] > 0
                && i64::from(ints[12]) * i64::from(ints[13]) == i64::from(ints[14]));

        if valid {
            reader.seek(SeekFrom::Start(start_pos + (number_of_skips * 60)))?;
            return Ok(true);
        }
        number_of_skips += 1;
    }

    if number_of_skips == 1 {
        number_of_skips = 0;
    }
    reader.seek(SeekFrom::Start(start_pos + (number_of_skips * 60)))?;
    Ok(false)
}

// ===========================================================================
// Frame rendering helpers
// ===========================================================================

pub fn compute_rect(frames: &[ImageInfo]) -> (i32, i32, i32, i32) {
    let mut max_left = 1;
    let mut max_right = 1;
    let mut max_up = 1;
    let mut max_down = 1;
    for frame in frames {
        let left = frame.origin_x;
        let right = frame.width - frame.origin_x;
        let up = frame.origin_y;
        let down = frame.height - frame.origin_y;
        if right > max_right {
            max_right = right;
        }
        if left > max_left {
            max_left = left;
        }
        if up > max_up {
            max_up = up;
        }
        if down > max_down {
            max_down = down;
        }
    }
    let rect_x = max_left;
    let rect_y = max_up;
    let rect_w = if frames.len() == 1 {
        frames[0].width
    } else {
        max_left + max_right
    };
    let rect_h = if frames.len() == 1 {
        frames[0].height
    } else {
        max_up + max_down
    };
    (rect_x, rect_y, rect_w, rect_h)
}

pub fn compute_frame_offset(
    frames: &[ImageInfo],
    frame_idx: usize,
    rect_x: i32,
    rect_y: i32,
) -> (u32, u32) {
    let frame = &frames[frame_idx];
    let offset_x: i32 = if frames.len() == 1 {
        0
    } else {
        rect_x - frame.origin_x
    };
    let offset_y: i32 = if frames.len() == 1 {
        0
    } else {
        rect_y - frame.origin_y
    };
    (offset_x.unsigned_abs(), offset_y.unsigned_abs())
}

pub fn render_frame_to_rgba(
    reader: &mut BufReader<File>,
    frame: &ImageInfo,
    rect_w: u32,
    rect_h: u32,
    offset_x: u32,
    offset_y: u32,
) -> Result<RgbaImage> {
    let mut imgbuf = RgbaImage::new(rect_w, rect_h);
    let frame_width = frame.width.unsigned_abs();

    reader.seek(SeekFrom::Start(frame.image_start_position))?;
    for pixel_idx in 0..(frame.width.unsigned_abs() * frame.height.unsigned_abs()) as usize {
        let pixel = reader.read_u16::<LittleEndian>()?;
        if pixel == 0 {
            continue;
        }
        let color = rgb16_565_produce_color(pixel);
        let x = (pixel_idx as u32 % frame_width) + offset_x;
        let y = (pixel_idx as u32 / frame_width) + offset_y;
        imgbuf.put_pixel(x, y, image::Rgba([color.r, color.g, color.b, 255]));
    }
    Ok(imgbuf)
}

// ===========================================================================
// Sequence iteration helper
// ===========================================================================

fn for_each_sequence<F>(file_path: &Path, mut f: F) -> Result<()>
where
    F: FnMut(&mut BufReader<File>, &SequenceInfo, usize) -> Result<()>,
{
    let file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(268))?;

    let mut seq_index = 0;
    loop {
        let pos = reader.stream_position()?;
        if pos >= file_len {
            break;
        }

        let valid = seek_next_sequence(&mut reader, pos, file_len)?;
        if !valid {
            break;
        }

        let info = get_sequence_info(&mut reader)?;
        f(&mut reader, &info, seq_index)?;
        reader.seek(SeekFrom::Start(info.sequence_end_position))?;
        seq_index += 1;
    }

    Ok(())
}

// ===========================================================================
// CLI commands (file extraction)
// ===========================================================================

pub fn animation(file_path: &Path) -> Result<()> {
    for_each_sequence(file_path, |reader, info, seq_idx| {
        save_sequence_anim(reader, &info.frame_infos, seq_idx as i32)
    })?;
    println!("Finished");
    Ok(())
}

pub fn extract(file_path: &Path, out_file_prefix: String) -> Result<()> {
    for_each_sequence(file_path, |reader, info, seq_idx| {
        save_sequence(reader, &info.frame_infos, seq_idx as i32, &out_file_prefix)
    })?;
    println!("Finished");
    Ok(())
}

pub fn save_sequence_anim(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    let (rect_x, rect_y, rect_w, rect_h) = compute_rect(frames);
    let rect_w = rect_w.unsigned_abs();
    let rect_h = rect_h.unsigned_abs();

    println!("x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    let atlas_w = rect_w * (frames.len() as u32);
    let mut imgbuf = RgbaImage::new(atlas_w, rect_h);
    let mut offset_x: u32 = 0;

    for (i, frame) in frames.iter().enumerate() {
        let (_, offset_y) = compute_frame_offset(frames, i, rect_x as i32, rect_y as i32);

        let frame_rgba = render_frame_to_rgba(reader, frame, rect_w, rect_h, 0, offset_y)?;

        for (px, py, pixel) in frame_rgba.enumerate_pixels() {
            imgbuf.put_pixel(px + offset_x, py, *pixel);
        }

        offset_x += rect_w;
    }

    imgbuf
        .save(format!("image_{:?}.png", sequence_counter))
        .unwrap();

    Ok(())
}

pub fn save_sequence(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    out_file_prefix: &str,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    let (rect_x, rect_y, rect_w, rect_h) = compute_rect(frames);
    let rect_w = rect_w.unsigned_abs();
    let rect_h = rect_h.unsigned_abs();

    println!("x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    for (i, frame) in frames.iter().enumerate() {
        let (offset_x, offset_y) = compute_frame_offset(frames, i, rect_x, rect_y);

        let frame_rgba = render_frame_to_rgba(reader, frame, rect_w, rect_h, offset_x, offset_y)?;

        let outfile = format!("./{}_{:?}-{:?}.png", out_file_prefix, sequence_counter, i);
        frame_rgba.save(outfile).unwrap();
    }

    Ok(())
}

// ===========================================================================
// Library API (memory buffers)
// ===========================================================================

pub fn get_sequence_frames_as_pngs(
    reader: &mut BufReader<File>,
    info: &SequenceInfo,
) -> Result<Vec<Vec<u8>>> {
    let (rect_x, rect_y, rect_w, rect_h) = compute_rect(&info.frame_infos);
    let rect_w = rect_w.unsigned_abs();
    let rect_h = rect_h.unsigned_abs();

    let mut pngs = Vec::new();

    for (i, frame) in info.frame_infos.iter().enumerate() {
        let (offset_x, offset_y) = compute_frame_offset(&info.frame_infos, i, rect_x, rect_y);

        let frame_rgba = render_frame_to_rgba(reader, frame, rect_w, rect_h, offset_x, offset_y)?;

        let mut buf = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buf));
        encoder
            .write_image(frame_rgba.as_raw(), rect_w, rect_h, image::ColorType::Rgba8)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        pngs.push(buf);
    }

    Ok(pngs)
}

pub fn get_sprite_metadata(file_path: &Path) -> Result<Vec<usize>> {
    let mut frame_counts = Vec::new();
    for_each_sequence(file_path, |_, info, _| {
        frame_counts.push(info.frame_count as usize);
        Ok(())
    })?;
    Ok(frame_counts)
}

pub fn get_sequence_pngs_by_index(file_path: &Path, sequence_idx: usize) -> Result<Vec<Vec<u8>>> {
    let file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(268))?;

    let mut seq_counter = 0;
    loop {
        let pos = reader.stream_position()?;
        if pos >= file_len {
            break;
        }

        let valid = seek_next_sequence(&mut reader, pos, file_len)?;
        if !valid {
            break;
        }

        let info = get_sequence_info(&mut reader)?;
        if seq_counter == sequence_idx {
            reader.seek(SeekFrom::Start(info.sequence_start_position))?;
            return get_sequence_frames_as_pngs(&mut reader, &info);
        }
        seq_counter += 1;
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("Sequence {} not found", sequence_idx),
    ))
}

pub fn get_sprite_info(file_path: &Path) -> Result<SpriteInfoJson> {
    let file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(268))?;

    let mut sequences = Vec::new();
    let mut total_frames = 0;
    let mut seq_index = 0;

    loop {
        let pos = reader.stream_position()?;
        if pos >= file_len {
            break;
        }

        let valid = seek_next_sequence(&mut reader, pos, file_len)?;
        if !valid {
            break;
        }

        let info = get_sequence_info(&mut reader)?;
        let frame_count = info.frame_count as usize;
        total_frames += frame_count;

        let frames: Vec<FrameInfoJson> = info
            .frame_infos
            .iter()
            .map(|f| FrameInfoJson {
                origin_x: f.origin_x,
                origin_y: f.origin_y,
                width: f.width,
                height: f.height,
            })
            .collect();

        sequences.push(SequenceInfoJson {
            sequence_index: seq_index,
            frame_count,
            frames,
        });

        seq_index += 1;
    }

    Ok(SpriteInfoJson {
        file_path: file_path.to_string_lossy().to_string(),
        file_size: file_len,
        sequence_count: sequences.len(),
        total_frames,
        sequences,
    })
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_black() {
        let color = rgb16_565_produce_color(0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn color_red_max() {
        let color = rgb16_565_produce_color(0xF800);
        assert_eq!(color.r, 248);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn color_green_max() {
        let color = rgb16_565_produce_color(0x07E0);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 252);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn color_blue_max() {
        let color = rgb16_565_produce_color(0x001F);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 248);
    }

    #[test]
    fn color_white() {
        let color = rgb16_565_produce_color(0xFFFF);
        assert_eq!(color.r, 248);
        assert_eq!(color.g, 252);
        assert_eq!(color.b, 248);
    }

    #[test]
    fn color_magenta() {
        let color = rgb16_565_produce_color(0xF81F);
        assert_eq!(color.r, 248);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 248);
    }

    #[test]
    fn color_cyan() {
        let color = rgb16_565_produce_color(0x07FF);
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 252);
        assert_eq!(color.b, 248);
    }
}
