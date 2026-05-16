use iced::color;

/// A user-defined annotated byte range in the hex editor.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    /// Index into the 16-color palette (0..15).
    pub color_idx: u8,
}

impl Pattern {
    pub fn new(id: usize, start: u64, end: u64, color_idx: u8) -> Self {
        Self {
            id,
            start,
            end,
            color_idx,
        }
    }

    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }

    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
}

/// Return the background colour for the given palette index (0..15).
pub fn pattern_bg(idx: u8) -> iced::Color {
    match idx % 16 {
        0 => color!(0x1a3a4f),
        1 => color!(0x4f2e1a),
        2 => color!(0x1a4f2e),
        3 => color!(0x3b1a4f),
        4 => color!(0x4f4a1a),
        5 => color!(0x2e1a4f),
        6 => color!(0x4f1a1a),
        7 => color!(0x1a3b3b),
        8 => color!(0x3b2e1a),
        9 => color!(0x2e4f1a),
        10 => color!(0x4f2e3b),
        11 => color!(0x1a4f4f),
        12 => color!(0x4f251a),
        13 => color!(0x1a3b25),
        14 => color!(0x3b3b1a),
        15 => color!(0x251a4f),
        _ => color!(0x1a3a4f),
    }
}

/// Return the foreground (text) colour for the given palette index (0..15).
pub fn pattern_fg(idx: u8) -> iced::Color {
    match idx % 16 {
        0 => color!(0x6ab0d0),
        1 => color!(0xd08a6a),
        2 => color!(0x6ad08a),
        3 => color!(0xa06ad0),
        4 => color!(0xd0cb6a),
        5 => color!(0x8a6ad0),
        6 => color!(0xd06a6a),
        7 => color!(0x6ad0d0),
        8 => color!(0xd0af6a),
        9 => color!(0x8ad06a),
        10 => color!(0xd06a9a),
        11 => color!(0x6ad0af),
        12 => color!(0xd0856a),
        13 => color!(0x6ad085),
        14 => color!(0xafd06a),
        15 => color!(0x856ad0),
        _ => color!(0x6ab0d0),
    }
}
