use super::character::{
    CHARACTER_DATA_SIZE, CharacterData, LEARNED_SPELL_COUNT, LearnedSpells, SPRITE_PATH_COUNT,
    SPRITE_PATH_SIZE, read_sprite_paths,
};
use super::events::{EVENT_RECORD_SIZE, PostEventsData, read_events};
use super::game_tmp::{EXTRA_OBJECT_TRAILER_RECORD_SIZE, read_maps};
use super::inventory::{INVENTORY_SLOTS_SIZE, InventoryData, InventorySlots};
use super::journal::{JOURNAL_HEADER_SIZE, JournalData};
use super::map_viewport::{MAP_VIEWPORT_STATE_SIZE, MapViewportState, PostMapsData};
use super::{MonsterRecord, PartyMember, SaveFile};
use std::io::Cursor;

#[test]
fn test_parse_save_file_reports_section_and_offset() {
    let error = SaveFile::parse(&0u32.to_le_bytes()).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
    assert!(error.to_string().contains("map count"));
    assert!(error.to_string().contains("byte offset 4"));
}

#[test]
fn test_read_sprite_paths_reads_four_fixed_windows_1250_strings() {
    let mut data = vec![0u8; SPRITE_PATH_COUNT * SPRITE_PATH_SIZE];
    data[..8].copy_from_slice(b"one.spr\0");
    data[SPRITE_PATH_SIZE..SPRITE_PATH_SIZE + 8].copy_from_slice(b"two.spr\0");

    let paths = read_sprite_paths(&mut Cursor::new(data)).unwrap();

    assert_eq!(paths, ["one.spr", "two.spr", "", ""]);
}

#[test]
fn test_read_maps_accepts_empty_map_section() {
    let mut data = Vec::new();
    data.extend_from_slice(&7u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&17u32.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.push(0);
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    for _ in 0..5 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }
    data.extend_from_slice(&0u32.to_le_bytes());

    let maps = read_maps(&mut Cursor::new(data), 1).unwrap();

    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].map_id, 7);
    assert!(maps[0].monsters.is_empty());
    assert!(maps[0].draw_items_weapon.is_empty());
}

#[test]
fn test_parse_monster_record_rejects_truncated_input() {
    let error = MonsterRecord::parse(&[0; 328]).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn test_read_character_data_rejects_truncated_input() {
    let error =
        CharacterData::read_from(&mut Cursor::new(vec![0; CHARACTER_DATA_SIZE - 1])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_learned_spells_preserves_all_flags() {
    let flags: Vec<u8> = (0..LEARNED_SPELL_COUNT as u8).collect();

    let spells = LearnedSpells::read_from(&mut Cursor::new(&flags)).unwrap();

    assert_eq!(spells.spells, flags);
}

#[test]
fn test_read_inventory_accepts_zero_items_in_each_category() {
    let inventory = InventoryData::read_from(&mut Cursor::new([0u8; 10])).unwrap();

    assert!(inventory.event_items.is_empty());
    assert!(inventory.misc_items.is_empty());
    assert!(inventory.edit_items.is_empty());
    assert!(inventory.weapon_items.is_empty());
    assert!(inventory.heal_items.is_empty());
}

#[test]
fn test_read_inventory_reads_one_count_prefixed_event_item() {
    let mut data = Vec::new();
    data.extend_from_slice(&1u16.to_le_bytes());
    data.extend_from_slice(&[0u8; 244]);
    for _ in 0..4 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }

    let inventory = InventoryData::read_from(&mut Cursor::new(data)).unwrap();

    assert_eq!(inventory.event_items.len(), 1);
    assert!(inventory.misc_items.is_empty());
}

#[test]
fn test_read_inventory_slots_rejects_truncated_input() {
    let error =
        InventorySlots::read_from(&mut Cursor::new(vec![0; INVENTORY_SLOTS_SIZE - 1])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_party_member_rejects_truncated_base_record() {
    let error = PartyMember::read_from(&mut Cursor::new(vec![0; 320])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_events_rejects_truncated_record() {
    let error = read_events(&mut Cursor::new(vec![0; EVENT_RECORD_SIZE - 1])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_post_events_accepts_empty_record_collection() {
    let mut data = vec![1u8; 12];
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&[2u8; 56]);

    let post_events = PostEventsData::read_from(&mut Cursor::new(data)).unwrap();

    assert_eq!(post_events.block_a, vec![1; 12]);
    assert!(post_events.records.is_empty());
    assert_eq!(post_events.block_b, vec![2; 56]);
}

#[test]
fn test_read_journal_rejects_truncated_main_section() {
    let error = JournalData::read_from(&mut Cursor::new(vec![0; JOURNAL_HEADER_SIZE])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_map_viewport_rejects_truncated_input() {
    let error = MapViewportState::read_from(&mut Cursor::new(vec![0; MAP_VIEWPORT_STATE_SIZE - 1]))
        .unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_post_maps_rejects_mismatched_duplicate_count() {
    let mut data = vec![0u8; 36];
    data.extend_from_slice(&2u32.to_le_bytes());

    let error = PostMapsData::read_from(&mut Cursor::new(data), 1).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("expected 1"));
}

#[test]
fn test_layout_constants_keep_verified_trailer_record_size() {
    assert_eq!(EXTRA_OBJECT_TRAILER_RECORD_SIZE, 24);
}
