//! Maps known game files to their immutable Hexedit structure overlays.

use std::path::Path;

use dispel_core::{
    RecordLayout,
    references::{monster_db::Monster, save_file::SaveFile},
};
use hexedit::{
    BinaryLayout, FixedRecordBinaryLayout, FixedRecordField, NamedSpan, SpanBinaryLayout,
};

/// Return a layout only for an explicitly recognized game-relative path.
pub fn layout_for_path(path: &Path, bytes: &[u8]) -> Option<Box<dyn BinaryLayout>> {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if normalized.ends_with("monsteringame/monster.db") {
        let definition = Monster::LAYOUT;
        let fields: Vec<FixedRecordField> = definition
            .fields
            .iter()
            .map(|field| FixedRecordField {
                name: field.name,
                offset: field.offset,
                size: field.size,
                ty: field.ty,
            })
            .collect();
        return Some(Box::new(FixedRecordBinaryLayout::new(
            definition.type_name,
            definition.header_size,
            definition.record_size,
            fields,
        )));
    }
    normalized
        .ends_with(".sav")
        .then(|| save_file_layout(bytes))
        .flatten()
}

/// Build nested section spans from the same counts the save-file parser accepts.
/// The parser must succeed first; an unrecognized or truncated `.sav` stays a
/// normal hex file instead of receiving misleading labels.
fn save_file_layout(bytes: &[u8]) -> Option<Box<dyn BinaryLayout>> {
    let save = SaveFile::parse(bytes).ok()?;
    let mut spans = Vec::new();
    let mut offset = 0u64;
    push_span(&mut spans, &mut offset, "Header", "u32", 4, 0)?;

    let maps_start = offset;
    push_span(&mut spans, &mut offset, "Map count", "u32", 4, 0)?;
    for (map_index, map) in save.maps.iter().enumerate() {
        let map_start = offset;
        let map_len = 4
            + 4
            + map.monsters.len().checked_mul(329)?
            + 4
            + map.npcs.len().checked_mul(349)?
            + 4
            + 4
            + map.extra_objects.len().checked_mul(200)?
            + 11
            + map.extra_objects_trailer.records.len().checked_mul(24)?
            + 2
            + map.draw_items_weapon.len().checked_mul(296)?
            + 2
            + map.draw_items_heal.len().checked_mul(264)?
            + 2
            + map.draw_items_edit.len().checked_mul(280)?
            + 2
            + map.draw_items_misc.len().checked_mul(268)?
            + 2
            + map.draw_items_event.len().checked_mul(252)?
            + 4;
        push_span(
            &mut spans,
            &mut offset,
            "Map",
            "section",
            map_len,
            map_index as u64,
        )?;
        let mut nested = map_start;
        push_span(
            &mut spans,
            &mut nested,
            "Map ID",
            "u32",
            4,
            map_index as u64,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Monster record",
            329,
            map.monsters.len(),
            4,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "NPC record",
            349,
            map.npcs.len(),
            4,
        )?;
        push_span(&mut spans, &mut nested, "Map separator", "u32", 4, 0)?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Extra-object record",
            200,
            map.extra_objects.len(),
            4,
        )?;
        push_span(
            &mut spans,
            &mut nested,
            "Extra-object trailer",
            "u32 + u16 + records + controls",
            11 + map.extra_objects_trailer.records.len().checked_mul(24)?,
            0,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Ground weapon",
            296,
            map.draw_items_weapon.len(),
            2,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Ground heal item",
            264,
            map.draw_items_heal.len(),
            2,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Ground edit item",
            280,
            map.draw_items_edit.len(),
            2,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Ground misc item",
            268,
            map.draw_items_misc.len(),
            2,
        )?;
        push_counted_records(
            &mut spans,
            &mut nested,
            "Ground event item",
            252,
            map.draw_items_event.len(),
            2,
        )?;
        push_span(&mut spans, &mut nested, "Map end separator", "u32", 4, 0)?;
        if nested != offset {
            return None;
        }
    }
    spans.push(NamedSpan {
        range: maps_start..offset,
        name: "Maps",
        ty: "section",
        record_index: 0,
    });
    let jump = u64::from(save.game_tmp_blob_size);
    if jump < offset || jump > bytes.len() as u64 {
        return None;
    }
    if jump > offset {
        spans.push(NamedSpan {
            range: offset..jump,
            name: "Map padding",
            ty: "opaque",
            record_index: 0,
        });
        offset = jump;
    }

    let post_maps_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Post-maps data",
        "section",
        40 + save.maps.len().checked_mul(4)? + 10_148,
        0,
    )?;
    let mut nested = post_maps_start;
    push_span(&mut spans, &mut nested, "Post-maps header", "opaque", 40, 0)?;
    push_records(
        &mut spans,
        &mut nested,
        "Visited map ID",
        4,
        save.maps.len(),
    )?;
    push_span(
        &mut spans,
        &mut nested,
        "Post-maps opaque data",
        "opaque",
        10_148,
        0,
    )?;
    if nested != offset {
        return None;
    }

    let sprites_start = offset;
    push_span(&mut spans, &mut offset, "Sprite paths", "section", 240, 0)?;
    for index in 0..4u64 {
        add_span(
            &mut spans,
            sprites_start + index * 60,
            60,
            "Sprite path",
            "string(WINDOWS-1250)",
            index,
        )?;
    }

    let stats_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Character stats",
        "section",
        109,
        0,
    )?;
    let mut nested = stats_start;
    push_span(&mut spans, &mut nested, "Belt data", "opaque", 8, 0)?;
    push_span(
        &mut spans,
        &mut nested,
        "Character position",
        "i16 pair",
        4,
        0,
    )?;
    push_span(&mut spans, &mut nested, "Stats header", "opaque", 28, 0)?;
    push_span(&mut spans, &mut nested, "Core stats", "record", 60, 0)?;
    push_span(
        &mut spans,
        &mut nested,
        "Stats trailing data",
        "opaque",
        9,
        0,
    )?;
    if nested != offset {
        return None;
    }
    let inventory_len = 2
        + save.inventory.event_items.len().checked_mul(244)?
        + 2
        + save.inventory.misc_items.len().checked_mul(264)?
        + 2
        + save.inventory.edit_items.len().checked_mul(272)?
        + 2
        + save.inventory.weapon_items.len().checked_mul(292)?
        + 2
        + save.inventory.heal_items.len().checked_mul(256)?;
    let inventory_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Inventory",
        "section",
        inventory_len,
        0,
    )?;
    let mut nested = inventory_start;
    push_counted_records(
        &mut spans,
        &mut nested,
        "Inventory event item",
        244,
        save.inventory.event_items.len(),
        2,
    )?;
    push_counted_records(
        &mut spans,
        &mut nested,
        "Inventory misc item",
        264,
        save.inventory.misc_items.len(),
        2,
    )?;
    push_counted_records(
        &mut spans,
        &mut nested,
        "Inventory edit item",
        272,
        save.inventory.edit_items.len(),
        2,
    )?;
    push_counted_records(
        &mut spans,
        &mut nested,
        "Inventory weapon",
        292,
        save.inventory.weapon_items.len(),
        2,
    )?;
    push_counted_records(
        &mut spans,
        &mut nested,
        "Inventory heal item",
        256,
        save.inventory.heal_items.len(),
        2,
    )?;
    if nested != offset {
        return None;
    }

    let identity_len = 4_160 + save.party_members.len().checked_mul(321)?;
    let identity_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Character identity",
        "section",
        identity_len,
        0,
    )?;
    let mut nested = identity_start;
    push_span(
        &mut spans,
        &mut nested,
        "Identity unknown data",
        "opaque",
        96,
        0,
    )?;
    push_span(
        &mut spans,
        &mut nested,
        "Player name",
        "string(WINDOWS-1250)",
        11,
        0,
    )?;
    push_span(&mut spans, &mut nested, "Player class", "record", 13, 0)?;
    push_span(
        &mut spans,
        &mut nested,
        "Character data header",
        "opaque",
        11,
        0,
    )?;
    push_records(&mut spans, &mut nested, "Equipment slot", 9, 12)?;
    push_records(&mut spans, &mut nested, "Belt potion slot", 16, 6)?;
    push_records(&mut spans, &mut nested, "Inventory placement", 20, 189)?;
    push_span(&mut spans, &mut nested, "Learned spells", "flags", 41, 0)?;
    push_span(&mut spans, &mut nested, "Party member count", "u32", 4, 0)?;
    push_records(
        &mut spans,
        &mut nested,
        "Party member",
        321,
        save.party_members.len(),
    )?;
    if nested != offset {
        return None;
    }

    let events_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Event scripts",
        "section",
        save.events.len().checked_mul(284)?,
        0,
    )?;
    for index in 0..save.events.len() {
        add_span(
            &mut spans,
            events_start + (index * 284) as u64,
            284,
            "Event script",
            "record",
            index as u64,
        )?;
    }

    let post_events_start = offset;
    let post_events_len = 8
        + (save.post_events.walk_milestones.len() + save.post_events.walk_completions.len())
            .checked_mul(24)?
        + 8 * 4
        + 24;
    push_span(
        &mut spans,
        &mut offset,
        "Post-events data",
        "section",
        post_events_len,
        0,
    )?;
    let mut nested = post_events_start;
    push_span(
        &mut spans,
        &mut nested,
        "Post-events field 244",
        "u32",
        4,
        0,
    )?;
    push_span(
        &mut spans,
        &mut nested,
        "Post-events field 248",
        "u32",
        4,
        0,
    )?;
    push_records(
        &mut spans,
        &mut nested,
        "Walk milestones",
        24,
        save.post_events.walk_milestones.len(),
    )?;
    push_records(
        &mut spans,
        &mut nested,
        "Walk completions",
        24,
        save.post_events.walk_completions.len(),
    )?;
    push_span(
        &mut spans,
        &mut nested,
        "Recruitable companion world presence",
        "u32[8]",
        8 * 4,
        0,
    )?;
    push_span(
        &mut spans,
        &mut nested,
        "Dismissed companion progression",
        "record[8]",
        24,
        0,
    )?;
    if nested != offset {
        return None;
    }
    let journal_len = 42
        + (save.journal.main.len() + save.journal.side.len() + save.journal.trade.len())
            .checked_mul(37)?;
    let journal_start = offset;
    push_span(
        &mut spans,
        &mut offset,
        "Journal",
        "section",
        journal_len,
        0,
    )?;
    let mut journal_nested = journal_start;
    push_span(
        &mut spans,
        &mut journal_nested,
        "Journal header",
        "42 bytes",
        42,
        0,
    )?;
    for (section_index, name) in [
        "Main journal entry",
        "Side journal entry",
        "Trade journal entry",
    ]
    .iter()
    .enumerate()
    {
        let entries = match section_index {
            0 => &save.journal.main,
            1 => &save.journal.side,
            _ => &save.journal.trade,
        };
        for index in 0..entries.len() {
            add_span(
                &mut spans,
                journal_start + 42 + (section_index * 3_700 + index * 37) as u64,
                37,
                name,
                "record",
                index as u64,
            )?;
        }
    }
    if offset > bytes.len() as u64 {
        return None;
    }
    if offset < bytes.len() as u64 {
        spans.push(NamedSpan {
            range: offset..bytes.len() as u64,
            name: "Trailing data",
            ty: "opaque",
            record_index: 0,
        });
    }
    Some(Box::new(SpanBinaryLayout::new("Save file", spans)))
}

fn push_span(
    spans: &mut Vec<NamedSpan>,
    offset: &mut u64,
    name: &'static str,
    ty: &'static str,
    len: usize,
    record_index: u64,
) -> Option<()> {
    let end = offset.checked_add(u64::try_from(len).ok()?)?;
    spans.push(NamedSpan {
        range: *offset..end,
        name,
        ty,
        record_index,
    });
    *offset = end;
    Some(())
}

fn push_counted_records(
    spans: &mut Vec<NamedSpan>,
    offset: &mut u64,
    name: &'static str,
    size: usize,
    count: usize,
    count_size: usize,
) -> Option<()> {
    push_span(spans, offset, "Record count", "u16/u32", count_size, 0)?;
    push_records(spans, offset, name, size, count)
}

fn push_records(
    spans: &mut Vec<NamedSpan>,
    offset: &mut u64,
    name: &'static str,
    size: usize,
    count: usize,
) -> Option<()> {
    for index in 0..count {
        push_span(spans, offset, name, "record", size, index as u64)?;
    }
    Some(())
}

fn add_span(
    spans: &mut Vec<NamedSpan>,
    start: u64,
    len: usize,
    name: &'static str,
    ty: &'static str,
    record_index: u64,
) -> Option<()> {
    let end = start.checked_add(u64::try_from(len).ok()?)?;
    spans.push(NamedSpan {
        range: start..end,
        name,
        ty,
        record_index,
    });
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_for_path_recognizes_only_monster_db_path() {
        assert!(layout_for_path(Path::new("MonsterInGame/Monster.db"), &[]).is_some());
        assert!(layout_for_path(Path::new("Monster.db"), &[]).is_none());
        assert!(layout_for_path(Path::new("MonsterInGame/Other.db"), &[]).is_none());
    }

    #[test]
    fn test_monster_layout_resolves_second_field_in_first_record() {
        let layout = layout_for_path(Path::new("MonsterInGame/Monster.db"), &[]).unwrap();
        let field = layout.field_at(24, 160).unwrap();
        assert_eq!(field.name, "health_points_max");
        assert_eq!(field.record_index, 0);
        assert_eq!(field.range, 24..28);
    }

    #[test]
    fn test_save_layout_ignores_invalid_save_data() {
        assert!(layout_for_path(Path::new("slot.sav"), &[]).is_none());
    }
}
