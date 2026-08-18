use crate::references::extractor::read_null_terminated_windows_1250;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Parsed character stats from a save file.
///
/// Maps the binary stats block (~68 bytes of structured data) that follows
/// the belt-data section and precedes the inventory section.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStats {
    // ── Core attributes ──
    pub strength: u16,
    pub agility: u16,
    pub wisdom: u16,
    pub constitution: u16,
    pub morale: u16,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub experience: u32,
    pub level: u16,
    pub gold: u32,
    // ── Combat stats ──
    pub offense: u16,
    pub defense: u16,
    pub dodge_rate: u8,
    pub hit_rate: u8,
    pub magic_power: u16,
    pub attack_modifier: u8,
    // ── Skills (5 × u8) ──
    pub pickpocketing: u8,
    pub lockpicking: u8,
    pub haggling: u8,
    pub perception: u8,
    pub traps: u8,
    // ── Weapon skills (7 types × {level: u8, kills: u16}) ──
    pub swords_level: u8,
    pub swords_kills: u16,
    pub axes_level: u8,
    pub axes_kills: u16,
    pub archery_level: u8,
    pub archery_kills: u16,
    pub polearm_level: u8,
    pub polearm_kills: u16,
    pub magic_level: u8,
    pub magic_kills: u16,
    pub holy_magic_level: u8,
    pub holy_magic_kills: u16,
    pub dark_magic_level: u8,
    pub dark_magic_kills: u16,
}

/// Data immediately before the character stats block (28 bytes).
///
/// Layout: `[unknown_a: u8][unknown_b: u32][selected_spell_id: u32][unknown_block: 19 bytes]`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStatsHeader {
    pub unknown_a: u8,
    pub unknown_b: u32,
    /// ID of the spell currently selected by the player.
    pub selected_spell_id: u32,
    /// Remaining unknown bytes in the header.
    pub unknown_block: [u8; 19],
}

/// Character data header block (11 bytes).
///
/// Read immediately after the player class name and before the
/// equipment/belt/inventory/spells blocks. Internal field meanings are
/// not yet decoded.
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterDataHeader {
    pub unknown_a: u32,
    pub unknown_b: u16,
    pub unknown_c: u16,
    pub unknown_d: u8,
    pub unknown_e: u8,
    pub unknown_f: u8,
}

/// One equipped weapon-item reference (9 bytes).
///
/// Part of the 12-slot equipment array (12 × 9 = 108 bytes total).
/// An empty entry has catalog index `100` and panel marker `0xff`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquipmentSlot {
    /// Equipment-panel marker used by the game to restore this slot's UI state; `0xff` is empty.
    pub panel_slot_marker: u8,
    /// Zero-based index in the weapon-item catalog; `100` is empty.
    pub weapon_catalog_index: i32,
    /// `InventoryWeaponItem::inventory_instance_id` of the equipped weapon; zero is empty.
    pub weapon_inventory_instance_id: i32,
}

/// One belt item placement cell (16 bytes).
///
/// Part of the 6-cell belt array (6 × 16 = 96 bytes total). Larger items can
/// occupy consecutive cells with the same catalog index and icon position.
/// Empty cells use category `10` and catalog index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeltPotionSlot {
    /// Item category; the belt uses category `1` for an occupied item.
    pub item_category: i32,
    /// Zero-based index in that category's catalog; `100` is empty.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the belt icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the belt icon is drawn.
    pub icon_y: i32,
}

/// One item reference and its position in the inventory placement grid (20 bytes).
///
/// The grid is serialized as three pages, each with seven 9-cell columns:
/// `[3 pages][7 columns][9 cells]`. Empty cells use category `10` and catalog
/// index `100`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryPlacementEntry {
    /// Zero-based item category used to select an item collection; `10` marks an empty cell.
    pub item_category: i32,
    /// Zero-based index of the item's definition within `item_category`; `100` marks an empty cell.
    pub item_catalog_index: i32,
    /// Horizontal pixel coordinate at which the inventory icon is drawn.
    pub icon_x: i32,
    /// Vertical pixel coordinate at which the inventory icon is drawn.
    pub icon_y: i32,
    /// Category-local index of the instantiated inventory item represented by this placement.
    pub item_instance_index: i32,
}

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
}

/// Character identity data (name, class, equipment, spells, party).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterIdentity {
    /// Unknown block before player name (96 bytes).
    pub unknown_block: Vec<u8>,
    /// Player name (11-byte WINDOWS-1250 null-terminated).
    pub player_name: String,
    /// Player class ID.
    pub player_class_id: u16,
    /// Player class name (11-byte WINDOWS-1250 null-terminated).
    pub player_class_name: String,
    /// Header block before equipment data (11 bytes).
    pub character_data_header: CharacterDataHeader,
    /// Equipped weapon items — 12 slots × 9 bytes = 108 bytes.
    pub equipped_equipment: Vec<EquipmentSlot>,
    /// Belt item placements — 6 cells × 16 bytes = 96 bytes.
    pub belt_potions: Vec<BeltPotionSlot>,
    /// Inventory item placements — 3 pages × 7 columns × 9 cells × 20 bytes.
    pub inventory_placement: Vec<InventoryPlacementEntry>,
    /// Learned spells — 41 bytes (one flag per spell).
    pub learned_spells: LearnedSpells,
    /// Number of NPCs that accompany the player on their adventures.
    pub party_members_count: u32,
    /// Party members (321 bytes each, with an optional 52-byte combat tail).
    pub party_members: Vec<PartyMember>,
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
///
/// The game writes the 300-byte state as overlapping four-byte reads from its
/// in-memory companion object. The named values below are
/// decoded from the first such snapshots. `serialized_runtime_state` retains
/// the complete original stream, including the repeated overlap bytes, so a
/// read/write round trip cannot discard data that has not yet been decoded.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PartyMember {
    /// Party character display name, stored in a 21-byte Windows-1250 buffer.
    pub name: String,
    /// Maximum health points from the party-level progression record.
    pub maximum_health_points: u16,
    /// Maximum mana points from the party-level progression record.
    pub maximum_mana_points: u16,
    /// Health points remaining at the time the save was written.
    pub current_health_points: u16,
    /// Mana points remaining at the time the save was written.
    pub current_mana_points: u16,
    /// Party character class ID from `PrtIni.db` (21–24 in shipped data).
    pub class_id: u8,
    /// Current progression level.
    pub level: u8,
    /// Class-specific runtime behaviour selected during companion creation.
    pub class_behaviour: u8,
    /// Range used by the companion AI when searching for a combat target.
    pub ai_target_search_range: u8,
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
    /// First magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_1: u8,
    /// Second magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_2: u8,
    /// Third magic spell unlocked at the member's current level.
    /// A value of `0xff` means that the slot is empty.
    pub magic_spell_id_3: u8,
    /// Zero-based index of this companion in the game's party-character table.
    pub party_character_index: u8,
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
    /// Percentage threshold used after level ten to trigger a tactical action.
    pub tactical_action_chance: u32,
    /// Index of the next node in the active movement path.
    pub path_node_index: u32,
    /// Current map-cell X coordinate.
    pub map_x: u16,
    /// Current map-cell Y coordinate.
    pub map_y: u16,
    /// Previous map-cell X coordinate, used by movement and formation logic.
    pub previous_map_x: u16,
    /// Previous map-cell Y coordinate, used by movement and formation logic.
    pub previous_map_y: u16,
    /// Runtime movement state for the companion.
    pub movement_state: u32,
    /// Whether the companion sprite is rendered horizontally flipped.
    pub sprite_horizontal_flip: bool,
    /// Number of nodes in the active movement path.
    pub path_node_count: u32,
    /// Horizontal screen-pixel offset used while drawing the companion sprite.
    pub sprite_offset_x: i8,
    /// Vertical screen-pixel offset used while drawing the companion sprite.
    pub sprite_offset_y: i8,
    /// Current frame within the companion's active animation.
    pub animation_frame_index: u8,
    /// Current facing direction; `-1` means that no directional animation is active.
    pub facing_direction: i8,
    /// Inferred movement-transition state, stored beside the direction bytes.
    pub movement_transition_state: u32,
    /// Inferred substate for the current movement transition.
    pub movement_transition_substate: u32,
    /// Map occupancy ID written into tiles and supplied to pathfinding for this companion.
    pub map_occupancy_id: u8,
    /// Direction index of the sprite's current movement state.
    pub movement_sprite_direction: u32,
    /// Number of animation frames processed in the current action.
    pub animation_tick_count: u32,
    /// Inferred phase value for the active movement animation.
    pub movement_animation_phase: u32,
    /// Map-cell X coordinate the companion is currently following.
    pub follow_target_x: i32,
    /// Map-cell Y coordinate the companion is currently following.
    pub follow_target_y: i32,
    /// Selected combat action or spell. Negative values are runtime sentinels.
    pub selected_combat_action_id: i32,
    /// ID of the map object currently selected as this companion's movement or action target.
    /// A negative value means that no map object is selected.
    pub selected_map_object_id: i16,
    /// Whether the companion's one-frame hit reaction still needs to be drawn.
    pub hit_animation_pending: bool,
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
    /// Auxiliary value for a timed status effect; `-1` means inactive.
    ///
    /// Depending on the effect phase, the game uses this as a countdown or a
    /// party-slot value, so it is not consistently an effect source.
    pub status_effect_auxiliary_value: i32,
    /// Number of attempts made to find a nearby walkable cell when the path is blocked.
    pub blocked_path_reposition_attempts: u32,
    /// X coordinate of the temporary target used by blocked-path recovery.
    pub blocked_path_target_x: i32,
    /// Y coordinate of the temporary target used by blocked-path recovery.
    pub blocked_path_target_y: i32,
    /// Whether the selected combat action is waiting for its execution delay.
    pub combat_action_delay_active: bool,
    /// Ticks remaining before the delayed combat action is executed.
    pub combat_action_delay_ticks_remaining: u32,
    /// Whether a delayed combat action has become ready for execution.
    pub combat_action_ready: bool,
    /// Current visual frame while the combat action waits for execution.
    pub combat_action_delay_animation_frame: u32,
    /// Current visual frame while the ready combat action is resolved.
    pub combat_action_resolution_animation_frame: u32,
    /// Whether the combat action's completion frame has been reached.
    pub combat_action_completion_latched: bool,
    /// Whether the companion is actively recovering from a blocked path.
    pub blocked_path_recovery_active: bool,
    /// Whether the companion has been instructed to rejoin the party leader.
    pub rejoin_leader_requested: bool,
    /// Whether the companion is currently moving to rejoin the party leader.
    pub rejoin_leader_in_progress: bool,
    /// Whether this companion has earned a level and awaits the level-up sequence.
    pub level_up_pending: bool,
    /// Whether the level-up animation is currently active.
    pub level_up_animation_active: bool,
    /// Current frame of the level-up animation.
    pub level_up_animation_frame: u32,
    /// Variant of the level-up animation selected for this companion's class.
    pub level_up_animation_variant: u32,
    /// X coordinate of the first node in the saved active-path buffer.
    pub active_path_node_x: u16,
    /// Y coordinate of the first node in the saved active-path buffer.
    pub active_path_node_y: u16,
    /// Base actor lifecycle state saved alongside the active-path buffer.
    pub base_actor_state: u32,
    /// Current health in the inherited base-actor state.
    pub base_actor_current_health_points: u16,
    /// Maximum health in the inherited base-actor state.
    pub base_actor_maximum_health_points: u16,
    /// Last render-buffer address saved by the runtime. It is not stable across sessions.
    pub last_render_buffer_address: u32,
    /// Last render parameter saved by the runtime. It is not stable gameplay data.
    pub last_render_parameter: i32,
    /// Marker for an optional combat snapshot appended after the base record.
    pub combat_snapshot_marker: u32,
    /// The exact 75-word serialized state stream after the name.
    ///
    /// This is authoritative on write because the game serializes overlapping
    /// windows of its runtime object rather than a conventional packed struct.
    pub serialized_runtime_state: Vec<u8>,
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
        let name = read_null_terminated_windows_1250(&data[..Self::NAME_SIZE])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let serialized_runtime_state = data[Self::NAME_SIZE..].to_vec();
        let state = &serialized_runtime_state;
        let u16_at = |offset| u16::from_le_bytes([state[offset], state[offset + 1]]);
        let u32_at = |offset| {
            u32::from_le_bytes([
                state[offset],
                state[offset + 1],
                state[offset + 2],
                state[offset + 3],
            ])
        };

        Ok(Self {
            name,
            maximum_health_points: u16_at(0),
            maximum_mana_points: u16_at(2),
            current_health_points: u16_at(80),
            current_mana_points: u16_at(84),
            class_id: state[6],
            level: state[7],
            class_behaviour: state[10],
            ai_target_search_range: state[11],
            ai_runtime_state: u32_at(24),
            strength: u32_at(28),
            constitution: u32_at(32),
            wisdom: u32_at(36),
            agility: state[40],
            attack: state[41],
            magic_spell_id_1: state[48],
            magic_spell_id_2: state[49],
            magic_spell_id_3: state[50],
            party_character_index: state[60],
            party_class_variant: u32_at(64),
            weapon_skill_level: u32_at(68),
            experience_points: u32_at(72),
            party_slot_index: u32_at(76),
            path_node_index: u32_at(244),
            map_x: u16_at(248),
            map_y: u16_at(250),
            previous_map_x: u16_at(256),
            previous_map_y: u16_at(258),
            movement_state: u32_at(268),
            sprite_horizontal_flip: state[264] != 0,
            path_node_count: u32_at(272),
            tactical_action_chance: u32_at(276),
            sprite_offset_x: state[88] as i8,
            sprite_offset_y: state[92] as i8,
            animation_frame_index: state[96],
            facing_direction: state[104] as i8,
            movement_transition_state: u32_at(108),
            movement_transition_substate: u32_at(112),
            map_occupancy_id: state[100],
            movement_sprite_direction: u32_at(128),
            animation_tick_count: u32_at(124),
            movement_animation_phase: u32_at(120),
            follow_target_x: u32_at(132) as i32,
            follow_target_y: u32_at(136) as i32,
            selected_combat_action_id: u32_at(164) as i32,
            selected_map_object_id: u16_at(116) as i16,
            hit_animation_pending: state[140] != 0,
            automatic_health_restorations_remaining: u32_at(176),
            automatic_mana_restorations_remaining: u32_at(180),
            active_status_effect_id: u32_at(184),
            status_effect_ticks_remaining: u32_at(188),
            poison_damage_tick_countdown: u32_at(192),
            status_effect_auxiliary_value: u32_at(172) as i32,
            blocked_path_reposition_attempts: u32_at(196),
            blocked_path_target_x: u32_at(200) as i32,
            blocked_path_target_y: u32_at(204) as i32,
            combat_action_delay_active: state[148] != 0,
            combat_action_delay_ticks_remaining: u32_at(152),
            combat_action_ready: state[156] != 0,
            combat_action_delay_animation_frame: u32_at(144),
            combat_action_resolution_animation_frame: u32_at(160),
            combat_action_completion_latched: state[168] != 0,
            blocked_path_recovery_active: state[208] != 0,
            rejoin_leader_requested: state[213] != 0,
            rejoin_leader_in_progress: state[214] != 0,
            level_up_pending: state[220] != 0,
            level_up_animation_active: state[224] != 0,
            level_up_animation_frame: u32_at(228),
            level_up_animation_variant: u32_at(232),
            active_path_node_x: u16_at(280),
            active_path_node_y: u16_at(284),
            base_actor_state: u32_at(288),
            base_actor_current_health_points: u16_at(292),
            base_actor_maximum_health_points: u16_at(294),
            last_render_buffer_address: u32_at(236),
            last_render_parameter: u32_at(240) as i32,
            combat_snapshot_marker: u32_at(296),
            serialized_runtime_state,
            combat_snapshot,
        })
    }

    /// Write the original serialized companion-state stream.
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.serialized_runtime_state.len() != Self::RUNTIME_STATE_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "PartyMember runtime state requires 300 bytes",
            ));
        }

        let mut name_buf = [0u8; Self::NAME_SIZE];
        let (encoded, _, _) = encoding_rs::WINDOWS_1250.encode(&self.name);
        let len = encoded.len().min(Self::NAME_SIZE);
        name_buf[..len].copy_from_slice(&encoded[..len]);
        writer.write_all(&name_buf)?;
        writer.write_all(&self.serialized_runtime_state)?;
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
