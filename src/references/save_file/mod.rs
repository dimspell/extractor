// Save file extraction and parsing for Dispel RPG.

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

use super::extractor::Extractor;
use byteorder::{LittleEndian, WriteBytesExt};
use character::CharacterData;
pub use character::{CharacterIdentity, LearnedSpells};
pub use events::{EventRecord, PostEventsData};
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

    // Write helpers remain here so the reader refactor does not alter serialization.
    fn write_maps_section<W: Write>(
        maps: &[MapSectionData],
        writer: &mut W,
    ) -> std::io::Result<()> {
        for map in maps {
            writer.write_u32::<LittleEndian>(map.map_id)?;

            writer.write_u32::<LittleEndian>(map.monsters.len() as u32)?;
            for monster in &map.monsters {
                monster.write(writer)?;
            }

            writer.write_u32::<LittleEndian>(map.npcs.len() as u32)?;
            for npc in &map.npcs {
                npc.write(writer)?;
            }

            writer.write_u32::<LittleEndian>(0)?;
            writer.write_u32::<LittleEndian>(map.extra_objects.len() as u32)?;
            for extra in &map.extra_objects {
                extra.write(writer)?;
            }

            let record_count =
                u16::try_from(map.extra_objects_trailer.records.len()).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "map extra-object trailer has more than u16::MAX records",
                    )
                })?;
            let expected_tail_size = 17usize
                + map.extra_objects_trailer.records.len() * 24
                + map.draw_items_weapon.len() * 296
                + map.draw_items_heal.len() * 264
                + map.draw_items_edit.len() * 280
                + map.draw_items_misc.len() * 268
                + map.draw_items_event.len() * 252;
            if map.extra_objects_trailer.tail_size as usize != expected_tail_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "map extra-object trailer size is {}, expected {expected_tail_size}",
                        map.extra_objects_trailer.tail_size
                    ),
                ));
            }
            writer.write_u32::<LittleEndian>(map.extra_objects_trailer.tail_size)?;
            writer.write_u16::<LittleEndian>(record_count)?;
            for record in &map.extra_objects_trailer.records {
                record.write(writer)?;
            }
            writer.write_u8(map.extra_objects_trailer.automatic_placement_active)?;
            writer
                .write_u16::<LittleEndian>(map.extra_objects_trailer.automatic_placement_value)?;
            writer.write_u16::<LittleEndian>(
                map.extra_objects_trailer
                    .automatic_placement_global_item_index,
            )?;

            writer.write_u16::<LittleEndian>(map.draw_items_weapon.len() as u16)?;
            for item in &map.draw_items_weapon {
                item.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_heal.len() as u16)?;
            for item in &map.draw_items_heal {
                item.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_edit.len() as u16)?;
            for item in &map.draw_items_edit {
                item.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_misc.len() as u16)?;
            for item in &map.draw_items_misc {
                item.write(writer)?;
            }
            writer.write_u16::<LittleEndian>(map.draw_items_event.len() as u16)?;
            for item in &map.draw_items_event {
                item.write(writer)?;
            }
            writer.write_u32::<LittleEndian>(0)?;
        }
        Ok(())
    }

    fn write_post_maps_data<W: Write>(data: &PostMapsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(data.map_section_terminator)?;
        writer.write_f32::<LittleEndian>(data.game_version)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_1)?;
        writer.write_u32::<LittleEndian>(data.all_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.ref_map_ini_id)?;
        writer.write_u32::<LittleEndian>(data.monster_block_size)?;
        writer.write_u32::<LittleEndian>(data.npc_block_size)?;
        writer.write_u32::<LittleEndian>(data.unknown_header_value_2)?;
        writer.write_u32::<LittleEndian>(data.extra_object_block_size)?;
        let map_id_count = u32::try_from(data.map_ids.len()).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "post-maps has more than u32::MAX map IDs",
            )
        })?;
        if data.number_of_visited_maps != map_id_count {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "post-maps visited-map count is {}, but {} map IDs were provided",
                    data.number_of_visited_maps,
                    data.map_ids.len()
                ),
            ));
        }
        writer.write_u32::<LittleEndian>(data.number_of_visited_maps)?;
        for map_id in &data.map_ids {
            writer.write_u32::<LittleEndian>(*map_id)?;
        }
        Ok(())
    }

    fn write_sprite_paths<W: Write>(paths: &[String], writer: &mut W) -> std::io::Result<()> {
        for index in 0..4 {
            let path = paths.get(index).map(String::as_str).unwrap_or("");
            let mut buffer = [0u8; 60];
            let (encoded, _, _) = encoding_rs::WINDOWS_1250.encode(path);
            let len = encoded.len().min(buffer.len());
            buffer[..len].copy_from_slice(&encoded[..len]);
            writer.write_all(&buffer)?;
        }
        Ok(())
    }

    fn write_character_stats<W: Write>(
        character: &CharacterData,
        writer: &mut W,
    ) -> std::io::Result<()> {
        character.write(writer)
    }

    fn write_inventory<W: Write>(inventory: &InventoryData, writer: &mut W) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(inventory.event_items.len() as u16)?;
        for item in &inventory.event_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inventory.misc_items.len() as u16)?;
        for item in &inventory.misc_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inventory.edit_items.len() as u16)?;
        for item in &inventory.edit_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inventory.weapon_items.len() as u16)?;
        for item in &inventory.weapon_items {
            item.write(writer)?;
        }
        writer.write_u16::<LittleEndian>(inventory.heal_items.len() as u16)?;
        for item in &inventory.heal_items {
            item.write(writer)?;
        }
        Ok(())
    }

    fn write_character_identity<W: Write>(
        identity: &CharacterIdentity,
        writer: &mut W,
    ) -> std::io::Result<()> {
        identity.write(writer)
    }

    fn write_inventory_slots<W: Write>(
        slots: &InventorySlots,
        writer: &mut W,
    ) -> std::io::Result<()> {
        slots.write(writer)
    }

    fn write_events<W: Write>(events: &[EventRecord], writer: &mut W) -> std::io::Result<()> {
        for event in events {
            event.write(writer)?;
        }
        Ok(())
    }

    fn write_post_events<W: Write>(data: &PostEventsData, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&data.block_a)?;
        writer.write_u32::<LittleEndian>((data.records.len() / 24) as u32)?;
        writer.write_all(&data.records)?;
        writer.write_all(&data.block_b)
    }

    fn write_journal<W: Write>(journal: &JournalData, writer: &mut W) -> std::io::Result<()> {
        journal.header.write(writer)?;
        for entry in &journal.main {
            entry.write(writer)?;
        }
        for entry in &journal.side {
            entry.write(writer)?;
        }
        for entry in &journal.trade {
            entry.write(writer)?;
        }
        Ok(())
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
        let save = &records[0];

        let mut maps = Vec::new();
        Self::write_maps_section(&save.maps, &mut maps)?;
        writer.write_u32::<LittleEndian>(8u32 + maps.len() as u32)?;
        writer.write_u32::<LittleEndian>(save.maps.len() as u32)?;
        writer.write_all(&maps)?;
        Self::write_post_maps_data(&save.post_maps, writer)?;
        save.map_viewport_state.write_to(writer)?;
        Self::write_sprite_paths(&save.sprite_paths, writer)?;
        Self::write_character_stats(&save.character, writer)?;
        Self::write_inventory(&save.inventory, writer)?;
        Self::write_character_identity(&save.character_identity, writer)?;
        Self::write_inventory_slots(&save.inventory_slots, writer)?;
        writer.write_all(&save.learned_spells.spells)?;
        writer.write_u32::<LittleEndian>(save.party_members_count)?;
        for member in &save.party_members {
            member.write(writer)?;
        }
        Self::write_events(&save.events, writer)?;
        Self::write_post_events(&save.post_events, writer)?;
        Self::write_journal(&save.journal, writer)
    }
}
