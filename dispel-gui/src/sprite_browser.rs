use dispel_core::sprite::get_sequence_frames_as_pngs;
use std::fs;
use std::io::{BufReader, Seek, SeekFrom};
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
                                    if let Ok(info) =
                                        dispel_core::sprite::get_sequence_info(&mut reader)
                                    {
                                        if seq_counter == self.selected_sequence {
                                            if let Ok(pngs) =
                                                get_sequence_frames_as_pngs(&mut reader, &info)
                                            {
                                                for (frame_idx, png_buf) in
                                                    pngs.into_iter().enumerate()
                                                {
                                                    self.frames.push(SpriteFrame {
                                                        sequence_idx: self.selected_sequence,
                                                        frame_idx,
                                                        image:
                                                            iced::widget::image::Handle::from_bytes(
                                                                png_buf,
                                                            ),
                                                    });
                                                }
                                            }
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
}
