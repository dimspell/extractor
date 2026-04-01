use dispel_core::sprite::{get_sequence_info, SequenceInfo};
use image::ImageEncoder;
use std::fs;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SpriteEntry {
    pub path: PathBuf,
    pub name: String,
    pub sequence_count: usize,
    pub frame_counts: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub sequence_idx: usize,
    pub frame_idx: usize,
    pub image: iced::widget::image::Handle,
}

#[derive(Debug, Clone, Default)]
pub struct SpriteBrowserState {
    pub sprites: Vec<SpriteEntry>,
    pub filtered_sprites: Vec<(usize, SpriteEntry)>,
    pub search_query: String,
    pub selected_sprite_idx: Option<usize>,
    pub selected_sequence: usize,
    pub selected_frame: usize,
    pub frames: Vec<SpriteFrame>,
    pub status_msg: String,
    pub is_loading: bool,
}

fn read_u16_le<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

impl SpriteBrowserState {
    pub fn filter_sprites(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_sprites = self
                .sprites
                .iter()
                .enumerate()
                .map(|(i, e)| (i, e.clone()))
                .collect();
        } else {
            self.filtered_sprites = self
                .sprites
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    let name = e.name.to_lowercase();
                    let mut score = 0;
                    let mut query_chars = query.chars().peekable();
                    let mut name_chars = name.chars();

                    while let Some(qc) = query_chars.next() {
                        let mut found = false;
                        while let Some(nc) = name_chars.next() {
                            if nc == qc {
                                score += 1;
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return false;
                        }
                    }
                    score > 0
                })
                .map(|(i, e)| (i, e.clone()))
                .collect();
        }
        if let Some(idx) = self.selected_sprite_idx {
            if !self.filtered_sprites.iter().any(|(orig, _)| *orig == idx) {
                self.selected_sprite_idx = None;
                self.frames.clear();
            }
        }
    }

    pub fn select_sprite(&mut self, orig_idx: usize) {
        self.selected_sprite_idx = Some(orig_idx);
        self.selected_sequence = 0;
        self.selected_frame = 0;
        self.load_frames();
    }

    pub fn select_sprite_filtered(&mut self, filtered_idx: usize) {
        if let Some((orig_idx, _)) = self.filtered_sprites.get(filtered_idx) {
            self.select_sprite(*orig_idx);
        }
    }

    pub fn select_sequence(&mut self, seq_idx: usize) {
        self.selected_sequence = seq_idx;
        self.selected_frame = 0;
        self.load_frames();
    }

    pub fn select_frame(&mut self, frame_idx: usize) {
        self.selected_frame = frame_idx;
    }

    fn load_frames(&mut self) {
        self.frames.clear();

        if let Some(idx) = self.selected_sprite_idx {
            if let Some(sprite) = self.sprites.get(idx) {
                if let Ok(file) = fs::File::open(&sprite.path) {
                    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
                    let mut reader = BufReader::new(file);

                    if reader.seek(SeekFrom::Start(268)).is_ok() {
                        let mut seq_counter = 0;
                        loop {
                            let pos = reader.stream_position().unwrap_or(0);
                            if pos >= file_len {
                                break;
                            }

                            if let Ok(valid) =
                                dispel_core::sprite::seek_next_sequence(&mut reader, pos, file_len)
                            {
                                if valid {
                                    if let Ok(info) = get_sequence_info(&mut reader) {
                                        if seq_counter == self.selected_sequence {
                                            self.load_sequence_frames(&mut reader, &info);
                                            break;
                                        }
                                        seq_counter += 1;
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn load_sequence_frames(&mut self, reader: &mut BufReader<fs::File>, info: &SequenceInfo) {
        let mut max_left = 1;
        let mut max_right = 1;
        let mut max_up = 1;
        let mut max_down = 1;

        for frame in &info.frame_infos {
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
        let rect_w = if info.frame_infos.len() == 1 {
            info.frame_infos[0].width
        } else {
            max_left + max_right
        }
        .unsigned_abs();
        let rect_h = if info.frame_infos.len() == 1 {
            info.frame_infos[0].height
        } else {
            max_up + max_down
        }
        .unsigned_abs();

        for (frame_idx, frame) in info.frame_infos.iter().enumerate() {
            let mut imgbuf = image::ImageBuffer::new(rect_w, rect_h);

            let offset_x = if info.frame_infos.len() == 1 {
                0
            } else {
                rect_x - frame.origin_x
            }
            .unsigned_abs();
            let offset_y = if info.frame_infos.len() == 1 {
                0
            } else {
                rect_y - frame.origin_y
            }
            .unsigned_abs();

            let frame_width: u32 = frame.width as u32;

            if reader
                .seek(SeekFrom::Start(frame.image_start_position))
                .is_ok()
            {
                for i in 0..(frame.width * frame.height) as u64 {
                    if let Ok(pixel) = read_u16_le(reader) {
                        if pixel == 0 {
                            continue;
                        }
                        let red_mask: u16 = 0xF800;
                        let green_mask: u16 = 0x7E0;
                        let blue_mask: u16 = 0x1F;

                        let red_value = (pixel & red_mask) >> 11;
                        let green_value = (pixel & green_mask) >> 5;
                        let blue_value = pixel & blue_mask;

                        let red: u8 = (red_value << 3) as u8;
                        let green: u8 = (green_value << 2) as u8;
                        let blue: u8 = (blue_value << 3) as u8;

                        let x: u32 = (i as u32 % frame_width) + offset_x;
                        let y: u32 = (i as u32 / frame_width) + offset_y;
                        imgbuf.put_pixel(x, y, image::Rgba([red, green, blue, 255]));
                    }
                }
            }

            let mut buf = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(Cursor::new(&mut buf));
            if encoder
                .write_image(
                    imgbuf.as_raw(),
                    rect_w,
                    rect_h,
                    image::ExtendedColorType::Rgba8,
                )
                .is_ok()
            {
                self.frames.push(SpriteFrame {
                    sequence_idx: self.selected_sequence,
                    frame_idx,
                    image: iced::widget::image::Handle::from_bytes(buf),
                });
            }
        }
    }
}
