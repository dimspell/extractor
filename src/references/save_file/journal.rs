use byteorder::{ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub(super) const JOURNAL_HEADER_SIZE: usize = 42;
pub(super) const JOURNAL_ENTRY_SIZE: usize = 37;
pub(super) const JOURNAL_ENTRIES_PER_SECTION: usize = 100;
const JOURNAL_SECTION_SIZE: usize = JOURNAL_ENTRY_SIZE * JOURNAL_ENTRIES_PER_SECTION;

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

impl JournalData {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut header_data = [0u8; JOURNAL_HEADER_SIZE];
        reader.read_exact(&mut header_data)?;
        let header = JournalHeader::parse(&header_data)?;

        Ok(Self {
            header,
            main: read_entries(reader)?,
            side: read_entries(reader)?,
            trade: read_entries(reader)?,
        })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.header.write(writer)?;
        write_entries(writer, &self.main)?;
        write_entries(writer, &self.side)?;
        write_entries(writer, &self.trade)
    }
}

fn read_entries<R: Read>(reader: &mut R) -> std::io::Result<Vec<JournalEntry>> {
    let mut data = vec![0u8; JOURNAL_SECTION_SIZE];
    reader.read_exact(&mut data)?;
    data.chunks_exact(JOURNAL_ENTRY_SIZE)
        .map(JournalEntry::parse)
        .collect()
}

fn write_entries<W: Write>(writer: &mut W, entries: &[JournalEntry]) -> std::io::Result<()> {
    for entry in entries {
        entry.write(writer)?;
    }
    Ok(())
}

/// The 42-byte journal header before the three 100-entry journal sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JournalHeader {
    /// Screen shown by the combined map and journal interface
    /// (`0`=journal, `1`=world map).
    pub is_world_map_open: u8,
    /// Selected world-map layer (`0`-`2`).
    pub selected_map_layer: u8,
    /// Persistent discovery state for world-map markers.
    pub map_marker_discovery: WorldMapMarkerDiscovery,
    /// Journal section currently displayed (`0`=main, `1`=side, `2`=trade).
    pub active_section: u8,
    /// Index of the first visible entry in each journal section.
    pub section_first_visible_entries: [u8; 3],
    /// Selected entry index in each journal section.
    pub section_selected_entries: [u8; 3],
    /// Number of active entries in main, side, and trade sections respectively.
    pub section_entry_counts: [u8; 3],
}

/// Persistent discovery flags for the three world-map layers.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WorldMapMarkerDiscovery {
    /// Marker flags for map layer 0 (10 slots; `0`=hidden, `1`=discovered).
    pub layer_0: [u8; 10],
    /// Marker flags for map layer 1 (10 slots; `0`=hidden, `1`=discovered).
    pub layer_1: [u8; 10],
    /// Marker flags for map layer 2 (7 slots; `0`=hidden, `1`=discovered).
    pub layer_2: [u8; 7],
    /// Unused tail of the ten-slot storage allocated for map layer 2.
    pub unused_layer_2_slots: [u8; 3],
}

impl JournalHeader {
    /// Parse the fixed 42-byte journal header.
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != JOURNAL_HEADER_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "JournalHeader requires 42 bytes",
            ));
        }

        let mut reader = std::io::Cursor::new(data);
        let is_world_map_open = reader.read_u8()?;
        let selected_map_layer = reader.read_u8()?;
        let map_marker_discovery = WorldMapMarkerDiscovery::read_from(&mut reader)?;
        let active_section = reader.read_u8()?;
        let mut section_first_visible_entries = [0; 3];
        reader.read_exact(&mut section_first_visible_entries)?;
        let mut section_selected_entries = [0; 3];
        reader.read_exact(&mut section_selected_entries)?;
        let mut section_entry_counts = [0; 3];
        reader.read_exact(&mut section_entry_counts)?;

        Ok(Self {
            is_world_map_open,
            selected_map_layer,
            map_marker_discovery,
            active_section,
            section_first_visible_entries,
            section_selected_entries,
            section_entry_counts,
        })
    }

    /// Write the fixed 42-byte journal header.
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u8(self.is_world_map_open)?;
        writer.write_u8(self.selected_map_layer)?;
        self.map_marker_discovery.write_to(writer)?;
        writer.write_u8(self.active_section)?;
        writer.write_all(&self.section_first_visible_entries)?;
        writer.write_all(&self.section_selected_entries)?;
        writer.write_all(&self.section_entry_counts)
    }
}

impl WorldMapMarkerDiscovery {
    fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut discovery = Self::default();
        reader.read_exact(&mut discovery.layer_0)?;
        reader.read_exact(&mut discovery.layer_1)?;
        reader.read_exact(&mut discovery.layer_2)?;
        reader.read_exact(&mut discovery.unused_layer_2_slots)?;
        Ok(discovery)
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.layer_0)?;
        writer.write_all(&self.layer_1)?;
        writer.write_all(&self.layer_2)?;
        writer.write_all(&self.unused_layer_2_slots)
    }
}

/// Journal entry (37 bytes each)
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct JournalEntry {
    /// Zero-based slot within this journal section.
    pub entry_index: u8,
    /// Title copied from the corresponding quest definition in `Quest.scr`.
    #[binary_record(string(encoding = "WINDOWS-1250", size = 32))]
    pub quest_title: String,
    /// ID of the quest definition.
    pub quest_id: u8,
    /// First follow-up quest ID linked to this journal entry, or `0` if absent.
    pub follow_up_quest_id_1: u8,
    /// Second follow-up quest ID linked to this journal entry, or `0` if absent.
    pub follow_up_quest_id_2: u8,
    /// Whether this journal entry is complete (`0`=active, `1`=completed).
    pub is_completed: u8,
}
