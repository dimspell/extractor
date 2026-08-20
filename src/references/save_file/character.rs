use crate::references::extractor::read_null_terminated_windows_1250;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub(super) const SPRITE_PATH_COUNT: usize = 4;
pub(super) const SPRITE_PATH_SIZE: usize = 60;
pub(super) const CHARACTER_DATA_SIZE: usize = 112;
pub(super) const CHARACTER_IDENTITY_SIZE: usize = 35;
pub(super) const LEARNED_SPELL_COUNT: usize = 41;
const WAYPOINT_SIZE: usize = 8;

pub(super) fn read_sprite_paths<R: Read>(reader: &mut R) -> std::io::Result<Vec<String>> {
    let mut paths = Vec::with_capacity(SPRITE_PATH_COUNT);
    for _ in 0..SPRITE_PATH_COUNT {
        let mut buffer = [0u8; SPRITE_PATH_SIZE];
        reader.read_exact(&mut buffer)?;
        paths.push(
            read_null_terminated_windows_1250(&buffer)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        );
    }
    Ok(paths)
}

pub(super) fn write_sprite_paths<W: Write>(
    paths: &[String],
    writer: &mut W,
) -> std::io::Result<()> {
    for path in paths {
        let mut buffer = [0u8; SPRITE_PATH_SIZE];
        let (encoded, _, _) = encoding_rs::WINDOWS_1250.encode(path);
        let len = encoded.len().min(buffer.len());
        buffer[..len].copy_from_slice(&encoded[..len]);
        writer.write_all(&buffer)?;
    }
    Ok(())
}

/// Parse actual position, character stats, and some unknown bytes (112 bytes).
/// The CharacterData bytes are after 4 x 60B blocks (sprites).
///
/// Layout:
///   `[script_event_active u32][armor_display_mode u32]
///    [position_x: i16][position_y: i16]
///    [is_moving u8][tile_value_under_player u8][busy_event_flag u8]
///    [target_tile_value u8][sub_action_state u8][selected_spell_id u8]
///    [attack_anim_frame i16][attack_action_state u8][action_mode u8]
///    [anim_frame_delay_counter u8][anim_frame_delay_threshold u8]
///    [movement_state u8][animation_frame u8][level_up_pending u8]
///    [elapsed_frame_counter i16][hit_buildup_counter i16][selected_action_index i16]
///    [clickable_item_count u8][active_status_effect u8][pending_status_effect u8]
///    [poison_tick_interval i16][poison_tick_counter i16]
///    [strength u16][agility u16][wisdom u16][constitution u16]
///    [morale u16][hp_cur u16][hp_max u16][mp_cur u16][mp_max u16]
///    [xp u32][level u8][unspent_stat_points u8][gold u32][offense u16][defense u16]
///    [dodge u8][hit u8][magic_power u16][attack_mod u8]
///    [thievery u8][lockpick u8][haggle u8][perception u8][traps u8]
///    [sword_lv u8][sword_kills u16][axe_lv u8][axe_kills u16]
///    [archery_lv u8][archery_kills u16][polearm_lv u8][polearm_kills u16]
///    [magic_lv u8][magic_kills u16][holy_lv u8][holy_kills u16]
///    [dark_lv u8][dark_kills u16]
///    [cached_tile_value u16][combat_action_state u8][reserved_05c u16]
///    [reserved_05d u16][ui_hover_active u8][status_effect_stack u8]`
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterData {
    /// Whether a script or event is currently running (dialogue being one
    /// case). While set, normal player input is suspended and the event takes
    /// over. Set to 1 when event script executes.
    pub script_event_active: u32,

    /// Rendering mode for the character's armor by gender. Selects between the
    /// `armor1_sp` and `armor1_w_` sprites when rendering the character.
    pub armor_display_mode: u32,

    pub character_position_x: i16,
    pub character_position_y: i16,

    /// Whether the character is currently moving. Selects the walk vs idle animation.
    pub is_moving: u8,

    /// Ground-tile ID of the tile the player is currently standing on.
    /// Updated as the player moves, so it always reflects the terrain beneath
    /// them. Used by the pathfinder and to carry terrain/footprint state to
    /// the tile the player is moving onto.
    pub tile_value_under_player: u8,

    /// Whether the character is engaged in a non-combat interaction (menu or
    /// event). While set to 1, click and key input is routed to that interaction
    /// instead of normal movement.
    pub busy_event_flag: u8,

    /// Tile value of the tile the character is stepping onto during a move,
    /// used for collision checks. Cleared to 0 once the move completes.
    pub target_tile_value: u8,

    /// Current movement sub-state: 0=idle, 1=start walk-to, 2=walking, 3=action.
    pub sub_action_state: u8,

    pub selected_spell_id: u8,

    /// Current frame of the attack/cast animation, advanced while the attack
    /// or cast action is in progress and capped by the spell's frame count.
    pub attack_anim_frame: i16,

    /// Current attack/cast state: idle, attack started, or actively
    /// attacking/casting.
    pub attack_action_state: u8,

    /// Transient action/interaction-mode selector (0–4) on the character.
    /// Value 1 or 2 picks between two interaction/action handlers, value 4
    /// enables a special mode that widens the pathfinding search radius and
    /// changes how a targeted action is resolved. Derived from a classified
    /// character code during rendering/equipment setup and reset to 0 when the
    /// interaction ends.
    pub action_mode: u8,

    /// Counter that advances the animation's frame delay, resetting once it
    /// reaches the delay threshold.
    pub anim_frame_delay_counter: u8,

    /// Number of frames to wait before the animation advances (default 50).
    pub anim_frame_delay_threshold: u8,

    /// The character's movement mode: 0=idle, 1=running (shift held), 2=walking to
    /// destination, 3=special move.
    pub movement_state: u8,

    /// Current index into the character's animation frames, advanced and
    /// wrapped as the animation cycle progresses.
    pub animation_frame: u8,

    /// Set right after a level-up and cleared once it is handled. Blocks
    /// combat actions while set.
    pub level_up_pending: u8,

    /// Frame counter for held actions. Ticks up each frame while an action is
    /// held; when it reaches the hold timeout it resets and consumes a charge
    /// from `hit_buildup_counter`. Reset whenever the held action ends.
    pub elapsed_frame_counter: i16,

    /// Counts rapid hits or clicks (up to 4), then counts down as a delay
    /// before the next action can be queued.
    pub hit_buildup_counter: i16,

    /// Index of the currently selected action - skill (<500), item (500–699), or spell (700+).
    /// Reset to 0 when deselected.
    pub selected_action_index: i16,

    /// Number of interactive map entities currently available to click
    /// (portals, NPCs, doors, ground items). Kept in sync with the list of
    /// click targets as the world changes; used to resolve which entity a
    /// click landed on.
    pub clickable_item_count: u8,

    /// The status effect currently affecting the character (0=none, 1=poison, 2–8=others.).
    pub active_status_effect: u8,

    /// A status effect queued to be applied next, promoted to the active
    /// status effect when it takes effect.
    pub pending_status_effect: u8,

    /// Number of frames between poison damage ticks (default 25).
    pub poison_tick_interval: i16,

    /// Frames accumulated toward the next poison damage tick; resets once the
    /// damage is applied.
    pub poison_tick_counter: i16,

    // Parsed character stats (core, combat, skills, weapon skills) (63 bytes).
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
    pub level: u8,

    /// Unspent stat/skill points. +10 per level-up, −1 per stat allocation.
    pub unspent_stat_points: u8,

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

    /// Tile value cached by the pathfinder while planning a move.
    pub cached_tile_value: u16,

    /// Current combat action state: 0=idle, 1=attack initiated, 2=targeting.
    pub combat_action_state: u8,

    /// Reserved. Persisted in the save file but not currently used by the
    /// game.
    pub reserved_05c: u16,

    /// Reserved. Persisted in the save file but not currently used by the
    /// game.
    pub reserved_05d: u16,

    /// Whether the cursor is over an inventory or equipment region. Blocks
    /// movement while a UI element is active.
    pub ui_hover_active: u8,

    /// Running count of status-effect applications, reset when the effect is
    /// cured.
    pub status_effect_stack: u8,
}

impl CharacterData {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = [0u8; CHARACTER_DATA_SIZE];
        reader.read_exact(&mut data)?;
        Self::parse(&data)
    }
}

/// Character runtime state block that precedes the identity block in the
/// save file.
///
/// Holds the per-category inventory serial counters, the scripted-action
/// state machine, the movement waypoint path, the movement/teleport state,
/// the class stat bonuses, and the character's map position. The game
/// persists this block so that in-progress movement, teleports, and
/// scripted actions survive a save/load cycle.
///
/// Layout (all integers little-endian):
///   `[5 × inventory_item_serial u16]
///    [current_action_id u32][waypoint_index u32][waypoint_count u32]
///    [waypoint_data: waypoint_count × 8 bytes]
///    [move_requested u8][move_destination_x u32][move_destination_y u32]
///    [movement_blocked u8][teleport_mode u8][teleport_destination_pending u8]
///    [teleport_execution_pending u8][model_animation_index u16]
///    [teleport_target_x u32][teleport_target_y u32][teleport_target_value u32]
///    [stop_after_path_end u8][movement_sub_state u8][character_class u8]
///    [position_changed u8][global_object_id_counter u32]
///    [interaction_state u8][interaction_state_paired u8]
///    [warrior_offense_bonus u8][knight_defense_bonus u8][archer_dodge_rate_bonus u8]
///    [archer_hit_rate_bonus u8][mage_magic_power_bonus u8]
///    [action_index u32][pathfinding_scratch_a u32][pathfinding_scratch_b u32]
///    [reserved_500 u32][action_current_step u32][action_total_steps u32]
///    [position_x u32][position_y u32]`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterState {
    /// Next-slot serial for the Event item category. Each new item in the
    /// category is stamped with this serial, which is then incremented.
    /// Kept in sync with the event-item count read from the inventory
    /// section.
    pub event_items_serial: u16,

    /// Next-slot serial for the Misc item category.
    pub misc_items_serial: u16,

    /// Next-slot serial for the Edit item category.
    pub edit_items_serial: u16,

    /// Next-slot serial for the Weapon item category.
    pub weapon_items_serial: u16,

    /// Next-slot serial for the Heal item category.
    pub heal_items_serial: u16,

    /// Current scripted action (0–7), or −1 when idle. Drives scripted
    /// movement and action sequences.
    pub current_action_id: u32,

    /// Index of the current waypoint in the path being followed.
    pub waypoint_index: u32,

    /// Number of waypoints in the path.
    pub waypoint_count: u32,

    /// Movement waypoint path: `waypoint_count` records of {u16 x, u16 y}
    /// tile coordinates (8 bytes each).
    pub waypoint_data: Vec<u8>,

    /// Pending move latch: set when a move to the destination is requested
    /// and cleared once the move completes.
    pub move_requested: u8,

    /// Destination tile X of the requested move.
    pub move_destination_x: u32,

    /// Destination tile Y of the requested move.
    pub move_destination_y: u32,

    /// Set while movement is blocked (event or cutscene in progress).
    pub movement_blocked: u8,

    /// Teleport mode selector.
    pub teleport_mode: u8,

    /// Set when a teleport destination has been queued.
    pub teleport_destination_pending: u8,

    /// Set when the queued teleport is about to execute.
    pub teleport_execution_pending: u8,

    /// Index into the model animation table used to render the character.
    pub model_animation_index: u16,

    /// Teleport target tile X.
    pub teleport_target_x: u32,

    /// Teleport target tile Y.
    pub teleport_target_y: u32,

    /// Teleport target value.
    pub teleport_target_value: u32,

    /// Set to stop the character once the waypoint path is exhausted.
    pub stop_after_path_end: u8,

    /// Movement sub-state: 0=idle, 1=started, 2=walking, 3=arrived.
    pub movement_sub_state: u8,

    /// Character class: 1=Paladin, 2=Hero. Derived from the morale
    /// (good/evil path), the class, and the level.
    pub character_class: u8,

    /// Latch set when the position changed; used to sync the position to
    /// the character's map coordinates.
    pub position_changed: u8,

    /// Monotonic counter of spawned world objects.
    pub global_object_id_counter: u32,

    /// Current interaction state.
    pub interaction_state: u8,

    /// Paired interaction state.
    pub interaction_state_paired: u8,

    /// Offense stat bonus applied to the Warrior class.
    pub warrior_offense_bonus: u8,

    /// Defense stat bonus applied to the Knight class.
    pub knight_defense_bonus: u8,

    /// Dodge-rate stat bonus applied to the Archer class.
    pub archer_dodge_rate_bonus: u8,

    /// Hit-rate stat bonus applied to the Archer class.
    pub archer_hit_rate_bonus: u8,

    /// Magic-power stat bonus applied to the Mage class.
    pub mage_magic_power_bonus: u8,

    /// Index of the current action.
    pub action_index: u32,

    /// Transient pathfinding scratch value.
    pub pathfinding_scratch_a: u32,

    /// Transient pathfinding scratch value.
    pub pathfinding_scratch_b: u32,

    /// Reserved. Persisted in the save file but not currently used by the
    /// game.
    pub reserved_500: u32,

    /// Current step of the action in progress.
    pub action_current_step: u32,

    /// Total steps of the action in progress.
    pub action_total_steps: u32,

    /// Character map position (tile X).
    pub position_x: u32,

    /// Character map position (tile Y).
    pub position_y: u32,
}

impl CharacterState {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let event_items_serial = reader.read_u16::<LittleEndian>()?;
        let misc_items_serial = reader.read_u16::<LittleEndian>()?;
        let edit_items_serial = reader.read_u16::<LittleEndian>()?;
        let weapon_items_serial = reader.read_u16::<LittleEndian>()?;
        let heal_items_serial = reader.read_u16::<LittleEndian>()?;

        let current_action_id = reader.read_u32::<LittleEndian>()?;
        let waypoint_index = reader.read_u32::<LittleEndian>()?;
        let waypoint_count = reader.read_u32::<LittleEndian>()?;

        let mut waypoint_data = vec![0u8; waypoint_count as usize * WAYPOINT_SIZE];
        reader.read_exact(&mut waypoint_data)?;

        let move_requested = reader.read_u8()?;
        let move_destination_x = reader.read_u32::<LittleEndian>()?;
        let move_destination_y = reader.read_u32::<LittleEndian>()?;
        let movement_blocked = reader.read_u8()?;
        let teleport_mode = reader.read_u8()?;
        let teleport_destination_pending = reader.read_u8()?;
        let teleport_execution_pending = reader.read_u8()?;
        let model_animation_index = reader.read_u16::<LittleEndian>()?;
        let teleport_target_x = reader.read_u32::<LittleEndian>()?;
        let teleport_target_y = reader.read_u32::<LittleEndian>()?;
        let teleport_target_value = reader.read_u32::<LittleEndian>()?;
        let stop_after_path_end = reader.read_u8()?;
        let movement_sub_state = reader.read_u8()?;
        let character_class = reader.read_u8()?;
        let position_changed = reader.read_u8()?;
        let global_object_id_counter = reader.read_u32::<LittleEndian>()?;
        let interaction_state = reader.read_u8()?;
        let interaction_state_paired = reader.read_u8()?;
        let warrior_offense_bonus = reader.read_u8()?;
        let knight_defense_bonus = reader.read_u8()?;
        let archer_dodge_rate_bonus = reader.read_u8()?;
        let archer_hit_rate_bonus = reader.read_u8()?;
        let mage_magic_power_bonus = reader.read_u8()?;
        let action_index = reader.read_u32::<LittleEndian>()?;
        let pathfinding_scratch_a = reader.read_u32::<LittleEndian>()?;
        let pathfinding_scratch_b = reader.read_u32::<LittleEndian>()?;
        let reserved_500 = reader.read_u32::<LittleEndian>()?;
        let action_current_step = reader.read_u32::<LittleEndian>()?;
        let action_total_steps = reader.read_u32::<LittleEndian>()?;
        let position_x = reader.read_u32::<LittleEndian>()?;
        let position_y = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            event_items_serial,
            misc_items_serial,
            edit_items_serial,
            weapon_items_serial,
            heal_items_serial,
            current_action_id,
            waypoint_index,
            waypoint_count,
            waypoint_data,
            move_requested,
            move_destination_x,
            move_destination_y,
            movement_blocked,
            teleport_mode,
            teleport_destination_pending,
            teleport_execution_pending,
            model_animation_index,
            teleport_target_x,
            teleport_target_y,
            teleport_target_value,
            stop_after_path_end,
            movement_sub_state,
            character_class,
            position_changed,
            global_object_id_counter,
            interaction_state,
            interaction_state_paired,
            warrior_offense_bonus,
            knight_defense_bonus,
            archer_dodge_rate_bonus,
            archer_hit_rate_bonus,
            mage_magic_power_bonus,
            action_index,
            pathfinding_scratch_a,
            pathfinding_scratch_b,
            reserved_500,
            action_current_step,
            action_total_steps,
            position_x,
            position_y,
        })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(self.event_items_serial)?;
        writer.write_u16::<LittleEndian>(self.misc_items_serial)?;
        writer.write_u16::<LittleEndian>(self.edit_items_serial)?;
        writer.write_u16::<LittleEndian>(self.weapon_items_serial)?;
        writer.write_u16::<LittleEndian>(self.heal_items_serial)?;

        writer.write_u32::<LittleEndian>(self.current_action_id)?;
        writer.write_u32::<LittleEndian>(self.waypoint_index)?;
        writer.write_u32::<LittleEndian>(self.waypoint_count)?;
        writer.write_all(&self.waypoint_data)?;

        writer.write_u8(self.move_requested)?;
        writer.write_u32::<LittleEndian>(self.move_destination_x)?;
        writer.write_u32::<LittleEndian>(self.move_destination_y)?;
        writer.write_u8(self.movement_blocked)?;
        writer.write_u8(self.teleport_mode)?;
        writer.write_u8(self.teleport_destination_pending)?;
        writer.write_u8(self.teleport_execution_pending)?;
        writer.write_u16::<LittleEndian>(self.model_animation_index)?;
        writer.write_u32::<LittleEndian>(self.teleport_target_x)?;
        writer.write_u32::<LittleEndian>(self.teleport_target_y)?;
        writer.write_u32::<LittleEndian>(self.teleport_target_value)?;
        writer.write_u8(self.stop_after_path_end)?;
        writer.write_u8(self.movement_sub_state)?;
        writer.write_u8(self.character_class)?;
        writer.write_u8(self.position_changed)?;
        writer.write_u32::<LittleEndian>(self.global_object_id_counter)?;
        writer.write_u8(self.interaction_state)?;
        writer.write_u8(self.interaction_state_paired)?;
        writer.write_u8(self.warrior_offense_bonus)?;
        writer.write_u8(self.knight_defense_bonus)?;
        writer.write_u8(self.archer_dodge_rate_bonus)?;
        writer.write_u8(self.archer_hit_rate_bonus)?;
        writer.write_u8(self.mage_magic_power_bonus)?;
        writer.write_u32::<LittleEndian>(self.action_index)?;
        writer.write_u32::<LittleEndian>(self.pathfinding_scratch_a)?;
        writer.write_u32::<LittleEndian>(self.pathfinding_scratch_b)?;
        writer.write_u32::<LittleEndian>(self.reserved_500)?;
        writer.write_u32::<LittleEndian>(self.action_current_step)?;
        writer.write_u32::<LittleEndian>(self.action_total_steps)?;
        writer.write_u32::<LittleEndian>(self.position_x)?;
        writer.write_u32::<LittleEndian>(self.position_y)?;

        Ok(())
    }
}

/// Character identity data (name, class, and persisted spell-bar state).
///
/// This is the trailing 35 bytes of the identity section. The preceding
/// runtime-state block is parsed separately as [`CharacterState`].
#[derive(Debug, Clone, Serialize, Deserialize, Default, BinaryRecord)]
pub struct CharacterIdentity {
    /// Player name (11 bytes, null-terminated string).
    #[binary_record(string(encoding = "WINDOWS-1250", size = 11))]
    pub player_name: String,
    /// Player class ID.
    pub player_class_id: u16,
    /// Player class name (20-byte WINDOWS-1250 null-terminated).
    #[binary_record(string(encoding = "WINDOWS-1250", size = 20))]
    pub player_class_name: String,
    /// UI index of the currently selected spell, derived from the spell ID
    /// and used by the renderer to position the spell bar.
    pub selected_spell_ui_index: u16,
}

impl CharacterIdentity {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut identity = [0u8; CHARACTER_IDENTITY_SIZE];
        reader.read_exact(&mut identity)?;
        Self::parse(&identity)
    }
}

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
}

impl LearnedSpells {
    pub(super) fn read_from<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut spells = vec![0u8; LEARNED_SPELL_COUNT];
        reader.read_exact(&mut spells)?;
        Ok(Self { spells })
    }

    pub(super) fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.spells)
    }
}
