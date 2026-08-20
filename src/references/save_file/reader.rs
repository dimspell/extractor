use super::SaveFile;
use super::character::{
    CharacterData, CharacterIdentity, CharacterState, LearnedSpells, read_sprite_paths,
};
use super::events::{PostEventsData, read_events};
use super::game_tmp::read_maps;
use super::inventory::{InventoryData, InventorySlots};
use super::journal::JournalData;
use super::map_viewport::{MapViewportState, PostMapsData};
use super::party_members::read_party_members;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub(super) struct SaveReader<'a> {
    reader: Cursor<&'a [u8]>,
}

impl<'a> SaveReader<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self {
            reader: Cursor::new(data),
        }
    }

    pub(super) fn read(mut self) -> std::io::Result<SaveFile> {
        let jump_addr_after_maps =
            self.section("header", |reader| reader.read_u32::<LittleEndian>())?;
        let number_of_visited_maps =
            self.section("map count", |reader| reader.read_u32::<LittleEndian>())?;
        let maps = self.section("maps", |reader| read_maps(reader, number_of_visited_maps))?;

        if jump_addr_after_maps as u64 != self.reader.position() {
            self.reader.set_position(jump_addr_after_maps as u64);
        }

        let post_maps = self.section("post-maps", |reader| {
            PostMapsData::read_from(reader, number_of_visited_maps)
        })?;
        let map_viewport_state = self.section("map viewport", MapViewportState::read_from)?;
        let sprite_paths = self.section("sprite paths", read_sprite_paths)?;
        let character = self.section("character stats", CharacterData::read_from)?;
        let inventory = self.section("inventory", InventoryData::read_from)?;
        let character_state = self.section("character state", CharacterState::read_from)?;
        let character_identity =
            self.section("character identity", CharacterIdentity::read_from)?;
        let inventory_slots = self.section("inventory slots", InventorySlots::read_from)?;
        let learned_spells = self.section("learned spells", LearnedSpells::read_from)?;
        let party_members_count = self.section("party member count", |reader| {
            reader.read_u32::<LittleEndian>()
        })?;
        let party_members = self.section("party members", |reader| {
            read_party_members(reader, party_members_count)
        })?;
        let events = self.section("events", read_events)?;
        let post_events = self.section("post-events", PostEventsData::read_from)?;
        let journal = self.section("journal", JournalData::read_from)?;

        Ok(SaveFile {
            game_tmp_blob_size: jump_addr_after_maps,
            maps,
            post_maps,
            map_viewport_state,
            sprite_paths,
            character,
            inventory,
            character_state,
            character_identity,
            inventory_slots,
            learned_spells,
            party_members_count,
            party_members,
            events,
            post_events,
            journal,
        })
    }

    fn section<T>(
        &mut self,
        section: &'static str,
        read: impl FnOnce(&mut Cursor<&'a [u8]>) -> std::io::Result<T>,
    ) -> std::io::Result<T> {
        read(&mut self.reader).map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!(
                    "failed to read {section} at byte offset {}: {error}",
                    self.reader.position()
                ),
            )
        })
    }
}
