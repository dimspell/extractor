use super::character::{
    CHARACTER_DATA_SIZE, CharacterData, LEARNED_SPELL_COUNT, LearnedSpells, SPRITE_PATH_COUNT,
    SPRITE_PATH_SIZE, read_sprite_paths, write_sprite_paths,
};
use super::events::{
    DismissedCompanionProgression, EVENT_RECORD_SIZE, PostEventsData, read_events,
};
use super::game_tmp::{EXTRA_OBJECT_TRAILER_RECORD_SIZE, read_maps, write_maps};
use super::inventory::{INVENTORY_SLOTS_SIZE, InventoryData, InventorySlots};
use super::journal::{JOURNAL_HEADER_SIZE, JournalData};
use super::map_viewport::{MAP_VIEWPORT_STATE_SIZE, MapViewportState, PostMapsData};
use super::{EventRecord, JournalEntry, MapSectionData, MonsterRecord, PartyMember, SaveFile};
use crate::references::extractor::Extractor;
use std::io::{Cursor, Write};

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
fn test_write_maps_round_trips_empty_map_section() {
    let map = MapSectionData {
        map_id: 7,
        extra_objects_trailer: super::MapExtraObjectsTrailer {
            tail_size: 17,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut output = Vec::new();

    write_maps(&[map], &mut output).unwrap();
    let parsed = read_maps(&mut Cursor::new(output), 1).unwrap();

    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].map_id, 7);
}

#[test]
fn test_parse_monster_record_rejects_truncated_input() {
    let error = MonsterRecord::parse(&[0; 328]).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn test_monster_record_decodes_runtime_subsystems_at_exact_offsets() {
    let mut bytes = vec![0u8; 329];
    bytes[85..89].copy_from_slice(&30u32.to_le_bytes());
    bytes[89..93].copy_from_slice(&3u32.to_le_bytes());
    bytes[94..98].copy_from_slice(&42u32.to_le_bytes());
    bytes[102..106].copy_from_slice(&7i32.to_le_bytes());
    bytes[118..121].copy_from_slice(&[1, 0x20, 0x10]);
    bytes[124] = 4;
    bytes[126] = 1;
    bytes[127..131].copy_from_slice(&8u32.to_le_bytes());
    bytes[153..157].copy_from_slice(&5u32.to_le_bytes());
    bytes[157..161].copy_from_slice(&2u32.to_le_bytes());
    bytes[205..209].copy_from_slice(&45u32.to_le_bytes());
    bytes[209..213].copy_from_slice(&6u32.to_le_bytes());
    bytes[222] = 1;
    bytes[223..227].copy_from_slice(&3u32.to_le_bytes());
    bytes[227] = 1;
    bytes[228..232].copy_from_slice(&2u32.to_le_bytes());
    bytes[232..236].copy_from_slice(&7u32.to_le_bytes());

    let parsed = MonsterRecord::parse(&bytes).unwrap();

    assert_eq!(parsed.status_effect_ticks_remaining, 30);
    assert_eq!(parsed.status_effect_type, 3);
    assert_eq!(parsed.combat_target_entity_index, 42);
    assert_eq!(parsed.status_effect_parameter, 7);
    assert_eq!(parsed.render_direction_flag, 1);
    assert_eq!(parsed.cell_offset_x, 0x20);
    assert_eq!(parsed.cell_offset_y, 0x10);
    assert_eq!(parsed.movement_animation_frame, 4);
    assert_eq!(parsed.sprite_render_override_pending, 1);
    assert_eq!(parsed.movement_animation_frame_count, 8);
    assert_eq!(parsed.path_buffer_length, 5);
    assert_eq!(parsed.path_buffer_index, 2);
    assert_eq!(parsed.special_attack_effect_ticks_remaining, 45);
    assert_eq!(parsed.special_attack_effect_frame, 6);
    assert_eq!(parsed.guard_effect_active, 1);
    assert_eq!(parsed.guard_effect_frame, 3);
    assert_eq!(parsed.blood_effect_active, 1);
    assert_eq!(parsed.blood_effect_frame, 2);
    assert_eq!(parsed.blood_effect_direction, 7);
    let mut output = Vec::new();
    parsed.write(&mut output).unwrap();
    assert_eq!(output, bytes);
}

#[test]
fn test_ground_item_records_decode_trailing_object_id_and_padding() {
    macro_rules! assert_trailing_id {
        ($record:ty, $size:expr) => {{
            let mut bytes = vec![0u8; $size];
            bytes[$size - 4..].copy_from_slice(&[0x34, 0x12, 0xaa, 0xbb]);

            let parsed = <$record>::parse(&bytes).unwrap();

            assert_eq!(parsed.ground_item_object_id, 0x1234);
            assert_eq!(parsed.ground_item_object_id_padding, [0xaa, 0xbb]);
            let mut output = Vec::new();
            parsed.write(&mut output).unwrap();
            assert_eq!(output, bytes);
        }};
    }

    assert_trailing_id!(super::DrawItemWeaponItem, 296);
    assert_trailing_id!(super::DrawItemHealItem, 264);
    assert_trailing_id!(super::DrawItemEditItem, 280);
    assert_trailing_id!(super::DrawItemMiscItem, 268);
    assert_trailing_id!(super::DrawItemEventItem, 252);
}

#[test]
fn test_npc_record_decodes_path_step_animation_state_at_exact_offsets() {
    let mut bytes = vec![0u8; 349];
    bytes[161] = 7;
    bytes[162] = 3;
    bytes[174..180].copy_from_slice(&[1, 1, 1, 1, 2, 1]);
    bytes[184..188].copy_from_slice(&0x1122_3344u32.to_le_bytes());
    bytes[188..192].copy_from_slice(&0x5566_7788u32.to_le_bytes());
    bytes[325..329].copy_from_slice(&1u32.to_le_bytes());
    bytes[337..341].copy_from_slice(&4321u32.to_le_bytes());

    let parsed = super::NpcRecord::parse(&bytes).unwrap();

    assert_eq!(parsed.path_step_direction, 7);
    assert_eq!(parsed.path_step_animation_frame, 3);
    assert_eq!(parsed.world_active, 1);
    assert_eq!(parsed.transient_spawn, 1);
    assert_eq!(parsed.removed_from_world, 1);
    assert_eq!(parsed.event_npc_origin, 1);
    assert_eq!(parsed.current_waypoint_index, 2);
    assert_eq!(parsed.player_interaction_latched, 1);
    assert_eq!(parsed.reserved_runtime_90, 0x1122_3344);
    assert_eq!(parsed.reserved_runtime_94, 0x5566_7788);
    assert_eq!(parsed.start_dialogue_on_arrival, 1);
    assert_eq!(parsed.arrival_dialogue_id, 4321);
    let mut output = Vec::new();
    parsed.write(&mut output).unwrap();
    assert_eq!(output, bytes);
}

#[test]
fn test_extra_object_decodes_runtime_tail_at_exact_offsets() {
    let mut bytes = vec![0u8; 200];
    bytes[184] = 10;
    bytes[185] = 0xa5;
    bytes[188..192].copy_from_slice(&1u32.to_le_bytes());
    bytes[192..196].copy_from_slice(&1u32.to_le_bytes());
    bytes[196..200].copy_from_slice(&1u32.to_le_bytes());

    let parsed = super::ExtraObjectRecord::parse(&bytes).unwrap();

    assert_eq!(parsed.activation_effect_id, 10);
    assert_eq!(parsed.activation_effect_reserved, 0xa5);
    assert_eq!(parsed.active_overlay_enabled, 1);
    assert_eq!(parsed.map_object_active, 1);
    assert_eq!(parsed.interaction_pending, 1);
    let mut output = Vec::new();
    parsed.write(&mut output).unwrap();
    assert_eq!(output, bytes);
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
fn test_write_sprite_paths_round_trips_fixed_width_strings() {
    let paths = vec![
        "one.spr".into(),
        "two.spr".into(),
        String::new(),
        String::new(),
    ];
    let mut output = Vec::new();

    write_sprite_paths(&paths, &mut output).unwrap();
    let parsed = read_sprite_paths(&mut Cursor::new(output)).unwrap();

    assert_eq!(parsed, paths);
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
fn test_write_inventory_round_trips_one_count_prefixed_event_item() {
    let inventory = InventoryData {
        event_items: vec![super::InventoryEventItem::default()],
        ..Default::default()
    };
    let mut output = Vec::new();

    inventory.write_to(&mut output).unwrap();
    let parsed = InventoryData::read_from(&mut Cursor::new(output)).unwrap();

    assert_eq!(parsed.event_items.len(), 1);
    assert!(parsed.misc_items.is_empty());
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
fn test_party_member_snapshot_presence_uses_first_marker_byte() {
    let mut data = vec![0u8; PartyMember::NAME_SIZE + PartyMember::RUNTIME_STATE_SIZE];
    let marker_offset = data.len() - 4;
    data[marker_offset..].copy_from_slice(&0x0000_0100u32.to_le_bytes());
    let mut reader = Cursor::new(data);

    let member = PartyMember::read_from(&mut reader).unwrap();

    assert!(member.combat_snapshot.is_none());
    assert_eq!(reader.position(), 321);
}

#[test]
fn test_party_member_write_rejects_snapshot_presence_mismatch() {
    let mut member = PartyMember::default();
    member.record.combat_snapshot_presence = 1;

    let error = member.write(&mut Vec::new()).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("presence byte"));
}

#[test]
fn test_read_events_rejects_truncated_record() {
    let error = read_events(&mut Cursor::new(vec![0; EVENT_RECORD_SIZE - 1])).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

#[test]
fn test_read_post_events_accepts_empty_record_collection() {
    // Layout: shake_active (4) + shake_frames_remaining (4) + collA count (4) + collB count (4)
    // + 8 party-member flags (32) + status block (24).
    let mut data = Vec::new();
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes()); // walk milestones count
    data.extend_from_slice(&0u32.to_le_bytes()); // walk completions count
    data.extend_from_slice(&[3u8; 32]); // party-member flags
    data.extend_from_slice(&[4u8; 24]); // status block

    let post_events = PostEventsData::read_from(&mut Cursor::new(data)).unwrap();

    assert_eq!(post_events.shake_active, 1);
    assert_eq!(post_events.shake_frames_remaining, 2);
    assert!(post_events.walk_milestones.is_empty());
    assert!(post_events.walk_completions.is_empty());
    assert_eq!(
        post_events.recruitable_companion_world_presence,
        [0x03030303; 8]
    );
    assert_eq!(
        post_events.dismissed_companion_progression,
        [DismissedCompanionProgression {
            is_saved: 4,
            companion_level: 4,
            player_level: 4,
        }; 8]
    );
}

#[test]
fn test_write_post_events_round_trips_empty_record_collection() {
    let post_events = PostEventsData {
        shake_active: 1,
        shake_frames_remaining: 2,
        walk_milestones: Vec::new(),
        walk_completions: Vec::new(),
        recruitable_companion_world_presence: [3; 8],
        dismissed_companion_progression: [DismissedCompanionProgression {
            is_saved: 4,
            companion_level: 5,
            player_level: 6,
        }; 8],
    };
    let mut output = Vec::new();

    post_events.write_to(&mut output).unwrap();
    let parsed = PostEventsData::read_from(&mut Cursor::new(output)).unwrap();

    assert_eq!(parsed.shake_active, post_events.shake_active);
    assert_eq!(
        parsed.shake_frames_remaining,
        post_events.shake_frames_remaining
    );
    assert_eq!(parsed.walk_milestones, post_events.walk_milestones);
    assert_eq!(parsed.walk_completions, post_events.walk_completions);
    assert_eq!(
        parsed.recruitable_companion_world_presence,
        post_events.recruitable_companion_world_presence
    );
    assert_eq!(
        parsed.dismissed_companion_progression,
        post_events.dismissed_companion_progression
    );
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
fn test_write_post_maps_round_trips_header_and_map_ids() {
    let post_maps = PostMapsData {
        game_version: 1.45,
        number_of_visited_maps: 2,
        map_ids: vec![7, 9],
        ..Default::default()
    };
    let mut output = Vec::new();

    post_maps.write_to(&mut output).unwrap();
    let parsed = PostMapsData::read_from(&mut Cursor::new(output), 2).unwrap();

    assert_eq!(parsed.game_version, post_maps.game_version);
    assert_eq!(parsed.map_ids, post_maps.map_ids);
}

#[test]
fn test_layout_constants_keep_verified_trailer_record_size() {
    assert_eq!(EXTRA_OBJECT_TRAILER_RECORD_SIZE, 24);
}

#[test]
fn test_write_save_file_invalid_model_emits_no_bytes() {
    let mut output = Vec::new();

    let error = SaveFile::default().write_to(&mut output).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("sprite paths"));
    assert!(output.is_empty());
}

#[test]
fn test_write_save_file_and_extractor_emit_identical_bytes() {
    let save = valid_save();
    let mut direct = Vec::new();
    let mut through_trait = Vec::new();

    save.write_to(&mut direct).unwrap();
    SaveFile::to_writer(&[save], &mut through_trait).unwrap();

    assert_eq!(direct, through_trait);
}

#[test]
fn test_write_save_file_validates_fixed_collection_lengths_before_output() {
    let cases = [
        (
            "sprite paths",
            invalid_save(|save| {
                save.sprite_paths.pop();
            }),
        ),
        (
            "map viewport",
            invalid_save(|save| {
                save.map_viewport_state.cells.pop();
            }),
        ),
        (
            "learned spells",
            invalid_save(|save| {
                save.learned_spells.spells.pop();
            }),
        ),
        (
            "events",
            invalid_save(|save| {
                save.events.pop();
            }),
        ),
        (
            "journal",
            invalid_save(|save| {
                save.journal.main.pop();
            }),
        ),
    ];

    for (section, save) in cases {
        assert_preflight_rejection(save, section);
    }
}

#[test]
fn test_write_save_file_validates_cross_field_counts_before_output() {
    let cases = [
        (
            "post-maps",
            invalid_save(|save| save.post_maps.number_of_visited_maps = 1),
        ),
        (
            "party members",
            invalid_save(|save| save.party_members_count = 1),
        ),
        (
            "maps",
            invalid_save(|save| save.maps.push(MapSectionData::default())),
        ),
    ];

    for (section, save) in cases {
        assert_preflight_rejection(save, section);
    }
}

#[test]
fn test_write_count_validation_rejects_values_above_wire_limits() {
    let u16_error =
        super::writer::checked_u16("inventory", "item count", usize::from(u16::MAX) + 1)
            .unwrap_err();

    assert_eq!(u16_error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(u16_error.to_string().contains("inventory"));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_write_count_validation_rejects_u32_overflow() {
    let error = super::writer::checked_u32("maps", "map count", u32::MAX as usize + 1).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("maps"));
}

#[test]
fn test_write_save_file_reports_section_and_output_offset() {
    let save = valid_save();
    let mut writer = FailAfter::new(10);

    let error = save.write_to(&mut writer).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::Other);
    assert!(error.to_string().contains("post-maps"));
    assert!(error.to_string().contains("byte offset 10"));
}

fn valid_save() -> SaveFile {
    SaveFile {
        sprite_paths: vec![String::new(); 4],
        learned_spells: super::LearnedSpells {
            spells: vec![0; 41],
        },
        events: vec![EventRecord::default(); 2_251],
        post_events: PostEventsData {
            shake_active: 0,
            shake_frames_remaining: 0,
            walk_milestones: Vec::new(),
            walk_completions: Vec::new(),
            recruitable_companion_world_presence: [0; 8],
            dismissed_companion_progression: [DismissedCompanionProgression::default(); 8],
        },
        journal: JournalData {
            main: vec![JournalEntry::default(); 100],
            side: vec![JournalEntry::default(); 100],
            trade: vec![JournalEntry::default(); 100],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn invalid_save(change: impl FnOnce(&mut SaveFile)) -> SaveFile {
    let mut save = valid_save();
    change(&mut save);
    save
}

fn assert_preflight_rejection(save: SaveFile, section: &str) {
    let mut output = Vec::new();
    let error = save.write_to(&mut output).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains(section), "{error}");
    assert!(output.is_empty());
}

struct FailAfter {
    remaining: usize,
}

impl FailAfter {
    fn new(remaining: usize) -> Self {
        Self { remaining }
    }
}

impl Write for FailAfter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self.remaining == 0 {
            return Err(std::io::Error::other("injected write failure"));
        }
        let written = buffer.len().min(self.remaining);
        self.remaining -= written;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
