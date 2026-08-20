use super::SaveFile;
use super::character::{LEARNED_SPELL_COUNT, SPRITE_PATH_COUNT, write_sprite_paths};
use super::events::{EVENT_COUNT, write_events};
use super::game_tmp::{
    DRAW_ITEM_EDIT_SIZE, DRAW_ITEM_EVENT_SIZE, DRAW_ITEM_HEAL_SIZE, DRAW_ITEM_MISC_SIZE,
    DRAW_ITEM_WEAPON_SIZE, EXTRA_OBJECT_TRAILER_FIXED_SIZE, EXTRA_OBJECT_TRAILER_RECORD_SIZE,
    write_maps,
};
use super::journal::JOURNAL_ENTRIES_PER_SECTION;
use super::map_viewport::MAP_VIEWPORT_CELL_COUNT;
use super::party_members::write_party_members;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

pub(super) struct SaveWriter<'a, W> {
    save: &'a SaveFile,
    output: PositionWriter<'a, W>,
}

impl<'a, W: Write> SaveWriter<'a, W> {
    pub(super) fn new(save: &'a SaveFile, output: &'a mut W) -> Self {
        Self {
            save,
            output: PositionWriter::new(output),
        }
    }

    pub(super) fn write(mut self) -> std::io::Result<()> {
        validate(self.save)?;

        let mut maps = Vec::new();
        write_maps(&self.save.maps, &mut maps)
            .map_err(|error| contextual_error("maps", 0, error))?;
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
        write: impl FnOnce(&mut PositionWriter<'a, W>) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        write(&mut self.output)
            .map_err(|error| contextual_error(name, self.output.position(), error))
    }
}

fn validate(save: &SaveFile) -> std::io::Result<()> {
    checked_u32("maps", "map count", save.maps.len())?;
    for map in &save.maps {
        checked_u32("maps", "monster count", map.monsters.len())?;
        checked_u32("maps", "NPC count", map.npcs.len())?;
        checked_u32("maps", "extra-object count", map.extra_objects.len())?;
        checked_u16(
            "maps",
            "extra-object trailer record count",
            map.extra_objects_trailer.records.len(),
        )?;
        checked_u16(
            "maps",
            "weapon ground-item count",
            map.draw_items_weapon.len(),
        )?;
        checked_u16("maps", "heal ground-item count", map.draw_items_heal.len())?;
        checked_u16("maps", "edit ground-item count", map.draw_items_edit.len())?;
        checked_u16("maps", "misc ground-item count", map.draw_items_misc.len())?;
        checked_u16(
            "maps",
            "event ground-item count",
            map.draw_items_event.len(),
        )?;

        let expected_tail_size = EXTRA_OBJECT_TRAILER_FIXED_SIZE
            + map.extra_objects_trailer.records.len() * EXTRA_OBJECT_TRAILER_RECORD_SIZE
            + map.draw_items_weapon.len() * DRAW_ITEM_WEAPON_SIZE
            + map.draw_items_heal.len() * DRAW_ITEM_HEAL_SIZE
            + map.draw_items_edit.len() * DRAW_ITEM_EDIT_SIZE
            + map.draw_items_misc.len() * DRAW_ITEM_MISC_SIZE
            + map.draw_items_event.len() * DRAW_ITEM_EVENT_SIZE;
        if map.extra_objects_trailer.tail_size as usize != expected_tail_size {
            return invalid(
                "maps",
                format!(
                    "map extra-object trailer size is {}, expected {expected_tail_size}",
                    map.extra_objects_trailer.tail_size
                ),
            );
        }
    }

    checked_u32("post-maps", "map ID count", save.post_maps.map_ids.len())?;
    if save.post_maps.number_of_visited_maps as usize != save.maps.len() {
        return invalid(
            "post-maps",
            "visited-map count does not match serialized map sections",
        );
    }
    if save.post_maps.number_of_visited_maps as usize != save.post_maps.map_ids.len() {
        return invalid("post-maps", "visited-map count does not match map IDs");
    }
    if save.sprite_paths.len() != SPRITE_PATH_COUNT {
        return invalid(
            "sprite paths",
            format!(
                "save has {} sprite paths, expected {SPRITE_PATH_COUNT}",
                save.sprite_paths.len()
            ),
        );
    }

    checked_u16(
        "inventory",
        "event-item count",
        save.inventory.event_items.len(),
    )?;
    checked_u16(
        "inventory",
        "misc-item count",
        save.inventory.misc_items.len(),
    )?;
    checked_u16(
        "inventory",
        "edit-item count",
        save.inventory.edit_items.len(),
    )?;
    checked_u16(
        "inventory",
        "weapon-item count",
        save.inventory.weapon_items.len(),
    )?;
    checked_u16(
        "inventory",
        "heal-item count",
        save.inventory.heal_items.len(),
    )?;

    if save.map_viewport_state.cells.len() != MAP_VIEWPORT_CELL_COUNT {
        return invalid(
            "map viewport",
            format!(
                "map viewport has {} cells, expected {MAP_VIEWPORT_CELL_COUNT}",
                save.map_viewport_state.cells.len()
            ),
        );
    }
    if save.learned_spells.spells.len() != LEARNED_SPELL_COUNT {
        return invalid(
            "learned spells",
            format!(
                "save has {} learned-spell flags, expected {LEARNED_SPELL_COUNT}",
                save.learned_spells.spells.len()
            ),
        );
    }

    checked_u32(
        "party members",
        "party-member count",
        save.party_members.len(),
    )?;
    if save.party_members_count as usize != save.party_members.len() {
        return invalid(
            "party members",
            "stored count does not match party-member records",
        );
    }
    if save.events.len() != EVENT_COUNT {
        return invalid(
            "events",
            format!(
                "save has {} events, expected {EVENT_COUNT}",
                save.events.len()
            ),
        );
    }
    if save.post_events.walk_milestones.len() > u32::MAX as usize
        || save.post_events.walk_completions.len() > u32::MAX as usize
    {
        return invalid("post-events", "walk record count exceeds u32");
    }
    if save.journal.main.len() != JOURNAL_ENTRIES_PER_SECTION
        || save.journal.side.len() != JOURNAL_ENTRIES_PER_SECTION
        || save.journal.trade.len() != JOURNAL_ENTRIES_PER_SECTION
    {
        return invalid("journal", "each journal section must contain 100 entries");
    }

    Ok(())
}

pub(super) fn checked_u16(
    section: &'static str,
    field: &'static str,
    value: usize,
) -> std::io::Result<u16> {
    u16::try_from(value).map_err(|_| {
        contextual_error(
            section,
            0,
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{field} exceeds u16::MAX"),
            ),
        )
    })
}

pub(super) fn checked_u32(
    section: &'static str,
    field: &'static str,
    value: usize,
) -> std::io::Result<u32> {
    u32::try_from(value).map_err(|_| {
        contextual_error(
            section,
            0,
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{field} exceeds u32::MAX"),
            ),
        )
    })
}

fn invalid<T>(section: &'static str, message: impl Into<String>) -> std::io::Result<T> {
    Err(contextual_error(
        section,
        0,
        std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into()),
    ))
}

fn contextual_error(section: &'static str, offset: u64, error: std::io::Error) -> std::io::Error {
    std::io::Error::new(
        error.kind(),
        format!("failed to write {section} at byte offset {offset}: {error}"),
    )
}

struct PositionWriter<'a, W> {
    inner: &'a mut W,
    position: u64,
}

impl<'a, W> PositionWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self { inner, position: 0 }
    }

    fn position(&self) -> u64 {
        self.position
    }
}

impl<W: Write> Write for PositionWriter<'_, W> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.position += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
