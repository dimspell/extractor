use super::SaveFile;
use super::character::write_sprite_paths;
use super::codec::{self, Positioned};
use super::events::write_events;
use super::game_tmp::write_maps;
use super::party_members::write_party_members;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub(super) struct SaveWriter<'a, W> {
    save: &'a SaveFile,
    output: Positioned<&'a mut W>,
}

impl<'a, W: Write> SaveWriter<'a, W> {
    pub(super) fn new(save: &'a SaveFile, output: &'a mut W) -> Self {
        Self {
            save,
            output: Positioned::new(output),
        }
    }

    pub(super) fn write(mut self) -> std::io::Result<()> {
        codec::validate(self.save, "write")?;

        let mut maps = Vec::new();
        write_maps(&self.save.maps, &mut maps)
            .map_err(|error| codec::contextual_error("write", "maps", 0, error))?;
        self.section("header and maps", |output| {
            output.write_u32::<LittleEndian>(8u32 + maps.len() as u32)?;
            output.write_u32::<LittleEndian>(self.save.maps.len() as u32)?;
            output.write_all(&maps)
        })?;
        self.section("post-maps", |output| self.save.post_maps.write_to(output))?;
        self.section("map viewport", |output| {
            self.save.map_viewport_state.write_to(output)
        })?;
        self.section("sprite paths", |output| {
            write_sprite_paths(&self.save.sprite_paths, output)
        })?;
        self.section("character stats", |output| {
            self.save.character.write(output)
        })?;
        self.section("inventory", |output| self.save.inventory.write_to(output))?;
        self.section("character state", |output| {
            self.save.character_state.write_to(output)
        })?;
        self.section("character identity", |output| {
            self.save.character_identity.write(output)
        })?;
        self.section("inventory slots", |output| {
            self.save.inventory_slots.write(output)
        })?;
        self.section("learned spells", |output| {
            self.save.learned_spells.write_to(output)
        })?;
        self.section("party members", |output| {
            write_party_members(&self.save.party_members, output)
        })?;
        self.section("events", |output| write_events(&self.save.events, output))?;
        self.section("post-events", |output| {
            self.save.post_events.write_to(output)
        })?;
        self.section("journal", |output| self.save.journal.write_to(output))
    }

    fn section(
        &mut self,
        name: &'static str,
        write: impl FnOnce(&mut Positioned<&'a mut W>) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        self.output.section("write", name, write)
    }
}
