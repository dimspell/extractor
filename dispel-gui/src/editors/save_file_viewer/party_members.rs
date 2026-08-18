use crate::editors::save_file_viewer::helpers::{label_row, section_header};
use crate::editors::save_file_viewer::state::SaveFileViewerState;
use crate::message::Message;
use dispel_core::references::save_file::PartyMember;
use iced::Element;
use iced::widget::{Column, container, scrollable, text};

/// Player party member section
pub fn view<'a>(state: &'a SaveFileViewerState) -> Element<'a, Message> {
    let sf = match state.save_file.as_ref() {
        Some(sf) => sf,
        None => return container(text("No save file loaded")).into(),
    };

    let identity = &sf.character_identity;
    let party_members = &sf.character_identity.party_members;

    let character_header = &identity.character_data_header;

    scrollable(
        Column::new()
            .push(section_header("Character Data Header"))
            .push(label_row(
                "unknown_a",
                character_header.unknown_a.to_string(),
            ))
            .push(label_row(
                "unknown_b",
                character_header.unknown_b.to_string(),
            ))
            .push(label_row(
                "unknown_c",
                character_header.unknown_c.to_string(),
            ))
            .push(label_row(
                "unknown_d",
                character_header.unknown_d.to_string(),
            ))
            .push(label_row(
                "unknown_e",
                character_header.unknown_e.to_string(),
            ))
            .push(label_row(
                "unknown_f",
                character_header.unknown_f.to_string(),
            ))
            .push(label_row(
                "Equipment Slots",
                identity.equipped_equipment.len().to_string(),
            ))
            .push(label_row(
                "Belt Potions",
                identity.belt_potions.len().to_string(),
            ))
            .push(label_row(
                "Inventory Placements",
                identity.inventory_placement.len().to_string(),
            ))
            .push(label_row(
                "Learned Spells",
                identity.learned_spells.spells.len().to_string(),
            ))
            .push(section_header("Learned Spells"))
            .push(identity.learned_spells.spells.iter().enumerate().fold(
                Column::new().spacing(4),
                |col, (i, flag)| {
                    col.push(label_row(
                        format!("Spell {:02}", i + 1),
                        if *flag != 0 { "learned" } else { "not learned" },
                    ))
                },
            ))
            .push(section_header("Player Identity"))
            .push(label_row(
                "Party Members Count",
                identity.party_members_count.to_string(),
            ))
            .push(Column::new().spacing(4).push(match party_members.first() {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 1").into(),
            }))
            .push(Column::new().spacing(4).push(match party_members.get(1) {
                Some(member) => party_member_block(member),
                None => text("No party member in slot 2").into(),
            }))
            .spacing(8)
            .padding(16),
    )
    .into()
}

fn party_member_block(member: &PartyMember) -> Element<'static, Message> {
    container(
        Column::new()
            .push(text(member.name.to_string()).size(16))
            .push(label_row("Class ID", member.record.class_id.to_string()))
            .push(label_row(
                "Class Variant",
                member.record.party_class_variant.to_string(),
            ))
            .push(label_row("Level", member.record.level.to_string()))
            .push(label_row(
                "HP",
                format!(
                    "{} / {}",
                    member.record.current_health_points, member.record.maximum_health_points
                ),
            ))
            .push(label_row(
                "MP",
                format!(
                    "{} / {}",
                    member.record.current_mana_points, member.record.maximum_mana_points
                ),
            ))
            .push(label_row("Strength", member.record.strength.to_string()))
            .push(label_row(
                "Constitution",
                member.record.constitution.to_string(),
            ))
            .push(label_row("Wisdom", member.record.wisdom.to_string()))
            .push(label_row("Agility", member.record.agility.to_string()))
            .push(label_row("Attack", member.record.attack.to_string()))
            .push(label_row(
                "Spells",
                format!(
                    "{}, {}, {}",
                    member.record.magic_spell_id_1,
                    member.record.magic_spell_id_2,
                    member.record.magic_spell_id_3
                ),
            ))
            .push(label_row("XP", member.record.experience_points.to_string()))
            .push(label_row(
                "Party Slot",
                member.record.party_slot_index.to_string(),
            ))
            .push(label_row(
                "Tactical Action Chance",
                format!("{}%", member.record.tactical_action_chance),
            ))
            .push(label_row(
                "Map Position",
                format!("{}, {}", member.record.map_x, member.record.map_y),
            ))
            .push(label_row(
                "Movement State",
                member.record.movement_state.to_string(),
            ))
            .push(label_row(
                "Sprite Flip",
                if member.record.sprite_horizontal_flip != 0 {
                    "horizontal"
                } else {
                    "normal"
                },
            ))
            .push(label_row(
                "Path",
                format!(
                    "node {} of {}",
                    member.record.path_node_index, member.record.path_node_count
                ),
            ))
            .push(label_row(
                "Weapon Skill",
                member.record.weapon_skill_level.to_string(),
            ))
            .push(label_row(
                "Facing Direction",
                member.record.facing_direction.to_string(),
            ))
            .push(label_row(
                "Map Occupancy ID",
                member.record.map_occupancy_id.to_string(),
            ))
            .push(label_row(
                "Movement Sprite Direction",
                member.record.movement_sprite_direction.to_string(),
            ))
            .push(label_row(
                "Animation",
                format!(
                    "frame {} ({} ticks)",
                    member.record.animation_frame_index, member.record.animation_tick_count
                ),
            ))
            .push(label_row(
                "Sprite Offset",
                format!(
                    "{}, {}",
                    member.record.sprite_offset_x, member.record.sprite_offset_y
                ),
            ))
            .push(label_row(
                "Follow Target",
                format!(
                    "{}, {}",
                    member.record.follow_target_x, member.record.follow_target_y
                ),
            ))
            .push(label_row(
                "Combat Action",
                member.record.selected_combat_action_id.to_string(),
            ))
            .push(label_row(
                "Map-object Target",
                member.record.selected_map_object_id.to_string(),
            ))
            .push(label_row(
                "Hit Reaction Pending",
                if member.record.hit_animation_pending != 0 {
                    "yes"
                } else {
                    "no"
                },
            ))
            .push(label_row(
                "Automatic Restorations",
                format!(
                    "{} health, {} mana",
                    member.record.automatic_health_restorations_remaining,
                    member.record.automatic_mana_restorations_remaining
                ),
            ))
            .push(label_row(
                "Status Effect",
                format!(
                    "{} ({} ticks; poison tick {}; auxiliary {})",
                    member.record.active_status_effect_id,
                    member.record.status_effect_ticks_remaining,
                    member.record.poison_damage_tick_countdown,
                    member.record.status_effect_auxiliary_value
                ),
            ))
            .push(label_row(
                "Blocked-path Recovery Target",
                format!(
                    "{} attempts toward {}, {}",
                    member.record.blocked_path_reposition_attempts,
                    member.record.blocked_path_target_x,
                    member.record.blocked_path_target_y
                ),
            ))
            .push(label_row(
                "AI Target Search Range",
                member.record.ai_target_search_range.to_string(),
            ))
            .push(label_row(
                "AI Runtime State",
                member.record.ai_runtime_state.to_string(),
            ))
            .push(label_row(
                "Movement Transition",
                format!(
                    "state {}, substate {}, animation phase {}",
                    member.record.movement_transition_state,
                    member.record.movement_transition_substate,
                    member.record.movement_animation_phase
                ),
            ))
            .push(label_row(
                "Combat Action Delay",
                if member.record.combat_action_delay_active != 0 {
                    format!(
                        "{} ticks{}",
                        member.record.combat_action_delay_ticks_remaining,
                        if member.record.combat_action_ready != 0 {
                            " (ready)"
                        } else {
                            ""
                        }
                    )
                } else {
                    "inactive".to_owned()
                },
            ))
            .push(label_row(
                "Combat Action Animation",
                format!(
                    "delay frame {}, resolution frame{}{}",
                    member.record.combat_action_delay_animation_frame,
                    member.record.combat_action_resolution_animation_frame,
                    if member.record.combat_action_completion_latched != 0 {
                        " (complete)"
                    } else {
                        ""
                    }
                ),
            ))
            .push(label_row(
                "Blocked-path Recovery",
                if member.record.blocked_path_recovery_active != 0 {
                    "active"
                } else {
                    "inactive"
                },
            ))
            .push(label_row(
                "Rejoin Leader",
                match (
                    member.record.rejoin_leader_requested != 0,
                    member.record.rejoin_leader_in_progress != 0,
                ) {
                    (_, true) => "in progress",
                    (true, false) => "requested",
                    (false, false) => "inactive",
                },
            ))
            .push(label_row(
                "Level-up",
                if member.record.level_up_pending != 0 {
                    format!(
                        "pending; {} frame {} ({})",
                        if member.record.level_up_animation_active != 0 {
                            "animation"
                        } else {
                            "no animation"
                        },
                        member.record.level_up_animation_frame,
                        member.record.level_up_animation_variant
                    )
                } else {
                    "not pending".to_owned()
                },
            ))
            .push(label_row(
                "Active Path Node",
                format!(
                    "{}, {} (base actor state {})",
                    member.record.active_path_node_x,
                    member.record.active_path_node_y,
                    member.record.base_actor_state
                ),
            ))
            .push(label_row(
                "Base Actor HP",
                format!(
                    "{} / {}",
                    member.record.base_actor_current_health_points,
                    member.record.base_actor_maximum_health_points
                ),
            ))
            .push(label_row(
                "Render Runtime",
                format!(
                    "buffer 0x{:08x}, parameter {}",
                    member.record.last_render_buffer_address, member.record.last_render_parameter
                ),
            ))
            .push(label_row(
                "Combat Snapshot Marker",
                member.record.combat_snapshot_marker.to_string(),
            ))
            .spacing(3),
    )
    .padding(8)
    .into()
}
