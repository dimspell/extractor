// Save file extraction and serialization for Dispel RPG.

pub mod character;
pub mod events;
pub mod game_tmp;
pub mod inventory;
pub mod journal;
pub mod map_viewport;
pub mod party_members;
mod reader;
#[cfg(test)]
pub mod tests;
mod writer;

use super::extractor::Extractor;
use character::CharacterData;
pub use character::{CharacterIdentity, CharacterState, LearnedSpells};
pub use events::{
    DismissedCompanionProgression, EventRecord, PostEventsData, WalkCompletionRecord,
    WalkMilestoneRecord,
};
pub use game_tmp::{
    DrawItemEditItem, DrawItemEventItem, DrawItemHealItem, DrawItemMiscItem, DrawItemWeaponItem,
    ExtraObjectRecord, ExtraObjectTrailerRecord, MapExtraObjectsTrailer, MapSectionData,
    MonsterRecord, NpcRecord,
};
use inventory::InventorySlots;
pub use inventory::{
    BeltPotionSlot, InventoryData, InventoryEditItem, InventoryEventItem, InventoryHealItem,
    InventoryMiscItem, InventoryWeaponItem,
};
pub use journal::{JournalData, JournalEntry, JournalHeader};
pub use map_viewport::{MapViewportCell, MapViewportState, PostMapsData};
pub use party_members::PartyMember;
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, Write};

/// Complete save file structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveFile {
    /// Jump address after all map data (first 4 bytes of the file).
    pub game_tmp_blob_size: u32,
    /// Per-map world state.
    pub maps: Vec<MapSectionData>,
    /// Unknown data between maps and sprite paths.
    pub post_maps: PostMapsData,
    /// Fixed-size serialized isometric map viewport state.
    pub map_viewport_state: MapViewportState,
    /// Character sprite paths.
    pub sprite_paths: Vec<String>,
    /// Character attributes, stats, and map position.
    pub character: CharacterData,
    /// Items grouped by inventory category.
    pub inventory: InventoryData,
    /// Character runtime state (serials, action/movement/teleport state,
    /// stat bonuses, position).
    pub character_state: CharacterState,
    /// Character name, class, and spell-bar state.
    pub character_identity: CharacterIdentity,
    /// Equipment, belt, and inventory placement state.
    pub inventory_slots: InventorySlots,
    /// One flag per learned spell.
    pub learned_spells: LearnedSpells,
    /// Serialized party-member count.
    pub party_members_count: u32,
    /// Recruited party members.
    pub party_members: Vec<PartyMember>,
    /// Event scripts.
    pub events: Vec<EventRecord>,
    /// Unknown data between events and journal.
    pub post_events: PostEventsData,
    /// Journal entries.
    pub journal: JournalData,
}

impl SaveFile {
    /// Parse a complete save file from binary data.
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        reader::SaveReader::new(data).read()
    }

    /// Serialize this save to its binary representation.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer::SaveWriter::new(self, writer).write()
    }
}

impl Extractor for SaveFile {
    fn parse<R: Read + Seek>(reader: &mut R, _len: u64) -> std::io::Result<Vec<Self>> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(vec![SaveFile::parse(&data)?])
    }

    fn to_writer<W: Write>(records: &[Self], writer: &mut W) -> std::io::Result<()> {
        if records.len() != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SaveFile can only serialize one record at a time",
            ));
        }
        records[0].write_to(writer)
    }
}
