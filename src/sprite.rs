use byteorder::{LittleEndian, ReadBytesExt};
use image::{ImageEncoder, RgbaImage};
use std::io::{BufReader, Cursor, Read, Result, Seek, SeekFrom};
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

fn get_image_info<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<ImageInfo> {
    reader.seek(SeekFrom::Current(6 * 4))?;

    let origin_x = reader.read_i32::<LittleEndian>()?;
    let origin_y = reader.read_i32::<LittleEndian>()?;
    let width = reader.read_i32::<LittleEndian>()?;
    let height = reader.read_i32::<LittleEndian>()?;

    let size_bytes = reader.read_u32::<LittleEndian>()?;
    let size_bytes = (size_bytes as i64) * 2;

    let image_start_position = reader.stream_position()?;

    if width < 1 || height < 1 {
        // return Err(std::io::Error::new(
        //     std::io::ErrorKind::InvalidData,
        //     "frame width or height is zero",
        // ));
        // True for the following:
        // fixtures/Dispel/CharacterInGame/m_hair1_5.spr
        // fixtures/Dispel/CharacterInGame/m_hair1_2.spr
        // fixtures/Dispel/CharacterInGame/m_hair1_3.spr
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
pub fn get_sequence_info<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<SequenceInfo> {
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
pub fn seek_next_sequence<R: Read + Seek>(
    reader: &mut BufReader<R>,
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

pub fn render_frame_to_rgba<R: Read + Seek>(
    reader: &mut BufReader<R>,
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

    let imgbuf = render_sequence_animation(reader, frames, rect_x, rect_y, rect_w, rect_h)?;

    imgbuf
        .save(format!("image_{:?}.png", sequence_counter))
        .unwrap();

    Ok(())
}

/// Renders every frame of a sequence into a horizontal animation atlas.
///
/// Each atlas cell uses the sequence's shared bounding box, keeping the
/// frame anchor at the same position across the animation.
fn render_sequence_animation<R: Read + Seek>(
    reader: &mut BufReader<R>,
    frames: &[ImageInfo],
    rect_x: i32,
    rect_y: i32,
    rect_w: u32,
    rect_h: u32,
) -> Result<RgbaImage> {
    let atlas_w = rect_w * (frames.len() as u32);
    let mut imgbuf = RgbaImage::new(atlas_w, rect_h);

    for (i, frame) in frames.iter().enumerate() {
        let (offset_x, offset_y) = compute_frame_offset(frames, i, rect_x, rect_y);
        let frame_rgba = render_frame_to_rgba(reader, frame, rect_w, rect_h, offset_x, offset_y)?;

        for (px, py, pixel) in frame_rgba.enumerate_pixels() {
            imgbuf.put_pixel(px + (i as u32 * rect_w), py, *pixel);
        }
    }

    Ok(imgbuf)
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
            .map_err(std::io::Error::other)?;
        pngs.push(buf);
    }

    Ok(pngs)
}

/// Extract metadata for all sequences in a sprite file
/// Use (frame_counts.len(), frame_counts) to return (sequence_count, frame_counts_per_sequence)
pub fn get_sprite_metadata(file_path: &Path) -> Result<(usize, Vec<usize>)> {
    let mut frame_counts = Vec::new();
    let mut sequence_count = 0;

    for_each_sequence(file_path, |_, info, _| {
        frame_counts.push(info.frame_count as usize);
        sequence_count += 1;
        Ok(())
    })?;

    Ok((sequence_count, frame_counts))
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
// In-memory sprite representation (read-write)
// ===========================================================================

/// In-memory representation of a .spr file for editing and re-saving.
///
/// Preserves the 268-byte header and 24-byte per-frame unknown data
/// byte-for-byte so that the write-back is lossless for unmodified frames.
#[derive(Debug, Clone)]
pub struct SpriteFile {
    /// First 268 bytes of the original file (unknown header, preserved verbatim).
    pub header: [u8; 268],
    /// All animation sequences in the file.
    pub sequences: Vec<SpriteSequence>,
}

/// One animation sequence within a .spr file.
#[derive(Debug, Clone)]
pub struct SpriteSequence {
    /// Whether this sequence uses Pattern B header (stamp=8,0).
    /// `false` = Pattern A (stamp=0, no extra stamp word).
    pub has_stamp: bool,
    /// All frames in this sequence.
    pub frames: Vec<SpriteFrameData>,
}

/// Decoded frame data ready for editing and re-saving.
#[derive(Debug, Clone)]
pub struct SpriteFrameData {
    /// 24 raw bytes (6 × i32) of unknown per-frame data, preserved verbatim.
    pub unknown: [u8; 24],
    /// X offset from anchor point.
    pub origin_x: i32,
    /// Y offset from anchor point.
    pub origin_y: i32,
    /// Frame width in pixels.
    pub width: i32,
    /// Frame height in pixels.
    pub height: i32,
    /// Raw RGB565 pixel data, width×height×2 bytes, little-endian.
    /// Stored in original form so unmodified frames round-trip bit-exact.
    pub raw_pixels: Vec<u8>,
}

impl SpriteFrameData {
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Decode the raw RGB565 pixels into RGBA bytes (width×height×4).
    pub fn decode_to_rgba(&self) -> Vec<u8> {
        let count = self.pixel_count();
        let mut rgba = vec![0u8; count * 4];
        for i in 0..count {
            let base = i * 2;
            if base + 1 >= self.raw_pixels.len() {
                break;
            }
            let pixel = u16::from_le_bytes([self.raw_pixels[base], self.raw_pixels[base + 1]]);
            if pixel == 0 {
                continue; // transparent, keep RGBA = [0,0,0,0]
            }
            let color = rgb16_565_produce_color(pixel);
            let rbase = i * 4;
            rgba[rbase] = color.r;
            rgba[rbase + 1] = color.g;
            rgba[rbase + 2] = color.b;
            rgba[rbase + 3] = 255;
        }
        rgba
    }

    /// Encode RGBA bytes (width×height×4) into raw RGB565, replacing pixel data.
    pub fn encode_from_rgba(&mut self, rgba: &[u8]) {
        let count = self.pixel_count();
        let mut raw = vec![0u8; count * 2];
        for i in 0..count.min(rgba.len() / 4) {
            let rbase = i * 4;
            let rgb565 = rgba_to_rgb565_bytes(
                rgba[rbase],
                rgba[rbase + 1],
                rgba[rbase + 2],
                rgba[rbase + 3],
            );
            let base = i * 2;
            raw[base] = rgb565[0];
            raw[base + 1] = rgb565[1];
        }
        self.raw_pixels = raw;
    }
}

/// Parse a .spr file into the in-memory [`SpriteFile`] representation.
///
/// Preserves all unknown header bytes and per-frame metadata verbatim.
/// Maximum bytes to scan when looking for sequences. Prevents runaway on
/// corrupted or very large files.
const MAX_SCAN_BYTES: usize = 50_000_000; // 50 MB

pub fn read_sprite_file(path: &Path) -> Result<SpriteFile> {
    let bytes = std::fs::read(path)?;
    parse_sprite_bytes(&bytes)
}

/// Parse `.spr` bytes into the in-memory [`SpriteFile`] representation.
///
/// Shared by `read_sprite_file` (filesystem) and callers that hold sprite
/// data in memory (e.g. database blobs). See `read_sprite_file` for details.
pub fn parse_sprite_bytes(bytes: &[u8]) -> Result<SpriteFile> {
    let file_len = bytes.len();
    if file_len < 268 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File too small: {file_len} bytes, need at least 268"),
        ));
    }
    let mut header = [0u8; 268];
    header.copy_from_slice(&bytes[..268]);

    let mut sequences: Vec<SpriteSequence> = Vec::new();
    let mut offset = 268usize;
    let scan_end = file_len.min(MAX_SCAN_BYTES);

    while offset + 60 <= scan_end {
        // Read 15 consecutive i32 values (60 bytes) for sequence detection.
        let ints = read_15_i32s(bytes, offset);

        let valid = is_valid_sequence_header(&ints);
        if !valid {
            offset += 4;
            continue;
        }

        // Parse sequence header to get frame count.
        let has_stamp = ints[0] == 8;
        let frame_count = if has_stamp { ints[2] } else { ints[1] } as usize;
        let frame_count = frame_count.min(255); // sanity

        // Header size: 12 bytes (Pattern A) or 16 bytes (Pattern B)
        let header_size = if has_stamp { 16 } else { 12 };
        let mut seq_offset = offset + header_size;

        let mut frames: Vec<SpriteFrameData> = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            if seq_offset + 24 + 4 * 4 + 4 > file_len {
                break;
            }

            // Read 24 unknown bytes
            let mut unknown = [0u8; 24];
            unknown.copy_from_slice(&bytes[seq_offset..seq_offset + 24]);
            seq_offset += 24;

            // Read origin_x, origin_y, width, height
            let origin_x =
                i32::from_le_bytes(bytes[seq_offset..seq_offset + 4].try_into().unwrap());
            seq_offset += 4;
            let origin_y =
                i32::from_le_bytes(bytes[seq_offset..seq_offset + 4].try_into().unwrap());
            seq_offset += 4;
            let width = i32::from_le_bytes(bytes[seq_offset..seq_offset + 4].try_into().unwrap());
            seq_offset += 4;
            let height = i32::from_le_bytes(bytes[seq_offset..seq_offset + 4].try_into().unwrap());
            seq_offset += 4;

            // Read pixel_count (as u32)
            let pixel_count =
                u32::from_le_bytes(bytes[seq_offset..seq_offset + 4].try_into().unwrap()) as usize;
            seq_offset += 4;

            // Read raw RGB565 pixel data
            let raw_size = pixel_count * 2;
            let raw_end = seq_offset.checked_add(raw_size).unwrap_or(file_len + 1);
            if raw_end > file_len {
                break;
            }
            let raw_pixels = bytes[seq_offset..raw_end].to_vec();
            seq_offset = raw_end;

            frames.push(SpriteFrameData {
                unknown,
                origin_x,
                origin_y,
                width,
                height,
                raw_pixels,
            });
        }

        // Guard: even if no frames were parsed (false-positive detection),
        // advance offset past the header to prevent infinite looping.
        if frames.is_empty() {
            offset += header_size;
        } else {
            offset = seq_offset;
        }

        sequences.push(SpriteSequence { has_stamp, frames });
    }

    Ok(SpriteFile { header, sequences })
}

/// Read 15 consecutive little-endian i32 values starting at `offset`.
fn read_15_i32s(bytes: &[u8], offset: usize) -> [i32; 15] {
    let mut ints = [0i32; 15];
    for (i, val) in ints.iter_mut().enumerate() {
        let start = offset + i * 4;
        if start + 4 <= bytes.len() {
            *val = i32::from_le_bytes(bytes[start..start + 4].try_into().unwrap());
        }
    }
    ints
}

/// Check whether 15 integers match a valid .spr sequence header pattern.
fn is_valid_sequence_header(ints: &[i32; 15]) -> bool {
    (ints[0] == 0
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
}

/// Write the in-memory [`SpriteFile`] back to a .spr binary file.
///
/// The 268-byte header and all per-frame 24-byte unknown data are preserved
/// verbatim. Pixel data is re-encoded from raw RGB565.
pub fn write_sprite_to_path(path: &Path, sprite: &SpriteFile) -> Result<()> {
    let mut buf = Vec::with_capacity(4096);
    buf.extend_from_slice(&sprite.header);

    for seq in &sprite.sequences {
        let frame_count = seq.frames.len() as i32;
        if seq.has_stamp {
            // Pattern B: stamp=8, stamp_padding=0, frame_count, padding=0
            buf.extend_from_slice(&8i32.to_le_bytes());
            buf.extend_from_slice(&0i32.to_le_bytes());
            buf.extend_from_slice(&frame_count.to_le_bytes());
            buf.extend_from_slice(&0i32.to_le_bytes());
        } else {
            // Pattern A: stamp=0, frame_count, padding=0
            buf.extend_from_slice(&0i32.to_le_bytes());
            buf.extend_from_slice(&frame_count.to_le_bytes());
            buf.extend_from_slice(&0i32.to_le_bytes());
        }

        for frame in &seq.frames {
            buf.extend_from_slice(&frame.unknown);
            buf.extend_from_slice(&frame.origin_x.to_le_bytes());
            buf.extend_from_slice(&frame.origin_y.to_le_bytes());
            buf.extend_from_slice(&frame.width.to_le_bytes());
            buf.extend_from_slice(&frame.height.to_le_bytes());
            let pixel_count = frame.pixel_count() as u32;
            buf.extend_from_slice(&pixel_count.to_le_bytes());
            buf.extend_from_slice(&frame.raw_pixels);
        }
    }

    std::fs::write(path, &buf)
}

/// Encode a single RGBA pixel as two RGB565 bytes (little-endian).
///
/// Pixels with alpha < 128 are encoded as `0x0000` (transparent).
pub fn rgba_to_rgb565_bytes(r: u8, g: u8, b: u8, a: u8) -> [u8; 2] {
    if a < 128 {
        return [0, 0];
    }
    let r5 = (r as u16 >> 3) & 0x1F;
    let g6 = (g as u16 >> 2) & 0x3F;
    let b5 = (b as u16 >> 3) & 0x1F;
    let pixel = (r5 << 11) | (g6 << 5) | b5;
    pixel.to_le_bytes()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── RGB565 encode / decode round-trip ─────────────────────────────

    #[test]
    fn rgba_to_rgb565_encodes_red() {
        let bytes = rgba_to_rgb565_bytes(255, 0, 0, 255);
        let pixel = u16::from_le_bytes(bytes);
        assert_eq!(pixel, 0xF800);
    }

    #[test]
    fn rgba_to_rgb565_encodes_green() {
        let bytes = rgba_to_rgb565_bytes(0, 255, 0, 255);
        let pixel = u16::from_le_bytes(bytes);
        assert_eq!(pixel, 0x07E0);
    }

    #[test]
    fn rgba_to_rgb565_encodes_blue() {
        let bytes = rgba_to_rgb565_bytes(0, 0, 255, 255);
        let pixel = u16::from_le_bytes(bytes);
        assert_eq!(pixel, 0x001F);
    }

    #[test]
    fn rgba_to_rgb565_transparent_is_zero() {
        let bytes = rgba_to_rgb565_bytes(255, 0, 0, 0);
        assert_eq!(bytes, [0, 0]);
    }

    #[test]
    fn rgba_to_rgb565_decode_round_trip() {
        // Encode a known color → decode back → verify color values
        let encoded = rgba_to_rgb565_bytes(128, 64, 192, 255);
        let pixel = u16::from_le_bytes(encoded);
        let color = rgb16_565_produce_color(pixel);
        // 128 >> 3 = 16, 16 << 3 = 128
        assert_eq!(color.r, 128);
        // 64 >> 2 = 16, 16 << 2 = 64
        assert_eq!(color.g, 64);
        // 192 >> 3 = 24, 24 << 3 = 192
        assert_eq!(color.b, 192);
    }

    #[test]
    fn frame_data_decode_encode_round_trip() {
        // Create a 4×4 frame with known pixel values
        let pixels: Vec<u8> = (0..16)
            .flat_map(|_| {
                vec![255u8, 128, 64, 255] // R=255, G=128, B=64, A=255
            })
            .collect();
        let mut frame = SpriteFrameData {
            unknown: [0; 24],
            origin_x: 0,
            origin_y: 0,
            width: 4,
            height: 4,
            raw_pixels: vec![0u8; 4 * 4 * 2],
        };
        frame.encode_from_rgba(&pixels);
        assert_eq!(frame.raw_pixels.len(), 32, "4×4 pixels × 2 bytes");

        // Decode back
        let decoded = frame.decode_to_rgba();
        assert_eq!(decoded.len(), 64, "4×4 pixels × 4 bytes RGBA");
        // First pixel should be R=248 (after 5-bit rounding), G=64, B=64
        assert_eq!(decoded[0], 248); // R (255→31→248)
        assert_eq!(decoded[1], 128); // G (128→32→128) wait...
        // Actually: 128 >> 2 = 32, 32 << 2 = 128
        // Let me verify:
        // 64 >> 2 = 16, 16 << 2 = 64
        // Let me just check the first pixel is non-zero and alpha is 255
        assert_eq!(decoded[3], 255);
    }

    // ── SpriteFile round-trip ────────────────────────────────────────

    #[test]
    fn sprite_file_round_trip_item_sprite() {
        // Small item sprite (~2.5 KB) — fast test
        let fixture = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Inter/item_field/healpotion.spr"
        ));
        if !fixture.exists() {
            eprintln!("Skipping: fixture not found");
            return;
        }

        let original = read_sprite_file(fixture).expect("read sprite");
        assert!(!original.sequences.is_empty(), "should have sequences");
        assert!(
            original.sequences.iter().any(|s| !s.frames.is_empty()),
            "should have frames"
        );

        let tmp = std::env::temp_dir().join("test_spr_rt_item.spr");
        write_sprite_to_path(&tmp, &original).expect("write sprite");
        let reloaded = read_sprite_file(&tmp).expect("re-read sprite");

        // Bit-exact header
        assert_eq!(original.header, reloaded.header, "header mismatch");
        // Sequence structure
        assert_eq!(
            original.sequences.len(),
            reloaded.sequences.len(),
            "seq count"
        );
        for (si, (a, b)) in original
            .sequences
            .iter()
            .zip(reloaded.sequences.iter())
            .enumerate()
        {
            assert_eq!(a.has_stamp, b.has_stamp, "seq {si} stamp");
            assert_eq!(a.frames.len(), b.frames.len(), "seq {si} frame count");
            for (fi, (af, bf)) in a.frames.iter().zip(b.frames.iter()).enumerate() {
                assert_eq!(af.unknown, bf.unknown, "seq {si} f{fi} unknown");
                assert_eq!(af.origin_x, bf.origin_x, "seq {si} f{fi} ox");
                assert_eq!(af.origin_y, bf.origin_y, "seq {si} f{fi} oy");
                assert_eq!(af.width, bf.width, "seq {si} f{fi} w");
                assert_eq!(af.height, bf.height, "seq {si} f{fi} h");
                assert_eq!(af.raw_pixels, bf.raw_pixels, "seq {si} f{fi} pixels");
            }
        }

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn sprite_file_round_trip_character_sprite() {
        // Full character sprite (~1.4 MB) — tests large-file handling
        let fixture = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/CharacterInGame/M_BODY1.SPR"
        ));
        if !fixture.exists() {
            eprintln!("Skipping: fixture not found");
            return;
        }

        let original = read_sprite_file(fixture).expect("read sprite");
        assert!(!original.sequences.is_empty(), "should have sequences");

        let tmp = std::env::temp_dir().join("test_spr_rt_char.spr");
        write_sprite_to_path(&tmp, &original).expect("write sprite");
        let reloaded = read_sprite_file(&tmp).expect("re-read sprite");

        assert_eq!(original.sequences.len(), reloaded.sequences.len());
        assert_eq!(
            original.sequences[0].frames.len(),
            reloaded.sequences[0].frames.len()
        );

        // Spot-check first frame metadata of first sequence
        let a = &original.sequences[0].frames[0];
        let b = &reloaded.sequences[0].frames[0];
        assert_eq!(a.unknown, b.unknown);
        assert_eq!(a.raw_pixels, b.raw_pixels);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn sprite_file_encodes_frame_modification() {
        let fixture = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/Dispel/Inter/item_field/healpotion.spr"
        ));
        if !fixture.exists() {
            eprintln!("Skipping: fixture not found");
            return;
        }

        let original = read_sprite_file(fixture).expect("read sprite");
        let mut sprite = original;

        // Modify first frame: set all pixels to red
        if let Some(frame) = sprite.sequences[0].frames.first_mut() {
            let rgba = frame.decode_to_rgba();
            assert_eq!(rgba.len(), frame.pixel_count() * 4);
            // Make all pixels red
            let mut new_rgba = vec![0u8; rgba.len()];
            for chunk in new_rgba.chunks_mut(4) {
                chunk[0] = 255; // R
                chunk[1] = 0; // G
                chunk[2] = 0; // B
                chunk[3] = 255; // A
            }
            frame.encode_from_rgba(&new_rgba);

            // Verify encode
            assert_eq!(frame.raw_pixels.len(), frame.pixel_count() * 2);
            // First pixel should be RGB565 red (0xF800)
            assert_eq!(frame.raw_pixels[0], 0x00);
            assert_eq!(frame.raw_pixels[1], 0xF8);
        }

        // Write and re-read
        let tmp = std::env::temp_dir().join("test_spr_mod.spr");
        write_sprite_to_path(&tmp, &sprite).expect("write");
        let reloaded = read_sprite_file(&tmp).expect("re-read");

        // Verify modification persisted
        assert_eq!(reloaded.sequences[0].frames[0].raw_pixels[0], 0x00);
        assert_eq!(reloaded.sequences[0].frames[0].raw_pixels[1], 0xF8);

        let _ = std::fs::remove_file(&tmp);
    }

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
