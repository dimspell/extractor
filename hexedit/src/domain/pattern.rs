use iced::widget::Id;
use serde::{Deserialize, Serialize};

use crate::ui::theme::DARK_THEME;

/// A user-defined annotated byte range in the hex editor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: usize,
    pub start: u64,
    pub end: u64,
    /// Index into the 16-color palette (0..15).
    pub color_idx: u8,
    /// If this pattern belongs to a repeated-pattern group, the group id.
    pub group_id: Option<usize>,
    /// Optional user note displayed to the right of the ASCII column.
    pub annotation: Option<String>,
}

impl Pattern {
    pub fn new(id: usize, start: u64, end: u64, color_idx: u8) -> Self {
        Self {
            id,
            start,
            end,
            color_idx,
            group_id: None,
            annotation: None,
        }
    }

    pub fn grouped(id: usize, start: u64, end: u64, color_idx: u8, group_id: usize) -> Self {
        Self {
            id,
            start,
            end,
            color_idx,
            group_id: Some(group_id),
            annotation: None,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedPatternGroup {
    pub id: usize,
    pub label: String,
    pub color_idx: u8,
}

impl RepeatedPatternGroup {
    pub fn new(id: usize, label: String, color_idx: u8) -> Self {
        Self {
            id,
            label,
            color_idx,
        }
    }
}

/// Return the background colour for the given palette index (0..15).
///
/// Delegates to [`DARK_THEME`] so the palette lives in a single place.
pub fn pattern_bg(idx: u8) -> iced::Color {
    DARK_THEME.pattern_bg_palette[idx as usize % 16]
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

/// Top-level JSON envelope for pattern export/import.
///
/// Versioned for forward compatibility — bump `VERSION` if the format changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExport {
    /// Schema version (currently 1).
    pub version: u32,
    pub groups: Vec<RepeatedPatternGroup>,
    pub patterns: Vec<Pattern>,
}

impl PatternExport {
    pub const VERSION: u32 = 1;
}

/// Return the foreground (text) colour for the given palette index (0..15).
///
/// Delegates to [`DARK_THEME`] so the palette lives in a single place.
pub fn pattern_fg(idx: u8) -> iced::Color {
    DARK_THEME.pattern_fg_palette[idx as usize % 16]
}
