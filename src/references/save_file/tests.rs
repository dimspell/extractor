#[cfg(test)]
mod tests {
    use super::*;
    use crate::references::save_file::character::PartyMemberCombatSnapshot;
    use crate::references::save_file::map_viewport::{
        MAP_VIEWPORT_CELL_COUNT, MAP_VIEWPORT_STATE_SIZE,
    };
    use crate::references::save_file::{
        CharacterStatsHeader, ExtraObjectTrailerRecord, MapExtraObjectsTrailer, MonsterRecord,
        NpcRecord, PartyMember,
    };
    use crate::{
        CharacterStats, MapSectionData, MapViewportCell, MapViewportState, PostMapsData, SaveFile,
    };
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Read;

    #[test]
    fn test_monster_record_preserves_verified_329_byte_layout() {
        let mut bytes = [0u8; 329];
        bytes[68..72].copy_from_slice(&0x0102_0304u32.to_le_bytes());
        bytes[72] = 5;
        bytes[73] = 1;
        bytes[74] = 0xfe;
        bytes[75] = 0xfc;
        bytes[76] = 7;
        bytes[77..81].copy_from_slice(&123u32.to_le_bytes());
        bytes[81..85].copy_from_slice(&456u32.to_le_bytes());
        bytes[93] = 1;
        bytes[121..123].copy_from_slice(&10u16.to_le_bytes());
        bytes[125] = 1;
        bytes[173..177].copy_from_slice(&1u32.to_le_bytes());
        bytes[177..181].copy_from_slice(&(-1i32).to_le_bytes());
        bytes[181..185].copy_from_slice(&12_000u32.to_le_bytes());
        bytes[193..197].copy_from_slice(&99u32.to_le_bytes());
        bytes[245] = 9;
        bytes[246] = 5;
        bytes[247] = 1;
        bytes[248..252].copy_from_slice(&123u32.to_le_bytes());
        bytes[252..256].copy_from_slice(&456u32.to_le_bytes());
        bytes[256] = 1;
        bytes[257..329].fill(0xaa);

        let record = MonsterRecord::parse(&bytes).unwrap();

        assert_eq!(record.magic_level, 0x0102_0304);
        assert_eq!(record.patrol_countdown, 5);
        assert_eq!(record.target_position_x, 123);
        assert_eq!(record.target_position_y, 456);
        assert_eq!(record.awake_flag, 1);
        assert_eq!(record.spawn_group_id, 10);
        assert_eq!(record.dead_or_removed_flag, 1);
        assert_eq!(record.force_ai_update, 1);
        assert_eq!(record.drop_all_loot, u32::MAX);
        assert_eq!(record.respawn_timer, 12_000);
        assert_eq!(record.special_attack, 99);
        assert_eq!(record.path_buffer_position_x, 123);
        assert_eq!(record.path_buffer_position_y, 456);
        assert_eq!(record.nested_summon_flag, 1);
        assert_eq!(record.nested_summon_record, vec![0xaa; 72]);

        let mut serialized = Vec::new();
        record.write(&mut serialized).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_npc_record_preserves_verified_349_byte_layout() {
        let mut bytes = [0u8; 349];
        bytes[192] = 9;
        bytes[193..197].copy_from_slice(&42u32.to_le_bytes());
        bytes[197] = 2;
        bytes[198..202].copy_from_slice(&1u32.to_le_bytes());
        bytes[202..206].copy_from_slice(&100u32.to_le_bytes());
        bytes[206..210].copy_from_slice(&200u32.to_le_bytes());
        bytes[210..214].copy_from_slice(&30u32.to_le_bytes());
        bytes[214..218].copy_from_slice(&7u32.to_le_bytes());
        bytes[294..298].copy_from_slice(&10u32.to_le_bytes());
        bytes[298..302].copy_from_slice(&20u32.to_le_bytes());
        bytes[302..306].copy_from_slice(&30u32.to_le_bytes());
        bytes[306..310].copy_from_slice(&40u32.to_le_bytes());
        bytes[310] = 1;
        bytes[311..315].copy_from_slice(&0x0010_0401u32.to_le_bytes());
        bytes[315] = 11;
        bytes[316..320].copy_from_slice(&81u32.to_le_bytes());
        bytes[320] = 6;
        bytes[321..325].copy_from_slice(&1u32.to_le_bytes());
        bytes[329..333].copy_from_slice(&300u32.to_le_bytes());
        bytes[333..337].copy_from_slice(&400u32.to_le_bytes());
        bytes[341..345].copy_from_slice(&1u32.to_le_bytes());
        bytes[345..349].copy_from_slice(&99u32.to_le_bytes());

        let record = NpcRecord::parse(&bytes).unwrap();

        assert_eq!(record.npc_ref_party_member_slot, 9);
        assert_eq!(record.npc_ref_show_on_event_id, 42);
        assert_eq!(record.npc_ref_movement_mode, 2);
        assert_eq!(record.waypoint1_wait_time, 30);
        assert_eq!(record.waypoint1_facing_direction, 7);
        assert_eq!(record.activation_rect_x1, 10);
        assert_eq!(record.activation_rect_y2, 40);
        assert_eq!(record.npc_ref_interaction_mode, 1);
        assert_eq!(record.npc_ref_interaction_result, 0x0010_0401);
        assert_eq!(record.npc_ref_interaction_range, 11);
        assert_eq!(record.npc_ref_dialog_id, 81);
        assert_eq!(record.dialogue_face_sprite_id, 6);
        assert_eq!(record.move_mode, 1);
        assert_eq!(record.runtime_target_position_x, 300);
        assert_eq!(record.runtime_target_position_y, 400);
        assert_eq!(record.freeze_flag, 1);
        assert_eq!(record.freeze_counter, 99);

        let mut serialized = Vec::new();
        record.write(&mut serialized).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn test_write_post_maps_data_matches_recognized_header_layout() {
        let post_maps = PostMapsData {
            map_section_terminator: 0,
            game_version: 1.5,
            unknown_header_value_1: 1,
            all_map_ini_id: 2,
            ref_map_ini_id: 3,
            monster_block_size: 5,
            npc_block_size: 6,
            unknown_header_value_2: 7,
            extra_object_block_size: 8,
            number_of_visited_maps: 2,
            map_ids: vec![9, 10],
            map_viewport_state: MapViewportState {
                render_bounds: [0x0b0b_0b0b; 4],
                viewport_bounds: [0x0b0b_0b0b; 4],
                geometry: [0x0b0b_0b0b; 24],
                cells: vec![
                    MapViewportCell {
                        screen_x: 0x0b0b_0b0b,
                        screen_y: 0x0b0b_0b0b,
                        map_x: 0x0b0b_0b0b,
                        map_y: 0x0b0b_0b0b,
                        map_tile_index: 0x0b0b_0b0b,
                    };
                    MAP_VIEWPORT_CELL_COUNT
                ],
                selected_tile_index: 0x0b0b_0b0b,
                renderer_global_state: [0x0b0b_0b0b; 2],
                runtime_state: [0x0b0b_0b0b; 2],
            },
        };
        let mut bytes = Vec::new();

        SaveFile::write_post_maps_data(&post_maps, &mut bytes).unwrap();

        let mut reader = std::io::Cursor::new(bytes);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 0);
        assert_eq!(reader.read_f32::<LittleEndian>().unwrap(), 1.5);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 1);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 3);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 5);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 6);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 7);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 8);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 2);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 9);
        assert_eq!(reader.read_u32::<LittleEndian>().unwrap(), 10);
        let mut map_viewport_state = vec![0u8; MAP_VIEWPORT_STATE_SIZE];
        reader.read_exact(&mut map_viewport_state).unwrap();
        assert_eq!(map_viewport_state, vec![11; MAP_VIEWPORT_STATE_SIZE]);
    }

    #[test]
    fn test_map_viewport_state_round_trips_documented_layout() {
        let mut state = MapViewportState {
            render_bounds: [1, 2, 3, 4],
            viewport_bounds: [5, 6, 7, 8],
            geometry: std::array::from_fn(|index| 100 + index as u32),
            ..Default::default()
        };
        state.cells[0] = MapViewportCell {
            screen_x: 10,
            screen_y: 20,
            map_x: 30,
            map_y: 40,
            map_tile_index: 50,
        };
        state.cells[MAP_VIEWPORT_CELL_COUNT - 1] = MapViewportCell {
            screen_x: 60,
            screen_y: 70,
            map_x: 80,
            map_y: 90,
            map_tile_index: 100,
        };
        state.selected_tile_index = u32::MAX;
        state.renderer_global_state = [101, 102];
        state.runtime_state = [103, 104];

        let mut bytes = Vec::new();
        state.write_to(&mut bytes).unwrap();

        assert_eq!(bytes.len(), MAP_VIEWPORT_STATE_SIZE);
        assert_eq!(
            &bytes[0..16],
            &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );
        assert_eq!(u32::from_le_bytes(bytes[128..132].try_into().unwrap()), 10);
        assert_eq!(u32::from_le_bytes(bytes[144..148].try_into().unwrap()), 50);
        assert_eq!(
            u32::from_le_bytes(bytes[10_108..10_112].try_into().unwrap()),
            60
        );
        assert_eq!(
            u32::from_le_bytes(bytes[10_128..10_132].try_into().unwrap()),
            u32::MAX
        );
        assert_eq!(
            u32::from_le_bytes(bytes[10_132..10_136].try_into().unwrap()),
            101
        );
        assert_eq!(
            u32::from_le_bytes(bytes[10_144..10_148].try_into().unwrap()),
            104
        );

        let parsed = MapViewportState::read_from(&mut std::io::Cursor::new(&bytes)).unwrap();
        assert_eq!(parsed.cells[0].map_tile_index, 50);
        assert_eq!(parsed.cells[MAP_VIEWPORT_CELL_COUNT - 1].map_y, 90);
        assert_eq!(parsed.renderer_global_state, [101, 102]);
        assert_eq!(parsed.runtime_state, [103, 104]);
    }

    #[test]
    fn test_parse_viewport_state_from_fixture() {
        let original = std::fs::read("fixtures/Dispel/0.sav").unwrap();
        let game_tmp_size = u32::from_le_bytes(original[0..4].try_into().unwrap()) as usize;
        let metadata_start = game_tmp_size + 4;
        let map_id_count = u32::from_le_bytes(
            original[metadata_start + 32..metadata_start + 36]
                .try_into()
                .unwrap(),
        ) as usize;
        let viewport_start = metadata_start + 36 + map_id_count * 4;
        let viewport_end = viewport_start + MAP_VIEWPORT_STATE_SIZE;
        let state = MapViewportState::read_from(&mut std::io::Cursor::new(
            &original[viewport_start..viewport_end],
        ))
        .unwrap();

        assert_eq!(state.raw_bytes(), original[viewport_start..viewport_end]);
        assert_eq!(state.cells.len(), MAP_VIEWPORT_CELL_COUNT);
        assert_eq!(state.cells[0].map_tile_index, 16_777);
    }

    #[test]
    fn test_write_character_stats_preserves_position_and_surrounding_blocks() {
        let mut bytes = Vec::new();
        let header = CharacterStatsHeader {
            unknown_a: 2,
            unknown_b: 3,
            selected_spell_id: 4,
            unknown_block: [5; 19],
        };

        SaveFile::write_character_stats(
            &[1; 8],
            -123,
            456,
            &header,
            &CharacterStats::default(),
            &[3; 9],
            &mut bytes,
        )
        .unwrap();

        assert_eq!(&bytes[..8], &[1; 8]);
        let mut reader = std::io::Cursor::new(&bytes[8..12]);
        assert_eq!(reader.read_i16::<LittleEndian>().unwrap(), -123);
        assert_eq!(reader.read_i16::<LittleEndian>().unwrap(), 456);
        assert_eq!(bytes[12], 2);
        assert_eq!(u32::from_le_bytes(bytes[13..17].try_into().unwrap()), 3);
        assert_eq!(u32::from_le_bytes(bytes[17..21].try_into().unwrap()), 4);
        assert_eq!(&bytes[21..40], &[5; 19]);
        assert_eq!(&bytes[bytes.len() - 9..], &[3; 9]);

        let (_, position_x, position_y, parsed_header, _, _) =
            SaveFile::parse_character_stats(&mut std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(position_x, -123);
        assert_eq!(position_y, 456);
        assert_eq!(parsed_header.selected_spell_id, header.selected_spell_id);
        assert_eq!(parsed_header.unknown_block, header.unknown_block);
    }

    #[test]
    fn test_maps_section_round_trips_extra_object_trailer() {
        assert_eq!(ExtraObjectTrailerRecord::record_size(), 24);
        let map = MapSectionData {
            map_id: 42,
            extra_objects_trailer: MapExtraObjectsTrailer {
                tail_size: 65,
                records: vec![
                    ExtraObjectTrailerRecord {
                        item_category: 4,
                        reserved_1: 0x80,
                        global_item_index: 780,
                        placement_attempt_count: 0,
                        placement_attempt_limit: 3,
                        unknown_6_7: [0xAA, 0xBB],
                        category_item_index: 7,
                        source_entity_id: 631,
                        unknown_14_15: [0xCC, 0xDD],
                        map_x: -1120,
                        map_y: -80,
                    },
                    ExtraObjectTrailerRecord {
                        item_category: 1,
                        ..Default::default()
                    },
                ],
                automatic_placement_active: 0,
                automatic_placement_value: 773,
                automatic_placement_global_item_index: 780,
            },
            ..Default::default()
        };
        let mut bytes = Vec::new();

        SaveFile::write_maps_section(&[map], &mut bytes).unwrap();
        let parsed = SaveFile::parse_maps_section(&mut std::io::Cursor::new(bytes), 1).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].extra_objects_trailer.tail_size, 65);
        assert_eq!(parsed[0].extra_objects_trailer.records.len(), 2);
        assert_eq!(parsed[0].extra_objects_trailer.records[0].item_category, 4);
        assert_eq!(
            parsed[0].extra_objects_trailer.records[0].global_item_index,
            780
        );
        assert_eq!(
            parsed[0].extra_objects_trailer.records[0].placement_attempt_limit,
            3
        );
        assert_eq!(parsed[0].extra_objects_trailer.records[0].map_x, -1120);
        assert_eq!(
            parsed[0].extra_objects_trailer.automatic_placement_value,
            773
        );
        assert_eq!(
            parsed[0]
                .extra_objects_trailer
                .automatic_placement_global_item_index,
            780
        );
    }

    #[test]
    fn test_party_member_round_trips_combat_snapshot_tail() {
        let mut base = vec![0u8; PartyMember::NAME_SIZE + PartyMember::RUNTIME_STATE_SIZE];
        base[..4].copy_from_slice(b"Test");
        let state_start = PartyMember::NAME_SIZE;
        base[state_start + 196..state_start + 200].copy_from_slice(&3u32.to_le_bytes());
        base[state_start + 200..state_start + 204].copy_from_slice(&44i32.to_le_bytes());
        base[state_start + 204..state_start + 208].copy_from_slice(&55i32.to_le_bytes());
        base[state_start + 208] = 1;
        base[state_start + 264] = 1;
        base[state_start + 296] = 1;

        let mut snapshot = [0u8; PartyMemberCombatSnapshot::SERIALIZED_SIZE];
        snapshot[0..2].copy_from_slice(&37u16.to_le_bytes());
        snapshot[4..6].copy_from_slice(&55u16.to_le_bytes());
        snapshot[8] = 14;
        snapshot[12] = 9;
        snapshot[16..18].copy_from_slice(&21u16.to_le_bytes());
        snapshot[20..22].copy_from_slice(&18u16.to_le_bytes());
        snapshot[24..26].copy_from_slice(&12u16.to_le_bytes());
        snapshot[28] = 5;
        snapshot[32] = 7;
        snapshot[36] = 1;
        snapshot[40] = 2;
        snapshot[44] = 3;

        let terminator: u32 = 0xdec0_adde;
        let mut bytes = base;
        bytes.extend_from_slice(&snapshot);
        bytes.extend_from_slice(&terminator.to_le_bytes());

        let member = PartyMember::read_from(&mut std::io::Cursor::new(&bytes)).unwrap();
        let combat = member.combat_snapshot.as_ref().unwrap();
        assert_eq!(combat.current_health_points, 37);
        assert_eq!(combat.maximum_health_points, 55);
        assert_eq!(combat.strength, 21);
        assert_eq!(combat.magic_spell_id_3, 3);
        assert_eq!(combat.terminator, terminator);
        assert_eq!(member.blocked_path_reposition_attempts, 3);
        assert_eq!(member.blocked_path_target_x, 44);
        assert_eq!(member.blocked_path_target_y, 55);
        assert!(member.blocked_path_recovery_active);
        assert!(member.sprite_horizontal_flip);

        let mut written = Vec::new();
        member.write(&mut written).unwrap();
        assert_eq!(written, bytes);
    }
}
