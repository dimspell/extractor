//! The single binary codec contract shared by every save-file section.
//!
//! Save sections differ only in the width of their record count (`u16` or
//! `u32`) and in the size/parser of an individual record. This module is the
//! single codec contract for all save-file sections: count widths, truncation
//! handling, bounds checks, section names and byte offsets are defined here so
//! that reading and writing cannot disagree about the format.
//!
//! [`Positioned`] is the position-aware IO wrapper used by both
//! [`super::reader::SaveReader`] and [`super::writer::SaveWriter`]. Its
//! [`Positioned::section`] method attaches the same
//! `failed to <verb> <section> at byte offset <offset>: <error>` context on
//! both paths, preserving the messages callers rely on.
//!
//! [`validate`] is also shared: the writer refuses to emit a model it cannot
//! represent, and the reader refuses to return a model the writer could not
//! reproduce. The following two conditions are not validated:
//!
//! * `game_tmp_blob_size` is preserved verbatim on read but recomputed as
//!   `8 + maps.len()` on write. Real saves may include padding between the map
//!   data and the post-maps header, so rejecting a differing jump address
//!   would reject readable saves. The synthetic round-trip fixture uses the
//!   writer's canonical value.
//! * Reserved/unknown bytes are preserved by their owning structs rather than
//!   checked here; they have no format invariant to validate.

use super::SaveFile;
use super::character::{LEARNED_SPELL_COUNT, SPRITE_PATH_COUNT};
use super::events::EVENT_COUNT;
use super::game_tmp::{
    DRAW_ITEM_EDIT_SIZE, DRAW_ITEM_EVENT_SIZE, DRAW_ITEM_HEAL_SIZE, DRAW_ITEM_MISC_SIZE,
    DRAW_ITEM_WEAPON_SIZE, EXTRA_OBJECT_TRAILER_FIXED_SIZE, EXTRA_OBJECT_TRAILER_RECORD_SIZE,
    MapSectionData,
};
use super::journal::JOURNAL_ENTRIES_PER_SECTION;
use super::map_viewport::MAP_VIEWPORT_CELL_COUNT;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Width of the count prefix stored before a section's records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CountWidth {
    U16,
    U32,
}

impl CountWidth {
    fn read<R: Read>(self, reader: &mut R) -> io::Result<usize> {
        match self {
            Self::U16 => Ok(reader.read_u16::<LittleEndian>()? as usize),
            Self::U32 => Ok(reader.read_u32::<LittleEndian>()? as usize),
        }
    }

    fn write<W: Write>(
        self,
        writer: &mut W,
        section: &'static str,
        field: &'static str,
        value: usize,
    ) -> io::Result<()> {
        match self {
            Self::U16 => {
                writer.write_u16::<LittleEndian>(checked_u16_for("write", section, field, value)?)
            }
            Self::U32 => {
                writer.write_u32::<LittleEndian>(checked_u32_for("write", section, field, value)?)
            }
        }
    }
}

/// Read a count with the width used by this format position.
pub(super) fn read_count<R: Read>(reader: &mut R, width: CountWidth) -> io::Result<usize> {
    width.read(reader)
}

/// Write a checked count with the width used by this format position.
pub(super) fn write_count<W: Write>(
    writer: &mut W,
    width: CountWidth,
    section: &'static str,
    field: &'static str,
    value: usize,
) -> io::Result<()> {
    width.write(writer, section, field, value)
}

/// Read a count prefix followed by `record_size`-byte records.
///
/// Records are read one at a time instead of preallocating `count * size`
/// bytes from an untrusted count: a truncated stream fails on the first
/// missing record without ever requesting an oversized allocation.
pub(super) fn read_counted_records<R: Read, T>(
    reader: &mut R,
    width: CountWidth,
    record_size: usize,
    parse: fn(&[u8]) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let count = read_count(reader, width)?;
    read_records(reader, count, record_size, parse)
}

/// Read an already-parsed count of fixed-size records (no count prefix).
pub(super) fn read_records<R: Read, T>(
    reader: &mut R,
    count: usize,
    record_size: usize,
    parse: fn(&[u8]) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let mut buffer = vec![0u8; record_size];
    let mut records = Vec::new();
    for _ in 0..count {
        reader.read_exact(&mut buffer)?;
        records.push(parse(&buffer)?);
    }
    Ok(records)
}

/// Write a count prefix followed by records of a fixed on-disk size.
pub(super) fn write_counted_records<W: Write, T>(
    writer: &mut W,
    width: CountWidth,
    records: &[T],
    section: &'static str,
    field: &'static str,
    mut write_record: impl FnMut(&T, &mut W) -> io::Result<()>,
) -> io::Result<()> {
    write_count(writer, width, section, field, records.len())?;
    for record in records {
        write_record(record, writer)?;
    }
    Ok(())
}

/// Write fixed-size records without a count prefix.
pub(super) fn write_records<W: Write, T>(
    writer: &mut W,
    records: &[T],
    mut write_record: impl FnMut(&T, &mut W) -> io::Result<()>,
) -> io::Result<()> {
    for record in records {
        write_record(record, writer)?;
    }
    Ok(())
}

/// Tracks bytes consumed/written and adds section context to IO errors.
pub(super) struct Positioned<T> {
    inner: T,
    position: u64,
}

impl<T> Positioned<T> {
    pub(super) fn new(inner: T) -> Self {
        Self { inner, position: 0 }
    }

    pub(super) fn position(&self) -> u64 {
        self.position
    }

    /// Run `op` and, on failure, report the section and the byte offset
    /// reached by the failed operation. `verb` is `"read"` or `"write"`.
    pub(super) fn section<F, R>(
        &mut self,
        verb: &'static str,
        section: &'static str,
        op: F,
    ) -> io::Result<R>
    where
        F: FnOnce(&mut Self) -> io::Result<R>,
    {
        match op(self) {
            Ok(value) => Ok(value),
            Err(error) => Err(contextual_error(verb, section, self.position, error)),
        }
    }
}

impl<T: Read> Read for Positioned<T> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.position += read as u64;
        Ok(read)
    }
}

impl<T: Write> Write for Positioned<T> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buffer)?;
        self.position += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for Positioned<T> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let position = self.inner.seek(position)?;
        self.position = position;
        Ok(position)
    }
}

/// Shared preflight validation. The writer runs it with `"write"` and the
/// reader with `"read"`, so both reject the same models with the same wording
/// apart from the verb.
pub(super) fn validate(save: &SaveFile, verb: &'static str) -> io::Result<()> {
    checked_u32_for(verb, "maps", "map count", save.maps.len())?;
    for map in &save.maps {
        checked_u32_for(verb, "maps", "monster count", map.monsters.len())?;
        checked_u32_for(verb, "maps", "NPC count", map.npcs.len())?;
        checked_u32_for(verb, "maps", "extra-object count", map.extra_objects.len())?;
        checked_u16_for(
            verb,
            "maps",
            "extra-object trailer record count",
            map.extra_objects_trailer.records.len(),
        )?;
        checked_u16_for(
            verb,
            "maps",
            "weapon ground-item count",
            map.draw_items_weapon.len(),
        )?;
        checked_u16_for(
            verb,
            "maps",
            "heal ground-item count",
            map.draw_items_heal.len(),
        )?;
        checked_u16_for(
            verb,
            "maps",
            "edit ground-item count",
            map.draw_items_edit.len(),
        )?;
        checked_u16_for(
            verb,
            "maps",
            "misc ground-item count",
            map.draw_items_misc.len(),
        )?;
        checked_u16_for(
            verb,
            "maps",
            "event ground-item count",
            map.draw_items_event.len(),
        )?;
        check_map_trailer_size(map).map_err(|error| {
            // `read_maps` reports a trailer mismatch as InvalidData; this
            // writer preflight reports InvalidInput.
            contextual_error(
                verb,
                "maps",
                0,
                io::Error::new(io::ErrorKind::InvalidInput, error.to_string()),
            )
        })?;
    }

    checked_u32_for(
        verb,
        "post-maps",
        "map ID count",
        save.post_maps.map_ids.len(),
    )?;
    if save.post_maps.number_of_visited_maps as usize != save.maps.len() {
        return Err(invalid(
            verb,
            "post-maps",
            "visited-map count does not match serialized map sections",
        ));
    }
    if save.post_maps.number_of_visited_maps as usize != save.post_maps.map_ids.len() {
        return Err(invalid(
            verb,
            "post-maps",
            "visited-map count does not match map IDs",
        ));
    }
    if save.sprite_paths.len() != SPRITE_PATH_COUNT {
        return Err(invalid(
            verb,
            "sprite paths",
            format!(
                "save has {} sprite paths, expected {SPRITE_PATH_COUNT}",
                save.sprite_paths.len()
            ),
        ));
    }

    checked_u16_for(
        verb,
        "inventory",
        "event-item count",
        save.inventory.event_items.len(),
    )?;
    checked_u16_for(
        verb,
        "inventory",
        "misc-item count",
        save.inventory.misc_items.len(),
    )?;
    checked_u16_for(
        verb,
        "inventory",
        "edit-item count",
        save.inventory.edit_items.len(),
    )?;
    checked_u16_for(
        verb,
        "inventory",
        "weapon-item count",
        save.inventory.weapon_items.len(),
    )?;
    checked_u16_for(
        verb,
        "inventory",
        "heal-item count",
        save.inventory.heal_items.len(),
    )?;

    if save.map_viewport_state.cells.len() != MAP_VIEWPORT_CELL_COUNT {
        return Err(invalid(
            verb,
            "map viewport",
            format!(
                "map viewport has {} cells, expected {MAP_VIEWPORT_CELL_COUNT}",
                save.map_viewport_state.cells.len()
            ),
        ));
    }
    if save.learned_spells.spells.len() != LEARNED_SPELL_COUNT {
        return Err(invalid(
            verb,
            "learned spells",
            format!(
                "save has {} learned-spell flags, expected {LEARNED_SPELL_COUNT}",
                save.learned_spells.spells.len()
            ),
        ));
    }

    checked_u32_for(
        verb,
        "party members",
        "party-member count",
        save.party_members.len(),
    )?;
    if save.party_members_count as usize != save.party_members.len() {
        return Err(invalid(
            verb,
            "party members",
            "stored count does not match party-member records",
        ));
    }
    if save.events.len() != EVENT_COUNT {
        return Err(invalid(
            verb,
            "events",
            format!(
                "save has {} events, expected {EVENT_COUNT}",
                save.events.len()
            ),
        ));
    }
    if save.post_events.walk_milestones.len() > u32::MAX as usize
        || save.post_events.walk_completions.len() > u32::MAX as usize
    {
        return Err(invalid(
            verb,
            "post-events",
            "walk record count exceeds u32",
        ));
    }
    if save.journal.main.len() != JOURNAL_ENTRIES_PER_SECTION
        || save.journal.side.len() != JOURNAL_ENTRIES_PER_SECTION
        || save.journal.trade.len() != JOURNAL_ENTRIES_PER_SECTION
    {
        return Err(invalid(
            verb,
            "journal",
            "each journal section must contain 100 entries",
        ));
    }

    Ok(())
}

/// Check the one cross-section size invariant: the trailer size covers its
/// own fixed header, its records and all five ground-item sections.
pub(super) fn check_map_trailer_size(map: &MapSectionData) -> io::Result<()> {
    let expected_tail_size = EXTRA_OBJECT_TRAILER_FIXED_SIZE
        + map.extra_objects_trailer.records.len() * EXTRA_OBJECT_TRAILER_RECORD_SIZE
        + map.draw_items_weapon.len() * DRAW_ITEM_WEAPON_SIZE
        + map.draw_items_heal.len() * DRAW_ITEM_HEAL_SIZE
        + map.draw_items_edit.len() * DRAW_ITEM_EDIT_SIZE
        + map.draw_items_misc.len() * DRAW_ITEM_MISC_SIZE
        + map.draw_items_event.len() * DRAW_ITEM_EVENT_SIZE;
    if map.extra_objects_trailer.tail_size as usize != expected_tail_size {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "map extra-object trailer size is {}, expected {expected_tail_size}",
                map.extra_objects_trailer.tail_size
            ),
        ))
    } else {
        Ok(())
    }
}

fn checked_u16_for(
    verb: &'static str,
    section: &'static str,
    field: &'static str,
    value: usize,
) -> io::Result<u16> {
    u16::try_from(value).map_err(|_| {
        contextual_error(
            verb,
            section,
            0,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{field} exceeds u16::MAX"),
            ),
        )
    })
}

fn checked_u32_for(
    verb: &'static str,
    section: &'static str,
    field: &'static str,
    value: usize,
) -> io::Result<u32> {
    u32::try_from(value).map_err(|_| {
        contextual_error(
            verb,
            section,
            0,
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{field} exceeds u32::MAX"),
            ),
        )
    })
}

fn invalid(verb: &'static str, section: &'static str, message: impl Into<String>) -> io::Error {
    contextual_error(
        verb,
        section,
        0,
        io::Error::new(io::ErrorKind::InvalidInput, message.into()),
    )
}

pub(super) fn contextual_error(
    verb: &'static str,
    section: &'static str,
    offset: u64,
    error: io::Error,
) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("failed to {verb} {section} at byte offset {offset}: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_u16_pair(data: &[u8]) -> io::Result<Vec<u16>> {
        read_counted_records(&mut io::Cursor::new(data), CountWidth::U16, 2, |bytes| {
            Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
        })
    }

    fn parse_u32_pair(data: &[u8]) -> io::Result<Vec<u32>> {
        read_counted_records(&mut io::Cursor::new(data), CountWidth::U32, 4, |bytes| {
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        })
    }

    #[test]
    fn test_codec_count_widths_round_trip_with_exact_prefix_bytes() {
        let mut u16_bytes = Vec::new();
        write_counted_records(
            &mut u16_bytes,
            CountWidth::U16,
            &[7u16, 9u16],
            "unit",
            "item count",
            |value, writer| writer.write_u16::<LittleEndian>(*value),
        )
        .unwrap();
        assert_eq!(u16_bytes, [2u8, 0, 7, 0, 9, 0]);
        assert_eq!(parse_u16_pair(&u16_bytes).unwrap(), [7, 9]);

        let mut u32_bytes = Vec::new();
        write_counted_records(
            &mut u32_bytes,
            CountWidth::U32,
            &[7u32, 9u32],
            "unit",
            "item count",
            |value, writer| writer.write_u32::<LittleEndian>(*value),
        )
        .unwrap();
        assert_eq!(u32_bytes, [2u8, 0, 0, 0, 7, 0, 0, 0, 9, 0, 0, 0]);
        assert_eq!(parse_u32_pair(&u32_bytes).unwrap(), [7, 9]);
    }

    #[test]
    fn test_codec_rejects_truncated_count_and_truncated_records() {
        let missing_count = parse_u16_pair(&[1u8]).unwrap_err();
        assert_eq!(missing_count.kind(), io::ErrorKind::UnexpectedEof);

        // Count says two records, but only one is present.
        let missing_record = parse_u16_pair(&[2u8, 0, 7, 0]).unwrap_err();
        assert_eq!(missing_record.kind(), io::ErrorKind::UnexpectedEof);

        let missing_u32_count = parse_u32_pair(&[0u8, 0, 0]).unwrap_err();
        assert_eq!(missing_u32_count.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_codec_checks_count_against_wire_width() {
        let u16_error = checked_u16_for(
            "write",
            "inventory",
            "item count",
            usize::from(u16::MAX) + 1,
        )
        .unwrap_err();
        assert_eq!(u16_error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(
            u16_error.to_string(),
            "failed to write inventory at byte offset 0: item count exceeds u16::MAX"
        );

        #[cfg(target_pointer_width = "64")]
        {
            let u32_error =
                checked_u32_for("write", "maps", "map count", u32::MAX as usize + 1).unwrap_err();
            assert_eq!(u32_error.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(
                u32_error.to_string(),
                "failed to write maps at byte offset 0: map count exceeds u32::MAX"
            );
        }
    }

    #[test]
    fn test_positioned_section_attaches_verb_section_and_offset() {
        let mut positioned = Positioned::new(io::Cursor::new([1u8, 2, 3]));
        let error = positioned
            .section("read", "unit section", |reader| {
                let mut buffer = [0u8; 4];
                reader.read_exact(&mut buffer)
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(
            error.to_string(),
            "failed to read unit section at byte offset 3: failed to fill whole buffer"
        );
        assert_eq!(positioned.position(), 3);
    }
}
