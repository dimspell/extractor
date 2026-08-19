use dispel_macros::BinaryRecord;
use serde::{Deserialize, Serialize};

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

/// Character identity data (name, class, and persisted spell-bar state).
///
/// Only the trailing 35 bytes of the identity block are retained here
/// (`player_name` through `selected_spell_ui_index`)
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

/// Learned spells block (41 bytes).
///
/// One byte per spell, likely boolean flags indicating whether each
/// spell has been learned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedSpells {
    pub spells: Vec<u8>,
}
