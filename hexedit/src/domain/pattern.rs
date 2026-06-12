use iced::{color, widget::Id};

/// A user-defined annotated byte range in the hex editor.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    /// Index into the 16-color palette (0..15).
    pub color_idx: u8,
    /// If this pattern belongs to a repeated-pattern group, the group id.
    pub group_id: Option<usize>,
}

impl Pattern {
    pub fn new(id: usize, start: u64, end: u64, color_idx: u8) -> Self {
        Self {
            id,
            start,
            end,
            color_idx,
            group_id: None,
        }
    }

    pub fn grouped(id: usize, start: u64, end: u64, color_idx: u8, group_id: usize) -> Self {
        Self {
            id,
            start,
            end,
            color_idx,
            group_id: Some(group_id),
        }
    }

    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }

    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
}

/// A named group of repeated patterns (from "Add repeated pattern").
///
/// All patterns in the same group share a single colour so the user can
/// visually distinguish different repetition groups from each other and from
/// individual patterns.
#[derive(Debug, Clone)]
pub struct RepeatedPatternGroup {
    pub id: usize,
    pub label: String,
    pub color_idx: u8,
}

impl RepeatedPatternGroup {
    pub fn new(id: usize, label: String, color_idx: u8) -> Self {
        Self { id, label, color_idx }
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

/// Transient dialog state for creating a repeated (zebra-striped) pattern
/// from the current selection.
///
/// The user selects a block of bytes, right-clicks → "Add repeated pattern",
/// then enters a repeat count. The implementation creates alternating-colour
/// `Pattern` entries for each repetition.
#[derive(Debug, Clone)]
pub struct RepeatPatternDialog {
    pub block_start: u64,
    pub block_size: u64,
    /// User-entered name for this group of repeated patterns.
    pub label_draft: String,
    /// User-entered repeat count.
    pub draft: String,
    pub error: Option<String>,
}

impl RepeatPatternDialog {
    pub fn new(block_start: u64, block_size: u64) -> Self {
        Self {
            block_start,
            block_size,
            label_draft: String::new(),
            draft: String::new(),
            error: None,
        }
    }

    pub fn input_id() -> Id {
        Id::new("hex_repeat_pattern_input")
    }

    /// Parse the repeat count from the draft input.
    pub fn parse_repeat_count(&self) -> Result<u64, String> {
        let s = self.draft.trim();
        if s.is_empty() {
            return Err("Enter a repeat count".to_string());
        }
        let count: u64 = s.parse().map_err(|_| format!("not a valid number: {s}"))?;
        if count < 1 {
            return Err("Repeat count must be at least 1".to_string());
        }
        if count > 10_000 {
            return Err("Repeat count capped at 10 000".to_string());
        }
        Ok(count)
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
