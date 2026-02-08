use byteorder::{LittleEndian, ReadBytesExt};
use image::RgbaImage;
use std::io::{BufReader, Result, Seek, SeekFrom};
use std::{fs::File, path::Path};

pub fn animation(file_path: &Path) -> Result<()> {
    let file = File::open(file_path)?;

    let metadata = file.metadata()?;
    let file_len = metadata.len();

    let mut reader = BufReader::new(file);

    // Start from 268th byte
    reader.seek(SeekFrom::Start(268))?;

    let mut sequence_counter = 0;
    loop {
        let pos = reader.seek(SeekFrom::Current(0))?;
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
    frames: &Vec<ImageInfo>,
    sequence_counter: i32,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    // start of CalculateDimensions
    let mut max_left = 1;
    let mut max_right = 1;
    let mut max_up = 1;
    let mut max_down = 1;
    for i in 0..frames.len() {
        let left = frames[i].origin_x;
        let right = frames[i].width - frames[i].origin_x;
        let up = frames[i].origin_y;
        let down = frames[i].height - frames[i].origin_y;
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
    // end of CalculateDimensions

    let rect_w = if frames.len() == 1 {
        frames[0].width
    } else {
        max_left + max_right
    }
    .unsigned_abs();
    let rect_h = if frames.len() == 1 {
        frames[0].height
    } else {
        max_up + max_down
    }
    .unsigned_abs();

    println!("{max_left}, {max_right}, {max_up}, {max_down} -> x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    let mut imgbuf: RgbaImage = image::ImageBuffer::new(rect_w * (frames.len() as u32), rect_h);
    let mut offset_x = 0;
    let mut offset_y = 0;

    for i in 0..frames.len() {
        let frame = &frames[i];

        // let offset_x = if frames.len() == 1 {
        //     0
        // } else {
        //     rect_x - frame.origin_x
        // }.unsigned_abs();
        offset_y = if frames.len() == 1 {
            0
        } else {
            rect_y - frame.origin_y
        }
        .unsigned_abs();

        let frame_width: u32 = frame.width.try_into().unwrap();

        reader.seek(SeekFrom::Start(frame.image_start_position))?;
        for i in 0..(frame.width * frame.height).try_into().unwrap() {
            let pixel = reader.read_u16::<LittleEndian>()?;
            if pixel == 0 {
                continue;
            }
            let color = rgb16_565_produce_color(pixel);
            let x: u32 = (i as u32 % frame_width) + offset_x;
            let y: u32 = (i as u32 / frame_width) + offset_y;
            imgbuf.put_pixel(x, y, image::Rgba([color.r, color.g, color.b, 255]));
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
        let pos = reader.seek(SeekFrom::Current(0))?;
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
    frames: &Vec<ImageInfo>,
    sequence_counter: i32,
    out_file_prefix: &String,
) -> Result<()> {
    println!("Frames: {:?}, Sequence: {sequence_counter}", frames.len());

    // start of CalculateDimensions
    let mut max_left = 1;
    let mut max_right = 1;
    let mut max_up = 1;
    let mut max_down = 1;
    for i in 0..frames.len() {
        let left = frames[i].origin_x;
        let right = frames[i].width - frames[i].origin_x;
        let up = frames[i].origin_y;
        let down = frames[i].height - frames[i].origin_y;
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
    // end of CalculateDimensions

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

    println!("{max_left}, {max_right}, {max_up}, {max_down} -> x:{rect_x} y:{rect_y} w:{rect_w} h:{rect_h}");

    for i in 0..frames.len() {
        let frame = frames[i];

        // let mut bytes = vec![0; (frame.width * frame.height * 3).try_into().unwrap()];

        let w: u32 = rect_w.try_into().unwrap();
        let h: u32 = rect_h.try_into().unwrap();
        let mut imgbuf = image::ImageBuffer::new(w, h);

        let offset_x = if frames.len() == 1 {
            0
        } else {
            rect_x - frame.origin_x
        };
        let offset_y = if frames.len() == 1 {
            0
        } else {
            rect_y - frame.origin_y
        };
        let offset_x: u32 = offset_x.try_into().unwrap();
        let offset_y: u32 = offset_y.try_into().unwrap();

        let frame_width: u32 = frame.width.try_into().unwrap();

        reader.seek(SeekFrom::Start(frame.image_start_position))?;
        for i in 0..(frame.width * frame.height).try_into().unwrap() {
            let buffer = reader.read_u16::<LittleEndian>()?;
            let color = rgb16_565_produce_color(buffer);
            // bytes[i * 3] = color.r;
            // bytes[(i * 3) + 1] = color.g;
            // bytes[(i * 3) + 2] = color.b;

            let index: u32 = i.try_into().unwrap();
            let x: u32 = (index % frame_width) + offset_x;
            let y: u32 = (index / frame_width) + offset_y;
            imgbuf.put_pixel(x, y, image::Rgb([color.r, color.g, color.b]));
        }

        let outfile = format!("./{}_{:?}-{:?}.png", out_file_prefix, sequence_counter, i);
        // println!("{outfile}");
        imgbuf.save(outfile).unwrap();

        // image::save_buffer_with_format(
        //     format!("image_raw_{i}.png"),
        //     bytes.as_slice(),
        //     frame.width.try_into().unwrap(),
        //     frame.height.try_into().unwrap(),
        //     image::ColorType::Rgb8,
        //     image::ImageFormat::Png,
        // )
        // .unwrap();
    }

    Ok(())
}

#[derive(Clone, Copy)]
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
    if info.frame_count == 0 {
        // Metrics.Count(MetricFile.SpriteFileMetric, filename, "zeroFrame");
        // Metrics.Count(MetricFile.SpriteFileMetric, "zeroFrames");
    }
    let start_position = reader.seek(SeekFrom::Current(0))?;
    info.sequence_start_position = start_position;
    for i in 0..info.frame_count.try_into().unwrap() {
        let image_info = get_image_info(reader)?;
        info.frame_infos[i] = image_info;
        reader.seek(SeekFrom::Current(image_info.size_bytes))?;

        // catch (FrameInfoException)
        // {
        //     var oldFrames = info.FrameInfos;
        //     info.FrameInfos = new ImageInfo[i];
        //     for (int j = 0; j < info.FrameInfos.Length; j++)
        //     {
        //         info.FrameInfos[j] = oldFrames[j];
        //     }
        // }
    }
    info.sequence_end_position = reader.seek(SeekFrom::Current(0))?;
    reader.seek(SeekFrom::Start(start_position))?;

    Ok(info)
}

#[derive(Debug, Clone, Copy)]
pub struct ImageInfo {
    origin_x: i32,
    origin_y: i32,
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

    let pos: u64 = reader.seek(SeekFrom::Current(0))?;
    info.image_start_position = pos;

    if info.width < 1 || info.height < 1 {
        // throw new FrameInfoException();//fix for soulnet.spr missing one frame
        unimplemented!();
    }
    // println!("Info: {info:?}");
    Ok(info)
}

fn seek_next_sequence(reader: &mut BufReader<File>, start_pos: u64, file_len: u64) -> Result<bool> {
    let mut valid_sprite_seq = false;
    let mut number_of_skips = 0;

    while !valid_sprite_seq {
        let pos: u64 = reader.seek(SeekFrom::Current(0))?;
        // println!("Seek position: {pos} of {file_len}. Skips: {number_of_skips}");
        if pos + 60 >= file_len {
            // println!("break at skips: {:?}", number_of_skips);
            break;
        }

        let mut ints = [0; 15];
        for i in 0..15 {
            ints[i] = reader.read_i32::<LittleEndian>()?;
        }

        if (ints[0] == 0
            && ints[1] > 0
            && ints[1] < 255
            && ints[2] == 0
            && ints[11] > 0
            && ints[12] > 0
            && ints[11] * ints[12] == ints[13])
            || (ints[0] == 8
                && ints[1] == 0
                && ints[2] > 0
                && ints[2] < 255
                && ints[3] == 0
                && ints[12] > 0
                && ints[13] > 0
                && ints[12] * ints[13] == ints[14])
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
