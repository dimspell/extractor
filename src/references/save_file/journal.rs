use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};

/// Journal data from a save file (42-byte header + 3 sections × 100 entries).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalData {
    /// Journal UI state and per-section entry counts.
    pub header: JournalHeader,
    /// Main quest entries (100 × 37 bytes)
    pub main: Vec<JournalEntry>,
    /// Side quest entries (100 × 37 bytes)
    pub side: Vec<JournalEntry>,
    /// Trading offer entries (100 × 37 bytes)
    pub trade: Vec<JournalEntry>,
}

/// The 42-byte journal header before the three 100-entry journal sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct JournalHeader {
    /// Runtime flag controlled by the journal UI; the game meaning is unknown.
    pub runtime_unknown_flag: u8,
    /// Journal section selected by the UI (zero-based).
    pub selected_section: u8,
    /// Per-section, per-visible-row selection flags (three sections × ten rows).
    pub visible_entry_selection_flags: [u8; 30],
    /// Journal section currently being displayed (zero-based).
    pub active_section: u8,
    /// Page offset in each journal section.
    pub section_page_offsets: [u8; 3],
    /// Selected entry offset in each journal section.
    pub section_selected_entry_offsets: [u8; 3],
    /// Number of active entries in main, side, and trade sections respectively.
    pub section_entry_counts: [u8; 3],
}

/// Journal entry (37 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct JournalEntry {
    /// Zero-based slot within this journal section.
    pub entry_index: u8,
    /// Title copied from the corresponding `Quest.scr` entry.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 24))]
    pub quest_title: String,
    /// Eight bytes of quest-specific state. The game does not access them in the
    /// journal code path, so their individual meanings are not yet known.
    pub quest_state: [u8; 8],
    /// ID of the quest from `ExtraInGame/Quest.scr`.
    pub quest_id: u8,
    /// Quest ID recorded when this quest advances to its first follow-up stage (multi-stage quest).
    pub progress_quest_id_1: u8,
    /// Quest ID recorded when this quest advances to its second follow-up stage (multi-stage quest).
    pub progress_quest_id_2: u8,
    /// Set when the game marks this journal entry as completed.
    pub is_completed: u8,
}
