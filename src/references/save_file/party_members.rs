use crate::references::extractor::read_null_terminated_windows_1250;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Fixed 321-byte party-member record as it appears on disk.
///
/// The runtime state contains many unknown bytes between the decoded values.
/// Keep those gaps in the derived record so `BinaryRecord` still consumes the
/// exact layout while the public [`PartyMember`] type exposes only named data.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct PartyMemberBinaryRecord {
    /// Maximum health points from the party-level progression record.
    pub maximum_health_points: u16,
    /// Maximum mana points from the party-level progression record.
    pub maximum_mana_points: u16,
    #[binary_record(size = 2)]
    pub unknown_04: Vec<u8>,
    /// Party character class ID from `PrtIni.db` (21–24 in shipped data).
    pub class_id: u8,
    /// Current progression level.
    pub level: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_08: Vec<u8>,
    /// Class-specific runtime behaviour selected during companion creation.
    pub class_behaviour: u8,
    /// Range used by the companion AI when searching for a combat target.
    pub ai_target_search_range: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 12)]
    pub unknown_12: Vec<u8>,
    /// Inferred AI action-state value saved between target-selection updates.
    pub ai_runtime_state: u32,
    /// Strength from `PrtLevel.db` for this character and level.
    pub strength: u32,
    /// Constitution from `PrtLevel.db` for this character and level.
    pub constitution: u32,
    /// Wisdom from `PrtLevel.db` for this character and level.
    pub wisdom: u32,
    /// Agility from `PrtLevel.db` for this character and level.
    pub agility: u8,
    /// Attack stat from `PrtLevel.db` for this character and level.
    pub attack: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 6)]
    pub unknown_42: Vec<u8>,
    /// First magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_1: u8,
    /// Second magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_2: u8,
    /// Third magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_3: u8,
    #[binary_record(size = 9)]
    // TODO: Recognise the unknown bytes
    pub unknown_51: Vec<u8>,
    /// Zero-based index of this companion in the game's party-character table.
    pub party_character_index: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_61: Vec<u8>,
    /// Visual/class variant selected for this companion from the party-character table.
    pub party_class_variant: u32,
    /// Weapon-skill level from `PrtLevel.db` for this character and level.
    pub weapon_skill_level: u32,
    /// Experience points accumulated by this party member.
    ///
    /// The game increments this value after combat and compares it with the
    /// next-level threshold before levelling up.
    pub experience_points: u32,
    /// Position of this companion in the active two-member party UI.
    pub party_slot_index: u32,
    /// Health points remaining at the time the save was written.
    pub current_health_points: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_82: Vec<u8>,
    /// Mana points remaining at the time the save was written.
    pub current_mana_points: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_86: Vec<u8>,
    /// Horizontal screen-pixel offset used while drawing the companion sprite.
    pub sprite_offset_x: i8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_89: Vec<u8>,
    /// Vertical screen-pixel offset used while drawing the companion sprite.
    pub sprite_offset_y: i8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_93: Vec<u8>,
    /// Current frame within the companion's active animation.
    pub animation_frame_index: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_97: Vec<u8>,
    /// Map occupancy ID written into tiles and supplied to pathfinding for this companion.
    pub map_occupancy_id: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_101: Vec<u8>,
    /// Current facing direction; `-1` means that no directional animation is active.
    pub facing_direction: i8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_105: Vec<u8>,
    /// Inferred movement-transition state, stored beside the direction bytes.
    pub movement_transition_state: u32,
    /// Inferred substate for the current movement transition.
    pub movement_transition_substate: u32,
    /// ID of the map object currently selected as this companion's movement or action target.
    /// A negative value means that no map object is selected.
    pub selected_map_object_id: i16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_118: Vec<u8>,
    /// Inferred phase value for the active movement animation.
    pub movement_animation_phase: u32,
    /// Number of animation frames processed in the current action.
    pub animation_tick_count: u32,
    /// Direction index of the sprite's current movement state.
    pub movement_sprite_direction: u32,
    /// Map-cell X coordinate the companion is currently following.
    pub follow_target_x: i32,
    /// Map-cell Y coordinate the companion is currently following.
    pub follow_target_y: i32,
    /// Whether the companion's one-frame hit reaction still needs to be drawn.
    pub hit_animation_pending: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_141: Vec<u8>,
    /// Current visual frame while the combat action waits for execution.
    pub combat_action_delay_animation_frame: u32,
    /// Whether the selected combat action is waiting for its execution delay (boolean).
    pub combat_action_delay_active: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_149: Vec<u8>,
    /// Ticks remaining before the delayed combat action is executed.
    pub combat_action_delay_ticks_remaining: u32,
    /// Whether a delayed combat action has become ready for execution (boolean).
    pub combat_action_ready: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_157: Vec<u8>,
    /// Current visual frame while the ready combat action is resolved.
    pub combat_action_resolution_animation_frame: u32,
    /// Selected combat action or spell. Negative values are runtime sentinels.
    pub selected_combat_action_id: i32,
    /// Whether the combat action's completion frame has been reached (boolean).
    pub combat_action_completion_latched: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_169: Vec<u8>,
    /// Auxiliary value for a timed status effect; `-1` means inactive.
    ///
    /// Depending on the effect phase, the game uses this as a countdown or a
    /// party-slot value, so it is not consistently an effect source.
    pub status_effect_auxiliary_value: i32,
    /// Remaining automatic full-health restorations available to this companion.
    pub automatic_health_restorations_remaining: u32,
    /// Remaining automatic full-mana restorations available to this companion.
    pub automatic_mana_restorations_remaining: u32,
    /// Active status-effect kind. One denotes poison; zero denotes no active timed effect.
    pub active_status_effect_id: u32,
    /// Ticks remaining before the active status effect is processed or expires.
    pub status_effect_ticks_remaining: u32,
    /// Countdown to the next poison-damage tick while poisoned.
    pub poison_damage_tick_countdown: u32,
    /// Number of attempts made to find a nearby walkable cell when the path is blocked.
    pub blocked_path_reposition_attempts: u32,
    /// X coordinate of the temporary target used by blocked-path recovery.
    pub blocked_path_target_x: i32,
    /// Y coordinate of the temporary target used by blocked-path recovery.
    pub blocked_path_target_y: i32,
    /// Whether the companion is actively recovering from a blocked path (boolean).
    pub blocked_path_recovery_active: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 4)]
    pub unknown_209: Vec<u8>,
    /// Whether the companion has been instructed to rejoin the party leader (boolean).
    pub rejoin_leader_requested: u8,
    /// Whether the companion is currently moving to rejoin the party leader (boolean).
    pub rejoin_leader_in_progress: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 5)]
    pub unknown_215: Vec<u8>,
    /// Whether this companion has earned a level and awaits the level-up sequence (boolean).
    pub level_up_pending: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_221: Vec<u8>,
    /// Whether the level-up animation is currently active (boolean.
    pub level_up_animation_active: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_225: Vec<u8>,
    /// Current frame of the level-up animation.
    pub level_up_animation_frame: u32,
    /// Variant of the level-up animation selected for this companion's class.
    pub level_up_animation_variant: u32,
    /// Last render-buffer address saved by the runtime. It is not stable across sessions.
    pub last_render_buffer_address: u32,
    /// Last render parameter saved by the runtime. It is not stable gameplay data.
    pub last_render_parameter: i32,
    /// Index of the next node in the active movement path.
    pub path_node_index: u32,
    /// Current map-cell X coordinate.
    pub map_x: u16,
    /// Current map-cell Y coordinate.
    pub map_y: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 4)]
    pub unknown_252: Vec<u8>,
    /// Previous map-cell X coordinate, used by movement and formation logic.
    pub previous_map_x: u16,
    /// Previous map-cell Y coordinate, used by movement and formation logic.
    pub previous_map_y: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 4)]
    pub unknown_260: Vec<u8>,
    /// Whether the companion sprite is rendered horizontally flipped (boolean).
    pub sprite_horizontal_flip: u8,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 3)]
    pub unknown_265: Vec<u8>,
    /// Runtime movement state for the companion.
    pub movement_state: u32,
    /// Number of nodes in the active movement path.
    pub path_node_count: u32,
    /// Percentage threshold used after level ten to trigger a tactical action.
    pub tactical_action_chance: u32,
    /// X coordinate of the first node in the saved active-path buffer.
    pub active_path_node_x: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_282: Vec<u8>,
    /// Y coordinate of the first node in the saved active-path buffer.
    pub active_path_node_y: u16,
    // TODO: Recognise the unknown bytes
    #[binary_record(size = 2)]
    pub unknown_286: Vec<u8>,
    /// Base actor lifecycle state saved alongside the active-path buffer.
    pub base_actor_state: u32,
    /// Current health in the inherited base-actor state.
    pub base_actor_current_health_points: u16,
    /// Maximum health in the inherited base-actor state.
    pub base_actor_maximum_health_points: u16,
    /// Marker for an optional combat snapshot appended after the base record.
    pub combat_snapshot_marker: u32,
}

/// Combat-only snapshot appended to a party-member record.
///
/// This 48-byte stream is followed by a four-byte terminator. The game writes
/// it only when the companion has an active combat object.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartyMemberCombatSnapshot {
    /// Current health points in the active combat object.
    pub current_health_points: u16,
    /// Maximum health points in the active combat object.
    pub maximum_health_points: u16,
    /// Agility copied into the active combat object.
    pub agility: u8,
    /// Attack stat copied into the active combat object.
    pub attack: u8,
    /// Strength copied into the active combat object.
    pub strength: u16,
    /// Constitution copied into the active combat object.
    pub constitution: u16,
    /// Wisdom copied into the active combat object.
    pub wisdom: u16,
    /// Class-specific combat AI behaviour.
    pub class_behaviour: u8,
    /// Combat AI target-search range.
    pub ai_target_search_range: u8,
    /// First combat spell ID.
    pub magic_spell_id_1: u8,
    /// Second combat spell ID.
    pub magic_spell_id_2: u8,
    /// Third combat spell ID.
    pub magic_spell_id_3: u8,
    /// Exact 48-byte combat snapshot stream.
    pub serialized_snapshot: Vec<u8>,
    /// Four-byte terminator written after the combat snapshot.
    pub terminator: u32,
}

impl PartyMemberCombatSnapshot {
    pub(crate) const SERIALIZED_SIZE: usize = 48;

    fn parse(data: &[u8], terminator: u32) -> std::io::Result<Self> {
        if data.len() != Self::SERIALIZED_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PartyMember combat snapshot requires 48 bytes",
            ));
        }

        let u16_at = |offset| u16::from_le_bytes([data[offset], data[offset + 1]]);
        Ok(Self {
            current_health_points: u16_at(0),
            maximum_health_points: u16_at(4),
            agility: data[8],
            attack: data[12],
            strength: u16_at(16),
            constitution: u16_at(20),
            wisdom: u16_at(24),
            class_behaviour: data[28],
            ai_target_search_range: data[32],
            magic_spell_id_1: data[36],
            magic_spell_id_2: data[40],
            magic_spell_id_3: data[44],
            serialized_snapshot: data.to_vec(),
            terminator,
        })
    }
}

/// Runtime snapshot of a recruited party character (321 bytes plus an optional combat tail).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartyMember {
    /// Party character display name, stored in a 21-byte Windows-1250 buffer.
    pub name: String,
    // TODO: Rename the field and document it.
    pub record: PartyMemberBinaryRecord,
    /// Combat-only state appended after the base record when present.
    pub combat_snapshot: Option<PartyMemberCombatSnapshot>,
}

impl PartyMember {
    pub(crate) const NAME_SIZE: usize = 21;
    pub(crate) const RUNTIME_STATE_SIZE: usize = 300;

    /// Parse a normal, non-combat 321-byte companion record.
    ///
    /// Use [`Self::read_from`] for a record inside a save stream because it
    /// also consumes the optional combat snapshot.
    pub fn parse(data: &[u8]) -> std::io::Result<Self> {
        if data.len() != Self::NAME_SIZE + Self::RUNTIME_STATE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PartyMember requires 321 bytes",
            ));
        }

        Self::parse_base(data, None)
    }

    /// Read one variable-length companion record from a save stream.
    pub fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut base_data = vec![0u8; Self::NAME_SIZE + Self::RUNTIME_STATE_SIZE];
        reader.read_exact(&mut base_data)?;

        // The last word of the base record is a marker for the optional
        // combat-object snapshot. When set, the writer appends twelve
        // four-byte snapshot windows and a four-byte terminator.
        let marker_offset = Self::NAME_SIZE + Self::RUNTIME_STATE_SIZE - 4;
        let combat_snapshot_marker = u32::from_le_bytes([
            base_data[marker_offset],
            base_data[marker_offset + 1],
            base_data[marker_offset + 2],
            base_data[marker_offset + 3],
        ]);
        let combat_snapshot = if combat_snapshot_marker != 0 {
            let mut snapshot_data = [0u8; PartyMemberCombatSnapshot::SERIALIZED_SIZE];
            reader.read_exact(&mut snapshot_data)?;
            let terminator = reader.read_u32::<LittleEndian>()?;
            Some(PartyMemberCombatSnapshot::parse(
                &snapshot_data,
                terminator,
            )?)
        } else {
            None
        };

        Self::parse_base(&base_data, combat_snapshot)
    }

    fn parse_base(
        data: &[u8],
        combat_snapshot: Option<PartyMemberCombatSnapshot>,
    ) -> std::io::Result<Self> {
        let name = read_null_terminated_windows_1250(&data[0..Self::NAME_SIZE])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let parsed = PartyMemberBinaryRecord::parse(&data[Self::NAME_SIZE..])?;

        Ok(Self {
            name,
            record: parsed,
            combat_snapshot,
        })
    }

    /// Write the original serialized companion-state stream.
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut name_buf = [0u8; Self::NAME_SIZE];
        let (encoded, _, _) = encoding_rs::WINDOWS_1250.encode(&self.name);
        let len = encoded.len().min(Self::NAME_SIZE);
        name_buf[..len].copy_from_slice(&encoded[..len]);
        writer.write_all(&name_buf)?;
        self.record.write(writer)?;
        if let Some(snapshot) = &self.combat_snapshot {
            if snapshot.serialized_snapshot.len() != PartyMemberCombatSnapshot::SERIALIZED_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "PartyMember combat snapshot requires 48 bytes",
                ));
            }
            writer.write_all(&snapshot.serialized_snapshot)?;
            writer.write_u32::<LittleEndian>(snapshot.terminator)?;
        }
        Ok(())
    }
}

pub(super) fn read_party_members<R: Read>(
    reader: &mut R,
    count: u32,
) -> std::io::Result<Vec<PartyMember>> {
    let mut members = Vec::with_capacity(count as usize);
    for _ in 0..count {
        members.push(PartyMember::read_from(reader)?);
    }
    Ok(members)
}

pub(super) fn write_party_members<W: Write>(
    members: &[PartyMember],
    writer: &mut W,
) -> std::io::Result<()> {
    writer.write_u32::<LittleEndian>(members.len() as u32)?;
    for member in members {
        member.write(writer)?;
    }
    Ok(())
}
