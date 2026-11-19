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
//   - leading i32 sequence count, followed by counted, variable-length groups
//   - each sequence has a 268-byte header and zero or more frame groups
//   - each group: 12-byte header → frame metadata blocks → pixel data
//   - 16-bit RGB555 or RGB565, 0x0000=transparent, little-endian
//   - Frames have origin_x/origin_y for alignment in a bounding rect
//
// Reading flow:
//   1. read sequence_count at offset 0
//   2. read every 268-byte sequence header and its groups sequentially
//   3. parse each non-empty group into a SequenceInfo
//   4. use each frame's recorded pixel-data offset to render it
//
// Decoding authority:
//   `read_sprite_sequence_infos` is the single byte-level decoder. It yields
//   the offset-based streaming view (`SequenceInfo`/`ImageInfo`) used by the
//   CLI extractors and the SQLite importer. The editor-facing `SpriteFile`
//   model is a conversion layer built on that same decoder by
//   `parse_sprite_bytes`: it copies the per-frame bytes into owned buffers and
//   synthesizes a normalized header. Both views share `pixels_to_rgba`, so a
//   given 16-bit pixel always decodes to the same RGBA value. Use
//   `SpriteRenderOptions` to select decode/rendering behavior explicitly.
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
    /// Size of the pixel data in bytes (width × height × `bytes_per_pixel`).
    pub size_bytes: i64,
    /// Number of bytes occupied by one pixel in the source stream.
    ///
    /// Normal groups use two-byte RGB555 or RGB565 pixels. The original loader
    /// also recognizes one-byte groups; they are parsed and skipped correctly,
    /// but cannot be rendered without their palette.
    pub bytes_per_pixel: u8,
    /// Channel packing for a two-byte pixel.
    pub pixel_format: SpritePixelFormat,
    /// File offset where this frame's pixel data begins.
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

// Color conversion and the 16-bit channel packing are owned by `crate::decode`
// (the version-agnostic rendering abstraction). Re-exported here so all
// existing `sprite::Color` / `sprite::SpritePixelFormat` paths keep working.
pub use crate::decode::{Color, SpritePixelFormat};

/// Explicit decode/render options for every sprite entry point.
///
/// Passing one value at the call boundary replaces the former `*_with_black`
/// boolean twins and scattered `apply_origin_offsets` flags. `Default` keeps
/// the historical behavior of the plain functions: `0x0000` pixels stay
/// transparent and frames are not shifted within an animation atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteRenderOptions {
    /// Render raw `0x0000` pixels as opaque black instead of transparent.
    pub preserve_black: bool,
    /// Place frames inside the sequence's shared bounding box so their anchor
    /// stays fixed across an animation atlas. Only affects atlas rendering;
    /// individual-frame exports always center frames in the sequence rect.
    pub apply_origin_offsets: bool,
}

impl SpriteRenderOptions {
    /// Options with only `preserve_black` enabled, matching
    /// `*_with_black(..., true)` calls.
    pub const fn with_black() -> Self {
        Self {
            preserve_black: true,
            apply_origin_offsets: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FrameInfoJson {
    pub origin_x: i32,
    pub origin_y: i32,
    pub width: i32,
    pub height: i32,
    pub pixel_format: SpritePixelFormat,
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

// The 16-bit → RGB888 conversion lives in `crate::decode` and is shared with
// the tileset renderer and the GUI. These aliases preserve the historical
// names used across the codebase (and by external doc comments).
pub use crate::decode::{
    rgb16_to_color as rgb16_produce_color, rgb555_to_color as rgb16_555_produce_color,
    rgb565_to_color as rgb16_565_produce_color,
};

fn get_image_info_with_pixel_stride<R: Read + Seek>(
    reader: &mut BufReader<R>,
    bytes_per_pixel: u8,
) -> Result<ImageInfo> {
    reader.seek(SeekFrom::Current(6 * 4))?;

    let origin_x = reader.read_i32::<LittleEndian>()?;
    let origin_y = reader.read_i32::<LittleEndian>()?;
    let width = reader.read_i32::<LittleEndian>()?;
    let height = reader.read_i32::<LittleEndian>()?;

    let pixel_count = reader.read_u32::<LittleEndian>()?;
    let size_bytes = i64::from(pixel_count)
        .checked_mul(i64::from(bytes_per_pixel))
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large")
        })?;

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
        bytes_per_pixel,
        pixel_format: SpritePixelFormat::Rgb565,
        image_start_position,
    })
}

fn get_image_info<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<ImageInfo> {
    get_image_info_with_pixel_stride(reader, 2)
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

fn checked_count(value: i32, field: &str) -> Result<usize> {
    usize::try_from(value).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("negative {field}: {value}"),
        )
    })
}

fn ensure_sprite_archive<R: Read + Seek>(reader: &mut BufReader<R>) -> Result<()> {
    reader.seek(SeekFrom::Start(0))?;
    let mut magic = [0_u8; 2];
    reader.read_exact(&mut magic)?;
    if magic == *b"BM" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "BMP data uses a .spr extension; it is not a DISPEL® sprite archive",
        ));
    }
    reader.seek(SeekFrom::Start(0))?;
    Ok(())
}

/// Read the sprite archive in the order used by the original game loader.
///
/// The public API calls each non-empty group a sequence. This preserves the
/// existing extractor/manifest convention while following the archive's real
/// layout: a leading sequence count, then 268-byte sequence headers, then
/// variable-length group records and their frame data.
fn read_sprite_sequence_infos<R: Read + Seek>(
    reader: &mut BufReader<R>,
) -> Result<Vec<SequenceInfo>> {
    ensure_sprite_archive(reader)?;

    let sequence_count = checked_count(reader.read_i32::<LittleEndian>()?, "sequence count")?;
    let mut sequences = Vec::new();

    for _ in 0..sequence_count {
        // Per-sequence header: id/unknown (4), name (260), group count (4).
        reader.seek(SeekFrom::Current(4 + 260))?;
        let group_count = checked_count(reader.read_i32::<LittleEndian>()?, "group count")?;

        for _ in 0..group_count {
            // The group header is { variant, frame_count, unknown }. Variant 1
            // has one byte per pixel; the normal (and all shipped) variant has
            // RGB565 pixels occupying two bytes each.
            let variant = reader.read_i32::<LittleEndian>()?;
            let frame_count = checked_count(reader.read_i32::<LittleEndian>()?, "frame count")?;
            _ = reader.read_i32::<LittleEndian>()?;
            let bytes_per_pixel = if variant == 1 { 1 } else { 2 };

            let sequence_start_position = reader.stream_position()?;
            let mut frame_infos = Vec::with_capacity(frame_count);
            for _ in 0..frame_count {
                let image_info = get_image_info_with_pixel_stride(reader, bytes_per_pixel)?;
                reader.seek(SeekFrom::Current(image_info.size_bytes))?;
                frame_infos.push(image_info);
            }
            let sequence_end_position = reader.stream_position()?;

            // Empty groups are direction/animation slots, not extractable
            // image sequences. The legacy scanner omitted them as well.
            if !frame_infos.is_empty() {
                sequences.push(SequenceInfo {
                    sequence_start_position,
                    sequence_end_position,
                    frame_count: frame_count as i32,
                    frame_infos,
                });
            }
        }
    }

    assign_16bit_pixel_format(reader, &mut sequences)?;

    Ok(sequences)
}

/// The archive has no explicit RGB555/RGB565 tag. The original blitter selects
/// its masks from the display configuration; assets themselves reveal RGB555
/// by never using bit 15. `Menu/MBack.spr` is the verified shipped example.
fn assign_16bit_pixel_format<R: Read + Seek>(
    reader: &mut BufReader<R>,
    sequences: &mut [SequenceInfo],
) -> Result<()> {
    let mut saw_nontransparent_pixel = false;
    let mut uses_bit_15 = false;

    for sequence in sequences.iter() {
        for frame in &sequence.frame_infos {
            if frame.bytes_per_pixel != 2 {
                continue;
            }
            reader.seek(SeekFrom::Start(frame.image_start_position))?;
            let word_count = u64::try_from(frame.size_bytes / 2).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid frame size")
            })?;
            for _ in 0..word_count {
                let pixel = reader.read_u16::<LittleEndian>()?;
                if pixel != 0 {
                    saw_nontransparent_pixel = true;
                    uses_bit_15 |= pixel & 0x8000 != 0;
                }
            }
        }
    }

    let pixel_format = if saw_nontransparent_pixel && !uses_bit_15 {
        SpritePixelFormat::Rgb555
    } else {
        SpritePixelFormat::Rgb565
    };
    for sequence in sequences {
        for frame in &mut sequence.frame_infos {
            if frame.bytes_per_pixel == 2 {
                frame.pixel_format = pixel_format;
            }
        }
    }
    Ok(())
}

/// Return every non-empty frame group in a standalone `.spr` archive.
pub fn get_sprite_sequence_infos(file_path: &Path) -> Result<Vec<SequenceInfo>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    read_sprite_sequence_infos(&mut reader)
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

/// Decode 16-bit sprite pixels into a tightly packed RGBA buffer.
///
/// This is the one place where sprite pixels become RGBA. Both the streaming
/// reader (`render_frame_to_rgba_with_options`) and the owned editor model
/// (`SpriteFrameData::decode_to_rgba_with_options`) call it, so they cannot
/// disagree on color or alpha for the same bytes. Missing pixels (truncated
/// input) stay transparent.
fn pixels_to_rgba(
    pixels: impl Iterator<Item = u16>,
    pixel_count: usize,
    pixel_format: SpritePixelFormat,
    preserve_black: bool,
) -> Vec<u8> {
    let mut rgba = vec![0u8; pixel_count * 4];
    for (i, pixel) in pixels.take(pixel_count).enumerate() {
        let base = i * 4;
        if pixel == 0 {
            if preserve_black {
                rgba[base + 3] = 255;
            }
            continue;
        }
        let color = rgb16_produce_color(pixel, pixel_format);
        rgba[base] = color.r;
        rgba[base + 1] = color.g;
        rgba[base + 2] = color.b;
        rgba[base + 3] = 255;
    }
    rgba
}

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

/// Render a frame into a (possibly larger) RGBA rectangle.
///
/// This is the canonical streaming-model frame decoder. Compatibility
/// wrappers below delegate here so every caller shares one decoding path.
pub fn render_frame_to_rgba_with_options<R: Read + Seek>(
    reader: &mut BufReader<R>,
    frame: &ImageInfo,
    rect_w: u32,
    rect_h: u32,
    offset_x: u32,
    offset_y: u32,
    options: SpriteRenderOptions,
) -> Result<RgbaImage> {
    if frame.bytes_per_pixel != 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "cannot render one-byte sprite pixels without a palette",
        ));
    }

    let pixel_count = (frame.width.unsigned_abs() * frame.height.unsigned_abs()) as usize;
    reader.seek(SeekFrom::Start(frame.image_start_position))?;
    let pixels = (0..pixel_count)
        .map(|_| reader.read_u16::<LittleEndian>())
        .collect::<Result<Vec<_>>>()?;
    let rgba = pixels_to_rgba(
        pixels.into_iter(),
        pixel_count,
        frame.pixel_format,
        options.preserve_black,
    );

    let mut imgbuf = RgbaImage::new(rect_w, rect_h);
    let frame_width = frame.width.unsigned_abs();
    for (pixel_idx, pixel) in rgba.chunks_exact(4).enumerate() {
        let x = (pixel_idx as u32 % frame_width) + offset_x;
        let y = (pixel_idx as u32 / frame_width) + offset_y;
        // Malformed rect/offset combinations must not panic during extraction.
        if x >= rect_w || y >= rect_h {
            continue;
        }
        imgbuf.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]));
    }
    Ok(imgbuf)
}

/// Compatibility wrapper: decode with default options (transparent black,
/// no origin offsets). New code should pass [`SpriteRenderOptions`] explicitly.
pub fn render_frame_to_rgba<R: Read + Seek>(
    reader: &mut BufReader<R>,
    frame: &ImageInfo,
    rect_w: u32,
    rect_h: u32,
    offset_x: u32,
    offset_y: u32,
) -> Result<RgbaImage> {
    render_frame_to_rgba_with_options(
        reader,
        frame,
        rect_w,
        rect_h,
        offset_x,
        offset_y,
        SpriteRenderOptions::default(),
    )
}

/// Compatibility wrapper: decode with `0x0000` kept as opaque black.
pub fn render_frame_to_rgba_with_black<R: Read + Seek>(
    reader: &mut BufReader<R>,
    frame: &ImageInfo,
    rect_w: u32,
    rect_h: u32,
    offset_x: u32,
    offset_y: u32,
    preserve_black: bool,
) -> Result<RgbaImage> {
    render_frame_to_rgba_with_options(
        reader,
        frame,
        rect_w,
        rect_h,
        offset_x,
        offset_y,
        SpriteRenderOptions {
            preserve_black,
            ..SpriteRenderOptions::default()
        },
    )
}

// ===========================================================================
// Sequence iteration helper
// ===========================================================================

fn for_each_sequence<F>(file_path: &Path, mut f: F) -> Result<()>
where
    F: FnMut(&mut BufReader<File>, &SequenceInfo, usize) -> Result<()>,
{
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let infos = read_sprite_sequence_infos(&mut reader)?;

    for (seq_index, info) in infos.iter().enumerate() {
        f(&mut reader, info, seq_index)?;
    }

    Ok(())
}

// ===========================================================================
// CLI commands (file extraction)
// ===========================================================================

/// Export every sequence as a horizontal animation atlas using explicit
/// [`SpriteRenderOptions`].
pub fn animation_with_options(file_path: &Path, options: SpriteRenderOptions) -> Result<()> {
    for_each_sequence(file_path, |reader, info, seq_idx| {
        save_sequence_anim_with_options(reader, &info.frame_infos, seq_idx as i32, options)
    })?;
    println!("Finished");
    Ok(())
}

/// Compatibility wrapper: export atlases with the given origin-offset policy
/// and transparent `0x0000` pixels.
pub fn animation(file_path: &Path, apply_origin_offsets: bool) -> Result<()> {
    animation_with_options(
        file_path,
        SpriteRenderOptions {
            apply_origin_offsets,
            ..SpriteRenderOptions::default()
        },
    )
}

/// Compatibility wrapper: export atlases with `0x0000` optionally kept as
/// opaque black.
pub fn animation_with_black(
    file_path: &Path,
    apply_origin_offsets: bool,
    preserve_black: bool,
) -> Result<()> {
    animation_with_options(
        file_path,
        SpriteRenderOptions {
            preserve_black,
            apply_origin_offsets,
        },
    )
}

/// Export individual frames using explicit [`SpriteRenderOptions`].
pub fn extract_with_options(
    file_path: &Path,
    out_file_prefix: String,
    options: SpriteRenderOptions,
) -> Result<()> {
    for_each_sequence(file_path, |reader, info, seq_idx| {
        save_sequence_with_options(
            reader,
            &info.frame_infos,
            seq_idx as i32,
            &out_file_prefix,
            options,
        )
    })?;
    println!("Finished");
    Ok(())
}

/// Compatibility wrapper: export individual frames with default options.
pub fn extract(file_path: &Path, out_file_prefix: String) -> Result<()> {
    extract_with_options(file_path, out_file_prefix, SpriteRenderOptions::default())
}

/// Compatibility wrapper: export individual frames with `0x0000` optionally
/// kept as opaque black.
pub fn extract_with_black(
    file_path: &Path,
    out_file_prefix: String,
    preserve_black: bool,
) -> Result<()> {
    extract_with_options(
        file_path,
        out_file_prefix,
        SpriteRenderOptions {
            preserve_black,
            ..SpriteRenderOptions::default()
        },
    )
}

/// Save one animation atlas using explicit [`SpriteRenderOptions`].
pub fn save_sequence_anim_with_options(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    options: SpriteRenderOptions,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    let (rect_x, rect_y, rect_w, rect_h) = compute_rect(frames);
    let rect_w = rect_w.unsigned_abs();
    let rect_h = rect_h.unsigned_abs();

    println!("x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    let imgbuf =
        render_sequence_animation(reader, frames, rect_x, rect_y, rect_w, rect_h, options)?;

    imgbuf
        .save(format!("image_{:?}.png", sequence_counter))
        .unwrap();

    Ok(())
}

/// Compatibility wrapper: save an atlas with the given origin-offset policy
/// and transparent `0x0000` pixels.
pub fn save_sequence_anim(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    apply_origin_offsets: bool,
) -> Result<()> {
    save_sequence_anim_with_options(
        reader,
        frames,
        sequence_counter,
        SpriteRenderOptions {
            apply_origin_offsets,
            ..SpriteRenderOptions::default()
        },
    )
}

/// Compatibility wrapper: save an atlas with `0x0000` optionally kept as
/// opaque black.
pub fn save_sequence_anim_with_black(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    apply_origin_offsets: bool,
    preserve_black: bool,
) -> Result<()> {
    save_sequence_anim_with_options(
        reader,
        frames,
        sequence_counter,
        SpriteRenderOptions {
            preserve_black,
            apply_origin_offsets,
        },
    )
}

/// Renders every frame of a sequence into a horizontal animation atlas.
///
/// Each atlas cell uses the sequence's shared bounding box, keeping the
/// frame anchor at the same position across the animation when
/// `options.apply_origin_offsets` is set.
fn render_sequence_animation<R: Read + Seek>(
    reader: &mut BufReader<R>,
    frames: &[ImageInfo],
    rect_x: i32,
    rect_y: i32,
    rect_w: u32,
    rect_h: u32,
    options: SpriteRenderOptions,
) -> Result<RgbaImage> {
    let atlas_w = rect_w * (frames.len() as u32);
    let mut imgbuf = RgbaImage::new(atlas_w, rect_h);

    for (i, frame) in frames.iter().enumerate() {
        let (offset_x, offset_y) = if options.apply_origin_offsets {
            compute_frame_offset(frames, i, rect_x, rect_y)
        } else {
            (0, 0)
        };
        let frame_rgba = render_frame_to_rgba_with_options(
            reader, frame, rect_w, rect_h, offset_x, offset_y, options,
        )?;

        for (px, py, pixel) in frame_rgba.enumerate_pixels() {
            imgbuf.put_pixel(px + (i as u32 * rect_w), py, *pixel);
        }
    }

    Ok(imgbuf)
}

/// Save individual frame PNGs using explicit [`SpriteRenderOptions`].
///
/// Individual frames are always centered in the sequence rect, regardless of
/// `options.apply_origin_offsets` (that flag only applies to atlases).
pub fn save_sequence_with_options(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    out_file_prefix: &str,
    options: SpriteRenderOptions,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    let (rect_x, rect_y, rect_w, rect_h) = compute_rect(frames);
    let rect_w = rect_w.unsigned_abs();
    let rect_h = rect_h.unsigned_abs();

    println!("x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    for (i, frame) in frames.iter().enumerate() {
        let (offset_x, offset_y) = compute_frame_offset(frames, i, rect_x, rect_y);

        let frame_rgba = render_frame_to_rgba_with_options(
            reader, frame, rect_w, rect_h, offset_x, offset_y, options,
        )?;

        let outfile = format!("./{}_{:?}-{:?}.png", out_file_prefix, sequence_counter, i);
        frame_rgba.save(outfile).unwrap();
    }

    Ok(())
}

/// Compatibility wrapper: save individual frames with default options.
pub fn save_sequence(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    out_file_prefix: &str,
) -> Result<()> {
    save_sequence_with_options(
        reader,
        frames,
        sequence_counter,
        out_file_prefix,
        SpriteRenderOptions::default(),
    )
}

/// Compatibility wrapper: save individual frames with `0x0000` optionally
/// kept as opaque black.
pub fn save_sequence_with_black(
    reader: &mut BufReader<File>,
    frames: &[ImageInfo],
    sequence_counter: i32,
    out_file_prefix: &str,
    preserve_black: bool,
) -> Result<()> {
    save_sequence_with_options(
        reader,
        frames,
        sequence_counter,
        out_file_prefix,
        SpriteRenderOptions {
            preserve_black,
            ..SpriteRenderOptions::default()
        },
    )
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

        let frame_rgba = render_frame_to_rgba_with_options(
            reader,
            frame,
            rect_w,
            rect_h,
            offset_x,
            offset_y,
            SpriteRenderOptions::default(),
        )?;

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
    let mut reader = BufReader::new(file);
    let infos = read_sprite_sequence_infos(&mut reader)?;
    if let Some(info) = infos.get(sequence_idx) {
        return get_sequence_frames_as_pngs(&mut reader, info);
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
    let infos = read_sprite_sequence_infos(&mut reader)?;

    let mut sequences = Vec::with_capacity(infos.len());
    let mut total_frames = 0;
    for (seq_index, info) in infos.iter().enumerate() {
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
                pixel_format: f.pixel_format,
            })
            .collect();

        sequences.push(SequenceInfoJson {
            sequence_index: seq_index,
            frame_count,
            frames,
        });
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

/// Editor-facing in-memory representation of a `.spr` archive.
///
/// This is a conversion layer over the authoritative streaming decoder, not a
/// second decoder: [`parse_sprite_bytes`] calls
/// [`read_sprite_sequence_infos`] and copies the decoded frames into owned
/// buffers. Per-frame unknown bytes and raw pixels are preserved, but the
/// outer 268-byte header is synthesized (original archives do not store one
/// in this view) and empty groups are dropped. Consequently
/// [`write_sprite_to_path`] emits a normalized one-record archive; unmodified
/// frames round-trip bit-exactly, while outer header/group partitioning does
/// not survive a write-back.
#[derive(Debug, Clone)]
pub struct SpriteFile {
    /// 268-byte header describing the normalized archive emitted by
    /// [`write_sprite_to_path`]; synthesized on parse, not copied from disk.
    pub header: [u8; 268],
    /// All animation sequences in the file.
    pub sequences: Vec<SpriteSequence>,
}

/// One non-empty frame group, exposed as a sequence in the editor view.
#[derive(Debug, Clone)]
pub struct SpriteSequence {
    /// Legacy Pattern-B marker. Always `false` for archives parsed through
    /// [`parse_sprite_bytes`], which flattens groups into sequences.
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
    /// Channel packing used by `raw_pixels`.
    pub pixel_format: SpritePixelFormat,
    /// Raw RGB555 or RGB565 pixel data, width×height×2 bytes, little-endian.
    /// Stored in original form so unmodified frames round-trip bit-exact.
    pub raw_pixels: Vec<u8>,
}

impl SpriteFrameData {
    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }

    /// Decode raw 16-bit pixels with explicit [`SpriteRenderOptions`].
    ///
    /// Uses the same pixel decoder as the streaming model, so both views
    /// produce byte-identical RGBA for the same frame bytes.
    pub fn decode_to_rgba_with_options(&self, options: SpriteRenderOptions) -> Vec<u8> {
        let count = self.pixel_count();
        let pixels = self
            .raw_pixels
            .chunks_exact(2)
            .take(count)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
        pixels_to_rgba(pixels, count, self.pixel_format, options.preserve_black)
    }

    /// Compatibility wrapper: decode with default options (transparent black).
    pub fn decode_to_rgba(&self) -> Vec<u8> {
        self.decode_to_rgba_with_options(SpriteRenderOptions::default())
    }

    /// Compatibility wrapper: decode with `0x0000` optionally kept as opaque
    /// black.
    pub fn decode_to_rgba_with_black(&self, preserve_black: bool) -> Vec<u8> {
        self.decode_to_rgba_with_options(SpriteRenderOptions {
            preserve_black,
            ..SpriteRenderOptions::default()
        })
    }

    /// Encode RGBA bytes (width×height×4) into this frame's 16-bit packing,
    /// replacing pixel data.
    pub fn encode_from_rgba(&mut self, rgba: &[u8]) {
        let count = self.pixel_count();
        let mut raw = vec![0u8; count * 2];
        for i in 0..count.min(rgba.len() / 4) {
            let rbase = i * 4;
            let encoded = rgba_to_rgb16_bytes(
                rgba[rbase],
                rgba[rbase + 1],
                rgba[rbase + 2],
                rgba[rbase + 3],
                self.pixel_format,
            );
            let base = i * 2;
            raw[base] = encoded[0];
            raw[base + 1] = encoded[1];
        }
        self.raw_pixels = raw;
    }
}

/// Parse a `.spr` file into the in-memory [`SpriteFile`] representation.
///
/// Reads the file into memory and delegates to [`parse_sprite_bytes`].
pub fn read_sprite_file(path: &Path) -> Result<SpriteFile> {
    let bytes = std::fs::read(path)?;
    parse_sprite_bytes(&bytes)
}

/// Parse a `.spr` archive for the editor-facing flattened group view.
///
/// `SpriteSequence` predates the discovered outer sequence/group structure;
/// it therefore continues to expose each non-empty group as a sequence.
pub fn parse_sprite_bytes(bytes: &[u8]) -> Result<SpriteFile> {
    if bytes.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "sprite archive is smaller than its sequence count",
        ));
    }

    let mut reader = BufReader::new(Cursor::new(bytes));
    let infos = read_sprite_sequence_infos(&mut reader)?;
    // SpriteFile is the editor's flattened group view. Its header describes
    // the normalized one-record archive emitted by write_sprite_to_path.
    let mut header = [0_u8; 268];
    header[..4].copy_from_slice(&1_i32.to_le_bytes());

    let mut sequences = Vec::with_capacity(infos.len());
    for info in infos {
        let mut frames = Vec::with_capacity(info.frame_infos.len());
        for frame in info.frame_infos {
            if frame.bytes_per_pixel != 2 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "editor support for one-byte sprite groups requires a palette",
                ));
            }
            let metadata_start = usize::try_from(frame.image_start_position)
                .ok()
                .and_then(|offset| offset.checked_sub(44))
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid frame offset")
                })?;
            let pixel_start = usize::try_from(frame.image_start_position).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid frame offset")
            })?;
            let pixel_end = pixel_start
                .checked_add(usize::try_from(frame.size_bytes).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid frame size")
                })?)
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large")
                })?;
            if pixel_end > bytes.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "truncated frame data",
                ));
            }

            let mut unknown = [0_u8; 24];
            unknown.copy_from_slice(&bytes[metadata_start..metadata_start + 24]);
            frames.push(SpriteFrameData {
                unknown,
                origin_x: frame.origin_x,
                origin_y: frame.origin_y,
                width: frame.width,
                height: frame.height,
                pixel_format: frame.pixel_format,
                raw_pixels: bytes[pixel_start..pixel_end].to_vec(),
            });
        }
        sequences.push(SpriteSequence {
            has_stamp: false,
            frames,
        });
    }

    Ok(SpriteFile { header, sequences })
}

/// Write the in-memory [`SpriteFile`] back to a .spr binary file.
///
/// The flattened editor view is emitted as a normalized counted archive.
/// Per-frame metadata and unmodified raw pixels are preserved, but outer
/// sequence headers, empty groups, and group partitioning are not.
pub fn write_sprite_to_path(path: &Path, sprite: &SpriteFile) -> Result<()> {
    let mut buf = Vec::with_capacity(4096);
    // The editor exposes groups as flat sequences. Serialize that view as one
    // valid outer sequence record rather than emitting the obsolete heuristic
    // format. The 4-byte id and 260-byte name remain zeroed.
    buf.extend_from_slice(&1_i32.to_le_bytes());
    buf.extend_from_slice(&[0_u8; 264]);
    buf.extend_from_slice(&(sprite.sequences.len() as i32).to_le_bytes());

    for seq in &sprite.sequences {
        let frame_count = seq.frames.len() as i32;
        // Group header: normal 16-bit variant, frame count, reserved word.
        buf.extend_from_slice(&0_i32.to_le_bytes());
        buf.extend_from_slice(&frame_count.to_le_bytes());
        buf.extend_from_slice(&0_i32.to_le_bytes());

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

/// Encode a single RGBA pixel as two RGB555 bytes (little-endian).
///
/// Pixels with alpha < 128 are encoded as `0x0000` (transparent).
pub fn rgba_to_rgb555_bytes(r: u8, g: u8, b: u8, a: u8) -> [u8; 2] {
    if a < 128 {
        return [0, 0];
    }
    let r5 = (r as u16 >> 3) & 0x1F;
    let g5 = (g as u16 >> 3) & 0x1F;
    let b5 = (b as u16 >> 3) & 0x1F;
    let pixel = (r5 << 10) | (g5 << 5) | b5;
    pixel.to_le_bytes()
}

/// Encode an RGBA pixel using the requested 16-bit sprite packing.
pub fn rgba_to_rgb16_bytes(r: u8, g: u8, b: u8, a: u8, pixel_format: SpritePixelFormat) -> [u8; 2] {
    match pixel_format {
        SpritePixelFormat::Rgb555 => rgba_to_rgb555_bytes(r, g, b, a),
        SpritePixelFormat::Rgb565 => rgba_to_rgb565_bytes(r, g, b, a),
    }
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
    fn rgba_to_rgb555_encodes_primary_channels() {
        assert_eq!(
            u16::from_le_bytes(rgba_to_rgb555_bytes(255, 0, 0, 255)),
            0x7C00
        );
        assert_eq!(
            u16::from_le_bytes(rgba_to_rgb555_bytes(0, 255, 0, 255)),
            0x03E0
        );
        assert_eq!(
            u16::from_le_bytes(rgba_to_rgb555_bytes(0, 0, 255, 255)),
            0x001F
        );
    }

    #[test]
    fn rgb555_decodes_five_bit_green() {
        let color = rgb16_produce_color(0x03E0, SpritePixelFormat::Rgb555);
        assert_eq!((color.r, color.g, color.b), (0, 255, 0));
    }

    // ── Synthetic archive fixtures (no user files) ────────────────────

    #[derive(Clone)]
    struct TestFrame {
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
        /// 16-bit pixels in row-major order; length must be width × height.
        pixels: Vec<u16>,
    }

    fn solid_frame(origin_x: i32, origin_y: i32, width: i32, height: i32, pixel: u16) -> TestFrame {
        TestFrame {
            origin_x,
            origin_y,
            width,
            height,
            pixels: vec![pixel; (width * height) as usize],
        }
    }

    /// Build one outer sequence record containing the given frame groups,
    /// using the byte layout read by `read_sprite_sequence_infos`.
    fn build_archive(groups: &[Vec<TestFrame>]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&1_i32.to_le_bytes()); // outer sequence count
        buf.extend_from_slice(&0_i32.to_le_bytes()); // sequence id
        buf.extend_from_slice(&[0_u8; 260]); // sequence name
        buf.extend_from_slice(&(groups.len() as i32).to_le_bytes());
        for group in groups {
            // Group header: 16-bit variant, frame count, reserved word.
            for word in [0_i32, group.len() as i32, 0] {
                buf.extend_from_slice(&word.to_le_bytes());
            }
            for frame in group {
                buf.extend_from_slice(&[0xA5_u8; 24]); // per-frame unknown bytes
                for value in [frame.origin_x, frame.origin_y, frame.width, frame.height] {
                    buf.extend_from_slice(&value.to_le_bytes());
                }
                buf.extend_from_slice(&(frame.pixels.len() as u32).to_le_bytes());
                for pixel in &frame.pixels {
                    buf.extend_from_slice(&pixel.to_le_bytes());
                }
            }
        }
        buf
    }

    fn temp_sprite_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("dispel-sprite-{}-{name}.spr", std::process::id()))
    }

    // ── Boundary: editor-model write → reparse fidelity ───────────────

    #[test]
    fn test_editor_model_write_reparse_fidelity() {
        // Rgb565 (bit 15 set) and Rgb555 (bit 15 clear) archives must both
        // survive a write → reparse through the authoritative decoder.
        for (label, pixel, format) in [
            ("rgb565", 0xF800_u16, SpritePixelFormat::Rgb565),
            ("rgb555", 0x03E0_u16, SpritePixelFormat::Rgb555),
        ] {
            let bytes = build_archive(&[
                vec![solid_frame(1, 2, 2, 2, pixel)],
                vec![solid_frame(0, 0, 1, 1, 0)], // includes a transparent frame
            ]);
            let original = parse_sprite_bytes(&bytes).expect("parse synthetic archive");
            assert_eq!(original.sequences.len(), 2, "{label} group count");
            for seq in &original.sequences {
                for frame in &seq.frames {
                    assert_eq!(frame.pixel_format, format, "{label} detected format");
                }
            }

            let path = temp_sprite_path(&format!("write-reparse-{label}"));
            write_sprite_to_path(&path, &original).expect("write normalized archive");
            let rewritten = std::fs::read(&path).expect("read normalized archive");
            let reloaded = parse_sprite_bytes(&rewritten).expect("reparse normalized archive");
            std::fs::remove_file(&path).expect("remove temp sprite");

            assert_eq!(reloaded.header, original.header, "{label} header");
            assert_eq!(
                reloaded.sequences.len(),
                original.sequences.len(),
                "{label} sequence count"
            );
            for (si, (a, b)) in original
                .sequences
                .iter()
                .zip(reloaded.sequences.iter())
                .enumerate()
            {
                assert_eq!(a.has_stamp, b.has_stamp, "{label} seq {si} stamp");
                assert_eq!(a.frames.len(), b.frames.len(), "{label} seq {si} frames");
                for (fi, (af, bf)) in a.frames.iter().zip(b.frames.iter()).enumerate() {
                    let where_ = format!("{label} seq {si} frame {fi}");
                    assert_eq!(af.unknown, bf.unknown, "{where_} unknown");
                    assert_eq!(af.origin_x, bf.origin_x, "{where_} origin_x");
                    assert_eq!(af.origin_y, bf.origin_y, "{where_} origin_y");
                    assert_eq!(af.width, bf.width, "{where_} width");
                    assert_eq!(af.height, bf.height, "{where_} height");
                    assert_eq!(af.pixel_format, bf.pixel_format, "{where_} format");
                    assert_eq!(af.raw_pixels, bf.raw_pixels, "{where_} pixels");
                }
            }
        }
    }

    // ── Boundary: both models decode the same bytes identically ──────

    #[test]
    fn test_both_models_decode_identical_rgba() {
        for (label, pixel, format) in [
            ("rgb565", 0xF800_u16, SpritePixelFormat::Rgb565),
            ("rgb555", 0x03E0_u16, SpritePixelFormat::Rgb555),
        ] {
            for options in [
                SpriteRenderOptions::default(),
                SpriteRenderOptions::with_black(),
            ] {
                // One colored pixel followed by a raw-0x0000 pixel.
                let bytes = build_archive(&[vec![TestFrame {
                    origin_x: 1,
                    origin_y: 1,
                    width: 2,
                    height: 1,
                    pixels: vec![pixel, 0],
                }]]);

                let mut reader = BufReader::new(Cursor::new(&bytes));
                let infos = read_sprite_sequence_infos(&mut reader).expect("streaming parse");
                assert_eq!(infos.len(), 1, "{label} streaming sequence count");
                let frame = &infos[0].frame_infos[0];
                let streaming =
                    render_frame_to_rgba_with_options(&mut reader, frame, 2, 1, 0, 0, options)
                        .expect("streaming render");

                let sprite = parse_sprite_bytes(&bytes).expect("editor parse");
                let editor = sprite.sequences[0].frames[0].decode_to_rgba_with_options(options);

                assert_eq!(
                    streaming.as_raw(),
                    editor.as_slice(),
                    "{label} RGBA mismatch"
                );
                let expected_color = rgb16_produce_color(pixel, format);
                assert_eq!(
                    &editor[..4],
                    &[expected_color.r, expected_color.g, expected_color.b, 255],
                    "{label} colored pixel"
                );
                let expected_zero = if options.preserve_black {
                    [0, 0, 0, 255]
                } else {
                    [0, 0, 0, 0]
                };
                assert_eq!(&editor[4..], &expected_zero, "{label} zero pixel");
            }
        }
    }

    // ── Boundary: black-transparency options and wrappers ────────────

    #[test]
    fn test_preserve_black_option_behavior_in_both_models() {
        let bytes = build_archive(&[vec![TestFrame {
            origin_x: 0,
            origin_y: 0,
            width: 2,
            height: 1,
            pixels: vec![0, 0xF800],
        }]]);

        let mut reader = BufReader::new(Cursor::new(&bytes));
        let infos = read_sprite_sequence_infos(&mut reader).expect("streaming parse");
        let frame = &infos[0].frame_infos[0];
        let sprite = parse_sprite_bytes(&bytes).expect("editor parse");
        let editor_frame = &sprite.sequences[0].frames[0];

        let transparent = render_frame_to_rgba_with_options(
            &mut reader,
            frame,
            2,
            1,
            0,
            0,
            SpriteRenderOptions::default(),
        )
        .expect("transparent render");
        let opaque = render_frame_to_rgba_with_options(
            &mut reader,
            frame,
            2,
            1,
            0,
            0,
            SpriteRenderOptions::with_black(),
        )
        .expect("opaque render");

        assert_eq!(transparent.get_pixel(0, 0).0, [0, 0, 0, 0]);
        assert_eq!(opaque.get_pixel(0, 0).0, [0, 0, 0, 255]);
        assert_eq!(transparent.get_pixel(1, 0).0, [255, 0, 0, 255]);
        assert_eq!(opaque.get_pixel(1, 0).0, [255, 0, 0, 255]);

        assert_eq!(
            editor_frame.decode_to_rgba(),
            transparent.as_raw().as_slice()
        );
        assert_eq!(
            editor_frame.decode_to_rgba_with_black(true),
            opaque.as_raw().as_slice()
        );
        // Compatibility wrappers must delegate to the same canonical path.
        let wrapper_transparent =
            render_frame_to_rgba(&mut reader, frame, 2, 1, 0, 0).expect("wrapper render");
        let wrapper_opaque = render_frame_to_rgba_with_black(&mut reader, frame, 2, 1, 0, 0, true)
            .expect("wrapper render with black");
        assert_eq!(wrapper_transparent.as_raw(), transparent.as_raw());
        assert_eq!(wrapper_opaque.as_raw(), opaque.as_raw());
    }

    // ── Boundary: origin-offset option ────────────────────────────────

    #[test]
    fn test_apply_origin_offsets_option_behavior() {
        // Frame A has origin (0,0), frame B origin (1,1); the shared rect is
        // (1,1,3,3), so only frame A shifts when offsets are applied.
        let bytes = build_archive(&[vec![
            solid_frame(0, 0, 2, 2, 0xF800), // red
            solid_frame(1, 1, 2, 2, 0x07E0), // green
        ]]);
        let mut reader = BufReader::new(Cursor::new(&bytes));
        let infos = read_sprite_sequence_infos(&mut reader).expect("streaming parse");
        let frames = &infos[0].frame_infos;
        let (rect_x, rect_y, rect_w, rect_h) = compute_rect(frames);
        assert_eq!((rect_x, rect_y, rect_w, rect_h), (1, 1, 3, 3));
        let (rect_w, rect_h) = (rect_w.unsigned_abs(), rect_h.unsigned_abs());

        let plain = render_sequence_animation(
            &mut reader,
            frames,
            rect_x,
            rect_y,
            rect_w,
            rect_h,
            SpriteRenderOptions::default(),
        )
        .expect("plain atlas");
        let aligned = render_sequence_animation(
            &mut reader,
            frames,
            rect_x,
            rect_y,
            rect_w,
            rect_h,
            SpriteRenderOptions {
                apply_origin_offsets: true,
                ..Default::default()
            },
        )
        .expect("aligned atlas");

        assert_eq!(plain.dimensions(), (6, 3));
        assert_eq!(aligned.dimensions(), (6, 3));
        assert_eq!(plain.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(aligned.get_pixel(0, 0).0, [0, 0, 0, 0]);
        assert_eq!(aligned.get_pixel(1, 1).0, [255, 0, 0, 255]);
        // Frame B's origin equals the rect origin: it stays put in both cases.
        assert_eq!(plain.get_pixel(3, 0).0, [0, 255, 0, 255]);
        assert_eq!(aligned.get_pixel(3, 0).0, [0, 255, 0, 255]);
    }

    #[test]
    fn rgba_to_rgb565_transparent_is_zero() {
        let bytes = rgba_to_rgb565_bytes(255, 0, 0, 0);
        assert_eq!(bytes, [0, 0]);
    }

    #[test]
    fn rgba_to_rgb565_decode_round_trip() {
        // Encode a known color → decode back → verify the 5/6/5 fields survive.
        // Decode expands to the full 0..=255 range (16 → 132, not 16 << 3).
        let encoded = rgba_to_rgb565_bytes(128, 64, 192, 255);
        let pixel = u16::from_le_bytes(encoded);
        let color = rgb16_565_produce_color(pixel);
        assert_eq!(color.r, 132); // r5 = 16
        assert_eq!(color.g, 65); // g6 = 16
        assert_eq!(color.b, 197); // b5 = 24
        assert_eq!(u16::from(color.r) >> 3, 16);
        assert_eq!(u16::from(color.g) >> 2, 16);
        assert_eq!(u16::from(color.b) >> 3, 24);
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
            pixel_format: SpritePixelFormat::Rgb565,
            raw_pixels: vec![0u8; 4 * 4 * 2],
        };
        frame.encode_from_rgba(&pixels);
        assert_eq!(frame.raw_pixels.len(), 32, "4×4 pixels × 2 bytes");

        // Decode back
        let decoded = frame.decode_to_rgba();
        assert_eq!(decoded.len(), 64, "4×4 pixels × 4 bytes RGBA");
        // R=255 → r5=31 → 255; G=128 → g6=32 → 130; B=64 → b5=8 → 66.
        assert_eq!(decoded[0], 255);
        assert_eq!(decoded[1], 130);
        assert_eq!(decoded[2], 66);
        assert_eq!(decoded[3], 255);
    }

    // Write → reparse fidelity is covered by `test_editor_model_write_reparse_fidelity`
    // with synthetic archives; no user-supplied fixture files are required.
    // Golden RGB565/RGB555 values and channel invariants live in
    // `crate::decode::tests` and the color tests above.
}
