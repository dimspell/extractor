use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageEncoder, RgbaImage};
use std::io::{BufReader, Cursor, Result, Seek, SeekFrom};
use std::{fs::File, path::Path};

// ===========================================================================
// DISPEL SPRITE FILE FORMAT (.SPR)
// ===========================================================================
//
// ASCII Structure:
//
// +--------------------------------------+
// | Sprite File - Animation Sequences   |
// +--------------------------------------+
// | Encoding: Binary (Little-Endian)     |
// | Color Format: RGB565 (2 bytes/pixel)|
// | Header Offset: 268 bytes             |
// +--------------------------------------+
// | [File Header]                       |
// | - 268 bytes unknown header           |
// +--------------------------------------+
// | [Sequence 1]                        |
// | - Sequence header (variable)        |
// | - Frame 1 metadata                 |
// | - Frame 1 pixel data (RGB565)       |
// | - Frame 2 metadata                 |
// | - Frame 2 pixel data (RGB565)       |
// | - ...                              |
// +--------------------------------------+
// | [Sequence 2]                        |
// | - Sequence header                  |
// | - Frame metadata + pixel data       |
// | - ...                              |
// +--------------------------------------+
//
// SEQUENCE STRUCTURE:
// - Stamp: i32 (8 or 0) - sequence marker
// - Frame count: i32 - number of frames
// - For each frame:
//   - 6 × i32 unknown data
//   - origin_x: i32 - X offset from origin
//   - origin_y: i32 - Y offset from origin
//   - width: i32 - frame width in pixels
//   - height: i32 - frame height in pixels
//   - size_bytes: u32 - pixel data size
//   - RGB565 pixel data (width × height × 2 bytes)
//
// COLOR FORMAT:
// - RGB565: 5 bits red, 6 bits green, 5 bits blue
// - 2 bytes per pixel
// - 0x0000 = transparent
// - Little-endian byte order
//
// FILE PURPOSE:
// Stores character sprites, animations, and visual effects.
// Used for rendering NPCs, monsters, party members, and special
// effects in the isometric game world.
//
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
    let frame_width: u32 = frame.width.unsigned_abs();

    reader.seek(SeekFrom::Start(frame.image_start_position))?;
    for pixel_idx in 0..(frame.width.unsigned_abs() * frame.height.unsigned_abs()) as usize {
        let pixel = reader.read_u16::<LittleEndian>()?;
        if pixel == 0 {
            continue;
        }
        let color = rgb16_565_produce_color(pixel);
        let x: u32 = (pixel_idx as u32 % frame_width) + offset_x;
        let y: u32 = (pixel_idx as u32 / frame_width) + offset_y;
        imgbuf.put_pixel(x, y, image::Rgba([color.r, color.g, color.b, 255]));
    }
    Ok(imgbuf)
}

pub fn animation(file_path: &Path) -> Result<()> {
    let file = File::open(file_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    // Start from 268th byte
    reader.seek(SeekFrom::Start(268))?;

    let mut sequence_counter = 0;
    loop {
        let pos = reader.stream_position()?;
        if pos >= file_len {
            println!("{pos} >= {file_len}");
            break;
        }
        let valid_sprite_sequence = seek_next_sequence(&mut reader, pos, file_len)?;
        if valid_sprite_sequence {
            let info: SequenceInfo = get_sequence_info(&mut reader)?;
            save_sequence_anim(&mut reader, &info.frame_infos, sequence_counter)?;
            sequence_counter += 1;
        } else {
            break;
        }
    }
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
    let mut imgbuf: RgbaImage = image::ImageBuffer::new(atlas_w, rect_h);
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

pub fn extract(file_path: &Path, out_file_prefix: String) -> Result<()> {
    let file = File::open(file_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    // Start from 268th byte
    reader.seek(SeekFrom::Start(268))?;

    let mut sequence_counter = 0;
    loop {
        let pos = reader.stream_position()?;
        if pos >= file_len {
            println!("{pos} >= {file_len}");
            break;
        }
        let valid_sprite_sequence = seek_next_sequence(&mut reader, pos, file_len)?;
        if valid_sprite_sequence {
            let info: SequenceInfo = get_sequence_info(&mut reader)?;
            save_sequence(
                &mut reader,
                &info.frame_infos,
                sequence_counter,
                &out_file_prefix,
            )?;
            sequence_counter += 1;
        } else {
            break;
        }
    }
    println!("Finished");
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

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn rgb16_565_produce_color(pixel: u16) -> Color {
    let red_mask: u16 = 0xF800;
    let green_mask: u16 = 0x7E0;
    let blue_mask: u16 = 0x1F;

    let red_value = (pixel & red_mask) >> 11;
    let green_value = (pixel & green_mask) >> 5;
    let blue_value = pixel & blue_mask;

    let red: u8 = (red_value << 3).try_into().unwrap();
    let green: u8 = (green_value << 2).try_into().unwrap();
    let blue: u8 = (blue_value << 3).try_into().unwrap();

    Color {
        b: blue,
        g: green,
        r: red,
    }
}

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

pub struct SequenceInfo {
    pub sequence_start_position: u64,
    pub sequence_end_position: u64,
    pub frame_count: i32,
    pub frame_infos: Vec<ImageInfo>,
}

pub fn get_sequence_info(reader: &mut BufReader<File>) -> Result<SequenceInfo> {
    let mut info: SequenceInfo = SequenceInfo {
        sequence_start_position: 0,
        sequence_end_position: 0,
        frame_count: 0,
        frame_infos: vec![
            ImageInfo {
                origin_x: 0,
                origin_y: 0,
                width: 0,
                height: 0,
                size_bytes: 0,
                image_start_position: 0
            };
            0
        ],
    };

    let mut stamp = reader.read_i32::<LittleEndian>()?; // 8
    if stamp == 8 {
        stamp = reader.read_i32::<LittleEndian>()?; // 0
    }
    if stamp == 0 {
        let frame_count = reader.read_i32::<LittleEndian>()?; // 1
        _ = reader.read_i32::<LittleEndian>()?; // 0
        info.frame_count = frame_count;
        info.frame_infos = vec![
            ImageInfo {
                origin_x: 0,
                origin_y: 0,
                width: 0,
                height: 0,
                size_bytes: 0,
                image_start_position: 0
            };
            frame_count as usize
        ];
    }

    let start_position = reader.stream_position()?;
    info.sequence_start_position = start_position;
    for i in 0..info.frame_count as usize {
        let image_info = get_image_info(reader)?;
        info.frame_infos[i] = image_info;
        reader.seek(SeekFrom::Current(image_info.size_bytes))?;
    }
    info.sequence_end_position = reader.stream_position()?;
    reader.seek(SeekFrom::Start(start_position))?;

    Ok(info)
}

#[derive(Debug, Clone, Copy)]
pub struct ImageInfo {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: i32,
    pub height: i32,
    pub size_bytes: i64,
    pub image_start_position: u64,
}

fn get_image_info(reader: &mut BufReader<File>) -> Result<ImageInfo> {
    let mut info: ImageInfo = ImageInfo {
        origin_x: 0,
        origin_y: 0,
        width: 0,
        height: 0,
        size_bytes: 0,
        image_start_position: 0,
    };

    reader.seek(SeekFrom::Current(6 * 4))?; // some data

    info.origin_x = reader.read_i32::<LittleEndian>()?;
    info.origin_y = reader.read_i32::<LittleEndian>()?;
    info.width = reader.read_i32::<LittleEndian>()?;
    info.height = reader.read_i32::<LittleEndian>()?;

    let size_bytes = reader.read_u32::<LittleEndian>()?;
    info.size_bytes = size_bytes.into();
    info.size_bytes *= 2;

    let pos: u64 = reader.stream_position()?;
    info.image_start_position = pos;

    if info.width < 1 || info.height < 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame width or height is zero",
        ));
    }
    Ok(info)
}

pub fn seek_next_sequence(
    reader: &mut BufReader<File>,
    start_pos: u64,
    file_len: u64,
) -> Result<bool> {
    let mut valid_sprite_seq = false;
    let mut number_of_skips = 0;

    while !valid_sprite_seq {
        let pos: u64 = reader.stream_position()?;
        // println!("Seek position: {pos} of {file_len}. Skips: {number_of_skips}");
        if pos + 60 >= file_len {
            // println!("break at skips: {:?}", number_of_skips);
            break;
        }

        let mut ints = [0; 15];
        for int in &mut ints {
            *int = reader.read_i32::<LittleEndian>()?;
        }

        if (ints[0] == 0
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
                && i64::from(ints[12]) * i64::from(ints[13]) == i64::from(ints[14]))
        {
            valid_sprite_seq = true;
        } else {
            // println!("Data (i32): {ints:?}");
            number_of_skips += 1;
        }
    }

    if number_of_skips == 1 {
        number_of_skips = 0;
    }

    reader.seek(SeekFrom::Start(start_pos + (number_of_skips * 4 * 15)))?;
    Ok(valid_sprite_seq)
}

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
    let file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(268))?;

    let mut frame_counts = Vec::new();
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
        frame_counts.push(info.frame_count as usize);
    }

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
